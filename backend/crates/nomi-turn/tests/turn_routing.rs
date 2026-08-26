use sqlx::PgPool;
use uuid::Uuid;

use nomi_turn::routing::find_active_agent_session;

async fn seed_session_and_speaker(pool: &PgPool) -> (Uuid, Uuid) {
    let org_id: Uuid = sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Acme') RETURNING id")
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
        "INSERT INTO channel_identities (user_id, channel, channel_user_id) VALUES ($1, 'telegram', 'u1') RETURNING id",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await
    .unwrap();
    (session_id, identity_id)
}

#[sqlx::test(migrations = "../../migrations")]
async fn no_active_agent_session_returns_none(pool: PgPool) {
    let (session_id, identity_id) = seed_session_and_speaker(&pool).await;
    let mut conn = pool.acquire().await.unwrap();

    let result = find_active_agent_session(&mut conn, session_id, identity_id).await.unwrap();
    assert_eq!(result, None);
}

#[sqlx::test(migrations = "../../migrations")]
async fn an_active_agent_session_returns_its_id(pool: PgPool) {
    let (session_id, identity_id) = seed_session_and_speaker(&pool).await;
    let mut conn = pool.acquire().await.unwrap();

    let agent_session_id: Uuid = sqlx::query_scalar(
        "INSERT INTO agent_sessions (session_id, sender_channel_identity_id, agent_type, status) VALUES ($1, $2, 'booking', 'active') RETURNING id",
    )
    .bind(session_id)
    .bind(identity_id)
    .fetch_one(&pool)
    .await
    .unwrap();

    let result = find_active_agent_session(&mut conn, session_id, identity_id).await.unwrap();
    assert_eq!(result, Some(agent_session_id));
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_completed_agent_session_is_not_returned(pool: PgPool) {
    let (session_id, identity_id) = seed_session_and_speaker(&pool).await;
    let mut conn = pool.acquire().await.unwrap();

    sqlx::query(
        "INSERT INTO agent_sessions (session_id, sender_channel_identity_id, agent_type, status, ended_at) VALUES ($1, $2, 'booking', 'completed', now())",
    )
    .bind(session_id)
    .bind(identity_id)
    .execute(&pool)
    .await
    .unwrap();

    let result = find_active_agent_session(&mut conn, session_id, identity_id).await.unwrap();
    assert_eq!(result, None);
}

#[sqlx::test(migrations = "../../migrations")]
async fn an_active_agent_session_for_a_different_speaker_is_not_returned(pool: PgPool) {
    let (session_id, identity_id) = seed_session_and_speaker(&pool).await;
    let mut conn = pool.acquire().await.unwrap();

    let other_user_id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id")
        .fetch_one(&pool)
        .await
        .unwrap();
    let other_identity_id: Uuid = sqlx::query_scalar(
        "INSERT INTO channel_identities (user_id, channel, channel_user_id) VALUES ($1, 'telegram', 'u2') RETURNING id",
    )
    .bind(other_user_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO agent_sessions (session_id, sender_channel_identity_id, agent_type, status) VALUES ($1, $2, 'booking', 'active')",
    )
    .bind(session_id)
    .bind(other_identity_id)
    .execute(&pool)
    .await
    .unwrap();

    let result = find_active_agent_session(&mut conn, session_id, identity_id).await.unwrap();
    assert_eq!(result, None);
}
