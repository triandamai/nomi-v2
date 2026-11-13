//! What keeps the sandbox runtime steady when the provider misbehaves: the watchdog for rows
//! stuck mid-change, the cooldown after repeated failures, backoff during an outage, and the
//! expiry of sandboxes that have sat stopped too long.

mod support;

use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use chrono::{DateTime, Duration as Span, Utc};
use nomi_sandbox::{FakeProvider, PreviewUrl, SandboxError, SandboxId};
use nomi_server::configuration::{Rollout, SandboxConfig};
use nomi_server::sandbox_manager::{ProjectSteps, ReconcileOutcome, SandboxManager, EXPIRY_WARN, MAX_ATTEMPTS, PROVIDER_COOLDOWN, STUCK_AFTER};
use nomi_server::sandbox_store::{self as store, Desired, State};
use nomi_server::sandbox_worker::SandboxWorker;
use sqlx::PgPool;
use support::App;
use uuid::Uuid;

/// The provider-side auto-stop used in these tests, in seconds.
const AUTO_STOP_SECS: u64 = 6 * 60 * 60;

const PROVIDER: &str = "boat";
/// The files version the steps report after a sync.
const SYNCED_VERSION: i64 = 3;
/// Start limits that never get in the way.
const GENEROUS_LIMIT: u32 = 1000;
/// Idle stop used in the test config, in seconds.
const IDLE_STOP_SECS: u64 = 600;
/// Ticks driven during the outage test.
const OUTAGE_TICKS: usize = 6;
/// Slack added past a limit when ageing a row beyond it, in seconds.
const PAST_LIMIT_SECS: i64 = 60;

#[derive(Default)]
struct Steps {
    calls: Mutex<Vec<String>>,
}

#[async_trait]
impl ProjectSteps for Steps {
    async fn sync(&self, _p: Uuid, sandbox: &SandboxId) -> Result<i64, String> {
        self.calls.lock().unwrap().push(format!("sync {}", sandbox.0));
        Ok(SYNCED_VERSION)
    }
    async fn start_preview(&self, _p: Uuid, _s: &SandboxId) -> Result<PreviewUrl, nomi_server::sandbox_manager::StepError> {
        Ok(PreviewUrl { url: "https://preview.example.test".into() })
    }
}

fn config() -> SandboxConfig {
    SandboxConfig {
        api_base: String::new(),
        api_key: String::new(),
        size: nomi_sandbox::Size::Small,
        idle_stop_secs: IDLE_STOP_SECS,
        auto_stop_secs: AUTO_STOP_SECS,
        rollout: Rollout::All,
        starts_per_minute: GENEROUS_LIMIT,
        starts_per_hour: GENEROUS_LIMIT,
        monthly_minutes_per_user: GENEROUS_LIMIT,
        reaper_mode: nomi_server::configuration::ReaperMode::DryRun,
    }
}

struct Fixture {
    pool: PgPool,
    provider: Arc<FakeProvider>,
    steps: Arc<Steps>,
    manager: Arc<SandboxManager>,
    project: Uuid,
}

async fn fixture(pool: PgPool) -> Fixture {
    let app = App::new(pool.clone());
    let owner = app.person("owner@resilience.test").await;
    let project = app.project(&owner, "resilience").await.0;
    let provider = Arc::new(FakeProvider::new());
    let steps = Arc::new(Steps::default());
    let manager = Arc::new(SandboxManager::new(provider.clone(), steps.clone(), pool.clone(), config()));
    Fixture { pool, provider, steps, manager, project }
}

impl Fixture {
    async fn row(&self) -> store::SandboxRow {
        let mut conn = self.pool.acquire().await.unwrap();
        store::get(&mut conn, self.project).await.unwrap().unwrap()
    }

    async fn want(&self, desired: Desired) {
        let mut conn = self.pool.acquire().await.unwrap();
        match desired {
            Desired::Running => store::want_running(&mut conn, self.project, PROVIDER).await.unwrap(),
            Desired::Stopped => store::want_stopped(&mut conn, self.project).await.unwrap(),
        }
    }

    async fn reconcile(&self) -> ReconcileOutcome {
        self.manager.reconcile(self.row().await).await
    }

