//! Renaming a project, duplicating it, and remixing an app from the showcase: a copy is a new
//! project of the person's own, with the files, stack, description and plan of the one it came
//! from, and a chat of its own. Its publishing, data, secrets and members stay behind.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

use nomi_agent_coding::files::{bump_files_version_by, project_file_key, save_bytes};
use nomi_auth::extractor::AuthClaims;

use crate::app::AppState;
use crate::routes::deploy::{internal, project_role, require_builder, ApiError};
use crate::routes::projects::{
    is_active_member, project_org, start_project, CreateProjectSessionResponse,
    DESCRIPTION_MAX_CHARS, NAME_MAX_CHARS,
};

#[derive(Deserialize)]
pub struct RenameRequest {
    pub name: Option<String>,
    /// An empty one clears it.
    pub description: Option<String>,
}

/// `PATCH /api/projects/:id` `{ name?, description? }`: anyone who can build in it.
pub async fn rename(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(project_id): Path<Uuid>,
    Json(req): Json<RenameRequest>,
) -> Result<Json<Value>, ApiError> {
    require_builder(&state.pool, project_id, claims.sub).await?;
    let name = req.name.as_deref().map(str::trim);
    if let Some(name) = name {
        if name.is_empty() {
            return Err((
                StatusCode::BAD_REQUEST,
                "a project needs a name".to_string(),
            ));
        }
        if name.chars().count() > NAME_MAX_CHARS {
            return Err((
                StatusCode::BAD_REQUEST,
                format!("a name can be at most {NAME_MAX_CHARS} characters"),
            ));
        }
    }
    let description = req.description.as_deref().map(str::trim);
    if description.is_some_and(|d| d.chars().count() > DESCRIPTION_MAX_CHARS) {
        return Err((
            StatusCode::BAD_REQUEST,
            format!("a description can be at most {DESCRIPTION_MAX_CHARS} characters"),
        ));
    }
    let (name, description): (String, Option<String>) = sqlx::query_as(
        "UPDATE projects SET name = COALESCE($2, name), \
         description = CASE WHEN $3::text IS NULL THEN description ELSE NULLIF($3, '') END, updated_at = now() \
         WHERE id = $1 RETURNING name, description",
    )
    .bind(project_id)
    .bind(name)
    .bind(description)
    .fetch_one(&state.pool)
    .await
    .map_err(internal("failed to rename the project"))?;
    Ok(Json(
        json!({ "id": project_id, "name": name, "description": description }),
    ))
}

#[derive(Deserialize, Default)]
pub struct CopyRequest {
    /// The copy's name; the original's when left out.
    pub name: Option<String>,
    /// Where the copy goes; for a duplicate, the original's organization when the person is in
    /// it, else the one they picked for projects.
    pub org_id: Option<Uuid>,
}

/// `POST /api/projects/:id/duplicate` → 201 `{ session_id, project_id }`, for anyone who can
/// open the project.
pub async fn duplicate(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(project_id): Path<Uuid>,
    req: Option<Json<CopyRequest>>,
) -> Result<(StatusCode, Json<CreateProjectSessionResponse>), ApiError> {
    project_role(&state.pool, project_id, claims.sub).await?;
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let source_org: Uuid = sqlx::query_scalar("SELECT org_id FROM projects WHERE id = $1")
        .bind(project_id)
        .fetch_one(&state.pool)
        .await
        .map_err(internal("failed to duplicate the project"))?;
    let preferred = match req.org_id {
        Some(org) => Some(org),
        None if is_active_member(&state.pool, claims.sub, source_org)
            .await
            .map_err(internal("failed to duplicate the project"))? =>
        {
            Some(source_org)
        }
        None => None,
    };
    let org_id = destination(&state, claims.sub, preferred).await?;
    let copy = copy_project(
        &state,
        project_id,
        claims.sub,
        claims.active_org_id,
        org_id,
        req.name,
    )
    .await?;
    Ok((StatusCode::CREATED, Json(copy)))
}

