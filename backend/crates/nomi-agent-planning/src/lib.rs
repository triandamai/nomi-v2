use async_trait::async_trait;
use serde_json::{json, Value};
use sqlx::pool::PoolConnection;
use sqlx::Postgres;
use uuid::Uuid;

use nomi_agent_core::SubAgent;
use nomi_llm::ToolDefinition;
use nomi_storage::S3Config;

pub const PLANNING_AGENT_TYPE: &str = "planning";

const PLANNING_SYSTEM_PROMPT: &str =
    "You help the user plan an app or script they want built. When they describe what they want, \
     call create_project with a short name and one-sentence description, then call write_plan \
     with the project_id it returns and a concise markdown plan: the files you intend to create \
     and the approach. The user will see this plan before anything gets built. After write_plan \
     succeeds, call delegate_to_agent with target_agent 'coding' and a task string that includes \
     the project ID verbatim, formatted exactly as 'Project <project_id>: <short summary of the \
     plan>' — the coding agent has no other way to know which project to write files into. Then \
     tell the user you'll let them know once it's built, and call complete_task. If project \
     creation fails because storage isn't configured, tell the user plainly that building apps \
     isn't available right now — don't retry.";

pub struct PlanningAgent {
    s3: Option<S3Config>,
}

impl PlanningAgent {
    pub fn new(s3: Option<S3Config>) -> Self {
        Self { s3 }
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
        vec![
            ToolDefinition {
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
            },
            ToolDefinition {
                name: "write_plan".to_string(),
                description: "Write or replace the build plan for a project. The user sees this before building starts.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "project_id": {"type": "string", "description": "The project ID from create_project"},
                        "plan": {"type": "string", "description": "The plan, in markdown"}
                    },
                    "required": ["project_id", "plan"]
                }),
            },
        ]
    }

    async fn execute_tool(
        &self,
        conn: &mut PoolConnection<Postgres>,
        session_id: Uuid,
        _agent_session_id: Uuid,
        user_id: Uuid,
        name: &str,
        input: Value,
    ) -> Result<String, String> {
        match name {
            "create_project" => create_project(conn, self.s3.is_some(), session_id, user_id, input).await,
            "write_plan" => write_plan(conn, user_id, input).await,
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
}

async fn create_project(
    conn: &mut PoolConnection<Postgres>,
    s3_configured: bool,
    session_id: Uuid,
    user_id: Uuid,
    input: Value,
) -> Result<String, String> {
    if !s3_configured {
        return Err("project creation is not available right now — code storage isn't configured".to_string());
    }

    let name = input.get("name").and_then(|v| v.as_str()).ok_or("name is required")?;
    let description = input.get("description").and_then(|v| v.as_str());

    let project_id: Uuid = sqlx::query_scalar(
        "INSERT INTO projects (user_id, session_id, name, description) VALUES ($1, $2, $3, $4) RETURNING id",
    )
    .bind(user_id)
    .bind(session_id)
    .bind(name)
    .bind(description)
    .fetch_one(&mut **conn)
    .await
    .map_err(|e| e.to_string())?;

    Ok(project_id.to_string())
}

async fn write_plan(conn: &mut PoolConnection<Postgres>, user_id: Uuid, input: Value) -> Result<String, String> {
    let project_id_str = input.get("project_id").and_then(|v| v.as_str()).ok_or("project_id is required")?;
    let project_id: Uuid = project_id_str.parse().map_err(|_| "project_id is not a valid UUID".to_string())?;
    let plan = input.get("plan").and_then(|v| v.as_str()).ok_or("plan is required")?;

    let updated = sqlx::query("UPDATE projects SET plan = $1, updated_at = now() WHERE id = $2 AND user_id = $3")
        .bind(plan)
        .bind(project_id)
        .bind(user_id)
        .execute(&mut **conn)
        .await
        .map_err(|e| e.to_string())?;

    if updated.rows_affected() == 0 {
        return Err("project not found".to_string());
    }

    Ok("plan saved".to_string())
}
