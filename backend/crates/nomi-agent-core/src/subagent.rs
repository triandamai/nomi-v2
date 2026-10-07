use std::borrow::Cow;

use async_trait::async_trait;
use serde_json::Value;
use sqlx::pool::PoolConnection;
use sqlx::Postgres;
use uuid::Uuid;

use nomi_llm::ToolDefinition;

use crate::content_block::ToolOutcome;

#[async_trait]
pub trait SubAgent: Send + Sync {
    fn agent_type(&self) -> Cow<'static, str>;
    fn system_prompt(&self) -> Cow<'static, str>;
    fn tools(&self) -> Vec<ToolDefinition>;
    async fn execute_tool(
        &self,
        conn: &mut PoolConnection<Postgres>,
        session_id: Uuid,
        agent_session_id: Uuid,
        user_id: Uuid,
        name: &str,
        input: Value,
    ) -> Result<ToolOutcome, String>;

    /// Fed verbatim into the intent classifier's prompt. Never called for the agent that
    /// returns `true` from `is_default()` — that agent is the fallback, not something the
    /// classifier picks between.
    fn intent_label(&self) -> Cow<'static, str>;
    fn intent_description(&self) -> Cow<'static, str>;

    /// Human-readable label shown on this agent's chat bubbles (e.g. "Money" for the money
    /// agent, stored on `messages.agent_display_name` at insert time). Defaults to Title Case
    /// of `agent_type()` — correct for every built-in specialist agent (money → "Money",
    /// planning → "Planning", etc.). Overridden by `ChitchatAgent` (the raw type "chitchat"
    /// would read oddly as a label; "Nomi" is the actual product name) and by `DynamicAgent`
    /// (which uses its own admin-configured name instead of its type, a stringified UUID).
    fn display_name(&self) -> Cow<'static, str> {
        let type_name = self.agent_type();
        let mut chars = type_name.chars();
        match chars.next() {
            Some(first) => Cow::Owned(first.to_uppercase().collect::<String>() + chars.as_str()),
            None => Cow::Borrowed(""),
        }
    }

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
    /// a request to another registered specialist (see `works_in_background`).
    fn can_delegate(&self) -> bool {
        false
    }

    /// When true, a delegation to this agent runs in the background (the delegation worker)
    /// and it reports back when done: for long jobs like building a project. Otherwise the
    /// agent takes the conversation over in the same turn and answers the user itself.
    fn works_in_background(&self) -> bool {
        false
    }

    /// When true, a reply that reads as a plan is always saved as a plan draft (`write_plan`)
    /// rather than posted as plain markdown, even if the model forgot to call the tool.
    fn keeps_plans_in_drafts(&self) -> bool {
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

    /// Whether Admin → Tools switches apply to this agent's own `tools()`. A built-in agent's
    /// tools are part of how it works and stay on; a custom agent's tools are granted from the
    /// catalog, where an admin can switch them off.
    fn own_tools_switchable(&self) -> bool {
        false
    }

    /// When true, run_agent_turn gives this agent the engine-level `update_todos` tool for
    /// maintaining a live multi-step task checklist. Off by default — most agents don't run
    /// long enough multi-step builds to need one.
    fn supports_todos(&self) -> bool {
        false
    }

    /// When true, run_agent_turn gives this agent the engine-level `write_plan` tool for writing
    /// a durable, versioned plan before or during a multi-step task — see
    /// docs/superpowers/specs/2026-09-17-agent-plan-artifacts-design.md. Off by default, same
    /// reasoning as supports_todos(): most agents don't do work substantial enough to plan first.
    fn supports_plans(&self) -> bool {
        false
    }

    /// When true, run_agent_turn gives this agent the engine-level `create_reminder`,
    /// `list_reminders`, and `cancel_reminder` tools for scheduling deferred/recurring tasks —
    /// see docs/superpowers/specs/2026-09-20-scheduled-reminders-design.md. Off by default, same
    /// reasoning as supports_plans(): only the conversational front door needs this.
    fn supports_reminders(&self) -> bool {
        false
    }

    /// Gives the agent private record storage (save/list/update/delete_record; see
    /// `crate::records`): for agents without tables of their own. Every dynamic agent has it.
    fn uses_records(&self) -> bool {
        false
    }

    /// Adds the current date and time (in the user's timezone) to the system prompt. Agents
    /// that schedule things want it; `supports_reminders` agents always get it.
    fn wants_current_time(&self) -> bool {
        false
    }

    /// Whether a call to one of this agent's own tools needs the user's OK (subject to their
    /// permission rules). Agents whose tools only touch the user's own records in the agent's
    /// own tables can say no for those.
    fn tool_needs_approval(&self, _tool_name: &str) -> bool {
        true
    }

    /// What the approval card says this call will do ("Send an email to …"), in the person's
    /// language. `None` falls back to the engine's generic wording.
    fn describe_action(&self, _tool_name: &str, _input: &Value, _locale: crate::Locale) -> Option<String> {
        None
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
