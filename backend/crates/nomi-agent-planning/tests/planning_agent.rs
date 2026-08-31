use sqlx::PgPool;
use uuid::Uuid;

use nomi_agent_core::SubAgent;
use nomi_agent_planning::PlanningAgent;

async fn seed_user_and_session(pool: &PgPool) -> (Uuid, Uuid) {
    let user_id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(pool).await.unwrap();
    let org_id: Uuid = sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Acme') RETURNING id")
        .fetch_one(pool)
        .await
        .unwrap();
    let chat_id = Uuid::new_v4().to_string();
    let session_id: Uuid = sqlx::query_scalar(
        "INSERT INTO sessions (org_id, channel, chat_id) VALUES ($1, 'telegram', $2) RETURNING id",
    )
    .bind(org_id)
    .bind(chat_id)
    .fetch_one(pool)
    .await
    .unwrap();
    (user_id, session_id)
}

#[sqlx::test(migrations = "../../migrations")]
async fn create_project_without_s3_configured_returns_an_error(pool: PgPool) {
    let (user_id, session_id) = seed_user_and_session(&pool).await;
    let agent = PlanningAgent::new(None);
    let mut conn = pool.acquire().await.unwrap();

    let result = agent
        .execute_tool(
            &mut conn,
            session_id,
            Uuid::new_v4(),
            user_id,
            "create_project",
            serde_json::json!({"name": "Todo app", "description": "A simple todo list"}),
        )
        .await;

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not available"));
}

#[sqlx::test(migrations = "../../migrations")]
async fn write_plan_updates_an_existing_project(pool: PgPool) {
    let (user_id, session_id) = seed_user_and_session(&pool).await;
    let project_id: Uuid = sqlx::query_scalar(
        "INSERT INTO projects (user_id, session_id, name) VALUES ($1, $2, 'Todo app') RETURNING id",
    )
    .bind(user_id)
    .bind(session_id)
    .fetch_one(&pool)
    .await
    .unwrap();

    let agent = PlanningAgent::new(None);
    let mut conn = pool.acquire().await.unwrap();
    let result = agent
        .execute_tool(
            &mut conn,
            session_id,
            Uuid::new_v4(),
            user_id,
            "write_plan",
            serde_json::json!({"project_id": project_id.to_string(), "plan": "1. index.html\n2. app.js"}),
        )
        .await
        .unwrap();
    assert_eq!(result, "plan saved");

    let stored_plan: Option<String> = sqlx::query_scalar("SELECT plan FROM projects WHERE id = $1")
        .bind(project_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(stored_plan.as_deref(), Some("1. index.html\n2. app.js"));
}

#[sqlx::test(migrations = "../../migrations")]
async fn write_plan_for_another_users_project_is_rejected(pool: PgPool) {
    let (owner_id, session_id) = seed_user_and_session(&pool).await;
    let (other_user_id, _) = seed_user_and_session(&pool).await;
    let project_id: Uuid = sqlx::query_scalar(
        "INSERT INTO projects (user_id, session_id, name) VALUES ($1, $2, 'Todo app') RETURNING id",
    )
    .bind(owner_id)
    .bind(session_id)
    .fetch_one(&pool)
    .await
    .unwrap();

    let agent = PlanningAgent::new(None);
    let mut conn = pool.acquire().await.unwrap();
    let result = agent
        .execute_tool(
            &mut conn,
            session_id,
            Uuid::new_v4(),
            other_user_id,
            "write_plan",
            serde_json::json!({"project_id": project_id.to_string(), "plan": "malicious overwrite"}),
        )
        .await;
    assert!(result.is_err());
}

#[sqlx::test(migrations = "../../migrations")]
async fn unknown_tool_name_returns_an_error(pool: PgPool) {
    let (user_id, session_id) = seed_user_and_session(&pool).await;
    let agent = PlanningAgent::new(None);
    let mut conn = pool.acquire().await.unwrap();
    let result = agent.execute_tool(&mut conn, session_id, Uuid::new_v4(), user_id, "delete_everything", serde_json::json!({})).await;
    assert!(result.is_err());
}
