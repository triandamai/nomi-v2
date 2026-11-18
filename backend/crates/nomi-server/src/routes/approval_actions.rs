//! Taking back a deny, and throwing away what a stop left half done.
//!
//! A deny shows on its card at once, but the turn only carries on from it once the undo window
//! has passed (`nomi_turn::approval::UNDO_WINDOW_SECONDS`, on the server's clock): undoing it
//! cancels that waiting resume and puts the card back to pending. Discarding a stop clears the
//! partial output of the work it stopped and marks the card.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

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
pub struct UndoResponse {
    pub status: &'static str,
}

/// `POST /api/sessions/:id/messages/:message_id/approval/undo`. The card goes back to pending
/// when it was denied within the undo window; asking again for a card already back to pending
/// changes nothing. 409 once the window has passed, or for a card that isn't a denied approval.
pub async fn undo_deny(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path((session_id, message_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<UndoResponse>, ApiError> {
    crate::routes::sessions::authorize_session_access(&state.pool, claims.sub, session_id).await?;

    let card: Option<(Option<String>, Option<String>, Option<String>)> = sqlx::query_as(
        "SELECT content_blocks->0->>'kind', content_blocks->0->>'status', content_blocks->0->>'decided_at' \
         FROM messages WHERE id = $1 AND session_id = $2",
    )
    .bind(message_id)
    .bind(session_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(internal("failed to look up the approval"))?;
    let (kind, status, decided_at) = card.ok_or((StatusCode::NOT_FOUND, "message not found"))?;
    if kind.as_deref() != Some("approval_request") {
        return Err((StatusCode::CONFLICT, "this message isn't an approval"));
    }
    match status.as_deref() {
        // Already undone (or never decided): nothing to do, and nothing is done twice.
        Some("pending") => return Ok(Json(UndoResponse { status: "pending" })),
        Some("denied") => {}
        _ => return Err((StatusCode::CONFLICT, "this approval can't be undone")),
    }
    let decided_at = decided_at.and_then(|t| t.parse::<DateTime<Utc>>().ok());
    if !decided_at.is_some_and(|at| nomi_turn::approval::undo_open(at, Utc::now())) {
        return Err((StatusCode::CONFLICT, "it's too late to undo this"));
    }

    // The queued resume is what a deny is waiting on; cancelled, the turn never carries on from it.
    let cancelled = nomi_turn::approval::cancel_waiting_deny(&state.pool, message_id)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to undo the deny"))?;
    if !cancelled {
        return Err((StatusCode::CONFLICT, "it's too late to undo this"));
    }
    sqlx::query(
        "UPDATE messages SET content_blocks = jsonb_set(jsonb_set(content_blocks, '{0,status}', '\"pending\"'), '{0,decided_at}', 'null'::jsonb) \
         WHERE id = $1 AND content_blocks->0->>'status' = 'denied'",
    )
    .bind(message_id)
    .execute(&state.pool)
    .await
    .map_err(internal("failed to put the approval back"))?;
    crate::chat_events::message_updated(claims.sub, session_id, message_id);
    Ok(Json(UndoResponse { status: "pending" }))
}

#[derive(Serialize, Debug, PartialEq)]
pub struct DiscardResponse {
    /// How many stopped tasks had partial output cleared this time (0 when already discarded).
    pub discarded: usize,
}

/// `POST /api/sessions/:id/stopped/:message_id/discard`. Clears the partial output of the work
/// the stop card lists and marks the card discarded. Doing it again changes nothing. 409 for a
/// message that isn't a stop card.
pub async fn discard_stopped(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path((session_id, message_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<DiscardResponse>, ApiError> {
    crate::routes::sessions::authorize_session_access(&state.pool, claims.sub, session_id).await?;

    let mut tx = state.pool.begin().await.map_err(internal("failed to start discard"))?;
    // Locked, so two requests at once can't both clear and both count.
    let card: Option<(serde_json::Value,)> = sqlx::query_as(
        "SELECT content_blocks->0 FROM messages WHERE id = $1 AND session_id = $2 AND content_blocks->0->>'kind' = 'stopped' FOR UPDATE",
    )
    .bind(message_id)
    .bind(session_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(internal("failed to look up the stop"))?;
    let (block,) = card.ok_or((StatusCode::CONFLICT, "this message isn't a stop"))?;
    if block["discarded"].as_bool() == Some(true) {
        return Ok(Json(DiscardResponse { discarded: 0 }));
    }

    let ids = |key: &str| -> Vec<Uuid> {
        block["tasks"].as_array().into_iter().flatten().filter_map(|t| t[key].as_str()?.parse().ok()).collect()
    };
    let delegations = sqlx::query(
        "UPDATE agent_delegations SET result = NULL, discarded_at = now() \
         WHERE id = ANY($1) AND session_id = $2 AND user_id = $3 AND status = 'cancelled' AND discarded_at IS NULL",
    )
    .bind(ids("delegation_id"))
    .bind(session_id)
    .bind(claims.sub)
    .execute(&mut *tx)
    .await
    .map_err(internal("failed to discard the work"))?
    .rows_affected();
    let background = sqlx::query(
        "UPDATE background_tasks SET output = NULL, discarded_at = now() \
         WHERE id = ANY($1) AND session_id = $2 AND user_id = $3 AND status = 'cancelled' AND discarded_at IS NULL",
    )
    .bind(ids("background_task_id"))
    .bind(session_id)
    .bind(claims.sub)
    .execute(&mut *tx)
    .await
    .map_err(internal("failed to discard the work"))?
    .rows_affected();
    sqlx::query("UPDATE messages SET content_blocks = jsonb_set(content_blocks, '{0,discarded}', 'true'::jsonb) WHERE id = $1")
        .bind(message_id)
        .execute(&mut *tx)
        .await
        .map_err(internal("failed to mark the stop discarded"))?;
    tx.commit().await.map_err(internal("failed to discard the work"))?;

    crate::chat_events::message_updated(claims.sub, session_id, message_id);
    Ok(Json(DiscardResponse { discarded: (delegations + background) as usize }))
}
