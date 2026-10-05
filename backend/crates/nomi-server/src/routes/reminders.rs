//! The Reminders page: the user's scheduled reminders, and creating or cancelling one directly.
//! (Agents create them too, through the reminder tools in nomi-agent-core's reminders.rs.)

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::app::AppState;
use crate::web_identity::ensure_web_channel_identity;
use nomi_auth::extractor::AuthClaims;
use nomi_turn::bootstrap::bootstrap_identity_and_session;

/// The chat that reminders created on this page report back into.
const REMINDERS_CHAT_TITLE: &str = "Reminders";
const PAST_LIMIT: i64 = 20;

#[derive(Serialize)]
pub struct Reminder {
    pub id: Uuid,
    pub label: String,
    pub run_at: DateTime<Utc>,
    pub recurrence: Option<String>,
    pub recurrence_weekday: Option<i16>,
    pub recurrence_day_of_month: Option<i16>,
    pub agent: String,
    pub status: String,
    pub last_fired_at: Option<DateTime<Utc>>,
    pub session_id: Uuid,
}

#[derive(Serialize)]
pub struct RemindersResponse {
    pub timezone: String,
    pub upcoming: Vec<Reminder>,
    pub past: Vec<Reminder>,
}

type ReminderRow = (Uuid, String, DateTime<Utc>, Option<String>, Option<i16>, Option<i16>, String, String, Option<DateTime<Utc>>, Uuid);

const COLUMNS: &str =
    "id, label, run_at, recurrence, recurrence_weekday, recurrence_day_of_month, target_agent_type, status, last_fired_at, session_id";

fn to_reminder(row: ReminderRow) -> Reminder {
    let (id, label, run_at, recurrence, recurrence_weekday, recurrence_day_of_month, agent, status, last_fired_at, session_id) = row;
    Reminder { id, label, run_at, recurrence, recurrence_weekday, recurrence_day_of_month, agent, status, last_fired_at, session_id }
}

fn internal(e: sqlx::Error) -> (StatusCode, &'static str) {
    tracing::error!(error = %e, "reminders request failed");
    (StatusCode::INTERNAL_SERVER_ERROR, "failed to load reminders")
}

pub async fn list_reminders(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
) -> Result<Json<RemindersResponse>, (StatusCode, &'static str)> {
    let timezone: String = sqlx::query_scalar("SELECT timezone FROM user_preferences WHERE user_id = $1")
        .bind(claims.sub)
        .fetch_optional(&state.pool)
        .await
        .map_err(internal)?
        .unwrap_or_else(|| "UTC".to_string());
    let upcoming: Vec<ReminderRow> =
        sqlx::query_as(&format!("SELECT {COLUMNS} FROM scheduled_jobs WHERE user_id = $1 AND status = 'active' ORDER BY run_at"))
            .bind(claims.sub)
            .fetch_all(&state.pool)
            .await
            .map_err(internal)?;
    let past: Vec<ReminderRow> = sqlx::query_as(&format!(
        "SELECT {COLUMNS} FROM scheduled_jobs WHERE user_id = $1 AND status <> 'active' \
         ORDER BY COALESCE(cancelled_at, last_fired_at, run_at) DESC LIMIT $2"
    ))
    .bind(claims.sub)
    .bind(PAST_LIMIT)
    .fetch_all(&state.pool)
    .await
    .map_err(internal)?;
    Ok(Json(RemindersResponse {
        timezone,
        upcoming: upcoming.into_iter().map(to_reminder).collect(),
        past: past.into_iter().map(to_reminder).collect(),
    }))
}

#[derive(Deserialize)]
pub struct CreateReminderRequest {
    pub label: String,
    /// RFC 3339, with the user's offset.
    pub run_at: String,
    /// "daily", "weekly" (on run_at's weekday) or "monthly" (on run_at's day of the month).
    pub recurrence: Option<String>,
}

