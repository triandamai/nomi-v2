//! Iris, the inspector: looks into a project's running app (its status, logs, tables and data,
//! read-only) through the server's `ProjectOps`.
//!
//! She works blind by default. Log lines and query rows go to the person, in the chat, as a table;
//! she gets only their shape (how many lines and errors, which columns, how many rows). Seeing
//! the contents takes a project admin's yes (`request_access`), kept in `agent_permissions` as
//! `inspector.<org>.<project>.read_logs` / `.read_data`.

use std::borrow::Cow;

use async_trait::async_trait;
use serde_json::{json, Map, Value};
use sqlx::pool::PoolConnection;
use sqlx::Postgres;
use uuid::Uuid;

use nomi_agent_core::agent_permissions::{self, Scope};
use nomi_agent_core::content_block::{ContentBlock, TableColumn, TableVariant};
use nomi_agent_core::project_ops::{DataSource, NoProjectOps, QueryRows, SharedProjectOps, DEFAULT_PUBLISH_ENVIRONMENT};
use nomi_agent_core::prompts::INSPECTOR_SYSTEM_PROMPT;
use nomi_agent_core::{SubAgent, ToolOutcome};
use nomi_llm::ToolDefinition;

pub const INSPECTOR_AGENT_TYPE: &str = "inspector";

/// Seeing what log lines say.
pub const READ_LOGS: &str = "read_logs";
/// Seeing the values a query returns.
pub const READ_DATA: &str = "read_data";

const APP_STATUS: &str = "app_status";
const READ_LOGS_TOOL: &str = "read_logs";
const LIST_TABLES: &str = "list_tables";
const QUERY_DATABASE: &str = "query_database";
const REQUEST_ACCESS: &str = "request_access";

/// Lines asked for when the call doesn't say.
const DEFAULT_LOG_LINES: u32 = 200;
/// The most lines one call reads.
const MAX_LOG_LINES: u32 = 1000;
/// The most lines or rows shown in the chat's table.
const MAX_SHOWN_ROWS: usize = 100;
/// The most rows passed to the model once it may read the data.
const MAX_ROWS_FOR_MODEL: usize = 50;

pub struct InspectorAgent {
    project_ops: SharedProjectOps,
}

impl InspectorAgent {
    pub fn new() -> Self {
        Self { project_ops: std::sync::Arc::new(NoProjectOps) }
    }

    pub fn with_project_ops(mut self, project_ops: SharedProjectOps) -> Self {
        self.project_ops = project_ops;
        self
    }
}

impl Default for InspectorAgent {
    fn default() -> Self {
        Self::new()
    }
}

fn project_property() -> Value {
    json!({"type": "string", "description": "The project's id."})
}

fn source_property() -> Value {
    json!({"type": "string", "enum": ["production", "development"], "description": "Which database: the published app's (production, the default) or the development one."})
}

fn tool_definitions() -> Vec<ToolDefinition> {
    vec![
        ToolDefinition {
            name: APP_STATUS.to_string(),
            description: "Whether the published app is running, at which address, and how its latest publishes went.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {"project_id": project_property(), "environment": {"type": "string", "description": "production unless the person named another."}},
                "required": ["project_id"]
            }),
        },
        ToolDefinition {
            name: READ_LOGS_TOOL.to_string(),
            description: "Read what the running app printed (newest last). Without read_logs access you get counts only (lines, errors, warnings); the person sees the lines.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_id": project_property(),
                    "environment": {"type": "string", "description": "production unless the person named another."},
                    "lines": {"type": "integer", "minimum": 1, "maximum": MAX_LOG_LINES, "description": "How many of the newest lines (default 200)."}
                },
                "required": ["project_id"]
            }),
        },
        ToolDefinition {
            name: LIST_TABLES.to_string(),
            description: "List the tables in the app's database, with roughly how many rows each has.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {"project_id": project_property(), "source": source_property()},
                "required": ["project_id"]
            }),
        },
        ToolDefinition {
            name: QUERY_DATABASE.to_string(),
            description: "Run one read-only SQL statement (SELECT, WITH, TABLE or VALUES) on the app's database. Without read_data access you get the columns, the row count and how many values each column has; the person sees the rows.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {"project_id": project_property(), "source": source_property(), "sql": {"type": "string"}},
                "required": ["project_id", "sql"]
            }),
        },
        ToolDefinition {
            name: REQUEST_ACCESS.to_string(),
            description: "Ask to see inside the app: read_logs (what log lines say) or read_data (the values queries return). The person approves; it lasts for this project until an admin turns it off.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_id": project_property(),
                    "permission": {"type": "string", "enum": [READ_LOGS, READ_DATA]},
                    "reason": {"type": "string", "description": "One sentence on why you need it."}
                },
                "required": ["project_id", "permission", "reason"]
            }),
        },
    ]
}

