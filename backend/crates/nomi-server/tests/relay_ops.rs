use nomi_agent_core::relay_ops::{ProjectFile, RelayEnv, RelayOps, NOT_SET_UP};
use nomi_relay_client::RelayConfig;
use nomi_server::relay_ops::{preview_access, ServerRelayOps};
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;
use wiremock::matchers::{body_json, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// A user, and a project of theirs named "Todo".
async fn seed_project(pool: &PgPool) -> (Uuid, Uuid) {
    let user_id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(pool).await.unwrap();
    let org_id: Uuid = sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Acme') RETURNING id").fetch_one(pool).await.unwrap();
    let session_id: Uuid = sqlx::query_scalar("INSERT INTO sessions (org_id, channel, chat_id) VALUES ($1, 'telegram', $2) RETURNING id")
        .bind(org_id)
        .bind(Uuid::new_v4().to_string())
        .fetch_one(pool)
        .await
        .unwrap();
    let project_id: Uuid = sqlx::query_scalar("INSERT INTO projects (user_id, session_id, name) VALUES ($1, $2, 'Todo') RETURNING id")
        .bind(user_id)
        .bind(session_id)
        .fetch_one(pool)
        .await
        .unwrap();
    (user_id, project_id)
}

/// Someone in the project's organization who was added to the project as a member (not an admin):
/// they can open it.
async fn add_member(pool: &PgPool, project_id: Uuid) -> Uuid {
    let member: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(pool).await.unwrap();
    let org_id: Uuid = sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Team') RETURNING id").fetch_one(pool).await.unwrap();
    sqlx::query("UPDATE projects SET org_id = $1 WHERE id = $2").bind(org_id).bind(project_id).execute(pool).await.unwrap();
    sqlx::query("INSERT INTO memberships (org_id, user_id, role) VALUES ($1, $2, 'member')").bind(org_id).bind(member).execute(pool).await.unwrap();
    sqlx::query("INSERT INTO project_members (project_id, user_id, role) VALUES ($1, $2, 'member')")
        .bind(project_id)
        .bind(member)
        .execute(pool)
        .await
        .unwrap();
    member
}

fn ops(pool: &PgPool, server: &MockServer) -> ServerRelayOps {
    ServerRelayOps::new(pool.clone(), reqwest::Client::new(), RelayConfig::from_values(Some(&server.uri()), Some("relay_control_key")))
}

/// What Relay answers when it made (or already has) a project's backend.
fn backend(id: Uuid) -> serde_json::Value {
    json!({
        "id": id,
        "slug": "todo-4f2c1a",
        "tier": "free",
        "environments": {
            "dev": { "url": "https://todo-4f2c1a-dev.api.relay.test", "anon_key": "dev-anon", "service_key": "dev-service" },
            "production": { "url": "https://todo-4f2c1a.api.relay.test", "anon_key": "prod-anon", "service_key": "prod-service" }
        }
    })
}

fn envelope(data: serde_json::Value) -> ResponseTemplate {
    ResponseTemplate::new(200).set_body_json(json!({ "data": data, "error": null }))
}

#[sqlx::test(migrations = "../../migrations")]
async fn without_relay_configured_nothing_is_available(pool: PgPool) {
    let (user_id, project_id) = seed_project(&pool).await;
    let ops = ServerRelayOps::new(pool, reqwest::Client::new(), None);
    assert!(!ops.is_available());
    assert_eq!(ops.ensure_backend(user_id, project_id).await.unwrap_err(), NOT_SET_UP);
}

#[sqlx::test(migrations = "../../migrations")]
async fn ensure_backend_makes_it_on_relay_and_remembers_relay_s_id(pool: PgPool) {
    let (user_id, project_id) = seed_project(&pool).await;
    let server = MockServer::start().await;
    let relay_id = Uuid::new_v4();
    Mock::given(method("PUT"))
        .and(path(format!("/control/projects/{project_id}")))
        .and(body_json(json!({ "name": "Todo" })))
        .respond_with(envelope(backend(relay_id)))
        .expect(1)
        .mount(&server)
        .await;

    assert_eq!(ops(&pool, &server).ensure_backend(user_id, project_id).await.unwrap(), relay_id);
    let stored: Option<Uuid> = sqlx::query_scalar("SELECT relay_project_id FROM projects WHERE id = $1").bind(project_id).fetch_one(&pool).await.unwrap();
    assert_eq!(stored, Some(relay_id));
}

#[sqlx::test(migrations = "../../migrations")]
async fn someone_who_cant_open_the_project_reaches_nothing_on_relay(pool: PgPool) {
    let (_, project_id) = seed_project(&pool).await;
    let stranger: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(&pool).await.unwrap();
    let server = MockServer::start().await;
    Mock::given(method("PUT")).respond_with(envelope(backend(Uuid::new_v4()))).expect(0).mount(&server).await;

    assert!(ops(&pool, &server).ensure_backend(stranger, project_id).await.is_err());
}

#[sqlx::test(migrations = "../../migrations")]
async fn applied_migrations_are_named_and_a_failure_says_where(pool: PgPool) {
    let (user_id, project_id) = seed_project(&pool).await;
    let server = MockServer::start().await;
    let files = [ProjectFile { path: "nomi/migrations/0001_init.sql".into(), content: "create table task (id int);".into() }];
    Mock::given(method("POST"))
        .and(path(format!("/control/projects/{project_id}/environments/dev/migrations")))
        .respond_with(envelope(json!({ "applied": ["0001_init.sql"] })))
        .up_to_n_times(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path(format!("/control/projects/{project_id}/environments/dev/migrations")))
        .respond_with(ResponseTemplate::new(422).set_body_json(json!({
            "data": null,
            "error": { "code": "MIGRATION_FAILED", "message": "relation \"task\" already exists", "details": { "file": "0002_task.sql", "line": 3 } }
        })))
        .mount(&server)
        .await;
    let ops = ops(&pool, &server);

    let applied = ops.apply_migrations(user_id, project_id, RelayEnv::Dev, &files).await.unwrap();
    assert!(applied.contains("0001_init.sql"), "{applied}");
    let failed = ops.apply_migrations(user_id, project_id, RelayEnv::Dev, &files).await.unwrap_err();
    assert!(failed.contains("0002_task.sql") && failed.contains("line 3") && failed.contains("already exists"), "{failed}");
}

#[sqlx::test(migrations = "../../migrations")]
async fn only_an_admin_changes_production(pool: PgPool) {
    let (owner, project_id) = seed_project(&pool).await;
    let member = add_member(&pool, project_id).await;
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(format!("/control/projects/{project_id}/environments/production/migrations")))
        .respond_with(envelope(json!({ "applied": [] })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path(format!("/control/projects/{project_id}/environments/dev/migrations")))
        .respond_with(envelope(json!({ "applied": [] })))
        .expect(1)
        .mount(&server)
        .await;
    let ops = ops(&pool, &server);

    assert!(ops.apply_migrations(member, project_id, RelayEnv::Production, &[]).await.is_err());
    assert!(ops.apply_migrations(owner, project_id, RelayEnv::Production, &[]).await.is_ok());
    assert!(ops.apply_migrations(member, project_id, RelayEnv::Dev, &[]).await.is_ok());
}

#[sqlx::test(migrations = "../../migrations")]
async fn only_an_admin_reads_production(pool: PgPool) {
    let (owner, project_id) = seed_project(&pool).await;
    let member = add_member(&pool, project_id).await;
    let server = MockServer::start().await;
    Mock::given(method("GET")).respond_with(envelope(json!({ "tables": [], "types": "" }))).mount(&server).await;
    let ops = ops(&pool, &server);

    assert!(ops.describe_backend(member, project_id, RelayEnv::Production).await.is_err());
    assert!(ops.types(member, project_id, RelayEnv::Production).await.is_err());
    assert!(ops.describe_backend(member, project_id, RelayEnv::Dev).await.is_ok());
    assert!(ops.describe_backend(owner, project_id, RelayEnv::Production).await.is_ok());
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_production_keys_become_the_apps_secrets_on_shipyard(pool: PgPool) {
    let (user_id, project_id) = seed_project(&pool).await;
    let relay = MockServer::start().await;
    Mock::given(method("PUT")).and(path(format!("/control/projects/{project_id}"))).respond_with(envelope(backend(project_id))).mount(&relay).await;
    let shipyard = MockServer::start().await;
    Mock::given(method("PUT"))
        .and(wiremock::matchers::path_regex(r"^/orgs/[^/]+(/apps/[^/]+)?$"))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({ "data": { "created": true }, "error": null })))
        .mount(&shipyard)
        .await;
    Mock::given(method("PUT"))
        .and(wiremock::matchers::path_regex(r"/secrets/NOMI_[A-Z_]+$"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "data": {}, "error": null })))
        .expect(3)
        .mount(&shipyard)
        .await;
    let ops = ServerRelayOps::new(pool.clone(), reqwest::Client::new(), RelayConfig::from_values(Some(&relay.uri()), Some("relay_control_key")))
        .with_shipyard(nomi_test_support::test_project_storage(), nomi_shipyard::ShipyardConfig {
            base_url: shipyard.uri(),
            key: "ship_int_manage".into(),
            reveal_key: None,
            webhook_secret: None,
        });

    ops.ensure_backend(user_id, project_id).await.unwrap();

    let sent: Vec<(String, serde_json::Value)> = shipyard
        .received_requests()
        .await
        .unwrap()
        .iter()
        .filter(|r| r.url.path().contains("/secrets/"))
        .map(|r| (r.url.path().rsplit('/').next().unwrap().to_string(), serde_json::from_slice(&r.body).unwrap()))
        .collect();
    let value = |name: &str| sent.iter().find(|(n, _)| n == name).map(|(_, body)| body["value"].clone()).unwrap();
    assert_eq!(value("NOMI_URL"), json!("https://todo-4f2c1a.api.relay.test"));
    assert_eq!(value("NOMI_ANON_KEY"), json!("prod-anon"));
    assert_eq!(value("NOMI_SERVICE_KEY"), json!("prod-service"));
    let names: Vec<String> = sqlx::query_scalar("SELECT name FROM project_secrets WHERE project_id = $1 ORDER BY name").bind(project_id).fetch_all(&pool).await.unwrap();
    assert_eq!(names, vec!["NOMI_ANON_KEY", "NOMI_SERVICE_KEY", "NOMI_URL"]);
}

