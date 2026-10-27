//! Background task execution for every agent in Nomi's crew.
//!
//! Any crew member can start a background command with `run_background_task`, check active or
//! recent background tasks with `list_background_tasks`, and stop one with
//! `cancel_background_task`. Starting a background task is always non-blocking: it records a
//! `background_tasks` row, notifies `background_tasks_channel`, and returns the `task_id`
//! immediately so the calling agent can either keep working on another task in the same turn or
//! finish its turn and wait while Nomi remains free to handle other tasks. When the background
//! task finishes, the background worker automatically spawns Nomi (the main assistant) with the
//! result.

use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::pool::PoolConnection;
use sqlx::{PgConnection, Postgres};
use tokio::io::AsyncReadExt;
use uuid::Uuid;

use nomi_llm::ToolDefinition;
use nomi_realtime::{MqttPublisher, StreamEnvelope};

pub const RUN_BACKGROUND_TASK_TOOL_NAME: &str = "run_background_task";
pub const LIST_BACKGROUND_TASKS_TOOL_NAME: &str = "list_background_tasks";
pub const CANCEL_BACKGROUND_TASK_TOOL_NAME: &str = "cancel_background_task";

pub const BACKGROUND_TOOL_NAMES: &[&str] = &[
    RUN_BACKGROUND_TASK_TOOL_NAME,
    LIST_BACKGROUND_TASKS_TOOL_NAME,
    CANCEL_BACKGROUND_TASK_TOOL_NAME,
];

pub const BACKGROUND_NOTIFY_CHANNEL: &str = "background_tasks_channel";
pub const DEFAULT_BACKGROUND_TIMEOUT_SECS: i32 = 180;
pub const MAX_BACKGROUND_TIMEOUT_SECS: i32 = 600;
const OUTPUT_KEPT_BYTES: usize = 64 * 1024;
const OUTPUT_SHOWN_BYTES: usize = 12 * 1024;
const WAKEUP_PREFIX: &str = "[background_task_completed:";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedBackgroundTask {
    pub label: String,
    pub command: String,
    pub project_id: Option<Uuid>,
    pub timeout_seconds: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BackgroundWakeup {
    pub task_id: Uuid,
    pub agent_type: String,
    pub label: String,
    pub command: String,
    pub status: String,
    pub exit_code: Option<i32>,
    pub output: String,
}

impl BackgroundWakeup {
    /// Formats the note injected into Nomi's conversation when she is spawned by a finished
    /// background task.
    pub fn prompt_note(&self) -> String {
        let who = crate::crew::display_name_for(&self.agent_type);
        let code = self.exit_code.map(|c| format!(" (exit code {c})")).unwrap_or_default();
        let output_section = if self.output.trim().is_empty() {
            "(no output)".to_string()
        } else {
            format!("```\n{}\n```", tail_utf8(&self.output, OUTPUT_SHOWN_BYTES))
        };
        format!(
            "(Background task \"{label}\" [{id}] started by {who} (`{cmd}`) finished with status '{status}'{code}.\n\
             Output:\n{output_section}\n\
             You are Nomi, spawned because this background task finished. Review the result, tell the user how it went \
             or use the output to continue their work, and delegate to a specialist if another step is needed.)",
            label = self.label,
            id = self.task_id,
            cmd = self.command,
            status = self.status,
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackgroundCommandOutcome {
    pub status: &'static str,
    pub exit_code: Option<i32>,
    pub output: String,
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackgroundBoardRow {
    pub id: Uuid,
    pub agent_type: String,
    pub label: String,
    pub command: String,
    pub status: String,
    pub exit_code: Option<i32>,
    pub output: Option<String>,
    pub error: Option<String>,
    pub elapsed_secs: i64,
}

pub fn run_background_task_tool_definition() -> ToolDefinition {
    ToolDefinition {
        name: RUN_BACKGROUND_TASK_TOOL_NAME.to_string(),
        description: "Execute a command or operation in the background without blocking this turn. Returns a task_id \
                      immediately so you can either start working on another task right now while waiting, or finish \
                      your turn and wait while Nomi remains free to work on other tasks. When the background task \
                      finishes, it automatically spawns Nomi with the output to review and follow up."
            .to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "label": {
                    "type": "string",
                    "description": "Short human-readable description of what this background task is doing"
                },
                "command": {
                    "type": "string",
                    "description": "The command to execute in the background"
                },
                "project_id": {
                    "type": "string",
                    "description": "Optional project UUID when the command is for a specific project"
                },
                "timeout_seconds": {
                    "type": "integer",
                    "minimum": 1,
                    "maximum": MAX_BACKGROUND_TIMEOUT_SECS,
                    "description": "Maximum seconds the background command may run (default 180, max 600)"
                }
            },
            "required": ["label", "command"]
        }),
    }
}

pub fn list_background_tasks_tool_definition() -> ToolDefinition {
    ToolDefinition {
        name: LIST_BACKGROUND_TASKS_TOOL_NAME.to_string(),
        description: "List active and recent background tasks in this chat session, with their status and output.".to_string(),
        input_schema: json!({"type": "object", "properties": {}}),
    }
}

pub fn cancel_background_task_tool_definition() -> ToolDefinition {
    ToolDefinition {
        name: CANCEL_BACKGROUND_TASK_TOOL_NAME.to_string(),
        description: "Cancel a pending or running background task by its task_id.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "task_id": {
                    "type": "string",
                    "description": "The UUID of the background task to cancel"
                }
            },
            "required": ["task_id"]
        }),
    }
}