fn project_id_of(input: &Value) -> Result<Uuid, String> {
    input
        .get("project_id")
        .and_then(Value::as_str)
        .ok_or("project_id is required")?
        .trim()
        .parse()
        .map_err(|_| "project_id is not a valid UUID".to_string())
}

fn environment_of(input: &Value) -> String {
    input
        .get("environment")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|e| !e.is_empty())
        .unwrap_or(DEFAULT_PUBLISH_ENVIRONMENT)
        .to_string()
}

fn source_of(input: &Value) -> Result<DataSource, String> {
    match input.get("source").and_then(Value::as_str) {
        None => Ok(DataSource::Production),
        Some(text) => DataSource::parse(text).ok_or_else(|| "source is production or development".to_string()),
    }
}

/// Whether Iris may see `permission`'s contents on the project.
async fn allowed(conn: &mut PoolConnection<Postgres>, project_id: Uuid, permission: &str) -> bool {
    matches!(agent_permissions::scope_for(conn, INSPECTOR_AGENT_TYPE, project_id, permission).await, Ok(Some(Scope::Allow)))
}

/// How a log line reads: an error, a warning, or neither.
fn severity(line: &str) -> Option<&'static str> {
    let lower = line.to_lowercase();
    if ["error", "exception", "panic", "fatal", "uncaught", "unhandled"].iter().any(|w| lower.contains(w)) {
        Some("error")
    } else if lower.contains("warn") {
        Some("warning")
    } else {
        None
    }
}

/// What Iris is told about log lines she may not read.
fn blind_log_summary(lines: &[String]) -> String {
    let errors = lines.iter().filter(|l| severity(l) == Some("error")).count();
    let warnings = lines.iter().filter(|l| severity(l) == Some("warning")).count();
    let last_error = lines.iter().rposition(|l| severity(l) == Some("error"));
    let mut summary = format!(
        "{} log lines: {errors} look like errors, {warnings} like warnings. The person sees the lines in the chat; you can't read them without read_logs access.",
        lines.len()
    );
    if let Some(at) = last_error {
        summary.push_str(&format!(" The newest error is {} lines from the end.", lines.len() - at));
    }
    summary
}

/// The lines as a table for the chat (the newest last), with how each reads.
fn log_table(lines: &[String]) -> ContentBlock {
    let shown = &lines[lines.len().saturating_sub(MAX_SHOWN_ROWS)..];
    ContentBlock::Table {
        variant: TableVariant::Data,
        columns: vec![TableColumn { key: "level".into(), label: "Level".into() }, TableColumn { key: "line".into(), label: "Line".into() }],
        rows: shown.iter().map(|line| json!({"level": severity(line).unwrap_or(""), "line": line})).collect(),
        title: Some("Log lines".into()),
        source: Some(format!("The newest {} of {} lines", shown.len(), lines.len())),
    }
}

/// What Iris is told about rows she may not read: their shape only.
fn blind_rows_summary(result: &QueryRows) -> String {
    let filled: Vec<String> = result
        .columns
        .iter()
        .enumerate()
        .map(|(i, column)| {
            let values = result.rows.iter().filter(|row| row.get(i).is_some_and(|v| !v.is_null())).count();
            format!("{column} ({values} with a value)")
        })
        .collect();
    format!(
        "{} row{}{}. Columns: {}. The person sees the rows in the chat; you can't read the values without read_data access.",
        result.rows.len(),
        if result.rows.len() == 1 { "" } else { "s" },
        if result.truncated { " (more matched)" } else { "" },
        if filled.is_empty() { "none".to_string() } else { filled.join(", ") }
    )
}

