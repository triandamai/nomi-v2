// tests/agent_sessions.rs
use sqlx::PgPool;
use uuid::Uuid;

async fn make_session_and_speaker(pool: &PgPool) -> (Uuid, Uuid) {
    let org_id: Uuid =
        sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Acme') RETURNING id")
            .fetch_one(pool)
            .await
            .unwrap();
    let session_id: Uuid = sqlx::query_scalar(
        "INSERT INTO sessions (org_id, channel, chat_id) VALUES ($1, 'telegram', 'chat-1') RETURNING id",
    )
    .bind(org_id)
    .fetch_one(pool)
    .await
    .unwrap();
    let user_id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id")
        .fetch_one(pool)
        .await
        .unwrap();
    let identity_id: Uuid = sqlx::query_scalar(
        "INSERT INTO channel_identities (user_id, channel, channel_user_id) VALUES ($1, 'telegram', '333') RETURNING id",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await
    .unwrap();
    (session_id, identity_id)
}

#[sqlx::test(migrations = "../../migrations")]
async fn only_one_active_agent_session_per_speaker(pool: PgPool) {
    let (session_id, identity_id) = make_session_and_speaker(&pool).await;

    sqlx::query(
        "INSERT INTO agent_sessions (session_id, sender_channel_identity_id, agent_type, status) VALUES ($1, $2, 'booking', 'active')",
    )
    .bind(session_id)
    .bind(identity_id)
    .execute(&pool)
    .await
    .unwrap();

    let err = sqlx::query(
        "INSERT INTO agent_sessions (session_id, sender_channel_identity_id, agent_type, status) VALUES ($1, $2, 'money', 'active')",
    )
    .bind(session_id)
    .bind(identity_id)
    .execute(&pool)
    .await
    .unwrap_err();

    assert_eq!(
        err.as_database_error().unwrap().constraint(),
        Some("agent_sessions_one_active_per_speaker")
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_completed_and_a_new_active_agent_session_can_coexist(pool: PgPool) {
    let (session_id, identity_id) = make_session_and_speaker(&pool).await;

    sqlx::query(
        "INSERT INTO agent_sessions (session_id, sender_channel_identity_id, agent_type, status, ended_at) VALUES ($1, $2, 'booking', 'completed', now())",
    )
    .bind(session_id)
    .bind(identity_id)
    .execute(&pool)
    .await
    .unwrap();

    // A new active row is fine — the partial index only guards 'active' rows.
    sqlx::query(
        "INSERT INTO agent_sessions (session_id, sender_channel_identity_id, agent_type, status) VALUES ($1, $2, 'money', 'active')",
    )
    .bind(session_id)
    .bind(identity_id)
    .execute(&pool)
    .await
    .unwrap();

    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM agent_sessions WHERE session_id = $1")
        .bind(session_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 2);
}

#[sqlx::test(migrations = "../../migrations")]
async fn agent_session_state_defaults_to_empty_json_object(pool: PgPool) {
    let (session_id, identity_id) = make_session_and_speaker(&pool).await;

    let agent_session_id: Uuid = sqlx::query_scalar(
        "INSERT INTO agent_sessions (session_id, sender_channel_identity_id, agent_type, status) VALUES ($1, $2, 'booking', 'active') RETURNING id",
    )
    .bind(session_id)
    .bind(identity_id)
    .fetch_one(&pool)
    .await
    .unwrap();

    let state: serde_json::Value =
        sqlx::query_scalar("SELECT state FROM agent_sessions WHERE id = $1")
            .bind(agent_session_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(state, serde_json::json!({}));
}
