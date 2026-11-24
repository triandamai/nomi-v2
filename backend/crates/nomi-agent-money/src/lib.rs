//! Dana, the money agent. Her tools and their data live in `nomi-tools-money`, re-exported here.

use std::borrow::Cow;

use async_trait::async_trait;
use serde_json::Value;
use sqlx::pool::PoolConnection;
use sqlx::Postgres;
use uuid::Uuid;

use nomi_agent_core::prompts::MONEY_SYSTEM_PROMPT;
use nomi_agent_core::{SubAgent, ToolContext};
use nomi_llm::ToolDefinition;

pub use nomi_tools_money::*;

/// Every money tool, for the catalog.
pub fn catalog_tools() -> Vec<Box<dyn nomi_agent_core::Tool>> {
    nomi_tools_money::tools()
}

pub struct MoneyAgent;

#[async_trait]
impl SubAgent for MoneyAgent {
    /// Remembers durable facts the user mentions here and recalls them in later turns.
    fn uses_memory(&self) -> bool {
        true
    }

    fn agent_type(&self) -> Cow<'static, str> {
        Cow::Borrowed(MONEY_AGENT_TYPE)
    }

    fn system_prompt(&self) -> Cow<'static, str> {
        Cow::Borrowed(MONEY_SYSTEM_PROMPT)
    }

    fn tools(&self) -> Vec<ToolDefinition> {
        nomi_agent_core::tool_definitions(&nomi_tools_money::tools())
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
        let tools = nomi_tools_money::tools();
        let tool = nomi_agent_core::find_tool(&tools, name).ok_or_else(|| format!("unknown tool: {name}"))?;
        tool.execute(conn, &ToolContext::new(session_id, agent_session_id, user_id, self.agent_type().as_ref(), self.display_name().as_ref()), input).await
    }

    fn intent_label(&self) -> Cow<'static, str> {
        Cow::Borrowed("money")
    }

    fn intent_description(&self) -> Cow<'static, str> {
        Cow::Borrowed("the user wants to look at their transactions, spending, income or budget, log an expense or income, set a budget, or change when their money month starts")
    }
}
