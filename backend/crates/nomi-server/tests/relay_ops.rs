use nomi_agent_core::relay_ops::{ProjectFile, RelayEnv, RelayOps, NOT_SET_UP};
use nomi_relay_client::RelayConfig;
use nomi_server::relay_ops::ServerRelayOps;
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

fn ops(pool: &PgPool, server: &MockServer) -> ServerRelayOps {
    ServerRelayOps::new(pool.clone(), reqwest::Client::new(), RelayConfig::from_values(Some(&server.uri()), Some("relay_control_key")))
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
        .respond_with(envelope(json!({ "id": relay_id })))
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
    Mock::given(method("PUT")).respond_with(envelope(json!({ "id": Uuid::new_v4() }))).expect(0).mount(&server).await;

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
    let member: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(&pool).await.unwrap();
    sqlx::query("INSERT INTO project_members (project_id, user_id, role) VALUES ($1, $2, 'member')")
        .bind(project_id)
        .bind(member)
        .execute(&pool)
        .await
        .unwrap();
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(format!("/control/projects/{project_id}/environments/production/migrations")))
        .respond_with(envelope(json!({ "applied": [] })))
        .expect(1)
        .mount(&server)
        .await;
    let ops = ops(&pool, &server);

    assert!(ops.apply_migrations(member, project_id, RelayEnv::Production, &[]).await.is_err());
    assert!(ops.apply_migrations(owner, project_id, RelayEnv::Production, &[]).await.is_ok());
}
