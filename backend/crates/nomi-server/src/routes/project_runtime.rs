//! What the open project page talks to while it runs the project in the browser (WebContainer;
//! see nomi_agent_coding::runtime): checking in for changed files and queued commands, posting
//! command output and the result of the check it runs on opening, and loading the files.

use std::time::{Duration, Instant};

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use futures_util::stream::{self, StreamExt, TryStreamExt};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::app::AppState;
use nomi_agent_coding::runtime::{self, ClaimedRun};
use nomi_agent_coding::{project_file_key, CODING_AGENT_TYPE};
use nomi_auth::extractor::AuthClaims;

/// The longest a check-in waits for something to happen.
const MAX_WAIT_SECS: u64 = 25;
const CHECK_IN_EVERY: Duration = Duration::from_millis(800);
/// Files read from storage at once when loading a whole project.
const LOAD_CONCURRENCY: usize = 16;
/// Nomi hands a failed check to Koda as if Nomi asked.
const REQUESTED_BY: &str = "chitchat";

type ApiError = (StatusCode, String);

fn internal(e: impl std::fmt::Display) -> ApiError {
    tracing::error!(error = %e, "project runtime");
    (StatusCode::INTERNAL_SERVER_ERROR, "something went wrong".to_string())
}

async fn ensure_owner(state: &AppState, project_id: Uuid, user_id: Uuid) -> Result<(), ApiError> {
    match crate::routes::projects::load_owned_project(&state.pool, project_id, user_id).await.map_err(internal)? {
        Some(_) => Ok(()),
        None => Err((StatusCode::NOT_FOUND, "project not found".to_string())),
    }
}

#[derive(Deserialize)]
pub struct CheckInQuery {
    /// The files version the page has; the call returns early once it moves.
    pub since: Option<i64>,
    /// Seconds to wait for a change or a command (0 = answer straight away).
    pub wait: Option<u64>,
}

#[derive(Serialize)]
pub struct CheckInResponse {
    pub files_version: i64,
    pub runs: Vec<ClaimedRun>,
}

/// `GET /api/projects/:id/runtime?since=&wait=`: marks the page online and returns the files
/// version and any commands to run, waiting up to `wait` seconds for either to change.
pub async fn check_in(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(project_id): Path<Uuid>,
    Query(query): Query<CheckInQuery>,
) -> Result<Json<CheckInResponse>, ApiError> {
    ensure_owner(&state, project_id, claims.sub).await?;
    let deadline = Instant::now() + Duration::from_secs(query.wait.unwrap_or(0).min(MAX_WAIT_SECS));
    loop {
        // A connection per round, so a waiting page doesn't hold one.
        let (files_version, runs) = {
            let mut conn = state.pool.acquire().await.map_err(internal)?;
            runtime::check_in(&mut conn, project_id).await.map_err(internal)?
        };
        if !runs.is_empty() || query.since != Some(files_version) || Instant::now() >= deadline {
            return Ok(Json(CheckInResponse { files_version, runs }));
        }
        tokio::time::sleep(CHECK_IN_EVERY).await;
    }
}

#[derive(Deserialize)]
pub struct RunResult {
    pub exit_code: Option<i32>,
    pub output: String,
}

/// `POST /api/projects/:id/runtime/runs/:run_id`: a command's output.
pub async fn finish_run(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path((project_id, run_id)): Path<(Uuid, Uuid)>,
    Json(result): Json<RunResult>,
) -> Result<StatusCode, ApiError> {
    ensure_owner(&state, project_id, claims.sub).await?;
    let mut conn = state.pool.acquire().await.map_err(internal)?;
    if runtime::finish_run(&mut conn, project_id, run_id, result.exit_code, &result.output).await.map_err(internal)? {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err((StatusCode::NOT_FOUND, "no such running command".to_string()))
    }
}

#[derive(Deserialize)]
pub struct CheckResult {
    pub files_version: i64,
    pub ok: bool,
    pub output: String,
}

#[derive(Serialize)]
pub struct CheckResponse {
    /// The failure went to Koda to fix.
    pub handed_to_koda: bool,
}

/// `POST /api/projects/:id/runtime/check`: the install-and-check the page ran on opening the
/// project. A failure goes to Koda as a task (once per version of the files).
pub async fn record_check(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(project_id): Path<Uuid>,
    Json(result): Json<CheckResult>,
) -> Result<Json<CheckResponse>, ApiError> {
    ensure_owner(&state, project_id, claims.sub).await?;
    let mut conn = state.pool.acquire().await.map_err(internal)?;
    let fix = runtime::record_check(&mut conn, project_id, result.files_version, result.ok, &result.output).await.map_err(internal)?;
    let Some(fix) = fix else { return Ok(Json(CheckResponse { handed_to_koda: false })) };
    nomi_agent_core::delegation::create_delegation(&mut conn, None, fix.session_id, REQUESTED_BY, CODING_AGENT_TYPE, &fix.task, claims.sub)
        .await
        .map_err(internal)?;
    Ok(Json(CheckResponse { handed_to_koda: true }))
}

#[derive(Serialize, sqlx::FromRow)]
pub struct ManifestEntry {
    pub path: String,
    pub size_bytes: i32,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Serialize)]
pub struct Manifest {
    pub files_version: i64,
    pub files: Vec<ManifestEntry>,
}

async fn manifest_of(state: &AppState, project_id: Uuid) -> Result<Manifest, ApiError> {
    let mut tx = state.pool.begin().await.map_err(internal)?;
    let files_version: i64 = sqlx::query_scalar("SELECT files_version FROM projects WHERE id = $1")
        .bind(project_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(internal)?;
    let files: Vec<ManifestEntry> = sqlx::query_as("SELECT path, size_bytes, updated_at FROM project_files WHERE project_id = $1 ORDER BY path")
        .bind(project_id)
        .fetch_all(&mut *tx)
        .await
        .map_err(internal)?;
    tx.commit().await.map_err(internal)?;
    Ok(Manifest { files_version, files })
}

/// `GET /api/projects/:id/manifest`: every file's path, size and last change.
pub async fn manifest(State(state): State<AppState>, AuthClaims(claims): AuthClaims, Path(project_id): Path<Uuid>) -> Result<Json<Manifest>, ApiError> {
    ensure_owner(&state, project_id, claims.sub).await?;
    Ok(Json(manifest_of(&state, project_id).await?))
}

#[derive(Serialize)]
pub struct SnapshotFile {
    pub path: String,
    pub content: String,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Serialize)]
pub struct Snapshot {
    pub files_version: i64,
    pub files: Vec<SnapshotFile>,
}

/// `GET /api/projects/:id/snapshot`: every file with its content, to start the preview.
pub async fn snapshot(State(state): State<AppState>, AuthClaims(claims): AuthClaims, Path(project_id): Path<Uuid>) -> Result<Json<Snapshot>, ApiError> {
    ensure_owner(&state, project_id, claims.sub).await?;
    let manifest = manifest_of(&state, project_id).await?;
    let store = &state.project_storage;
    let files: Vec<Option<SnapshotFile>> = stream::iter(manifest.files)
        .map(|entry| async move {
            let content = store.get_object(&project_file_key(project_id, &entry.path)).await.map_err(internal)?;
            Ok::<_, ApiError>(content.map(|content| SnapshotFile { path: entry.path, content, updated_at: entry.updated_at }))
        })
        .buffered(LOAD_CONCURRENCY)
        .try_collect()
        .await?;
    Ok(Json(Snapshot { files_version: manifest.files_version, files: files.into_iter().flatten().collect() }))
}
