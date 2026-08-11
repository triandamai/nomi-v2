use sqlx::PgPool;
use uuid::Uuid;

use nomi_orchestrator::turn::queue;

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

#[sqlx::test]
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

#[sqlx::test]
async fn claim_next_returns_none_when_no_pending_jobs(pool: PgPool) {
    let claimed = queue::claim_next(&pool).await.unwrap();
    assert!(claimed.is_none());
}

#[sqlx::test]
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

#[sqlx::test]
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
