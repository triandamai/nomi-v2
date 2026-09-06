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
        session_id: Uuid,
        agent_session_id: Uuid,
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

    /// When `true`, `run_agent_turn` folds the user's currently stored personality (if any)
    /// into the system prompt for this turn. See `nomi_agent_core::personality`.
    fn uses_personality(&self) -> bool {
        false
    }

    /// When true, run_agent_turn gives this agent an extra `delegate_to_agent` tool that hands
    /// a task to another registered specialist to run in the background.
    fn can_delegate(&self) -> bool {
        false
    }

    /// When false, this agent is never offered as a delegation target — used by the supervisor
    /// agent, which delivers/reports on delegated work rather than being delegated to itself.
    fn is_delegation_target(&self) -> bool {
        true
    }

    /// When true, run_agent_turn posts a chat message for every tool call this agent makes
    /// (and for any text it writes alongside one, treated as its "thinking out loud") — so the
    /// user watching a long-running build sees each step as it happens, not just a final
    /// summary. Off by default: most agents' tool calls are internal bookkeeping the user
    /// never needs to see.
    fn surfaces_activity(&self) -> bool {
        false
    }

    /// Called on the delegation *target* before `delegate_to_agent` creates anything, with the
    /// exact task string the delegating agent wrote. Return `Err(reason)` to reject the
    /// delegation outright — no delegation row is created, and `reason` is fed straight back to
    /// the delegating agent as a tool error, giving it a chance to self-correct in the same turn
    /// (e.g. actually create a project before delegating to the coding agent) instead of a
    /// malformed delegation reaching a target agent that has no way to act on it. Default
    /// accepts anything: only targets with a real structural precondition on the task string
    /// need to override this.
    fn validate_delegation_task(&self, _task: &str) -> Result<(), String> {
        Ok(())
    }
}
