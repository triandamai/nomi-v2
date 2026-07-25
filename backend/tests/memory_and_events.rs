use sqlx::PgPool;
use uuid::Uuid;

#[sqlx::test]
async fn memory_item_accepts_a_1536_dimension_embedding_and_default_weight(pool: PgPool) {
    let user_id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id")
        .fetch_one(&pool)
        .await
        .unwrap();

    let embedding = format!("[{}]", vec!["0.01"; 1536].join(","));
    let memory_id: Uuid = sqlx::query_scalar(
        "INSERT INTO memory_items (user_id, content, embedding) VALUES ($1, 'likes window seats', $2::vector) RETURNING id",
    )
    .bind(user_id)
    .bind(&embedding)
    .fetch_one(&pool)
    .await
    .unwrap();

    let weight: f64 = sqlx::query_scalar("SELECT weight FROM memory_items WHERE id = $1")
        .bind(memory_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(weight, 1.0);
}

#[sqlx::test]
async fn memory_item_rejects_wrong_dimension_embedding(pool: PgPool) {
    let user_id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id")
        .fetch_one(&pool)
        .await
        .unwrap();

    let wrong_size_embedding = format!("[{}]", vec!["0.01"; 3].join(","));
    let err = sqlx::query(
        "INSERT INTO memory_items (user_id, content, embedding) VALUES ($1, 'bad row', $2::vector)",
    )
    .bind(user_id)
    .bind(&wrong_size_embedding)
    .execute(&pool)
    .await
    .unwrap_err();

    assert!(err
        .as_database_error()
        .unwrap()
        .message()
        .contains("expected 1536 dimensions"));
}

#[sqlx::test]
async fn agent_event_stores_jsonb_payload_and_links_to_agent_session(pool: PgPool) {
    let org_id: Uuid =
        sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Acme') RETURNING id")
            .fetch_one(&pool)
            .await
            .unwrap();
    let session_id: Uuid = sqlx::query_scalar(
        "INSERT INTO sessions (org_id, channel, chat_id) VALUES ($1, 'telegram', 'chat-1') RETURNING id",
    )
    .bind(org_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    let user_id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id")
        .fetch_one(&pool)
        .await
        .unwrap();
    let identity_id: Uuid = sqlx::query_scalar(
        "INSERT INTO channel_identities (user_id, channel, channel_user_id) VALUES ($1, 'telegram', '444') RETURNING id",
    )
    .bind(user_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    let agent_session_id: Uuid = sqlx::query_scalar(
        "INSERT INTO agent_sessions (session_id, sender_channel_identity_id, agent_type, status) VALUES ($1, $2, 'booking', 'active') RETURNING id",
    )
    .bind(session_id)
    .bind(identity_id)
    .fetch_one(&pool)
    .await
    .unwrap();

    let payload = serde_json::json!({"summary": "spawned booking agent"});
    sqlx::query(
        "INSERT INTO agent_events (session_id, sender_channel_identity_id, agent_session_id, agent_type, event_type, payload) VALUES ($1, $2, $3, 'booking', 'AgentSpawned', $4)",
    )
    .bind(session_id)
    .bind(identity_id)
    .bind(agent_session_id)
    .bind(&payload)
    .execute(&pool)
    .await
    .unwrap();

    let stored_payload: serde_json::Value =
        sqlx::query_scalar("SELECT payload FROM agent_events WHERE agent_session_id = $1")
            .bind(agent_session_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(stored_payload, payload);
}
