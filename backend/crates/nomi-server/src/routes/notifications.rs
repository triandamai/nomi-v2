//! A person's notification inbox, which emails they get, and admin broadcasts (promos and news).

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use chrono::{DateTime, NaiveTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use nomi_auth::extractor::AuthClaims;

use crate::app::AppState;
use crate::notifications::{self, Audience, Kind, Notice};

type ApiError = (StatusCode, &'static str);

const PAGE_SIZE: i64 = 20;

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
    /// This page (from 1) and how many pages the current filter has (at least 1).
    pub page: i64,
    pub pages: i64,
    pub page_size: i64,
    /// What the current filter matches in all.
    pub total: i64,
    /// How many each filter chip would show.
    pub counts: InboxCounts,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct InboxCounts {
    pub all: i64,
    pub unread: i64,
    pub read: i64,
    pub promo: i64,
}

#[derive(Deserialize)]
pub struct InboxQuery {
    /// "all" (default), "unread", "read" or "promo".
    pub filter: Option<String>,
    pub page: Option<i64>,
}

/// The WHERE condition for a filter, after `user_id = $1`. Fixed strings only, never input.
fn filter_condition(filter: Option<&str>) -> Result<&'static str, ApiError> {
    match filter.unwrap_or("all") {
        "all" => Ok("TRUE"),
        "unread" => Ok("read_at IS NULL"),
        "read" => Ok("read_at IS NOT NULL"),
        "promo" => Ok("kind = 'promo'"),
        _ => Err((StatusCode::BAD_REQUEST, "filter must be all, unread, read or promo")),
    }
}

async fn unread_count(state: &AppState, user_id: Uuid) -> Result<i64, ApiError> {
    sqlx::query_scalar("SELECT count(*) FROM notifications WHERE user_id = $1 AND read_at IS NULL")
        .bind(user_id)
        .fetch_one(&state.pool)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to load notifications"))
}

/// `GET /api/notifications?filter=&page=`: newest first, a page at a time, with the count behind
/// each filter.
pub async fn list(State(state): State<AppState>, AuthClaims(claims): AuthClaims, Query(query): Query<InboxQuery>) -> Result<Json<InboxPage>, ApiError> {
    let condition = filter_condition(query.filter.as_deref())?;
    let failed = |_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to load notifications");
    let counts: InboxCounts = sqlx::query_as(
        "SELECT count(*) AS \"all\", count(*) FILTER (WHERE read_at IS NULL) AS unread, \
                count(*) FILTER (WHERE read_at IS NOT NULL) AS \"read\", count(*) FILTER (WHERE kind = 'promo') AS promo \
         FROM notifications WHERE user_id = $1",
    )
    .bind(claims.sub)
    .fetch_one(&state.pool)
    .await
    .map_err(failed)?;
    let total = match query.filter.as_deref().unwrap_or("all") {
        "unread" => counts.unread,
        "read" => counts.read,
        "promo" => counts.promo,
        _ => counts.all,
    };
    let pages = ((total + PAGE_SIZE - 1) / PAGE_SIZE).max(1);
    let page = query.page.unwrap_or(1).clamp(1, pages);
    let items: Vec<NotificationItem> = sqlx::query_as(&format!(
        "SELECT id, kind, title, body, link, read_at IS NOT NULL AS read, created_at FROM notifications \
         WHERE user_id = $1 AND {condition} ORDER BY created_at DESC, id DESC LIMIT $2 OFFSET $3"
    ))
    .bind(claims.sub)
    .bind(PAGE_SIZE)
    .bind((page - 1) * PAGE_SIZE)
    .fetch_all(&state.pool)
    .await
    .map_err(failed)?;
    Ok(Json(InboxPage { items, unread: counts.unread, page, pages, page_size: PAGE_SIZE, total, counts }))
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

/// Quiet hours as local times ("HH:MM") in the person's time zone; `start` after `end` runs over
/// midnight.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct QuietHours {
    pub start: String,
    pub end: String,
}

struct NotificationPreferences {
    /// Plan, quota and other changes to their account.
    email_account: bool,
    /// News and offers.
    email_promos: bool,
    /// Reminders pushed to their phones as they go off.
    push_reminders: bool,
    /// Reminders due inside these hours wait until they end; null when there are none.
    quiet_hours: Option<QuietHours>,
}

/// The body of `GET|PUT /api/notifications/preferences`: which emails the person gets. Kept to
/// these two fields, as the clients built on it compare it whole.
#[derive(Serialize)]
pub struct EmailPreferences {
    pub email_account: bool,
    pub email_promos: bool,
}

/// The body of `GET|PUT /api/notifications/reminder-preferences`.
#[derive(Serialize)]
pub struct ReminderPreferences {
    pub push_reminders: bool,
    pub quiet_hours: Option<QuietHours>,
}

impl From<NotificationPreferences> for EmailPreferences {
    fn from(prefs: NotificationPreferences) -> Self {
        Self { email_account: prefs.email_account, email_promos: prefs.email_promos }
    }
}

impl From<NotificationPreferences> for ReminderPreferences {
    fn from(prefs: NotificationPreferences) -> Self {
        Self { push_reminders: prefs.push_reminders, quiet_hours: prefs.quiet_hours }
    }
}

type PreferencesRow = (bool, bool, bool, Option<NaiveTime>, Option<NaiveTime>);

const TIME_FORMAT: &str = "%H:%M";

fn preferences_from(row: Option<PreferencesRow>) -> NotificationPreferences {
    let (email_account, email_promos, push_reminders, start, end) = row.unwrap_or((true, true, true, None, None));
    let quiet_hours = match (start, end) {
        (Some(start), Some(end)) => Some(QuietHours { start: start.format(TIME_FORMAT).to_string(), end: end.format(TIME_FORMAT).to_string() }),
        _ => None,
    };
    NotificationPreferences { email_account, email_promos, push_reminders, quiet_hours }
}

async fn load_preferences(state: &AppState, user_id: Uuid) -> Result<NotificationPreferences, ApiError> {
    let row: Option<PreferencesRow> = sqlx::query_as(
        "SELECT email_account, email_promos, push_reminders, quiet_hours_start, quiet_hours_end FROM user_preferences WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to load preferences"))?;
    Ok(preferences_from(row))
}