pub fn background_tool_definitions() -> Vec<ToolDefinition> {
    vec![
        run_background_task_tool_definition(),
        list_background_tasks_tool_definition(),
        cancel_background_task_tool_definition(),
    ]
}

pub fn parse_background_task_input(input: &Value) -> Result<ParsedBackgroundTask, String> {
    let label = input
        .get("label")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| "label is required".to_string())?
        .to_string();

    let command = input
        .get("command")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| "command is required".to_string())?
        .to_string();

    let project_id = match input.get("project_id").and_then(Value::as_str).map(str::trim).filter(|s| !s.is_empty()) {
        Some(raw) => Some(raw.parse::<Uuid>().map_err(|_| "project_id must be a valid UUID".to_string())?),
        None => None,
    };

    let timeout_seconds = match input.get("timeout_seconds").and_then(Value::as_i64) {
        Some(secs) if !(1..=MAX_BACKGROUND_TIMEOUT_SECS as i64).contains(&secs) => {
            return Err(format!("timeout_seconds must be between 1 and {MAX_BACKGROUND_TIMEOUT_SECS}"));
        }
        Some(secs) => secs as i32,
        None => DEFAULT_BACKGROUND_TIMEOUT_SECS,
    };

    Ok(ParsedBackgroundTask { label, command, project_id, timeout_seconds })
}

pub fn format_task_started_receipt(task_id: Uuid, label: &str) -> String {
    format!(
        "Background task {task_id} (\"{label}\") is now running in the background. \
         You can either start working on another task right now while waiting, or finish your turn and wait — \
         Nomi can also work on other tasks in the meantime, and when this background task finishes it will \
         automatically spawn Nomi with the result."
    )
}

pub fn format_wakeup_marker(wakeup: &BackgroundWakeup) -> String {
    let payload = serde_json::to_string(wakeup).unwrap_or_default();
    format!("{WAKEUP_PREFIX}{payload}]")
}

pub fn parse_wakeup_marker(text: &str) -> Option<BackgroundWakeup> {
    let trimmed = text.trim();
    let rest = trimmed.strip_prefix(WAKEUP_PREFIX)?;
    let json_str = rest.strip_suffix(']')?;
    serde_json::from_str(json_str).ok()
}

pub fn tail_utf8(output: &str, max_bytes: usize) -> &str {
    if output.len() <= max_bytes {
        return output;
    }
    let mut start = output.len() - max_bytes;
    while !output.is_char_boundary(start) {
        start += 1;
    }
    &output[start..]
}

