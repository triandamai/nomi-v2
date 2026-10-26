//! A project's hosted functions per environment. Each deploy of `nomi/functions/` is a numbered
//! build kept in Relay's state; the functions runtime type-checks and loads it before it becomes
//! the build the gateway serves, so a broken deploy never replaces a working one. Rollback makes
//! the previous good build current again.

use std::collections::BTreeMap;
use std::time::Duration;

use relay_core::backend::{function_name, FunctionSettings, MAX_FUNCTION_TIMEOUT};
use relay_core::settings::FunctionsRuntime;
use relay_core::Env;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::backend::backend_keys;
use crate::storage::invalid;
use crate::{Relay, RelayError};

pub const FUNCTIONS_NOT_SET_UP: &str = "FUNCTIONS_NOT_SET_UP";
pub const FUNCTIONS_FAILED: &str = "FUNCTIONS_FAILED";
/// Type-checking and loading a build can take a while the first time its npm packages download.
const LOAD_TIMEOUT: Duration = Duration::from_secs(150);
/// The header the runtime trusts calls by.
pub const RUNTIME_SECRET_HEADER: &str = "x-relay-runtime";
const CHECKING: &str = "checking";
const CURRENT: &str = "current";
const REPLACED: &str = "replaced";
const FAILED: &str = "failed";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SourceFile {
    pub path: String,
    pub content: String,
}

/// The build being served: its number and each function's settings.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CurrentBuild {
    pub version: i32,
    pub functions: BTreeMap<String, FunctionSettings>,
}

#[derive(Deserialize)]
struct RuntimeAnswer {
    ok: bool,
    #[serde(default)]
    error: Option<String>,
}

fn runtime(relay: &Relay) -> Result<FunctionsRuntime, RelayError> {
    relay
        .live()?
        .settings
        .functions
        .clone()
        .filter(|f| !f.url.is_empty())
        .ok_or_else(|| RelayError::rejected(FUNCTIONS_NOT_SET_UP, "Hosted functions aren't set up on this Relay: set its functions runtime in Nomi's admin"))
}

/// Whether `schedule` is a cron expression Relay can run.
pub fn valid_schedule(schedule: &str) -> bool {
    croner::Cron::new(schedule).parse().is_ok()
}

/// Every function the files define, with its settings (defaults where `backend.json` says nothing).
fn functions_of(files: &[SourceFile], settings: &BTreeMap<String, FunctionSettings>) -> Result<BTreeMap<String, FunctionSettings>, RelayError> {
    let mut problems = Vec::new();
    let mut found = BTreeMap::new();
    for file in files {
        if let Some(name) = function_name(&file.path) {
            found.insert(name.clone(), settings.get(&name).cloned().unwrap_or_default());
        }
    }
    if found.is_empty() {
        problems.push("nomi/functions/ has no functions: one file per function, like nomi/functions/place-order.ts".to_string());
    }
    for (name, setting) in settings {
        if !found.contains_key(name) {
            problems.push(format!("backend.json's functions names {name}, but there's no nomi/functions/{name}.ts"));
        }
        if setting.timeout == 0 || setting.timeout > MAX_FUNCTION_TIMEOUT {
            problems.push(format!("{name}: timeout is 1 to {MAX_FUNCTION_TIMEOUT} seconds"));
        }
        if let Some(schedule) = &setting.schedule {
            if !valid_schedule(schedule) {
                problems.push(format!("{name}: schedule {schedule:?} isn't a cron expression like \"0 * * * *\""));
            }
        }
    }
    if problems.is_empty() {
        Ok(found)
    } else {
        Err(invalid(problems))
    }
}

/// `ctx.env`: the project's secrets, its address and keys for `env`.
async fn function_env(relay: &Relay, project: Uuid, env: Env) -> Result<BTreeMap<String, String>, RelayError> {
    let secrets = relay_auth::config::load_secrets(&relay.state, &relay.encryption_key, project).await.map_err(|e| RelayError::Internal(format!("{e:?}")))?;
    let keys = backend_keys(relay, project).await?;
    let mine = keys.environments.get(env.as_str()).ok_or_else(|| RelayError::NotFound("no backend for this project".into()))?;
    let mut out = secrets.0;
    out.insert("NOMI_URL".into(), mine.url.clone());
    out.insert("NOMI_ANON_KEY".into(), mine.anon_key.clone());
    out.insert("NOMI_SERVICE_KEY".into(), mine.service_key.clone());
    Ok(out)
}