/// `GET /api/notifications/preferences`.
pub async fn get_preferences(State(state): State<AppState>, AuthClaims(claims): AuthClaims) -> Result<Json<EmailPreferences>, ApiError> {
    Ok(Json(load_preferences(&state, claims.sub).await?.into()))
}

/// `GET /api/notifications/reminder-preferences`.
pub async fn get_reminder_preferences(State(state): State<AppState>, AuthClaims(claims): AuthClaims) -> Result<Json<ReminderPreferences>, ApiError> {
    Ok(Json(load_preferences(&state, claims.sub).await?.into()))
}

/// Tells a field left out (keep it) from one sent as null (clear it).
fn present<'de, T: Deserialize<'de>, D: serde::Deserializer<'de>>(deserializer: D) -> Result<Option<Option<T>>, D::Error> {
    Option::<T>::deserialize(deserializer).map(Some)
}

/// Each field is optional: what's left out stays as it is.
#[derive(Deserialize)]
pub struct UpdateNotificationPreferences {
    pub email_account: Option<bool>,
    pub email_promos: Option<bool>,
    pub push_reminders: Option<bool>,
    #[serde(default, deserialize_with = "present")]
    pub quiet_hours: Option<Option<QuietHours>>,
}

fn parse_time(text: &str) -> Result<NaiveTime, ApiError> {
    NaiveTime::parse_from_str(text.trim(), TIME_FORMAT).map_err(|_| (StatusCode::BAD_REQUEST, "quiet hours are times like 22:00"))
}

/// `PUT /api/notifications/preferences`: changes the fields sent (any of the email and reminder
/// ones) and answers with the email ones.
pub async fn put_preferences(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Json(req): Json<UpdateNotificationPreferences>,
) -> Result<Json<EmailPreferences>, ApiError> {
    Ok(Json(save_preferences(&state, claims.sub, req).await?.into()))
}

/// `PUT /api/notifications/reminder-preferences`: changes the fields sent and answers with the
/// reminder ones.
pub async fn put_reminder_preferences(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Json(req): Json<UpdateNotificationPreferences>,
) -> Result<Json<ReminderPreferences>, ApiError> {
    Ok(Json(save_preferences(&state, claims.sub, req).await?.into()))
}

/// Saves the fields sent. Changing quiet hours lets reminders held by the old ones go off by the
/// new ones.
async fn save_preferences(state: &AppState, user_id: Uuid, req: UpdateNotificationPreferences) -> Result<NotificationPreferences, ApiError> {
    let (quiet_set, start, end) = match &req.quiet_hours {
        None => (false, None, None),
        Some(None) => (true, None, None),
        Some(Some(hours)) => {
            let (start, end) = (parse_time(&hours.start)?, parse_time(&hours.end)?);
            if start == end {
                return Err((StatusCode::BAD_REQUEST, "quiet hours must start and end at different times"));
            }
            (true, Some(start), Some(end))
        }
    };
    let failed = |_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to save preferences");
    let mut tx = state.pool.begin().await.map_err(failed)?;
    sqlx::query(
        "INSERT INTO user_preferences (user_id, email_account, email_promos, push_reminders, quiet_hours_start, quiet_hours_end) \
         VALUES ($1, COALESCE($2, true), COALESCE($3, true), COALESCE($4, true), $6, $7) \
         ON CONFLICT (user_id) DO UPDATE SET \
           email_account = COALESCE($2, user_preferences.email_account), \
           email_promos = COALESCE($3, user_preferences.email_promos), \
           push_reminders = COALESCE($4, user_preferences.push_reminders), \
           quiet_hours_start = CASE WHEN $5 THEN $6 ELSE user_preferences.quiet_hours_start END, \
           quiet_hours_end = CASE WHEN $5 THEN $7 ELSE user_preferences.quiet_hours_end END, \
           updated_at = now()",
    )
    .bind(user_id)
    .bind(req.email_account)
    .bind(req.email_promos)
    .bind(req.push_reminders)
    .bind(quiet_set)
    .bind(start)
    .bind(end)
    .execute(&mut *tx)
    .await
    .map_err(failed)?;
    if quiet_set {
        sqlx::query("UPDATE reminders SET held_until = NULL WHERE user_id = $1 AND held_until IS NOT NULL")
            .bind(user_id)
            .execute(&mut *tx)
            .await
            .map_err(failed)?;
    }
    tx.commit().await.map_err(failed)?;
    load_preferences(state, user_id).await
}

fn require_broadcast_permission(claims: &nomi_auth::claims::Claims, level: &str) -> Result<(), ApiError> {
    if claims.has_permission("admin", "notifications", level) {
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
    require_broadcast_permission(&claims, "write")?;
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
    require_broadcast_permission(&claims, "view")?;
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
