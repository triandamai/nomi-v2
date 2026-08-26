use sqlx::PgPool;

use nomi_turn::ingest::ingest_inbound_message;

#[sqlx::test(migrations = "../../migrations")]
async fn ingest_bootstraps_persists_the_message_and_enqueues_a_job(pool: PgPool) {
    let result = ingest_inbound_message(&pool, "telegram", "dm", "chat-1", "tg-1", "hello", None)
        .await
        .unwrap();

    let (content, sender_id): (String, Option<uuid::Uuid>) =
        sqlx::query_as("SELECT content, sender_channel_identity_id FROM messages WHERE id = $1")
            .bind(result.user_message_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(content, "hello");
    assert_eq!(sender_id, Some(result.sender_channel_identity_id));

    let (job_session_id, job_text, status): (uuid::Uuid, String, String) =
        sqlx::query_as("SELECT session_id, text, status FROM turn_jobs WHERE id = $1")
            .bind(result.turn_job_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(job_session_id, result.session_id);
    assert_eq!(job_text, "hello");
    assert_eq!(status, "pending");
}

#[sqlx::test(migrations = "../../migrations")]
async fn ingest_reuses_the_same_session_and_identity_across_two_calls(pool: PgPool) {
    let first = ingest_inbound_message(&pool, "telegram", "dm", "chat-1", "tg-1", "first", None).await.unwrap();
    let second = ingest_inbound_message(&pool, "telegram", "dm", "chat-1", "tg-1", "second", None).await.unwrap();

    assert_eq!(first.session_id, second.session_id);
    assert_eq!(first.sender_channel_identity_id, second.sender_channel_identity_id);
    assert_ne!(first.turn_job_id, second.turn_job_id);

    let job_count: i64 = sqlx::query_scalar("SELECT count(*) FROM turn_jobs WHERE session_id = $1")
        .bind(first.session_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(job_count, 2);
}
