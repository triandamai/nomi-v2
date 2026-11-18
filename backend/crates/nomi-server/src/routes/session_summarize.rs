//! "Summarise and continue" (`POST /api/sessions/:id/summarize`): a chat that grew past what the
//! model can take has its older messages folded into one summary message (see
//! `nomi_agent_core::chat_summary`), and the turn that failed for being too long is tried again
//! from there. The summary is written by the model the chat's turns run on, metered like any
//! other call in the chat. It runs in the background: the request is answered `202`.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

use nomi_agent_core::chat_summary::{self, Summarized};
use nomi_auth::extractor::AuthClaims;

use crate::app::AppState;

/// A summary that has been running this long is taken to have been lost (the server restarted
/// part-way), and another may start.
const STALE_AFTER_SECS: f64 = 300.0;

fn refused(status: StatusCode, error: &'static str) -> Response {
    (status, Json(json!({ "error": error }))).into_response()
}

fn failed(context: &'static str) -> impl Fn(sqlx::Error) -> Response {
    move |e| {
        tracing::error!(error = %e, "{context}");
        refused(StatusCode::INTERNAL_SERVER_ERROR, context)
    }
}

/// Marks the chat as being summarised. False when another summary is already running for it.
async fn begin(pool: &PgPool, session_id: Uuid) -> Result<bool, sqlx::Error> {
    let started: Option<Uuid> = sqlx::query_scalar(
        "UPDATE sessions SET summarizing_since = now() \
         WHERE id = $1 AND (summarizing_since IS NULL OR summarizing_since < now() - make_interval(secs => $2)) RETURNING id",
    )
    .bind(session_id)
    .bind(STALE_AFTER_SECS)
    .fetch_optional(pool)
    .await?;
    Ok(started.is_some())
}

async fn finish(pool: &PgPool, session_id: Uuid) {
    if let Err(e) = sqlx::query("UPDATE sessions SET summarizing_since = NULL WHERE id = $1").bind(session_id).execute(pool).await {
        tracing::warn!(error = %e, %session_id, "couldn't mark the chat's summary as finished");
    }
}

/// `POST /api/sessions/:id/summarize`: 202 once it has started; 409 `already_summarizing` while
/// one is running for the chat, or `nothing_to_summarise` when there's nothing older to fold.
pub async fn summarize(State(state): State<AppState>, AuthClaims(claims): AuthClaims, Path(session_id): Path<Uuid>) -> Response {
    if let Err((status, message)) = crate::routes::sessions::authorize_session_access(&state.pool, claims.sub, session_id).await {
        return (status, message).into_response();
    }
    match begin(&state.pool, session_id).await {
        Ok(true) => {}
        Ok(false) => return refused(StatusCode::CONFLICT, "already_summarizing"),
        Err(e) => return failed("failed to start the summary")(e),
    }
    match chat_summary::has_older_messages(&state.pool, session_id).await {
        Ok(true) => {}
        Ok(false) => {
            finish(&state.pool, session_id).await;
            return refused(StatusCode::CONFLICT, "nothing_to_summarise");
        }
        Err(e) => {
            finish(&state.pool, session_id).await;
            return failed("failed to look at the chat")(e);
        }
    }

    tokio::spawn(run(state, claims.sub, session_id));
    (StatusCode::ACCEPTED, Json(json!({ "summarizing": true }))).into_response()
}

/// The background run: summarise, then try the failed turn again.
async fn run(state: AppState, user_id: Uuid, session_id: Uuid) {
    let provider = crate::bootstrap::build_llm_provider_in_session(
        &state.pool,
        user_id,
        session_id,
        &state.settings_key,
        state.http_client.clone(),
        crate::bootstrap::ModelPurpose::Chat,
    )
    .await;
    let outcome = chat_summary::summarize_chat(&state.pool, provider.as_ref(), session_id).await;
    finish(&state.pool, session_id).await;
    match outcome {
        Ok(Summarized::Done { message_id, .. }) => {
            crate::routes::sessions::announce_sent(&state.pool, user_id, session_id, None, [message_id]).await;
            retry_failed_turn(&state.pool, user_id, session_id).await;
        }
        Ok(Summarized::NothingToSummarise) => {}
        Err(e) => tracing::warn!(error = %e, %session_id, "summarising the chat failed"),
    }
}

/// The failed run at the end of the chat (the turn that was too long), tried again through the
/// same path as its Try again button.
async fn retry_failed_turn(pool: &PgPool, user_id: Uuid, session_id: Uuid) {
    let newest: Result<Option<Uuid>, sqlx::Error> = sqlx::query_scalar(
        "SELECT id FROM (SELECT id, content_blocks FROM messages WHERE session_id = $1 ORDER BY created_at DESC, id DESC LIMIT 1) newest \
         WHERE content_blocks->0->>'kind' = 'run_failed' AND content_blocks->0->'retry' IS NOT NULL",
    )
    .bind(session_id)
    .fetch_optional(pool)
    .await;
    match newest {
        Ok(Some(message_id)) => {
            if let Err((status, why)) = crate::routes::message_retry::retry_run(pool, user_id, session_id, message_id).await {
                tracing::warn!(%session_id, %status, why, "couldn't retry the turn after summarising");
            }
        }
        Ok(None) => {}
        Err(e) => tracing::warn!(error = %e, %session_id, "couldn't look for the failed turn"),
    }
}
