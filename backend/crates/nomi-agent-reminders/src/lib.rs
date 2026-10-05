//! The Reminders agent: plain "remind me to …" reminders, kept in its own `reminders` table
//! (migration 0037). A due reminder is posted into its chat by `fire_due` (driven by the server's
//! reminders worker) as a message with Done / Snooze, with no model call. Agent tasks scheduled for
//! later still go through the core scheduler (`scheduled_jobs`), which this crate never touches.

use std::borrow::Cow;

use async_trait::async_trait;
use chrono::{DateTime, Datelike, FixedOffset, NaiveDateTime, TimeZone, Utc};
use serde::Serialize;
use serde_json::{json, Value};
use sqlx::pool::PoolConnection;
use sqlx::{PgConnection, PgPool, Postgres};
use uuid::Uuid;

use nomi_agent_core::SubAgent;
use nomi_llm::ToolDefinition;

pub const REMINDERS_AGENT_TYPE: &str = "reminders";
pub const REMINDERS_DISPLAY_NAME: &str = "Reminders";

const SYSTEM_PROMPT: &str = "You are Nomi's Reminders agent. You keep the user's reminders: add new ones \
     (add_reminder), show what's coming up (show_reminders), change, tick off or remove one \
     (edit_reminder, complete_reminder, remove_reminder). Resolve times like \"tomorrow at 4\" \
     against the current date and time you're given, in the user's timezone, and pass due_at with \
     that timezone's offset. For a repeating reminder set recurrence to daily, weekly (on due_at's \
     weekday) or monthly (on due_at's day). Confirm briefly what you set, with the day and time. \
     You only keep reminders; for work an agent should do later (\"every Monday, summarize my \
     spending\"), say that Nomi can schedule it. When you're done, call complete_task.";

const TOOL_NAMES: [&str; 5] = ["add_reminder", "show_reminders", "edit_reminder", "complete_reminder", "remove_reminder"];

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Reminder {
    pub id: Uuid,
    pub user_id: Uuid,
    pub session_id: Uuid,
    pub title: String,
    pub notes: Option<String>,
    pub due_at: DateTime<Utc>,
    pub recurrence: Option<String>,
    pub recurrence_weekday: Option<i16>,
    pub recurrence_day_of_month: Option<i16>,
    pub status: String,
    pub created_by: String,
    pub last_fired_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

const COLUMNS: &str = "id, user_id, session_id, title, notes, due_at, recurrence, recurrence_weekday, \
     recurrence_day_of_month, status, created_by, last_fired_at, created_at";

pub async fn user_timezone(conn: &mut PgConnection, user_id: Uuid) -> chrono_tz::Tz {
    sqlx::query_scalar::<_, String>("SELECT timezone FROM user_preferences WHERE user_id = $1")
        .bind(user_id)
        .fetch_optional(&mut *conn)
        .await
        .ok()
        .flatten()
        .and_then(|tz| tz.parse().ok())
        .unwrap_or(chrono_tz::UTC)
}

/// RFC 3339 with an offset, or a bare local date-time read in the user's timezone.
pub fn parse_due_at(text: &str, tz: chrono_tz::Tz) -> Result<DateTime<FixedOffset>, String> {
    if let Ok(at) = DateTime::parse_from_rfc3339(text) {
        return Ok(at);
    }
    for format in ["%Y-%m-%dT%H:%M:%S", "%Y-%m-%dT%H:%M", "%Y-%m-%d %H:%M"] {
        if let Ok(naive) = NaiveDateTime::parse_from_str(text, format) {
            let local = tz.from_local_datetime(&naive).earliest().ok_or("that local time doesn't exist (a clock change)")?;
            return Ok(local.fixed_offset());
        }
    }
    Err("due_at must be an ISO 8601 date and time".to_string())
}

/// The recurrence anchors (0 = Sunday weekday, day of month), taken from due_at in the user's own
/// offset so "every Monday" stays Monday where they live.
/// `(recurrence, weekday, day_of_month)` as stored on a reminder.
type RecurrenceFields = (Option<String>, Option<i16>, Option<i16>);

fn recurrence_fields(recurrence: Option<&str>, due_at: &DateTime<FixedOffset>) -> Result<RecurrenceFields, String> {
    match recurrence {
        None | Some("") | Some("none") | Some("once") => Ok((None, None, None)),
        Some("daily") => Ok((Some("daily".into()), None, None)),
        Some("weekly") => Ok((Some("weekly".into()), Some(due_at.weekday().num_days_from_sunday() as i16), None)),
        Some("monthly") => Ok((Some("monthly".into()), None, Some(due_at.day() as i16))),
        Some(other) => Err(format!("recurrence must be daily, weekly or monthly, not {other}")),
    }
}

/// The next time a repeating reminder is due, stepped in the user's own timezone so it keeps
/// its local weekday, day of month and clock time (across daylight-saving changes too).
pub fn next_due(due: DateTime<Utc>, recurrence: &str, day_of_month: Option<i16>, tz: chrono_tz::Tz) -> DateTime<Utc> {
    let local = due.with_timezone(&tz);
    let time = local.time();
    let date = local.date_naive();
    let next_date = match recurrence {
        "daily" => date + chrono::Days::new(1),
        "weekly" => date + chrono::Days::new(7),
        "monthly" => {
            let first_of_next = date.with_day(1).unwrap_or(date) + chrono::Months::new(1);
            let last_day = (first_of_next + chrono::Months::new(1)).pred_opt().map(|d| d.day()).unwrap_or(28);
            let day = (day_of_month.unwrap_or(date.day() as i16) as u32).min(last_day);
            first_of_next.with_day(day).unwrap_or(first_of_next)
        }
        _ => return due + chrono::Duration::days(1),
    };
    tz.from_local_datetime(&next_date.and_time(time))
        .earliest()
        .map(|t| t.with_timezone(&Utc))
        .unwrap_or_else(|| due + chrono::Duration::days(1))
}

pub struct NewReminder<'a> {
    pub title: &'a str,
    pub notes: Option<&'a str>,
    pub due_at: DateTime<FixedOffset>,
    pub recurrence: Option<&'a str>,
}

