use sqlx::PgPool;
use uuid::Uuid;

use nomi_agent_core::SubAgent;
use nomi_agent_personality::PersonalityAgent;

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

async fn seed_agent_session(pool: &PgPool, session_id: Uuid) -> (Uuid, Uuid) {
    let user_id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id")
        .fetch_one(pool)
        .await
        .unwrap();
    let identity_id: Uuid = sqlx::query_scalar(
        "INSERT INTO channel_identities (user_id, channel, channel_user_id) VALUES ($1, 'telegram', 'u1') RETURNING id",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await
    .unwrap();
    let agent_session_id: Uuid = sqlx::query_scalar(
        "INSERT INTO agent_sessions (session_id, sender_channel_identity_id, agent_type, status) VALUES ($1, $2, 'personality', 'active') RETURNING id",
    )
    .bind(session_id)
    .bind(identity_id)
    .fetch_one(pool)
    .await
    .unwrap();
    (user_id, agent_session_id)
}

#[sqlx::test(migrations = "../../migrations")]
async fn set_personality_upserts_and_records_an_audit_event(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let (user_id, agent_session_id) = seed_agent_session(&pool, session_id).await;
    let mut conn = pool.acquire().await.unwrap();

    let result = PersonalityAgent
        .execute_tool(
            &mut conn,
            session_id,
            agent_session_id,
            user_id,
            "set_personality",
            serde_json::json!({"description": "Be sarcastic and blunt."}),
        )
        .await
        .unwrap();

    assert!(result.contains("Be sarcastic and blunt."));

    let stored: String = sqlx::query_scalar("SELECT description FROM user_personality WHERE user_id = $1")
        .bind(user_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(stored, "Be sarcastic and blunt.");

    let event_type: String =
        sqlx::query_scalar("SELECT event_type FROM agent_events WHERE agent_session_id = $1")
            .bind(agent_session_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(event_type, "PersonalityChanged");
}

#[sqlx::test(migrations = "../../migrations")]
async fn set_personality_rejects_an_empty_description(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let (user_id, agent_session_id) = seed_agent_session(&pool, session_id).await;
    let mut conn = pool.acquire().await.unwrap();

    let result = PersonalityAgent
        .execute_tool(
            &mut conn,
            session_id,
            agent_session_id,
            user_id,
            "set_personality",
            serde_json::json!({"description": "   "}),
        )
        .await;

    assert!(result.is_err());

    let row_count: i64 = sqlx::query_scalar("SELECT count(*) FROM user_personality WHERE user_id = $1")
        .bind(user_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(row_count, 0);
}

#[sqlx::test(migrations = "../../migrations")]
async fn set_personality_rejects_a_missing_description(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let (user_id, agent_session_id) = seed_agent_session(&pool, session_id).await;
    let mut conn = pool.acquire().await.unwrap();

    let result = PersonalityAgent
        .execute_tool(&mut conn, session_id, agent_session_id, user_id, "set_personality", serde_json::json!({}))
        .await;

    assert!(result.is_err());
}

#[sqlx::test(migrations = "../../migrations")]
async fn an_unknown_tool_name_is_an_error(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let (user_id, agent_session_id) = seed_agent_session(&pool, session_id).await;
    let mut conn = pool.acquire().await.unwrap();

    let result = PersonalityAgent
        .execute_tool(&mut conn, session_id, agent_session_id, user_id, "delete_everything", serde_json::json!({}))
        .await;

    assert!(result.is_err());
}
