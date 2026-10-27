//! Making a project's own Relay on Shipyard (a mock here), and the Relay it starts (another):
//! the steps run in order, each only once, and a failure says where it stopped and can be tried
//! again from there.
use nomi_server::project_relays::{self, FAILED, READY};
use nomi_server::relay_backend::{run, Deps};
use nomi_shipyard::ShipyardConfig;
use serde_json::{json, Value};
use sqlx::PgPool;
use uuid::Uuid;
use wiremock::matchers::{method, path, path_regex};
use wiremock::{Mock, MockServer, ResponseTemplate};

const KEY: [u8; 32] = [3u8; 32];

async fn seed_project(pool: &PgPool) -> Uuid {
    let user_id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(pool).await.unwrap();
    let org_id: Uuid = sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Acme') RETURNING id").fetch_one(pool).await.unwrap();
    let session_id: Uuid = sqlx::query_scalar("INSERT INTO sessions (org_id, channel, chat_id) VALUES ($1, 'telegram', $2) RETURNING id")
        .bind(org_id)
        .bind(Uuid::new_v4().to_string())
        .fetch_one(pool)
        .await
        .unwrap();
    sqlx::query_scalar("INSERT INTO projects (user_id, session_id, name, backend) VALUES ($1, $2, 'Todo', 'relay') RETURNING id")
        .bind(user_id)
        .bind(session_id)
        .fetch_one(pool)
        .await
        .unwrap()
}

fn envelope(data: Value) -> ResponseTemplate {
    ResponseTemplate::new(200).set_body_json(json!({ "data": data, "error": null }))
}

/// A Shipyard that accepts everything and answers the backend's publish with `outcome`.
async fn shipyard(relay_url: &str, outcome: Value) -> MockServer {
    let server = MockServer::start().await;
    Mock::given(method("PUT")).and(path_regex(r"^/orgs/[^/]+$")).respond_with(envelope(json!({}))).mount(&server).await;
    Mock::given(method("PUT")).and(path_regex(r"^/orgs/[^/]+/apps/[^/]+$")).respond_with(envelope(json!({}))).mount(&server).await;
    Mock::given(method("PUT")).and(path_regex(r"^/orgs/[^/]+/apps/[^/]+/secrets/[A-Z_]+$")).respond_with(envelope(json!({ "ref": "sec" }))).mount(&server).await;
    Mock::given(method("POST"))
        .and(path_regex(r"^/orgs/[^/]+/apps/[^/]+-backend/envs/(dev|production)/publish$"))
        .respond_with(ResponseTemplate::new(202).set_body_json(json!({ "data": { "publish_id": "p1", "status": "running", "url": relay_url, "warnings": [] }, "error": null })))
        .mount(&server)
        .await;
    Mock::given(method("GET")).and(path_regex(r"^/orgs/[^/]+/apps/[^/]+-backend/envs/(dev|production)/publishes/p1$")).respond_with(envelope(outcome)).mount(&server).await;
    server
}

async fn relay(project: Uuid) -> MockServer {
    let server = MockServer::start().await;
    Mock::given(method("GET")).and(path("/health")).respond_with(ResponseTemplate::new(200)).mount(&server).await;
    Mock::given(method("PUT")).and(path("/control/settings")).respond_with(envelope(json!({ "configured": true }))).expect(1..).mount(&server).await;
    Mock::given(method("PUT"))
        .and(path(format!("/control/projects/{project}")))
        .respond_with(envelope(json!({
            "id": project, "slug": "todo", "tier": "dedicated",
            "environments": { "dev": { "url": "https://x", "anon_key": "a", "service_key": "s" }, "production": { "url": "https://prod.backend.test", "anon_key": "prod-anon", "service_key": "prod-service" } }
        })))
        .expect(1..)
        .mount(&server)
        .await;
    server
}

fn deps<'a>(pool: &'a PgPool, http: &'a reqwest::Client, config: &'a ShipyardConfig) -> Deps<'a> {
    Deps { pool, http, shipyard: config, settings_key: &KEY }
}

fn config(server: &MockServer) -> ShipyardConfig {
    ShipyardConfig { base_url: server.uri(), key: "ship_int_manage".into(), reveal_key: None, webhook_secret: None }
}

