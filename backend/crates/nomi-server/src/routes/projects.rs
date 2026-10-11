use axum::extract::{Path, Query, State};
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::app::AppState;
use nomi_turn::bootstrap::ensure_web_channel_identity;
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
    /// Who started it (their display name, or the start of their email).
    pub created_by: String,
    pub open_issues: i64,
}

#[derive(Debug, Deserialize)]
pub struct ListProjectsQuery {
    pub org_id: Option<Uuid>,
}

#[derive(Serialize)]
pub struct ListProjectsResponse {
    /// The organization shown; null until the person picks one for projects.
    pub org_id: Option<Uuid>,
    pub projects: Vec<ProjectSummary>,
}

fn internal(context: &'static str) -> impl Fn(sqlx::Error) -> (StatusCode, &'static str) {
    move |e| {
        tracing::error!(error = %e, "{context}");
        (StatusCode::INTERNAL_SERVER_ERROR, context)
    }
}

/// The organization the person picked for projects, while they're still an active member of it.
pub(crate) async fn project_org(pool: &sqlx::PgPool, user_id: Uuid) -> Result<Option<Uuid>, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT p.project_org_id FROM user_preferences p \
         JOIN memberships m ON m.org_id = p.project_org_id AND m.user_id = p.user_id AND m.status = 'active' \
         WHERE p.user_id = $1",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
}

pub(crate) async fn is_active_member(pool: &sqlx::PgPool, user_id: Uuid, org_id: Uuid) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM memberships WHERE org_id = $1 AND user_id = $2 AND status = 'active')")
        .bind(org_id)
        .bind(user_id)
        .fetch_one(pool)
        .await
}

/// The projects of one organization the person is in: the one asked for, else the one they
/// picked for projects. Nothing is listed until they've picked one.
#[tracing::instrument(skip(state, claims))]
pub async fn list_projects(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Query(query): Query<ListProjectsQuery>,
) -> Result<Json<ListProjectsResponse>, (StatusCode, &'static str)> {
    let org_id = match query.org_id {
        Some(org_id) => {
            if !is_active_member(&state.pool, claims.sub, org_id).await.map_err(internal("failed to list projects"))? {
                return Err((StatusCode::NOT_FOUND, "organization not found"));
            }
            Some(org_id)
        }
        None => project_org(&state.pool, claims.sub).await.map_err(internal("failed to list projects"))?,
    };
    let Some(org_id) = org_id else {
        return Ok(Json(ListProjectsResponse { org_id: None, projects: Vec::new() }));
    };

    let projects: Vec<ProjectSummary> = sqlx::query_as(
        "SELECT p.id, p.session_id, p.name, p.description, p.status, p.created_at, p.updated_at, \
                COALESCE(NULLIF(up.display_name, ''), split_part(COALESCE(w.email, g.email, ''), '@', 1)) AS created_by, \
                (SELECT count(*) FROM project_issues i WHERE i.project_id = p.id AND i.status = 'open') AS open_issues \
         FROM projects p \
         LEFT JOIN user_profiles up ON up.user_id = p.user_id \
         LEFT JOIN web_credentials w ON w.user_id = p.user_id \
         LEFT JOIN google_identities g ON g.user_id = p.user_id \
         WHERE p.org_id = $1 ORDER BY p.updated_at DESC",
    )
    .bind(org_id)
    .fetch_all(&state.pool)
    .await
    .map_err(internal("failed to list projects"))?;

    Ok(Json(ListProjectsResponse { org_id: Some(org_id), projects }))
}

#[derive(Serialize)]
pub struct CreateProjectSessionResponse {
    pub session_id: Uuid,
    pub project_id: Uuid,
}

#[derive(Deserialize, Default)]
pub struct CreateProjectRequest {
    /// The organization the project belongs to; the one picked for projects when left out.
    pub org_id: Option<Uuid>,
}