/// Sends a build to the runtime to check and load. Answers the runtime's error when it refuses it.
async fn load(relay: &Relay, project: Uuid, env: Env, version: i32, files: &[SourceFile]) -> Result<Result<(), String>, RelayError> {
    let runtime = runtime(relay)?;
    let body = json!({ "files": files, "env": function_env(relay, project, env).await? });
    let url = format!("{}/builds/{project}/{}/{version}", runtime.url.trim_end_matches('/'), env.as_str());
    let answer = relay
        .http
        .put(url)
        .header(RUNTIME_SECRET_HEADER, &runtime.secret)
        .timeout(LOAD_TIMEOUT)
        .json(&body)
        .send()
        .await
        .map_err(|e| RelayError::Internal(format!("the functions runtime didn't answer: {e}")))?;
    let status = answer.status();
    let text = answer.text().await.unwrap_or_default();
    match serde_json::from_str::<RuntimeAnswer>(&text) {
        Ok(RuntimeAnswer { ok: true, .. }) => Ok(Ok(())),
        Ok(RuntimeAnswer { error, .. }) => Ok(Err(error.unwrap_or_else(|| "the build didn't load".into()))),
        Err(_) if status.is_client_error() => Ok(Err(serde_json::from_str::<Value>(&text).ok().and_then(|v| v["message"].as_str().map(str::to_string)).unwrap_or(text))),
        Err(_) => Err(RelayError::Internal(format!("the functions runtime answered {status}: {text}"))),
    }
}

/// Deploys `files` (all of `nomi/functions/`) to `env` with `settings` from `backend.json`.
pub async fn deploy(relay: &Relay, project: Uuid, env: Env, files: &[SourceFile], settings: &BTreeMap<String, FunctionSettings>) -> Result<CurrentBuild, RelayError> {
    runtime(relay)?;
    let functions = functions_of(files, settings)?;
    crate::state::environment(relay, project, env).await?.ok_or_else(|| RelayError::NotFound("no backend for this project".into()))?;
    let version: i32 = sqlx::query_scalar(
        "INSERT INTO relay.function_builds (project_id, env, version, files, functions, status) \
         SELECT $1, $2, COALESCE(MAX(version), 0) + 1, $3, $4, $5 FROM relay.function_builds WHERE project_id = $1 AND env = $2 \
         RETURNING version",
    )
    .bind(project)
    .bind(env.as_str())
    .bind(json!(files))
    .bind(json!(functions))
    .bind(CHECKING)
    .fetch_one(&relay.state)
    .await?;
    match load(relay, project, env, version, files).await {
        Ok(Ok(())) => {
            make_current(relay, project, env, version).await?;
            Ok(CurrentBuild { version, functions })
        }
        Ok(Err(error)) => {
            mark_failed(relay, project, env, version, &error).await?;
            Err(RelayError::rejected(FUNCTIONS_FAILED, error))
        }
        Err(e) => {
            mark_failed(relay, project, env, version, &e.to_string()).await?;
            Err(e)
        }
    }
}

async fn mark_failed(relay: &Relay, project: Uuid, env: Env, version: i32, error: &str) -> Result<(), RelayError> {
    sqlx::query("UPDATE relay.function_builds SET status = $4, error = $5 WHERE project_id = $1 AND env = $2 AND version = $3")
        .bind(project)
        .bind(env.as_str())
        .bind(version)
        .bind(FAILED)
        .bind(error)
        .execute(&relay.state)
        .await?;
    Ok(())
}

