use sqlx::PgPool;
use uuid::Uuid;

use nomi_agent_core::personality::{
    get_current_personality, list_versions, rollback_to_version, set_personality, RollbackError,
};

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
async fn returns_none_when_no_personality_is_set(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let (user_id, _agent_session_id) = seed_agent_session(&pool, session_id).await;
    let mut conn = pool.acquire().await.unwrap();

    let result = get_current_personality(&mut conn, user_id).await;
    assert_eq!(result, None);
}

#[sqlx::test(migrations = "../../migrations")]
async fn first_change_upserts_the_row_and_records_a_null_old_description(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let (user_id, agent_session_id) = seed_agent_session(&pool, session_id).await;
    let mut conn = pool.acquire().await.unwrap();

    set_personality(&mut conn, Some(session_id), Some(agent_session_id), user_id, "Be sarcastic and blunt.").await.unwrap();

    let stored = get_current_personality(&mut conn, user_id).await;
    assert_eq!(stored, Some("Be sarcastic and blunt.".to_string()));

    let (event_type, payload): (String, serde_json::Value) =
        sqlx::query_as("SELECT event_type, payload FROM agent_events WHERE agent_session_id = $1")
            .bind(agent_session_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(event_type, "PersonalityChanged");
    assert_eq!(payload["old_description"], serde_json::Value::Null);
    assert_eq!(payload["new_description"], "Be sarcastic and blunt.");
}

#[sqlx::test(migrations = "../../migrations")]
async fn second_change_updates_the_same_row_and_records_the_prior_value_as_old_description(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let (user_id, agent_session_id) = seed_agent_session(&pool, session_id).await;
    let mut conn = pool.acquire().await.unwrap();

    set_personality(&mut conn, Some(session_id), Some(agent_session_id), user_id, "Be sarcastic and blunt.").await.unwrap();
    set_personality(&mut conn, Some(session_id), Some(agent_session_id), user_id, "Be warm and encouraging.").await.unwrap();

    let stored = get_current_personality(&mut conn, user_id).await;
    assert_eq!(stored, Some("Be warm and encouraging.".to_string()));

    let row_count: i64 = sqlx::query_scalar("SELECT count(*) FROM user_personality WHERE user_id = $1")
        .bind(user_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(row_count, 1);

    let event_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM agent_events WHERE agent_session_id = $1 AND event_type = 'PersonalityChanged'",
    )
    .bind(agent_session_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(event_count, 2);

    let (payload,): (serde_json::Value,) = sqlx::query_as(
        "SELECT payload FROM agent_events WHERE agent_session_id = $1 AND event_type = 'PersonalityChanged' \
         ORDER BY created_at DESC LIMIT 1",
    )
    .bind(agent_session_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(payload["old_description"], "Be sarcastic and blunt.");
    assert_eq!(payload["new_description"], "Be warm and encouraging.");
}

#[sqlx::test(migrations = "../../migrations")]
async fn list_versions_returns_newest_first_with_the_current_one_marked(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let (user_id, agent_session_id) = seed_agent_session(&pool, session_id).await;
    let mut conn = pool.acquire().await.unwrap();

    set_personality(&mut conn, Some(session_id), Some(agent_session_id), user_id, "v1 desc").await.unwrap();
    set_personality(&mut conn, Some(session_id), Some(agent_session_id), user_id, "v2 desc").await.unwrap();
    set_personality(&mut conn, Some(session_id), Some(agent_session_id), user_id, "v3 desc").await.unwrap();

    let versions = list_versions(&mut conn, user_id, 10).await.unwrap();

    assert_eq!(versions.len(), 3);
    assert_eq!(versions[0].version, 3);
    assert_eq!(versions[0].description, "v3 desc");
    assert!(versions[0].is_current);
    assert_eq!(versions[1].version, 2);
    assert!(!versions[1].is_current);
    assert_eq!(versions[2].version, 1);
    assert!(!versions[2].is_current);
}

#[sqlx::test(migrations = "../../migrations")]
async fn list_versions_respects_the_limit(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let (user_id, agent_session_id) = seed_agent_session(&pool, session_id).await;
    let mut conn = pool.acquire().await.unwrap();

    for i in 1..=5 {
        set_personality(&mut conn, Some(session_id), Some(agent_session_id), user_id, &format!("v{i}")).await.unwrap();
    }

    let versions = list_versions(&mut conn, user_id, 2).await.unwrap();
    assert_eq!(versions.len(), 2);
    assert_eq!(versions[0].version, 5);
    assert_eq!(versions[1].version, 4);
}

#[sqlx::test(migrations = "../../migrations")]
async fn list_versions_returns_empty_for_a_user_with_no_personality_set(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let (user_id, _agent_session_id) = seed_agent_session(&pool, session_id).await;
    let mut conn = pool.acquire().await.unwrap();

    let versions = list_versions(&mut conn, user_id, 10).await.unwrap();
    assert!(versions.is_empty());
}

#[sqlx::test(migrations = "../../migrations")]
async fn rollback_creates_a_new_version_with_the_targets_description_rather_than_mutating_history(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let (user_id, agent_session_id) = seed_agent_session(&pool, session_id).await;
    let mut conn = pool.acquire().await.unwrap();

    set_personality(&mut conn, Some(session_id), Some(agent_session_id), user_id, "Be sarcastic.").await.unwrap();
    set_personality(&mut conn, Some(session_id), Some(agent_session_id), user_id, "Be warm.").await.unwrap();

    rollback_to_version(&mut conn, Some(session_id), Some(agent_session_id), user_id, 1).await.unwrap();

    let stored = get_current_personality(&mut conn, user_id).await;
    assert_eq!(stored, Some("Be sarcastic.".to_string()));

    let versions = list_versions(&mut conn, user_id, 10).await.unwrap();
    assert_eq!(versions.len(), 3);
    assert_eq!(versions[0].version, 3);
    assert_eq!(versions[0].description, "Be sarcastic.");
    assert!(versions[0].is_current);
    // Version 1's own row is untouched — history is append-only.
    assert_eq!(versions[2].version, 1);
    assert_eq!(versions[2].description, "Be sarcastic.");
    assert!(!versions[2].is_current);
}

#[sqlx::test(migrations = "../../migrations")]
async fn rollback_to_an_unknown_version_returns_version_not_found(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let (user_id, agent_session_id) = seed_agent_session(&pool, session_id).await;
    let mut conn = pool.acquire().await.unwrap();

    set_personality(&mut conn, Some(session_id), Some(agent_session_id), user_id, "Be sarcastic.").await.unwrap();

    let result = rollback_to_version(&mut conn, Some(session_id), Some(agent_session_id), user_id, 99).await;
    assert!(matches!(result, Err(RollbackError::VersionNotFound(99))));
}

#[sqlx::test(migrations = "../../migrations")]
async fn set_personality_with_no_turn_context_records_a_null_session_and_agent_session(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let (user_id, _agent_session_id) = seed_agent_session(&pool, session_id).await;
    let mut conn = pool.acquire().await.unwrap();

    set_personality(&mut conn, None, None, user_id, "Be sarcastic.").await.unwrap();

    let stored = get_current_personality(&mut conn, user_id).await;
    assert_eq!(stored, Some("Be sarcastic.".to_string()));

    let (session_id_col, agent_session_id_col): (Option<Uuid>, Option<Uuid>) = sqlx::query_as(
        "SELECT session_id, agent_session_id FROM agent_events \
         WHERE event_type = 'PersonalityChanged' AND payload->>'new_description' = 'Be sarcastic.'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(session_id_col, None);
    assert_eq!(agent_session_id_col, None);
}
