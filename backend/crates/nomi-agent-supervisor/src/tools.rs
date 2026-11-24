//! The supervisor's tools as `Tool`s.

use nomi_agent_core::ToolContext;
use async_trait::async_trait;
use serde_json::Value;
use sqlx::pool::PoolConnection;
use sqlx::Postgres;

use nomi_agent_core::{Tool, ToolOutcome};
use nomi_llm::ToolDefinition;

use crate::SUPERVISOR_AGENT_TYPE;

/// The supervisor's tools that run for any agent. `stop_agents` is left out: it spares the
/// supervisor's own turn from the stop it makes, and from any other agent's turn it would stop
/// that very turn.
pub fn catalog_tools() -> Vec<Box<dyn Tool>> {
    vec![Box::new(ListRecentAgentActivity)]
}

/// Every tool the supervisor is offered, in order.
pub(crate) fn agent_tools() -> Vec<Box<dyn Tool>> {
    vec![Box::new(ListRecentAgentActivity), Box::new(StopAgents)]
}

pub struct ListRecentAgentActivity;

#[async_trait]
impl Tool for ListRecentAgentActivity {
    fn definition(&self) -> ToolDefinition {
        crate::list_recent_agent_activity_tool_definition()
    }
    fn group(&self) -> &'static str {
        SUPERVISOR_AGENT_TYPE
    }
    fn keywords(&self) -> &'static [&'static str] {
        &["status", "delegations", "background", "progress"]
    }
    async fn execute(&self, conn: &mut PoolConnection<Postgres>, ctx: &ToolContext<'_>, _input: Value) -> Result<ToolOutcome, String> {
        let (session_id, _agent_session_id, _user_id) = (ctx.session_id, ctx.agent_session_id, ctx.user_id);
        crate::list_recent_agent_activity(conn, session_id).await.map(ToolOutcome::text)
    }
}

pub struct StopAgents;

#[async_trait]
impl Tool for StopAgents {
    fn definition(&self) -> ToolDefinition {
        crate::stop_agents_tool_definition()
    }
    fn group(&self) -> &'static str {
        SUPERVISOR_AGENT_TYPE
    }
    fn keywords(&self) -> &'static [&'static str] {
        &["cancel", "abort", "halt"]
    }
    async fn execute(&self, conn: &mut PoolConnection<Postgres>, ctx: &ToolContext<'_>, input: Value) -> Result<ToolOutcome, String> {
        let (session_id, _agent_session_id, user_id) = (ctx.session_id, ctx.agent_session_id, ctx.user_id);
        crate::stop_agents_tool(conn, session_id, user_id, &input).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supervisor_tools_metadata() {
        let catalog = catalog_tools();
        assert_eq!(catalog.len(), 1);
        assert_eq!(catalog[0].definition().name, "list_recent_agent_activity");
        assert_eq!(catalog[0].group(), SUPERVISOR_AGENT_TYPE);
        assert_eq!(catalog[0].keywords(), &["status", "delegations", "background", "progress"]);

        let all = agent_tools();
        assert_eq!(all.len(), 2);
        assert_eq!(all[1].definition().name, "stop_agents");
        assert_eq!(all[1].group(), SUPERVISOR_AGENT_TYPE);
        assert_eq!(all[1].keywords(), &["cancel", "abort", "halt"]);
    }
}
