use axum::extract::State;
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
