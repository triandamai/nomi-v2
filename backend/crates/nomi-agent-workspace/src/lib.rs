//! The Workspace agent: works in the user's own Google account (Gmail, Sheets, Docs, Drive,
//! Calendar). Each person connects their own account from Connections; every tool call loads the
//! token of the user the turn belongs to, so a hand-off only ever reaches the asker's account.
//!
//! Storage (docs/agents/storage.md): `workspace_connections`, `workspace_oauth_states`,
//! `workspace_activity`.

pub mod connection;
pub mod google;

use std::borrow::Cow;
use std::sync::{Arc, OnceLock};

use async_trait::async_trait;
use serde_json::{json, Value};
use sqlx::pool::PoolConnection;
use sqlx::Postgres;
use uuid::Uuid;

use nomi_agent_core::content_block::{ContentBlock, ToolOutcome};
use nomi_agent_core::SubAgent;
use nomi_llm::ToolDefinition;

use connection::{AccessError, GoogleConfig};
use google::{Google, GoogleError};

pub const WORKSPACE_AGENT_TYPE: &str = "workspace";

const SYSTEM_PROMPT: &str = "You are Tara, the crew member who works in the user's own Google \
account: Gmail, Sheets, Docs, Drive and Calendar. You only ever see the account of the person \
you are helping.

How you work:
- Find before you act: search Gmail or Drive for what the user means, read it, then do the job.
- Sheets: read the tab names first (sheets_read without a range) if you don't know the layout; \
append rows under the existing headers rather than overwriting.
- Email: draft by default. Use gmail_send only when the user clearly asked to send; they approve \
every send in chat. Keep emails short, in the user's voice and language.
- Never invent message, file or spreadsheet ids: take them from search results.
- After acting, say plainly what you changed and where (with the link when you have one). For \
lists of emails, files or events, use show_table.
- If a tool says Google isn't connected or a service isn't allowed, a Connect card is already \
shown to the user: tell them in one short sentence and stop.";

/// One shared HTTP client for every registry this process builds.
fn http_client() -> reqwest::Client {
    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    CLIENT.get_or_init(reqwest::Client::new).clone()
}

pub struct WorkspaceAgent {
    http: reqwest::Client,
    /// `None` until whoever runs Nomi adds Google OAuth credentials.
    config: Option<Arc<GoogleConfig>>,
    key: [u8; 32],
}

impl WorkspaceAgent {
    pub fn new(http: reqwest::Client, config: Option<Arc<GoogleConfig>>, key: [u8; 32]) -> Self {
        Self { http, config, key }
    }

    /// Google settings and the encryption key from the environment (see `GoogleConfig::from_env`).
    pub fn from_env() -> Self {
        let key = std::env::var("SETTINGS_ENCRYPTION_KEY").ok().and_then(|k| nomi_settings::crypto::parse_key(&k)).unwrap_or([0u8; 32]);
        Self::new(http_client(), GoogleConfig::from_env().map(Arc::new), key)
    }
}

/// Which service each tool needs.
fn tool_service(name: &str) -> Option<&'static str> {
    Some(match name {
        "gmail_search" | "gmail_read" | "gmail_draft" | "gmail_send" => "gmail",
        "sheets_read" | "sheets_append" | "sheets_update" | "sheets_create" => "sheets",
        "docs_read" | "docs_create" | "docs_append" => "docs",
        "drive_search" => "drive",
        "calendar_events" | "calendar_add_event" => "calendar",
        _ => return None,
    })
}

fn tool(name: &str, description: &str, schema: Value) -> ToolDefinition {
    ToolDefinition { name: name.to_string(), description: description.to_string(), input_schema: schema }
}

fn email_schema() -> Value {
    json!({"type": "object", "properties": {
        "to": {"type": "string", "description": "Recipient email address(es), comma-separated."},
        "subject": {"type": "string", "description": "Leave empty when replying to keep the thread's subject."},
        "body": {"type": "string", "description": "Plain-text email body."},
        "reply_to_message_id": {"type": "string", "description": "Gmail message id to reply to (threads the email)."}
    }, "required": ["to", "body"]})
}