/// Runs a shell command in the background with a timeout, capturing combined stdout and stderr
/// and redacting any secrets from the output.
pub async fn execute_shell_command(command: &str, timeout: Duration) -> BackgroundCommandOutcome {
    let mut cmd = tokio::process::Command::new("sh");
    cmd.arg("-c")
        .arg(command)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true);

    let mut child = match cmd.spawn() {
        Ok(child) => child,
        Err(e) => {
            let msg = nomi_llm::redact::redacted(format!("failed to spawn command: {e}"));
            return BackgroundCommandOutcome {
                status: "failed",
                exit_code: None,
                output: msg.clone(),
                error: Some(msg),
            };
        }
    };

    let mut stdout = child.stdout.take();
    let mut stderr = child.stderr.take();

    let wait_future = async {
        let mut out_bytes = Vec::new();
        let mut err_bytes = Vec::new();
        if let Some(ref mut out) = stdout {
            let _ = out.read_to_end(&mut out_bytes).await;
        }
        if let Some(ref mut err) = stderr {
            let _ = err.read_to_end(&mut err_bytes).await;
        }
        let status = child.wait().await;
        (status, out_bytes, err_bytes)
    };

    match tokio::time::timeout(timeout, wait_future).await {
        Ok((Ok(exit_status), out_bytes, err_bytes)) => {
            let code = exit_status.code();
            let mut combined = String::from_utf8_lossy(&out_bytes).into_owned();
            let err_str = String::from_utf8_lossy(&err_bytes);
            if !err_str.trim().is_empty() {
                if !combined.is_empty() && !combined.ends_with('\n') {
                    combined.push('\n');
                }
                combined.push_str(&err_str);
            }
            let kept = tail_utf8(&combined, OUTPUT_KEPT_BYTES);
            let redacted = nomi_llm::redact::redacted(kept);
            if code == Some(0) {
                BackgroundCommandOutcome {
                    status: "completed",
                    exit_code: Some(0),
                    output: redacted,
                    error: None,
                }
            } else {
                let err_msg = format!("command exited with code {}", code.map_or_else(|| "?".to_string(), |c| c.to_string()));
                BackgroundCommandOutcome {
                    status: "failed",
                    exit_code: code,
                    output: redacted,
                    error: Some(err_msg),
                }
            }
        }
        Ok((Err(e), _, _)) => {
            let msg = nomi_llm::redact::redacted(format!("command wait failed: {e}"));
            BackgroundCommandOutcome {
                status: "failed",
                exit_code: None,
                output: msg.clone(),
                error: Some(msg),
            }
        }
        Err(_) => {
            let _ = child.kill().await;
            let msg = format!("timed out after {}s", timeout.as_secs());
            BackgroundCommandOutcome {
                status: "failed",
                exit_code: None,
                output: msg.clone(),
                error: Some(msg),
            }
        }
    }
}

pub async fn start_background_task(
    conn: &mut PoolConnection<Postgres>,
    mqtt: Option<(&MqttPublisher, Uuid)>,
    session_id: Uuid,
    user_id: Uuid,
    agent_type: &str,
    input: &Value,
) -> Result<String, String> {
    let parsed = parse_background_task_input(input)?;

    let id: Uuid = sqlx::query_scalar(
        "INSERT INTO background_tasks (session_id, user_id, agent_type, label, command, project_id, timeout_seconds) \
         VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING id",
    )
    .bind(session_id)
    .bind(user_id)
    .bind(agent_type)
    .bind(&parsed.label)
    .bind(&parsed.command)
    .bind(parsed.project_id)
    .bind(parsed.timeout_seconds)
    .fetch_one(&mut **conn)
    .await
    .map_err(|e| format!("failed to start background task: {e}"))?;

    let _ = sqlx::query("SELECT pg_notify($1, $2)")
        .bind(BACKGROUND_NOTIFY_CHANNEL)
        .bind(id.to_string())
        .execute(&mut **conn)
        .await;

    if let Some((publisher, _)) = mqtt {
        let _ = publisher
            .publish(session_id, &StreamEnvelope::AgentDelegationUpdated { delegation_id: id })
            .await;
    }

    Ok(format_task_started_receipt(id, &parsed.label))
}

pub async fn list_background_tasks(
    conn: &mut PoolConnection<Postgres>,
    session_id: Uuid,
    user_id: Uuid,
) -> Result<String, String> {
    let rows: Vec<(Uuid, String, String, String, String, Option<i32>, Option<String>, Option<String>)> = sqlx::query_as(
        "SELECT id, agent_type, label, command, status, exit_code, output, error \
         FROM background_tasks \
         WHERE session_id = $1 AND user_id = $2 \
         ORDER BY created_at DESC LIMIT 10",
    )
    .bind(session_id)
    .bind(user_id)
    .fetch_all(&mut **conn)
    .await
    .map_err(|e| e.to_string())?;

    if rows.is_empty() {
        return Ok("No background tasks in this chat yet.".to_string());
    }

    let mut out = String::new();
    for (id, agent_type, label, command, status, exit_code, output, error) in rows {
        let who = crate::crew::display_name_for(&agent_type);
        let code = exit_code.map(|c| format!(", exit {c}")).unwrap_or_default();
        out.push_str(&format!("- {id} [{status}{code}] ({who}): \"{label}\" (`{command}`)"));
        if let Some(err) = error.filter(|s| !s.trim().is_empty()) {
            out.push_str(&format!(" — error: {}", clip_one_line(&err, 160)));
        } else if let Some(text) = output.filter(|s| !s.trim().is_empty()) {
            out.push_str(&format!(" — output: {}", clip_one_line(&text, 200)));
        }
        out.push('\n');
    }
    Ok(out.trim_end().to_string())
}