async fn calls(server: &MockServer, verb: &str, ending: &str) -> Vec<Value> {
    server
        .received_requests()
        .await
        .unwrap()
        .into_iter()
        .filter(|r| r.method.as_str() == verb && r.url.path().ends_with(ending))
        .map(|r| serde_json::from_slice(&r.body).unwrap_or_else(|_| Value::String(String::from_utf8_lossy(&r.body).into_owned())))
        .collect()
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_backend_is_made_in_order_and_recorded(pool: PgPool) {
    let project = seed_project(&pool).await;
    let relay = relay(project).await;
    let shipyard = shipyard(&relay.uri(), json!({ "status": "succeeded", "urls": [relay.uri()] })).await;
    let http = reqwest::Client::new();

    run(&deps(&pool, &http, &config(&shipyard)), project, "dev", true).await;

    let stack = project_relays::stack(&pool, project, "dev").await.unwrap().unwrap();
    assert_eq!(stack.state, READY, "{stack:?}");
    assert_eq!(stack.url.as_deref(), Some(relay.uri().as_str()));

    // The control key Shipyard was given is the one Nomi keeps, and the rest are Shipyard's to make.
    let control = project_relays::control_key(&pool, &KEY, project, "dev").await.unwrap().unwrap();
    let key_calls = calls(&shipyard, "PUT", "/secrets/RELAY_CONTROL_KEY").await;
    assert_eq!(key_calls, vec![json!({ "value": control, "environment": "dev" })]);
    for name in ["RELAY_ENCRYPTION_KEY", "RELAY_PGRST_PASSWORD", "RELAY_FUNCTIONS_SECRET"] {
        assert_eq!(calls(&shipyard, "PUT", &format!("/secrets/{name}")).await, vec![json!({ "generate": "hex32", "environment": "dev" })], "{name}");
    }

    // Relay is told its own address and who may call it from a preview.
    let settings = &calls(&relay, "PUT", "/control/settings").await[0];
    assert_eq!(settings["public_domain"], "127.0.0.1");
    assert_eq!(settings["preview_origins"], json!([".webcontainer-api.io"]));
    let publish = &shipyard.received_requests().await.unwrap().into_iter().find(|r| r.url.path().ends_with("/envs/dev/publish")).unwrap();
    let body = String::from_utf8_lossy(&publish.body);
    assert!(body.contains(&project.to_string()) && body.contains("\"RELAY_ENV\":\"dev\""), "{body}");
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_failed_deploy_says_where_and_trying_again_keeps_what_was_made(pool: PgPool) {
    let project = seed_project(&pool).await;
    let relay = relay(project).await;
    let http = reqwest::Client::new();

    let broken = shipyard(&relay.uri(), json!({ "status": "failed", "error": "the image couldn't be pulled" })).await;
    run(&deps(&pool, &http, &config(&broken)), project, "dev", true).await;
    let failed = project_relays::stack(&pool, project, "dev").await.unwrap().unwrap();
    assert_eq!(failed.state, FAILED);
    assert_eq!(failed.step.as_deref(), Some("waiting for it to start"));
    assert!(failed.error.unwrap().contains("the image couldn't be pulled"));
    let control = project_relays::control_key(&pool, &KEY, project, "dev").await.unwrap().unwrap();
    assert!(project_relays::config_for(&pool, Some(&KEY), None, project, "dev").await.unwrap_err().contains("couldn't be set up"));

    // Trying again against a Shipyard that works: it ends ready, with the same control key, and the
    // secrets aren't made a second time (the encryption key would lock Relay out of its own state).
    let working = shipyard(&relay.uri(), json!({ "status": "succeeded", "urls": [relay.uri()] })).await;
    run(&deps(&pool, &http, &config(&working)), project, "dev", true).await;

    assert_eq!(project_relays::stack(&pool, project, "dev").await.unwrap().unwrap().state, READY);
    assert_eq!(project_relays::control_key(&pool, &KEY, project, "dev").await.unwrap().unwrap(), control);
    assert!(calls(&working, "PUT", "/secrets/RELAY_ENCRYPTION_KEY").await.is_empty());
    assert!(calls(&working, "PUT", "/secrets/RELAY_CONTROL_KEY").await.is_empty());
    assert_eq!(calls(&broken, "PUT", "/secrets/RELAY_ENCRYPTION_KEY").await.len(), 1);
}

// ---- Sleeping and waking ----

use nomi_server::relay_backend::{ensure_awake, sleep_idle};
use nomi_server::project_relays::SLEEPING;
use std::time::Duration;

const IDLE_AFTER: Duration = Duration::from_secs(3600);

/// A ready backend in `env`, last used `idle_hours` ago, in a project with an app on Shipyard.
async fn ready_backend(pool: &PgPool, env: &str, idle_hours: i64, url: &str) -> Uuid {
    let project = seed_project(pool).await;
    // Each project's app name is its own: they're unique.
    sqlx::query("UPDATE projects SET shipyard_app = 'todo-' || left(id::text, 8) WHERE id = $1").bind(project).execute(pool).await.unwrap();
    project_relays::begin(pool, &KEY, project, env, "control-key", "0.1.5").await.unwrap();
    project_relays::set_url(pool, project, env, url).await.unwrap();
    project_relays::set_state(pool, project, env, READY, None, None).await.unwrap();
    sqlx::query("UPDATE project_relays SET last_used_at = now() - make_interval(hours => $3) WHERE project_id = $1 AND env = $2")
        .bind(project)
        .bind(env)
        .bind(idle_hours as i32)
        .execute(pool)
        .await
        .unwrap();
    project
}

async fn scale_mock(replicas: u32, times: u64) -> MockServer {
    let server = MockServer::start().await;
    Mock::given(method("PATCH"))
        .and(path_regex(r"^/orgs/[^/]+/apps/todo-[0-9a-f]{8}-backend/envs/dev/scale$"))
        .and(wiremock::matchers::body_json(json!({ "replicas": replicas })))
        .respond_with(envelope(json!({ "replicas": replicas })))
        .expect(times)
        .mount(&server)
        .await;
    server
}

#[sqlx::test(migrations = "../../migrations")]
async fn an_idle_development_backend_is_scaled_to_zero_and_a_used_one_is_not(pool: PgPool) {
    let idle = ready_backend(&pool, "dev", 2, "https://a.test").await;
    let busy = ready_backend(&pool, "dev", 0, "https://b.test").await;
    let production = ready_backend(&pool, "production", 48, "https://c.test").await;
    let shipyard = scale_mock(0, 1).await;
    let http = reqwest::Client::new();

    let stopped = sleep_idle(&deps(&pool, &http, &config(&shipyard)), IDLE_AFTER).await.unwrap();

    assert_eq!(stopped, 1);
    let state = |project: Uuid, env: &'static str| {
        let pool = pool.clone();
        async move { project_relays::stack(&pool, project, env).await.unwrap().unwrap().state }
    };
    assert_eq!(state(idle, "dev").await, SLEEPING);
    assert_eq!(state(busy, "dev").await, READY);
    assert_eq!(state(production, "production").await, READY, "production never sleeps");
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_shipyard_that_cannot_scale_yet_leaves_the_backends_running(pool: PgPool) {
    let idle = ready_backend(&pool, "dev", 5, "https://a.test").await;
    let shipyard = MockServer::start().await;
    Mock::given(method("PATCH")).respond_with(ResponseTemplate::new(404).set_body_json(json!({ "data": null, "error": { "code": "NOT_FOUND", "message": "no such route" } }))).mount(&shipyard).await;
    let http = reqwest::Client::new();

    let outcome = sleep_idle(&deps(&pool, &http, &config(&shipyard)), IDLE_AFTER).await;

    assert!(outcome.unwrap_err().contains("can't scale"));
    assert_eq!(project_relays::stack(&pool, idle, "dev").await.unwrap().unwrap().state, READY);
}

#[sqlx::test(migrations = "../../migrations")]
async fn calling_a_backend_keeps_it_awake(pool: PgPool) {
    let project = ready_backend(&pool, "dev", 3, "https://a.test").await;

    project_relays::config_for(&pool, Some(&KEY), None, project, "dev").await.unwrap();

    let idle = project_relays::idle_dev(&pool, 3600).await.unwrap();
    assert!(!idle.contains(&project), "a call just used it");
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_sleeping_backend_is_woken_once_and_waited_for(pool: PgPool) {
    let relay = MockServer::start().await;
    Mock::given(method("GET")).and(path("/health")).respond_with(ResponseTemplate::new(200)).mount(&relay).await;
    let project = ready_backend(&pool, "dev", 2, &relay.uri()).await;
    project_relays::set_state(&pool, project, "dev", SLEEPING, None, None).await.unwrap();
    let shipyard = scale_mock(1, 1).await;
    let http = reqwest::Client::new();
    let config = config(&shipyard);

    // Two people open the project at once: one starts it, both are answered when it's up.
    let (one, two) = (deps(&pool, &http, &config), deps(&pool, &http, &config));
    let (first, second) = tokio::join!(ensure_awake(&one, project), ensure_awake(&two, project));

    first.unwrap();
    second.unwrap();
    assert_eq!(project_relays::stack(&pool, project, "dev").await.unwrap().unwrap().state, READY);
    assert!(project_relays::config_for(&pool, Some(&KEY), None, project, "dev").await.unwrap().is_some());
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_production_backend_gives_the_published_app_its_keys(pool: PgPool) {
    let project = seed_project(&pool).await;
    let relay = relay(project).await;
    let shipyard = shipyard(&relay.uri(), json!({ "status": "succeeded", "urls": [relay.uri()] })).await;
    let http = reqwest::Client::new();

    run(&deps(&pool, &http, &config(&shipyard)), project, "production", false).await;

    assert_eq!(project_relays::stack(&pool, project, "production").await.unwrap().unwrap().state, READY);
    // App-wide secrets on the app itself, which every publish binds by name.
    for (name, value) in [("NOMI_URL", "https://prod.backend.test"), ("NOMI_ANON_KEY", "prod-anon"), ("NOMI_SERVICE_KEY", "prod-service")] {
        assert_eq!(calls(&shipyard, "PUT", &format!("/secrets/{name}")).await, vec![json!({ "value": value, "environment": null })], "{name}");
    }
    let names: Vec<String> = sqlx::query_scalar("SELECT name FROM project_secrets WHERE project_id = $1 ORDER BY name").bind(project).fetch_all(&pool).await.unwrap();
    assert_eq!(names, vec!["NOMI_ANON_KEY", "NOMI_SERVICE_KEY", "NOMI_URL"]);
    // Production has no preview origins, and no functions runtime when the app has none.
    assert_eq!(calls(&relay, "PUT", "/control/settings").await[0]["preview_origins"], json!([]));
    let publish = shipyard.received_requests().await.unwrap().into_iter().find(|r| r.url.path().ends_with("/envs/production/publish")).unwrap();
    assert!(!String::from_utf8_lossy(&publish.body).contains("\"name\":\"functions\""));
}