fn rows_schema() -> Value {
    json!({"type": "array", "items": {"type": "array", "items": {}}, "description": "Rows of cell values, e.g. [[\"2026-03-12\", \"Ubud Villas\", 2850000]]."})
}

fn str_arg<'a>(input: &'a Value, key: &str) -> Option<&'a str> {
    input.get(key).and_then(|v| v.as_str()).map(str::trim).filter(|v| !v.is_empty())
}

fn required<'a>(input: &'a Value, key: &str) -> Result<&'a str, String> {
    str_arg(input, key).ok_or_else(|| format!("missing {key}"))
}

fn max_arg(input: &Value, default: u32) -> u32 {
    input.get("max_results").and_then(|v| v.as_u64()).map(|n| n.clamp(1, 25) as u32).unwrap_or(default)
}

fn clip(text: &str, max: usize) -> String {
    let flat = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if flat.chars().count() <= max {
        flat
    } else {
        format!("{}…", flat.chars().take(max - 1).collect::<String>())
    }
}

fn connect_card(reason: &str, service: &str, text: String) -> ToolOutcome {
    ToolOutcome { display_text: text, block: Some(ContentBlock::WorkspaceConnect { reason: reason.to_string(), services: vec![service.to_string()] }) }
}

#[async_trait]
impl SubAgent for WorkspaceAgent {
    fn agent_type(&self) -> Cow<'static, str> {
        Cow::Borrowed(WORKSPACE_AGENT_TYPE)
    }

    fn system_prompt(&self) -> Cow<'static, str> {
        Cow::Borrowed(SYSTEM_PROMPT)
    }

    fn intent_label(&self) -> Cow<'static, str> {
        Cow::Borrowed("workspace")
    }

    fn intent_description(&self) -> Cow<'static, str> {
        Cow::Borrowed("anything in the user's Google account: reading, searching, drafting or sending Gmail; reading or editing Google Sheets or Docs; finding Drive files; checking or adding Google Calendar events")
    }

    fn uses_memory(&self) -> bool {
        true
    }

    fn uses_personality(&self) -> bool {
        true
    }

    fn is_delegation_target(&self) -> bool {
        true
    }

    fn supports_todos(&self) -> bool {
        true
    }

    fn wants_current_time(&self) -> bool {
        true
    }

    /// Sending email and inviting people to events reach other people: those wait for the
    /// user's OK. Reading, drafting and editing their own files don't.
    fn tool_needs_approval(&self, tool_name: &str) -> bool {
        matches!(tool_name, "gmail_send" | "calendar_add_event")
    }

    fn describe_action(&self, tool_name: &str, input: &Value, locale: nomi_agent_core::Locale) -> Option<String> {
        match tool_name {
            "gmail_send" => {
                let someone = locale.t("workspace.someone");
                let to = str_arg(input, "to").unwrap_or(&someone);
                let subject = str_arg(input, "subject").map(|s| format!(" “{}”", clip(s, 60))).unwrap_or_default();
                let body = str_arg(input, "body").map(|b| format!(": {}", clip(b, 160))).unwrap_or_default();
                Some(locale.tf("workspace.approve_email", &[("to", to), ("subject", &subject), ("body", &body)]))
            }
            "calendar_add_event" => {
                let an_event = locale.t("workspace.an_event");
                let summary = str_arg(input, "summary").unwrap_or(&an_event);
                let start = str_arg(input, "start").unwrap_or("");
                let guests = input.get("attendees").and_then(|a| a.as_array()).map(|a| a.len()).unwrap_or(0);
                let invite = match guests {
                    0 => String::new(),
                    1 => locale.t("workspace.invite_one"),
                    n => locale.tf("workspace.invite_many", &[("count", &n.to_string())]),
                };
                Some(locale.tf("workspace.approve_event", &[("title", &clip(summary, 60)), ("start", start), ("invite", &invite)]))
            }
            _ => None,
        }
    }

    fn tools(&self) -> Vec<ToolDefinition> {
        vec![
            tool("gmail_search", "Search the user's Gmail with Gmail search syntax (from:, subject:, newer_than:7d, has:attachment, ...). Returns id, from, subject, date and a snippet for each.",
                json!({"type": "object", "properties": {"query": {"type": "string"}, "max_results": {"type": "integer", "description": "1-25, default 10"}}, "required": ["query"]})),
            tool("gmail_read", "Read one Gmail message in full (headers and plain-text body).",
                json!({"type": "object", "properties": {"message_id": {"type": "string"}}, "required": ["message_id"]})),
            tool("gmail_draft", "Save an email as a Gmail draft (not sent). The default for writing email.", email_schema()),
            tool("gmail_send", "Send an email from the user's Gmail. Only when they clearly asked to send; they approve it in chat first.", email_schema()),
            tool("sheets_read", "Read a Google Sheet. Without a range: its title and tab names. With a range in A1 notation (e.g. 'Stays!A1:F50'): the cell values.",
                json!({"type": "object", "properties": {"spreadsheet_id": {"type": "string"}, "range": {"type": "string"}}, "required": ["spreadsheet_id"]})),
            tool("sheets_append", "Add rows after the last row of a table in a Google Sheet (e.g. range 'Stays!A:F').",
                json!({"type": "object", "properties": {"spreadsheet_id": {"type": "string"}, "range": {"type": "string"}, "rows": rows_schema()}, "required": ["spreadsheet_id", "range", "rows"]})),
            tool("sheets_update", "Overwrite cells in a Google Sheet range with the given rows.",
                json!({"type": "object", "properties": {"spreadsheet_id": {"type": "string"}, "range": {"type": "string"}, "rows": rows_schema()}, "required": ["spreadsheet_id", "range", "rows"]})),
            tool("sheets_create", "Create a new Google Sheet, optionally with a header row.",
                json!({"type": "object", "properties": {"title": {"type": "string"}, "header": {"type": "array", "items": {"type": "string"}}}, "required": ["title"]})),
            tool("docs_read", "Read a Google Doc's text.",
                json!({"type": "object", "properties": {"document_id": {"type": "string"}}, "required": ["document_id"]})),
            tool("docs_create", "Create a new Google Doc, optionally with text.",
                json!({"type": "object", "properties": {"title": {"type": "string"}, "text": {"type": "string"}}, "required": ["title"]})),
            tool("docs_append", "Add text to the end of a Google Doc.",
                json!({"type": "object", "properties": {"document_id": {"type": "string"}, "text": {"type": "string"}}, "required": ["document_id", "text"]})),
            tool("drive_search", "Find files in the user's Google Drive by name. Returns id, name, type and link. Use it to get the id of a Sheet or Doc.",
                json!({"type": "object", "properties": {"query": {"type": "string", "description": "Part of the file name."}, "max_results": {"type": "integer"}}, "required": ["query"]})),
            tool("calendar_events", "List events on the user's primary Google Calendar between two times (RFC 3339).",
                json!({"type": "object", "properties": {"time_min": {"type": "string"}, "time_max": {"type": "string"}, "max_results": {"type": "integer"}}, "required": ["time_min"]})),
            tool("calendar_add_event", "Add an event to the user's primary Google Calendar. Times are RFC 3339 with the user's offset. Attendees get an invitation. The user approves it first.",
                json!({"type": "object", "properties": {
                    "summary": {"type": "string"}, "start": {"type": "string"}, "end": {"type": "string"},
                    "description": {"type": "string"}, "location": {"type": "string"},
                    "attendees": {"type": "array", "items": {"type": "string"}, "description": "Email addresses to invite."}
                }, "required": ["summary", "start", "end"]})),
        ]
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
        let service = tool_service(name).ok_or_else(|| format!("unknown tool {name}"))?;
        let Some(config) = self.config.as_deref() else {
            return Ok(ToolOutcome::text(
                "Google Workspace isn't set up on this Nomi server yet. Tell the user whoever runs Nomi needs to add Google OAuth credentials (GOOGLE_CLIENT_ID, GOOGLE_CLIENT_SECRET, GOOGLE_REDIRECT_URI).",
            ));
        };

        let token = match connection::access_token(conn, &self.http, config, &self.key, user_id, service).await {
            Ok(token) => token,
            Err(AccessError::NotConnected) => {
                return Ok(connect_card("not_connected", service, "The user hasn't connected their Google account. A Connect card is shown to them.".to_string()));
            }
            Err(AccessError::ServiceNotAllowed(service)) => {
                return Ok(connect_card(
                    "service_not_allowed",
                    &service,
                    format!("The user hasn't allowed {service} for Workspace. A card to turn it on is shown to them."),
                ));
            }
            Err(AccessError::Other(e)) => return Err(format!("couldn't load the Google connection: {e}")),
        };

        let google = Google { http: &self.http, config, token: &token };
        let result = run_tool(&google, conn, user_id, name, &input).await;
        match result {
            Ok(value) => Ok(ToolOutcome::text(value.to_string())),
            Err(GoogleError::Unauthorized) => {
                let _ = sqlx::query("DELETE FROM workspace_connections WHERE user_id = $1").bind(user_id).execute(&mut **conn).await;
                Ok(connect_card("not_connected", service, "Google rejected the connection (access was revoked). A card to reconnect is shown to the user.".to_string()))
            }
            Err(GoogleError::Failed(message)) => Err(message),
        }
    }
}

