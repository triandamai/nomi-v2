use std::borrow::Cow;

use nomi_agent_core::project_ops::{publish_from_tool, publish_tool_definition, NoProjectOps, SharedProjectOps, PUBLISH_TOOL_NAME};
use nomi_agent_core::relay_ops::{NoRelayOps, RelayOps, SharedRelayOps};
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

/// What a project's server side is built on: Nomi Relay, or the app's own Drizzle code.
pub const BACKENDS: &[&str] = &[BACKEND_RELAY, BACKEND_DRIZZLE];
pub const BACKEND_RELAY: &str = "relay";
pub const BACKEND_DRIZZLE: &str = "drizzle";

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
                },
                "backend": {
                    "type": "string",
                    "enum": BACKENDS,
                    "description": "relay for anything that keeps data on a server, signs people in, stores files or calls paid APIs (Nomi Relay gives it a database, sign-in, storage and functions); drizzle only for an app that brings its own database code. Leave it out for browser-only apps."
                }
            },
            "required": ["name", "description"]
        }),
    }
}

pub struct PlanningAgent {
    project_ops: SharedProjectOps,
    relay_ops: SharedRelayOps,
}

impl PlanningAgent {
    pub fn new() -> Self {
        Self { project_ops: std::sync::Arc::new(NoProjectOps), relay_ops: std::sync::Arc::new(NoRelayOps) }
    }

    /// Relay backends for the projects Rena creates with `backend: relay`.
    pub fn with_relay_ops(mut self, relay_ops: SharedRelayOps) -> Self {
        self.relay_ops = relay_ops;
        self
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
            "create_project" => {
                create_project(conn, session_id, user_id, input, self.relay_ops.as_ref()).await.map(nomi_agent_core::ToolOutcome::text)
            }
            PUBLISH_TOOL_NAME => publish_from_tool(self.project_ops.as_ref(), user_id, &self.display_name(), &input).await.map(nomi_agent_core::ToolOutcome::text),
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
///
/// `backend` sets what the project's server side is built on (see `choose_backend`); left out,
/// the project keeps the backend it has (Drizzle for a new one).
pub async fn create_project(
    conn: &mut PoolConnection<Postgres>,
    session_id: Uuid,
    user_id: Uuid,
    input: Value,
    relay: &dyn RelayOps,
) -> Result<String, String> {
    let name = input.get("name").and_then(|v| v.as_str()).ok_or("name is required")?;
    let description = input.get("description").and_then(|v| v.as_str());
    let stack = input.get("stack").and_then(|v| v.as_str()).map(str::trim).filter(|s| !s.is_empty()).unwrap_or("sveltekit");
    if !STACKS.contains(&stack) {
        return Err(format!("stack must be one of {}", STACKS.join(", ")));
    }
    let backend = input.get("backend").and_then(|v| v.as_str()).map(str::trim).filter(|s| !s.is_empty());
    if let Some(backend) = backend {
        if !BACKENDS.contains(&backend) {
            return Err(format!("backend must be one of {}", BACKENDS.join(", ")));
        }
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

    let backend_note = match backend {
        Some(backend) => choose_backend(conn, user_id, project_id, backend, relay).await?,
        None => None,
    };
    let backend_note = backend_note.map(|note| format!("{note} ")).unwrap_or_default();
    Ok(format!(
        "Project ready (id {project_id}). {backend_note}When you hand the build to the coding agent, start the task with 'Project {project_id}:'."
    ))
}

/// Puts the project on `backend`, and says what came of it when that isn't simply what was
/// asked. A project that already has files keeps its backend: moving an app between backends
/// is a task of its own, never a side effect of renaming it. Relay is used only when this Nomi
/// has it and it made the project's backend; otherwise the project is built on Drizzle.
async fn choose_backend(
    conn: &mut PoolConnection<Postgres>,
    user_id: Uuid,
    project_id: Uuid,
    backend: &str,
    relay: &dyn RelayOps,
) -> Result<Option<String>, String> {
    let (current, has_files): (String, bool) =
        sqlx::query_as("SELECT backend, EXISTS (SELECT 1 FROM project_files WHERE project_id = $1) FROM projects WHERE id = $1")
            .bind(project_id)
            .fetch_one(&mut **conn)
            .await
            .map_err(|e| e.to_string())?;
    if backend == current {
        return Ok(None);
    }
    if has_files {
        return Ok(Some(format!(
            "It keeps its {current} backend: it already has files, and moving an app to {backend} is a separate task."
        )));
    }
    let note = if backend == BACKEND_RELAY {
        if !relay.is_available().await {
            return Ok(Some("Relay isn't set up on this Nomi, so it's built on Drizzle.".to_string()));
        }
        if let Err(e) = relay.ensure_backend(user_id, project_id).await {
            return Ok(Some(format!("Relay couldn't make its backend ({e}), so it's built on Drizzle for now.")));
        }
        Some("Its backend is Nomi Relay: a database, sign-in, storage and functions are ready.".to_string())
    } else {
        None
    };
    sqlx::query("UPDATE projects SET backend = $1, updated_at = now() WHERE id = $2")
        .bind(backend)
        .bind(project_id)
        .execute(&mut **conn)
        .await
        .map_err(|e| e.to_string())?;
    Ok(note)
}
