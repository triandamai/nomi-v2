//! Starting a project's dev server inside its sandbox: install, start, wait, expose.

mod support;

use std::sync::Arc;
use std::time::Duration;

use nomi_relay_client::RelayConfig;
use nomi_sandbox::{CreateSpec, ExecOutput, FakeProvider, ResumeSpec, SandboxId, SandboxProvider, Size};
use nomi_server::sandbox_devserver::DevServerTiming;
use nomi_server::sandbox_manager::{ProjectSteps, StepError};
use nomi_server::sandbox_steps::ServerSteps;
use nomi_server::sandbox_store as store;
use serde_json::json;
use sqlx::PgPool;
use support::App;
use uuid::Uuid;
use wiremock::matchers::{method, path, path_regex};
use wiremock::{Mock, MockServer, ResponseTemplate};

const PROVIDER: &str = "boat";
/// A readiness wait short enough for a test that is meant to time out.
const SHORT_READY: Duration = Duration::from_millis(150);
/// The pause between readiness checks in tests.
const FAST_POLL: Duration = Duration::from_millis(10);
/// The room a test's readiness wait gets when the server is meant to come up.
const ROOMY_READY: Duration = Duration::from_secs(5);

struct Fixture {
    pool: PgPool,
    provider: Arc<FakeProvider>,
    steps: ServerSteps,
    project: Uuid,
    owner: Uuid,
    sandbox: SandboxId,
}

fn out(code: i32, stdout: &str) -> ExecOutput {
    ExecOutput { exit_code: Some(code), stdout: stdout.to_string(), stderr: String::new() }
}

async fn fixture(pool: PgPool, ready: Duration) -> Fixture {
    let app = App::new(pool.clone());
    let owner = app.person(&format!("owner-{}@dev.test", Uuid::new_v4())).await;
    let project = app.project(&owner, "dev").await.0;
    let provider = Arc::new(FakeProvider::new());
    let sandbox = provider
        .create(CreateSpec { size: Size::Small, ttl: nomi_sandbox::Ttl::ProviderDefault, from_snapshot: None, idempotency_key: "k".into(), labels: Default::default() })
        .await
        .unwrap()
        .id;
    let mut conn = pool.acquire().await.unwrap();
    store::want_running(&mut conn, project, PROVIDER).await.unwrap();
    sqlx::query("UPDATE project_sandboxes SET sandbox_id = $2 WHERE project_id = $1").bind(project).bind(&sandbox.0).execute(&mut *conn).await.unwrap();
    drop(conn);
    // The page answers once curl is asked for it.
    provider.on_exec("curl", out(0, "200"));
    let steps = ServerSteps::new(pool.clone(), nomi_test_support::test_project_storage(), provider.clone())
        .with_devserver_timing(DevServerTiming { ready_timeout: ready, poll_interval: FAST_POLL });
    Fixture { pool, provider, steps, project, owner: owner.id, sandbox }
}