/// `POST /api/showcase/:id/remix` → 201 `{ session_id, project_id }`: a copy of an app on show,
/// for anyone signed in, into the organization they name or picked for projects.
pub async fn remix(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(project_id): Path<Uuid>,
    req: Option<Json<CopyRequest>>,
) -> Result<(StatusCode, Json<CreateProjectSessionResponse>), ApiError> {
    let shown: bool = sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM projects WHERE id = $1 AND showcase AND shipyard_app IS NOT NULL)")
        .bind(project_id)
        .fetch_one(&state.pool)
        .await
        .map_err(internal("failed to remix the app"))?;
    if !shown {
        return Err((StatusCode::NOT_FOUND, "that app isn't on show".to_string()));
    }
    let req = req.map(|Json(r)| r).unwrap_or_default();
    let org_id = destination(&state, claims.sub, req.org_id).await?;
    let copy = copy_project(
        &state,
        project_id,
        claims.sub,
        claims.active_org_id,
        org_id,
        req.name,
    )
    .await?;
    Ok((StatusCode::CREATED, Json(copy)))
}

/// The organization a copy goes to: `preferred` when the person is in it, else the one they
/// picked for projects (409 until they have).
async fn destination(
    state: &AppState,
    user_id: Uuid,
    preferred: Option<Uuid>,
) -> Result<Uuid, ApiError> {
    if let Some(org) = preferred {
        if !is_active_member(&state.pool, user_id, org)
            .await
            .map_err(internal("failed to copy the project"))?
        {
            return Err((StatusCode::NOT_FOUND, "organization not found".to_string()));
        }
        return Ok(org);
    }
    project_org(&state.pool, user_id)
        .await
        .map_err(internal("failed to copy the project"))?
        .ok_or((
            StatusCode::CONFLICT,
            "choose an organization for projects first".to_string(),
        ))
}

async fn copy_project(
    state: &AppState,
    source: Uuid,
    user_id: Uuid,
    active_org_id: Uuid,
    org_id: Uuid,
    name: Option<String>,
) -> Result<CreateProjectSessionResponse, ApiError> {
    let source_name: String = sqlx::query_scalar("SELECT name FROM projects WHERE id = $1")
        .bind(source)
        .fetch_one(&state.pool)
        .await
        .map_err(internal("failed to copy the project"))?;
    let name = name
        .map(|n| n.trim().to_string())
        .filter(|n| !n.is_empty())
        .unwrap_or(source_name);
    if name.chars().count() > NAME_MAX_CHARS {
        return Err((
            StatusCode::BAD_REQUEST,
            format!("a name can be at most {NAME_MAX_CHARS} characters"),
        ));
    }
    let created = start_project(state, user_id, active_org_id, org_id, &name)
        .await
        .map_err(|(code, why)| (code, why.to_string()))?;
    sqlx::query(
        "UPDATE projects p SET description = s.description, plan = s.plan, stack = s.stack, status = s.status \
         FROM projects s WHERE p.id = $1 AND s.id = $2",
    )
    .bind(created.project_id)
    .bind(source)
    .execute(&state.pool)
    .await
    .map_err(internal("failed to copy the project"))?;

    let paths: Vec<String> =
        sqlx::query_scalar("SELECT path FROM project_files WHERE project_id = $1 ORDER BY path")
            .bind(source)
            .fetch_all(&state.pool)
            .await
            .map_err(internal("failed to copy the project's files"))?;
    if !paths.is_empty() {
        let mut conn = state
            .pool
            .acquire()
            .await
            .map_err(internal("failed to copy the project's files"))?;
        for path in &paths {
            let Some(bytes) = state
                .project_storage
                .get_bytes(&project_file_key(source, path))
                .await
                .map_err(internal("failed to copy the project's files"))?
            else {
                continue;
            };
            save_bytes(
                &mut conn,
                &state.project_storage,
                created.project_id,
                path,
                &bytes,
            )
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
        }
        bump_files_version_by(&mut conn, created.project_id, user_id)
            .await
            .map_err(internal("failed to copy the project's files"))?;
    }
    Ok(created)
}
