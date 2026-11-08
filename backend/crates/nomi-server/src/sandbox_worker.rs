//! The background loop that keeps sandboxes where their projects want them: it claims the rows
//! that are due, reconciles each one, and stops the sandboxes whose page has gone quiet. It only
//! does anything while the admin switch is on; with it off, projects stay on WebContainer.

use std::sync::Arc;
use std::time::{Duration, Instant};

use futures_util::stream::{self, StreamExt};
use nomi_sandbox::{BoatConfig, BoatProvider};
use nomi_storage::ProjectStore;
use sqlx::PgPool;
use uuid::Uuid;

use crate::configuration::{sandbox_config, SandboxConfig};
use crate::sandbox_manager::{ReconcileOutcome, SandboxManager};
use crate::sandbox_steps::ServerSteps;
use crate::sandbox_store as store;

/// The provider name stored on a project's sandbox row.
pub const PROVIDER_NAME: &str = "boat";
/// How long the loop sleeps between looks for due rows.
const POLL_INTERVAL: Duration = Duration::from_secs(2);
/// How long the loop sleeps while the admin switch is off.
const OFF_POLL_INTERVAL: Duration = Duration::from_secs(15);
/// How often idle sandboxes are looked for.
const SWEEP_INTERVAL: Duration = Duration::from_secs(30);
/// Most rows claimed in one pass.
const BATCH_SIZE: i64 = 10;
/// Most rows reconciled at the same time.
const CONCURRENCY: usize = 4;

/// The page checking in for a project that runs in a sandbox: marks the page online, asks for the
/// sandbox to be running and returns the files version. Returns `None` for a project that runs in
/// the browser, or when the sandbox runtime is off, so the caller uses the WebContainer check-in.
/// A sandbox project never claims runs: Koda's commands go to the sandbox, not to the page.
pub async fn sandbox_check_in(conn: &mut sqlx::PgConnection, project_id: Uuid, sandbox_enabled: bool) -> Result<Option<i64>, sqlx::Error> {
    if !sandbox_enabled {
        return Ok(None);
    }
    let runtime: Option<String> = sqlx::query_scalar("SELECT runtime FROM projects WHERE id = $1").bind(project_id).fetch_optional(&mut *conn).await?;
    if runtime.as_deref() != Some("sandbox") {
        return Ok(None);
    }
    nomi_agent_coding::runtime::touch_runner(conn, project_id).await?;
    store::want_running(conn, project_id, PROVIDER_NAME).await?;
    Ok(Some(nomi_agent_coding::runtime::files_version(conn, project_id).await?))
}

pub struct SandboxWorker {
    pool: PgPool,
    manager: Arc<SandboxManager>,
    idle_stop_secs: u64,
}

impl SandboxWorker {
    pub fn new(pool: PgPool, manager: Arc<SandboxManager>, idle_stop_secs: u64) -> Self {
        Self { pool, manager, idle_stop_secs }
    }

    /// Claim the rows that are due and reconcile them, a few at a time. Returns how many were
    /// reconciled. The claim is committed before any work starts: it leases the rows, and the
    /// reconciler writes through the pool, which would wait on the claim's locks.
    pub async fn tick(&self) -> usize {
        let rows = match self.claim().await {
            Ok(rows) => rows,
            Err(e) => {
                tracing::warn!(error = %e, "sandbox worker: claiming due rows failed");
                return 0;
            }
        };
        let count = rows.len();
        stream::iter(rows)
            .for_each_concurrent(CONCURRENCY, |row| async move {
                let project_id = row.project_id;
                match self.manager.reconcile(row).await {
                    ReconcileOutcome::Settled => {}
                    ReconcileOutcome::RetryAt(at) => tracing::debug!(%project_id, %at, "sandbox: will retry"),
                    ReconcileOutcome::Failed(why) => tracing::warn!(%project_id, %why, "sandbox: failed"),
                }
            })
            .await;
        count
    }

    async fn claim(&self) -> Result<Vec<store::SandboxRow>, sqlx::Error> {
        let mut tx = self.pool.begin().await?;
        let rows = store::claim_due(&mut tx, BATCH_SIZE).await?;
        tx.commit().await?;
        Ok(rows)
    }

    /// Ask for the sandboxes of projects whose page has gone quiet to be stopped.
    pub async fn sweep_idle(&self) {
        let result = async {
            let mut conn = self.pool.acquire().await?;
            let idle = store::idle_candidates(&mut conn, self.idle_stop_secs).await?;
            for project_id in idle {
                store::want_stopped(&mut conn, project_id).await?;
            }
            Ok::<(), sqlx::Error>(())
        }
        .await;
        if let Err(e) = result {
            tracing::warn!(error = %e, "sandbox worker: idle sweep failed");
        }
    }
}

/// What makes two configs need different managers (the key is compared, never logged).
type Signature = (String, String, String, u64, u32, u32);

fn signature(cfg: &SandboxConfig) -> Signature {
    (cfg.api_base.clone(), cfg.api_key.clone(), format!("{:?}", cfg.size), cfg.idle_stop_secs, cfg.starts_per_minute, cfg.starts_per_hour)
}

/// Run until the process ends (like the other background workers). The admin switch is read on
/// every pass, so turning it off stops new work without a restart.
pub async fn run(pool: PgPool, http_client: reqwest::Client, project_storage: ProjectStore, settings_key: [u8; 32], shipyard: Option<nomi_shipyard::ShipyardConfig>) {
    let mut current: Option<(Signature, SandboxWorker)> = None;
    let mut last_sweep = Instant::now();
    loop {
        let Some(cfg) = sandbox_config().await else {
            current = None;
            tokio::time::sleep(OFF_POLL_INTERVAL).await;
            continue;
        };
        let sig = signature(&cfg);
        if current.as_ref().map(|(s, _)| s) != Some(&sig) {
            let provider = Arc::new(BoatProvider::new(BoatConfig { base_url: cfg.api_base.clone(), api_key: cfg.api_key.clone() }, http_client.clone()));
            let steps = Arc::new(ServerSteps::new(pool.clone(), project_storage.clone(), provider.clone()).with_relay_access(Some(settings_key), http_client.clone()).with_shipyard(shipyard.clone()));
            let manager = Arc::new(SandboxManager::new(provider, steps, pool.clone(), cfg.clone()));
            current = Some((sig, SandboxWorker::new(pool.clone(), manager, cfg.idle_stop_secs)));
        }
        if let Some((_, worker)) = &current {
            worker.tick().await;
            if last_sweep.elapsed() >= SWEEP_INTERVAL {
                worker.sweep_idle().await;
                last_sweep = Instant::now();
            }
        }
        tokio::time::sleep(POLL_INTERVAL).await;
    }
}
