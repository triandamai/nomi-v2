//! The worker that reconciles due sandbox rows, the idle sweep, and the sandbox check-in.

mod support;

use std::sync::Arc;

use async_trait::async_trait;
use nomi_sandbox::{FakeProvider, PreviewUrl, SandboxError, SandboxId};
use nomi_server::configuration::{Rollout, SandboxConfig};
use nomi_server::sandbox_manager::{ProjectSteps, SandboxManager};
use nomi_server::sandbox_store::{self as store, Desired, State};
use nomi_server::sandbox_worker::{pass_mode, sandbox_check_in, PassMode, SandboxWorker, PROVIDER_NAME};
use sqlx::PgPool;
use support::App;
use uuid::Uuid;

const IDLE_STOP_SECS: u64 = 600;
const GENEROUS_LIMIT: u32 = 1000;
/// A check-in this old is stale against `IDLE_STOP_SECS`.
const STALE_AGE_SECS: i64 = 3600;

struct OkSteps;

#[async_trait]
impl ProjectSteps for OkSteps {
    async fn sync(&self, _: Uuid, _: &SandboxId) -> Result<i64, String> {
        Ok(1)
    }
    async fn start_preview(&self, _: Uuid, _: &SandboxId) -> Result<PreviewUrl, String> {
        Ok(PreviewUrl { url: "https://preview.example.test".into() })
    }
}

fn config() -> SandboxConfig {
    SandboxConfig {
        api_base: String::new(),
        api_key: String::new(),
        size: nomi_sandbox::Size::Small,
        idle_stop_secs: IDLE_STOP_SECS,
        rollout: Rollout::All,
        starts_per_minute: GENEROUS_LIMIT,
        starts_per_hour: GENEROUS_LIMIT,
        monthly_minutes_per_user: GENEROUS_LIMIT,
        reaper_mode: nomi_server::configuration::ReaperMode::DryRun,
    }
}

fn worker(pool: &PgPool) -> (SandboxWorker, Arc<FakeProvider>) {
    let provider = Arc::new(FakeProvider::new());
    let manager = Arc::new(SandboxManager::new(provider.clone(), Arc::new(OkSteps), pool.clone(), config()));
    (SandboxWorker::new(pool.clone(), manager, IDLE_STOP_SECS), provider)
}

async fn project(pool: &PgPool, name: &str) -> Uuid {
    let app = App::new(pool.clone());
    let owner = app.person(&format!("{name}@worker.test")).await;
    app.project(&owner, name).await.0
}

async fn want_running(pool: &PgPool, p: Uuid) {
    let mut conn = pool.acquire().await.unwrap();
    store::want_running(&mut conn, p, PROVIDER_NAME).await.unwrap();
}

async fn row(pool: &PgPool, p: Uuid) -> store::SandboxRow {
    let mut conn = pool.acquire().await.unwrap();
    store::get(&mut conn, p).await.unwrap().unwrap()
}

async fn make_sandbox_project(pool: &PgPool, p: Uuid) {
    sqlx::query("UPDATE projects SET runtime = 'sandbox' WHERE id = $1").bind(p).execute(pool).await.unwrap();
}

async fn add_pending_run(pool: &PgPool, p: Uuid) {
    sqlx::query("INSERT INTO project_runs (project_id, command, status) VALUES ($1, 'npm install', 'pending')").bind(p).execute(pool).await.unwrap();
}

#[sqlx::test(migrations = "../../migrations")]
async fn check_in_for_a_sandbox_project_never_claims_runs(pool: PgPool) {
    let p = project(&pool, "claims").await;
    make_sandbox_project(&pool, p).await;
    add_pending_run(&pool, p).await;
    let mut conn = pool.acquire().await.unwrap();
    let version = sandbox_check_in(&mut conn, p, true).await.unwrap();
    assert!(version.is_some());
    let status: String = sqlx::query_scalar("SELECT status FROM project_runs WHERE project_id = $1").bind(p).fetch_one(&pool).await.unwrap();
    assert_eq!(status, "pending");
}

#[sqlx::test(migrations = "../../migrations")]
async fn check_in_marks_desired_running(pool: PgPool) {
    let p = project(&pool, "desired").await;
    make_sandbox_project(&pool, p).await;
    let mut conn = pool.acquire().await.unwrap();
    sandbox_check_in(&mut conn, p, true).await.unwrap();
    assert_eq!(row(&pool, p).await.desired, Desired::Running);
    let seen: bool = sqlx::query_scalar("SELECT runner_seen_at IS NOT NULL FROM projects WHERE id = $1").bind(p).fetch_one(&pool).await.unwrap();
    assert!(seen);
}

