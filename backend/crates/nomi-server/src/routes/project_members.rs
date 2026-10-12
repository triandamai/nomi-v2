//! Who can open a project. By default everyone active in its organization can; a project can
//! instead be limited to the people added to it. Added people are members (they build it) or
//! admins (they also publish, roll back, take it offline, see the database and manage members).

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use nomi_auth::extractor::AuthClaims;

use crate::app::AppState;
use crate::routes::deploy::{internal, project_role, require_admin, ApiError};

#[derive(Serialize, sqlx::FromRow)]
pub struct ProjectPerson {
    pub user_id: Uuid,
    pub name: String,
    pub email: Option<String>,
    pub avatar_url: Option<String>,
    /// Their role in the organization (owner, admin, member).
    pub org_role: String,
    /// Their role on the project when added to it.
    pub project_role: Option<String>,
    /// Whether they started the project.
    pub is_owner: bool,
    pub added_at: Option<DateTime<Utc>>,
}

#[derive(Serialize)]
pub struct ProjectMembers {
    /// `org`: everyone in the organization; `members`: only the people added.
    pub access: String,
    pub my_role: &'static str,
    pub me: Uuid,
    /// Everyone active in the organization, with their place on the project.
    pub people: Vec<ProjectPerson>,
}

async fn load(state: &AppState, project_id: Uuid, user_id: Uuid) -> Result<ProjectMembers, ApiError> {
    let my_role = project_role(&state.pool, project_id, user_id).await?;
    let access: String = sqlx::query_scalar("SELECT access FROM projects WHERE id = $1")
        .bind(project_id)
        .fetch_one(&state.pool)
        .await
        .map_err(internal("failed to load project"))?;
    let people: Vec<ProjectPerson> = sqlx::query_as(
        "SELECT m.user_id, \
                COALESCE(NULLIF(up.display_name, ''), split_part(COALESCE(w.email, g.email, ''), '@', 1)) AS name, \
                COALESCE(w.email, g.email) AS email, up.avatar_url, m.role AS org_role, \
                pm.role AS project_role, (p.user_id = m.user_id) AS is_owner, pm.created_at AS added_at \
         FROM projects p \
         JOIN memberships m ON m.org_id = p.org_id AND m.status = 'active' \
         LEFT JOIN project_members pm ON pm.project_id = p.id AND pm.user_id = m.user_id \
         LEFT JOIN user_profiles up ON up.user_id = m.user_id \
         LEFT JOIN web_credentials w ON w.user_id = m.user_id \
         LEFT JOIN google_identities g ON g.user_id = m.user_id \
         WHERE p.id = $1 \
         ORDER BY (p.user_id = m.user_id) DESC, (pm.role IS NULL), pm.role, name",
    )
    .bind(project_id)
    .fetch_all(&state.pool)
    .await
    .map_err(internal("failed to load project members"))?;
    Ok(ProjectMembers { access, my_role, me: user_id, people })
}

/// `GET /api/projects/:id/members`
pub async fn list(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(project_id): Path<Uuid>,
) -> Result<Json<ProjectMembers>, ApiError> {
    Ok(Json(load(&state, project_id, claims.sub).await?))
}

#[derive(Deserialize)]
pub struct AccessRequest {
    pub access: String,
}

/// `PUT /api/projects/:id/access`: open to the organization, or only to the people added. The
/// person limiting it is added as an admin so they don't lock themselves out.
pub async fn set_access(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(project_id): Path<Uuid>,
    Json(req): Json<AccessRequest>,
) -> Result<Json<ProjectMembers>, ApiError> {
    require_admin(&state.pool, project_id, claims.sub).await?;
    if req.access != "org" && req.access != "members" {
        return Err((StatusCode::BAD_REQUEST, "access is org or members".to_string()));
    }
    let mut tx = state.pool.begin().await.map_err(internal("failed to change access"))?;
    sqlx::query("UPDATE projects SET access = $2 WHERE id = $1")
        .bind(project_id)
        .bind(&req.access)
        .execute(&mut *tx)
        .await
        .map_err(internal("failed to change access"))?;
    if req.access == "members" {
        sqlx::query(
            "INSERT INTO project_members (project_id, user_id, role, added_by) VALUES ($1, $2, 'admin', $2) \
             ON CONFLICT (project_id, user_id) DO UPDATE SET role = 'admin'",
        )
        .bind(project_id)
        .bind(claims.sub)
        .execute(&mut *tx)
        .await
        .map_err(internal("failed to change access"))?;
    }
    tx.commit().await.map_err(internal("failed to change access"))?;
    Ok(Json(load(&state, project_id, claims.sub).await?))
}

#[derive(Deserialize)]
pub struct MemberRequest {
    /// admin or member.
    pub role: String,
}

/// `PUT /api/projects/:id/members/:user_id`: adds someone from the organization, or changes
/// their role.
pub async fn put_member(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path((project_id, user_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<MemberRequest>,
) -> Result<Json<ProjectMembers>, ApiError> {
    require_admin(&state.pool, project_id, claims.sub).await?;
    if req.role != "admin" && req.role != "member" {
        return Err((StatusCode::BAD_REQUEST, "role is admin or member".to_string()));
    }
    let in_org: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM projects p JOIN memberships m ON m.org_id = p.org_id \
         WHERE p.id = $1 AND m.user_id = $2 AND m.status = 'active')",
    )
    .bind(project_id)
    .bind(user_id)
    .fetch_one(&state.pool)
    .await
    .map_err(internal("failed to add member"))?;
    if !in_org {
        return Err((StatusCode::UNPROCESSABLE_ENTITY, "invite them to the organization first".to_string()));
    }
    sqlx::query(
        "INSERT INTO project_members (project_id, user_id, role, added_by) VALUES ($1, $2, $3, $4) \
         ON CONFLICT (project_id, user_id) DO UPDATE SET role = $3",
    )
    .bind(project_id)
    .bind(user_id)
    .bind(&req.role)
    .bind(claims.sub)
    .execute(&state.pool)
    .await
    .map_err(internal("failed to add member"))?;
    Ok(Json(load(&state, project_id, claims.sub).await?))
}

/// `DELETE /api/projects/:id/members/:user_id`. The person who started the project can't be
/// removed (they always keep it), and the last admin of a limited project stays.
pub async fn remove_member(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path((project_id, user_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ProjectMembers>, ApiError> {
    if user_id != claims.sub {
        require_admin(&state.pool, project_id, claims.sub).await?;
    } else {
        project_role(&state.pool, project_id, claims.sub).await?;
    }
    sqlx::query("DELETE FROM project_members WHERE project_id = $1 AND user_id = $2")
        .bind(project_id)
        .bind(user_id)
        .execute(&state.pool)
        .await
        .map_err(internal("failed to remove member"))?;
    if user_id == claims.sub && project_role(&state.pool, project_id, claims.sub).await.is_err() {
        // They left a project limited to its members: nothing more for them to see.
        return Err((StatusCode::NOT_FOUND, "project not found".to_string()));
    }
    Ok(Json(load(&state, project_id, claims.sub).await?))
}
