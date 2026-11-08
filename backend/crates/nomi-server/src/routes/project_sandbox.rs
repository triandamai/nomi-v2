//! What the project page asks about a project's server-side sandbox: whether it can be used,
//! what it is doing, its preview address, the dev server's log, and starting or stopping it.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::app::AppState;
use crate::configuration::{sandbox_config, SandboxConfig};
use crate::sandbox_store::{self as store, State as SandboxState, Status, PHASE_IDLE};
use crate::sandbox_worker::PROVIDER_NAME;
use nomi_auth::extractor::AuthClaims;

/// The project runs in the user's browser (WebContainer).
pub const RUNTIME_BROWSER: &str = "browser";
/// The project runs in a server-side sandbox.
pub const RUNTIME_SANDBOX: &str = "sandbox";

type ApiError = (StatusCode, String);

fn internal(e: impl std::fmt::Display) -> ApiError {
    tracing::error!(error = %e, "project sandbox");
    (StatusCode::INTERNAL_SERVER_ERROR, "something went wrong".to_string())
}

/// The project's stored runtime and its owner, for anyone who can open it; 404 for anyone else.
async fn load_project(state: &AppState, project_id: Uuid, user_id: Uuid) -> Result<(String, Uuid), ApiError> {
    sqlx::query_as("SELECT runtime, user_id FROM projects WHERE id = $1 AND can_open_project(id, $2)")
        .bind(project_id)
        .bind(user_id)
        .fetch_optional(&state.pool)
        .await
        .map_err(internal)?
        .ok_or((StatusCode::NOT_FOUND, "project not found".to_string()))
}

/// Whether the sandbox can be used for this project by this person right now: the admin switch
/// is on, the rollout reaches them (the owner of a project already in a sandbox always does), the
/// row isn't cooling down after a provider error, and the owner has minutes left this month.
/// Running out of minutes only stops new starts: a sandbox that is up idles out as usual.
pub fn sandbox_available(config: Option<&SandboxConfig>, runtime: &str, is_owner: bool, user_id: Uuid, status: Option<&Status>, within_allowance: bool) -> bool {
    let Some(config) = config else { return false };
    let reached = (runtime == RUNTIME_SANDBOX && is_owner) || config.rollout.includes(user_id);
    reached && within_allowance && !status.is_some_and(cooling_down)
}

/// A row in `error` that is held back until a later time.
fn cooling_down(status: &Status) -> bool {
    status.state == SandboxState::Error && status.retry_at.is_some_and(|at| at > Utc::now())
}

#[derive(Serialize)]
pub struct SandboxResponse {
    pub runtime: String,
    pub available: bool,
    pub state: &'static str,
    pub phase: &'static str,
    /// Only while the sandbox is running.
    pub preview_url: Option<String>,
    pub error: Option<String>,
}

async fn describe(state: &AppState, project_id: Uuid, user_id: Uuid, runtime: String, owner: Uuid) -> Result<SandboxResponse, ApiError> {
    let status = {
        let mut conn = state.pool.acquire().await.map_err(internal)?;
        store::status(&mut conn, project_id).await.map_err(internal)?
    };
    let config = sandbox_config().await;
    let within_allowance = match &config {
        Some(cfg) => {
            let mut conn = state.pool.acquire().await.map_err(internal)?;
            crate::sandbox_usage::within_allowance(&mut conn, owner, cfg).await.map_err(internal)?
        }
        None => false,
    };
    let available = sandbox_available(config.as_ref(), &runtime, owner == user_id, user_id, status.as_ref(), within_allowance);
    let sandbox_state = status.as_ref().map_or(SandboxState::None, |s| s.state);
    let running = sandbox_state == SandboxState::Running;
    Ok(SandboxResponse {
        runtime,
        available,
        state: sandbox_state.as_str(),
        phase: status.as_ref().map_or(PHASE_IDLE, |s| s.state.phase()),
        preview_url: status.as_ref().filter(|_| running).and_then(|s| s.preview_url.clone()),
        error: status.filter(|s| s.state == SandboxState::Error).and_then(|s| s.last_error),
    })
}

/// `GET /api/projects/:id/sandbox`
pub async fn get_sandbox(State(state): State<AppState>, AuthClaims(claims): AuthClaims, Path(project_id): Path<Uuid>) -> Result<Json<SandboxResponse>, ApiError> {
    let (runtime, owner) = load_project(&state, project_id, claims.sub).await?;
    Ok(Json(describe(&state, project_id, claims.sub, runtime, owner).await?))
}

/// `POST /api/projects/:id/sandbox/start`: asks for the sandbox to be running. Answers 409 when
/// the project doesn't use a sandbox or the sandbox can't be used now (switch off, not in the
/// rollout, cooling down); nothing is started then.
pub async fn start_sandbox(State(state): State<AppState>, AuthClaims(claims): AuthClaims, Path(project_id): Path<Uuid>) -> Result<Json<SandboxResponse>, ApiError> {
    let (runtime, owner) = load_project(&state, project_id, claims.sub).await?;
    let current = describe(&state, project_id, claims.sub, runtime.clone(), owner).await?;
    if runtime != RUNTIME_SANDBOX {
        return Err((StatusCode::CONFLICT, "this project runs in the browser".to_string()));
    }
    if !current.available {
        return Err((StatusCode::CONFLICT, "the sandbox isn't available right now; the project runs in the browser".to_string()));
    }
    let mut conn = state.pool.acquire().await.map_err(internal)?;
    store::want_running(&mut conn, project_id, PROVIDER_NAME).await.map_err(internal)?;
    drop(conn);
    Ok(Json(describe(&state, project_id, claims.sub, runtime, owner).await?))
}

/// `POST /api/projects/:id/sandbox/stop`: asks for the sandbox to be stopped. Allowed with the
/// switch off too: stopping never starts anything.
pub async fn stop_sandbox(State(state): State<AppState>, AuthClaims(claims): AuthClaims, Path(project_id): Path<Uuid>) -> Result<Json<SandboxResponse>, ApiError> {
    let (runtime, owner) = load_project(&state, project_id, claims.sub).await?;
    let mut conn = state.pool.acquire().await.map_err(internal)?;
    store::want_stopped(&mut conn, project_id).await.map_err(internal)?;
    drop(conn);
    Ok(Json(describe(&state, project_id, claims.sub, runtime, owner).await?))
}

#[derive(Deserialize)]
pub struct LogsQuery {
    /// How far into the log the page has read; defaults to the start.
    pub offset: Option<usize>,
}

#[derive(Serialize)]
pub struct LogsResponse {
    pub text: String,
    /// Pass this as `offset` next time to get only what came after.
    pub next_offset: usize,
}

/// `GET /api/projects/:id/sandbox/logs?offset=N`: the dev server's output from byte `offset` on.
/// An offset past the end (the log was trimmed or restarted) starts over from the beginning.
pub async fn sandbox_logs(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(project_id): Path<Uuid>,
    Query(query): Query<LogsQuery>,
) -> Result<Json<LogsResponse>, ApiError> {
    load_project(&state, project_id, claims.sub).await?;
    let log = crate::sandbox_devserver::log_tail(project_id);
    let mut from = query.offset.unwrap_or(0);
    if from > log.len() {
        from = 0;
    }
    while !log.is_char_boundary(from) {
        from += 1;
    }
    Ok(Json(LogsResponse { text: log[from..].to_string(), next_offset: log.len() }))
}
