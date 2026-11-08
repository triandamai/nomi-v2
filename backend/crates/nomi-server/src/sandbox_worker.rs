//! The background loop that keeps sandboxes where their projects want them: it claims the rows
//! that are due, reconciles each one, and stops the sandboxes whose page has gone quiet. It only
//! does anything while the admin switch is on. With it off, projects stay on WebContainer and the
//! worker only winds down what is already running (see `PassMode`).

use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use std::future::Future;
use std::panic::AssertUnwindSafe;

use futures_util::stream::{self, StreamExt};
use futures_util::FutureExt;
use nomi_sandbox::{BoatConfig, BoatProvider, SandboxId};
use nomi_storage::ProjectStore;
use sqlx::PgPool;
use uuid::Uuid;

use crate::configuration::{sandbox_config, sandbox_credentials, ReaperMode, SandboxConfig};
use crate::routes::project_sandbox::{availability, RUNTIME_SANDBOX};
use crate::sandbox_cleanup::{self, REAP_INTERVAL};
use crate::sandbox_manager::{ReconcileOutcome, SandboxManager};
use crate::sandbox_runs::RUN_LEASE;
use crate::sandbox_steps::ServerSteps;
use crate::sandbox_usage::DRIFT_INTERVAL;
use crate::sandbox_store as store;

/// The provider name stored on a project's sandbox row.
pub const PROVIDER_NAME: &str = "boat";
/// How long the loop sleeps between looks for due rows.
const POLL_INTERVAL: Duration = Duration::from_secs(2);
/// How long the loop sleeps while the admin switch is off and there is nothing to wind down.
const OFF_POLL_INTERVAL: Duration = Duration::from_secs(15);
/// How often idle sandboxes are looked for.
const SWEEP_INTERVAL: Duration = Duration::from_secs(30);
/// How often long-stopped sandboxes are looked for.
const EXPIRY_INTERVAL: Duration = Duration::from_secs(24 * 60 * 60);
/// Most rows claimed in one pass.
const BATCH_SIZE: i64 = 10;
/// Most rows reconciled at the same time.
const CONCURRENCY: usize = 4;

/// Run one piece of the worker's work, catching a panic in it: the panic is logged as `what`
/// and `None` comes back, so one bad row or pass never ends the loop that every sandbox depends
/// on. A walk that panicked keeps its lease until it runs out, then is claimed again.
pub async fn guarded<F: Future>(what: &'static str, work: F) -> Option<F::Output> {
    match AssertUnwindSafe(work).catch_unwind().await {
        Ok(out) => Some(out),
        Err(panic) => {
            let message = panic.downcast_ref::<&str>().map(|s| s.to_string()).or_else(|| panic.downcast_ref::<String>().cloned()).unwrap_or_default();
            tracing::error!(what, panic = %message, "sandbox worker: a task panicked; carrying on");
            None
        }
    }
}

/// What a pass of the loop may do, from the admin switch and the saved credentials.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PassMode {
    /// The switch is on: everything.
    On,
    /// The switch is off but a key is saved: stop what is running and delete what is queued for
    /// deletion, so nothing keeps billing. Never create or resume.
    StopOnly,
}

/// The mode for this pass and the config to build the provider from. `enabled` is
/// `configuration::sandbox_config()`, `credentials` is `configuration::sandbox_credentials()`.
/// With the switch off and no key there is nothing to reach the provider with, so: nothing.
pub fn pass_mode(enabled: Option<SandboxConfig>, credentials: Option<SandboxConfig>) -> Option<(PassMode, SandboxConfig)> {
    match (enabled, credentials) {
        (Some(cfg), _) => Some((PassMode::On, cfg)),
        (None, Some(cfg)) => Some((PassMode::StopOnly, cfg)),
        (None, None) => None,
    }
}

