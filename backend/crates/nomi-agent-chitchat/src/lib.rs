use async_trait::async_trait;
use serde_json::Value;
use sqlx::pool::PoolConnection;
use sqlx::Postgres;
use uuid::Uuid;

use nomi_agent_core::SubAgent;
use nomi_llm::ToolDefinition;

pub const CHITCHAT_AGENT_TYPE: &str = "chitchat";

const CHITCHAT_SYSTEM_PROMPT: &str =
    "You are a helpful, friendly assistant chatting with the user. Keep replies concise.";

pub struct ChitchatAgent;

#[async_trait]
impl SubAgent for ChitchatAgent {
    fn agent_type(&self) -> &'static str {
        CHITCHAT_AGENT_TYPE
    }

    fn system_prompt(&self) -> &'static str {
        CHITCHAT_SYSTEM_PROMPT
    }

    fn tools(&self) -> Vec<ToolDefinition> {
        vec![]
    }

    async fn execute_tool(
        &self,
        _conn: &mut PoolConnection<Postgres>,
        _user_id: Uuid,
        _name: &str,
        _input: Value,
    ) -> Result<String, String> {
        // Unreachable: tools() returns an empty list, so the engine never calls this for
        // chitchat (the only tool it could ever see is complete_task, which the engine
        // handles itself before reaching an agent's execute_tool).
        Err("chitchat has no tools".to_string())
    }

    fn intent_label(&self) -> &'static str {
        CHITCHAT_AGENT_TYPE
    }

    fn intent_description(&self) -> &'static str {
        "General conversation, questions, or anything not covered by another agent"
    }

    fn is_default(&self) -> bool {
        true
    }

    fn uses_memory(&self) -> bool {
        true
    }
}
