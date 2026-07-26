use std::time::{Duration, Instant};

use sqlx::PgPool;
use uuid::Uuid;

use nomi_orchestrator::turn::lock::{acquire_session_lock, insert_inbound_message, release_session_lock};

async fn seed_session(pool: &PgPool) -> (Uuid, Uuid) {
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

#[sqlx::test]
async fn acquire_and_release_round_trip(pool: PgPool) {
    let (session_id, _) = seed_session(&pool).await;
    let mut conn = acquire_session_lock(&pool, session_id).await.unwrap();
    release_session_lock(&mut conn, session_id).await.unwrap();
}

#[sqlx::test]
async fn insert_inbound_message_persists_a_durable_row(pool: PgPool) {
    let (session_id, identity_id) = seed_session(&pool).await;
    let mut conn = acquire_session_lock(&pool, session_id).await.unwrap();

    let message_id = insert_inbound_message(&mut conn, session_id, identity_id, "hello")
        .await
        .unwrap();

    release_session_lock(&mut conn, session_id).await.unwrap();

    let content: String = sqlx::query_scalar("SELECT content FROM messages WHERE id = $1")
        .bind(message_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(content, "hello");
}

#[sqlx::test]
async fn advisory_lock_serializes_two_concurrent_acquires_for_the_same_session(pool: PgPool) {
    let (session_id, _) = seed_session(&pool).await;

    let mut conn1 = acquire_session_lock(&pool, session_id).await.unwrap();

    let pool2 = pool.clone();
    let handle = tokio::spawn(async move {
        let start = Instant::now();
        let mut conn2 = acquire_session_lock(&pool2, session_id).await.unwrap();
        let waited = start.elapsed();
        release_session_lock(&mut conn2, session_id).await.unwrap();
        waited
    });

    tokio::time::sleep(Duration::from_millis(200)).await;
    release_session_lock(&mut conn1, session_id).await.unwrap();

    let waited = handle.await.unwrap();
    assert!(waited >= Duration::from_millis(150), "second acquire should have blocked on the first, waited {waited:?}");
}

#[sqlx::test]
async fn locks_for_different_sessions_do_not_contend(pool: PgPool) {
    let (session_a, _) = seed_session(&pool).await;
    let session_b = Uuid::new_v4();

    let mut conn_a = acquire_session_lock(&pool, session_a).await.unwrap();

    let pool2 = pool.clone();
    let handle = tokio::spawn(async move {
        let start = Instant::now();
        let mut conn_b = acquire_session_lock(&pool2, session_b).await.unwrap();
        let waited = start.elapsed();
        release_session_lock(&mut conn_b, session_b).await.unwrap();
        waited
    });

    let waited = handle.await.unwrap();
    release_session_lock(&mut conn_a, session_a).await.unwrap();

    assert!(waited < Duration::from_millis(100), "different sessions should not contend, waited {waited:?}");
}