/// The page checking in. `page_runtime` is what the page says it runs: only the sandbox page
/// (`SandboxRuntime`) sends `runtime=sandbox`; the browser runtime (WebContainer, also the
/// fallback and the person's "run in my browser" choice) sends nothing and gets `None`, so the
/// caller does the WebContainer check-in exactly as before and the page claims Koda's commands.
///
/// For the sandbox page this returns the files version and never claims commands (they run in the
/// sandbox). It asks for the sandbox to be running only while the sandbox may be used
/// (`routes::project_sandbox::availability`: switch on, rollout, owner's allowance, no cooldown);
/// otherwise it starts nothing and doesn't keep a running sandbox alive, which then idles out.
/// It never marks the browser runner online, so Koda can tell the two pages apart.
pub async fn sandbox_check_in(
    conn: &mut sqlx::PgConnection,
    project_id: Uuid,
    user_id: Uuid,
    page_runtime: Option<&str>,
    config: Option<&SandboxConfig>,
) -> Result<Option<i64>, sqlx::Error> {
    if page_runtime != Some(RUNTIME_SANDBOX) {
        return Ok(None);
    }
    let found = availability(conn, project_id, Some(user_id), config).await?;
    if found.is_some_and(|a| a.runtime == RUNTIME_SANDBOX && a.available) {
        store::want_running(conn, project_id, PROVIDER_NAME).await?;
    }
    Ok(Some(nomi_agent_coding::runtime::files_version(conn, project_id).await?))
}

pub struct SandboxWorker {
    pool: PgPool,
    manager: Arc<SandboxManager>,
    idle_stop_secs: u64,
    /// Whether the admin switch is off, so claims take only rows that want to stop.
    stop_only: std::sync::atomic::AtomicBool,
    /// What the orphan sweep does with sandboxes no project owns.
    reaper_mode: ReaperMode,
    /// Runs Koda's commands in the sandboxes; without it the worker doesn't touch the run queue.
    runner: Option<Arc<ServerSteps>>,
    /// Projects whose commands are being run right now, so a project never runs two at once.
    running_now: Arc<Mutex<HashSet<Uuid>>>,
}

/// Removes a project from the set of running ones when its task ends, however it ends.
struct RunningGuard {
    set: Arc<Mutex<HashSet<Uuid>>>,
    project_id: Uuid,
}

impl Drop for RunningGuard {
    fn drop(&mut self) {
        self.set.lock().unwrap_or_else(|e| e.into_inner()).remove(&self.project_id);
    }
}

impl SandboxWorker {
    pub fn new(pool: PgPool, manager: Arc<SandboxManager>, idle_stop_secs: u64) -> Self {
        Self { pool, manager, idle_stop_secs, stop_only: Default::default(), reaper_mode: ReaperMode::default(), runner: None, running_now: Default::default() }
    }

    /// Set what the orphan sweep does (it only logs by default).
    pub fn with_reaper_mode(mut self, mode: ReaperMode) -> Self {
        self.reaper_mode = mode;
        self
    }

    /// Set the mode for the next claims: stop-only means nothing is created or resumed.
    pub fn set_mode(&self, mode: PassMode) {
        let stop_only = mode == PassMode::StopOnly;
        self.stop_only.store(stop_only, std::sync::atomic::Ordering::SeqCst);
        self.manager.set_stop_only(stop_only);
    }

    /// Delete the sandboxes of removed projects at the provider (see `sandbox_cleanup`).
    pub async fn drain_deletions(&self) -> usize {
        sandbox_cleanup::drain_deletions(&self.pool, self.manager.provider().as_ref()).await
    }

    /// Find (and, unless dry-run, delete) provider sandboxes no project owns.
    pub async fn reap_orphans(&self) -> usize {
        sandbox_cleanup::reap_orphans(&self.pool, self.manager.provider().as_ref(), self.reaper_mode).await
    }

    /// Also run the commands Koda queues for sandbox projects, through `steps`.
    pub fn with_runs(mut self, steps: Arc<ServerSteps>) -> Self {
        self.runner = Some(steps);
        self
    }

