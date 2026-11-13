//! The reconciler that moves one project's sandbox toward what the project wants.

mod support;

use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use nomi_sandbox::{FakeProvider, PreviewUrl, SandboxError, SandboxId, SandboxProvider, SandboxState};
use nomi_server::configuration::{Rollout, SandboxConfig};
use nomi_server::sandbox_manager::{ProjectSteps, ReconcileOutcome, SandboxManager, StepError, AUTO_STOP_EXTEND_BEFORE, AUTO_STOP_RETRY_BASE, DEVSERVER_RESTART_GAP, MAX_ATTEMPTS};

use nomi_server::sandbox_store::{self as store, Desired, State};
use sqlx::PgPool;
use support::App;
use uuid::Uuid;

/// The provider-side auto-stop used in these tests, in seconds.
const AUTO_STOP_SECS: u64 = 6 * 60 * 60;
/// The same, as the provider is asked for it.
const TTL: nomi_sandbox::Ttl = nomi_sandbox::Ttl::Seconds(AUTO_STOP_SECS);

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
    /// The files version the next syncs report; `SYNCED_VERSION` until set.
    version: Mutex<Option<i64>>,
    /// What every preview start fails with, when set.
    preview_fails: Mutex<Option<StepError>>,
    /// What every dependency update fails with, when set.
    install_fails: Mutex<Option<StepError>>,
    /// How long each sync takes.
    sync_takes: Mutex<Duration>,
    /// Whether each dependency update actually runs an install.
    installs: Mutex<bool>,
    /// Whether the dev server has stopped answering.
    dev_server_dead: Mutex<bool>,
    /// The preview address the next starts hand back; `PREVIEW` until set.
    preview: Mutex<Option<String>>,
}

impl RecordingSteps {
    fn calls(&self) -> Vec<String> {
        self.calls.lock().unwrap().clone()
    }

    fn count(&self, prefix: &str) -> usize {
        self.calls().iter().filter(|c| c.starts_with(prefix)).count()
    }
}

#[async_trait]
impl ProjectSteps for RecordingSteps {
    async fn sync(&self, _project_id: Uuid, sandbox: &SandboxId) -> Result<i64, String> {
        self.calls.lock().unwrap().push(format!("sync {}", sandbox.0));
        let takes = *self.sync_takes.lock().unwrap();
        tokio::time::sleep(takes).await;
        Ok(self.version.lock().unwrap().unwrap_or(SYNCED_VERSION))
    }

    async fn start_preview(&self, _project_id: Uuid, sandbox: &SandboxId) -> Result<PreviewUrl, StepError> {
        self.calls.lock().unwrap().push(format!("preview {}", sandbox.0));
        match self.preview_fails.lock().unwrap().clone() {
            Some(err) => Err(err),
            None => {
                // A started dev server answers.
                *self.dev_server_dead.lock().unwrap() = false;
                Ok(PreviewUrl { url: self.preview.lock().unwrap().clone().unwrap_or_else(|| PREVIEW.into()) })
            }
        }
    }

    async fn update_dependencies(&self, _project_id: Uuid, sandbox: &SandboxId) -> Result<bool, StepError> {
        self.calls.lock().unwrap().push(format!("deps {}", sandbox.0));
        match self.install_fails.lock().unwrap().clone() {
            Some(err) => Err(err),
            None => Ok(*self.installs.lock().unwrap()),
        }
    }

    async fn dev_server_answers(&self, _project_id: Uuid, sandbox: &SandboxId) -> Result<bool, String> {
        self.calls.lock().unwrap().push(format!("probe {}", sandbox.0));
        Ok(!*self.dev_server_dead.lock().unwrap())
    }
}

