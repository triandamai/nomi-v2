use sqlx::PgPool;
use uuid::Uuid;

use nomi_agent_core::SubAgent;
use nomi_agent_supervisor::SupervisorAgent;

async fn seed_session(pool: &PgPool) -> Uuid {
    let org_id: Uuid = sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Acme') RETURNING id")
        .fetch_one(pool)
        .await
        .unwrap();
    sqlx::query_scalar("INSERT INTO sessions (org_id, channel, chat_id) VALUES ($1, 'telegram', 'chat-1') RETURNING id")
        .bind(org_id)
        .fetch_one(pool)
        .await
        .unwrap()
}

async fn seed_delegation(pool: &PgPool, session_id: Uuid, user_id: Uuid, target: &str, status: &str, result: Option<&str>) {
    sqlx::query(
        "INSERT INTO agent_delegations (session_id, user_id, requesting_agent_type, target_agent_type, task, status, result) \
         VALUES ($1, $2, 'chitchat', $3, 'do a thing', $4, $5)",
    )
    .bind(session_id)
    .bind(user_id)
    .bind(target)
    .bind(status)
    .bind(result)
    .execute(pool)
    .await
    .unwrap();
}

#[sqlx::test(migrations = "../../migrations")]
async fn list_recent_agent_activity_reports_no_delegations_when_none_exist(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let mut conn = pool.acquire().await.unwrap();

    let result = SupervisorAgent
        .execute_tool(&mut conn, session_id, Uuid::new_v4(), Uuid::new_v4(), "list_recent_agent_activity", serde_json::json!({}))
        .await
        .unwrap();

    assert_eq!(result.display_text, "No background delegations for this session yet.");
}

#[sqlx::test(migrations = "../../migrations")]
async fn list_recent_agent_activity_summarizes_completed_and_pending_delegations(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let user_id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(&pool).await.unwrap();
    seed_delegation(&pool, session_id, user_id, "money", "completed", Some("spent $42 on groceries")).await;
    seed_delegation(&pool, session_id, user_id, "personality", "pending", None).await;
    let mut conn = pool.acquire().await.unwrap();

    let result = SupervisorAgent
        .execute_tool(&mut conn, session_id, Uuid::new_v4(), Uuid::new_v4(), "list_recent_agent_activity", serde_json::json!({}))
        .await
        .unwrap();

    assert!(result.display_text.contains("money"));
    assert!(result.display_text.contains("completed"));
    assert!(result.display_text.contains("spent $42 on groceries"));
    assert!(result.display_text.contains("personality"));
    assert!(result.display_text.contains("pending"));
}

#[sqlx::test(migrations = "../../migrations")]
async fn an_unknown_tool_name_is_an_error(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let mut conn = pool.acquire().await.unwrap();

    let result = SupervisorAgent
        .execute_tool(&mut conn, session_id, Uuid::new_v4(), Uuid::new_v4(), "not_a_real_tool", serde_json::json!({}))
        .await;

    assert!(result.is_err());
}
