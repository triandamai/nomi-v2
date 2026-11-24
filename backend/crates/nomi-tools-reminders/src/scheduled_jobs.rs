//! The crew's scheduled jobs (`scheduled_jobs`): work an agent will run later, such as a weekly
//! spending summary, run by the server's scheduler worker. The engine offers these to agents that
//! opt in as `create_reminder`, `list_reminders` and `cancel_reminder`. The person's own reminders
//! are a separate thing, owned by `nomi-agent-reminders`.

use chrono::Datelike;
use nomi_llm::ToolDefinition;
use serde_json::json;
use sqlx::pool::PoolConnection;
use sqlx::Postgres;
use uuid::Uuid;

mod tools;
pub use tools::{tools, GROUP};

pub const CREATE_REMINDER_TOOL_NAME: &str = "create_reminder";
pub const LIST_REMINDERS_TOOL_NAME: &str = "list_reminders";
pub const CANCEL_REMINDER_TOOL_NAME: &str = "cancel_reminder";

pub async fn get_user_timezone(conn: &mut PoolConnection<Postgres>, user_id: Uuid) -> String {
    sqlx::query_scalar("SELECT timezone FROM user_preferences WHERE user_id = $1")
        .bind(user_id)
        .fetch_optional(&mut **conn)
        .await
        .ok()
        .flatten()
        .unwrap_or_else(|| "UTC".to_string())
}

/// Parses and validates the recurrence-related fields together, since `recurrence_weekday`/
/// `recurrence_day_of_month` are each required exactly when their matching `recurrence` value is
/// given — not enforced by a DB CHECK constraint (see the design spec's Data Model section), so
/// it's validated here instead, the same way `parse_todo_items`/`views::parse_show_input` validate
/// their own tools' input in engine.rs.
fn parse_recurrence(input: &serde_json::Value) -> Result<(Option<String>, Option<i16>, Option<i16>), String> {
    let recurrence = input.get("recurrence").and_then(|v| v.as_str());
    match recurrence {
        None => Ok((None, None, None)),
        Some("daily") => Ok((Some("daily".to_string()), None, None)),
        Some("weekly") => {
            let weekday = input
                .get("recurrence_weekday")
                .and_then(|v| v.as_i64())
                .ok_or("recurrence_weekday is required when recurrence is weekly")?;
            if !(0..=6).contains(&weekday) {
                return Err("recurrence_weekday must be between 0 and 6".to_string());
            }
            Ok((Some("weekly".to_string()), Some(weekday as i16), None))
        }
        Some("monthly") => {
            let day = input
                .get("recurrence_day_of_month")
                .and_then(|v| v.as_i64())
                .ok_or("recurrence_day_of_month is required when recurrence is monthly")?;
            if !(1..=31).contains(&day) {
                return Err("recurrence_day_of_month must be between 1 and 31".to_string());
            }
            Ok((Some("monthly".to_string()), None, Some(day as i16)))
        }
        Some(other) => Err(format!("recurrence must be daily, weekly, or monthly, got '{other}'")),
    }
}

pub async fn create_reminder(
    conn: &mut PoolConnection<Postgres>,
    session_id: Uuid,
    user_id: Uuid,
    created_by_agent_type: &str,
    input: &serde_json::Value,
) -> Result<String, String> {
    let run_at_str = input.get("run_at").and_then(|v| v.as_str()).ok_or("run_at is required")?;
    let run_at = chrono::DateTime::parse_from_rfc3339(run_at_str)
        .map_err(|e| format!("run_at must be a valid ISO 8601 datetime: {e}"))?
        .with_timezone(&chrono::Utc);
    let label = input.get("label").and_then(|v| v.as_str()).ok_or("label is required")?.to_string();
    let prompt = input.get("prompt").and_then(|v| v.as_str()).ok_or("prompt is required")?.to_string();
    let target_agent = input.get("target_agent").and_then(|v| v.as_str()).ok_or("target_agent is required")?.to_string();
    let (recurrence, recurrence_weekday, recurrence_day_of_month) = parse_recurrence(input)?;

    sqlx::query(
        "INSERT INTO scheduled_jobs \
             (session_id, user_id, created_by_agent_type, target_agent_type, label, prompt, run_at, \
              recurrence, recurrence_weekday, recurrence_day_of_month) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
    )
    .bind(session_id)
    .bind(user_id)
    .bind(created_by_agent_type)
    .bind(&target_agent)
    .bind(&label)
    .bind(&prompt)
    .bind(run_at)
    .bind(&recurrence)
    .bind(recurrence_weekday)
    .bind(recurrence_day_of_month)
    .execute(&mut **conn)
    .await
    .map_err(|e| e.to_string())?;

    Ok(format!("Reminder set for {} ({}).", run_at.to_rfc3339(), label))
}

pub async fn list_reminders(conn: &mut PoolConnection<Postgres>, user_id: Uuid) -> Result<String, String> {
    let rows: Vec<(Uuid, String, chrono::DateTime<chrono::Utc>, Option<String>)> = sqlx::query_as(
        "SELECT id, label, run_at, recurrence FROM scheduled_jobs \
         WHERE user_id = $1 AND status = 'active' ORDER BY run_at",
    )
    .bind(user_id)
    .fetch_all(&mut **conn)
    .await
    .map_err(|e| e.to_string())?;

    if rows.is_empty() {
        return Ok("No active reminders.".to_string());
    }

    let mut out = String::new();
    for (id, label, run_at, recurrence) in rows {
        let cadence = recurrence.map(|r| format!(", repeats {r}")).unwrap_or_default();
        out.push_str(&format!("- {id}: \"{label}\" at {}{cadence}\n", run_at.to_rfc3339()));
    }
    Ok(out)
}

