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

use crate::brief::project_for;
use crate::files::{owned_project_id, project_file_key};

/// Where a project's migrations live, as plain SQL files named `0001_init.sql`, `0002_…`.
pub const MIGRATIONS_DIR: &str = "nomi/migrations/";
const BACKEND_RELAY: &str = "relay";

async fn backend_of(conn: &mut PoolConnection<Postgres>, project_id: Uuid) -> Option<String> {
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
        APPLY_MIGRATIONS_TOOL_NAME => {
            let files = migration_files(conn, storage, project_id).await?;
            relay.apply_migrations(user_id, project_id, RelayEnv::Dev, &files).await
        }
        DESCRIBE_BACKEND_TOOL_NAME => relay.describe_backend(user_id, project_id, environment_from(input)?).await,
        other => Err(format!("unknown tool: {other}")),
    }
}
