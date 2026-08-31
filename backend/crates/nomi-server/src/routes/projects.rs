use axum::extract::{Path, State};
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;
use uuid::Uuid;

use crate::app::AppState;
use nomi_agent_coding::{guess_content_type, s3_key};
use nomi_auth::extractor::AuthClaims;

#[derive(Serialize, sqlx::FromRow)]
pub struct ProjectSummary {
    pub id: Uuid,
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
        "SELECT id, name, description, status, created_at, updated_at FROM projects \
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

#[tracing::instrument(skip(state, claims))]
pub async fn get_project_file(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path((project_id, path)): Path<(Uuid, String)>,
) -> Result<Response, (StatusCode, String)> {
    if load_owned_project(&state.pool, project_id, claims.sub).await.map_err(|e| {
        tracing::error!(error = %e, "failed to load project");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to load project".to_string())
    })?.is_none() {
        return Err((StatusCode::NOT_FOUND, "project not found".to_string()));
    }

    let Some(s3) = &state.s3 else {
        return Err((StatusCode::SERVICE_UNAVAILABLE, "code storage is not configured".to_string()));
    };

    let content = s3.get_object(&s3_key(project_id, &path)).await.map_err(|e| {
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
    if load_owned_project(&state.pool, project_id, claims.sub).await.map_err(|e| {
        tracing::error!(error = %e, "failed to load project");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to load project".to_string())
    })?.is_none() {
        return Err((StatusCode::NOT_FOUND, "project not found".to_string()));
    }

    let Some(s3) = &state.s3 else {
        return Err((StatusCode::SERVICE_UNAVAILABLE, "code storage is not configured".to_string()));
    };

    let content_type = guess_content_type(&path);
    s3.put_object(&s3_key(project_id, &path), &req.content, content_type).await.map_err(|e| {
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
    if load_owned_project(&state.pool, project_id, claims.sub).await.map_err(|e| {
        tracing::error!(error = %e, "failed to load project");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to load project".to_string())
    })?.is_none() {
        return Err((StatusCode::NOT_FOUND, "project not found".to_string()));
    }

    let Some(s3) = &state.s3 else {
        return Err((StatusCode::SERVICE_UNAVAILABLE, "code storage is not configured".to_string()));
    };

    s3.delete_object(&s3_key(project_id, &path)).await.map_err(|e| {
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

    let Some(s3) = &state.s3 else {
        return Err((StatusCode::SERVICE_UNAVAILABLE, "code storage is not configured".to_string()));
    };

    let content = s3.get_object(&s3_key(project_id, &path)).await.map_err(|e| {
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