pub async fn add(
    conn: &mut PgConnection,
    user_id: Uuid,
    session_id: Uuid,
    new: NewReminder<'_>,
    created_by: &str,
) -> Result<Reminder, String> {
    let title = new.title.trim();
    if title.is_empty() {
        return Err("title is required".to_string());
    }
    if new.due_at.with_timezone(&Utc) <= Utc::now() {
        return Err("due_at must be in the future".to_string());
    }
    let (recurrence, weekday, day) = recurrence_fields(new.recurrence, &new.due_at)?;
    sqlx::query_as(&format!(
        "INSERT INTO reminders (user_id, session_id, title, notes, due_at, recurrence, recurrence_weekday, recurrence_day_of_month, created_by) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) RETURNING {COLUMNS}"
    ))
    .bind(user_id)
    .bind(session_id)
    .bind(title)
    .bind(new.notes.map(str::trim).filter(|n| !n.is_empty()))
    .bind(new.due_at.with_timezone(&Utc))
    .bind(recurrence)
    .bind(weekday)
    .bind(day)
    .bind(created_by)
    .fetch_one(&mut *conn)
    .await
    .map_err(|e| e.to_string())
}

pub async fn list(conn: &mut PgConnection, user_id: Uuid, statuses: &[&str], limit: i64) -> Result<Vec<Reminder>, sqlx::Error> {
    let statuses: Vec<String> = statuses.iter().map(|s| s.to_string()).collect();
    sqlx::query_as(&format!(
        "SELECT {COLUMNS} FROM reminders WHERE user_id = $1 AND status = ANY($2) \
         ORDER BY CASE WHEN status = 'active' THEN due_at END ASC, updated_at DESC LIMIT $3"
    ))
    .bind(user_id)
    .bind(&statuses)
    .bind(limit)
    .fetch_all(&mut *conn)
    .await
}

/// Marks one of the user's reminders done or cancelled. Returns false when there's no such
/// reminder of theirs still open.
pub async fn set_status(conn: &mut PgConnection, user_id: Uuid, id: Uuid, status: &str) -> Result<bool, sqlx::Error> {
    let changed = sqlx::query(
        "UPDATE reminders SET status = $3, updated_at = now() WHERE id = $1 AND user_id = $2 AND status IN ('active', 'fired')",
    )
    .bind(id)
    .bind(user_id)
    .bind(status)
    .execute(&mut *conn)
    .await?
    .rows_affected();
    Ok(changed > 0)
}

