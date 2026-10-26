//! Koda's tools for a project whose backend is Nomi Relay (`projects.backend = 'relay'`):
//! applying its migrations to the dev backend and reading what that backend has. A Drizzle
//! project doesn't get them, and they refuse to act on one.

use serde_json::Value;
use sqlx::pool::PoolConnection;
use sqlx::Postgres;
use uuid::Uuid;

use nomi_agent_core::relay_ops::{
    apply_migrations_tool_definition, deploy_functions_tool_definition, describe_backend_tool_definition, environment_from, ProjectFile, RelayEnv,
    RelayOps, APPLY_MIGRATIONS_TOOL_NAME, DEPLOY_FUNCTIONS_TOOL_NAME, DESCRIBE_BACKEND_TOOL_NAME,
};
use nomi_llm::{LlmMessage, ToolDefinition};
use nomi_storage::ProjectStore;

use nomi_agent_core::ToolOutcome;

use crate::brief::project_for;
use crate::files::{owned_project_id, project_file_key, write_file};

/// Where a project's migrations live, as plain SQL files named `0001_init.sql`, `0002_…`.
pub const MIGRATIONS_DIR: &str = "nomi/migrations/";
/// The project's backend settings: `auth` (sign-in), `storage` (buckets) and `realtime` (tables).
pub const BACKEND_JSON_PATH: &str = "nomi/backend.json";
/// The project's hosted functions: one file per function, helpers in `_shared/`.
pub const FUNCTIONS_DIR: &str = "nomi/functions/";
/// What a functions build is made of.
const FUNCTION_SOURCE_EXTENSIONS: [&str; 5] = [".ts", ".js", ".mts", ".mjs", ".json"];
/// Where the TypeScript for the project's tables goes, rewritten after every migration.
pub const TYPES_PATH: &str = "src/lib/nomi.types.ts";
pub(crate) const BACKEND_RELAY: &str = "relay";
/// File tools whose writes can change a migration.
const WRITING_TOOLS: [&str; 4] = ["write_file", "write_files", "edit_file", "delete_file"];

fn is_migration(path: &str) -> bool {
    path.starts_with(MIGRATIONS_DIR) && path.ends_with(".sql")
}

fn is_function_source(path: &str) -> bool {
    path.starts_with(FUNCTIONS_DIR) && FUNCTION_SOURCE_EXTENSIONS.iter().any(|ext| path.ends_with(ext))
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
    vec![apply_migrations_tool_definition(), describe_backend_tool_definition(), deploy_functions_tool_definition()]
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
    read_files(storage, project_id, paths).await
}

/// Every source file under `nomi/functions/`, helpers included.
async fn function_files(conn: &mut PoolConnection<Postgres>, storage: &ProjectStore, project_id: Uuid) -> Result<Vec<ProjectFile>, String> {
    let paths: Vec<String> = sqlx::query_scalar("SELECT path FROM project_files WHERE project_id = $1 AND starts_with(path, $2) ORDER BY path")
        .bind(project_id)
        .bind(FUNCTIONS_DIR)
        .fetch_all(&mut **conn)
        .await
        .map_err(|e| e.to_string())?;
    // Byte order, the same everywhere (the database's collation may skip the `_` of `_shared`).
    let mut paths: Vec<String> = paths.into_iter().filter(|p| is_function_source(p)).collect();
    paths.sort();
    read_files(storage, project_id, paths).await
}

/// Deploys `nomi/functions/` to dev with `backend.json`'s settings for each function.
async fn deploy_to_dev(conn: &mut PoolConnection<Postgres>, storage: &ProjectStore, relay: &dyn RelayOps, user_id: Uuid, project_id: Uuid) -> Result<String, String> {
    let files = function_files(conn, storage, project_id).await?;
    if files.is_empty() {
        return Ok(format!("{FUNCTIONS_DIR} is empty, so there's nothing to deploy."));
    }
    let settings = read_backend_json(storage, project_id).await?.and_then(|b| b.get(FUNCTIONS_BLOCK).cloned()).unwrap_or_else(|| serde_json::json!({}));
    relay.deploy_functions(user_id, project_id, RelayEnv::Dev, &files, &settings).await
}

