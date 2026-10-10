use std::borrow::Cow;

use async_trait::async_trait;
use serde_json::{json, Value};
use sqlx::pool::PoolConnection;
use sqlx::Postgres;
use uuid::Uuid;

use nomi_agent_core::prompts::PLANNING_SYSTEM_PROMPT;
use nomi_agent_core::SubAgent;
use nomi_llm::ToolDefinition;

pub const PLANNING_AGENT_TYPE: &str = "planning";

/// The stacks a project can be built on (see the coding agent's guides).
pub const STACKS: &[&str] = &["sveltekit", "react", "vue", "astro", "static"];

pub fn create_project_tool_definition() -> ToolDefinition {
    ToolDefinition {
        name: "create_project".to_string(),
        description: "Create a new project for the app the user wants built. Call this once, before writing a plan.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "name": {"type": "string", "description": "Short project name"},
                "description": {"type": "string", "description": "One-sentence description of what it does"},
                "stack": {
                    "type": "string",
                    "enum": STACKS,
                    "description": "sveltekit (default: anything with pages and a server or database), react or vue (browser-only apps, when the user asks), astro (content sites), static (a single plain HTML page)"
                }
            },
            "required": ["name", "description"]
        }),
    }
}

pub struct PlanningAgent;

impl PlanningAgent {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl SubAgent for PlanningAgent {
    /// Remembers durable facts the user mentions here and recalls them in later turns.
    fn uses_memory(&self) -> bool {
        true
    }

    fn agent_type(&self) -> Cow<'static, str> {
        Cow::Borrowed(PLANNING_AGENT_TYPE)
    }

    fn system_prompt(&self) -> Cow<'static, str> {
        Cow::Borrowed(PLANNING_SYSTEM_PROMPT)
    }

    fn tools(&self) -> Vec<ToolDefinition> {
        vec![create_project_tool_definition()]
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

    fn intent_label(&self) -> Cow<'static, str> {
        Cow::Borrowed(PLANNING_AGENT_TYPE)
    }

    fn intent_description(&self) -> Cow<'static, str> {
        Cow::Borrowed(
            "the user wants something built (an app, web app, website, landing page, game, tool, bot or script: Rena plans it and Koda builds it), or a plan made or changed: a trip or itinerary, an event, a schedule or routine, a study or work plan",
        )
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

    fn keeps_plans_in_drafts(&self) -> bool {
        true
    }
}

/// A project row may already exist for this session — the "+ Add new project" entry point
/// creates one upfront (placeholder-named "New project"), before this tool is ever called, so
/// the workspace page recognizes the session as a project immediately rather than depending on
/// the model reliably calling this tool mid-conversation. Rename that existing row instead of
/// inserting a second one; only insert fresh when a session organically becomes a project with
/// no pre-existing row (e.g. "build me X" typed into a plain chat).
pub async fn create_project(
    conn: &mut PoolConnection<Postgres>,
    session_id: Uuid,
    user_id: Uuid,
    input: Value,
) -> Result<String, String> {
    let name = input.get("name").and_then(|v| v.as_str()).ok_or("name is required")?;
    let description = input.get("description").and_then(|v| v.as_str());
    let stack = input.get("stack").and_then(|v| v.as_str()).map(str::trim).filter(|s| !s.is_empty()).unwrap_or("sveltekit");
    if !STACKS.contains(&stack) {
        return Err(format!("stack must be one of {}", STACKS.join(", ")));
    }

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
            sqlx::query("UPDATE projects SET name = $1, description = $2, stack = $4, updated_at = now() WHERE id = $3")
                .bind(name)
                .bind(description)
                .bind(id)
                .bind(stack)
                .execute(&mut **conn)
                .await
                .map_err(|e| e.to_string())?;
            id
        }
        None => sqlx::query_scalar(
            "INSERT INTO projects (user_id, session_id, name, description, stack) VALUES ($1, $2, $3, $4, $5) RETURNING id",
        )
        .bind(user_id)
        .bind(session_id)
        .bind(name)
        .bind(description)
        .bind(stack)
        .fetch_one(&mut **conn)
        .await
        .map_err(|e| e.to_string())?,
    };

    Ok(project_id.to_string())
}