async fn run_tool(google: &Google<'_>, conn: &mut PoolConnection<Postgres>, user_id: Uuid, name: &str, input: &Value) -> Result<Value, GoogleError> {
    // The Connections page lists this activity back to the person, in their language.
    let locale = nomi_agent_core::user_locale(conn, user_id).await;
    let arg = |key: &str| required(input, key).map_err(GoogleError::Failed);
    let rows = || input.get("rows").filter(|r| r.is_array()).cloned().ok_or_else(|| GoogleError::Failed("rows must be a list of rows".to_string()));
    match name {
        "gmail_search" => google.gmail_search(arg("query")?, max_arg(input, 10)).await,
        "gmail_read" => google.gmail_read(arg("message_id")?).await,
        "gmail_draft" | "gmail_send" => {
            let (to, body) = (arg("to")?, arg("body")?);
            let subject = str_arg(input, "subject").unwrap_or_default();
            let reply_to = str_arg(input, "reply_to_message_id");
            let sending = name == "gmail_send";
            let result = if sending { google.gmail_send(to, subject, body, reply_to).await? } else { google.gmail_draft(to, subject, body, reply_to).await? };
            let about = if subject.is_empty() { String::new() } else { format!(": {}", clip(subject, 60)) };
            let key = if sending { "workspace.sent_email" } else { "workspace.drafted_email" };
            let summary = locale.tf(key, &[("to", &clip(to, 60)), ("about", &about)]);
            connection::record_activity(conn, user_id, "gmail", &summary, None).await;
            Ok(result)
        }
        "sheets_read" => google.sheets_read(arg("spreadsheet_id")?, str_arg(input, "range")).await,
        "sheets_append" | "sheets_update" => {
            let (id, range, rows) = (arg("spreadsheet_id")?, arg("range")?, rows()?);
            let count = rows.as_array().map(|r| r.len()).unwrap_or(0);
            let result = if name == "sheets_append" { google.sheets_append(id, range, &rows).await? } else { google.sheets_update(id, range, &rows).await? };
            let range = clip(range, 40);
            let summary = match (name, count) {
                ("sheets_append", 1) => locale.tf("workspace.added_rows_one", &[("range", &range)]),
                ("sheets_append", n) => locale.tf("workspace.added_rows_many", &[("count", &n.to_string()), ("range", &range)]),
                _ => locale.tf("workspace.updated_range", &[("range", &range)]),
            };
            connection::record_activity(conn, user_id, "sheets", &summary, Some(&google::sheet_link(id))).await;
            Ok(result)
        }
        "sheets_create" => {
            let title = arg("title")?;
            let result = google.sheets_create(title).await?;
            let id = result.get("spreadsheet_id").and_then(|v| v.as_str()).unwrap_or_default().to_string();
            if let Some(header) = input.get("header").and_then(|h| h.as_array()).filter(|h| !h.is_empty()) {
                google.sheets_update(&id, "A1", &json!([header])).await?;
            }
            connection::record_activity(conn, user_id, "sheets", &locale.tf("workspace.created_sheet", &[("title", &clip(title, 60))]), Some(&google::sheet_link(&id))).await;
            Ok(result)
        }
        "docs_read" => google.docs_read(arg("document_id")?).await,
        "docs_create" => {
            let title = arg("title")?;
            let result = google.docs_create(title, str_arg(input, "text")).await?;
            let id = result.get("document_id").and_then(|v| v.as_str()).unwrap_or_default().to_string();
            connection::record_activity(conn, user_id, "docs", &locale.tf("workspace.wrote_doc", &[("title", &clip(title, 60))]), Some(&google::doc_link(&id))).await;
            Ok(result)
        }
        "docs_append" => {
            let id = arg("document_id")?;
            let result = google.docs_append(id, arg("text")?).await?;
            connection::record_activity(conn, user_id, "docs", &locale.t("workspace.appended_doc"), Some(&google::doc_link(id))).await;
            Ok(result)
        }
        "drive_search" => google.drive_search(arg("query")?, max_arg(input, 10)).await,
        "calendar_events" => google.calendar_events(arg("time_min")?, str_arg(input, "time_max"), max_arg(input, 15)).await,
        "calendar_add_event" => {
            let summary = arg("summary")?;
            let mut event = json!({ "summary": summary, "start": { "dateTime": arg("start")? }, "end": { "dateTime": arg("end")? } });
            for key in ["description", "location"] {
                if let Some(value) = str_arg(input, key) {
                    event[key] = json!(value);
                }
            }
            if let Some(attendees) = input.get("attendees").and_then(|a| a.as_array()) {
                event["attendees"] = json!(attendees.iter().filter_map(|a| a.as_str()).map(|email| json!({ "email": email })).collect::<Vec<_>>());
            }
            let result = google.calendar_add_event(&event).await?;
            let link = result.get("url").and_then(|v| v.as_str()).map(str::to_string);
            connection::record_activity(conn, user_id, "calendar", &locale.tf("workspace.added_event", &[("title", &clip(summary, 60))]), link.as_deref()).await;
            Ok(result)
        }
        other => Err(GoogleError::Failed(format!("unknown tool {other}"))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_sending_and_inviting_wait_for_the_user() {
        let agent = WorkspaceAgent::new(reqwest::Client::new(), None, [0; 32]);
        for tool in agent.tools() {
            let gated = agent.tool_needs_approval(&tool.name);
            assert_eq!(gated, tool.name == "gmail_send" || tool.name == "calendar_add_event", "{}", tool.name);
            assert!(tool_service(&tool.name).is_some(), "{} has no service", tool.name);
        }
    }

    #[test]
    fn a_send_is_described_with_its_recipient_subject_and_opening() {
        let agent = WorkspaceAgent::new(reqwest::Client::new(), None, [0; 32]);
        let input = json!({"to": "stay@ubudvillas.example", "subject": "Booking", "body": "Hi, confirming 3 nights."});
        let text = agent.describe_action("gmail_send", &input, nomi_agent_core::Locale::En).unwrap();
        assert_eq!(text, "Send an email to stay@ubudvillas.example “Booking”: Hi, confirming 3 nights.");
        let text = agent.describe_action("gmail_send", &input, nomi_agent_core::Locale::Id).unwrap();
        assert_eq!(text, "Kirim email ke stay@ubudvillas.example “Booking”: Hi, confirming 3 nights.");
    }
}
