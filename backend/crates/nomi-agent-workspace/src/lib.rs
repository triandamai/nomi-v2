//! The workspace agent. Its tools, and the client they run through, live in `nomi-tools-workspace`,
//! re-exported here.

use std::borrow::Cow;
use std::ops::Deref;

use async_trait::async_trait;
use serde_json::Value;
use sqlx::pool::PoolConnection;
use sqlx::Postgres;
use uuid::Uuid;

use nomi_agent_core::content_block::ToolOutcome;
use nomi_agent_core::SubAgent;
use nomi_llm::ToolDefinition;

pub use nomi_tools_workspace::*;

const SYSTEM_PROMPT: &str = "You are Tara, the crew member who works in the user's own Google \
account: Gmail, Sheets, Docs, Drive and Calendar. You only ever see the account of the person \
you are helping.

How you work:
- Find before you act: search Gmail or Drive for what the user means, read it, then do the job.
- Make sure before you write: when something you can't find or sensibly assume is missing (who \
an email goes to, an event's day or time, which of several files they mean), ask with ask_user, \
everything missing in one card.
- Sheets: read the tab names first (sheets_read without a range) if you don't know the layout; \
append rows under the existing headers rather than overwriting.
- Email: draft by default. Use gmail_send only when the user clearly asked to send; they approve \
every send in chat. Keep emails short, in the user's voice and language.
- Never invent message, file or spreadsheet ids: take them from search results.
- After acting, say plainly what you changed and where (with the link when you have one). For \
lists of emails or files, use `show` (a list or a table); for events, a timeline.
- If a tool says Google isn't connected or a service isn't allowed, a Connect card is already \
shown to the user: tell them in one short sentence and stop.";

/// The crew member, running every tool through its `WorkspaceClient`.
#[derive(Clone)]
pub struct WorkspaceAgent {
    client: WorkspaceClient,
}

impl WorkspaceAgent {
    pub fn new(http: reqwest::Client, config: Option<std::sync::Arc<connection::GoogleConfig>>, key: [u8; 32]) -> Self {
        Self::from_client(WorkspaceClient::new(http, config, key))
    }

    pub fn from_client(client: WorkspaceClient) -> Self {
        Self { client }
    }

    pub fn from_env() -> Self {
        Self::from_client(WorkspaceClient::from_env())
    }

    pub fn client(&self) -> &WorkspaceClient {
        &self.client
    }
}

impl Deref for WorkspaceAgent {
    type Target = WorkspaceClient;
    fn deref(&self) -> &WorkspaceClient {
        &self.client
    }
}

/// Every workspace tool, run through `agent`'s client, for the catalog.
pub fn catalog_tools(agent: &WorkspaceAgent) -> Vec<Box<dyn nomi_agent_core::Tool>> {
    nomi_tools_workspace::tools(&agent.client)
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
        Cow::Borrowed("only when the user's own Google account is involved: reading, searching, drafting or sending Gmail; reading or editing their Google Sheets or Docs; finding their Drive files; checking or adding Google Calendar events. Never for building an app or website")
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
        self.run(conn, user_id, name, input).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

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
