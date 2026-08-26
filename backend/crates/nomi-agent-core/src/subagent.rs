use async_trait::async_trait;
use serde_json::Value;
use sqlx::pool::PoolConnection;
use sqlx::Postgres;
use uuid::Uuid;

use nomi_llm::ToolDefinition;

#[async_trait]
pub trait SubAgent: Send + Sync {
    fn agent_type(&self) -> &'static str;
    fn system_prompt(&self) -> &'static str;
    fn tools(&self) -> Vec<ToolDefinition>;
    async fn execute_tool(
        &self,
        conn: &mut PoolConnection<Postgres>,
        user_id: Uuid,
        name: &str,
        input: Value,
    ) -> Result<String, String>;

    /// Fed verbatim into the intent classifier's prompt. Never called for the agent that
    /// returns `true` from `is_default()` — that agent is the fallback, not something the
    /// classifier picks between.
    fn intent_label(&self) -> &'static str;
    fn intent_description(&self) -> &'static str;

    /// Exactly one registered agent must return `true`. See `AgentRegistry::new`.
    fn is_default(&self) -> bool {
        false
    }

    /// When `true`, `run_agent_turn` retrieves relevant memories before the first LLM call
    /// (folded into the system prompt) and extracts+stores new memories after a `Reply`
    /// outcome (never after `Completed` — a completed task summary isn't a conversational
    /// reply worth remembering facts from).
    fn uses_memory(&self) -> bool {
        false
    }
}
