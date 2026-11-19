use sqlx::PgPool;
use uuid::Uuid;

use super::bootstrap;
use super::queue;
use nomi_agent_core::TurnError;

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
    ingest_inbound_message_with(pool, channel, chat_type, chat_id, sender_channel_user_id, text, org_id_hint, None).await
}

/// [`ingest_inbound_message`], keeping `left_out` (the files the person sent the message without)
/// on the message in the same transaction that queues its turn, so the crew reads it.
#[allow(clippy::too_many_arguments)]
pub async fn ingest_inbound_message_with(
    pool: &PgPool,
    channel: &str,
    chat_type: &str,
    chat_id: &str,
    sender_channel_user_id: &str,
    text: &str,
    org_id_hint: Option<Uuid>,
    left_out: Option<&serde_json::Value>,
) -> Result<IngestResult, TurnError> {
    let bootstrap::BootstrapResult { sender_channel_identity_id, session_id, .. } =
        bootstrap::bootstrap_identity_and_session(pool, channel, chat_type, chat_id, sender_channel_user_id, org_id_hint)
            .await?;

    let mut tx = pool.begin().await?;

    let user_message_id: Uuid = sqlx::query_scalar(
        "INSERT INTO messages (session_id, sender_channel_identity_id, content, left_out) VALUES ($1, $2, $3, $4) RETURNING id",
    )
    .bind(session_id)
    .bind(sender_channel_identity_id)
    .bind(text)
    .bind(left_out)
    .fetch_one(&mut *tx)
    .await?;

    let turn_job_id = queue::enqueue(&mut tx, session_id, sender_channel_identity_id, text, org_id_hint).await?;

    tx.commit().await?;

    Ok(IngestResult { session_id, sender_channel_identity_id, user_message_id, turn_job_id })
}