pub async fn cancel_background_task(
    conn: &mut PoolConnection<Postgres>,
    session_id: Uuid,
    user_id: Uuid,
    input: &Value,
) -> Result<String, String> {
    let raw_id = input
        .get("task_id")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| "task_id is required".to_string())?;
    let task_id: Uuid = raw_id.parse().map_err(|_| "task_id must be a valid UUID".to_string())?;

    let affected = sqlx::query(
        "UPDATE background_tasks SET status = 'cancelled', completed_at = now(), error = 'Cancelled' \
         WHERE id = $1 AND session_id = $2 AND user_id = $3 AND status IN ('pending', 'running')",
    )
    .bind(task_id)
    .bind(session_id)
    .bind(user_id)
    .execute(&mut **conn)
    .await
    .map_err(|e| e.to_string())?
    .rows_affected();

    if affected == 0 {
        return Err("no pending or running background task with that task_id".to_string());
    }

    Ok(format!("Background task {task_id} cancelled."))
}

pub async fn fetch_background_board_rows(conn: &mut PgConnection, session_id: Uuid) -> Vec<BackgroundBoardRow> {
    let rows: Vec<(Uuid, String, String, String, String, Option<i32>, Option<String>, Option<String>, i64)> = sqlx::query_as(
        "SELECT id, agent_type, label, command, status, exit_code, output, error, \
                EXTRACT(EPOCH FROM (now() - COALESCE(completed_at, claimed_at, created_at)))::bigint \
         FROM background_tasks \
         WHERE session_id = $1 \
           AND (status IN ('pending', 'running') OR completed_at > now() - interval '20 minutes') \
         ORDER BY created_at DESC LIMIT 8",
    )
    .bind(session_id)
    .fetch_all(conn)
    .await
    .unwrap_or_default();

    rows.into_iter()
        .map(|(id, agent_type, label, command, status, exit_code, output, error, elapsed_secs)| BackgroundBoardRow {
            id,
            agent_type,
            label,
            command,
            status,
            exit_code,
            output,
            error,
            elapsed_secs,
        })
        .collect()
}

pub fn format_background_board_lines(rows: &[BackgroundBoardRow]) -> Vec<String> {
    let mut lines = Vec::new();
    for row in rows {
        let who = crate::crew::display_name_for(&row.agent_type);
        let minutes = (row.elapsed_secs / 60).max(0);
        let label = clip_one_line(&row.label, 120);
        let cmd = clip_one_line(&row.command, 100);
        let line = match row.status.as_str() {
            "pending" => format!("- Background task [{id}] ({who}): queued \"{label}\" (`{cmd}`)", id = row.id),
            "running" => format!("- Background task [{id}] ({who}): running \"{label}\" (`{cmd}`, started {minutes} min ago)", id = row.id),
            "completed" => {
                let snippet = clip_one_line(row.output.as_deref().unwrap_or(""), 160);
                if snippet.is_empty() {
                    format!("- Background task [{id}] ({who}): finished \"{label}\" {minutes} min ago", id = row.id)
                } else {
                    format!("- Background task [{id}] ({who}): finished \"{label}\" {minutes} min ago: {snippet}", id = row.id)
                }
            }
            "failed" => {
                let reason = clip_one_line(row.error.as_deref().or(row.output.as_deref()).unwrap_or("failed"), 160);
                format!("- Background task [{id}] ({who}): failed \"{label}\" {minutes} min ago ({reason})", id = row.id)
            }
            _ => continue,
        };
        lines.push(line);
    }
    lines
}