async fn set_backend(pool: &PgPool, project_id: Uuid, backend: &str) {
    sqlx::query("UPDATE projects SET backend = $1 WHERE id = $2").bind(backend).bind(project_id).execute(pool).await.unwrap();
}

fn publish_files() -> Vec<(String, Vec<u8>)> {
    vec![
        ("package.json".into(), b"{}".to_vec()),
        ("nomi/migrations/0001_init.sql".into(), b"create table t (id int);".to_vec()),
        ("nomi/backend.json".into(), b"{}".to_vec()),
    ]
}

#[sqlx::test(migrations = "../../migrations")]
async fn publishing_a_relay_project_applies_its_production_migrations_first(pool: PgPool) {
    let (_, project_id) = seed_project(&pool).await;
    set_backend(&pool, project_id, "relay").await;
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(format!("/control/projects/{project_id}/environments/production/migrations")))
        .and(body_json(json!({ "files": [{ "path": "nomi/migrations/0001_init.sql", "content": "create table t (id int);" }] })))
        .respond_with(envelope(json!({ "applied": ["0001_init.sql"] })))
        .expect(1)
        .mount(&server)
        .await;
    let config = RelayConfig::from_values(Some(&server.uri()), Some("relay_control_key"));

    nomi_server::relay_ops::apply_before_publish(&pool, &reqwest::Client::new(), config.as_ref(), project_id, &publish_files()).await.unwrap();
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_failed_production_migration_stops_the_publish_and_says_where(pool: PgPool) {
    let (_, project_id) = seed_project(&pool).await;
    set_backend(&pool, project_id, "relay").await;
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(422).set_body_json(json!({
            "data": null,
            "error": { "code": "MIGRATION_FAILED", "message": "column \"x\" does not exist", "details": { "file": "0002_x.sql", "line": 4 } }
        })))
        .mount(&server)
        .await;
    let config = RelayConfig::from_values(Some(&server.uri()), Some("relay_control_key"));

    let (status, message) = nomi_server::relay_ops::apply_before_publish(&pool, &reqwest::Client::new(), config.as_ref(), project_id, &publish_files()).await.unwrap_err();

    assert_eq!(status.as_u16(), 422);
    assert!(message.contains("0002_x.sql line 4"), "{message}");
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_drizzle_project_publishes_without_relay_and_a_relay_one_needs_it(pool: PgPool) {
    let (_, project_id) = seed_project(&pool).await;
    let http = reqwest::Client::new();
    assert!(nomi_server::relay_ops::apply_before_publish(&pool, &http, None, project_id, &publish_files()).await.is_ok());
    set_backend(&pool, project_id, "relay").await;
    assert!(nomi_server::relay_ops::apply_before_publish(&pool, &http, None, project_id, &publish_files()).await.is_err());
}

