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
        .and(path_regex(r"^/orgs/[^/]+/apps/[^/]+-backend/envs/dev/publish$"))
        .respond_with(ResponseTemplate::new(202).set_body_json(json!({ "data": { "publish_id": "p1", "status": "running", "url": relay_url, "warnings": [] }, "error": null })))
        .mount(&server)
        .await;
    Mock::given(method("GET")).and(path_regex(r"^/orgs/[^/]+/apps/[^/]+-backend/envs/dev/publishes/p1$")).respond_with(envelope(outcome)).mount(&server).await;
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
            "environments": { "dev": { "url": "https://x", "anon_key": "a", "service_key": "s" }, "production": { "url": "https://x", "anon_key": "a", "service_key": "s" } }
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