fn clip_one_line(text: &str, max: usize) -> String {
    let flat = text.trim().replace('\n', " ");
    if flat.chars().count() <= max {
        flat
    } else {
        format!("{}…", flat.chars().take(max).collect::<String>())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_valid_background_task_input_and_rejects_invalid_fields() {
        let project_id = Uuid::new_v4();
        let parsed = parse_background_task_input(&json!({
            "label": "  Run build checks ",
            "command": " npm run check ",
            "project_id": project_id.to_string(),
            "timeout_seconds": 120
        }))
        .unwrap();
        assert_eq!(parsed.label, "Run build checks");
        assert_eq!(parsed.command, "npm run check");
        assert_eq!(parsed.project_id, Some(project_id));
        assert_eq!(parsed.timeout_seconds, 120);

        let defaults = parse_background_task_input(&json!({
            "label": "Quick check",
            "command": "echo ok"
        }))
        .unwrap();
        assert_eq!(defaults.timeout_seconds, DEFAULT_BACKGROUND_TIMEOUT_SECS);
        assert_eq!(defaults.project_id, None);

        assert!(parse_background_task_input(&json!({"label": "", "command": "echo ok"})).is_err());
        assert!(parse_background_task_input(&json!({"label": "x", "command": "   "})).is_err());
        assert!(parse_background_task_input(&json!({"label": "x", "command": "echo", "project_id": "not-a-uuid"})).is_err());
        assert!(parse_background_task_input(&json!({"label": "x", "command": "echo", "timeout_seconds": 0})).is_err());
        assert!(parse_background_task_input(&json!({"label": "x", "command": "echo", "timeout_seconds": 9999})).is_err());
    }

    #[test]
    fn receipt_explains_non_blocking_wait_or_work_and_nomi_spawn() {
        let id = Uuid::new_v4();
        let receipt = format_task_started_receipt(id, "Compile assets");
        assert!(receipt.contains(&id.to_string()));
        assert!(receipt.contains("Compile assets"));
        assert!(receipt.contains("start working on another task right now while waiting"));
        assert!(receipt.contains("finish your turn and wait"));
        assert!(receipt.contains("automatically spawn Nomi"));
    }

    #[test]
    fn wakeup_marker_round_trips_and_formats_prompt_note_for_nomi() {
        let wakeup = BackgroundWakeup {
            task_id: Uuid::new_v4(),
            agent_type: "coding".to_string(),
            label: "Run test suite".to_string(),
            command: "npm test".to_string(),
            status: "completed".to_string(),
            exit_code: Some(0),
            output: "All 42 tests passed".to_string(),
        };
        let marker = format_wakeup_marker(&wakeup);
        let parsed = parse_wakeup_marker(&marker).expect("should parse wakeup marker");
        assert_eq!(parsed, wakeup);

        let note = parsed.prompt_note();
        assert!(note.contains("Koda"));
        assert!(note.contains("Run test suite"));
        assert!(note.contains("All 42 tests passed"));
        assert!(note.contains("spawned because this background task finished"));
        assert!(parse_wakeup_marker("ordinary user message").is_none());
    }

    #[tokio::test]
    async fn execute_shell_command_captures_output_exit_code_and_timeout() {
        let ok = execute_shell_command("printf 'hello background'", Duration::from_secs(5)).await;
        assert_eq!(ok.status, "completed");
        assert_eq!(ok.exit_code, Some(0));
        assert_eq!(ok.output, "hello background");
        assert!(ok.error.is_none());

        let fail = execute_shell_command("echo 'bad' >&2; exit 2", Duration::from_secs(5)).await;
        assert_eq!(fail.status, "failed");
        assert_eq!(fail.exit_code, Some(2));
        assert!(fail.output.contains("bad"));
        assert!(fail.error.as_deref().unwrap_or_default().contains("code 2"));

        let timed_out = execute_shell_command("sleep 5", Duration::from_millis(100)).await;
        assert_eq!(timed_out.status, "failed");
        assert!(timed_out.exit_code.is_none());
        assert!(timed_out.output.contains("timed out"));
    }

    #[test]
    fn formats_background_board_lines_for_crew_awareness() {
        let id = Uuid::new_v4();
        let rows = vec![
            BackgroundBoardRow {
                id,
                agent_type: "money".to_string(),
                label: "Export ledger".to_string(),
                command: "node export.js".to_string(),
                status: "running".to_string(),
                exit_code: None,
                output: None,
                error: None,
                elapsed_secs: 125,
            },
            BackgroundBoardRow {
                id,
                agent_type: "coding".to_string(),
                label: "Typecheck".to_string(),
                command: "npm run check".to_string(),
                status: "completed".to_string(),
                exit_code: Some(0),
                output: Some("0 errors, 0 warnings".to_string()),
                error: None,
                elapsed_secs: 30,
            },
        ];
        let lines = format_background_board_lines(&rows);
        assert_eq!(lines.len(), 2);
        assert!(lines[0].contains("Dana") && lines[0].contains("running \"Export ledger\""));
        assert!(lines[1].contains("Koda") && lines[1].contains("0 errors, 0 warnings"));
    }
}
