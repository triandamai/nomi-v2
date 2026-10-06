use sqlx::PgPool;
use uuid::Uuid;

use nomi_turn::ingest::ingest_inbound_message;
use nomi_turn::queue;

async fn seed_session(pool: &PgPool) -> Uuid {
    let org_id: Uuid = sqlx::query_scalar("INSERT INTO organizations (name, is_personal) VALUES ('t', true) RETURNING id")
        .fetch_one(pool)
        .await
        .unwrap();
    sqlx::query_scalar("INSERT INTO sessions (org_id, channel, chat_type, chat_id) VALUES ($1, 'telegram', 'dm', 'chat-1') RETURNING id")
        .bind(org_id)
        .fetch_one(pool)
        .await
        .unwrap()
}

#[sqlx::test(migrations = "../../migrations")]
async fn enqueue_then_claim_returns_the_job(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let sender_id = Uuid::new_v4();

    let mut tx = pool.begin().await.unwrap();
    let job_id = queue::enqueue(&mut tx, session_id, sender_id, "hello", None).await.unwrap();
    tx.commit().await.unwrap();

    let claimed = queue::claim_next(&pool).await.unwrap().expect("expected a pending job");
    assert_eq!(claimed.id, job_id);
    assert_eq!(claimed.session_id, session_id);
    assert_eq!(claimed.sender_channel_identity_id, sender_id);
    assert_eq!(claimed.text, "hello");

    let status: String = sqlx::query_scalar("SELECT status FROM turn_jobs WHERE id = $1")
        .bind(job_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(status, "processing");
}

#[sqlx::test(migrations = "../../migrations")]
async fn claim_next_returns_none_when_no_pending_jobs(pool: PgPool) {
    let claimed = queue::claim_next(&pool).await.unwrap();
    assert!(claimed.is_none());
}

#[sqlx::test(migrations = "../../migrations")]
async fn two_concurrent_claims_never_return_the_same_job(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let sender_id = Uuid::new_v4();

    let mut tx = pool.begin().await.unwrap();
    let job_id = queue::enqueue(&mut tx, session_id, sender_id, "hello", None).await.unwrap();
    tx.commit().await.unwrap();

    let (a, b) = tokio::join!(queue::claim_next(&pool), queue::claim_next(&pool));
    let (a, b) = (a.unwrap(), b.unwrap());

    // Exactly one of the two concurrent claims won the single pending job; the other found nothing.
    let winners: Vec<_> = [a, b].into_iter().flatten().collect();
    assert_eq!(winners.len(), 1);
    assert_eq!(winners[0].id, job_id);
}

#[sqlx::test(migrations = "../../migrations")]
async fn mark_completed_and_mark_failed_update_status(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let sender_id = Uuid::new_v4();

    let mut tx = pool.begin().await.unwrap();
    let job_id = queue::enqueue(&mut tx, session_id, sender_id, "hello", None).await.unwrap();
    tx.commit().await.unwrap();
    queue::claim_next(&pool).await.unwrap();

    queue::mark_completed(&pool, job_id).await.unwrap();
    let (status, completed_at): (String, Option<chrono::DateTime<chrono::Utc>>) =
        sqlx::query_as("SELECT status, completed_at FROM turn_jobs WHERE id = $1")
            .bind(job_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(status, "completed");
    assert!(completed_at.is_some());

    let mut tx = pool.begin().await.unwrap();
    let job_id_2 = queue::enqueue(&mut tx, session_id, sender_id, "hello again", None).await.unwrap();
    tx.commit().await.unwrap();
    queue::claim_next(&pool).await.unwrap();

    queue::mark_failed(&pool, job_id_2, "boom").await.unwrap();
    let (status, error): (String, Option<String>) =
        sqlx::query_as("SELECT status, error FROM turn_jobs WHERE id = $1")
            .bind(job_id_2)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(status, "failed");
    assert_eq!(error.as_deref(), Some("boom"));
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_turn_left_processing_past_the_limit_is_failed_so_the_chat_stops_working(pool: PgPool) {
    let stuck = ingest_inbound_message(&pool, "telegram", "dm", "chat-1", "tg-1", "hello", None).await.unwrap();
    let fresh = ingest_inbound_message(&pool, "telegram", "dm", "chat-2", "tg-2", "hi", None).await.unwrap();
    // Both claimed; the first by a worker that died an hour ago.
    sqlx::query("UPDATE turn_jobs SET status = 'processing', claimed_at = now() WHERE id IN ($1, $2)")
        .bind(stuck.turn_job_id)
        .bind(fresh.turn_job_id)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("UPDATE turn_jobs SET claimed_at = now() - interval '1 hour' WHERE id = $1").bind(stuck.turn_job_id).execute(&pool).await.unwrap();

    let failed = nomi_turn::queue::fail_abandoned(&pool, std::time::Duration::from_secs(12 * 60)).await.unwrap();

    assert_eq!(failed, 1);
    let status = |id| {
        let pool = pool.clone();
        async move { sqlx::query_scalar::<_, String>("SELECT status FROM turn_jobs WHERE id = $1").bind(id).fetch_one(&pool).await.unwrap() }
    };
    assert_eq!(status(stuck.turn_job_id).await, "failed");
    assert_eq!(status(fresh.turn_job_id).await, "processing");
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_busy_chat_waits_its_turn_while_other_chats_go_ahead(pool: PgPool) {
    let first = ingest_inbound_message(&pool, "telegram", "dm", "chat-1", "tg-1", "plan my day", None).await.unwrap();
    let follow_up = ingest_inbound_message(&pool, "telegram", "dm", "chat-1", "tg-1", "hello?", None).await.unwrap();
    let other_chat = ingest_inbound_message(&pool, "telegram", "dm", "chat-2", "tg-2", "hi", None).await.unwrap();

    let claimed = queue::claim_next(&pool).await.unwrap().unwrap();
    assert_eq!(claimed.id, first.turn_job_id);
    // chat-1 is busy, so its follow-up waits and chat-2 goes ahead.
    let claimed = queue::claim_next(&pool).await.unwrap().unwrap();
    assert_eq!(claimed.id, other_chat.turn_job_id);
    assert!(queue::claim_next(&pool).await.unwrap().is_none());

    queue::mark_completed(&pool, first.turn_job_id).await.unwrap();
    let claimed = queue::claim_next(&pool).await.unwrap().unwrap();
    assert_eq!(claimed.id, follow_up.turn_job_id);
}
