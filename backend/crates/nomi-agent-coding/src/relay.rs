//! Koda's tools for a project whose backend is Nomi Relay (`projects.backend = 'relay'`):
//! applying its migrations to the dev backend and reading what that backend has. A Drizzle
//! project doesn't get them, and they refuse to act on one.

use serde_json::Value;
use sqlx::pool::PoolConnection;
use sqlx::Postgres;
use uuid::Uuid;

use nomi_agent_core::relay_ops::{
    apply_migrations_tool_definition, describe_backend_tool_definition, environment_from, ProjectFile, RelayEnv, RelayOps,
    APPLY_MIGRATIONS_TOOL_NAME, DESCRIBE_BACKEND_TOOL_NAME,
};
use nomi_llm::{LlmMessage, ToolDefinition};
use nomi_storage::ProjectStore;

use nomi_agent_core::ToolOutcome;

use crate::brief::project_for;
use crate::files::{owned_project_id, project_file_key, write_file};

/// Where a project's migrations live, as plain SQL files named `0001_init.sql`, `0002_…`.
pub const MIGRATIONS_DIR: &str = "nomi/migrations/";
/// Where the TypeScript for the project's tables goes, rewritten after every migration.
pub const TYPES_PATH: &str = "src/lib/nomi.types.ts";
pub(crate) const BACKEND_RELAY: &str = "relay";
/// File tools whose writes can change a migration.
const WRITING_TOOLS: [&str; 4] = ["write_file", "write_files", "edit_file", "delete_file"];

fn is_migration(path: &str) -> bool {
    path.starts_with(MIGRATIONS_DIR) && path.ends_with(".sql")
}

/// The paths a file tool's input touches.
fn touched_paths(input: &Value) -> Vec<String> {
    let single = input.get("path").and_then(Value::as_str).map(str::to_string);
    let many = input.get("files").and_then(Value::as_array).into_iter().flatten().filter_map(|f| f.get("path").and_then(Value::as_str).map(str::to_string));
    single.into_iter().chain(many).collect()
}

pub(crate) async fn backend_of(conn: &mut PoolConnection<Postgres>, project_id: Uuid) -> Option<String> {
    sqlx::query_scalar("SELECT backend FROM projects WHERE id = $1").bind(project_id).fetch_optional(&mut **conn).await.ok().flatten()
}

/// The Relay tools, when the project this chat builds is on Relay.
pub(crate) async fn chat_tools(conn: &mut PoolConnection<Postgres>, session_id: Uuid, user_id: Uuid, messages: &[LlmMessage]) -> Vec<ToolDefinition> {
    let Some(project_id) = project_for(conn, session_id, user_id, messages).await else {
        return Vec::new();
    };
    if backend_of(conn, project_id).await.as_deref() != Some(BACKEND_RELAY) {
        return Vec::new();
    }
    vec![apply_migrations_tool_definition(), describe_backend_tool_definition()]
}

/// The project's migration files, in the order they apply.
async fn migration_files(conn: &mut PoolConnection<Postgres>, storage: &ProjectStore, project_id: Uuid) -> Result<Vec<ProjectFile>, String> {
    let paths: Vec<String> = sqlx::query_scalar(
        "SELECT path FROM project_files WHERE project_id = $1 AND starts_with(path, $2) AND path LIKE '%.sql' ORDER BY path",
    )
    .bind(project_id)
    .bind(MIGRATIONS_DIR)
    .fetch_all(&mut **conn)
    .await
    .map_err(|e| e.to_string())?;
    let mut files = Vec::with_capacity(paths.len());
    for path in paths {
        let content = storage.get_object(&project_file_key(project_id, &path)).await.map_err(|e| e.to_string())?.unwrap_or_default();
        files.push(ProjectFile { path, content });
    }
    Ok(files)
}

/// Applies the project's migrations to dev and, when they applied, rewrites the types file.
/// Answers what to tell Koda either way: a failure is something for him to fix, not a tool error.
async fn apply_to_dev(conn: &mut PoolConnection<Postgres>, storage: &ProjectStore, relay: &dyn RelayOps, user_id: Uuid, project_id: Uuid) -> Result<String, String> {
    let files = migration_files(conn, storage, project_id).await?;
    let applied = relay.apply_migrations(user_id, project_id, RelayEnv::Dev, &files).await?;
    match relay.types(user_id, project_id, RelayEnv::Dev).await {
        Ok(types) => {
            let input = serde_json::json!({ "project_id": project_id.to_string(), "path": TYPES_PATH, "content": types });
            write_file(conn, storage, user_id, input).await?;
            Ok(format!("{applied} {TYPES_PATH} is up to date."))
        }
        Err(e) => Ok(format!("{applied} (Couldn't refresh {TYPES_PATH}: {e})")),
    }
}

/// After a file tool wrote under `nomi/migrations/` in a Relay project: applies the migrations to
/// dev and adds the outcome to what the tool says, the way `npm run check` output comes back.
pub(crate) async fn after_write(
    conn: &mut PoolConnection<Postgres>,
    storage: &ProjectStore,
    relay: &dyn RelayOps,
    user_id: Uuid,
    name: &str,
    input: &Value,
    mut outcome: ToolOutcome,
) -> ToolOutcome {
    if !WRITING_TOOLS.contains(&name) || !touched_paths(input).iter().any(|p| is_migration(p)) {
        return outcome;
    }
    let Ok(project_id) = owned_project_id(conn, user_id, input).await else { return outcome };
    if backend_of(conn, project_id).await.as_deref() != Some(BACKEND_RELAY) {
        return outcome;
    }
    let said = match apply_to_dev(conn, storage, relay, user_id, project_id).await {
        Ok(applied) => format!("Relay dev: {applied}"),
        Err(e) => format!(
            "Relay dev couldn't apply the migrations: {e}. Fix that file if it never applied; if it was applied before, write the next migration instead."
        ),
    };
    outcome.display_text = format!("{}\n\n{said}", outcome.display_text);
    outcome
}

/// Runs `apply_migrations` or `describe_backend` for a project the person can open, on Relay.
pub(crate) async fn execute(
    conn: &mut PoolConnection<Postgres>,
    storage: &ProjectStore,
    relay: &dyn RelayOps,
    user_id: Uuid,
    name: &str,
    input: &Value,
) -> Result<String, String> {
    let project_id = owned_project_id(conn, user_id, input).await?;
    let backend = backend_of(conn, project_id).await.unwrap_or_default();
    if backend != BACKEND_RELAY {
        return Err(format!(
            "This project isn't on Relay (its backend is {backend}): its database code lives in the app. Don't call {name}."
        ));
    }
    match name {
        APPLY_MIGRATIONS_TOOL_NAME => apply_to_dev(conn, storage, relay, user_id, project_id).await,
        DESCRIBE_BACKEND_TOOL_NAME => relay.describe_backend(user_id, project_id, environment_from(input)?).await,
        other => Err(format!("unknown tool: {other}")),
    }
}