/// Pushes a reminder `minutes` from now (reopening a fired one-off).
pub async fn snooze(conn: &mut PgConnection, user_id: Uuid, id: Uuid, minutes: i64) -> Result<bool, sqlx::Error> {
    let changed = sqlx::query(
        "UPDATE reminders SET due_at = now() + make_interval(mins => $3), status = 'active', claimed_at = NULL, updated_at = now() \
         WHERE id = $1 AND user_id = $2 AND status IN ('active', 'fired')",
    )
    .bind(id)
    .bind(user_id)
    .bind(minutes.clamp(1, 7 * 24 * 60) as i32)
    .execute(&mut *conn)
    .await?
    .rows_affected();
    Ok(changed > 0)
}

/// Claims due reminders (any worker instance, without double-firing) and posts each into its chat.
/// Returns `(session_id, message_id)` for every reminder posted, for the caller to announce.
pub async fn fire_due(pool: &PgPool, limit: i64) -> Result<Vec<(Uuid, Uuid)>, sqlx::Error> {
    let due: Vec<Reminder> = sqlx::query_as(&format!(
        "WITH claimed AS (SELECT id FROM reminders WHERE status = 'active' AND claimed_at IS NULL AND due_at <= now() \
                          ORDER BY due_at FOR UPDATE SKIP LOCKED LIMIT $1) \
         UPDATE reminders SET claimed_at = now() WHERE id IN (SELECT id FROM claimed) RETURNING {COLUMNS}"
    ))
    .bind(limit)
    .fetch_all(pool)
    .await?;

    let mut posted = Vec::new();
    for reminder in due {
        let mut tx = pool.begin().await?;
        let block = nomi_agent_core::ContentBlock::Reminder {
            reminder_id: reminder.id,
            title: reminder.title.clone(),
            notes: reminder.notes.clone(),
            due_at: reminder.due_at,
            recurrence: reminder.recurrence.clone(),
        };
        let text = match &reminder.notes {
            Some(notes) => format!("⏰ {}: {notes}", reminder.title),
            None => format!("⏰ {}", reminder.title),
        };
        let message_id: Uuid = sqlx::query_scalar(
            "INSERT INTO messages (session_id, sender_channel_identity_id, content, content_blocks, agent_display_name) \
             VALUES ($1, NULL, $2, $3, $4) RETURNING id",
        )
        .bind(reminder.session_id)
        .bind(&text)
        .bind(json!([block]))
        .bind(REMINDERS_DISPLAY_NAME)
        .fetch_one(&mut *tx)
        .await?;
        match reminder.recurrence.as_deref() {
            Some(recurrence) => {
                let tz = user_timezone(&mut tx, reminder.user_id).await;
                // Advance past now, so a reminder missed while the server was down fires once.
                let mut next = reminder.due_at;
                while next <= Utc::now() {
                    next = next_due(next, recurrence, reminder.recurrence_day_of_month, tz);
                }
                sqlx::query("UPDATE reminders SET due_at = $2, claimed_at = NULL, last_fired_at = now(), updated_at = now() WHERE id = $1")
                    .bind(reminder.id)
                    .bind(next)
                    .execute(&mut *tx)
                    .await?;
            }
            None => {
                sqlx::query("UPDATE reminders SET status = 'fired', last_fired_at = now(), updated_at = now() WHERE id = $1")
                    .bind(reminder.id)
                    .execute(&mut *tx)
                    .await?;
            }
        }
        tx.commit().await?;
        posted.push((reminder.session_id, message_id));
    }
    Ok(posted)
}

fn describe(reminder: &Reminder, tz: chrono_tz::Tz) -> String {
    let when = reminder.due_at.with_timezone(&tz).format("%a %-d %b %H:%M");
    let repeat = reminder.recurrence.as_deref().map(|r| format!(", repeats {r}")).unwrap_or_default();
    format!("{} | {} | {when}{repeat} | {}", reminder.id, reminder.title, reminder.status)
}