async fn make_current(relay: &Relay, project: Uuid, env: Env, version: i32) -> Result<(), RelayError> {
    let mut tx = relay.state.begin().await?;
    sqlx::query("UPDATE relay.function_builds SET status = $3 WHERE project_id = $1 AND env = $2 AND status = $4")
        .bind(project)
        .bind(env.as_str())
        .bind(REPLACED)
        .bind(CURRENT)
        .execute(&mut *tx)
        .await?;
    sqlx::query("UPDATE relay.function_builds SET status = $4, error = NULL WHERE project_id = $1 AND env = $2 AND version = $3")
        .bind(project)
        .bind(env.as_str())
        .bind(version)
        .bind(CURRENT)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(())
}

/// The build `env` serves, if it has one.
pub async fn current(relay: &Relay, project: Uuid, env: Env) -> Result<Option<CurrentBuild>, RelayError> {
    let row: Option<(i32, Value)> = sqlx::query_as("SELECT version, functions FROM relay.function_builds WHERE project_id = $1 AND env = $2 AND status = $3")
        .bind(project)
        .bind(env.as_str())
        .bind(CURRENT)
        .fetch_optional(&relay.state)
        .await?;
    row.map(|(version, functions)| {
        serde_json::from_value(functions).map(|functions| CurrentBuild { version, functions }).map_err(|e| RelayError::Internal(format!("saved functions can't be read: {e}")))
    })
    .transpose()
}

/// Sends the current build to the runtime again (it restarted and lost it).
pub async fn reload(relay: &Relay, project: Uuid, env: Env, version: i32) -> Result<(), RelayError> {
    let files: Value = sqlx::query_scalar("SELECT files FROM relay.function_builds WHERE project_id = $1 AND env = $2 AND version = $3")
        .bind(project)
        .bind(env.as_str())
        .bind(version)
        .fetch_one(&relay.state)
        .await?;
    let files: Vec<SourceFile> = serde_json::from_value(files).map_err(|e| RelayError::Internal(e.to_string()))?;
    load(relay, project, env, version, &files).await?.map_err(|error| RelayError::rejected(FUNCTIONS_FAILED, error))
}

/// After a project secret changed: each environment's current build is loaded again, so its
/// functions see the new value. A runtime that isn't set up, or an environment with no functions,
/// has nothing to do.
pub async fn secrets_changed(relay: &Relay, project: Uuid) -> Result<(), RelayError> {
    if runtime(relay).is_err() {
        return Ok(());
    }
    for env in Env::ALL {
        if let Some(build) = current(relay, project, env).await? {
            reload(relay, project, env, build.version).await?;
        }
    }
    Ok(())
}

/// Makes the newest good build before the current one current again.
pub async fn rollback(relay: &Relay, project: Uuid, env: Env) -> Result<CurrentBuild, RelayError> {
    let current = current(relay, project, env).await?.ok_or_else(|| RelayError::rejected("NOTHING_TO_ROLL_BACK", "these functions have never been deployed"))?;
    let previous: Option<(i32, Value)> = sqlx::query_as(
        "SELECT version, functions FROM relay.function_builds WHERE project_id = $1 AND env = $2 AND status = $3 AND version < $4 ORDER BY version DESC LIMIT 1",
    )
    .bind(project)
    .bind(env.as_str())
    .bind(REPLACED)
    .bind(current.version)
    .fetch_optional(&relay.state)
    .await?;
    let (version, functions) = previous.ok_or_else(|| RelayError::rejected("NOTHING_TO_ROLL_BACK", "there's no earlier build of these functions"))?;
    reload(relay, project, env, version).await?;
    make_current(relay, project, env, version).await?;
    let functions = serde_json::from_value(functions).map_err(|e| RelayError::Internal(e.to_string()))?;
    Ok(CurrentBuild { version, functions })
}

/// Checks the runtime answers with this secret.
pub async fn check_runtime(http: &reqwest::Client, runtime: &FunctionsRuntime, timeout: Duration) -> Result<(), String> {
    let answer = http
        .get(format!("{}/health", runtime.url.trim_end_matches('/')))
        .header(RUNTIME_SECRET_HEADER, &runtime.secret)
        .timeout(timeout)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    match answer.status().as_u16() {
        200 => Ok(()),
        401 => Err("it refused the secret: use the runtime's RELAY_FUNCTIONS_SECRET".into()),
        other => Err(format!("it answered {other}")),
    }
}