/// The rows as a table for the chat.
fn rows_table(result: &QueryRows) -> ContentBlock {
    ContentBlock::Table {
        variant: TableVariant::Data,
        columns: result.columns.iter().map(|c| TableColumn { key: c.clone(), label: c.clone() }).collect(),
        rows: result
            .rows
            .iter()
            .take(MAX_SHOWN_ROWS)
            .map(|row| Value::Object(result.columns.iter().cloned().zip(row.iter().cloned()).collect::<Map<String, Value>>()))
            .collect(),
        title: Some("Query results".into()),
        source: Some(format!(
            "{} row{}{}",
            result.rows.len(),
            if result.rows.len() == 1 { "" } else { "s" },
            if result.rows.len() > MAX_SHOWN_ROWS || result.truncated { ", more matched than shown" } else { "" }
        )),
    }
}

/// The rows as JSON lines, for Iris once she may read them.
fn rows_for_model(result: &QueryRows) -> String {
    let lines: Vec<String> = result
        .rows
        .iter()
        .take(MAX_ROWS_FOR_MODEL)
        .map(|row| Value::Object(result.columns.iter().cloned().zip(row.iter().cloned()).collect::<Map<String, Value>>()).to_string())
        .collect();
    let more = result.rows.len().saturating_sub(MAX_ROWS_FOR_MODEL);
    let tail = if more > 0 || result.truncated { "\n(more rows matched than are shown)" } else { "" };
    format!("{} row(s):\n{}{tail}", result.rows.len(), nomi_llm::redact::redacted(lines.join("\n")))
}

#[async_trait]
impl SubAgent for InspectorAgent {
    fn agent_type(&self) -> Cow<'static, str> {
        Cow::Borrowed(INSPECTOR_AGENT_TYPE)
    }

    fn system_prompt(&self) -> Cow<'static, str> {
        Cow::Borrowed(INSPECTOR_SYSTEM_PROMPT)
    }

    fn tools(&self) -> Vec<ToolDefinition> {
        tool_definitions()
    }

    async fn execute_tool(
        &self,
        conn: &mut PoolConnection<Postgres>,
        _session_id: Uuid,
        _agent_session_id: Uuid,
        user_id: Uuid,
        name: &str,
        input: Value,
    ) -> Result<ToolOutcome, String> {
        let ops = self.project_ops.as_ref();
        let project_id = project_id_of(&input)?;
        match name {
            APP_STATUS => ops.status(user_id, project_id, &environment_of(&input)).await.map(ToolOutcome::text),
            READ_LOGS_TOOL => {
                let lines = input.get("lines").and_then(Value::as_u64).map(|n| n.clamp(1, MAX_LOG_LINES as u64) as u32).unwrap_or(DEFAULT_LOG_LINES);
                let logs = ops.logs(user_id, project_id, &environment_of(&input), lines).await?;
                if logs.is_empty() {
                    return Ok(ToolOutcome::text("The app printed nothing in that window."));
                }
                if allowed(conn, project_id, READ_LOGS).await {
                    return Ok(ToolOutcome::text(nomi_llm::redact::redacted(logs.join("\n"))));
                }
                Ok(ToolOutcome { display_text: blind_log_summary(&logs), block: Some(log_table(&logs)) })
            }
            LIST_TABLES => {
                let tables = ops.tables(user_id, project_id, source_of(&input)?).await?;
                if tables.is_empty() {
                    return Ok(ToolOutcome::text("The database has no tables yet."));
                }
                let listing = tables
                    .iter()
                    .map(|t| format!("{}.{} (~{} rows)", t.schema, t.name, t.rows_estimate.max(0)))
                    .collect::<Vec<_>>()
                    .join("\n");
                Ok(ToolOutcome::text(listing))
            }
            QUERY_DATABASE => {
                let sql = input.get("sql").and_then(Value::as_str).map(str::trim).filter(|s| !s.is_empty()).ok_or("sql is required")?;
                let result = ops.query(user_id, project_id, source_of(&input)?, sql).await?;
                if allowed(conn, project_id, READ_DATA).await {
                    return Ok(ToolOutcome { display_text: rows_for_model(&result), block: None });
                }
                Ok(ToolOutcome { display_text: blind_rows_summary(&result), block: Some(rows_table(&result)) })
            }
            REQUEST_ACCESS => {
                let permission = input.get("permission").and_then(Value::as_str).unwrap_or_default();
                if permission != READ_LOGS && permission != READ_DATA {
                    return Err("permission is read_logs or read_data".to_string());
                }
                if !agent_permissions::can_decide(conn, project_id, user_id).await.map_err(|e| format!("couldn't check the project: {e}"))? {
                    return Err("Only the project's admins can allow this; ask one of them.".to_string());
                }
                agent_permissions::set_for_project(conn, INSPECTOR_AGENT_TYPE, project_id, permission, Scope::Allow, user_id)
                    .await
                    .map_err(|e| format!("couldn't save the permission: {e}"))?;
                Ok(ToolOutcome::text(format!("Allowed: you can now {}. Run the call again to see it.", access_phrase(permission))))
            }
            other => Err(format!("unknown tool: {other}")),
        }
    }

    fn intent_label(&self) -> Cow<'static, str> {
        Cow::Borrowed(INSPECTOR_AGENT_TYPE)
    }

    fn intent_description(&self) -> Cow<'static, str> {
        Cow::Borrowed(
            "looking into a project's published or development app: is it up, why it errors or is slow, what its logs say, what's in its database",
        )
    }

    /// Reading the app's status, logs and tables only reads: no card for those. Seeing what's
    /// inside always asks.
    fn tool_needs_approval(&self, tool_name: &str) -> bool {
        tool_name == REQUEST_ACCESS
    }

    fn describe_action(&self, tool_name: &str, input: &Value, _locale: nomi_agent_core::Locale) -> Option<String> {
        if tool_name != REQUEST_ACCESS {
            return None;
        }
        let permission = input.get("permission").and_then(Value::as_str).unwrap_or_default();
        let reason = input.get("reason").and_then(Value::as_str).map(str::trim).filter(|r| !r.is_empty());
        let mut text = format!("Let Iris {} in this project", access_phrase(permission));
        if let Some(reason) = reason {
            text.push_str(&format!(": {reason}"));
        }
        Some(text)
    }
}