async fn read_files(storage: &ProjectStore, project_id: Uuid, paths: Vec<String>) -> Result<Vec<ProjectFile>, String> {
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
    let paths = touched_paths(input);
    let migrations = paths.iter().any(|p| is_migration(p));
    let backend_json = paths.iter().any(|p| p == BACKEND_JSON_PATH);
    let functions = paths.iter().any(|p| is_function_source(p));
    if !WRITING_TOOLS.contains(&name) || !(migrations || backend_json || functions) {
        return outcome;
    }
    let Ok(project_id) = owned_project_id(conn, user_id, input).await else { return outcome };
    if backend_of(conn, project_id).await.as_deref() != Some(BACKEND_RELAY) {
        return outcome;
    }
    let mut said = Vec::new();
    if migrations {
        match apply_to_dev(conn, storage, relay, user_id, project_id).await {
            Ok(applied) => {
                said.push(format!("Relay dev: {applied}"));
                // A table listed in `realtime` before its migration gets its trigger now.
                if !backend_json {
                    if let Ok(Some(backend)) = read_backend_json(storage, project_id).await {
                        if let Some(tables) = backend.get(REALTIME_BLOCK) {
                            said.push(block_outcome(REALTIME_BLOCK, relay.apply_realtime(user_id, project_id, RelayEnv::Dev, tables).await));
                        }
                    }
                }
            }
            Err(e) => said.push(format!(
                "Relay dev couldn't apply the migrations: {e}. Fix that file if it never applied; if it was applied before, write the next migration instead."
            )),
        }
    }
    if backend_json && name != "delete_file" {
        said.extend(backend_to_dev(storage, relay, user_id, project_id).await);
    }
    // New function code, or new settings for the functions there are.
    let settings_changed = backend_json && read_backend_json(storage, project_id).await.ok().flatten().is_some_and(|b| b.get(FUNCTIONS_BLOCK).is_some());
    if functions || settings_changed {
        let has_functions = functions || !function_files(conn, storage, project_id).await.unwrap_or_default().is_empty();
        if has_functions {
            said.push(block_outcome(FUNCTIONS_BLOCK, deploy_to_dev(conn, storage, relay, user_id, project_id).await));
        }
    }
    for line in said {
        outcome.display_text = format!("{}\n\n{line}", outcome.display_text);
    }
    outcome
}

const AUTH_BLOCK: &str = "auth";
const STORAGE_BLOCK: &str = "storage";
const REALTIME_BLOCK: &str = "realtime";
const FUNCTIONS_BLOCK: &str = "functions";

/// `nomi/backend.json`, parsed; `None` when the project has none.
async fn read_backend_json(storage: &ProjectStore, project_id: Uuid) -> Result<Option<Value>, String> {
    let Some(text) = storage.get_object(&project_file_key(project_id, BACKEND_JSON_PATH)).await.map_err(|e| e.to_string())? else { return Ok(None) };
    serde_json::from_str(&text).map(Some).map_err(|e| format!("{BACKEND_JSON_PATH} isn't valid JSON ({e}); fix it and write it again"))
}

/// How one block's save went, in the words Koda reads after the write.
fn block_outcome(block: &str, result: Result<String, String>) -> String {
    let label = match block {
        AUTH_BLOCK => "sign-in",
        other => other,
    };
    match result {
        Ok(said) => format!("Relay dev {label}: {said}"),
        Err(e) => format!("Relay dev {label} settings weren't saved: {e}"),
    }
}

/// Sends each of `nomi/backend.json`'s blocks to the dev backend: sign-in, buckets, realtime.
async fn backend_to_dev(storage: &ProjectStore, relay: &dyn RelayOps, user_id: Uuid, project_id: Uuid) -> Vec<String> {
    let backend = match read_backend_json(storage, project_id).await {
        Ok(Some(backend)) => backend,
        Ok(None) => return Vec::new(),
        Err(e) => return vec![format!("Relay dev settings weren't saved: {e}")],
    };
    let mut said = Vec::new();
    if let Some(auth) = backend.get(AUTH_BLOCK) {
        said.push(block_outcome(AUTH_BLOCK, relay.apply_auth(user_id, project_id, RelayEnv::Dev, auth).await));
    }
    if let Some(buckets) = backend.get(STORAGE_BLOCK) {
        said.push(block_outcome(STORAGE_BLOCK, relay.apply_storage(user_id, project_id, RelayEnv::Dev, buckets).await));
    }
    if let Some(tables) = backend.get(REALTIME_BLOCK) {
        said.push(block_outcome(REALTIME_BLOCK, relay.apply_realtime(user_id, project_id, RelayEnv::Dev, tables).await));
    }
    if said.is_empty() {
        said.push(format!("{BACKEND_JSON_PATH} has no auth, storage or realtime block, so nothing changed (sign-in stays email codes)."));
    }
    said
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
        DEPLOY_FUNCTIONS_TOOL_NAME => deploy_to_dev(conn, storage, relay, user_id, project_id).await,
        other => Err(format!("unknown tool: {other}")),
    }
}
