use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::app::AppState;
use nomi_agent_core::personality::{list_versions, rollback_to_version, RollbackError};
use nomi_auth::extractor::AuthClaims;

#[derive(Serialize)]
pub struct PersonalityVersionItem {
    pub version: i32,
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub is_current: bool,
}

#[derive(Serialize)]
pub struct PersonalityHistoryResponse {
    pub versions: Vec<PersonalityVersionItem>,
}

pub async fn get_personality_history(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
) -> Result<Json<PersonalityHistoryResponse>, (StatusCode, &'static str)> {
    let mut conn = state
        .pool
        .acquire()
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to acquire connection"))?;

    let versions = list_versions(&mut conn, claims.sub, 50)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to load personality history"))?
        .into_iter()
        .map(|v| PersonalityVersionItem {
            version: v.version,
            description: v.description,
            created_at: v.created_at,
            is_current: v.is_current,
        })
        .collect();

    Ok(Json(PersonalityHistoryResponse { versions }))
}

#[derive(Deserialize)]
pub struct RollbackPersonalityRequest {
    pub version: i32,
}

pub async fn rollback_personality(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Json(req): Json<RollbackPersonalityRequest>,
) -> Result<StatusCode, (StatusCode, &'static str)> {
    let mut conn = state
        .pool
        .acquire()
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to acquire connection"))?;

    rollback_to_version(&mut conn, None, None, claims.sub, req.version)
        .await
        .map_err(|e| match e {
            RollbackError::VersionNotFound(_) => (StatusCode::NOT_FOUND, "version not found"),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "failed to roll back personality"),
        })?;

    Ok(StatusCode::NO_CONTENT)
}

#[derive(Serialize)]
pub struct ChatPersonalityResponse {
    /// This chat's own personality, when it has one.
    pub chat: Option<String>,
    /// The person's default, which chats without their own use.
    pub default: Option<String>,
}

async fn chat_personality_response(conn: &mut sqlx::pool::PoolConnection<sqlx::Postgres>, user_id: uuid::Uuid, session_id: uuid::Uuid) -> ChatPersonalityResponse {
    ChatPersonalityResponse {
        chat: nomi_agent_core::personality::chat_personality(conn, session_id).await,
        default: nomi_agent_core::personality::get_current_personality(conn, user_id).await,
    }
}

/// `GET /api/sessions/:id/personality`: which personality this chat replies in.
pub async fn get_chat_personality(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(session_id): Path<uuid::Uuid>,
) -> Result<Json<ChatPersonalityResponse>, (StatusCode, &'static str)> {
    crate::routes::sessions::authorize_session_access(&state.pool, claims.sub, session_id).await?;
    let mut conn = state.pool.acquire().await.map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to acquire connection"))?;
    Ok(Json(chat_personality_response(&mut conn, claims.sub, session_id).await))
}

#[derive(Deserialize)]
pub struct SetChatPersonalityRequest {
    pub description: String,
    /// "chat" (default): only this chat. "all_chats": the person's default, everywhere.
    #[serde(default)]
    pub scope: Option<String>,
}

/// `PUT /api/sessions/:id/personality`: this chat's personality, or (scope all_chats) the default.
pub async fn put_chat_personality(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(session_id): Path<uuid::Uuid>,
    Json(req): Json<SetChatPersonalityRequest>,
) -> Result<Json<ChatPersonalityResponse>, (StatusCode, &'static str)> {
    crate::routes::sessions::authorize_session_access(&state.pool, claims.sub, session_id).await?;
    let description = req.description.trim();
    if description.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "description must not be empty"));
    }
    let mut conn = state.pool.acquire().await.map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to acquire connection"))?;
    let saved = match req.scope.as_deref() {
        Some("all_chats") => nomi_agent_core::personality::set_personality(&mut conn, Some(session_id), None, claims.sub, description).await,
        None | Some("chat") => nomi_agent_core::personality::set_chat_personality(&mut conn, session_id, None, claims.sub, description).await,
        Some(_) => return Err((StatusCode::BAD_REQUEST, "scope must be chat or all_chats")),
    };
    saved.map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to save personality"))?;
    Ok(Json(chat_personality_response(&mut conn, claims.sub, session_id).await))
}

/// `DELETE /api/sessions/:id/personality`: this chat goes back to the default.
pub async fn reset_chat_personality(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(session_id): Path<uuid::Uuid>,
) -> Result<Json<ChatPersonalityResponse>, (StatusCode, &'static str)> {
    crate::routes::sessions::authorize_session_access(&state.pool, claims.sub, session_id).await?;
    let mut conn = state.pool.acquire().await.map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to acquire connection"))?;
    nomi_agent_core::personality::clear_chat_personality(&mut conn, session_id)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to reset personality"))?;
    Ok(Json(chat_personality_response(&mut conn, claims.sub, session_id).await))
}
