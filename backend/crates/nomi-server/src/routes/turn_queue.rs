//! The asks waiting for the crew in a chat (Build together → Asks in line): two people, one
//! Koda, one change at a time. Who asked what, which one is running, which is next, and moving
//! one up to be answered next.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

use nomi_auth::extractor::AuthClaims;

use crate::app::AppState;

#[derive(Serialize)]
pub struct QueuedAsk {
    pub id: Uuid,
    pub text: String,
    pub who: String,
    pub mine: bool,
    /// running, next or queued.
    pub state: &'static str,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct Queue {
    pub asks: Vec<QueuedAsk>,
}

type Row = (Uuid, String, String, Uuid, String, DateTime<Utc>);

/// The asks still waiting or running in the chat, in the order they'll be answered.
pub async fn queue_of(pool: &sqlx::PgPool, me: Uuid, session_id: Uuid) -> Result<Vec<QueuedAsk>, sqlx::Error> {
    let rows: Vec<Row> = sqlx::query_as(
        "SELECT j.id, j.text, \
                COALESCE(NULLIF(up.display_name, ''), split_part(COALESCE(w.email, g.email, ''), '@', 1)), \
                ci.user_id, j.status, j.created_at \
         FROM turn_jobs j \
         JOIN channel_identities ci ON ci.id = j.sender_channel_identity_id \
         LEFT JOIN user_profiles up ON up.user_id = ci.user_id \
         LEFT JOIN web_credentials w ON w.user_id = ci.user_id \
         LEFT JOIN google_identities g ON g.user_id = ci.user_id \
         WHERE j.session_id = $1 AND j.status IN ('pending', 'processing') \
         ORDER BY (j.status = 'processing') DESC, j.priority DESC, j.created_at",
    )
    .bind(session_id)
    .fetch_all(pool)
    .await?;
    let mut next_given = false;
    Ok(rows
        .into_iter()
        .map(|(id, text, who, user_id, status, created_at)| {
            let state = if status == "processing" {
                "running"
            } else if !next_given {
                next_given = true;
                "next"
            } else {
                "queued"
            };
            QueuedAsk { id, text, who, mine: user_id == me, state, created_at }
        })
        .collect())
}

/// `GET /api/sessions/:id/queue`
pub async fn list(State(state): State<AppState>, AuthClaims(claims): AuthClaims, Path(session_id): Path<Uuid>) -> Result<Json<Queue>, (StatusCode, &'static str)> {
    super::sessions::authorize_session_access(&state.pool, claims.sub, session_id).await?;
    let asks = queue_of(&state.pool, claims.sub, session_id).await.map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to load the queue"))?;
    Ok(Json(Queue { asks }))
}

/// `POST /api/sessions/:id/queue/:job_id/first`: answer this waiting ask next. Anyone who can
/// write in the chat may reorder it; an ask already running can't move.
pub async fn move_first(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path((session_id, job_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<Queue>, (StatusCode, &'static str)> {
    super::sessions::authorize_session_access(&state.pool, claims.sub, session_id).await?;
    super::sessions::refuse_project_viewer(&state.pool, claims.sub, session_id).await?;
    let moved = sqlx::query(
        "UPDATE turn_jobs SET priority = (SELECT COALESCE(MAX(priority), 0) + 1 FROM turn_jobs WHERE session_id = $1 AND status = 'pending') \
         WHERE id = $2 AND session_id = $1 AND status = 'pending'",
    )
    .bind(session_id)
    .bind(job_id)
    .execute(&state.pool)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to move the ask"))?;
    if moved.rows_affected() == 0 {
        return Err((StatusCode::CONFLICT, "that ask isn't waiting any more"));
    }
    let asks = queue_of(&state.pool, claims.sub, session_id).await.map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to load the queue"))?;
    Ok(Json(Queue { asks }))
}
