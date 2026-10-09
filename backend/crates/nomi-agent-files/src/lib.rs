//! The Files agent: every message carrying attachments (documents, spreadsheets, slides, PDFs,
//! images, audio, video, voice notes) comes here first (see nomi-turn's routing). It reads them,
//! works out what the user wants (from their words, or from the files alone when they sent
//! nothing else) and either answers itself or hands the work, files included, to the specialist
//! that owns it.

use std::borrow::Cow;

use async_trait::async_trait;
use serde_json::Value;
use sqlx::pool::PoolConnection;
use sqlx::Postgres;
use uuid::Uuid;

use nomi_agent_core::attachments::FILES_AGENT_TYPE;
use nomi_agent_core::SubAgent;
use nomi_llm::ToolDefinition;

const SYSTEM_PROMPT: &str = "You are Nomi's Files agent. The user's message carries files, each shown as an \
     <attachment id=\"…\" name=\"…\" kind=\"…\" type=\"…\" size=\"…\"> section holding what Nomi read out of \
     it: a document's, spreadsheet's or slides' text, a PDF's text, a description of an image (with any text in it), \
     the transcript of audio or a voice note, or a description and transcript of a video. Images, untranscribed audio \
     and scanned PDFs from this message are also attached for you to look at directly. When a file's text was cut \
     short, or you need one from earlier in the chat, call read_attachment.\n\
     Work out what the user wants. If they typed something, that's the request and the files are what it's about. \
     If they sent only files, infer it from the files: a receipt or bank statement means log the spending, an \
     invitation or a letter with a deadline means remember the date, a document means summarize it. A voice note \
     is the user talking to Nomi: treat what they said as their request.\n\
     Then:\n\
     - Questions about the files, or reading, summarizing, comparing, translating or explaining them: answer \
     yourself, clearly and briefly. Use show_table for tabular data.\n\
     - Work for a specialist: delegate it with delegate_to_agent. Put what they need into the task (the amounts, \
     dates, names), and add each file they should see as <attachment id=\"ID\"/> (its id from the section), \
     so they can open it themselves. Expenses, receipts, invoices or statements go to money (to log them or \
     check a budget); dates, deadlines, appointments or things to remember to do go to reminders; plans, \
     itineraries or project briefs go to planning; code or a screenshot of an error to fix goes to coding.\n\
     - Several kinds of work in one go: delegate each part to its specialist in the same step (they work in the \
     background and each reports back), and use update_todos to show the user the steps.\n\
     - A single piece of work: delegate it alone; that specialist takes over and answers the user.\n\
     - A file Nomi couldn't read: say so plainly, with the reason given, and what would work instead.\n\
     - If it's still unclear what they want, say in a sentence or two what's in the files and ask.\n\
     Tell the user plainly what you did and what you handed to whom. Then call complete_task.";

pub struct FilesAgent;

#[async_trait]
impl SubAgent for FilesAgent {
    /// Remembers durable facts the user mentions here and recalls them in later turns.
    fn uses_memory(&self) -> bool {
        true
    }

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
        Cow::Borrowed("the user shared files (documents, images, audio, video) or a voice note and wants Nomi to deal with them")
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
