use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::agent_core::TurnError;

pub struct ClaimedJob {
    pub id: Uuid,
    pub session_id: Uuid,
    pub sender_channel_identity_id: Uuid,
    pub text: String,
    pub org_id_hint: Option<Uuid>,
}

pub async fn enqueue(
    tx: &mut Transaction<'_, Postgres>,
    session_id: Uuid,
    sender_channel_identity_id: Uuid,
    text: &str,
    org_id_hint: Option<Uuid>,
) -> Result<Uuid, TurnError> {
    let id: Uuid = sqlx::query_scalar(
        "INSERT INTO turn_jobs (session_id, sender_channel_identity_id, text, org_id_hint) \
         VALUES ($1, $2, $3, $4) RETURNING id",
    )
    .bind(session_id)
    .bind(sender_channel_identity_id)
    .bind(text)
    .bind(org_id_hint)
    .fetch_one(&mut **tx)
    .await?;

    sqlx::query("SELECT pg_notify('turn_jobs_channel', $1)")
        .bind(id.to_string())
        .execute(&mut **tx)
        .await?;

    Ok(id)
}

pub async fn claim_next(pool: &PgPool) -> Result<Option<ClaimedJob>, TurnError> {
    let row: Option<(Uuid, Uuid, Uuid, String, Option<Uuid>)> = sqlx::query_as(
        "WITH claimed AS ( \
             SELECT id FROM turn_jobs \
             WHERE status = 'pending' \
             ORDER BY created_at \
             FOR UPDATE SKIP LOCKED \
             LIMIT 1 \
         ) \
         UPDATE turn_jobs SET status = 'processing', claimed_at = now() \
         WHERE id IN (SELECT id FROM claimed) \
         RETURNING id, session_id, sender_channel_identity_id, text, org_id_hint",
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|(id, session_id, sender_channel_identity_id, text, org_id_hint)| ClaimedJob {
        id,
        session_id,
        sender_channel_identity_id,
        text,
        org_id_hint,
    }))
}

pub async fn mark_completed(pool: &PgPool, job_id: Uuid) -> Result<(), TurnError> {
    sqlx::query("UPDATE turn_jobs SET status = 'completed', completed_at = now() WHERE id = $1")
        .bind(job_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn mark_failed(pool: &PgPool, job_id: Uuid, error: &str) -> Result<(), TurnError> {
    sqlx::query("UPDATE turn_jobs SET status = 'failed', completed_at = now(), error = $2 WHERE id = $1")
        .bind(job_id)
        .bind(error)
        .execute(pool)
        .await?;
    Ok(())
}
