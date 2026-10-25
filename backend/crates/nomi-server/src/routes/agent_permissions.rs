//! What the crew may do on a project (`agent_permissions`), for the project's Settings sheet:
//! everyone who can open the project sees it; only its admins change it.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use nomi_agent_core::agent_permissions::{self, Scope};
use nomi_auth::extractor::AuthClaims;

use crate::app::AppState;
use crate::routes::deploy::{internal, project_role, require_admin, ApiError};

/// The permissions a person can set from the sheet, by agent.
const SETTABLE: &[(&str, &str)] = &[
    (nomi_agent_inspector::INSPECTOR_AGENT_TYPE, nomi_agent_inspector::READ_LOGS),
    (nomi_agent_inspector::INSPECTOR_AGENT_TYPE, nomi_agent_inspector::READ_DATA),
];

#[derive(Serialize, sqlx::FromRow)]
pub struct AgentPermissionRow {
    pub id: Uuid,
    pub key: String,
    pub agent_type: String,
    pub permission: String,
    pub scope: String,
    /// False when it covers every project in the organization.
    pub for_this_project: bool,
    pub decided_by: Option<String>,
    pub updated_at: DateTime<Utc>,
}

/// `GET /api/projects/:id/agent-permissions`: this project's and its organization's.
pub async fn list(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(project_id): Path<Uuid>,
) -> Result<Json<Vec<AgentPermissionRow>>, ApiError> {
    project_role(&state.pool, project_id, claims.sub).await?;
    let rows: Vec<AgentPermissionRow> = sqlx::query_as(
        "SELECT ap.id, ap.key, ap.agent_type, ap.permission, ap.scope, ap.project_id IS NOT NULL AS for_this_project, \
                NULLIF(up.display_name, '') AS decided_by, ap.updated_at \
         FROM agent_permissions ap JOIN projects p ON p.org_id = ap.org_id \
         LEFT JOIN user_profiles up ON up.user_id = ap.decided_by \
         WHERE p.id = $1 AND (ap.project_id = p.id OR ap.project_id IS NULL) \
         ORDER BY ap.agent_type, ap.permission, ap.project_id IS NULL",
    )
    .bind(project_id)
    .fetch_all(&state.pool)
    .await
    .map_err(internal("failed to list agent permissions"))?;
    Ok(Json(rows))
}

#[derive(Deserialize)]
pub struct SetPermission {
    pub agent_type: String,
    pub permission: String,
    pub scope: String,
}

/// `PUT /api/projects/:id/agent-permissions`: allows or denies one, for this project.
pub async fn set(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(project_id): Path<Uuid>,
    Json(req): Json<SetPermission>,
) -> Result<StatusCode, ApiError> {
    require_admin(&state.pool, project_id, claims.sub).await?;
    if !SETTABLE.contains(&(req.agent_type.as_str(), req.permission.as_str())) {
        return Err((StatusCode::BAD_REQUEST, "unknown agent permission".to_string()));
    }
    let scope = Scope::parse(&req.scope).ok_or((StatusCode::BAD_REQUEST, "scope is allow or denied".to_string()))?;
    let mut conn = state.pool.acquire().await.map_err(internal("failed to save the permission"))?;
    agent_permissions::set_for_project(&mut conn, &req.agent_type, project_id, &req.permission, scope, claims.sub)
        .await
        .map_err(internal("failed to save the permission"))?;
    Ok(StatusCode::NO_CONTENT)
}

/// `DELETE /api/projects/:id/agent-permissions/:permission_id`: back to asking.
pub async fn remove(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path((project_id, permission_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, ApiError> {
    require_admin(&state.pool, project_id, claims.sub).await?;
    let removed = sqlx::query(
        "DELETE FROM agent_permissions ap USING projects p \
         WHERE ap.id = $2 AND p.id = $1 AND ap.org_id = p.org_id AND (ap.project_id = p.id OR ap.project_id IS NULL)",
    )
    .bind(project_id)
    .bind(permission_id)
    .execute(&state.pool)
    .await
    .map_err(internal("failed to remove the permission"))?
    .rows_affected();
    if removed == 0 {
        return Err((StatusCode::NOT_FOUND, "permission not found".to_string()));
    }
    Ok(StatusCode::NO_CONTENT)
}
