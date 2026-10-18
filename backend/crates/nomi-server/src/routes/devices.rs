//! The devices a person's native apps run on. Apps register on sign-in and whenever their push
//! token changes, and remove themselves on sign-out; phones then get inbox items and chat replies
//! by push (crate::push).

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use nomi_auth::extractor::AuthClaims;

use crate::app::AppState;

type ApiError = (StatusCode, &'static str);

const PLATFORMS: [&str; 5] = ["android", "ios", "macos", "windows", "linux"];

#[derive(Deserialize)]
pub struct RegisterDevice {
    pub platform: String,
    pub push_token: Option<String>,
    /// iOS: the ActivityKit token Live Activity pushes go to.
    pub live_activity_start_token: Option<String>,
    pub app_version: String,
}

#[derive(Serialize)]
pub struct Registered {
    pub id: Uuid,
}

fn non_empty(value: Option<String>) -> Option<String> {
    value.map(|v| v.trim().to_string()).filter(|v| !v.is_empty())
}

/// `POST /api/devices`: registers (or refreshes) this device; the same token keeps the same id.
/// A token another person registered before is theirs no longer: one phone, one owner.
pub async fn register(State(state): State<AppState>, AuthClaims(claims): AuthClaims, Json(body): Json<RegisterDevice>) -> Result<Json<Registered>, ApiError> {
    if !PLATFORMS.contains(&body.platform.as_str()) {
        return Err((StatusCode::BAD_REQUEST, "platform must be android, ios, macos, windows or linux"));
    }
    let app_version = body.app_version.trim();
    if app_version.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "app_version is required"));
    }
    let push_token = non_empty(body.push_token);
    let activity_token = non_empty(body.live_activity_start_token);
    let failed = |_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to register the device");

    let mut tx = state.pool.begin().await.map_err(failed)?;
    let id: Uuid = match &push_token {
        Some(token) => {
            sqlx::query("DELETE FROM devices WHERE push_token = $1 AND user_id <> $2")
                .bind(token)
                .bind(claims.sub)
                .execute(&mut *tx)
                .await
                .map_err(failed)?;
            sqlx::query_scalar(
                "INSERT INTO devices (user_id, platform, push_token, live_activity_start_token, app_version) VALUES ($1, $2, $3, $4, $5) \
                 ON CONFLICT (user_id, push_token) DO UPDATE SET platform = EXCLUDED.platform, \
                   live_activity_start_token = COALESCE(EXCLUDED.live_activity_start_token, devices.live_activity_start_token), \
                   app_version = EXCLUDED.app_version, last_seen_at = now() \
                 RETURNING id",
            )
            .bind(claims.sub)
            .bind(&body.platform)
            .bind(token)
            .bind(&activity_token)
            .bind(app_version)
            .fetch_one(&mut *tx)
            .await
            .map_err(failed)?
        }
        // No token (desktops): one row per platform, so signing in again doesn't pile up rows.
        None => {
            let existing: Option<Uuid> = sqlx::query_scalar(
                "UPDATE devices SET app_version = $3, live_activity_start_token = COALESCE($4, live_activity_start_token), last_seen_at = now() \
                 WHERE id = (SELECT id FROM devices WHERE user_id = $1 AND platform = $2 AND push_token IS NULL ORDER BY last_seen_at DESC LIMIT 1) \
                 RETURNING id",
            )
            .bind(claims.sub)
            .bind(&body.platform)
            .bind(app_version)
            .bind(&activity_token)
            .fetch_optional(&mut *tx)
            .await
            .map_err(failed)?;
            match existing {
                Some(id) => id,
                None => sqlx::query_scalar(
                    "INSERT INTO devices (user_id, platform, live_activity_start_token, app_version) VALUES ($1, $2, $3, $4) RETURNING id",
                )
                .bind(claims.sub)
                .bind(&body.platform)
                .bind(&activity_token)
                .bind(app_version)
                .fetch_one(&mut *tx)
                .await
                .map_err(failed)?,
            }
        }
    };
    tx.commit().await.map_err(failed)?;
    Ok(Json(Registered { id }))
}

/// `DELETE /api/devices/:id`: on sign-out. Someone else's device is 404.
pub async fn remove(State(state): State<AppState>, AuthClaims(claims): AuthClaims, Path(id): Path<Uuid>) -> Result<StatusCode, ApiError> {
    let deleted = sqlx::query("DELETE FROM devices WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(claims.sub)
        .execute(&state.pool)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to remove the device"))?;
    if deleted.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "no such device"));
    }
    Ok(StatusCode::NO_CONTENT)
}
