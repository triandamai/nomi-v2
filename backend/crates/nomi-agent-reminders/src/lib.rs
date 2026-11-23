//! Kala, the reminders agent. Her tools and the reminders they keep live in
//! `nomi-tools-reminders`, re-exported here.

use std::borrow::Cow;

use async_trait::async_trait;
use serde_json::Value;
use sqlx::pool::PoolConnection;
use sqlx::Postgres;
use uuid::Uuid;

use nomi_agent_core::{SubAgent, ToolContext};
use nomi_llm::ToolDefinition;

pub use nomi_tools_reminders::*;

const SYSTEM_PROMPT: &str = "You are Kala, the member of Nomi's crew who keeps reminders. You keep the user's reminders: add new ones \
     (add_reminder), show what's coming up (show_reminders), change, tick off or remove one \
     (edit_reminder, complete_reminder, remove_reminder). Resolve times like \"tomorrow at 4\" \
     against the current date and time you're given, in the user's timezone, and pass due_at with \
     that timezone's offset. For a repeating reminder set recurrence to daily, weekly (on due_at's \
     weekday) or monthly (on due_at's day). Make sure before you add one: if they didn't say what \
     it's for, when (the day and the time), or whether it's one-time or repeating (and how often), \
     ask with ask_user, everything missing in one card (e.g. \"Just once, or repeating?\" with Just \
     once, Every day, Every week, Every month, and then the time), and add it once you know. Confirm \
     briefly what you set, with the day and time. \
     You only keep reminders; for work an agent should do later (\"every Monday, summarize my \
     spending\"), say that Nomi can schedule it. When you're done, call complete_task.";

/// Every reminders tool, for the catalog.
pub fn catalog_tools() -> Vec<Box<dyn nomi_agent_core::Tool>> {
    nomi_tools_reminders::tools()
}

pub struct RemindersAgent;

#[async_trait]
impl SubAgent for RemindersAgent {
    /// Remembers durable facts the user mentions here and recalls them in later turns.
    fn uses_memory(&self) -> bool {
        true
    }

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
        nomi_agent_core::tool_definitions(&nomi_tools_reminders::tools())
    }

    async fn execute_tool(
        &self,
        conn: &mut PoolConnection<Postgres>,
        session_id: Uuid,
        agent_session_id: Uuid,
        user_id: Uuid,
        name: &str,
        input: Value,
    ) -> Result<nomi_agent_core::ToolOutcome, String> {
        let tools = nomi_tools_reminders::tools();
        let tool = nomi_agent_core::find_tool(&tools, name).ok_or_else(|| format!("unknown tool: {name}"))?;
        tool.execute(conn, &ToolContext::new(session_id, agent_session_id, user_id, self.agent_type().as_ref(), self.display_name().as_ref()), input).await
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
