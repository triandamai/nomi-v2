//! Trying a failed run again from its error bubble (`ContentBlock::RunFailed`).
//!
//! The bubble names what failed: a chat turn (queued again with the same message), a
//! specialist's background run (handed the same task again) or, for a turn with no job of its
//! own, the person's latest message. The bubble goes once the run is queued again; whatever
//! happens next shows in the chat like any other run.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Serialize;
use uuid::Uuid;

use nomi_agent_core::content_block::{ContentBlock, RetryTarget};
use nomi_auth::extractor::AuthClaims;

use crate::app::AppState;

type ApiError = (StatusCode, &'static str);

fn internal(context: &'static str) -> impl Fn(sqlx::Error) -> ApiError {
    move |e| {
        tracing::error!(error = %e, "{context}");
        (StatusCode::INTERNAL_SERVER_ERROR, context)
    }
}

#[derive(Serialize, Debug, PartialEq)]
pub struct RetryResponse {
    /// The error bubble, now removed from the chat.
    pub removed_message_id: Uuid,
}

/// The retry target in a message's content blocks, if it's a failed run that can be tried again.
fn retry_target(blocks: Option<serde_json::Value>) -> Option<RetryTarget> {
    let blocks: Vec<ContentBlock> = serde_json::from_value(blocks?).ok()?;
    blocks.into_iter().find_map(|block| match block {
        ContentBlock::RunFailed { retry } => retry,
        _ => None,
    })
}

/// `POST /api/sessions/:id/messages/:message_id/retry`. 404 for a message that isn't in this
/// chat; 409 for one that isn't a failed run that can be retried, or when the chat is busy.
pub async fn retry_failed_run(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path((session_id, message_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<RetryResponse>, ApiError> {
    crate::routes::sessions::authorize_session_access(&state.pool, claims.sub, session_id).await?;

    let blocks: Option<Option<serde_json::Value>> = sqlx::query_scalar("SELECT content_blocks FROM messages WHERE id = $1 AND session_id = $2")
        .bind(message_id)
        .bind(session_id)
        .fetch_optional(&state.pool)
        .await
        .map_err(internal("failed to look up message"))?;
    let blocks = blocks.ok_or((StatusCode::NOT_FOUND, "message not found"))?;
    let target = retry_target(blocks).ok_or((StatusCode::CONFLICT, "this message can't be retried"))?;

    let mut tx = state.pool.begin().await.map_err(internal("failed to start retry"))?;
    match target {
        RetryTarget::Turn { turn_job_id } => requeue_turn(&mut tx, session_id, Some(turn_job_id)).await?,
        RetryTarget::LastMessage => requeue_turn(&mut tx, session_id, None).await?,
        RetryTarget::Delegation { delegation_id } => redelegate(&mut tx, session_id, delegation_id).await?,
    }
    sqlx::query("DELETE FROM messages WHERE id = $1").bind(message_id).execute(&mut *tx).await.map_err(internal("failed to clear the error"))?;
    tx.commit().await.map_err(internal("failed to retry"))?;
    crate::chat_events::messages_deleted(claims.sub, session_id, vec![message_id]);

    Ok(Json(RetryResponse { removed_message_id: message_id }))
}

/// Queues the failed turn's message again: the job's own, or the person's latest message when
/// there's no job (a turn that resumed after an approval).
async fn requeue_turn(tx: &mut sqlx::Transaction<'_, sqlx::Postgres>, session_id: Uuid, turn_job_id: Option<Uuid>) -> Result<(), ApiError> {
    let busy: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM turn_jobs WHERE session_id = $1 AND status IN ('pending', 'processing'))")
        .bind(session_id)
        .fetch_one(&mut **tx)
        .await
        .map_err(internal("failed to check the chat"))?;
    if busy {
        return Err((StatusCode::CONFLICT, "the crew is already working in this chat"));
    }

    let job: Option<(Uuid, String, Option<Uuid>)> = match turn_job_id {
        Some(id) => sqlx::query_as("SELECT sender_channel_identity_id, text, org_id_hint FROM turn_jobs WHERE id = $1 AND session_id = $2")
            .bind(id)
            .bind(session_id)
            .fetch_optional(&mut **tx)
            .await
            .map_err(internal("failed to look up the turn"))?,
        None => sqlx::query_as(
            "SELECT m.sender_channel_identity_id, m.content, s.org_id FROM messages m JOIN sessions s ON s.id = m.session_id \
             WHERE m.session_id = $1 AND m.sender_channel_identity_id IS NOT NULL ORDER BY m.created_at DESC, m.id DESC LIMIT 1",
        )
        .bind(session_id)
        .fetch_optional(&mut **tx)
        .await
        .map_err(internal("failed to look up the last message"))?,
    };
    let (sender, text, org_id_hint) = job.ok_or((StatusCode::CONFLICT, "there's nothing to try again"))?;
    nomi_turn::queue::enqueue(tx, session_id, sender, &text, org_id_hint).await.map_err(|e| {
        tracing::error!(error = %e, "failed to queue the retry");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to retry")
    })?;
    Ok(())
}

/// Hands a failed background run's task to the same specialist again.
async fn redelegate(tx: &mut sqlx::Transaction<'_, sqlx::Postgres>, session_id: Uuid, delegation_id: Uuid) -> Result<(), ApiError> {
    let new_id: Option<Uuid> = sqlx::query_scalar(
        "INSERT INTO agent_delegations (session_id, user_id, requesting_agent_type, target_agent_type, task) \
         SELECT session_id, user_id, requesting_agent_type, target_agent_type, task FROM agent_delegations d \
         WHERE d.id = $1 AND d.session_id = $2 AND d.status = 'failed' \
           AND NOT EXISTS (SELECT 1 FROM agent_delegations o WHERE o.session_id = d.session_id \
                           AND o.target_agent_type = d.target_agent_type AND o.status IN ('pending', 'claimed', 'processing')) \
         RETURNING id",
    )
    .bind(delegation_id)
    .bind(session_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(internal("failed to hand the task back"))?;
    let new_id = new_id.ok_or((StatusCode::CONFLICT, "that run is already going again"))?;
    sqlx::query("SELECT pg_notify('agent_delegations_channel', $1)")
        .bind(new_id.to_string())
        .execute(&mut **tx)
        .await
        .map_err(internal("failed to hand the task back"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_a_failed_run_with_a_target_can_be_retried() {
        let turn_job_id = Uuid::new_v4();
        let blocks = serde_json::json!([{"kind": "run_failed", "retry": {"target": "turn", "turn_job_id": turn_job_id}}]);
        assert_eq!(retry_target(Some(blocks)), Some(RetryTarget::Turn { turn_job_id }));
        assert_eq!(retry_target(Some(serde_json::json!([{"kind": "run_failed"}]))), None);
        assert_eq!(retry_target(Some(serde_json::json!([{"kind": "quota_notice", "reason": "used_up"}]))), None);
        assert_eq!(retry_target(None), None);
    }
}
