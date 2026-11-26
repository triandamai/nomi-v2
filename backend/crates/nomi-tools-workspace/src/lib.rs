//! The Workspace agent: works in the user's own Google account (Gmail, Sheets, Docs, Drive,
//! Calendar). Each person connects their own account from Connections; every tool call loads the
//! token of the user the turn belongs to, so a hand-off only ever reaches the asker's account.
//!
//! Storage (docs/agents/storage.md): `workspace_connections`, `workspace_oauth_states`,
//! `workspace_activity`.

pub mod connection;
pub mod google;

use nomi_tool::ToolContext;
use std::sync::{Arc, OnceLock};

use async_trait::async_trait;
use serde_json::{json, Value};
use sqlx::pool::PoolConnection;
use sqlx::Postgres;
use uuid::Uuid;

use nomi_tool::content_block::{ContentBlock, ToolOutcome};
use nomi_llm::ToolDefinition;

use connection::{AccessError, GoogleConfig};
use google::{Google, GoogleError};

pub const WORKSPACE_AGENT_TYPE: &str = "workspace";

/// One shared HTTP client for every registry this process builds.
fn http_client() -> reqwest::Client {
    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    CLIENT.get_or_init(reqwest::Client::new).clone()
}

/// Where the agent's Google OAuth client comes from.
#[derive(Clone)]
enum GoogleSource {
    /// Given once (tests); `None` means Google isn't set up.
    Fixed(Option<Arc<GoogleConfig>>),
    /// Admin → Configuration as it is at each call, so a saved client applies at once.
    Current,
}

#[derive(Clone)]
pub struct WorkspaceClient {
    http: reqwest::Client,
    config: GoogleSource,
    key: [u8; 32],
}

impl WorkspaceClient {
    pub fn new(http: reqwest::Client, config: Option<Arc<GoogleConfig>>, key: [u8; 32]) -> Self {
        Self { http, config: GoogleSource::Fixed(config), key }
    }

    /// The encryption key from the environment; Google's client from Admin → Configuration on
    /// every call (see `GoogleConfig::current`).
    pub fn from_env() -> Self {
        let key = std::env::var("SETTINGS_ENCRYPTION_KEY").ok().and_then(|k| nomi_settings::crypto::parse_key(&k)).unwrap_or([0u8; 32]);
        Self { http: http_client(), config: GoogleSource::Current, key }
    }

    async fn google_config(&self) -> Option<Arc<GoogleConfig>> {
        match &self.config {
            GoogleSource::Fixed(config) => config.clone(),
            GoogleSource::Current => GoogleConfig::current().await.map(Arc::new),
        }
    }
}

/// Which service each tool needs.
pub fn tool_service(name: &str) -> Option<&'static str> {
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

pub fn str_arg<'a>(input: &'a Value, key: &str) -> Option<&'a str> {
    input.get(key).and_then(|v| v.as_str()).map(str::trim).filter(|v| !v.is_empty())
}

fn required<'a>(input: &'a Value, key: &str) -> Result<&'a str, String> {
    str_arg(input, key).ok_or_else(|| format!("missing {key}"))
}

fn max_arg(input: &Value, default: u32) -> u32 {
    input.get("max_results").and_then(|v| v.as_u64()).map(|n| n.clamp(1, 25) as u32).unwrap_or(default)
}

pub fn clip(text: &str, max: usize) -> String {
    let flat = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if flat.chars().count() <= max {
        flat
    } else {
        format!("{}…", flat.chars().take(max - 1).collect::<String>())
    }
}

fn connect_card(reason: &str, service: &str, text: String) -> ToolOutcome {
    ToolOutcome {
        display_text: text,
        block: Some(ContentBlock::WorkspaceConnect {
            reason: reason.to_string(),
            services: vec![service.to_string()],
            agent_type: Some(WORKSPACE_AGENT_TYPE.to_string()),
        }),
        media: Vec::new(),
    }
}

impl WorkspaceClient {
    /// Runs Workspace tool `name` in `user_id`'s own Google account.
    pub async fn run(&self, conn: &mut PoolConnection<Postgres>, user_id: Uuid, name: &str, input: Value) -> Result<ToolOutcome, String> {
        let service = tool_service(name).ok_or_else(|| format!("unknown tool {name}"))?;
        let google_config = self.google_config().await;
        let Some(config) = google_config.as_deref() else {
            return Ok(ToolOutcome::text(
                "Google Workspace isn't set up on this Nomi server yet. Tell the user whoever runs Nomi needs to add Google OAuth credentials in Admin → Configuration → Google.",
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
            Ok(value) => {
                let locale = nomi_tool::locale::user_locale(conn, user_id).await;
                let block = result_card(name, &input, &value, locale).map(ContentBlock::View);
                Ok(ToolOutcome { display_text: value.to_string(), block, media: Vec::new() })
            }
            Err(GoogleError::Unauthorized) => {
                connection::mark_expired(conn, user_id).await;
                Ok(connect_card("not_connected", service, "Google rejected the connection (access was revoked). A card to reconnect is shown to the user.".to_string()))
            }
            Err(GoogleError::Failed(message)) => Err(message),
        }
    }
}

/// Extra search words for each Workspace tool.
fn keywords_for(name: &str) -> &'static [&'static str] {
    match name {
        "gmail_search" => &["email", "inbox", "google", "find"],
        "gmail_read" => &["email", "inbox", "google", "open"],
        "gmail_draft" => &["email", "compose", "google", "write"],
        "gmail_send" => &["email", "compose", "google", "mail"],
        "sheets_read" => &["spreadsheet", "google", "table", "cells"],
        "sheets_append" => &["spreadsheet", "google", "rows", "add"],
        "sheets_update" => &["spreadsheet", "google", "cells", "edit"],
        "sheets_create" => &["spreadsheet", "google", "new"],
        "docs_read" => &["document", "google", "text"],
        "docs_create" => &["document", "google", "write", "new"],
        "docs_append" => &["document", "google", "write", "add"],
        "drive_search" => &["files", "google", "find"],
        "calendar_events" => &["schedule", "agenda", "google", "meetings"],
        "calendar_add_event" => &["schedule", "meeting", "invite", "google"],
        _ => &[],
    }
}

/// One Workspace tool, run through `agent`'s Google access.
pub struct WorkspaceTool {
    definition: ToolDefinition,
    agent: Arc<WorkspaceClient>,
}

#[async_trait]
impl nomi_tool::Tool for WorkspaceTool {
    fn definition(&self) -> ToolDefinition {
        self.definition.clone()
    }
    fn group(&self) -> &'static str {
        WORKSPACE_AGENT_TYPE
    }
    fn keywords(&self) -> &'static [&'static str] {
        keywords_for(&self.definition.name)
    }
    async fn execute(&self, conn: &mut PoolConnection<Postgres>, ctx: &ToolContext<'_>, input: Value) -> Result<ToolOutcome, String> {
        let (_session_id, _agent_session_id, user_id) = (ctx.session_id, ctx.agent_session_id, ctx.user_id);
        self.agent.run(conn, user_id, &self.definition.name, input).await
    }
}

