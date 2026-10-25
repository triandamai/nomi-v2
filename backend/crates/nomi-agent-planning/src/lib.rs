use std::borrow::Cow;

use nomi_agent_core::project_ops::{publish_from_tool, publish_tool_definition, NoProjectOps, SharedProjectOps, PUBLISH_TOOL_NAME};
use async_trait::async_trait;
use serde_json::{json, Value};
use sqlx::pool::PoolConnection;
use sqlx::Postgres;
use uuid::Uuid;

use nomi_agent_core::prompts::{PLANNING_APP_PLAYBOOK, PLANNING_SYSTEM_PROMPT};
use nomi_agent_core::SubAgent;
use nomi_llm::{ContentBlock, LlmMessage, LlmRole, ToolDefinition};

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

pub struct PlanningAgent {
    project_ops: SharedProjectOps,
}

impl PlanningAgent {
    pub fn new() -> Self {
        Self { project_ops: std::sync::Arc::new(NoProjectOps) }
    }

    /// Publishing (`publish_project`) through the server's Shipyard access.
    pub fn with_project_ops(mut self, project_ops: SharedProjectOps) -> Self {
        self.project_ops = project_ops;
        self
    }
}

#[async_trait]
impl SubAgent for PlanningAgent {
    /// The app-building playbook (questions, create_project, the plan, handing it to Koda) only
    /// when this is about building something: a project chat, or a conversation where the person
    /// asked for an app, site or the like. Trips, schedules and other plans don't get it.
    async fn domain_context(
        &self,
        conn: &mut PoolConnection<Postgres>,
        _s3: Option<&nomi_storage::S3Config>,
        session_id: Uuid,
        _user_id: Uuid,
        messages: &[LlmMessage],
    ) -> Option<String> {
        let asked_to_build = messages
            .iter()
            .filter(|m| m.role == LlmRole::User)
            .flat_map(|m| m.content.iter())
            .any(|block| matches!(block, ContentBlock::Text { text } if nomi_agent_core::build_intent::wants_to_build(text)));
        if asked_to_build {
            return Some(PLANNING_APP_PLAYBOOK.to_string());
        }
        let project_chat: bool = sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM projects WHERE session_id = $1)")
            .bind(session_id)
            .fetch_one(&mut **conn)
            .await
            .unwrap_or(false);
        project_chat.then(|| PLANNING_APP_PLAYBOOK.to_string())
    }

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
        vec![create_project_tool_definition(), publish_tool_definition()]
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
            PUBLISH_TOOL_NAME => publish_from_tool(self.project_ops.as_ref(), user_id, &input).await.map(nomi_agent_core::ToolOutcome::text),
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

    /// Rena's tool results (a project's id, a saved plan) are bookkeeping: they show as steps in
    /// her reply's activity, not as messages of their own.
    fn surfaces_activity(&self) -> bool {
        false
    }

    fn supports_todos(&self) -> bool {
        true
    }

    /// Rena only names and describes the chat's own project: nothing to approve.
    fn tool_needs_approval(&self, tool_name: &str) -> bool {
        tool_name != "create_project"
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
        "SELECT id FROM projects WHERE session_id = $1 AND can_open_project(id, $2) ORDER BY created_at DESC LIMIT 1",
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

    Ok(format!(
        "Project ready (id {project_id}). When you hand the build to the coding agent, start the task with 'Project {project_id}:'."
    ))
}
