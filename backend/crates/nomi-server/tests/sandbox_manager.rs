//! The reconciler that moves one project's sandbox toward what the project wants.

mod support;

use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use nomi_sandbox::{FakeProvider, PreviewUrl, SandboxError, SandboxId, SandboxProvider, SandboxState};
use nomi_server::configuration::{Rollout, SandboxConfig};
use nomi_server::sandbox_manager::{ProjectSteps, ReconcileOutcome, SandboxManager};
use nomi_server::sandbox_store::{self as store, Desired, State};
use sqlx::PgPool;
use support::App;
use uuid::Uuid;

const PROVIDER: &str = "boat";
/// The files version the recording steps report after a sync.
const SYNCED_VERSION: i64 = 7;
/// The preview address the recording steps hand back.
const PREVIEW: &str = "https://preview.example.test";
/// Start limits that never get in the way.
const GENEROUS_LIMIT: u32 = 1000;
/// Idle stop used in the test config, in seconds.
const IDLE_STOP_SECS: u64 = 600;

#[derive(Default)]
struct RecordingSteps {
    calls: Mutex<Vec<String>>,
}

impl RecordingSteps {
    fn calls(&self) -> Vec<String> {
        self.calls.lock().unwrap().clone()
    }
}

#[async_trait]
impl ProjectSteps for RecordingSteps {
    async fn sync(&self, _project_id: Uuid, sandbox: &SandboxId) -> Result<i64, String> {
        self.calls.lock().unwrap().push(format!("sync {}", sandbox.0));
        Ok(SYNCED_VERSION)
    }

    async fn start_preview(&self, _project_id: Uuid, sandbox: &SandboxId) -> Result<PreviewUrl, String> {
        self.calls.lock().unwrap().push(format!("preview {}", sandbox.0));
        Ok(PreviewUrl { url: PREVIEW.into() })
    }
}

fn config(per_minute: u32) -> SandboxConfig {
    SandboxConfig {
        api_base: String::new(),
        api_key: String::new(),
        size: nomi_sandbox::Size::Small,
        idle_stop_secs: IDLE_STOP_SECS,
        rollout: Rollout::All,
        starts_per_minute: per_minute,
        starts_per_hour: GENEROUS_LIMIT,
        monthly_minutes_per_user: GENEROUS_LIMIT,
    }
}

struct Fixture {
    pool: PgPool,
    provider: Arc<FakeProvider>,
    steps: Arc<RecordingSteps>,
    manager: SandboxManager,
    project: Uuid,
}