    async fn retry_at(&self) -> Option<DateTime<Utc>> {
        sqlx::query_scalar("SELECT retry_at FROM project_sandboxes WHERE project_id = $1").bind(self.project).fetch_one(&self.pool).await.unwrap()
    }

    async fn sql(&self, set: &str) {
        sqlx::query(&format!("UPDATE project_sandboxes SET {set} WHERE project_id = $1")).bind(self.project).execute(&self.pool).await.unwrap();
    }

    async fn run(&self) -> SandboxId {
        self.want(Desired::Running).await;
        assert_eq!(self.reconcile().await, ReconcileOutcome::Settled);
        SandboxId(self.row().await.sandbox_id.unwrap())
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn stuck_creating_is_retried(pool: PgPool) {
    let f = fixture(pool).await;
    f.want(Desired::Running).await;
    let age = STUCK_AFTER.as_secs() as i64 + PAST_LIMIT_SECS;
    f.sql(&format!("state = 'creating', attempts = 2, retry_at = now() + interval '1 hour', leased_until = now() - interval '1 minute', updated_at = now() - interval '{age} seconds'")).await;

    assert_eq!(f.manager.unstick().await, 1);
    let row = f.row().await;
    assert_eq!(row.attempts, 2, "the watchdog must not count an attempt");
    assert!(f.retry_at().await.unwrap() <= Utc::now());

    let worker = SandboxWorker::new(f.pool.clone(), f.manager.clone(), IDLE_STOP_SECS);
    assert_eq!(worker.tick().await, 1);
    assert_eq!(f.row().await.state, State::Running);
}

#[sqlx::test(migrations = "../../migrations")]
async fn watchdog_leaves_live_walks_and_fresh_rows_alone(pool: PgPool) {
    let f = fixture(pool).await;
    f.want(Desired::Running).await;
    let age = STUCK_AFTER.as_secs() as i64 + PAST_LIMIT_SECS;
    // Old but a walk still holds it.
    f.sql(&format!("state = 'starting', leased_until = now() + interval '5 minutes', updated_at = now() - interval '{age} seconds'")).await;
    assert_eq!(f.manager.unstick().await, 0);
    // Not held, but not old.
    f.sql("leased_until = NULL, updated_at = now()").await;
    assert_eq!(f.manager.unstick().await, 0);
}

#[sqlx::test(migrations = "../../migrations")]
async fn five_failures_mark_error_and_set_cooldown(pool: PgPool) {
    let f = fixture(pool).await;
    f.want(Desired::Running).await;
    for i in 1..=MAX_ATTEMPTS {
        f.provider.fail_next(SandboxError::Other("boom".into()));
        f.sql("retry_at = now()").await;
        f.reconcile().await;
        if i < MAX_ATTEMPTS {
            assert_eq!(f.row().await.attempts, i);
        }
    }
    let row = f.row().await;
    assert_eq!(row.state, State::Error);
    assert_eq!(row.attempts, 0, "the count starts again after the cooldown");
    let until = f.retry_at().await.unwrap();
    let cooldown = Span::from_std(PROVIDER_COOLDOWN).unwrap();
    assert!(until > Utc::now() + cooldown - Span::seconds(PAST_LIMIT_SECS));

    // The page falls back while it cools down, and the row is not claimed.
    let mut conn = f.pool.acquire().await.unwrap();
    let status = store::status(&mut conn, f.project).await.unwrap().unwrap();
    let cfg = config();
    assert!(nomi_server::routes::project_sandbox::sandbox_available(Some(&cfg), "sandbox", true, Uuid::nil(), None, true));
    assert!(!nomi_server::routes::project_sandbox::sandbox_available(Some(&cfg), "sandbox", true, Uuid::nil(), Some(&status), true));
    let worker = SandboxWorker::new(f.pool.clone(), f.manager.clone(), IDLE_STOP_SECS);
    assert_eq!(worker.tick().await, 0);

    // Once it is over the row is retried normally.
    f.sql("retry_at = now() - interval '1 second'").await;
    assert_eq!(f.reconcile().await, ReconcileOutcome::Settled);
    assert_eq!(f.row().await.state, State::Running);
}

#[sqlx::test(migrations = "../../migrations")]
async fn waits_never_count_toward_the_limit(pool: PgPool) {
    let f = fixture(pool).await;
    f.want(Desired::Running).await;
    for _ in 0..MAX_ATTEMPTS + 2 {
        f.provider.fail_next(SandboxError::Unavailable("down".into()));
        f.sql("retry_at = now()").await;
        assert!(matches!(f.reconcile().await, ReconcileOutcome::RetryAt(_)));
    }
    let row = f.row().await;
    assert_eq!((row.attempts, row.state != State::Error), (0, true));
}

#[sqlx::test(migrations = "../../migrations")]
async fn provider_outage_does_not_hot_loop(pool: PgPool) {
    let f = fixture(pool).await;
    f.want(Desired::Running).await;
    for _ in 0..OUTAGE_TICKS {
        f.provider.fail_next(SandboxError::Unavailable("down".into()));
    }
    let worker = SandboxWorker::new(f.pool.clone(), f.manager.clone(), IDLE_STOP_SECS);
    for _ in 0..OUTAGE_TICKS {
        worker.tick().await;
    }
    // One try; the rest of the ticks found the row held back until its retry time.
    let tries = f.provider.calls().iter().filter(|c| c.starts_with("create")).count();
    assert_eq!(tries, 1, "{:?}", f.provider.calls());
    assert!(f.retry_at().await.unwrap() > Utc::now());
}

#[sqlx::test(migrations = "../../migrations")]
async fn long_stopped_sandbox_is_deleted_and_next_start_rebuilds(pool: PgPool) {
    let f = fixture(pool).await;
    let first = f.run().await;
    f.want(Desired::Stopped).await;
    assert_eq!(f.reconcile().await, ReconcileOutcome::Settled);

    // Not old enough yet.
    assert_eq!(f.manager.expire_stopped().await, 0);
    let age = EXPIRY_WARN.as_secs() as i64 + PAST_LIMIT_SECS;
    f.sql(&format!("updated_at = now() - interval '{age} seconds'")).await;
    assert_eq!(f.manager.expire_stopped().await, 1);

    assert!(f.provider.calls().contains(&format!("delete {}", first.0)));
    let (state, sandbox_id, version, deps): (String, Option<String>, i64, Option<String>) =
        sqlx::query_as("SELECT state, sandbox_id, synced_version, deps_hash FROM project_sandboxes WHERE project_id = $1").bind(f.project).fetch_one(&f.pool).await.unwrap();
    assert_eq!((state.as_str(), sandbox_id, version, deps), ("gone", None, 0, None));

    f.want(Desired::Running).await;
    assert_eq!(f.reconcile().await, ReconcileOutcome::Settled);
    let row = f.row().await;
    assert_ne!(row.sandbox_id.as_deref(), Some(first.0.as_str()));
    assert_eq!(f.steps.calls.lock().unwrap().iter().filter(|c| c.starts_with("sync")).count(), 2);
}

#[sqlx::test(migrations = "../../migrations")]
async fn expiry_never_touches_a_running_sandbox(pool: PgPool) {
    let f = fixture(pool).await;
    f.run().await;
    let age = EXPIRY_WARN.as_secs() as i64 + PAST_LIMIT_SECS;
    f.sql(&format!("updated_at = now() - interval '{age} seconds'")).await;
    assert_eq!(f.manager.expire_stopped().await, 0);
    assert!(!f.provider.calls().iter().any(|c| c.starts_with("delete")));
    assert_eq!(f.row().await.state, State::Running);
}

#[sqlx::test(migrations = "../../migrations")]
async fn server_restart_mid_sync_converges(pool: PgPool) {
    let f = fixture(pool).await;
    let id = f.run().await;
    // The server died after the sandbox came up but before the sync was recorded.
    f.sql("state = 'starting', synced_version = 0, leased_until = now() - interval '1 minute'").await;
    let restarted = SandboxManager::new(f.provider.clone(), f.steps.clone(), f.pool.clone(), config());
    assert_eq!(restarted.reconcile(f.row().await).await, ReconcileOutcome::Settled);
    let row = f.row().await;
    assert_eq!((row.state, row.synced_version, row.sandbox_id), (State::Running, SYNCED_VERSION, Some(id.0)));
    let open: i64 = sqlx::query_scalar("SELECT count(*) FROM sandbox_sessions WHERE project_id = $1 AND stopped_at IS NULL").bind(f.project).fetch_one(&f.pool).await.unwrap();
    assert_eq!(open, 1);
}