fn tool_definitions() -> Vec<ToolDefinition> {
    vec![
        ToolDefinition {
            name: "add_reminder".into(),
            description: "Set a reminder for the user.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "title": {"type": "string", "description": "What to remind them of, short"},
                    "due_at": {"type": "string", "description": "ISO 8601 date and time with the user's offset"},
                    "recurrence": {"type": "string", "enum": ["daily", "weekly", "monthly"], "description": "Omit for a one-off"},
                    "notes": {"type": "string"}
                },
                "required": ["title", "due_at"]
            }),
        },
        ToolDefinition {
            name: "show_reminders".into(),
            description: "List the user's upcoming reminders (with ids).".into(),
            input_schema: json!({"type": "object", "properties": {}}),
        },
        ToolDefinition {
            name: "edit_reminder".into(),
            description: "Change a reminder's title, time, repeat or notes.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "id": {"type": "string"},
                    "title": {"type": "string"},
                    "due_at": {"type": "string"},
                    "recurrence": {"type": "string", "enum": ["none", "daily", "weekly", "monthly"]},
                    "notes": {"type": "string"}
                },
                "required": ["id"]
            }),
        },
        ToolDefinition {
            name: "complete_reminder".into(),
            description: "Tick a reminder off as done.".into(),
            input_schema: json!({"type": "object", "properties": {"id": {"type": "string"}}, "required": ["id"]}),
        },
        ToolDefinition {
            name: "remove_reminder".into(),
            description: "Cancel a reminder.".into(),
            input_schema: json!({"type": "object", "properties": {"id": {"type": "string"}}, "required": ["id"]}),
        },
    ]
}

fn reminder_id(input: &Value) -> Result<Uuid, String> {
    input.get("id").and_then(|v| v.as_str()).and_then(|s| s.parse().ok()).ok_or("id must be a reminder id from show_reminders".to_string())
}

async fn edit(conn: &mut PgConnection, user_id: Uuid, input: &Value, tz: chrono_tz::Tz) -> Result<String, String> {
    let id = reminder_id(input)?;
    let current: Reminder = sqlx::query_as(&format!("SELECT {COLUMNS} FROM reminders WHERE id = $1 AND user_id = $2 AND status IN ('active', 'fired')"))
        .bind(id)
        .bind(user_id)
        .fetch_optional(&mut *conn)
        .await
        .map_err(|e| e.to_string())?
        .ok_or("no open reminder with that id")?;
    let due_at = match input.get("due_at").and_then(|v| v.as_str()) {
        Some(text) => parse_due_at(text, tz)?,
        None => current.due_at.with_timezone(&tz).fixed_offset(),
    };
    let recurrence = match input.get("recurrence").and_then(|v| v.as_str()) {
        Some(r) => Some(r.to_string()),
        None => current.recurrence.clone(),
    };
    let (recurrence, weekday, day) = recurrence_fields(recurrence.as_deref(), &due_at)?;
    let title = input.get("title").and_then(|v| v.as_str()).map(str::trim).filter(|t| !t.is_empty()).unwrap_or(&current.title).to_string();
    let notes = input.get("notes").and_then(|v| v.as_str()).map(str::to_string).or(current.notes);
    sqlx::query(
        "UPDATE reminders SET title = $3, notes = $4, due_at = $5, recurrence = $6, recurrence_weekday = $7, \
         recurrence_day_of_month = $8, status = 'active', claimed_at = NULL, updated_at = now() WHERE id = $1 AND user_id = $2",
    )
    .bind(id)
    .bind(user_id)
    .bind(&title)
    .bind(&notes)
    .bind(due_at.with_timezone(&Utc))
    .bind(&recurrence)
    .bind(weekday)
    .bind(day)
    .execute(&mut *conn)
    .await
    .map_err(|e| e.to_string())?;
    Ok(format!("Updated: {title}, {}.", due_at.format("%a %-d %b %H:%M")))
}

pub struct RemindersAgent;

