//! Rena's tools as `Tool`s.

use async_trait::async_trait;
use serde_json::Value;
use sqlx::pool::PoolConnection;
use sqlx::Postgres;
use uuid::Uuid;

use nomi_agent_core::relay_ops::{NoRelayOps, SharedRelayOps};
use nomi_agent_core::{Tool, ToolOutcome};
use nomi_llm::ToolDefinition;

use crate::PLANNING_AGENT_TYPE;

/// Rena's tools that run for any agent. Custom agents create projects on Drizzle: only Rena gets
/// Relay backends made. `publish_project` is left out: it publishes under the calling agent's own
/// name, which a catalog tool isn't told.
pub fn catalog_tools() -> Vec<Box<dyn Tool>> {
    vec![
        Box::new(CreateProject { relay: std::sync::Arc::new(NoRelayOps) }),
        Box::new(RenameProject { relay: std::sync::Arc::new(NoRelayOps) }),
    ]
}

pub struct CreateProject {
    pub relay: SharedRelayOps,
}

#[async_trait]
impl Tool for CreateProject {
    fn definition(&self) -> ToolDefinition {
        crate::create_project_tool_definition()
    }
    fn group(&self) -> &'static str {
        PLANNING_AGENT_TYPE
    }
    fn keywords(&self) -> &'static [&'static str] {
        &["app", "website", "build", "new project"]
    }
    async fn execute(&self, conn: &mut PoolConnection<Postgres>, session_id: Uuid, _agent_session_id: Uuid, user_id: Uuid, input: Value) -> Result<ToolOutcome, String> {
        crate::create_project(conn, session_id, user_id, input, self.relay.as_ref()).await.map(ToolOutcome::text)
    }
}

pub struct RenameProject {
    pub relay: SharedRelayOps,
}

#[async_trait]
impl Tool for RenameProject {
    fn definition(&self) -> ToolDefinition {
        crate::rename_project_tool_definition()
    }
    fn group(&self) -> &'static str {
        PLANNING_AGENT_TYPE
    }
    fn keywords(&self) -> &'static [&'static str] {
        &["rename project", "change title", "change project name", "company name"]
    }
    async fn execute(&self, conn: &mut PoolConnection<Postgres>, session_id: Uuid, _agent_session_id: Uuid, user_id: Uuid, input: Value) -> Result<ToolOutcome, String> {
        crate::rename_project(conn, session_id, user_id, input, self.relay.as_ref()).await.map(ToolOutcome::text)
    }
}

