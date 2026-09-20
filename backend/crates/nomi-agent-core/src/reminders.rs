use sqlx::pool::PoolConnection;
use sqlx::Postgres;
use uuid::Uuid;

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
/// it's validated here instead, the same way `parse_todo_items`/`parse_table_input` validate
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
