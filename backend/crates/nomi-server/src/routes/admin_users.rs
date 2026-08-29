use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::app::AppState;
use nomi_auth::claims::Claims;
use nomi_auth::extractor::AuthClaims;
use nomi_auth::grants::{grant_permission, list_grants_for_user, revoke_permission, GrantError, NewGrant};

fn require_user_permission(claims: &Claims, action: &str) -> Result<(), (StatusCode, &'static str)> {
    if claims.has_permission("admin", "user", action) {
        Ok(())
    } else {
        Err((StatusCode::FORBIDDEN, "not authorized to manage users"))
    }
}

fn grant_error_to_response(e: GrantError) -> (StatusCode, String) {
    match e {
        GrantError::InvalidResource | GrantError::InvalidActions | GrantError::OrgRequiredForOrgScope | GrantError::OrgNotAllowedForAdminScope => {
            (StatusCode::BAD_REQUEST, e.to_string())
        }
        GrantError::Db(err) => {
            tracing::error!(error = %err, "grant operation failed");
            (StatusCode::INTERNAL_SERVER_ERROR, "failed to save permission grant".to_string())
        }
    }
}

#[derive(Serialize)]
pub struct UserSummary {
    pub id: Uuid,
    pub email: String,
    pub is_platform_admin: bool,
    pub is_staff: bool,
    pub org_count: i64,
}

#[derive(Serialize)]
pub struct UserListResponse {
    pub users: Vec<UserSummary>,
    pub total: i64,
}

#[derive(Deserialize)]
pub struct UserListQuery {
    pub query: Option<String>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

#[tracing::instrument(skip(state, claims, q))]
pub async fn list_users(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Query(q): Query<UserListQuery>,
) -> Result<Json<UserListResponse>, (StatusCode, &'static str)> {
    require_user_permission(&claims, "view")?;

    let page = q.page.unwrap_or(1).max(1);
    let page_size = q.page_size.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * page_size;
    let search = q.query.filter(|s| !s.trim().is_empty());

    let rows: Vec<(Uuid, String, bool, bool, i64)> = sqlx::query_as(
        "SELECT u.id, wc.email, u.is_platform_admin, \
                EXISTS(SELECT 1 FROM user_permissions up WHERE up.user_id = u.id AND up.scope_type = 'admin') AS has_admin_grant, \
                (SELECT COUNT(*) FROM memberships m WHERE m.user_id = u.id AND m.status = 'active') AS org_count \
         FROM users u \
         JOIN web_credentials wc ON wc.user_id = u.id \
         WHERE $1::text IS NULL OR wc.email ILIKE '%' || $1 || '%' \
         ORDER BY wc.email ASC \
         LIMIT $2 OFFSET $3",
    )
    .bind(&search)
    .bind(page_size)
    .bind(offset)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "failed to list users");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to list users")
    })?;

    let total: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM users u JOIN web_credentials wc ON wc.user_id = u.id WHERE $1::text IS NULL OR wc.email ILIKE '%' || $1 || '%'",
    )
    .bind(&search)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "failed to count users");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to count users")
    })?;

    let users = rows
        .into_iter()
        .map(|(id, email, is_platform_admin, has_admin_grant, org_count)| UserSummary {
            id,
            email,
            is_platform_admin,
            is_staff: is_platform_admin || has_admin_grant,
            org_count,
        })
        .collect();

    Ok(Json(UserListResponse { users, total }))
}

#[derive(Serialize, sqlx::FromRow)]
pub struct MembershipRow {
    pub org_id: Uuid,
    pub org_name: String,
    pub role: String,
}

#[derive(Serialize)]
pub struct UserDetailResponse {
    pub id: Uuid,
    pub email: String,
    pub is_platform_admin: bool,
    pub permissions: Vec<nomi_auth::grants::PermissionGrant>,
    pub memberships: Vec<MembershipRow>,
}

async fn load_memberships(pool: &sqlx::PgPool, user_id: Uuid) -> Result<Vec<MembershipRow>, sqlx::Error> {
    sqlx::query_as(
        "SELECT m.org_id, o.name AS org_name, m.role \
         FROM memberships m JOIN organizations o ON o.id = m.org_id \
         WHERE m.user_id = $1 AND m.status = 'active' \
         ORDER BY o.name ASC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}

#[tracing::instrument(skip(state, claims))]
pub async fn get_user_detail(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(user_id): Path<Uuid>,
) -> Result<Json<UserDetailResponse>, (StatusCode, &'static str)> {
    require_user_permission(&claims, "view")?;

    let row: Option<(String, bool)> =
        sqlx::query_as("SELECT wc.email, u.is_platform_admin FROM users u JOIN web_credentials wc ON wc.user_id = u.id WHERE u.id = $1")
            .bind(user_id)
            .fetch_optional(&state.pool)
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "failed to load user");
                (StatusCode::INTERNAL_SERVER_ERROR, "failed to load user")
            })?;
    let (email, is_platform_admin) = row.ok_or((StatusCode::NOT_FOUND, "user not found"))?;

    let permissions = list_grants_for_user(&state.pool, user_id).await.map_err(|e| {
        tracing::error!(error = %e, "failed to load permissions");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to load permissions")
    })?;
    let memberships = load_memberships(&state.pool, user_id).await.map_err(|e| {
        tracing::error!(error = %e, "failed to load memberships");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to load memberships")
    })?;

    Ok(Json(UserDetailResponse { id: user_id, email, is_platform_admin, permissions, memberships }))
}

