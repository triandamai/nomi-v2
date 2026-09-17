use async_trait::async_trait;
use serde_json::{json, Value};
use sqlx::pool::PoolConnection;
use sqlx::Postgres;
use uuid::Uuid;

use nomi_agent_core::prompts::PLANNING_SYSTEM_PROMPT;
use nomi_agent_core::SubAgent;
use nomi_llm::ToolDefinition;

pub const PLANNING_AGENT_TYPE: &str = "planning";

pub struct PlanningAgent;

impl PlanningAgent {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl SubAgent for PlanningAgent {
    fn agent_type(&self) -> &'static str {
        PLANNING_AGENT_TYPE
    }

    fn system_prompt(&self) -> &'static str {
        PLANNING_SYSTEM_PROMPT
    }

    fn tools(&self) -> Vec<ToolDefinition> {
        vec![ToolDefinition {
            name: "create_project".to_string(),
            description: "Create a new project for the app the user wants built. Call this once, before writing a plan.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "name": {"type": "string", "description": "Short project name"},
                    "description": {"type": "string", "description": "One-sentence description of what it does"}
                },
                "required": ["name", "description"]
            }),
        }]
    }

    async fn execute_tool(
        &self,
        conn: &mut PoolConnection<Postgres>,
        session_id: Uuid,
        _agent_session_id: Uuid,
        user_id: Uuid,
        name: &str,
        input: Value,
    ) -> Result<nomi_agent_core::ToolOutcome, String> {
        match name {
            "create_project" => create_project(conn, session_id, user_id, input).await.map(nomi_agent_core::ToolOutcome::text),
            other => Err(format!("unknown tool: {other}")),
        }
    }

    fn intent_label(&self) -> &'static str {
        PLANNING_AGENT_TYPE
    }

    fn intent_description(&self) -> &'static str {
        "the user wants to build, create, or plan an app, website, or script"
    }

    fn uses_personality(&self) -> bool {
        true
    }

    fn can_delegate(&self) -> bool {
        true
    }

    fn surfaces_activity(&self) -> bool {
        true
    }

    fn supports_todos(&self) -> bool {
        true
    }

    fn supports_plans(&self) -> bool {
        true
    }
}

/// A project row may already exist for this session — the "+ Add new project" entry point
/// creates one upfront (placeholder-named "New project"), before this tool is ever called, so
/// the workspace page recognizes the session as a project immediately rather than depending on
/// the model reliably calling this tool mid-conversation. Rename that existing row instead of
/// inserting a second one; only insert fresh when a session organically becomes a project with
/// no pre-existing row (e.g. "build me X" typed into a plain chat).
async fn create_project(
    conn: &mut PoolConnection<Postgres>,
    session_id: Uuid,
    user_id: Uuid,
    input: Value,
) -> Result<String, String> {
    let name = input.get("name").and_then(|v| v.as_str()).ok_or("name is required")?;
    let description = input.get("description").and_then(|v| v.as_str());

    let existing_id: Option<Uuid> = sqlx::query_scalar(
        "SELECT id FROM projects WHERE session_id = $1 AND user_id = $2 ORDER BY created_at DESC LIMIT 1",
    )
    .bind(session_id)
    .bind(user_id)
    .fetch_optional(&mut **conn)
    .await
    .map_err(|e| e.to_string())?;

    let project_id = match existing_id {
        Some(id) => {
            sqlx::query("UPDATE projects SET name = $1, description = $2, updated_at = now() WHERE id = $3")
                .bind(name)
                .bind(description)
                .bind(id)
                .execute(&mut **conn)
                .await
                .map_err(|e| e.to_string())?;
            id
        }
        None => sqlx::query_scalar(
            "INSERT INTO projects (user_id, session_id, name, description) VALUES ($1, $2, $3, $4) RETURNING id",
        )
        .bind(user_id)
        .bind(session_id)
        .bind(name)
        .bind(description)
        .fetch_one(&mut **conn)
        .await
        .map_err(|e| e.to_string())?,
    };

    Ok(project_id.to_string())
}
