use axum::extract::{Path, State};
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;
use uuid::Uuid;

use crate::app::AppState;
use crate::web_identity::ensure_web_channel_identity;
use nomi_agent_coding::{guess_content_type, project_file_key, validate_path};
use nomi_auth::extractor::AuthClaims;
use nomi_turn::bootstrap::bootstrap_identity_and_session;

#[derive(Serialize, sqlx::FromRow)]
pub struct ProjectSummary {
    pub id: Uuid,
    pub session_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[tracing::instrument(skip(state, claims))]
pub async fn list_projects(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
) -> Result<Json<Vec<ProjectSummary>>, (StatusCode, &'static str)> {
    let projects: Vec<ProjectSummary> = sqlx::query_as(
        "SELECT id, session_id, name, description, status, created_at, updated_at FROM projects \
         WHERE user_id = $1 ORDER BY created_at DESC",
    )
    .bind(claims.sub)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "failed to list projects");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to list projects")
    })?;

    Ok(Json(projects))
}

#[derive(Serialize)]
pub struct CreateProjectSessionResponse {
    pub session_id: Uuid,
    pub project_id: Uuid,
}

/// The "+ Add new project" entry point: creates the project row *before* any conversation
/// happens, in the same request that creates the chat session, rather than waiting on the
/// planning agent to remember to call its own create_project tool mid-conversation. That
/// dependency was unreliable in practice (see the delegation-guardrail fix) — a session could
/// end up looking like a plain chat, with no project, even after the user explicitly asked to
/// start one from this button. Placeholder-named "New project"; the planning agent's
/// create_project renames this same row once it knows what's actually being built, rather than
/// inserting a second one (see create_project in nomi-agent-planning).
#[tracing::instrument(skip(state, claims))]
pub async fn create_project_session(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
) -> Result<(StatusCode, Json<CreateProjectSessionResponse>), (StatusCode, &'static str)> {
    ensure_web_channel_identity(&state.pool, claims.sub).await.map_err(|e| {
        tracing::error!(error = %e, "failed to resolve web identity");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to create project")
    })?;

    let chat_id = Uuid::new_v4().to_string();
    let bootstrap_result = bootstrap_identity_and_session(
        &state.pool,
        "web",
        "dm",
        &chat_id,
        &claims.sub.to_string(),
        Some(claims.active_org_id),
    )
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "failed to create session");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to create project")
    })?;

    let project_id: Uuid =
        sqlx::query_scalar("INSERT INTO projects (user_id, session_id, name) VALUES ($1, $2, 'New project') RETURNING id")
            .bind(claims.sub)
            .bind(bootstrap_result.session_id)
            .fetch_one(&state.pool)
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "failed to create project row");
                (StatusCode::INTERNAL_SERVER_ERROR, "failed to create project")
            })?;

    Ok((
        StatusCode::CREATED,
        Json(CreateProjectSessionResponse { session_id: bootstrap_result.session_id, project_id }),
    ))
}

#[derive(Serialize, sqlx::FromRow)]
pub struct ProjectFileSummary {
    pub path: String,
    pub content_type: String,
    pub size_bytes: i32,
}

#[derive(Serialize)]
pub struct ProjectDetailResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub plan: Option<String>,
    pub status: String,
    pub files: Vec<ProjectFileSummary>,
}

async fn load_owned_project(
    pool: &sqlx::PgPool,
    project_id: Uuid,
    user_id: Uuid,
) -> Result<Option<(String, Option<String>, Option<String>, String)>, sqlx::Error> {
    sqlx::query_as(
        "SELECT name, description, plan, status FROM projects WHERE id = $1 AND user_id = $2",
    )
    .bind(project_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await
}

#[tracing::instrument(skip(state, claims))]
pub async fn get_project(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(project_id): Path<Uuid>,
) -> Result<Json<ProjectDetailResponse>, (StatusCode, &'static str)> {
    let row = load_owned_project(&state.pool, project_id, claims.sub).await.map_err(|e| {
        tracing::error!(error = %e, "failed to load project");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to load project")
    })?;
    let (name, description, plan, status) = row.ok_or((StatusCode::NOT_FOUND, "project not found"))?;

    let files: Vec<ProjectFileSummary> = sqlx::query_as(
        "SELECT path, content_type, size_bytes FROM project_files WHERE project_id = $1 ORDER BY path",
    )
    .bind(project_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "failed to list project files");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to load project")
    })?;

    Ok(Json(ProjectDetailResponse { id: project_id, name, description, plan, status, files }))
}

async fn load_owned_project_by_session(
    pool: &sqlx::PgPool,
    session_id: Uuid,
    user_id: Uuid,
) -> Result<Option<(Uuid, String, Option<String>, Option<String>, String)>, sqlx::Error> {
    // A session can accumulate more than one project if the user asks to build multiple things
    // in the same chat over time — most recent wins, so the workspace page always matches what
    // the conversation most recently started building.
    sqlx::query_as(
        "SELECT id, name, description, plan, status FROM projects \
         WHERE session_id = $1 AND user_id = $2 ORDER BY created_at DESC LIMIT 1",
    )
    .bind(session_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await
}

/// The project workspace page is keyed by session_id (a chat session may or may not have a
/// project attached yet) — this is how it finds out. A 404 here is a normal, expected state
/// (the user hasn't described anything to build yet), not an error.
#[tracing::instrument(skip(state, claims))]
pub async fn get_project_by_session(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(session_id): Path<Uuid>,
) -> Result<Json<ProjectDetailResponse>, (StatusCode, &'static str)> {
    let row = load_owned_project_by_session(&state.pool, session_id, claims.sub).await.map_err(|e| {
        tracing::error!(error = %e, "failed to load project by session");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to load project")
    })?;
    let (project_id, name, description, plan, status) = row.ok_or((StatusCode::NOT_FOUND, "no project for this session"))?;

    let files: Vec<ProjectFileSummary> = sqlx::query_as(
        "SELECT path, content_type, size_bytes FROM project_files WHERE project_id = $1 ORDER BY path",
    )
    .bind(project_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "failed to list project files");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to load project")
    })?;

    Ok(Json(ProjectDetailResponse { id: project_id, name, description, plan, status, files }))
}

