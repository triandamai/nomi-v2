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
async fn set_personality_for_all_chats_upserts_the_default_and_records_an_audit_event(pool: PgPool) {
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
            serde_json::json!({"description": "Be sarcastic and blunt.", "scope": "all_chats"}),
        )
        .await
        .unwrap();

    assert!(result.display_text.contains("Be sarcastic and blunt."));

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

#[sqlx::test(migrations = "../../migrations")]
async fn list_personality_versions_reports_history_with_the_current_one_marked(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let (user_id, agent_session_id) = seed_agent_session(&pool, session_id).await;
    let mut conn = pool.acquire().await.unwrap();

    PersonalityAgent
        .execute_tool(
            &mut conn,
            session_id,
            agent_session_id,
            user_id,
            "set_personality",
            serde_json::json!({"description": "Be sarcastic.", "scope": "all_chats"}),
        )
        .await
        .unwrap();
    PersonalityAgent
        .execute_tool(
            &mut conn,
            session_id,
            agent_session_id,
            user_id,
            "set_personality",
            serde_json::json!({"description": "Be warm.", "scope": "all_chats"}),
        )
        .await
        .unwrap();

    let result = PersonalityAgent
        .execute_tool(&mut conn, session_id, agent_session_id, user_id, "list_personality_versions", serde_json::json!({}))
        .await
        .unwrap();

    assert!(result.display_text.contains("v2"));
    assert!(result.display_text.contains("Be warm."));
    assert!(result.display_text.contains("(current)"));
    assert!(result.display_text.contains("v1"));
    assert!(result.display_text.contains("Be sarcastic."));
}

#[sqlx::test(migrations = "../../migrations")]
async fn list_personality_versions_with_no_history_says_so(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let (user_id, agent_session_id) = seed_agent_session(&pool, session_id).await;
    let mut conn = pool.acquire().await.unwrap();

    let result = PersonalityAgent
        .execute_tool(&mut conn, session_id, agent_session_id, user_id, "list_personality_versions", serde_json::json!({}))
        .await
        .unwrap();

    assert!(result.display_text.contains("No default personality history yet."));
    assert!(result.display_text.contains("This chat uses the default personality."));
}

#[sqlx::test(migrations = "../../migrations")]
async fn rollback_personality_restores_an_old_version(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let (user_id, agent_session_id) = seed_agent_session(&pool, session_id).await;
    let mut conn = pool.acquire().await.unwrap();

    PersonalityAgent
        .execute_tool(
            &mut conn,
            session_id,
            agent_session_id,
            user_id,
            "set_personality",
            serde_json::json!({"description": "Be sarcastic.", "scope": "all_chats"}),
        )
        .await
        .unwrap();
    PersonalityAgent
        .execute_tool(
            &mut conn,
            session_id,
            agent_session_id,
            user_id,
            "set_personality",
            serde_json::json!({"description": "Be warm.", "scope": "all_chats"}),
        )
        .await
        .unwrap();

    let result = PersonalityAgent
        .execute_tool(&mut conn, session_id, agent_session_id, user_id, "rollback_personality", serde_json::json!({"version": 1}))
        .await
        .unwrap();

    assert!(result.display_text.contains("version 1"));

    let stored: String = sqlx::query_scalar("SELECT description FROM user_personality WHERE user_id = $1")
        .bind(user_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(stored, "Be sarcastic.");
}

#[sqlx::test(migrations = "../../migrations")]
async fn rollback_personality_to_an_unknown_version_is_an_error(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let (user_id, agent_session_id) = seed_agent_session(&pool, session_id).await;
    let mut conn = pool.acquire().await.unwrap();

    let result = PersonalityAgent
        .execute_tool(&mut conn, session_id, agent_session_id, user_id, "rollback_personality", serde_json::json!({"version": 99}))
        .await;

    assert!(result.is_err());
}

#[sqlx::test(migrations = "../../migrations")]
async fn rollback_personality_rejects_a_missing_version(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let (user_id, agent_session_id) = seed_agent_session(&pool, session_id).await;
    let mut conn = pool.acquire().await.unwrap();

    let result = PersonalityAgent
        .execute_tool(&mut conn, session_id, agent_session_id, user_id, "rollback_personality", serde_json::json!({}))
        .await;

    assert!(result.is_err());
}

async fn seed_second_session(pool: &PgPool) -> Uuid {
    let org_id: Uuid = sqlx::query_scalar("SELECT org_id FROM sessions LIMIT 1").fetch_one(pool).await.unwrap();
    sqlx::query_scalar("INSERT INTO sessions (org_id, channel, chat_id) VALUES ($1, 'telegram', 'chat-2') RETURNING id")
        .bind(org_id)
        .fetch_one(pool)
        .await
        .unwrap()
}

#[sqlx::test(migrations = "../../migrations")]
async fn set_personality_defaults_to_this_chat_only(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let other_chat = seed_second_session(&pool).await;
    let (user_id, agent_session_id) = seed_agent_session(&pool, session_id).await;
    let mut conn = pool.acquire().await.unwrap();

    PersonalityAgent
        .execute_tool(&mut conn, session_id, agent_session_id, user_id, "set_personality", serde_json::json!({"description": "Be warm.", "scope": "all_chats"}))
        .await
        .unwrap();
    let result = PersonalityAgent
        .execute_tool(&mut conn, session_id, agent_session_id, user_id, "set_personality", serde_json::json!({"description": "Talk like a pirate."}))
        .await
        .unwrap();
    assert!(result.display_text.contains("this chat only"));

    use nomi_agent_core::personality::personality_for_chat;
    assert_eq!(personality_for_chat(&mut conn, user_id, session_id).await.as_deref(), Some("Talk like a pirate."));
    // Another chat keeps the default.
    assert_eq!(personality_for_chat(&mut conn, user_id, other_chat).await.as_deref(), Some("Be warm."));

    // Back to the default.
    let result = PersonalityAgent
        .execute_tool(&mut conn, session_id, agent_session_id, user_id, "reset_chat_personality", serde_json::json!({}))
        .await
        .unwrap();
    assert!(result.display_text.contains("back to the default"));
    assert_eq!(personality_for_chat(&mut conn, user_id, session_id).await.as_deref(), Some("Be warm."));
}

#[sqlx::test(migrations = "../../migrations")]
async fn setting_it_for_all_chats_replaces_chat_only_personalities(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let (user_id, agent_session_id) = seed_agent_session(&pool, session_id).await;
    sqlx::query("UPDATE sessions SET user_id = $2 WHERE id = $1").bind(session_id).bind(user_id).execute(&pool).await.unwrap();
    let mut conn = pool.acquire().await.unwrap();

    PersonalityAgent
        .execute_tool(&mut conn, session_id, agent_session_id, user_id, "set_personality", serde_json::json!({"description": "Talk like a pirate."}))
        .await
        .unwrap();
    PersonalityAgent
        .execute_tool(&mut conn, session_id, agent_session_id, user_id, "set_personality", serde_json::json!({"description": "Be formal.", "scope": "all_chats"}))
        .await
        .unwrap();

    use nomi_agent_core::personality::personality_for_chat;
    assert_eq!(personality_for_chat(&mut conn, user_id, session_id).await.as_deref(), Some("Be formal."));
}
