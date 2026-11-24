//! The personality agent. Its tools, and the personality store they write, live in
//! `nomi-tools-personality`, re-exported here.

use std::borrow::Cow;

use async_trait::async_trait;
use serde_json::Value;
use sqlx::pool::PoolConnection;
use sqlx::Postgres;
use uuid::Uuid;

use nomi_agent_core::prompts::PERSONALITY_SYSTEM_PROMPT;
use nomi_agent_core::{SubAgent, ToolContext};
use nomi_llm::ToolDefinition;

pub use nomi_tools_personality::*;

/// Every personality tool, for the catalog.
pub fn catalog_tools() -> Vec<Box<dyn nomi_agent_core::Tool>> {
    nomi_tools_personality::tools()
}

pub struct PersonalityAgent;

#[async_trait]
impl SubAgent for PersonalityAgent {
    fn agent_type(&self) -> Cow<'static, str> {
        Cow::Borrowed(PERSONALITY_AGENT_TYPE)
    }

    fn system_prompt(&self) -> Cow<'static, str> {
        Cow::Borrowed(PERSONALITY_SYSTEM_PROMPT)
    }

    fn tools(&self) -> Vec<ToolDefinition> {
        nomi_agent_core::tool_definitions(&nomi_tools_personality::tools())
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
        let tools = nomi_tools_personality::tools();
        let tool = nomi_agent_core::find_tool(&tools, name).ok_or_else(|| format!("unknown tool: {name}"))?;
        tool.execute(conn, &ToolContext::new(session_id, agent_session_id, user_id, self.agent_type().as_ref(), self.display_name().as_ref()), input).await
    }

    fn intent_label(&self) -> Cow<'static, str> {
        Cow::Borrowed(PERSONALITY_AGENT_TYPE)
    }

    fn intent_description(&self) -> Cow<'static, str> {
        Cow::Borrowed("The user wants to change how nomi talks or behaves — its tone, style, or personality")
    }

    fn uses_personality(&self) -> bool {
        true
    }
}