pub async fn cancel_reminder(
    conn: &mut PoolConnection<Postgres>,
    user_id: Uuid,
    input: &serde_json::Value,
) -> Result<String, String> {
    let reminder_id_str = input.get("reminder_id").and_then(|v| v.as_str()).ok_or("reminder_id is required")?;
    let reminder_id: Uuid = reminder_id_str.parse().map_err(|_| "reminder_id must be a valid UUID".to_string())?;

    let result = sqlx::query(
        "UPDATE scheduled_jobs SET status = 'cancelled', cancelled_at = now() \
         WHERE id = $1 AND user_id = $2 AND status = 'active'",
    )
    .bind(reminder_id)
    .bind(user_id)
    .execute(&mut **conn)
    .await
    .map_err(|e| e.to_string())?;

    if result.rows_affected() == 0 {
        return Err("no active reminder with that id".to_string());
    }
    Ok("Reminder cancelled.".to_string())
}

/// The next time a recurring schedule fires after `from` (`weekday` 0 = Sunday). Shared by the
/// scheduler (agent tasks) and the Reminders agent's worker.
pub fn next_occurrence(
    from: chrono::DateTime<chrono::Utc>,
    recurrence: &str,
    weekday: Option<i16>,
    day_of_month: Option<i16>,
) -> chrono::DateTime<chrono::Utc> {
    match recurrence {
        "daily" => from + chrono::Duration::days(1),
        "weekly" => {
            // `weekday` is 0=Sunday..6=Saturday (matches the create_reminder tool schema's
            // description). chrono's Weekday::num_days_from_sunday() uses the same convention.
            let target = weekday.unwrap_or(0) as u32;
            let mut candidate = from + chrono::Duration::days(1);
            while candidate.weekday().num_days_from_sunday() != target {
                candidate += chrono::Duration::days(1);
            }
            candidate
        }
        "monthly" => {
            let day = day_of_month.unwrap_or(1) as u32;
            let (mut year, mut month) = (from.year(), from.month());
            month += 1;
            if month > 12 {
                month = 1;
                year += 1;
            }
            let last_day_of_month = chrono::NaiveDate::from_ymd_opt(year, month, 1)
                .unwrap()
                .checked_add_months(chrono::Months::new(1))
                .unwrap()
                .pred_opt()
                .unwrap()
                .day();
            let clamped_day = day.min(last_day_of_month);
            // Step through day=1 first: with_month()/with_year() reject a result that isn't a
            // real calendar date, and `from`'s own day-of-month (e.g. 31) may not exist in the
            // target month — day=1 always does, in every month.
            from.with_day(1).unwrap().with_year(year).unwrap().with_month(month).unwrap().with_day(clamped_day).unwrap()
        }
        _ => from,
    }
}

pub fn create_reminder_tool_definition(targets: &[String]) -> ToolDefinition {
    ToolDefinition {
        name: CREATE_REMINDER_TOOL_NAME.to_string(),
        description: "Schedule a reminder to fire at a specific future time. Resolve any \
                       relative time the user gives (\"tomorrow\", \"in an hour\") to an \
                       absolute ISO 8601 datetime yourself, using the current date/time and the \
                       user's timezone given in your system prompt. Before you set it, make sure you \
                       know what it's for, when, and whether it's one-time or repeating (and how \
                       often): ask with ask_user for whatever they didn't say, all of it in one card.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "run_at": {"type": "string", "description": "Absolute ISO 8601 datetime, e.g. 2026-09-21T11:00:00-04:00"},
                "label": {"type": "string", "description": "Short human label, e.g. \"take a bath\""},
                "prompt": {"type": "string", "description": "The instruction the target agent will act on when this fires"},
                "target_agent": {"type": "string", "enum": targets, "description": "Which agent takes this job when it fires"},
                "recurrence": {"type": "string", "enum": ["daily", "weekly", "monthly"], "description": "Omit for a one-time reminder"},
                "recurrence_weekday": {"type": "integer", "minimum": 0, "maximum": 6, "description": "Required when recurrence is weekly (0 = Sunday)"},
                "recurrence_day_of_month": {"type": "integer", "minimum": 1, "maximum": 31, "description": "Required when recurrence is monthly"}
            },
            "required": ["run_at", "label", "prompt", "target_agent"]
        }),
    }
}

pub fn list_reminders_tool_definition() -> ToolDefinition {
    ToolDefinition {
        name: LIST_REMINDERS_TOOL_NAME.to_string(),
        description: "List your active reminders.".to_string(),
        input_schema: json!({"type": "object", "properties": {}}),
    }
}

pub fn cancel_reminder_tool_definition() -> ToolDefinition {
    ToolDefinition {
        name: CANCEL_REMINDER_TOOL_NAME.to_string(),
        description: "Cancel an active reminder by id. Call list_reminders first if you don't \
                       already know the id.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "reminder_id": {"type": "string", "description": "The reminder's id, from a prior list_reminders call"}
            },
            "required": ["reminder_id"]
        }),
    }
}
