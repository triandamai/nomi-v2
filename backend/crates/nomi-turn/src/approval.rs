use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use nomi_agent_core::TurnError;

pub struct ClaimedApprovalResume {
    pub id: Uuid,
    pub message_id: Uuid,
    pub decision: String,
    pub remember: bool,
}

pub async fn enqueue(
    tx: &mut Transaction<'_, Postgres>,
    message_id: Uuid,
    decision: &str,
    remember: bool,
) -> Result<Uuid, TurnError> {
    let id: Uuid = sqlx::query_scalar(
        "INSERT INTO approval_resumes (message_id, decision, remember) VALUES ($1, $2, $3) RETURNING id",
    )
    .bind(message_id)
    .bind(decision)
    .bind(remember)
    .fetch_one(&mut **tx)
    .await?;

    sqlx::query("SELECT pg_notify('approval_resumes_channel', $1)")
        .bind(id.to_string())
        .execute(&mut **tx)
        .await?;

    Ok(id)
}

pub async fn claim_next(pool: &PgPool) -> Result<Option<ClaimedApprovalResume>, TurnError> {
    let row: Option<(Uuid, Uuid, String, bool)> = sqlx::query_as(
        "WITH claimed AS ( \
             SELECT id FROM approval_resumes \
             WHERE status = 'pending' \
             ORDER BY created_at \
             FOR UPDATE SKIP LOCKED \
             LIMIT 1 \
         ) \
         UPDATE approval_resumes SET status = 'processing', claimed_at = now() \
         WHERE id IN (SELECT id FROM claimed) \
         RETURNING id, message_id, decision, remember",
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|(id, message_id, decision, remember)| ClaimedApprovalResume { id, message_id, decision, remember }))
}

pub async fn mark_completed(pool: &PgPool, id: Uuid) -> Result<(), TurnError> {
    sqlx::query("UPDATE approval_resumes SET status = 'completed', completed_at = now() WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn mark_failed(pool: &PgPool, id: Uuid, error: &str) -> Result<(), TurnError> {
    sqlx::query("UPDATE approval_resumes SET status = 'failed', completed_at = now(), error = $2 WHERE id = $1")
        .bind(id)
        .bind(error)
        .execute(pool)
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn seed_message(pool: &PgPool) -> Uuid {
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
        sqlx::query_scalar("INSERT INTO messages (session_id, content) VALUES ($1, 'pending action') RETURNING id")
            .bind(session_id)
            .fetch_one(pool)
            .await
            .unwrap()
    }

    #[sqlx::test(migrations = "../../migrations")]
    async fn enqueue_then_claim_next_marks_it_processing(pool: PgPool) {
        let message_id = seed_message(&pool).await;

        let mut tx = pool.begin().await.unwrap();
        let resume_id = enqueue(&mut tx, message_id, "approve", false).await.unwrap();
        tx.commit().await.unwrap();

        let claimed = claim_next(&pool).await.unwrap().expect("expected a claimable resume");
        assert_eq!(claimed.id, resume_id);
        assert_eq!(claimed.message_id, message_id);
        assert_eq!(claimed.decision, "approve");
        assert!(!claimed.remember);

        let status: String = sqlx::query_scalar("SELECT status FROM approval_resumes WHERE id = $1")
            .bind(resume_id)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(status, "processing");
    }

    #[sqlx::test(migrations = "../../migrations")]
    async fn claim_next_returns_none_when_nothing_is_pending(pool: PgPool) {
        let message_id = seed_message(&pool).await;

        let mut tx = pool.begin().await.unwrap();
        enqueue(&mut tx, message_id, "deny", true).await.unwrap();
        tx.commit().await.unwrap();

        // First claim takes the only pending row.
        assert!(claim_next(&pool).await.unwrap().is_some());
        // Second claim finds nothing left pending.
        assert!(claim_next(&pool).await.unwrap().is_none());
    }

    #[sqlx::test(migrations = "../../migrations")]
    async fn mark_completed_updates_status_and_completed_at(pool: PgPool) {
        let message_id = seed_message(&pool).await;
        let mut tx = pool.begin().await.unwrap();
        let resume_id = enqueue(&mut tx, message_id, "approve", false).await.unwrap();
        tx.commit().await.unwrap();
        claim_next(&pool).await.unwrap();

        mark_completed(&pool, resume_id).await.unwrap();

        let (status, completed_at): (String, Option<chrono::DateTime<chrono::Utc>>) =
            sqlx::query_as("SELECT status, completed_at FROM approval_resumes WHERE id = $1")
                .bind(resume_id)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(status, "completed");
        assert!(completed_at.is_some());
    }

    #[sqlx::test(migrations = "../../migrations")]
    async fn mark_failed_updates_status_and_error(pool: PgPool) {
        let message_id = seed_message(&pool).await;
        let mut tx = pool.begin().await.unwrap();
        let resume_id = enqueue(&mut tx, message_id, "approve", false).await.unwrap();
        tx.commit().await.unwrap();
        claim_next(&pool).await.unwrap();

        mark_failed(&pool, resume_id, "boom").await.unwrap();

        let (status, error): (String, Option<String>) =
            sqlx::query_as("SELECT status, error FROM approval_resumes WHERE id = $1")
                .bind(resume_id)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(status, "failed");
        assert_eq!(error.as_deref(), Some("boom"));
    }
}
