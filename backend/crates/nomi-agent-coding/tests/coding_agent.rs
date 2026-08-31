use sqlx::PgPool;
use uuid::Uuid;

use nomi_agent_coding::{guess_content_type, CodingAgent};
use nomi_agent_core::SubAgent;

async fn seed_project(pool: &PgPool) -> (Uuid, Uuid) {
    let user_id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(pool).await.unwrap();
    // sessions has no user_id column (migration 0004_sessions.sql: org_id, channel, chat_id) —
    // seed an org first, matching nomi-agent-supervisor's tests/supervisor_agent.rs::seed_session.
    // chat_id is randomized since (channel, chat_id) is unique and this helper may be called more
    // than once per test.
    let org_id: Uuid = sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Acme') RETURNING id").fetch_one(pool).await.unwrap();
    let session_id: Uuid = sqlx::query_scalar("INSERT INTO sessions (org_id, channel, chat_id) VALUES ($1, 'telegram', $2) RETURNING id")
        .bind(org_id)
        .bind(Uuid::new_v4().to_string())
        .fetch_one(pool)
        .await
        .unwrap();
    let project_id: Uuid = sqlx::query_scalar(
        "INSERT INTO projects (user_id, session_id, name) VALUES ($1, $2, 'Todo app') RETURNING id",
    )
    .bind(user_id)
    .bind(session_id)
    .fetch_one(pool)
    .await
    .unwrap();
    (user_id, project_id)
}

#[test]
fn guess_content_type_covers_common_extensions() {
    assert_eq!(guess_content_type("index.html"), "text/html");
    assert_eq!(guess_content_type("styles.css"), "text/css");
    assert_eq!(guess_content_type("app.js"), "application/javascript");
    assert_eq!(guess_content_type("data.json"), "application/json");
    assert_eq!(guess_content_type("README"), "text/plain");
}

#[sqlx::test(migrations = "../../migrations")]
async fn write_file_without_s3_configured_returns_an_error(pool: PgPool) {
    let (user_id, project_id) = seed_project(&pool).await;
    let agent = CodingAgent::new(None);
    let mut conn = pool.acquire().await.unwrap();

    let result = agent
        .execute_tool(
            &mut conn,
            Uuid::new_v4(),
            Uuid::new_v4(),
            user_id,
            "write_file",
            serde_json::json!({"project_id": project_id.to_string(), "path": "index.html", "content": "<h1>hi</h1>"}),
        )
        .await;
    assert!(result.is_err());
}

#[sqlx::test(migrations = "../../migrations")]
async fn write_file_for_a_project_you_do_not_own_is_rejected(pool: PgPool) {
    let (_owner_id, project_id) = seed_project(&pool).await;
    let other_user_id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(&pool).await.unwrap();
    let agent = CodingAgent::new(None);
    let mut conn = pool.acquire().await.unwrap();

    let result = agent
        .execute_tool(
            &mut conn,
            Uuid::new_v4(),
            Uuid::new_v4(),
            other_user_id,
            "write_file",
            serde_json::json!({"project_id": project_id.to_string(), "path": "index.html", "content": "<h1>hi</h1>"}),
        )
        .await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "project not found");
}

#[sqlx::test(migrations = "../../migrations")]
async fn list_files_on_an_empty_project_says_so(pool: PgPool) {
    let (user_id, project_id) = seed_project(&pool).await;
    let agent = CodingAgent::new(None);
    let mut conn = pool.acquire().await.unwrap();

    let result = agent
        .execute_tool(&mut conn, Uuid::new_v4(), Uuid::new_v4(), user_id, "list_files", serde_json::json!({"project_id": project_id.to_string()}))
        .await
        .unwrap();
    assert_eq!(result, "no files yet");
}

#[sqlx::test(migrations = "../../migrations")]
async fn unknown_tool_name_returns_an_error(pool: PgPool) {
    let (user_id, project_id) = seed_project(&pool).await;
    let agent = CodingAgent::new(None);
    let mut conn = pool.acquire().await.unwrap();
    let result = agent
        .execute_tool(&mut conn, Uuid::new_v4(), Uuid::new_v4(), user_id, "run_shell", serde_json::json!({"project_id": project_id.to_string()}))
        .await;
    assert!(result.is_err());
}
