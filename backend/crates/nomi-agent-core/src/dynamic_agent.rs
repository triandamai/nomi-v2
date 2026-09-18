use std::borrow::Cow;
use std::sync::Arc;

use async_trait::async_trait;
use serde_json::Value;
use sqlx::pool::PoolConnection;
use sqlx::Postgres;
use uuid::Uuid;

use nomi_llm::ToolDefinition;

use crate::content_block::ToolOutcome;
use crate::subagent::SubAgent;
use crate::tool_catalog::ToolCatalog;

#[derive(Debug, Clone, PartialEq)]
pub struct DynamicAgentRow {
    pub id: Uuid,
    pub name: String,
    pub system_prompt: String,
    pub intent_label: String,
    pub intent_description: String,
    pub granted_tools: Vec<String>,
    pub supports_todos: bool,
    pub supports_plans: bool,
    pub can_delegate: bool,
}

type DynamicAgentSqlRow = (Uuid, String, String, String, String, Vec<String>, bool, bool, bool);

fn row_from_sql(row: DynamicAgentSqlRow) -> DynamicAgentRow {
    let (id, name, system_prompt, intent_label, intent_description, granted_tools, supports_todos, supports_plans, can_delegate) = row;
    DynamicAgentRow { id, name, system_prompt, intent_label, intent_description, granted_tools, supports_todos, supports_plans, can_delegate }
}

const DYNAMIC_AGENT_COLUMNS: &str =
    "id, name, system_prompt, intent_label, intent_description, granted_tools, supports_todos, supports_plans, can_delegate";

/// Every currently-routable dynamic agent — fetched fresh on every call, no cache. Used by
/// `nomi_turn::routing::classify_intent` to build the combined classification prompt and to
/// resolve a classified label into a `DynamicAgent`.
pub async fn fetch_active_dynamic_agents(conn: &mut PoolConnection<Postgres>) -> Result<Vec<DynamicAgentRow>, sqlx::Error> {
    let rows: Vec<DynamicAgentSqlRow> = sqlx::query_as(&format!(
        "SELECT {DYNAMIC_AGENT_COLUMNS} FROM dynamic_agents WHERE is_active = true"
    ))
    .fetch_all(&mut **conn)
    .await?;
    Ok(rows.into_iter().map(row_from_sql).collect())
}

/// Looks up a dynamic agent by id regardless of `is_active` — used to resume a turn already
/// mid-conversation with a dynamic agent even if it was soft-disabled since the turn started
/// (soft-disable only excludes an agent from *future* classification, not turns already
/// underway; see the design spec's Error Handling section).
pub async fn find_dynamic_agent_by_id(conn: &mut PoolConnection<Postgres>, id: Uuid) -> Result<Option<DynamicAgentRow>, sqlx::Error> {
    let row: Option<DynamicAgentSqlRow> = sqlx::query_as(&format!(
        "SELECT {DYNAMIC_AGENT_COLUMNS} FROM dynamic_agents WHERE id = $1"
    ))
    .bind(id)
    .fetch_optional(&mut **conn)
    .await?;
    Ok(row.map(row_from_sql))
}

/// An admin-authored agent, built fresh from a `dynamic_agents` row — never cached, never held
/// in `AgentRegistry`. `agent_type()` is the row's id (stringified), which is how
/// `nomi_turn::lib::resolve_agent` recognizes and re-fetches a dynamic agent when resuming a
/// turn or continuing an active `agent_sessions` row.
pub struct DynamicAgent {
    id: Uuid,
    name: String,
    system_prompt: String,
    intent_label: String,
    intent_description: String,
    granted_tools: Vec<String>,
    supports_todos_flag: bool,
    supports_plans_flag: bool,
    can_delegate_flag: bool,
    catalog: Arc<ToolCatalog>,
}

impl DynamicAgent {
    pub fn from_row(row: DynamicAgentRow, catalog: Arc<ToolCatalog>) -> Self {
        Self {
            id: row.id,
            name: row.name,
            system_prompt: row.system_prompt,
            intent_label: row.intent_label,
            intent_description: row.intent_description,
            granted_tools: row.granted_tools,
            supports_todos_flag: row.supports_todos,
            supports_plans_flag: row.supports_plans,
            can_delegate_flag: row.can_delegate,
            catalog,
        }
    }
}

#[async_trait]
impl SubAgent for DynamicAgent {
    fn agent_type(&self) -> Cow<'static, str> {
        Cow::Owned(self.id.to_string())
    }

    fn system_prompt(&self) -> Cow<'static, str> {
        Cow::Owned(self.system_prompt.clone())
    }

    fn tools(&self) -> Vec<ToolDefinition> {
        self.catalog.definitions_for(&self.granted_tools)
    }

    async fn execute_tool(
        &self,
        conn: &mut PoolConnection<Postgres>,
        session_id: Uuid,
        agent_session_id: Uuid,
        user_id: Uuid,
        name: &str,
        input: Value,
    ) -> Result<ToolOutcome, String> {
        // Belt-and-suspenders beyond `tools()` only offering granted tools to the LLM: the
        // catalog itself doesn't know which agent is calling, so an ungranted-but-catalog-known
        // tool name (hallucinated, or left over from a stale tool list) is rejected here too.
        if !self.granted_tools.iter().any(|t| t == name) {
            return Err(format!("tool '{name}' is not granted to this agent"));
        }
        self.catalog.execute(name, conn, session_id, agent_session_id, user_id, input).await
    }

    fn intent_label(&self) -> Cow<'static, str> {
        Cow::Owned(self.intent_label.clone())
    }

    fn intent_description(&self) -> Cow<'static, str> {
        Cow::Owned(self.intent_description.clone())
    }

    fn display_name(&self) -> Cow<'static, str> {
        Cow::Owned(self.name.clone())
    }

    // v1 scope cut: both features assume a single, well-known prompt shape tuned per built-in
    // agent — extending them to admin-authored agents is a real design question of its own,
    // deliberately deferred rather than half-implemented. See the design spec's Out of Scope.
    fn uses_memory(&self) -> bool {
        false
    }

    fn uses_personality(&self) -> bool {
        false
    }

    fn can_delegate(&self) -> bool {
        self.can_delegate_flag
    }

    // Matches the built-in specialist agents (planning, coding) that do substantial background
    // work — a dynamic agent's tool calls are worth surfacing to the user by default.
    fn surfaces_activity(&self) -> bool {
        true
    }

    fn supports_todos(&self) -> bool {
        self.supports_todos_flag
    }

    fn supports_plans(&self) -> bool {
        self.supports_plans_flag
    }

    fn is_default(&self) -> bool {
        false
    }
}