/// Every Workspace tool, run through `agent`'s HTTP client, Google client and key. All of them
/// run for any agent: each works in the asking person's own Google account, and a custom agent
/// asks the person before every tool call.
pub fn tools(agent: &WorkspaceClient) -> Vec<Box<dyn nomi_tool::Tool>> {
    let agent = Arc::new(agent.clone());
    tool_definitions()
        .into_iter()
        .map(|definition| Box::new(WorkspaceTool { definition, agent: agent.clone() }) as Box<dyn nomi_tool::Tool>)
        .collect()
}

/// Every Workspace tool, in the order Tara is offered them.
pub fn tool_definitions() -> Vec<ToolDefinition> {
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

/// What a write shows in the chat: a card of the event, doc, sheet or email it made, with a link
/// to open it in Google. Reads show nothing of their own (the agent shows what's worth seeing).
fn result_card(name: &str, input: &Value, result: &Value, locale: nomi_tool::locale::Locale) -> Option<nomi_tool::views::View> {
    use nomi_tool::views::{CardView, Field, Format, LinkRef, Scalar, View};
    let text = |key: &str| str_arg(input, key).map(|s| s.to_string());
    let link = |key: &str, label: &str| {
        result.get(key).and_then(|v| v.as_str()).filter(|u| u.starts_with("https://")).map(|url| LinkRef { label: locale.t(label), url: url.to_string() })
    };
    let field = |label: &str, value: String, format: Format| Field { label: locale.t(label), value: Scalar::Text(value), format };
    let card = |eyebrow: &str, title: String| CardView {
        eyebrow: Some(locale.t(eyebrow)),
        title,
        subtitle: None,
        highlight: None,
        fields: Vec::new(),
        body: None,
        link: None,
        actions: Vec::new(),
        tone: Default::default(),
    };
    let view = match name {
        "calendar_add_event" => {
            let mut c = card("workspace.card_event", text("summary")?);
            c.fields.extend(text("start").map(|v| field("workspace.card_starts", v, Format::DateTime)));
            c.fields.extend(text("end").map(|v| field("workspace.card_ends", v, Format::DateTime)));
            c.fields.extend(text("location").map(|v| field("workspace.card_where", v, Format::Text)));
            let guests: Vec<&str> = input.get("attendees").and_then(|a| a.as_array()).into_iter().flatten().filter_map(|a| a.as_str()).collect();
            if !guests.is_empty() {
                c.fields.push(field("workspace.card_guests", guests.join(", "), Format::Text));
            }
            c.link = link("url", "workspace.card_open_calendar");
            c
        }
        "docs_create" => {
            let mut c = card("workspace.card_doc", text("title")?);
            let id = result.get("document_id").and_then(|v| v.as_str())?;
            c.link = Some(LinkRef { label: locale.t("workspace.card_open"), url: google::doc_link(id) });
            c
        }
        "sheets_create" => {
            let mut c = card("workspace.card_sheet", text("title")?);
            let id = result.get("spreadsheet_id").and_then(|v| v.as_str())?;
            c.link = Some(LinkRef { label: locale.t("workspace.card_open"), url: google::sheet_link(id) });
            c
        }
        "gmail_send" | "gmail_draft" => {
            let eyebrow = if name == "gmail_send" { "workspace.card_sent" } else { "workspace.card_drafted" };
            let mut c = card(eyebrow, text("subject").filter(|s| !s.trim().is_empty()).unwrap_or_else(|| locale.t("workspace.card_no_subject")));
            c.fields.push(field("workspace.card_to", text("to")?, Format::Text));
            c.body = text("body").map(|b| clip(&b, 280));
            c
        }
        _ => return None,
    };
    Some(View::Card(view))
}

async fn run_tool(google: &Google<'_>, conn: &mut PoolConnection<Postgres>, user_id: Uuid, name: &str, input: &Value) -> Result<Value, GoogleError> {
    // The Connections page lists this activity back to the person, in their language.
    let locale = nomi_tool::locale::user_locale(conn, user_id).await;
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
