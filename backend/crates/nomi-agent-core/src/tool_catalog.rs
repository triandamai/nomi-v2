use std::collections::HashMap;

use async_trait::async_trait;
use serde_json::Value;
use sqlx::pool::PoolConnection;
use sqlx::Postgres;
use uuid::Uuid;

use nomi_llm::ToolDefinition;

use crate::content_block::ToolOutcome;

/// One grantable tool's execution behavior — an adapter over an existing free function that
/// already lives in one of the built-in agent crates. Implementors ignore whichever of
/// `session_id`/`agent_session_id`/`input` the underlying function doesn't need.
#[async_trait]
pub trait CatalogTool: Send + Sync {
    async fn execute(
        &self,
        conn: &mut PoolConnection<Postgres>,
        session_id: Uuid,
        agent_session_id: Uuid,
        user_id: Uuid,
        input: Value,
    ) -> Result<ToolOutcome, String>;
}

/// A fixed, Rust-reviewed catalog of tools a `DynamicAgent` can be granted — never arbitrary
/// code, never admin-configurable beyond picking from this set. Built once at startup (see
/// `nomi_server::build_tool_catalog`) and shared behind an `Arc` everywhere it's needed.
pub struct ToolCatalog {
    entries: HashMap<&'static str, (ToolDefinition, Box<dyn CatalogTool>)>,
}

impl ToolCatalog {
    pub fn new(entries: HashMap<&'static str, (ToolDefinition, Box<dyn CatalogTool>)>) -> Self {
        Self { entries }
    }

    /// No entries — for tests that need a `&ToolCatalog`/`Arc<ToolCatalog>` but never exercise
    /// dynamic-agent tool execution.
    pub fn empty() -> Self {
        Self { entries: HashMap::new() }
    }

    /// Every registered tool name — admin CRUD write-time validation rejects any
    /// `granted_tools` entry not in this list before it's ever saved.
    pub fn known_tool_names(&self) -> Vec<&'static str> {
        self.entries.keys().copied().collect()
    }

    /// Tolerates unknown names by skipping them (unlike write-time validation, which rejects
    /// them) — covers a tool being retired from the Rust catalog after an agent was already
    /// granted it: that agent should degrade gracefully, not break. See the design spec's
    /// "Write-time validation, runtime tolerance" note.
    pub fn definitions_for(&self, granted: &[String]) -> Vec<ToolDefinition> {
        granted.iter().filter_map(|name| self.entries.get(name.as_str())).map(|(def, _)| def.clone()).collect()
    }

    pub async fn execute(
        &self,
        name: &str,
        conn: &mut PoolConnection<Postgres>,
        session_id: Uuid,
        agent_session_id: Uuid,
        user_id: Uuid,
        input: Value,
    ) -> Result<ToolOutcome, String> {
        match self.entries.get(name) {
            Some((_, tool)) => tool.execute(conn, session_id, agent_session_id, user_id, input).await,
            None => Err(format!("tool not granted: {name}")),
        }
    }
}