fn config(per_minute: u32) -> SandboxConfig {
    SandboxConfig {
        api_base: String::new(),
        api_key: String::new(),
        size: nomi_sandbox::Size::Small,
        idle_stop_secs: IDLE_STOP_SECS,
        auto_stop_secs: AUTO_STOP_SECS,
        rollout: Rollout::All,
        starts_per_minute: per_minute,
        starts_per_hour: GENEROUS_LIMIT,
        monthly_minutes_per_user: GENEROUS_LIMIT,
        reaper_mode: nomi_server::configuration::ReaperMode::DryRun,
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

    /// The rows a worker would claim now (the claim is rolled back, so nothing is leased).
    async fn due(&self) -> Vec<Uuid> {
        let mut tx = self.pool.begin().await.unwrap();
        let rows = store::claim_due(&mut tx, 10).await.unwrap();
        tx.rollback().await.unwrap();
        rows.into_iter().map(|r| r.project_id).collect()
    }

    async fn set_files_version(&self, version: i64) {
        sqlx::query("UPDATE projects SET files_version = $2 WHERE id = $1").bind(self.project).bind(version).execute(&self.pool).await.unwrap();
    }

    /// Make the row due for the next liveness pass.
    async fn recheck_alive(&self) {
        sqlx::query("UPDATE project_sandboxes SET alive_checked_at = NULL WHERE project_id = $1").bind(self.project).execute(&self.pool).await.unwrap();
    }

    /// Put the recorded auto-stop within the extension margin.
    async fn near_auto_stop(&self) {
        sqlx::query("UPDATE project_sandboxes SET auto_stop_at = now() + interval '30 minutes' WHERE project_id = $1").bind(self.project).execute(&self.pool).await.unwrap();
    }

    /// End the extension backoff now.
    async fn extension_due_now(&self) {
        sqlx::query("UPDATE project_sandboxes SET auto_stop_retry_at = now() - interval '1 second' WHERE project_id = $1").bind(self.project).execute(&self.pool).await.unwrap();
    }

    /// Failed extensions in a row, and seconds until the next may be tried.
    async fn extension_backoff(&self) -> (i32, Option<f64>) {
        sqlx::query_as("SELECT auto_stop_failures, EXTRACT(EPOCH FROM auto_stop_retry_at - now())::float8 FROM project_sandboxes WHERE project_id = $1").bind(self.project).fetch_one(&self.pool).await.unwrap()
    }

    /// Seconds until the recorded auto-stop deadline.
    async fn auto_stop_in(&self) -> f64 {
        sqlx::query_scalar("SELECT EXTRACT(EPOCH FROM auto_stop_at - now())::float8 FROM project_sandboxes WHERE project_id = $1").bind(self.project).fetch_one(&self.pool).await.unwrap()
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
    let outcome = f.reconcile().await;
    let row = f.row().await;
    assert!(matches!(outcome, ReconcileOutcome::RetryAt(_)), "got {outcome:?}, row state {:?}, last_error {:?}", row.state, row.last_error);
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
        ttl: nomi_sandbox::Ttl::ProviderDefault,
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

#[sqlx::test(migrations = "../../migrations")]
async fn repeated_deaths_back_off_exponentially(pool: PgPool) {
    let f = fixture(pool, GENEROUS_LIMIT).await;
    f.want(Desired::Running).await;
    let mut delays = Vec::new();
    for n in 0..3 {
        // Each create sends the next key; kill the sandbox it would get before it can run.
        let spec = nomi_sandbox::CreateSpec {
            size: nomi_sandbox::Size::Small,
            ttl: nomi_sandbox::Ttl::ProviderDefault,
            from_snapshot: None,
            idempotency_key: format!("nomi-project-{}-{n}", f.project),
            labels: Default::default(),
        };
        let dead = f.provider.create(spec).await.unwrap().id;
        f.provider.set_state(&dead, SandboxState::Gone);
        let before = chrono::Utc::now();
        match f.reconcile().await {
            ReconcileOutcome::RetryAt(at) => delays.push((at - before).num_seconds()),
            other => panic!("{other:?}"),
        }
        assert_eq!(attempts_and_retry(&f).await.0, n + 1);
    }
    assert!(delays[0] < delays[1] && delays[1] < delays[2], "{delays:?}");
}

#[sqlx::test(migrations = "../../migrations")]
async fn sandbox_that_times_out_then_vanishes_keeps_backing_off(pool: PgPool) {
    let f = fixture(pool, GENEROUS_LIMIT).await;
    let id = f.run().await;
    // A start that timed out earlier: still starting, two failed attempts so far, sandbox now gone.
    sqlx::query("UPDATE project_sandboxes SET state = 'starting', attempts = 2 WHERE project_id = $1").bind(f.project).execute(&f.pool).await.unwrap();
    f.provider.set_state(&id, SandboxState::Gone);
    let before = chrono::Utc::now();
    match f.reconcile().await {
        ReconcileOutcome::RetryAt(at) => assert!(at >= before + chrono::Duration::seconds(30), "{at}"),
        other => panic!("{other:?}"),
    }
    assert_eq!(attempts_and_retry(&f).await.0, 3);
    assert_eq!(f.row().await.sandbox_id, None);
}

#[sqlx::test(migrations = "../../migrations")]
async fn stopped_sandbox_that_vanished_is_recreated_at_once(pool: PgPool) {
    let f = fixture(pool, GENEROUS_LIMIT).await;
    let id = f.run().await;
    f.want(Desired::Stopped).await;
    f.reconcile().await;
    f.provider.set_state(&id, SandboxState::Gone);
    f.want(Desired::Running).await;
    let before = chrono::Utc::now();
    match f.reconcile().await {
        ReconcileOutcome::RetryAt(at) => assert!(at <= before + chrono::Duration::seconds(2)),
        other => panic!("{other:?}"),
    }
    assert_eq!(attempts_and_retry(&f).await.0, 0);
    assert_eq!(f.reconcile().await, ReconcileOutcome::Settled);
}

/// What the page would read when it hears the error event.
type SeenAtEvent = Mutex<Vec<(Uuid, String, Option<String>)>>;

#[sqlx::test(migrations = "../../migrations")]
async fn error_event_is_sent_after_last_error_is_stored(pool: PgPool) {
    static SEEN: SeenAtEvent = Mutex::new(Vec::new());
    let f = fixture(pool.clone(), GENEROUS_LIMIT).await;
    let reader = pool.clone();
    let watched = f.project;
    nomi_server::project_events::observe_sandbox_changes_for_test(move |project, phase| {
        // Other tests in this process announce too, against their own databases.
        if project != watched {
            return;
        }
        // Read the row from another thread: the event is sent from inside the reconciler's task.
        // (its own connection: the pool belongs to the runtime this call is blocking)
        let options = reader.connect_options();
        let error = std::thread::spawn(move || {
            tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(async move {
                use sqlx::ConnectOptions;
                let mut conn = options.connect().await.unwrap();
                sqlx::query_scalar::<_, Option<String>>("SELECT last_error FROM project_sandboxes WHERE project_id = $1").bind(project).fetch_one(&mut conn).await.unwrap()
            })
        })
        .join()
        .unwrap();
        SEEN.lock().unwrap().push((project, phase.to_string(), error));
    });

    f.want(Desired::Running).await;
    f.provider.fail_next(SandboxError::Refused("no capacity for you".into()));
    assert!(matches!(f.reconcile().await, ReconcileOutcome::Failed(_)));
    let seen = SEEN.lock().unwrap().clone();
    let error_event = seen.iter().find(|(p, phase, _)| *p == f.project && phase == "error").expect("an error event was sent");
    assert!(error_event.2.as_deref().is_some_and(|e| e.contains("no capacity")), "last_error was stored by the time of the event: {error_event:?}");
}

#[sqlx::test(migrations = "../../migrations")]
async fn edits_after_running_are_synced_into_the_running_sandbox(pool: PgPool) {
    let f = fixture(pool, GENEROUS_LIMIT).await;
    let id = f.run().await;
    assert!(f.due().await.is_empty(), "nothing changed: the running row isn't due");

    // Koda (or the person) saves a file: the project moves past what the sandbox holds.
    f.set_files_version(SYNCED_VERSION + 1).await;
    *f.steps.version.lock().unwrap() = Some(SYNCED_VERSION + 1);
    assert_eq!(f.due().await, vec![f.project], "a running sandbox behind its files is due");
    assert_eq!(f.reconcile().await, ReconcileOutcome::Settled);

    let row = f.row().await;
    assert_eq!((row.state, row.synced_version), (State::Running, SYNCED_VERSION + 1));
    assert_eq!(f.steps.count("sync"), 2, "the edit was copied in");
    assert_eq!(f.steps.calls().last().map(String::as_str), Some(format!("deps {}", id.0).as_str()), "dependencies are checked after the sync");
    assert_eq!(f.steps.count("preview"), 1, "the dev server keeps running (HMR picks the change up)");
    assert_eq!(f.calls_starting_with("create") + f.calls_starting_with("resume"), 1, "no restart");
    assert_eq!(f.open_sessions().await, 1);
    assert!(f.due().await.is_empty());
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_running_command_holds_back_the_resync_until_it_ends(pool: PgPool) {
    let f = fixture(pool, GENEROUS_LIMIT).await;
    f.run().await;
    f.set_files_version(SYNCED_VERSION + 1).await;
    let run: Uuid = sqlx::query_scalar("INSERT INTO project_runs (project_id, command, status, started_at) VALUES ($1, 'npm', 'running', now()) RETURNING id")
        .bind(f.project)
        .fetch_one(&f.pool)
        .await
        .unwrap();
    assert!(f.due().await.is_empty(), "no sync or install under a running command");
    sqlx::query("UPDATE project_runs SET status = 'done', finished_at = now() WHERE id = $1").bind(run).execute(&f.pool).await.unwrap();
    assert_eq!(f.due().await, vec![f.project]);
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_failed_install_leaves_a_running_metered_sandbox_koda_can_use(pool: PgPool) {
    let f = fixture(pool, GENEROUS_LIMIT).await;
    *f.steps.preview_fails.lock().unwrap() = Some(StepError::Usable("installing dependencies failed: ERESOLVE".into()));
    f.want(Desired::Running).await;
    assert_eq!(f.reconcile().await, ReconcileOutcome::Settled);
    let row = f.row().await;
    assert_eq!((row.state, row.preview_url.clone(), row.attempts), (State::Running, None, 0));
    assert!(row.last_error.as_deref().is_some_and(|e| e.contains("ERESOLVE")), "{row:?}");
    assert_eq!(f.open_sessions().await, 1, "the sandbox is metered while it runs");
    assert!(f.due().await.is_empty(), "no retry loop: the next try waits for a change or the person");

    // Koda fixes package.json: the new files are synced, installed, and the preview tried again.
    *f.steps.preview_fails.lock().unwrap() = None;
    f.set_files_version(SYNCED_VERSION + 1).await;
    *f.steps.version.lock().unwrap() = Some(SYNCED_VERSION + 1);
    assert_eq!(f.due().await, vec![f.project]);
    assert_eq!(f.reconcile().await, ReconcileOutcome::Settled);
    let row = f.row().await;
    assert_eq!((row.preview_url.as_deref(), row.last_error.clone()), (Some(PREVIEW), None));

    // The person's Retry on a failed preview makes the row due again too.
    sqlx::query("UPDATE project_sandboxes SET preview_url = NULL, last_error = 'dev server down' WHERE project_id = $1").bind(f.project).execute(&f.pool).await.unwrap();
    assert!(f.due().await.is_empty());
    let mut conn = f.pool.acquire().await.unwrap();
    store::retry_preview(&mut conn, f.project).await.unwrap();
    assert_eq!(f.due().await, vec![f.project]);
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_failed_reinstall_keeps_the_sandbox_running_with_the_reason(pool: PgPool) {
    let f = fixture(pool, GENEROUS_LIMIT).await;
    f.run().await;
    *f.steps.install_fails.lock().unwrap() = Some(StepError::Usable("installing dependencies failed: ETARGET".into()));
    f.set_files_version(SYNCED_VERSION + 1).await;
    *f.steps.version.lock().unwrap() = Some(SYNCED_VERSION + 1);
    assert_eq!(f.reconcile().await, ReconcileOutcome::Settled);
    let row = f.row().await;
    assert_eq!(row.state, State::Running);
    assert!(row.last_error.as_deref().is_some_and(|e| e.contains("ETARGET")));
    assert_eq!(row.synced_version, SYNCED_VERSION + 1);
}

#[sqlx::test(migrations = "../../migrations")]
async fn entering_the_cooldown_stops_the_sandbox_and_a_close_stops_the_row_at_once(pool: PgPool) {
    let f = fixture(pool, GENEROUS_LIMIT).await;
    *f.steps.preview_fails.lock().unwrap() = Some(StepError::Retry("the project's backend isn't reachable".into()));
    f.want(Desired::Running).await;
    sqlx::query("UPDATE project_sandboxes SET attempts = $2 WHERE project_id = $1").bind(f.project).bind(MAX_ATTEMPTS - 1).execute(&f.pool).await.unwrap();
    assert!(matches!(f.reconcile().await, ReconcileOutcome::Failed(_)));
    let row = f.row().await;
    assert_eq!(row.state, State::Error);
    let id = SandboxId(row.sandbox_id.clone().unwrap());
    assert_eq!(f.provider.get(&id).await.unwrap().state, SandboxState::Stopped, "a cooling-down row doesn't keep a sandbox running");
    assert_eq!(f.open_sessions().await, 0);

    // The page closes (or the sweep finds it idle): the row is stopped without waiting out the cooldown.
    f.want(Desired::Stopped).await;
    assert_eq!(f.due().await, vec![f.project]);
    assert_eq!(f.reconcile().await, ReconcileOutcome::Settled);
    assert_eq!(f.row().await.state, State::Stopped);
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_database_error_holds_the_row_back_instead_of_a_hot_loop(pool: PgPool) {
    let f = fixture(pool, GENEROUS_LIMIT).await;
    f.want(Desired::Running).await;
    sqlx::query("CREATE FUNCTION refuse_creating() RETURNS trigger AS $$ BEGIN IF NEW.state = 'creating' THEN RAISE EXCEPTION 'disk full'; END IF; RETURN NEW; END $$ LANGUAGE plpgsql")
        .execute(&f.pool)
        .await
        .unwrap();
    sqlx::query("CREATE TRIGGER refuse_creating BEFORE UPDATE ON project_sandboxes FOR EACH ROW EXECUTE FUNCTION refuse_creating()").execute(&f.pool).await.unwrap();
    match f.reconcile().await {
        ReconcileOutcome::Failed(why) => assert!(why.contains("database"), "{why}"),
        other => panic!("{other:?}"),
    }
    assert!(attempts_and_retry(&f).await.1.is_some_and(|at| at > chrono::Utc::now()), "held back, not re-claimed on the next poll");
    assert!(f.due().await.is_empty());
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_long_walk_keeps_its_lease(pool: PgPool) {
    let app = App::new(pool.clone());
    let owner = app.person("lease@manager.test").await;
    let project = app.project(&owner, "lease").await.0;
    let provider = Arc::new(FakeProvider::new());
    let steps = Arc::new(RecordingSteps::default());
    *steps.sync_takes.lock().unwrap() = Duration::from_millis(600);
    let manager = SandboxManager::new(provider, steps, pool.clone(), config(GENEROUS_LIMIT)).with_lease_renewal(Duration::from_millis(50));
    let mut conn = pool.acquire().await.unwrap();
    store::want_running(&mut conn, project, PROVIDER).await.unwrap();
    let row = store::get(&mut conn, project).await.unwrap().unwrap();
    // A claim's lease that is about to run out under a slow walk.
    sqlx::query("UPDATE project_sandboxes SET leased_until = now() + interval '200 milliseconds' WHERE project_id = $1").bind(project).execute(&pool).await.unwrap();
    let check = async {
        tokio::time::sleep(Duration::from_millis(400)).await;
        let held: bool = sqlx::query_scalar("SELECT leased_until > now() + interval '1 minute' FROM project_sandboxes WHERE project_id = $1").bind(project).fetch_one(&pool).await.unwrap();
        held
    };
    let (outcome, held) = tokio::join!(manager.reconcile(row), check);
    assert_eq!(outcome, ReconcileOutcome::Settled);
    assert!(held, "the walk renewed its lease while it ran");
    let after: Option<chrono::DateTime<chrono::Utc>> = sqlx::query_scalar("SELECT leased_until FROM project_sandboxes WHERE project_id = $1").bind(project).fetch_one(&pool).await.unwrap();
    assert_eq!(after, None, "released when the walk ended");
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_sandbox_that_died_while_running_is_walked_again(pool: PgPool) {
    let f = fixture(pool, GENEROUS_LIMIT).await;
    let id = f.run().await;
    // Just brought up and still up: not asked about again within the interval.
    assert_eq!(f.manager.check_alive().await, 0);
    let gets = f.calls_starting_with("get");
    assert_eq!(f.manager.check_alive().await, 0);
    assert_eq!(f.calls_starting_with("get"), gets, "at most one look per sandbox per interval");

    // It stops by itself (boat's auto-stop, a crash): the next pass finds it.
    f.provider.set_state(&id, SandboxState::Stopped);
    sqlx::query("UPDATE project_sandboxes SET alive_checked_at = NULL WHERE project_id = $1").bind(f.project).execute(&f.pool).await.unwrap();
    assert_eq!(f.manager.check_alive().await, 1);
    let row = f.row().await;
    assert_eq!((row.state, row.preview_url.clone()), (State::Stopped, None));
    assert_eq!(f.open_sessions().await, 0, "its metering stopped with it");
    assert_eq!(f.due().await, vec![f.project], "wanted running, so it is walked again");
    assert_eq!(f.reconcile().await, ReconcileOutcome::Settled);
    assert_eq!(f.row().await.state, State::Running);
    assert!(f.provider.calls().contains(&format!("resume {}", id.0)));
}

#[sqlx::test(migrations = "../../migrations")]
async fn sandboxes_are_created_and_resumed_with_a_long_finite_auto_stop(pool: PgPool) {
    let f = fixture(pool, GENEROUS_LIMIT).await;
    f.run().await;
    f.want(Desired::Stopped).await;
    f.reconcile().await;
    f.want(Desired::Running).await;
    f.reconcile().await;
    assert_eq!(f.provider.ttls(), vec![TTL, TTL]);
    assert!(f.auto_stop_in().await > AUTO_STOP_EXTEND_BEFORE.as_secs_f64(), "the deadline of the resume is recorded");
}

#[sqlx::test(migrations = "../../migrations")]
async fn liveness_check_extends_the_auto_stop_deadline(pool: PgPool) {
    let f = fixture(pool, GENEROUS_LIMIT).await;
    let id = f.run().await;
    // Far from its deadline: nothing to extend.
    f.recheck_alive().await;
    f.manager.check_alive().await;
    assert!(f.provider.extensions().is_empty());

    // Close to the deadline the provider was given: pushed out again, well before it.
    sqlx::query("UPDATE project_sandboxes SET auto_stop_at = now() + interval '30 minutes' WHERE project_id = $1").bind(f.project).execute(&f.pool).await.unwrap();
    f.recheck_alive().await;
    f.manager.check_alive().await;
    assert_eq!(f.provider.extensions(), vec![(id, TTL)]);
    assert!(f.auto_stop_in().await > AUTO_STOP_EXTEND_BEFORE.as_secs_f64(), "the new deadline is recorded");
    assert_eq!(f.row().await.state, State::Running);
}

#[sqlx::test(migrations = "../../migrations")]
async fn failed_auto_stop_extension_backs_off(pool: PgPool) {
    let f = fixture(pool, GENEROUS_LIMIT).await;
    f.run().await;
    f.provider.fail_extensions(Some(SandboxError::Refused("sandbox_running_stop_first".into())));
    f.near_auto_stop().await;
    f.recheck_alive().await;
    f.manager.check_alive().await;
    assert_eq!(f.provider.extensions().len(), 1);
    let (failures, wait) = f.extension_backoff().await;
    assert_eq!(failures, 1);
    assert!(wait.unwrap() >= AUTO_STOP_RETRY_BASE.as_secs_f64() - 5.0, "the next try waits: {wait:?}");
    assert!(f.auto_stop_in().await < AUTO_STOP_EXTEND_BEFORE.as_secs_f64(), "a failed extension moves no deadline");
    assert_eq!(f.row().await.state, State::Running, "the sandbox keeps running");

    // Later liveness passes leave it alone until then: no call, no start taken.
    f.recheck_alive().await;
    f.manager.check_alive().await;
    assert_eq!(f.provider.extensions().len(), 1, "backed off");

    // Due again and failing again: the wait grows.
    f.extension_due_now().await;
    f.recheck_alive().await;
    f.manager.check_alive().await;
    assert_eq!(f.provider.extensions().len(), 2);
    let (failures, wait) = f.extension_backoff().await;
    assert_eq!(failures, 2);
    assert!(wait.unwrap() >= 2.0 * AUTO_STOP_RETRY_BASE.as_secs_f64() - 5.0, "{wait:?}");

    // Once it works, the deadline moves and the backoff is forgotten.
    f.provider.fail_extensions(None);
    f.extension_due_now().await;
    f.recheck_alive().await;
    f.manager.check_alive().await;
    assert_eq!(f.provider.extensions().len(), 3);
    assert_eq!(f.extension_backoff().await, (0, None));
    assert!(f.auto_stop_in().await > AUTO_STOP_EXTEND_BEFORE.as_secs_f64());
}

#[sqlx::test(migrations = "../../migrations")]
async fn extension_records_the_provider_deadline_when_reported(pool: PgPool) {
    let f = fixture(pool, GENEROUS_LIMIT).await;
    f.run().await;
    let reported = chrono::Utc::now() + chrono::Duration::hours(5);
    f.provider.report_auto_stop(&reported.to_rfc3339());
    f.near_auto_stop().await;
    f.recheck_alive().await;
    f.manager.check_alive().await;
    let saved: chrono::DateTime<chrono::Utc> = sqlx::query_scalar("SELECT auto_stop_at FROM project_sandboxes WHERE project_id = $1").bind(f.project).fetch_one(&f.pool).await.unwrap();
    assert!((saved - reported).num_milliseconds().abs() < 1000, "the provider's deadline is saved, not an assumed one: {saved} vs {reported}");
    assert_eq!(f.extension_backoff().await, (0, None));

    // The provider took the call but its clock didn't move: saved as it is, and backed off like a failure.
    let unmoved = chrono::Utc::now() + chrono::Duration::minutes(30);
    f.provider.report_auto_stop(&unmoved.to_rfc3339());
    f.near_auto_stop().await;
    f.recheck_alive().await;
    f.manager.check_alive().await;
    let saved: chrono::DateTime<chrono::Utc> = sqlx::query_scalar("SELECT auto_stop_at FROM project_sandboxes WHERE project_id = $1").bind(f.project).fetch_one(&f.pool).await.unwrap();
    assert!((saved - unmoved).num_milliseconds().abs() < 1000);
    let (failures, wait) = f.extension_backoff().await;
    assert_eq!(failures, 1);
    assert!(wait.is_some());
}

#[sqlx::test(migrations = "../../migrations")]
async fn dead_dev_server_in_a_live_sandbox_is_restarted(pool: PgPool) {
    let f = fixture(pool, GENEROUS_LIMIT).await;
    let id = f.run().await;
    *f.steps.preview.lock().unwrap() = Some(format!("{PREVIEW}/restarted"));

    // The sandbox is up, but its dev server crashed.
    *f.steps.dev_server_dead.lock().unwrap() = true;
    f.recheck_alive().await;
    assert_eq!(f.manager.check_alive().await, 0, "the sandbox itself is up");
    assert!(f.steps.calls().contains(&format!("probe {}", id.0)), "the liveness check asked the dev server");
    let reason: Option<String> = sqlx::query_scalar("SELECT devserver_restart_reason FROM project_sandboxes WHERE project_id = $1").bind(f.project).fetch_one(&f.pool).await.unwrap();
    assert!(reason.is_some(), "why it was restarted is recorded");
    assert_eq!(f.due().await, vec![f.project], "the row is walked to restart the dev server");
    assert_eq!(f.reconcile().await, ReconcileOutcome::Settled);
    let row = f.row().await;
    assert_eq!((row.state, row.preview_url.as_deref()), (State::Running, Some(format!("{PREVIEW}/restarted").as_str())));
    assert_eq!(f.steps.count("preview"), 2, "restarted through the preview start");
    assert_eq!(f.calls_starting_with("create") + f.calls_starting_with("resume"), 1, "the sandbox itself is left alone");
    assert_eq!(f.open_sessions().await, 1);

    // It dies again at once: restarts are bounded, so this one waits.
    *f.steps.dev_server_dead.lock().unwrap() = true;
    f.recheck_alive().await;
    f.manager.check_alive().await;
    assert!(f.due().await.is_empty(), "no second restart within the gap");
    assert!(f.row().await.preview_url.is_some());

    // Past the gap it is restarted again.
    sqlx::query("UPDATE project_sandboxes SET devserver_restarted_at = now() - make_interval(secs => $2) WHERE project_id = $1")
        .bind(f.project)
        .bind(DEVSERVER_RESTART_GAP.as_secs_f64() + 1.0)
        .execute(&f.pool)
        .await
        .unwrap();
    f.recheck_alive().await;
    f.manager.check_alive().await;
    assert_eq!(f.due().await, vec![f.project]);
}

#[sqlx::test(migrations = "../../migrations")]
async fn reinstall_in_a_running_sandbox_restarts_the_dev_server(pool: PgPool) {
    let f = fixture(pool, GENEROUS_LIMIT).await;
    f.run().await;
    // An edit that doesn't touch the dependencies: the dev server keeps running.
    f.set_files_version(SYNCED_VERSION + 1).await;
    *f.steps.version.lock().unwrap() = Some(SYNCED_VERSION + 1);
    assert_eq!(f.reconcile().await, ReconcileOutcome::Settled);
    assert_eq!(f.steps.count("preview"), 1);

    // package.json changed, so the sync ran an install: the dev server is started again.
    *f.steps.installs.lock().unwrap() = true;
    *f.steps.preview.lock().unwrap() = Some(format!("{PREVIEW}/after-install"));
    f.set_files_version(SYNCED_VERSION + 2).await;
    *f.steps.version.lock().unwrap() = Some(SYNCED_VERSION + 2);
    assert_eq!(f.reconcile().await, ReconcileOutcome::Settled);
    assert_eq!(f.steps.count("preview"), 2, "restarted after the install");
    let calls = f.steps.calls();
    let deps = calls.iter().rposition(|c| c.starts_with("deps")).unwrap();
    let preview = calls.iter().rposition(|c| c.starts_with("preview")).unwrap();
    assert!(deps < preview, "the restart comes after the install: {calls:?}");
    let row = f.row().await;
    assert_eq!((row.state, row.synced_version), (State::Running, SYNCED_VERSION + 2));
    assert_eq!(row.preview_url.as_deref(), Some(format!("{PREVIEW}/after-install").as_str()), "the new address is saved");
    assert_eq!(row.last_error, None);
}