#[sqlx::test(migrations = "../../migrations")]
async fn check_in_leaves_browser_projects_and_a_disabled_runtime_alone(pool: PgPool) {
    let browser = project(&pool, "browser").await;
    let sandboxed = project(&pool, "off").await;
    make_sandbox_project(&pool, sandboxed).await;
    let mut conn = pool.acquire().await.unwrap();
    assert!(sandbox_check_in(&mut conn, browser, true).await.unwrap().is_none());
    assert!(sandbox_check_in(&mut conn, sandboxed, false).await.unwrap().is_none());
    assert!(store::get(&mut conn, browser).await.unwrap().is_none());
    assert!(store::get(&mut conn, sandboxed).await.unwrap().is_none());
}

#[sqlx::test(migrations = "../../migrations")]
async fn sweep_stops_only_idle_projects(pool: PgPool) {
    let idle = project(&pool, "idle").await;
    let busy = project(&pool, "busy").await;
    for p in [idle, busy] {
        want_running(&pool, p).await;
    }
    sqlx::query("UPDATE projects SET runner_seen_at = now() - make_interval(secs => $2) WHERE id = $1").bind(idle).bind(STALE_AGE_SECS as f64).execute(&pool).await.unwrap();
    sqlx::query("UPDATE projects SET runner_seen_at = now() WHERE id = $1").bind(busy).execute(&pool).await.unwrap();
    let (worker, _) = worker(&pool);
    worker.sweep_idle().await;
    assert_eq!(row(&pool, idle).await.desired, Desired::Stopped);
    assert_eq!(row(&pool, busy).await.desired, Desired::Running);
}

#[sqlx::test(migrations = "../../migrations")]
async fn worker_reconciles_due_rows_and_skips_not_yet_due(pool: PgPool) {
    let due = project(&pool, "due").await;
    let later = project(&pool, "later").await;
    for p in [due, later] {
        want_running(&pool, p).await;
    }
    sqlx::query("UPDATE project_sandboxes SET retry_at = now() + interval '1 hour' WHERE project_id = $1").bind(later).execute(&pool).await.unwrap();
    let (worker, _) = worker(&pool);
    assert_eq!(worker.tick().await, 1);
    assert_eq!(row(&pool, due).await.state, State::Running);
    assert_eq!(row(&pool, later).await.state, State::None);
}

#[sqlx::test(migrations = "../../migrations")]
async fn worker_survives_a_provider_error_and_continues(pool: PgPool) {
    let a = project(&pool, "a").await;
    let b = project(&pool, "b").await;
    for p in [a, b] {
        want_running(&pool, p).await;
    }
    let (worker, provider) = worker(&pool);
    provider.fail_next(SandboxError::Refused("nope".into()));
    assert_eq!(worker.tick().await, 2);
    let states = [row(&pool, a).await.state, row(&pool, b).await.state];
    assert_eq!(states.iter().filter(|s| **s == State::Running).count(), 1, "{states:?}");
    // The failed row is backed off; the loop keeps going and has nothing more to do.
    assert_eq!(worker.tick().await, 0);
}

#[sqlx::test(migrations = "../../migrations")]
async fn dispatch_runs_executes_waiting_commands_of_running_sandbox_projects(pool: PgPool) {
    let up = project(&pool, "up").await;
    let down = project(&pool, "down").await;
    for p in [up, down] {
        make_sandbox_project(&pool, p).await;
        want_running(&pool, p).await;
        sqlx::query("INSERT INTO project_runs (project_id, command, args) VALUES ($1, 'npm', '[\"install\"]')").bind(p).execute(&pool).await.unwrap();
    }
    let (worker, provider) = worker(&pool);
    let steps = Arc::new(nomi_server::sandbox_steps::ServerSteps::new(pool.clone(), nomi_test_support::test_project_storage(), provider.clone()));
    let worker = worker.with_runs(steps);
    assert_eq!(worker.dispatch_runs().await, 0, "no sandbox is up yet");
    assert_eq!(worker.tick().await, 2);
    // Only `up` has its sandbox up: stop the other one's row.
    sqlx::query("UPDATE project_sandboxes SET state = 'stopped' WHERE project_id = $1").bind(down).execute(&pool).await.unwrap();
    assert_eq!(worker.dispatch_runs().await, 1);
    for _ in 0..200 {
        let status: String = sqlx::query_scalar("SELECT status FROM project_runs WHERE project_id = $1").bind(up).fetch_one(&pool).await.unwrap();
        if status == "done" {
            let other: String = sqlx::query_scalar("SELECT status FROM project_runs WHERE project_id = $1").bind(down).fetch_one(&pool).await.unwrap();
            assert_eq!(other, "pending");
            return;
        }
        tokio::time::sleep(std::time::Duration::from_millis(25)).await;
    }
    panic!("the run never finished");
}

