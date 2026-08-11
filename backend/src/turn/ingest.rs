use sqlx::PgPool;
use uuid::Uuid;

use super::bootstrap;
use super::queue;
use super::types::TurnError;

#[derive(Debug, Clone, PartialEq)]
pub struct IngestResult {
    pub session_id: Uuid,
    pub sender_channel_identity_id: Uuid,
    pub user_message_id: Uuid,
    pub turn_job_id: Uuid,
}

pub async fn ingest_inbound_message(
    pool: &PgPool,
    channel: &str,
    chat_type: &str,
    chat_id: &str,
    sender_channel_user_id: &str,
    text: &str,
    org_id_hint: Option<Uuid>,
) -> Result<IngestResult, TurnError> {
    let bootstrap::BootstrapResult { sender_channel_identity_id, session_id, .. } =
        bootstrap::bootstrap_identity_and_session(pool, channel, chat_type, chat_id, sender_channel_user_id, org_id_hint)
            .await?;

    let mut tx = pool.begin().await?;

    let user_message_id: Uuid = sqlx::query_scalar(
        "INSERT INTO messages (session_id, sender_channel_identity_id, content) VALUES ($1, $2, $3) RETURNING id",
    )
    .bind(session_id)
    .bind(sender_channel_identity_id)
    .bind(text)
    .fetch_one(&mut *tx)
    .await?;

    let turn_job_id = queue::enqueue(&mut tx, session_id, sender_channel_identity_id, text, org_id_hint).await?;

    tx.commit().await?;

    Ok(IngestResult { session_id, sender_channel_identity_id, user_message_id, turn_job_id })
}