impl Fixture {
    fn calls(&self, needle: &str) -> Vec<String> {
        self.provider.calls().into_iter().filter(|c| c.contains(needle)).collect()
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn starts_dev_server_after_install_and_exposes_port(pool: PgPool) {
    let f = fixture(pool, ROOMY_READY).await;
    // No node_modules in the sandbox yet.
    f.provider.on_exec("test -d", out(1, ""));
    let url = f.steps.ensure_dev_server(f.project, &f.sandbox).await.unwrap();
    assert!(url.url.contains("5173"), "the dev port is exposed");
    let calls = f.provider.calls();
    let install = calls.iter().position(|c| c.contains("npm install")).expect("installs");
    let start = calls.iter().position(|c| c.starts_with("exec_detached") && c.contains("npm run dev")).expect("starts the dev server");
    let expose = calls.iter().position(|c| c.starts_with("expose_port")).expect("exposes the port");
    assert!(install < start && start < expose);
    assert!(calls[start].contains("--host 0.0.0.0") && calls[start].contains("--port 5173 --strictPort"));
    assert!(calls[start].contains("NOMI_PREVIEW='1'") && calls[start].contains("NODE_NO_WARNINGS='1'"));
}

#[sqlx::test(migrations = "../../migrations")]
async fn skips_install_when_lockfile_unchanged(pool: PgPool) {
    let f = fixture(pool, ROOMY_READY).await;
    f.steps.ensure_dev_server(f.project, &f.sandbox).await.unwrap();
    assert_eq!(f.calls("npm install").len(), 1, "first start installs: nothing is recorded yet");
    f.steps.ensure_dev_server(f.project, &f.sandbox).await.unwrap();
    assert_eq!(f.calls("npm install").len(), 1, "same dependency files and node_modules present: no second install");
    // A changed lockfile installs again.
    let storage = nomi_test_support::test_project_storage();
    let mut conn = f.pool.acquire().await.unwrap();
    nomi_agent_coding::files::save_bytes(&mut conn, &storage, f.project, "package-lock.json", b"{}").await.unwrap();
    drop(conn);
    f.steps.ensure_dev_server(f.project, &f.sandbox).await.unwrap();
    assert_eq!(f.calls("npm install").len(), 2);
}

#[sqlx::test(migrations = "../../migrations")]
async fn restarts_dev_server_after_resume(pool: PgPool) {
    let f = fixture(pool, ROOMY_READY).await;
    f.steps.ensure_dev_server(f.project, &f.sandbox).await.unwrap();
    f.provider.stop(&f.sandbox, false).await.unwrap();
    f.provider.resume(&f.sandbox, ResumeSpec::default()).await.unwrap();
    let url = f.steps.ensure_dev_server(f.project, &f.sandbox).await.unwrap();
    assert_eq!(f.calls("exec_detached").len(), 2, "the dev server is started again");
    assert!(url.url.contains("-g1."), "the new preview address is the one returned");
    assert_eq!(f.calls("npm install").len(), 1, "files survived the stop, so nothing is installed again");
}

#[sqlx::test(migrations = "../../migrations")]
async fn readiness_timeout_marks_row_error_with_log_tail(pool: PgPool) {
    let f = fixture(pool, SHORT_READY).await;
    // The page never answers, and the dev server printed why.
    f.provider.on_exec("curl", out(0, "000"));
    f.provider.set_process_logs("", "Error: Cannot find module 'left-pad'");
    let err = f.steps.ensure_dev_server(f.project, &f.sandbox).await.unwrap_err();
    assert!(err.to_string().contains("Cannot find module 'left-pad'"), "{err}");
    let mut conn = f.pool.acquire().await.unwrap();
    let saved = store::log_tail(&mut conn, f.project).await.unwrap();
    assert!(saved.contains("left-pad"), "the tail is saved on the row, for whichever process serves the log: {saved}");
}

#[sqlx::test(migrations = "../../migrations")]
async fn another_nomi_process_kills_the_dev_server_this_one_started(pool: PgPool) {
    let f = fixture(pool, ROOMY_READY).await;
    f.steps.ensure_dev_server(f.project, &f.sandbox).await.unwrap();
    let first: Option<String> = sqlx::query_scalar("SELECT devserver_process FROM project_sandboxes WHERE project_id = $1").bind(f.project).fetch_one(&f.pool).await.unwrap();
    let first = first.expect("the dev server's process is recorded on the row");
    // A fresh ServerSteps shares nothing in memory with the first, like the worker binary next to the API.
    let other = ServerSteps::new(f.pool.clone(), nomi_test_support::test_project_storage(), f.provider.clone())
        .with_devserver_timing(DevServerTiming { ready_timeout: ROOMY_READY, poll_interval: FAST_POLL });
    other.ensure_dev_server(f.project, &f.sandbox).await.unwrap();
    assert!(f.provider.calls().iter().any(|c| c.starts_with("kill_process") && c.contains(&first)), "{:?}", f.provider.calls());
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_changed_package_json_is_installed_again_in_a_running_sandbox(pool: PgPool) {
    let f = fixture(pool, ROOMY_READY).await;
    f.steps.ensure_dev_server(f.project, &f.sandbox).await.unwrap();
    assert_eq!(f.calls("npm install").len(), 1);
    ProjectSteps::update_dependencies(&f.steps, f.project, &f.sandbox).await.unwrap();
    assert_eq!(f.calls("npm install").len(), 1, "nothing changed: no install");
    let storage = nomi_test_support::test_project_storage();
    let mut conn = f.pool.acquire().await.unwrap();
    nomi_agent_coding::files::save_bytes(&mut conn, &storage, f.project, "package.json", br#"{"dependencies":{"zod":"3"}}"#).await.unwrap();
    drop(conn);
    ProjectSteps::update_dependencies(&f.steps, f.project, &f.sandbox).await.unwrap();
    assert_eq!(f.calls("npm install").len(), 2, "package.json changed: installed again");
    assert_eq!(f.calls("exec_detached").len(), 1, "the dev server keeps running");
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_failed_install_is_a_usable_failure_handed_to_koda(pool: PgPool) {
    let f = fixture(pool, ROOMY_READY).await;
    f.provider.on_exec("test -d", out(1, ""));
    f.provider.on_exec("npm install", ExecOutput { exit_code: Some(1), stdout: String::new(), stderr: "ERESOLVE".into() });
    match ProjectSteps::start_preview(&f.steps, f.project, &f.sandbox).await {
        Err(StepError::Usable(why)) => assert!(why.contains("ERESOLVE"), "{why}"),
        other => panic!("{other:?}"),
    }
    let tasks: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM agent_delegations").fetch_one(&f.pool).await.unwrap();
    assert_eq!(tasks, 1, "the failed install went to Koda");
}

#[sqlx::test(migrations = "../../migrations")]
async fn preview_env_includes_relay_variables_for_projects_with_a_backend(pool: PgPool) {
    let mut f = fixture(pool, ROOMY_READY).await;
    sqlx::query("UPDATE projects SET backend = 'relay' WHERE id = $1").bind(f.project).execute(&f.pool).await.unwrap();
    let server = MockServer::start().await;
    let backend = json!({
        "id": f.project, "slug": "todo", "tier": "free",
        "environments": {
            "dev": { "url": "https://todo-dev.api.relay.test", "anon_key": "dev-anon", "service_key": "dev-service" },
            "production": { "url": "https://todo.api.relay.test", "anon_key": "prod-anon", "service_key": "prod-service" }
        }
    });
    Mock::given(method("GET"))
        .and(path(format!("/control/projects/{}", f.project)))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "data": backend, "error": null })))
        .mount(&server)
        .await;
    let config = RelayConfig::from_values(Some(&server.uri()), Some("relay_control_key")).unwrap();
    f.steps = ServerSteps::new(f.pool.clone(), nomi_test_support::test_project_storage(), f.provider.clone())
        .with_devserver_timing(DevServerTiming { ready_timeout: ROOMY_READY, poll_interval: FAST_POLL })
        .with_relay_config(config);
    f.steps.ensure_dev_server(f.project, &f.sandbox).await.unwrap();
    let start = f.calls("exec_detached").pop().unwrap();
    for expected in [
        "PUBLIC_NOMI_URL='https://todo-dev.api.relay.test'",
        "PUBLIC_NOMI_KEY='dev-anon'",
        "NOMI_URL='https://todo-dev.api.relay.test'",
        "NOMI_ANON_KEY='dev-anon'",
        "NOMI_SERVICE_KEY='dev-service'",
        "NOMI_PREVIEW='1'",
    ] {
        assert!(start.contains(expected), "missing {expected} in the start command");
    }
    let _ = f.owner;
}

#[sqlx::test(migrations = "../../migrations")]
async fn sleeping_relay_is_woken_before_the_dev_server_starts(pool: PgPool) {
    use nomi_server::project_relays::{self, READY, SLEEPING};
    let mut f = fixture(pool, ROOMY_READY).await;
    sqlx::query("UPDATE projects SET backend = 'relay', shipyard_app = 'todo-' || left(id::text, 8) WHERE id = $1").bind(f.project).execute(&f.pool).await.unwrap();
    let key = [3u8; 32];
    let relay = MockServer::start().await;
    let backend = json!({
        "id": f.project, "slug": "todo", "tier": "free",
        "environments": { "dev": { "url": "https://todo-dev.api.relay.test", "anon_key": "dev-anon", "service_key": "dev-service" } }
    });
    Mock::given(method("GET")).and(path("/health")).respond_with(ResponseTemplate::new(200)).mount(&relay).await;
    Mock::given(method("GET"))
        .and(path(format!("/control/projects/{}", f.project)))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "data": backend, "error": null })))
        .mount(&relay)
        .await;
    project_relays::begin(&f.pool, &key, f.project, "dev", "control-key", "0.1.5").await.unwrap();
    project_relays::set_url(&f.pool, f.project, "dev", &relay.uri()).await.unwrap();
    project_relays::set_state(&f.pool, f.project, "dev", SLEEPING, None, None).await.unwrap();
    let shipyard = MockServer::start().await;
    Mock::given(method("PATCH"))
        .and(path_regex(r"^/orgs/[^/]+/apps/todo-[0-9a-f]{8}/envs/dev/scale$"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "data": { "replicas": 1 }, "error": null })))
        .expect(1)
        .mount(&shipyard)
        .await;
    let config = RelayConfig::from_values(Some(&relay.uri()), Some("relay_control_key")).unwrap();
    f.steps = ServerSteps::new(f.pool.clone(), nomi_test_support::test_project_storage(), f.provider.clone())
        .with_devserver_timing(DevServerTiming { ready_timeout: ROOMY_READY, poll_interval: FAST_POLL })
        .with_relay_config(config)
        .with_relay_access(Some(key), reqwest::Client::new())
        .with_shipyard(Some(nomi_shipyard::ShipyardConfig { base_url: shipyard.uri(), key: "ship_int_manage".into(), reveal_key: None, webhook_secret: None }));
    f.steps.ensure_dev_server(f.project, &f.sandbox).await.unwrap();
    assert_eq!(project_relays::stack(&f.pool, f.project, "dev").await.unwrap().unwrap().state, READY);
    assert!(f.calls("exec_detached").pop().unwrap().contains("PUBLIC_NOMI_URL"));
}