fn access_phrase(permission: &str) -> &'static str {
    if permission == READ_LOGS {
        "read what the app's logs say"
    } else {
        "see the values in the app's database"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn log_tables_say_how_much_of_the_log_they_show() {
        let lines = vec!["GET / 200".to_string(), "warn: slow".to_string()];
        let ContentBlock::Table { title, source, .. } = log_table(&lines) else { panic!("a table") };
        assert_eq!(title.as_deref(), Some("Log lines"));
        assert_eq!(source.as_deref(), Some("The newest 2 of 2 lines"));
    }

    #[test]
    fn blind_log_summaries_count_without_quoting() {
        let lines = vec!["GET / 200".to_string(), "Error: secret value leaked".to_string(), "warn: slow".to_string()];
        let summary = blind_log_summary(&lines);
        assert!(summary.starts_with("3 log lines: 1 look like errors, 1 like warnings."));
        assert!(!summary.contains("secret"));
    }

    #[test]
    fn blind_row_summaries_give_the_shape_not_the_values() {
        let result = QueryRows {
            columns: vec!["email".into(), "note".into()],
            rows: vec![vec![json!("a@b.c"), Value::Null], vec![json!("d@e.f"), json!("hi")]],
            truncated: false,
        };
        let summary = blind_rows_summary(&result);
        assert!(summary.starts_with("2 rows. Columns: email (2 with a value), note (1 with a value)."));
        assert!(!summary.contains("a@b.c"));
        let ContentBlock::Table { rows, title, source, .. } = rows_table(&result) else { panic!("a table") };
        assert_eq!(rows[0], json!({"email": "a@b.c", "note": null}));
        assert_eq!(title.as_deref(), Some("Query results"));
        assert_eq!(source.as_deref(), Some("2 rows"));
    }

    #[test]
    fn only_asking_for_access_needs_approval() {
        let agent = InspectorAgent::new();
        assert!(agent.tool_needs_approval(REQUEST_ACCESS));
        assert!(!agent.tool_needs_approval(QUERY_DATABASE));
        assert!(!agent.tool_needs_approval(READ_LOGS_TOOL));
    }
}