#[derive(Deserialize)]
pub struct GrantPermissionRequest {
    pub scope_type: String,
    pub org_id: Option<Uuid>,
    pub resource: String,
    pub actions: Vec<String>,
}

#[tracing::instrument(skip(state, claims, req))]
pub async fn grant_user_permission(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(user_id): Path<Uuid>,
    Json(req): Json<GrantPermissionRequest>,
) -> Result<Json<Vec<nomi_auth::grants::PermissionGrant>>, (StatusCode, String)> {
    require_user_permission(&claims, "manage").map_err(|(status, msg)| (status, msg.to_string()))?;

    if req.scope_type != "admin" && req.scope_type != "org" {
        return Err((StatusCode::BAD_REQUEST, "scope_type must be 'admin' or 'org'".to_string()));
    }

    let grants = grant_permission(
        &state.pool,
        NewGrant {
            user_id,
            scope_type: &req.scope_type,
            org_id: req.org_id,
            resource: &req.resource,
            actions: &req.actions,
            granted_by: claims.sub,
        },
    )
    .await
    .map_err(grant_error_to_response)?;

    Ok(Json(grants))
}

#[tracing::instrument(skip(state, claims))]
pub async fn revoke_user_permission(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path((user_id, permission_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<Vec<nomi_auth::grants::PermissionGrant>>, (StatusCode, &'static str)> {
    require_user_permission(&claims, "manage")?;

    let grants = revoke_permission(&state.pool, user_id, permission_id).await.map_err(|e| {
        tracing::error!(error = %e, "failed to revoke permission");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to revoke permission")
    })?;

    Ok(Json(grants))
}

#[derive(Deserialize)]
pub struct AssignOrgRequest {
    pub org_id: Uuid,
    pub role: String,
}

const ALLOWED_ROLES: [&str; 3] = ["owner", "admin", "member"];

#[tracing::instrument(skip(state, claims, req))]
pub async fn assign_user_to_org(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(user_id): Path<Uuid>,
    Json(req): Json<AssignOrgRequest>,
) -> Result<Json<Vec<MembershipRow>>, (StatusCode, &'static str)> {
    require_user_permission(&claims, "manage")?;

    if !ALLOWED_ROLES.contains(&req.role.as_str()) {
        return Err((StatusCode::BAD_REQUEST, "role must be owner, admin, or member"));
    }

    sqlx::query(
        "INSERT INTO memberships (org_id, user_id, role, status) VALUES ($1, $2, $3, 'active') \
         ON CONFLICT (org_id, user_id) DO UPDATE SET role = EXCLUDED.role, status = 'active'",
    )
    .bind(req.org_id)
    .bind(user_id)
    .bind(&req.role)
    .execute(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "failed to assign user to org");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to assign user to org")
    })?;

    let memberships = load_memberships(&state.pool, user_id).await.map_err(|e| {
        tracing::error!(error = %e, "failed to reload memberships");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to reload memberships")
    })?;
    Ok(Json(memberships))
}

#[tracing::instrument(skip(state, claims))]
pub async fn remove_user_from_org(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path((user_id, org_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<Vec<MembershipRow>>, (StatusCode, &'static str)> {
    require_user_permission(&claims, "manage")?;

    sqlx::query("UPDATE memberships SET status = 'removed' WHERE org_id = $1 AND user_id = $2")
        .bind(org_id)
        .bind(user_id)
        .execute(&state.pool)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "failed to remove membership");
            (StatusCode::INTERNAL_SERVER_ERROR, "failed to remove membership")
        })?;

    let memberships = load_memberships(&state.pool, user_id).await.map_err(|e| {
        tracing::error!(error = %e, "failed to reload memberships");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to reload memberships")
    })?;
    Ok(Json(memberships))
}

#[derive(Serialize)]
pub struct OrgOption {
    pub id: Uuid,
    pub name: String,
}

#[tracing::instrument(skip(state, claims))]
pub async fn list_orgs(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
) -> Result<Json<Vec<OrgOption>>, (StatusCode, &'static str)> {
    require_user_permission(&claims, "view")?;

    let orgs = sqlx::query_as("SELECT id, name FROM organizations WHERE is_personal = false ORDER BY name ASC")
        .fetch_all(&state.pool)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "failed to list orgs");
            (StatusCode::INTERNAL_SERVER_ERROR, "failed to list orgs")
        })?
        .into_iter()
        .map(|(id, name)| OrgOption { id, name })
        .collect();

    Ok(Json(orgs))
}