fn relay_config(server: &MockServer) -> RelayConfig {
    RelayConfig::from_values(Some(&server.uri()), Some("relay_control_key")).unwrap()
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_preview_gets_dev_access_and_only_admins_get_the_service_key(pool: PgPool) {
    let (owner, project_id) = seed_project(&pool).await;
    sqlx::query("UPDATE projects SET backend = 'relay' WHERE id = $1").bind(project_id).execute(&pool).await.unwrap();
    let member = add_member(&pool, project_id).await;
    let server = MockServer::start().await;
    Mock::given(method("GET")).and(path(format!("/control/projects/{project_id}"))).respond_with(envelope(backend(project_id))).mount(&server).await;
    let http = reqwest::Client::new();
    let config = relay_config(&server);

    let admin = preview_access(&pool, &http, Some(&config), project_id, owner).await.unwrap();
    assert_eq!(admin.url, "https://todo-4f2c1a-dev.api.relay.test");
    assert_eq!(admin.anon_key, "dev-anon");
    assert_eq!(admin.service_key.as_deref(), Some("dev-service"));

    let member = preview_access(&pool, &http, Some(&config), project_id, member).await.unwrap();
    assert_eq!(member.anon_key, "dev-anon");
    assert_eq!(member.service_key, None);
}

#[sqlx::test(migrations = "../../migrations")]
async fn strangers_and_projects_off_relay_get_no_preview_access(pool: PgPool) {
    let (owner, project_id) = seed_project(&pool).await;
    let stranger: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(&pool).await.unwrap();
    let server = MockServer::start().await;
    Mock::given(method("GET")).respond_with(envelope(backend(project_id))).expect(0).mount(&server).await;
    let http = reqwest::Client::new();
    let config = relay_config(&server);

    let off_relay = preview_access(&pool, &http, Some(&config), project_id, owner).await.unwrap_err();
    assert_eq!(off_relay.0, axum::http::StatusCode::NOT_FOUND);
    sqlx::query("UPDATE projects SET backend = 'relay' WHERE id = $1").bind(project_id).execute(&pool).await.unwrap();
    let stranger = preview_access(&pool, &http, Some(&config), project_id, stranger).await.unwrap_err();
    assert_eq!(stranger.0, axum::http::StatusCode::NOT_FOUND);
}
