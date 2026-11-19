//! Taking up a proposed plan: run it, or only research it first.
//!
//! The choice starts a turn exactly as the person's own message does (the same ingest and turn
//! queue as `POST /api/sessions/:id/messages`), worded by `nomi_agent_core::plans::RunMode`.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use nomi_agent_core::plans::RunMode;
use nomi_auth::extractor::AuthClaims;

use crate::app::AppState;

type ApiError = (StatusCode, &'static str);

#[derive(Deserialize)]
pub struct RunPlanRequest {
    /// `run` or `research`.
    #[serde(default)]
    pub mode: String,
}

#[derive(Serialize, Debug, PartialEq)]
pub struct RunPlanResponse {
    /// What was asked of the crew, now in the chat as the person's message.
    pub user_message_id: Uuid,
    pub turn_job_id: Uuid,
}

/// `POST /api/sessions/:id/agent-plans/:agent_session_id/run { mode }` -> 202. 400 for a mode
/// that isn't `run` or `research`; 404 when the chat has no plan by that agent.
pub async fn run_plan(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path((session_id, agent_session_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<RunPlanRequest>,
) -> Result<(StatusCode, Json<RunPlanResponse>), ApiError> {
    crate::routes::sessions::authorize_session_access(&state.pool, claims.sub, session_id).await?;
    crate::routes::sessions::refuse_project_viewer(&state.pool, claims.sub, session_id).await?;
    let mode = RunMode::parse(&req.mode).ok_or((StatusCode::BAD_REQUEST, "mode must be 'run' or 'research'"))?;

    let title: Option<String> = sqlx::query_scalar(
        "SELECT title FROM agent_plans WHERE session_id = $1 AND agent_session_id = $2 ORDER BY version DESC LIMIT 1",
    )
    .bind(session_id)
    .bind(agent_session_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to look up the plan"))?;
    let title = title.ok_or((StatusCode::NOT_FOUND, "plan not found"))?;

    let (channel, chat_type, chat_id, chat_title): (String, String, String, Option<String>) =
        sqlx::query_as("SELECT channel, chat_type, chat_id, title FROM sessions WHERE id = $1")
            .bind(session_id)
            .fetch_one(&state.pool)
            .await
            .map_err(|_| (StatusCode::NOT_FOUND, "session not found"))?;
    nomi_turn::bootstrap::ensure_web_channel_identity(&state.pool, claims.sub).await.map_err(|e| {
        tracing::error!(error = %e, "failed to resolve web identity");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to start the plan")
    })?;

    let ingested = nomi_turn::ingest::ingest_inbound_message(
        &state.pool,
        &channel,
        &chat_type,
        &chat_id,
        &claims.sub.to_string(),
        &mode.request(&title),
        Some(claims.active_org_id),
    )
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "failed to start the plan");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to start the plan")
    })?;
    crate::routes::sessions::announce_sent(&state.pool, claims.sub, session_id, chat_title, [ingested.user_message_id]).await;

    Ok((StatusCode::ACCEPTED, Json(RunPlanResponse { user_message_id: ingested.user_message_id, turn_job_id: ingested.turn_job_id })))
}
