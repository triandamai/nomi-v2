//! Maintenance mode: an admin switches it on with a message (and when it should be over), and
//! everyone else sees the maintenance page until it's off again. The state is public so the
//! web app can check it on every request and the page can notice when it ends.

use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use nomi_auth::extractor::AuthClaims;

use crate::app::AppState;
use crate::routes::settings::require_system_config_permission;

#[derive(Serialize, sqlx::FromRow)]
pub struct Maintenance {
    pub enabled: bool,
    pub message: String,
    pub until: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
}

async fn load(pool: &sqlx::PgPool) -> Result<Maintenance, (StatusCode, &'static str)> {
    sqlx::query_as("SELECT enabled, message, until, updated_at FROM maintenance WHERE singleton")
        .fetch_one(pool)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "failed to read maintenance mode");
            (StatusCode::INTERNAL_SERVER_ERROR, "failed to read maintenance mode")
        })
}

/// `GET /api/maintenance` (no sign-in needed).
pub async fn get(State(state): State<AppState>) -> Result<Json<Maintenance>, (StatusCode, &'static str)> {
    Ok(Json(load(&state.pool).await?))
}

#[derive(Deserialize)]
pub struct MaintenanceInput {
    pub enabled: bool,
    #[serde(default)]
    pub message: String,
    pub until: Option<DateTime<Utc>>,
}

/// `PUT /api/admin/maintenance`
pub async fn put(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Json(input): Json<MaintenanceInput>,
) -> Result<Json<Maintenance>, (StatusCode, &'static str)> {
    require_system_config_permission(&claims)?;
    if input.message.chars().count() > 500 {
        return Err((StatusCode::BAD_REQUEST, "the message is at most 500 characters"));
    }
    sqlx::query("UPDATE maintenance SET enabled = $1, message = $2, until = $3, updated_by = $4, updated_at = now() WHERE singleton")
        .bind(input.enabled)
        .bind(input.message.trim())
        .bind(input.until)
        .bind(claims.sub)
        .execute(&state.pool)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "failed to save maintenance mode");
            (StatusCode::INTERNAL_SERVER_ERROR, "failed to save maintenance mode")
        })?;
    Ok(Json(load(&state.pool).await?))
}
