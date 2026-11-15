//! The supervisor's tools as `Tool`s.

use async_trait::async_trait;
use serde_json::Value;
use sqlx::pool::PoolConnection;
use sqlx::Postgres;
use uuid::Uuid;

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
    async fn execute(&self, conn: &mut PoolConnection<Postgres>, session_id: Uuid, _agent_session_id: Uuid, _user_id: Uuid, _input: Value) -> Result<ToolOutcome, String> {
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
    async fn execute(&self, conn: &mut PoolConnection<Postgres>, session_id: Uuid, _agent_session_id: Uuid, user_id: Uuid, input: Value) -> Result<ToolOutcome, String> {
        crate::stop_agents_tool(conn, session_id, user_id, &input).await
    }
}