    /// Start running the waiting commands of every sandbox project that is up and has some, at
    /// most `CONCURRENCY` projects at once. A project open in the browser runtime is left to that
    /// page, which claims its commands (see `sandbox_runs::claim_next`). Each project runs in its own task, so a long command
    /// never holds up reconciling. Returns how many projects were started.
    pub async fn dispatch_runs(&self) -> usize {
        let Some(runner) = &self.runner else { return 0 };
        let due: Result<Vec<(Uuid, String)>, sqlx::Error> = sqlx::query_as(
            "SELECT s.project_id, s.sandbox_id FROM project_sandboxes s JOIN projects p ON p.id = s.project_id \
             WHERE s.state = 'running' AND s.sandbox_id IS NOT NULL AND p.runtime = 'sandbox' \
             AND s.synced_version >= p.files_version AND (s.leased_until IS NULL OR s.leased_until <= now()) \
             AND NOT COALESCE(p.runner_seen_at > now() - make_interval(secs => $2), false) AND EXISTS ( \
                 SELECT 1 FROM project_runs r WHERE r.project_id = s.project_id AND (r.status = 'pending' \
                 OR (r.status = 'running' AND r.started_at < now() - make_interval(secs => $1)))) \
             ORDER BY s.project_id",
        )
        .bind(RUN_LEASE.as_secs() as f64)
        .bind(crate::sandbox_runs::BROWSER_ONLINE_SECS)
        .fetch_all(&self.pool)
        .await;
        let due = match due {
            Ok(due) => due,
            Err(e) => {
                tracing::warn!(error = %e, "sandbox worker: looking for waiting commands failed");
                return 0;
            }
        };
        let mut started = 0;
        for (project_id, sandbox_id) in due {
            {
                let mut now = self.running_now.lock().unwrap_or_else(|e| e.into_inner());
                if now.len() >= CONCURRENCY {
                    break;
                }
                if !now.insert(project_id) {
                    continue;
                }
            }
            let guard = RunningGuard { set: self.running_now.clone(), project_id };
            let runner = runner.clone();
            tokio::spawn(async move {
                let _guard = guard;
                runner.execute_pending(project_id, &SandboxId(sandbox_id)).await;
            });
            started += 1;
        }
        started
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
                match guarded("reconcile", self.manager.reconcile(row)).await {
                    Some(ReconcileOutcome::Settled) => {}
                    Some(ReconcileOutcome::RetryAt(at)) => tracing::debug!(%project_id, %at, "sandbox: will retry"),
                    Some(ReconcileOutcome::Failed(why)) => tracing::warn!(%project_id, why = %self.manager.redact(&why), "sandbox: failed"),
                    None => tracing::error!(%project_id, "sandbox: reconciling panicked"),
                }
            })
            .await;
        count
    }

    async fn claim(&self) -> Result<Vec<store::SandboxRow>, sqlx::Error> {
        let mut tx = self.pool.begin().await?;
        let rows = store::claim_due_in(&mut tx, BATCH_SIZE, self.stop_only.load(std::sync::atomic::Ordering::SeqCst)).await?;
        tx.commit().await?;
        Ok(rows)
    }

    /// Compare our metered time with the provider's and log the differences.
    pub async fn check_usage_drift(&self) {
        crate::sandbox_usage::reconcile_usage_drift(&self.pool, self.manager.provider().as_ref()).await;
    }

    /// Ask the provider whether running sandboxes are still up (see `SandboxManager::check_alive`).
    pub async fn check_alive(&self) -> usize {
        self.manager.check_alive().await
    }

    /// Hand back rows stuck mid-change (see `SandboxManager::unstick`).
    pub async fn run_watchdog(&self) -> usize {
        self.manager.unstick().await
    }

    /// Delete long-stopped sandboxes at the provider (see `SandboxManager::expire_stopped`).
    pub async fn expire_old(&self) -> usize {
        self.manager.expire_stopped().await
    }

    /// The switch is off: ask every sandbox wanted running to stop, so `tick` winds them down.
    /// Returns how many rows changed.
    pub async fn wind_down(&self) -> usize {
        let result = async {
            let mut conn = self.pool.acquire().await?;
            store::want_all_stopped(&mut conn).await
        }
        .await;
        match result {
            Ok(n) => n as usize,
            Err(e) => {
                tracing::warn!(error = %e, "sandbox worker: winding down failed");
                0
            }
        }
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
type Signature = (String, String, String, u64, u32, u32, ReaperMode);

fn signature(cfg: &SandboxConfig) -> Signature {
    (cfg.api_base.clone(), cfg.api_key.clone(), format!("{:?}", cfg.size), cfg.idle_stop_secs, cfg.starts_per_minute, cfg.starts_per_hour, cfg.reaper_mode)
}

/// Run until the process ends (like the other background workers). The admin switch is read on
/// every pass, so turning it off stops new work without a restart.
pub async fn run(pool: PgPool, http_client: reqwest::Client, project_storage: ProjectStore, settings_key: [u8; 32], shipyard: Option<nomi_shipyard::ShipyardConfig>) {
    let mut current: Option<(Signature, SandboxWorker)> = None;
    let mut last_sweep = Instant::now();
    // The first check waits a full interval, so a restart doesn't repeat it.
    let mut last_drift_check = Instant::now();
    // Expiry runs on the first pass after a start, then once a day.
    let mut last_expiry: Option<Instant> = None;
    // The orphan sweep runs on the first pass after a start, then once an hour.
    let mut last_reap: Option<Instant> = None;
    loop {
        let enabled = sandbox_config().await;
        let credentials = if enabled.is_some() { None } else { sandbox_credentials().await };
        let Some((mode, cfg)) = pass_mode(enabled, credentials) else {
            current = None;
            tokio::time::sleep(OFF_POLL_INTERVAL).await;
            continue;
        };
        let sig = signature(&cfg);
        if current.as_ref().map(|(s, _)| s) != Some(&sig) {
            let provider = Arc::new(BoatProvider::new(BoatConfig { base_url: cfg.api_base.clone(), api_key: cfg.api_key.clone() }, http_client.clone()));
            let steps = Arc::new(ServerSteps::new(pool.clone(), project_storage.clone(), provider.clone()).with_relay_access(Some(settings_key), http_client.clone()).with_shipyard(shipyard.clone()));
            let manager = Arc::new(SandboxManager::new(provider, steps.clone(), pool.clone(), cfg.clone()));
            current = Some((sig, SandboxWorker::new(pool.clone(), manager, cfg.idle_stop_secs).with_runs(steps).with_reaper_mode(cfg.reaper_mode)));
        }
        let mut busy = true;
        if let Some((_, worker)) = &current {
            // Every pass is guarded: a panic in one is logged and the loop goes on.
            worker.set_mode(mode);
            if mode == PassMode::StopOnly {
                busy = guarded("wind down", worker.wind_down()).await.unwrap_or(0) > 0;
            }
            busy |= guarded("tick", worker.tick()).await.unwrap_or(0) > 0;
            if mode == PassMode::On {
                guarded("dispatch runs", worker.dispatch_runs()).await;
            }
            guarded("drain deletions", worker.drain_deletions()).await;
            if last_sweep.elapsed() >= SWEEP_INTERVAL {
                guarded("idle sweep", worker.sweep_idle()).await;
                guarded("watchdog", worker.run_watchdog()).await;
                if mode == PassMode::On {
                    guarded("liveness", worker.check_alive()).await;
                }
                last_sweep = Instant::now();
            }
            if last_expiry.is_none_or(|t| t.elapsed() >= EXPIRY_INTERVAL) {
                guarded("expiry", worker.expire_old()).await;
                last_expiry = Some(Instant::now());
            }
            if mode == PassMode::On {
                if last_reap.is_none_or(|t| t.elapsed() >= REAP_INTERVAL) {
                    guarded("orphan sweep", worker.reap_orphans()).await;
                    last_reap = Some(Instant::now());
                }
                if last_drift_check.elapsed() >= DRIFT_INTERVAL {
                    guarded("usage drift", worker.check_usage_drift()).await;
                    last_drift_check = Instant::now();
                }
            }
        }
        if mode == PassMode::StopOnly && !busy {
            tokio::time::sleep(OFF_POLL_INTERVAL).await;
            continue;
        }
        tokio::time::sleep(POLL_INTERVAL).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn a_panicking_task_is_caught_and_the_caller_carries_on() {
        let caught = guarded("test", async { if true { panic!("one bad row") } }).await;
        assert_eq!(caught, None::<()>);
        assert_eq!(guarded("test", async { 7 }).await, Some(7));
    }
}
