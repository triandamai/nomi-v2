//! The Files agent: every message carrying attachments (text files, voice-note transcripts) comes
//! here first (see nomi-turn's routing). It reads them and decides what Nomi should do: answer
//! from them directly, or hand the right parts to the specialist that owns that kind of work.

use std::borrow::Cow;

use async_trait::async_trait;
use serde_json::Value;
use sqlx::pool::PoolConnection;
use sqlx::Postgres;
use uuid::Uuid;

use nomi_agent_core::attachments::FILES_AGENT_TYPE;
use nomi_agent_core::SubAgent;
use nomi_llm::ToolDefinition;

const SYSTEM_PROMPT: &str = "You are Nomi's Files agent. The user's message carries attachments, each in an \
     <attachment name=\"…\" kind=\"…\"> section: kind=\"file\" is a text file (notes, CSV, JSON, code), \
     kind=\"voice\" is the transcript of a voice note they recorded. Read them and decide what to do:\n\
     - If they asked a question about the files, or just want them read, summarized, compared or \
     explained, answer yourself, clearly and briefly. Use show_table for tabular data.\n\
     - If the files hold work for a specialist, delegate it with delegate_to_agent, putting the \
     relevant content into the task, because the specialist can't see the attachment: \
     expenses, receipts or statements → money (to log them or check a budget); dates, deadlines \
     or things to remember to do → reminders; plans, itineraries or project briefs → planning; \
     code to add to or change in a project → coding.\n\
     - A voice note is the user talking to Nomi: treat what they said as their request.\n\
     - Several kinds of work in one go: delegate each part to its specialist, and use update_todos \
     to show the user the steps.\n\
     - If it's unclear what they want, say what's in the files in a sentence or two and ask.\n\
     Tell the user plainly what you did and what you handed to whom. Then call complete_task.";

pub struct FilesAgent;

#[async_trait]
impl SubAgent for FilesAgent {
    fn agent_type(&self) -> Cow<'static, str> {
        Cow::Borrowed(FILES_AGENT_TYPE)
    }

    fn display_name(&self) -> Cow<'static, str> {
        Cow::Borrowed("Files")
    }

    fn system_prompt(&self) -> Cow<'static, str> {
        Cow::Borrowed(SYSTEM_PROMPT)
    }

    fn tools(&self) -> Vec<ToolDefinition> {
        Vec::new()
    }

    async fn execute_tool(
        &self,
        _conn: &mut PoolConnection<Postgres>,
        _session_id: Uuid,
        _agent_session_id: Uuid,
        _user_id: Uuid,
        name: &str,
        _input: Value,
    ) -> Result<nomi_agent_core::ToolOutcome, String> {
        Err(format!("unknown tool: {name}"))
    }

    fn intent_label(&self) -> Cow<'static, str> {
        Cow::Borrowed(FILES_AGENT_TYPE)
    }

    fn intent_description(&self) -> Cow<'static, str> {
        Cow::Borrowed("the user shared files or a voice note and wants Nomi to deal with them")
    }

    fn can_delegate(&self) -> bool {
        true
    }

    // Files arrive with the user's own message; nobody hands work to the Files agent.
    fn is_delegation_target(&self) -> bool {
        false
    }

    fn supports_todos(&self) -> bool {
        true
    }

    fn uses_personality(&self) -> bool {
        true
    }
}