/// The "+ Add new project" entry point: creates the project row *before* any conversation
/// happens, in the same request that creates the chat session, rather than waiting on the
/// planning agent to remember to call its own create_project tool mid-conversation. That
/// dependency was unreliable in practice (see the delegation-guardrail fix) — a session could
/// end up looking like a plain chat, with no project, even after the user explicitly asked to
/// start one from this button. Placeholder-named "New project"; the planning agent's
/// create_project renames this same row once it knows what's actually being built, rather than
/// inserting a second one (see create_project in nomi-agent-planning).
///
/// A project always belongs to an organization: until the person has picked one for projects,
/// this answers 409 and the app asks them to.
#[tracing::instrument(skip(state, claims, req))]
pub async fn create_project_session(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    req: Option<Json<CreateProjectRequest>>,
) -> Result<(StatusCode, Json<CreateProjectSessionResponse>), (StatusCode, &'static str)> {
    let req = req.map(|Json(req)| req).unwrap_or_default();
    let org_id = match req.org_id {
        Some(org_id) => {
            if !is_active_member(&state.pool, claims.sub, org_id).await.map_err(internal("failed to create project"))? {
                return Err((StatusCode::NOT_FOUND, "organization not found"));
            }
            org_id
        }
        None => project_org(&state.pool, claims.sub)
            .await
            .map_err(internal("failed to create project"))?
            .ok_or((StatusCode::CONFLICT, "choose an organization for projects first"))?,
    };

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

    let project_id: Uuid = sqlx::query_scalar(
        "INSERT INTO projects (user_id, session_id, org_id, name) VALUES ($1, $2, $3, 'New project') RETURNING id",
    )
    .bind(claims.sub)
    .bind(bootstrap_result.session_id)
    .bind(org_id)
    .fetch_one(&state.pool)
    .await
    .map_err(internal("failed to create project"))?;

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
    /// sveltekit, react, vue, astro or static (see nomi_agent_planning::STACKS).
    pub stack: String,
    /// Goes up on every file change; the preview copies changes in when it moves.
    pub files_version: i64,
    pub files: Vec<ProjectFileSummary>,
    /// The project's chat.
    pub session_id: Uuid,
    pub org_id: Uuid,
    pub org_name: String,
    pub open_issues: i64,
}

/// The chat, organization and open issue count of a project.
async fn project_context(pool: &sqlx::PgPool, project_id: Uuid) -> Result<(Uuid, Uuid, String, i64), sqlx::Error> {
    sqlx::query_as(
        "SELECT p.session_id, p.org_id, o.name, \
                (SELECT count(*) FROM project_issues i WHERE i.project_id = p.id AND i.status = 'open') \
         FROM projects p JOIN organizations o ON o.id = p.org_id WHERE p.id = $1",
    )
    .bind(project_id)
    .fetch_one(pool)
    .await
}

pub(crate) async fn load_owned_project(
    pool: &sqlx::PgPool,
    project_id: Uuid,
    user_id: Uuid,
) -> Result<Option<(String, Option<String>, Option<String>, String, String, i64)>, sqlx::Error> {
    sqlx::query_as(
        "SELECT name, description, plan, status, stack, files_version FROM projects WHERE id = $1 AND can_open_project(id, $2)",
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
    let (name, description, plan, status, stack, files_version) = row.ok_or((StatusCode::NOT_FOUND, "project not found"))?;

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

    let (session_id, org_id, org_name, open_issues) = project_context(&state.pool, project_id).await.map_err(|e| {
        tracing::error!(error = %e, "failed to load project context");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to load project")
    })?;

    Ok(Json(ProjectDetailResponse {
        id: project_id,
        name,
        description,
        plan,
        status,
        stack,
        files_version,
        files,
        session_id,
        org_id,
        org_name,
        open_issues,
    }))
}

async fn load_owned_project_by_session(
    pool: &sqlx::PgPool,
    session_id: Uuid,
    user_id: Uuid,
) -> Result<Option<(Uuid, String, Option<String>, Option<String>, String, String, i64)>, sqlx::Error> {
    // A session can accumulate more than one project if the user asks to build multiple things
    // in the same chat over time — most recent wins, so the workspace page always matches what
    // the conversation most recently started building.
    sqlx::query_as(
        "SELECT id, name, description, plan, status, stack, files_version FROM projects \
         WHERE session_id = $1 AND can_open_project(id, $2) ORDER BY created_at DESC LIMIT 1",
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
    let (project_id, name, description, plan, status, stack, files_version) = row.ok_or((StatusCode::NOT_FOUND, "no project for this session"))?;

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

    let (session_id, org_id, org_name, open_issues) = project_context(&state.pool, project_id).await.map_err(|e| {
        tracing::error!(error = %e, "failed to load project context");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to load project")
    })?;

    Ok(Json(ProjectDetailResponse {
        id: project_id,
        name,
        description,
        plan,
        status,
        stack,
        files_version,
        files,
        session_id,
        org_id,
        org_name,
        open_issues,
    }))
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

    let mut conn = state.pool.acquire().await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    nomi_agent_coding::files::save_file(&mut conn, &state.project_storage, project_id, &path, &req.content)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;
    nomi_agent_coding::files::bump_files_version(&mut conn, project_id).await.map_err(|e| {
        tracing::error!(error = %e, "failed to bump project files version");
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

    let mut conn = state.pool.acquire().await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    nomi_agent_coding::files::remove_file(&mut conn, &state.project_storage, project_id, &path).await.map_err(|e| {
        tracing::error!(error = %e, "failed to delete project file");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to delete file".to_string())
    })?;
    nomi_agent_coding::files::bump_files_version(&mut conn, project_id).await.map_err(|e| {
        tracing::error!(error = %e, "failed to bump project files version");
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