#[async_trait]
impl SubAgent for RemindersAgent {
    fn agent_type(&self) -> Cow<'static, str> {
        Cow::Borrowed(REMINDERS_AGENT_TYPE)
    }

    fn display_name(&self) -> Cow<'static, str> {
        Cow::Borrowed(REMINDERS_DISPLAY_NAME)
    }

    fn system_prompt(&self) -> Cow<'static, str> {
        Cow::Borrowed(SYSTEM_PROMPT)
    }

    fn tools(&self) -> Vec<ToolDefinition> {
        tool_definitions()
    }

    async fn execute_tool(
        &self,
        conn: &mut PoolConnection<Postgres>,
        session_id: Uuid,
        _agent_session_id: Uuid,
        user_id: Uuid,
        name: &str,
        input: Value,
    ) -> Result<nomi_agent_core::ToolOutcome, String> {
        let tz = user_timezone(conn, user_id).await;
        let text = match name {
            "add_reminder" => {
                let due_at = parse_due_at(input.get("due_at").and_then(|v| v.as_str()).ok_or("due_at is required")?, tz)?;
                let reminder = add(
                    conn,
                    user_id,
                    session_id,
                    NewReminder {
                        title: input.get("title").and_then(|v| v.as_str()).unwrap_or_default(),
                        notes: input.get("notes").and_then(|v| v.as_str()),
                        due_at,
                        recurrence: input.get("recurrence").and_then(|v| v.as_str()),
                    },
                    "agent",
                )
                .await?;
                format!("Reminder set: {}", describe(&reminder, tz))
            }
            "show_reminders" => {
                let upcoming = list(conn, user_id, &["active"], 25).await.map_err(|e| e.to_string())?;
                if upcoming.is_empty() {
                    "No upcoming reminders.".to_string()
                } else {
                    upcoming.iter().map(|r| describe(r, tz)).collect::<Vec<_>>().join("\n")
                }
            }
            "edit_reminder" => edit(conn, user_id, &input, tz).await?,
            "complete_reminder" | "remove_reminder" => {
                let status = if name == "complete_reminder" { "done" } else { "cancelled" };
                if !set_status(conn, user_id, reminder_id(&input)?, status).await.map_err(|e| e.to_string())? {
                    return Err("no open reminder with that id".to_string());
                }
                if status == "done" { "Ticked off.".to_string() } else { "Removed.".to_string() }
            }
            other => return Err(format!("unknown tool: {other}")),
        };
        Ok(nomi_agent_core::ToolOutcome::text(text))
    }

    fn intent_label(&self) -> Cow<'static, str> {
        Cow::Borrowed(REMINDERS_AGENT_TYPE)
    }

    fn intent_description(&self) -> Cow<'static, str> {
        Cow::Borrowed("the user wants to be reminded of something, or to see, change, tick off or cancel their reminders")
    }

    fn wants_current_time(&self) -> bool {
        true
    }

    // Its tools only touch the user's own reminders in this agent's own table.
    fn tool_needs_approval(&self, tool_name: &str) -> bool {
        !TOOL_NAMES.contains(&tool_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn due_at_without_an_offset_is_read_in_the_users_timezone() {
        let at = parse_due_at("2026-10-06T16:00", chrono_tz::Asia::Jakarta).unwrap();
        assert_eq!(at.to_rfc3339(), "2026-10-06T16:00:00+07:00");
        assert_eq!(parse_due_at("2026-10-06T16:00:00Z", chrono_tz::UTC).unwrap().to_rfc3339(), "2026-10-06T16:00:00+00:00");
        assert!(parse_due_at("tomorrow", chrono_tz::UTC).is_err());
    }

    #[test]
    fn repeats_step_in_local_time() {
        let jakarta = chrono_tz::Asia::Jakarta;
        let monday = parse_due_at("2026-10-05T06:00:00+07:00", jakarta).unwrap().with_timezone(&Utc);
        assert_eq!(next_due(monday, "weekly", None, jakarta).with_timezone(&jakarta).to_rfc3339(), "2026-10-12T06:00:00+07:00");
        assert_eq!(next_due(monday, "daily", None, jakarta).with_timezone(&jakarta).to_rfc3339(), "2026-10-06T06:00:00+07:00");
        let jan31 = parse_due_at("2026-01-31T09:00:00+07:00", jakarta).unwrap().with_timezone(&Utc);
        assert_eq!(next_due(jan31, "monthly", Some(31), jakarta).with_timezone(&jakarta).to_rfc3339(), "2026-02-28T09:00:00+07:00");
        // Across a DST change the clock time holds.
        let ny = chrono_tz::America::New_York;
        let before = parse_due_at("2026-10-31T09:00:00-04:00", ny).unwrap().with_timezone(&Utc);
        assert_eq!(next_due(before, "daily", None, ny).with_timezone(&ny).to_rfc3339(), "2026-11-01T09:00:00-05:00");
    }

    #[test]
    fn a_weekly_reminder_keeps_the_local_weekday() {
        // Monday 06:00 in Jakarta is still Sunday in UTC.
        let monday_morning = parse_due_at("2026-10-05T06:00:00+07:00", chrono_tz::UTC).unwrap();
        assert_eq!(recurrence_fields(Some("weekly"), &monday_morning).unwrap(), (Some("weekly".into()), Some(1), None));
        assert!(recurrence_fields(Some("hourly"), &monday_morning).is_err());
    }
}
