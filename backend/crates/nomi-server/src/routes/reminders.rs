//! The Reminders page. Reminders live in the Reminders agent's own table (nomi-agent-reminders);
//! agent tasks scheduled for later (core `scheduled_jobs`) are listed alongside, read-only except
//! for cancelling.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::app::AppState;
use crate::web_identity::ensure_web_channel_identity;
use nomi_agent_reminders::Reminder;
use nomi_auth::extractor::AuthClaims;
use nomi_turn::bootstrap::bootstrap_identity_and_session;

/// The chat that reminders created on this page report back into.
const REMINDERS_CHAT_TITLE: &str = "Reminders";
const PAST_LIMIT: i64 = 20;

#[derive(Serialize)]
pub struct ScheduledTask {
    pub id: Uuid,
    pub label: String,
    pub run_at: DateTime<Utc>,
    pub recurrence: Option<String>,
    pub agent: String,
    pub session_id: Uuid,
}

#[derive(Serialize)]
pub struct RemindersResponse {
    pub timezone: String,
    pub upcoming: Vec<Reminder>,
    pub past: Vec<Reminder>,
    /// Work an agent will do later (core scheduler), not reminders.
    pub scheduled_tasks: Vec<ScheduledTask>,
}

type ScheduledTaskRow = (Uuid, String, DateTime<Utc>, Option<String>, String, Uuid);

fn internal<E: std::fmt::Display>(e: E) -> (StatusCode, String) {
    tracing::error!(error = %e, "reminders request failed");
    (StatusCode::INTERNAL_SERVER_ERROR, "reminders request failed".to_string())
}

pub async fn list_reminders(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
) -> Result<Json<RemindersResponse>, (StatusCode, String)> {
    let mut conn = state.pool.acquire().await.map_err(internal)?;
    let timezone = nomi_agent_reminders::user_timezone(&mut conn, claims.sub).await.name().to_string();
    let upcoming = nomi_agent_reminders::list(&mut conn, claims.sub, &["active"], 200).await.map_err(internal)?;
    let past = nomi_agent_reminders::list(&mut conn, claims.sub, &["fired", "done", "cancelled"], PAST_LIMIT).await.map_err(internal)?;
    let tasks: Vec<ScheduledTaskRow> = sqlx::query_as(
        "SELECT id, label, run_at, recurrence, target_agent_type, session_id FROM scheduled_jobs \
         WHERE user_id = $1 AND status = 'active' ORDER BY run_at",
    )
    .bind(claims.sub)
    .fetch_all(&mut *conn)
    .await
    .map_err(internal)?;
    Ok(Json(RemindersResponse {
        timezone,
        upcoming,
        past,
        scheduled_tasks: tasks
            .into_iter()
            .map(|(id, label, run_at, recurrence, agent, session_id)| ScheduledTask { id, label, run_at, recurrence, agent, session_id })
            .collect(),
    }))
}

#[derive(Deserialize)]
pub struct CreateReminderRequest {
    pub title: String,
    /// RFC 3339 with the user's offset, or a local date-time read in their timezone.
    pub due_at: String,
    /// "daily", "weekly" (on due_at's weekday) or "monthly" (on due_at's day of the month).
    pub recurrence: Option<String>,
    pub notes: Option<String>,
}

/// The user's "Reminders" chat in their active org, created on first use.
async fn reminders_session(state: &AppState, user_id: Uuid, org_id: Uuid) -> Result<Uuid, (StatusCode, String)> {
    let existing: Option<Uuid> = sqlx::query_scalar(
        "SELECT s.id FROM sessions s WHERE s.org_id = $1 AND s.user_id = $3 AND s.channel = 'web' AND s.title = $2 \
         AND EXISTS (SELECT 1 FROM reminders r WHERE r.session_id = s.id AND r.user_id = $3) \
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
    ensure_web_channel_identity(&state.pool, user_id).await.map_err(internal)?;
    let created = bootstrap_identity_and_session(&state.pool, "web", "dm", &Uuid::new_v4().to_string(), &user_id.to_string(), Some(org_id))
        .await
        .map_err(internal)?;
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
) -> Result<(StatusCode, Json<Reminder>), (StatusCode, String)> {
    let mut conn = state.pool.acquire().await.map_err(internal)?;
    let tz = nomi_agent_reminders::user_timezone(&mut conn, claims.sub).await;
    let due_at = nomi_agent_reminders::parse_due_at(&req.due_at, tz).map_err(|e| (StatusCode::BAD_REQUEST, e))?;
    let session_id = reminders_session(&state, claims.sub, claims.active_org_id).await?;
    let reminder = nomi_agent_reminders::add(
        &mut conn,
        claims.sub,
        session_id,
        nomi_agent_reminders::NewReminder { title: &req.title, notes: req.notes.as_deref(), due_at, recurrence: req.recurrence.as_deref() },
        "user",
    )
    .await
    .map_err(|e| (StatusCode::BAD_REQUEST, e))?;
    Ok((StatusCode::CREATED, Json(reminder)))
}

#[derive(Deserialize)]
pub struct ReminderAction {
    /// "done", "cancel" or "snooze".
    pub action: String,
    /// For snooze; defaults to 10.
    pub minutes: Option<i64>,
}

/// Done / cancel / snooze, from the page or from a reminder's bubble in chat.
pub async fn reminder_action(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(id): Path<Uuid>,
    Json(req): Json<ReminderAction>,
) -> Result<StatusCode, (StatusCode, String)> {
    let mut conn = state.pool.acquire().await.map_err(internal)?;
    let changed = match req.action.as_str() {
        "done" => nomi_agent_reminders::set_status(&mut conn, claims.sub, id, "done").await,
        "cancel" => nomi_agent_reminders::set_status(&mut conn, claims.sub, id, "cancelled").await,
        "snooze" => nomi_agent_reminders::snooze(&mut conn, claims.sub, id, req.minutes.unwrap_or(10)).await,
        _ => return Err((StatusCode::BAD_REQUEST, "action must be done, cancel or snooze".to_string())),
    }
    .map_err(internal)?;
    if !changed {
        return Err((StatusCode::NOT_FOUND, "reminder not found".to_string()));
    }
    Ok(StatusCode::NO_CONTENT)
}

/// Cancels an agent task scheduled for later (core scheduler).
pub async fn cancel_scheduled_task(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
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
        return Err((StatusCode::NOT_FOUND, "task not found".to_string()));
    }
    Ok(StatusCode::NO_CONTENT)
}