#[tracing::instrument(skip(state, claims))]
pub async fn get_project_file(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path((project_id, path)): Path<(Uuid, String)>,
) -> Result<Response, (StatusCode, String)> {
    validate_path(&path).map_err(|e| (StatusCode::BAD_REQUEST, e))?;

    if load_owned_project(&state.pool, project_id, claims.sub).await.map_err(|e| {
        tracing::error!(error = %e, "failed to load project");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to load project".to_string())
    })?.is_none() {
        return Err((StatusCode::NOT_FOUND, "project not found".to_string()));
    }

    let content = state.project_storage.get_object(&project_file_key(project_id, &path)).await.map_err(|e| {
        tracing::error!(error = %e, "failed to read project file");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to read file".to_string())
    })?;
    let content = content.ok_or((StatusCode::NOT_FOUND, "file not found".to_string()))?;

    Ok(([(header::CONTENT_TYPE, guess_content_type(&path))], content).into_response())
}

#[derive(serde::Deserialize)]
pub struct PutFileRequest {
    pub content: String,
}

#[tracing::instrument(skip(state, claims, req))]
pub async fn put_project_file(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path((project_id, path)): Path<(Uuid, String)>,
    Json(req): Json<PutFileRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    validate_path(&path).map_err(|e| (StatusCode::BAD_REQUEST, e))?;

    if load_owned_project(&state.pool, project_id, claims.sub).await.map_err(|e| {
        tracing::error!(error = %e, "failed to load project");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to load project".to_string())
    })?.is_none() {
        return Err((StatusCode::NOT_FOUND, "project not found".to_string()));
    }

    let content_type = guess_content_type(&path);
    state.project_storage.put_object(&project_file_key(project_id, &path), &req.content, content_type).await.map_err(|e| {
        tracing::error!(error = %e, "failed to write project file");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to save file".to_string())
    })?;

    sqlx::query(
        "INSERT INTO project_files (project_id, path, content_type, size_bytes) VALUES ($1, $2, $3, $4) \
         ON CONFLICT (project_id, path) DO UPDATE SET content_type = EXCLUDED.content_type, size_bytes = EXCLUDED.size_bytes, updated_at = now()",
    )
    .bind(project_id)
    .bind(&path)
    .bind(content_type)
    .bind(req.content.len() as i32)
    .execute(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "failed to save project file metadata");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to save file".to_string())
    })?;

    Ok(StatusCode::OK)
}

#[tracing::instrument(skip(state, claims))]
pub async fn delete_project_file(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path((project_id, path)): Path<(Uuid, String)>,
) -> Result<StatusCode, (StatusCode, String)> {
    validate_path(&path).map_err(|e| (StatusCode::BAD_REQUEST, e))?;

    if load_owned_project(&state.pool, project_id, claims.sub).await.map_err(|e| {
        tracing::error!(error = %e, "failed to load project");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to load project".to_string())
    })?.is_none() {
        return Err((StatusCode::NOT_FOUND, "project not found".to_string()));
    }

    state.project_storage.delete_object(&project_file_key(project_id, &path)).await.map_err(|e| {
        tracing::error!(error = %e, "failed to delete project file");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to delete file".to_string())
    })?;
    sqlx::query("DELETE FROM project_files WHERE project_id = $1 AND path = $2")
        .bind(project_id)
        .bind(&path)
        .execute(&state.pool)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "failed to delete project file metadata");
            (StatusCode::INTERNAL_SERVER_ERROR, "failed to delete file".to_string())
        })?;

    Ok(StatusCode::OK)
}

/// Shared by both preview routes (Step 2) — one with just `:id` (defaults to `index.html`), one
/// with `:id/*path`. axum's `Path` tuple extractor requires the capture count to match the route
/// pattern exactly, so a single handler can't use `Path<(Uuid, Option<String>)>` to cover a route
/// that has no second capture at all; two thin wrappers around this shared function is the fix.
async fn render_preview(state: &AppState, user_id: Uuid, project_id: Uuid, path: String) -> Result<Response, (StatusCode, String)> {
    if load_owned_project(&state.pool, project_id, user_id).await.map_err(|e| {
        tracing::error!(error = %e, "failed to load project");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to load project".to_string())
    })?.is_none() {
        return Err((StatusCode::NOT_FOUND, "project not found".to_string()));
    }

    let content = state.project_storage.get_object(&project_file_key(project_id, &path)).await.map_err(|e| {
        tracing::error!(error = %e, "failed to read project file for preview");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to load preview".to_string())
    })?;
    let content = content.ok_or((StatusCode::NOT_FOUND, "file not found".to_string()))?;

    Ok(([(header::CONTENT_TYPE, guess_content_type(&path))], content).into_response())
}

#[tracing::instrument(skip(state, claims))]
pub async fn preview_project_index(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(project_id): Path<Uuid>,
) -> Result<Response, (StatusCode, String)> {
    render_preview(&state, claims.sub, project_id, "index.html".to_string()).await
}

#[tracing::instrument(skip(state, claims))]
pub async fn preview_project_file(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path((project_id, path)): Path<(Uuid, String)>,
) -> Result<Response, (StatusCode, String)> {
    render_preview(&state, claims.sub, project_id, path).await
}