/// The user's "Reminders" chat in their active org, created on first use.
async fn reminders_session(state: &AppState, user_id: Uuid, org_id: Uuid) -> Result<Uuid, (StatusCode, &'static str)> {
    let existing: Option<Uuid> = sqlx::query_scalar(
        "SELECT s.id FROM sessions s WHERE s.org_id = $1 AND s.channel = 'web' AND s.title = $2 \
         AND EXISTS (SELECT 1 FROM scheduled_jobs j WHERE j.session_id = s.id AND j.user_id = $3) \
         ORDER BY s.created_at LIMIT 1",
    )
    .bind(org_id)
    .bind(REMINDERS_CHAT_TITLE)
    .bind(user_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(internal)?;
    if let Some(id) = existing {
        return Ok(id);
    }
    ensure_web_channel_identity(&state.pool, user_id).await.map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to resolve web identity"))?;
    let created = bootstrap_identity_and_session(&state.pool, "web", "dm", &Uuid::new_v4().to_string(), &user_id.to_string(), Some(org_id))
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to create the reminders chat"))?;
    sqlx::query("UPDATE sessions SET title = $1 WHERE id = $2")
        .bind(REMINDERS_CHAT_TITLE)
        .bind(created.session_id)
        .execute(&state.pool)
        .await
        .map_err(internal)?;
    Ok(created.session_id)
}

pub async fn create_reminder(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Json(req): Json<CreateReminderRequest>,
) -> Result<(StatusCode, Json<Reminder>), (StatusCode, &'static str)> {
    let label = req.label.trim();
    if label.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "label is required"));
    }
    let run_at = DateTime::parse_from_rfc3339(&req.run_at).map_err(|_| (StatusCode::BAD_REQUEST, "run_at must be an RFC 3339 time"))?;
    if run_at.with_timezone(&Utc) <= Utc::now() {
        return Err((StatusCode::BAD_REQUEST, "run_at must be in the future"));
    }
    let (weekday, day_of_month) = match req.recurrence.as_deref() {
        None => (None, None),
        Some("daily") => (None, None),
        Some("weekly") => (Some(chrono::Datelike::weekday(&run_at).num_days_from_sunday() as i16), None),
        Some("monthly") => (None, Some(chrono::Datelike::day(&run_at) as i16)),
        Some(_) => return Err((StatusCode::BAD_REQUEST, "recurrence must be daily, weekly or monthly")),
    };
    let session_id = reminders_session(&state, claims.sub, claims.active_org_id).await?;
    let row: ReminderRow = sqlx::query_as(&format!(
        "INSERT INTO scheduled_jobs (session_id, user_id, created_by_agent_type, target_agent_type, label, prompt, run_at, \
         recurrence, recurrence_weekday, recurrence_day_of_month) \
         VALUES ($1, $2, 'user', 'planning', $3, $4, $5, $6, $7, $8) RETURNING {COLUMNS}"
    ))
    .bind(session_id)
    .bind(claims.sub)
    .bind(label)
    .bind(format!("It's time for the reminder the user set: \"{label}\". Remind them briefly and warmly."))
    .bind(run_at.with_timezone(&Utc))
    .bind(&req.recurrence)
    .bind(weekday)
    .bind(day_of_month)
    .fetch_one(&state.pool)
    .await
    .map_err(internal)?;
    Ok((StatusCode::CREATED, Json(to_reminder(row))))
}

pub async fn cancel_reminder(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, &'static str)> {
    let cancelled = sqlx::query(
        "UPDATE scheduled_jobs SET status = 'cancelled', cancelled_at = now() WHERE id = $1 AND user_id = $2 AND status = 'active'",
    )
    .bind(id)
    .bind(claims.sub)
    .execute(&state.pool)
    .await
    .map_err(internal)?
    .rows_affected();
    if cancelled == 0 {
        return Err((StatusCode::NOT_FOUND, "reminder not found"));
    }
    Ok(StatusCode::NO_CONTENT)
}