async fn fixture(pool: PgPool, per_minute: u32) -> Fixture {
    let app = App::new(pool.clone());
    let owner = app.person("owner@manager.test").await;
    let project = app.project(&owner, "manager").await.0;
    let provider = Arc::new(FakeProvider::new());
    let steps = Arc::new(RecordingSteps::default());
    let manager = SandboxManager::new(provider.clone(), steps.clone(), pool.clone(), config(per_minute));
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
        let row = self.row().await;
        self.manager.reconcile(row).await
    }

    fn calls_starting_with(&self, prefix: &str) -> usize {
        self.provider.calls().iter().filter(|c| c.starts_with(prefix)).count()
    }

    async fn open_sessions(&self) -> i64 {
        sqlx::query_scalar("SELECT count(*) FROM sandbox_sessions WHERE project_id = $1 AND stopped_at IS NULL")
            .bind(self.project)
            .fetch_one(&self.pool)
            .await
            .unwrap()
    }

    /// Bring the sandbox up and return its id.
    async fn run(&self) -> SandboxId {
        self.want(Desired::Running).await;
        assert_eq!(self.reconcile().await, ReconcileOutcome::Settled);
        SandboxId(self.row().await.sandbox_id.unwrap())
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn first_start_creates_syncs_and_runs(pool: PgPool) {
    let f = fixture(pool, GENEROUS_LIMIT).await;
    let id = f.run().await;
    let row = f.row().await;
    assert_eq!(row.state, State::Running);
    assert_eq!(row.synced_version, SYNCED_VERSION);
    assert_eq!(row.preview_url.as_deref(), Some(PREVIEW));
    assert_eq!(f.steps.calls(), vec![format!("sync {}", id.0), format!("preview {}", id.0)]);
    assert!(f.provider.calls().contains(&format!("create nomi-project-{}-0", f.project)));
    assert_eq!(f.open_sessions().await, 1);
}

#[sqlx::test(migrations = "../../migrations")]
async fn stopped_sandbox_is_resumed_not_recreated(pool: PgPool) {
    let f = fixture(pool, GENEROUS_LIMIT).await;
    let id = f.run().await;
    f.want(Desired::Stopped).await;
    assert_eq!(f.reconcile().await, ReconcileOutcome::Settled);
    assert_eq!(f.row().await.state, State::Stopped);
    assert_eq!(f.open_sessions().await, 0);

    f.want(Desired::Running).await;
    assert_eq!(f.reconcile().await, ReconcileOutcome::Settled);
    assert_eq!(f.calls_starting_with("create"), 1);
    assert!(f.provider.calls().contains(&format!("resume {}", id.0)));
    assert_eq!(f.row().await.state, State::Running);
    assert_eq!(f.open_sessions().await, 1);
}

#[sqlx::test(migrations = "../../migrations")]
async fn snapshot_expired_recreates_and_resyncs(pool: PgPool) {
    let f = fixture(pool, GENEROUS_LIMIT).await;
    let first = f.run().await;
    f.want(Desired::Stopped).await;
    f.reconcile().await;
    f.want(Desired::Running).await;
    f.provider.fail_next(SandboxError::SnapshotExpired);
    assert_eq!(f.reconcile().await, ReconcileOutcome::Settled);

    let row = f.row().await;
    assert_eq!(row.state, State::Running);
    assert_ne!(row.sandbox_id.as_deref(), Some(first.0.as_str()));
    assert_eq!(f.calls_starting_with("create"), 2);
    assert_eq!(f.steps.calls().iter().filter(|c| c.starts_with("sync")).count(), 2);
}

#[sqlx::test(migrations = "../../migrations")]
async fn rate_limited_start_schedules_retry_and_does_not_fail(pool: PgPool) {
    let f = fixture(pool, 0).await;
    f.want(Desired::Running).await;
    let outcome = f.reconcile().await;
    assert!(matches!(outcome, ReconcileOutcome::RetryAt(_)), "{outcome:?}");
    assert!(f.provider.calls().is_empty());
    let row = f.row().await;
    assert_ne!(row.state, State::Error);
    let retry: Option<chrono::DateTime<chrono::Utc>> =
        sqlx::query_scalar("SELECT retry_at FROM project_sandboxes WHERE project_id = $1").bind(f.project).fetch_one(&f.pool).await.unwrap();
    assert!(retry.unwrap() > chrono::Utc::now());
}

#[sqlx::test(migrations = "../../migrations")]
async fn provider_rate_limit_waits_as_long_as_the_provider_said(pool: PgPool) {
    let f = fixture(pool, GENEROUS_LIMIT).await;
    f.want(Desired::Running).await;
    f.provider.fail_next(SandboxError::RateLimited { retry_after: Duration::from_secs(90) });
    let before = chrono::Utc::now();
    match f.reconcile().await {
        ReconcileOutcome::RetryAt(at) => assert!(at >= before + chrono::Duration::seconds(89)),
        other => panic!("{other:?}"),
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn stop_refused_is_retried_without_force(pool: PgPool) {
    let f = fixture(pool, GENEROUS_LIMIT).await;
    f.run().await;
    f.want(Desired::Stopped).await;
    f.provider.fail_next(SandboxError::Refused("busy".into()));
    assert!(matches!(f.reconcile().await, ReconcileOutcome::RetryAt(_)));
    assert_eq!(f.open_sessions().await, 1);
    sqlx::query("UPDATE project_sandboxes SET retry_at = now() WHERE project_id = $1").bind(f.project).execute(&f.pool).await.unwrap();
    assert_eq!(f.reconcile().await, ReconcileOutcome::Settled);
    let stops: Vec<String> = f.provider.calls().into_iter().filter(|c| c.starts_with("stop")).collect();
    assert_eq!(stops.len(), 2);
    assert!(stops.iter().all(|c| c.ends_with("force=false")));
    assert_eq!(f.row().await.state, State::Stopped);
    assert_eq!(f.open_sessions().await, 0);
}

#[sqlx::test(migrations = "../../migrations")]
async fn two_reconciles_of_the_same_row_create_one_sandbox(pool: PgPool) {
    let f = fixture(pool, GENEROUS_LIMIT).await;
    f.want(Desired::Running).await;
    let row = f.row().await;
    let (a, b) = tokio::join!(f.manager.reconcile(row.clone()), f.manager.reconcile(row));
    assert!(matches!(a, ReconcileOutcome::Settled) || matches!(b, ReconcileOutcome::Settled));
    let created = f.provider.list().await.unwrap();
    assert_eq!(created.len(), 1);
    assert_eq!(f.open_sessions().await, 1);
}

#[sqlx::test(migrations = "../../migrations")]
async fn provider_gone_marks_row_none_and_recreates_next_time(pool: PgPool) {
    let f = fixture(pool, GENEROUS_LIMIT).await;
    let first = f.run().await;
    f.provider.set_state(&first, SandboxState::Gone);
    sqlx::query("UPDATE project_sandboxes SET state = 'starting' WHERE project_id = $1").bind(f.project).execute(&f.pool).await.unwrap();
    assert!(matches!(f.reconcile().await, ReconcileOutcome::RetryAt(_)));
    let row = f.row().await;
    assert_eq!((row.state, row.sandbox_id.clone()), (State::None, None));
    assert_eq!(f.open_sessions().await, 0);

    assert_eq!(f.reconcile().await, ReconcileOutcome::Settled);
    let row = f.row().await;
    assert_eq!(row.state, State::Running);
    assert_ne!(row.sandbox_id.as_deref(), Some(first.0.as_str()));
}

async fn attempts_and_retry(f: &Fixture) -> (i32, Option<chrono::DateTime<chrono::Utc>>) {
    sqlx::query_as("SELECT attempts, retry_at FROM project_sandboxes WHERE project_id = $1").bind(f.project).fetch_one(&f.pool).await.unwrap()
}

#[sqlx::test(migrations = "../../migrations")]
async fn limiter_waits_and_rate_limits_do_not_count_as_attempts(pool: PgPool) {
    let f = fixture(pool, 0).await;
    f.want(Desired::Running).await;
    assert!(matches!(f.reconcile().await, ReconcileOutcome::RetryAt(_)));
    assert_eq!(attempts_and_retry(&f).await.0, 0);

    let g_pool = f.pool.clone();
    let open = SandboxManager::new(f.provider.clone(), f.steps.clone(), g_pool, config(GENEROUS_LIMIT));
    f.provider.fail_next(SandboxError::RateLimited { retry_after: Duration::from_secs(30) });
    assert!(matches!(open.reconcile(f.row().await).await, ReconcileOutcome::RetryAt(_)));
    let (attempts, retry) = attempts_and_retry(&f).await;
    assert_eq!(attempts, 0);
    assert!(retry.unwrap() > chrono::Utc::now());
}

#[sqlx::test(migrations = "../../migrations")]
async fn sandbox_gone_before_running_gets_a_new_key_and_backs_off(pool: PgPool) {
    let f = fixture(pool, GENEROUS_LIMIT).await;
    f.want(Desired::Running).await;
    // A sandbox already made under the key the first create will send, and dead.
    let spec = nomi_sandbox::CreateSpec {
        size: nomi_sandbox::Size::Small,
        ttl_seconds: None,
        from_snapshot: None,
        idempotency_key: format!("nomi-project-{}-0", f.project),
        labels: Default::default(),
    };
    let dead = f.provider.create(spec).await.unwrap().id;
    f.provider.set_state(&dead, SandboxState::Gone);

    assert!(matches!(f.reconcile().await, ReconcileOutcome::RetryAt(_)));
    let (attempts, retry) = attempts_and_retry(&f).await;
    assert_eq!(attempts, 1);
    assert!(retry.unwrap() > chrono::Utc::now() + chrono::Duration::seconds(3));
    assert_eq!(f.row().await.sandbox_id, None);

    assert_eq!(f.reconcile().await, ReconcileOutcome::Settled);
    assert!(f.provider.calls().contains(&format!("create nomi-project-{}-1", f.project)));
    assert_ne!(f.row().await.sandbox_id.as_deref(), Some(dead.0.as_str()));
}

#[sqlx::test(migrations = "../../migrations")]
async fn stale_walk_cannot_overwrite_sandbox_id(pool: PgPool) {
    let f = fixture(pool, GENEROUS_LIMIT).await;
    f.want(Desired::Running).await;
    // Another walk records its sandbox right after this one marks the row creating.
    sqlx::query(
        "CREATE FUNCTION other_walk() RETURNS trigger AS $$ BEGIN
             IF NEW.state = 'creating' THEN NEW.sandbox_id := 'someone-else'; END IF; RETURN NEW; END $$ LANGUAGE plpgsql",
    )
    .execute(&f.pool)
    .await
    .unwrap();
    sqlx::query("CREATE TRIGGER other_walk BEFORE UPDATE ON project_sandboxes FOR EACH ROW EXECUTE FUNCTION other_walk()")
        .execute(&f.pool)
        .await
        .unwrap();
    assert!(matches!(f.reconcile().await, ReconcileOutcome::RetryAt(_)));
    assert_eq!(f.row().await.sandbox_id.as_deref(), Some("someone-else"));
    assert_eq!(f.provider.list().await.unwrap().len(), 0, "the sandbox this walk made is deleted");
    assert!(f.steps.calls().is_empty());
}
