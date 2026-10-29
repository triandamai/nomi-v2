//! A Relay project's data, for its data sheet and for Iris: read through Relay's control plane
//! (wiremock stands in for it), never from Shipyard's Postgres.
use nomi_agent_core::project_ops::{DataSource, ProjectOps};
use nomi_relay_client::RelayConfig;
use nomi_server::project_ops::ServerProjectOps;
use nomi_server::relay_connection::save;
use nomi_server::relay_ops::overview;
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;
use wiremock::matchers::{body_json, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const KEY: [u8; 32] = [3u8; 32];

async fn seed_project(pool: &PgPool, backend: &str) -> (Uuid, Uuid) {
    let user_id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(pool).await.unwrap();
    let org_id: Uuid = sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Acme') RETURNING id").fetch_one(pool).await.unwrap();
    let session_id: Uuid = sqlx::query_scalar("INSERT INTO sessions (org_id, channel, chat_id) VALUES ($1, 'telegram', $2) RETURNING id")
        .bind(org_id)
        .bind(Uuid::new_v4().to_string())
        .fetch_one(pool)
        .await
        .unwrap();
    let project_id: Uuid = sqlx::query_scalar("INSERT INTO projects (user_id, session_id, name, backend) VALUES ($1, $2, 'Todo', $3) RETURNING id")
        .bind(user_id)
        .bind(session_id)
        .bind(backend)
        .fetch_one(pool)
        .await
        .unwrap();
    (user_id, project_id)
}

fn envelope(data: serde_json::Value) -> ResponseTemplate {
    ResponseTemplate::new(200).set_body_json(json!({ "data": data, "error": null }))
}

async fn iris(pool: &PgPool, relay: &MockServer, user: Uuid) -> ServerProjectOps {
    save(pool, &KEY, &relay.uri(), Some("relay_control_key"), user).await.unwrap();
    ServerProjectOps::with_relay(pool.clone(), nomi_storage::ProjectStore::at(std::env::temp_dir()), reqwest::Client::new(), Some(KEY))
}

#[sqlx::test(migrations = "../../migrations")]
async fn iris_reads_a_relay_project_through_relay_and_only_reads(pool: PgPool) {
    let relay = MockServer::start().await;
    let (user, project) = seed_project(&pool, "relay").await;
    let env = format!("/control/projects/{project}/environments/production");
    Mock::given(method("GET")).and(path(format!("{env}/schema"))).respond_with(envelope(json!({ "tables": [{ "name": "task" }, { "name": "note" }] }))).mount(&relay).await;
    Mock::given(method("POST"))
        .and(path(format!("{env}/query")))
        .and(body_json(json!({ "sql": "select title from task", "access": "read_only" })))
        .respond_with(envelope(json!({ "columns": ["title"], "rows": [["milk"]], "truncated": false, "rows_affected": null, "took_ms": 2 })))
        .expect(1)
        .mount(&relay)
        .await;
    let ops = iris(&pool, &relay, user).await;

    let tables = ops.tables(user, project, DataSource::Production).await.unwrap();
    assert_eq!(tables.iter().map(|t| t.name.as_str()).collect::<Vec<_>>(), vec!["task", "note"]);
    let rows = ops.query(user, project, DataSource::Production, "select title from task;").await.unwrap();
    assert_eq!(rows.columns, vec!["title"]);
    assert_eq!(rows.rows, vec![vec![json!("milk")]]);
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_drizzle_project_never_asks_relay(pool: PgPool) {
    let relay = MockServer::start().await;
    Mock::given(method("POST")).respond_with(envelope(json!({}))).expect(0).mount(&relay).await;
    let (user, project) = seed_project(&pool, "drizzle").await;
    let ops = iris(&pool, &relay, user).await;

    let refused = ops.query(user, project, DataSource::Development, "select 1").await.unwrap_err();

    assert!(refused.contains("publishing isn't set up"), "it went to Shipyard's Postgres, not Relay: {refused}");
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_data_sheet_gets_the_environments_address_key_tables_and_sign_in(pool: PgPool) {
    let relay = MockServer::start().await;
    let project = Uuid::new_v4();
    let base = format!("/control/projects/{project}");
    let keys = json!({ "url": "https://todo-dev.api.relay.test", "anon_key": "dev-anon", "service_key": "dev-service" });
    Mock::given(method("GET"))
        .and(path(&base))
        .respond_with(envelope(json!({ "id": project, "slug": "todo", "tier": "free", "environments": { "dev": keys, "production": keys } })))
        .mount(&relay)
        .await;
    Mock::given(method("GET")).and(path(format!("{base}/environments/dev/schema"))).respond_with(envelope(json!({ "tables": [{ "name": "task" }], "buckets": {}, "realtime": ["task"], "functions": null }))).mount(&relay).await;
    Mock::given(method("GET")).and(path(format!("{base}/environments/dev/auth"))).respond_with(envelope(json!({ "email": true, "missing_secrets": [] }))).mount(&relay).await;
    let config = RelayConfig::from_values(Some(&relay.uri()), Some("relay_control_key")).unwrap();

    let answer = overview(&reqwest::Client::new(), &config, project, "dev").await.unwrap();

    assert_eq!(answer["backend"], "relay");
    assert_eq!(answer["url"], "https://todo-dev.api.relay.test");
    assert_eq!(answer["anon_key"], "dev-anon");
    assert!(answer.get("service_key").is_none() && !answer.to_string().contains("dev-service"), "the service key stays on the server");
    assert_eq!(answer["tables"][0]["name"], "task");
    assert_eq!(answer["realtime"], json!(["task"]));
    assert_eq!(answer["auth"]["email"], true);
    let _ = pool;
}
