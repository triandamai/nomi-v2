use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

use crate::app::AppState;
use crate::auth::extractor::AuthClaims;
use crate::turn::bootstrap::bootstrap_identity_and_session;
use crate::web_identity::ensure_web_channel_identity;

#[derive(Serialize)]
pub struct CreateSessionResponse {
    pub session_id: Uuid,
}

pub async fn create_session(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
) -> Result<(StatusCode, Json<CreateSessionResponse>), (StatusCode, &'static str)> {
    ensure_web_channel_identity(&state.pool, claims.sub)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to resolve web identity"))?;

    let chat_id = Uuid::new_v4().to_string();
    let bootstrap_result = bootstrap_identity_and_session(
        &state.pool,
        "web",
        "dm",
        &chat_id,
        &claims.sub.to_string(),
        Some(claims.active_org_id),
    )
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to create session"))?;

    Ok((StatusCode::CREATED, Json(CreateSessionResponse { session_id: bootstrap_result.session_id })))
}

#[derive(Serialize)]
pub struct LastMessagePreview {
    pub content: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct SessionListItem {
    pub id: Uuid,
    pub channel: String,
    pub chat_type: String,
    pub chat_id: String,
    pub last_message: Option<LastMessagePreview>,
    pub agent_active: bool,
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct ListSessionsResponse {
    pub sessions: Vec<SessionListItem>,
}

pub async fn list_sessions(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
) -> Result<Json<ListSessionsResponse>, (StatusCode, &'static str)> {
    let rows: Vec<(Uuid, String, String, String, Option<String>, Option<DateTime<Utc>>, bool, DateTime<Utc>)> = sqlx::query_as(
        "SELECT \
            s.id, s.channel, s.chat_type, s.chat_id, \
            lm.content, lm.created_at, \
            EXISTS (SELECT 1 FROM agent_sessions ag WHERE ag.session_id = s.id AND ag.status = 'active'), \
            COALESCE(lm.created_at, s.created_at) \
         FROM sessions s \
         LEFT JOIN LATERAL ( \
             SELECT content, created_at FROM messages m WHERE m.session_id = s.id ORDER BY m.created_at DESC LIMIT 1 \
         ) lm ON true \
         WHERE s.org_id = $1 \
         ORDER BY COALESCE(lm.created_at, s.created_at) DESC",
    )
    .bind(claims.active_org_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to list sessions"))?;

    let sessions = rows
        .into_iter()
        .map(|(id, channel, chat_type, chat_id, last_content, last_created_at, agent_active, updated_at)| {
            let last_message = match (last_content, last_created_at) {
                (Some(content), Some(created_at)) => Some(LastMessagePreview { content, created_at }),
                _ => None,
            };
            SessionListItem { id, channel, chat_type, chat_id, last_message, agent_active, updated_at }
        })
        .collect();

    Ok(Json(ListSessionsResponse { sessions }))
}
