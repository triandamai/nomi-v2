//! A person's notification inbox, which emails they get, and admin broadcasts (promos and news).

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use nomi_auth::extractor::AuthClaims;

use crate::app::AppState;
use crate::notifications::{self, Audience, Kind, Notice};

type ApiError = (StatusCode, &'static str);

const PAGE_SIZE: i64 = 30;

#[derive(Serialize, sqlx::FromRow)]
pub struct NotificationItem {
    pub id: Uuid,
    pub kind: String,
    pub title: String,
    pub body: String,
    pub link: Option<String>,
    pub read: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct InboxPage {
    pub items: Vec<NotificationItem>,
    pub unread: i64,
    /// Pass as `before` for the next, older page; `None` when there's no more.
    pub next_before: Option<DateTime<Utc>>,
}

#[derive(Deserialize)]
pub struct InboxQuery {
    pub before: Option<DateTime<Utc>>,
}

async fn unread_count(state: &AppState, user_id: Uuid) -> Result<i64, ApiError> {
    sqlx::query_scalar("SELECT count(*) FROM notifications WHERE user_id = $1 AND read_at IS NULL")
        .bind(user_id)
        .fetch_one(&state.pool)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to load notifications"))
}

/// `GET /api/notifications`: newest first, a page at a time.
pub async fn list(State(state): State<AppState>, AuthClaims(claims): AuthClaims, Query(query): Query<InboxQuery>) -> Result<Json<InboxPage>, ApiError> {
    let items: Vec<NotificationItem> = sqlx::query_as(
        "SELECT id, kind, title, body, link, read_at IS NOT NULL AS read, created_at FROM notifications \
         WHERE user_id = $1 AND ($2::timestamptz IS NULL OR created_at < $2) \
         ORDER BY created_at DESC LIMIT $3",
    )
    .bind(claims.sub)
    .bind(query.before)
    .bind(PAGE_SIZE)
    .fetch_all(&state.pool)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to load notifications"))?;
    let next_before = if items.len() as i64 == PAGE_SIZE { items.last().map(|n| n.created_at) } else { None };
    Ok(Json(InboxPage { items, unread: unread_count(&state, claims.sub).await?, next_before }))
}

#[derive(Serialize)]
pub struct UnreadCount {
    pub unread: i64,
    /// The newest unread one, for the app to pop up when it arrives.
    pub latest: Option<LatestNotification>,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct LatestNotification {
    pub id: Uuid,
    pub title: String,
    pub link: Option<String>,
}

/// `GET /api/notifications/unread`: the number on the bell, and the newest unread notification.
pub async fn unread(State(state): State<AppState>, AuthClaims(claims): AuthClaims) -> Result<Json<UnreadCount>, ApiError> {
    let latest: Option<LatestNotification> =
        sqlx::query_as("SELECT id, title, link FROM notifications WHERE user_id = $1 AND read_at IS NULL ORDER BY created_at DESC LIMIT 1")
            .bind(claims.sub)
            .fetch_optional(&state.pool)
            .await
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to load notifications"))?;
    Ok(Json(UnreadCount { unread: unread_count(&state, claims.sub).await?, latest }))
}

/// `POST /api/notifications/:id/read`.
pub async fn mark_read(State(state): State<AppState>, AuthClaims(claims): AuthClaims, Path(id): Path<Uuid>) -> Result<StatusCode, ApiError> {
    sqlx::query("UPDATE notifications SET read_at = now() WHERE id = $1 AND user_id = $2 AND read_at IS NULL")
        .bind(id)
        .bind(claims.sub)
        .execute(&state.pool)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to update notification"))?;
    Ok(StatusCode::NO_CONTENT)
}

/// `POST /api/notifications/read-all`.
pub async fn mark_all_read(State(state): State<AppState>, AuthClaims(claims): AuthClaims) -> Result<StatusCode, ApiError> {
    sqlx::query("UPDATE notifications SET read_at = now() WHERE user_id = $1 AND read_at IS NULL")
        .bind(claims.sub)
        .execute(&state.pool)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to update notifications"))?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Serialize, Deserialize, sqlx::FromRow)]
pub struct EmailPreferences {
    /// Plan, quota and other changes to their account.
    pub email_account: bool,
    /// News and offers.
    pub email_promos: bool,
}

/// `GET /api/notifications/preferences`.
pub async fn get_preferences(State(state): State<AppState>, AuthClaims(claims): AuthClaims) -> Result<Json<EmailPreferences>, ApiError> {
    let prefs: Option<EmailPreferences> = sqlx::query_as("SELECT email_account, email_promos FROM user_preferences WHERE user_id = $1")
        .bind(claims.sub)
        .fetch_optional(&state.pool)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to load preferences"))?;
    Ok(Json(prefs.unwrap_or(EmailPreferences { email_account: true, email_promos: true })))
}

/// `PUT /api/notifications/preferences`.
pub async fn put_preferences(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Json(prefs): Json<EmailPreferences>,
) -> Result<Json<EmailPreferences>, ApiError> {
    sqlx::query(
        "INSERT INTO user_preferences (user_id, email_account, email_promos) VALUES ($1, $2, $3) \
         ON CONFLICT (user_id) DO UPDATE SET email_account = $2, email_promos = $3, updated_at = now()",
    )
    .bind(claims.sub)
    .bind(prefs.email_account)
    .bind(prefs.email_promos)
    .execute(&state.pool)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to save preferences"))?;
    Ok(Json(prefs))
}

fn require_broadcast_permission(claims: &nomi_auth::claims::Claims) -> Result<(), ApiError> {
    if claims.has_permission("admin", "user", "manage") {
        Ok(())
    } else {
        Err((StatusCode::FORBIDDEN, "not authorized to send notifications"))
    }
}

#[derive(Deserialize)]
pub struct BroadcastRequest {
    pub title: String,
    pub body: String,
    pub link: Option<String>,
    /// "all", or a plan's id.
    #[serde(default)]
    pub audience: Option<String>,
}

#[derive(Serialize)]
pub struct BroadcastResponse {
    pub id: Uuid,
    pub recipients: i64,
}

/// `POST /api/admin/notifications/broadcast`: a promo or announcement for everyone, or for
/// everyone on one plan. It reaches each inbox, and the email of those who allow promos.
pub async fn broadcast(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Json(req): Json<BroadcastRequest>,
) -> Result<(StatusCode, Json<BroadcastResponse>), ApiError> {
    require_broadcast_permission(&claims)?;
    let (title, body) = (req.title.trim().to_string(), req.body.trim().to_string());
    if title.is_empty() || body.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "title and body are required"));
    }
    let link = req.link.map(|l| l.trim().to_string()).filter(|l| !l.is_empty());
    if link.as_deref().is_some_and(|l| !l.starts_with('/') || l.starts_with("//")) {
        return Err((StatusCode::BAD_REQUEST, "link must be a page in the app, like /billing"));
    }
    let audience = match req.audience.as_deref().map(str::trim) {
        None | Some("") | Some("all") => Audience::Everyone,
        Some(id) => Audience::Plan(id.parse().map_err(|_| (StatusCode::BAD_REQUEST, "audience must be all or a plan id"))?),
    };
    let (id, recipients) = notifications::broadcast(&state.pool, Notice { kind: Kind::Promo, title, body, link }, audience, claims.sub)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "failed to broadcast a notification");
            (StatusCode::INTERNAL_SERVER_ERROR, "failed to send")
        })?;
    Ok((StatusCode::CREATED, Json(BroadcastResponse { id, recipients })))
}

#[derive(Serialize, sqlx::FromRow)]
pub struct BroadcastItem {
    pub id: Uuid,
    pub title: String,
    pub body: String,
    pub link: Option<String>,
    pub audience: String,
    pub recipients: i32,
    pub read: i64,
    pub created_at: DateTime<Utc>,
}

/// `GET /api/admin/notifications/broadcasts`: what was sent, newest first, with how many read it.
pub async fn list_broadcasts(State(state): State<AppState>, AuthClaims(claims): AuthClaims) -> Result<Json<Vec<BroadcastItem>>, ApiError> {
    require_broadcast_permission(&claims)?;
    let items = sqlx::query_as(
        "SELECT b.id, b.title, b.body, b.link, b.audience, b.recipients, \
                (SELECT count(*) FROM notifications n WHERE n.broadcast_id = b.id AND n.read_at IS NOT NULL) AS read, b.created_at \
         FROM notification_broadcasts b ORDER BY b.created_at DESC LIMIT 50",
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to load broadcasts"))?;
    Ok(Json(items))
}
