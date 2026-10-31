//! The devices a person signs in on: phones, computers and browsers. Each app makes a device id
//! once and keeps it (`X-Device-Id` on every request, and the MQTT client id of its realtime
//! connection). Apps register on start and whenever their push token changes; phones then get
//! inbox items and chat replies by push (crate::push). The person sees their devices, with whether
//! each is on (`crate::presence`), and can sign one out from another.

use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::Json;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use nomi_auth::extractor::AuthClaims;

use crate::app::AppState;

type ApiError = (StatusCode, &'static str);

const PLATFORMS: [&str; 6] = ["android", "ios", "macos", "windows", "linux", "web"];
/// The header every app sends its device id in.
pub const DEVICE_ID_HEADER: &str = "x-device-id";
/// The header a browser tab sends its own id in, so each tab gets its own broker connection.
pub const TAB_ID_HEADER: &str = "x-tab-id";
/// The longest name, brand, model or OS a device may give itself.
const MAX_DETAIL_CHARS: usize = 100;

/// The device a request came from, when it said (`X-Device-Id`, a UUID).
pub fn device_id(headers: &HeaderMap) -> Option<Uuid> {
    headers.get(DEVICE_ID_HEADER)?.to_str().ok().and_then(|value| Uuid::parse_str(value.trim()).ok())
}

#[derive(Deserialize)]
pub struct RegisterDevice {
    /// The id the app made for itself; older apps without one get an id from the server.
    pub device_id: Option<Uuid>,
    pub platform: String,
    /// "Pixel 9", "Chrome on macOS".
    pub name: Option<String>,
    pub brand: Option<String>,
    pub model: Option<String>,
    pub os_name: Option<String>,
    pub os_version: Option<String>,
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

/// A detail the device gave about itself, trimmed and cut to `MAX_DETAIL_CHARS`.
fn detail(value: Option<String>) -> Option<String> {
    non_empty(value).map(|v| v.chars().take(MAX_DETAIL_CHARS).collect())
}

/// One of the person's devices, as their device list shows it.
#[derive(Serialize, sqlx::FromRow)]
pub struct DeviceItem {
    pub id: Uuid,
    pub platform: String,
    pub name: Option<String>,
    pub brand: Option<String>,
    pub model: Option<String>,
    pub os_name: Option<String>,
    pub os_version: Option<String>,
    pub app_version: String,
    /// `online`, `away` or `offline`.
    pub presence: String,
    pub last_seen_at: chrono::DateTime<chrono::Utc>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// The device this request came from.
    #[sqlx(default)]
    pub current: bool,
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
    if let Some(device_id) = body.device_id {
        // One device, one owner: a phone handed on, or a browser someone else signed in on, is
        // the new person's now, and so is its push token.
        sqlx::query("DELETE FROM devices WHERE (id = $1 AND user_id <> $2) OR (push_token = $3 AND id <> $1)")
            .bind(device_id)
            .bind(claims.sub)
            .bind(&push_token)
            .execute(&mut *tx)
            .await
            .map_err(failed)?;
        sqlx::query(
            "INSERT INTO devices (id, user_id, platform, push_token, live_activity_start_token, app_version, name, brand, model, os_name, os_version) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11) \
             ON CONFLICT (id) DO UPDATE SET platform = EXCLUDED.platform, push_token = COALESCE(EXCLUDED.push_token, devices.push_token), \
               live_activity_start_token = COALESCE(EXCLUDED.live_activity_start_token, devices.live_activity_start_token), \
               app_version = EXCLUDED.app_version, name = EXCLUDED.name, brand = EXCLUDED.brand, model = EXCLUDED.model, \
               os_name = EXCLUDED.os_name, os_version = EXCLUDED.os_version, last_seen_at = now()",
        )
        .bind(device_id)
        .bind(claims.sub)
        .bind(&body.platform)
        .bind(&push_token)
        .bind(&activity_token)
        .bind(app_version)
        .bind(detail(body.name))
        .bind(detail(body.brand))
        .bind(detail(body.model))
        .bind(detail(body.os_name))
        .bind(detail(body.os_version))
        .execute(&mut *tx)
        .await
        .map_err(failed)?;
        tx.commit().await.map_err(failed)?;
        return Ok(Json(Registered { id: device_id }));
    }
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

/// `GET /api/devices`: the person's devices, the ones that are on first.
pub async fn list(State(state): State<AppState>, AuthClaims(claims): AuthClaims, headers: HeaderMap) -> Result<Json<Vec<DeviceItem>>, ApiError> {
    let mut devices: Vec<DeviceItem> = sqlx::query_as(
        "SELECT id, platform, name, brand, model, os_name, os_version, app_version, presence, last_seen_at, created_at FROM devices \
         WHERE user_id = $1 ORDER BY presence = 'offline', last_seen_at DESC",
    )
    .bind(claims.sub)
    .fetch_all(&state.pool)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to load devices"))?;
    let this_device = device_id(&headers);
    for device in &mut devices {
        device.current = Some(device.id) == this_device;
    }
    Ok(Json(devices))
}

/// `DELETE /api/devices/:id`: signs that device out (its own sign-out, or from another device).
/// Its sign-ins end, and it hears `DeviceSignedOut` and leaves; its current access token lapses on
/// its own within the half hour. Someone else's device is 404.
pub async fn remove(State(state): State<AppState>, AuthClaims(claims): AuthClaims, Path(id): Path<Uuid>) -> Result<StatusCode, ApiError> {
    let failed = |_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to remove the device");
    let deleted = sqlx::query("DELETE FROM devices WHERE id = $1 AND user_id = $2").bind(id).bind(claims.sub).execute(&state.pool).await.map_err(failed)?;
    if deleted.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "no such device"));
    }
    nomi_auth::refresh_token::revoke_device(&state.pool, claims.sub, id).await.map_err(failed)?;
    crate::presence::announce(claims.sub, &nomi_realtime::UserEnvelope::DeviceSignedOut { device_id: id });
    Ok(StatusCode::NO_CONTENT)
}