fn creds() -> SandboxConfig {
    config()
}

#[sqlx::test(migrations = "../../migrations")]
async fn kill_switch_stops_new_starts_but_winds_down_running_sandboxes(pool: PgPool) {
    let up = project(&pool, "up").await;
    let waiting = project(&pool, "waiting").await;
    want_running(&pool, up).await;
    let (worker, provider) = worker(&pool);
    // Bring `up` up while the switch is still on.
    assert_eq!(worker.tick().await, 1);
    assert_eq!(row(&pool, up).await.state, State::Running);
    let creates_before = provider.calls().iter().filter(|c| c.starts_with("create") || c.starts_with("resume")).count();
    // A page asks for another sandbox, then the switch goes off.
    want_running(&pool, waiting).await;
    assert_eq!(worker.wind_down().await, 2);
    assert_eq!(row(&pool, up).await.desired, Desired::Stopped);
    assert_eq!(row(&pool, waiting).await.desired, Desired::Stopped);
    worker.tick().await;
    assert_eq!(row(&pool, up).await.state, State::Stopped);
    assert_eq!(row(&pool, waiting).await.state, State::None);
    let starts_after = provider.calls().iter().filter(|c| c.starts_with("create") || c.starts_with("resume")).count();
    assert_eq!(starts_after, creates_before, "{:?}", provider.calls());
}

#[sqlx::test(migrations = "../../migrations")]
async fn kill_switch_wind_down_leaves_already_stopped_rows_alone(pool: PgPool) {
    let p = project(&pool, "quiet").await;
    want_running(&pool, p).await;
    let (worker, _) = worker(&pool);
    worker.tick().await;
    worker.wind_down().await;
    worker.tick().await;
    assert_eq!(worker.wind_down().await, 0);
}

#[test]
fn kill_switch_off_without_credentials_does_nothing() {
    assert!(pass_mode(None, None).is_none());
    let (mode, _) = pass_mode(None, Some(creds())).unwrap();
    assert_eq!(mode, PassMode::StopOnly);
    let (mode, _) = pass_mode(Some(creds()), Some(creds())).unwrap();
    assert_eq!(mode, PassMode::On);
}

#[sqlx::test(migrations = "../../migrations")]
async fn stop_only_never_creates_even_if_a_row_wants_running(pool: PgPool) {
    let p = project(&pool, "racer").await;
    let (worker, provider) = worker(&pool);
    worker.set_mode(PassMode::StopOnly);
    assert_eq!(worker.wind_down().await, 0);
    // A heartbeat that slipped in after the wind-down.
    want_running(&pool, p).await;
    assert_eq!(worker.tick().await, 0);
    assert!(provider.calls().is_empty(), "{:?}", provider.calls());
    assert_eq!(row(&pool, p).await.state, State::None);
    // Even a row claimed some other way is not brought up.
    let row_now = row(&pool, p).await;
    let manager = Arc::new(SandboxManager::new(provider.clone(), Arc::new(OkSteps), pool.clone(), config()));
    manager.set_stop_only(true);
    manager.reconcile(row_now).await;
    assert!(provider.calls().is_empty(), "{:?}", provider.calls());
    assert_eq!(row(&pool, p).await.attempts, 0);
    // The next pass flips it to stopped.
    assert_eq!(worker.wind_down().await, 1);
}

#[sqlx::test(migrations = "../../migrations")]
async fn wake_and_wait_returns_unavailable_when_switch_off(pool: PgPool) {
    use nomi_agent_core::sandbox_ops::{SandboxOps, WakeOutcome};
    let p = project(&pool, "wake").await;
    make_sandbox_project(&pool, p).await;
    let ops = nomi_server::sandbox_ops::ServerSandboxOps::new(pool.clone());
    let outcome = ops.wake_and_wait(p, std::time::Duration::from_millis(50)).await;
    assert!(matches!(outcome, WakeOutcome::Failed(_)), "{outcome:?}");
    let mut conn = pool.acquire().await.unwrap();
    assert!(store::get(&mut conn, p).await.unwrap().is_none());
}
