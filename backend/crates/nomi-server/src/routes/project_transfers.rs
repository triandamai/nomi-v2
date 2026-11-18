//! Moving a project into a workspace the person doesn't own waits for that workspace's owner:
//! `POST /api/projects/:id/transfer` (see `projects::transfer_project`) leaves a pending request
//! here, which the owner approves (the project moves) or declines. Moving within the person's own
//! workspaces stays immediate.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

use nomi_auth::extractor::AuthClaims;

use crate::app::AppState;

type ApiError = (StatusCode, String);

/// The most requests a list answers with.
const LIST_LIMIT: i64 = 50;
/// The role that owns a workspace, and so may take a project into it without asking.
const OWNER_ROLE: &str = "owner";

/// Whether a move needs the target's owner to approve, given the mover's role there (`None`
/// when they aren't a member): only a workspace's owner may put a project in without asking.
pub fn transfer_needs_approval(role_in_target: Option<&str>) -> bool {
    role_in_target != Some(OWNER_ROLE)
}

#[derive(Serialize, sqlx::FromRow)]
pub struct TransferRequest {
    pub id: Uuid,
    pub project_id: Uuid,
    pub project_name: String,
    pub from_org_id: Uuid,
    pub from_org_name: String,
    pub to_org_id: Uuid,
    pub to_org_name: String,
    pub requested_by: Uuid,
    pub requested_by_name: String,
    /// `pending`, `approved`, `declined` or `cancelled`.
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub decided_at: Option<DateTime<Utc>>,
}


const REQUEST_COLUMNS: &str = "r.id, r.project_id, p.name AS project_name, r.from_org_id, fo.name AS from_org_name, r.to_org_id, tn.name AS to_org_name, \
    r.requested_by, COALESCE(NULLIF(up.display_name, ''), split_part(COALESCE(w.email, g.email, ''), '@', 1)) AS requested_by_name, \
    r.status, r.created_at, r.decided_at \
    FROM project_transfer_requests r \
    JOIN projects p ON p.id = r.project_id \
    JOIN organizations fo ON fo.id = r.from_org_id \
    JOIN organizations tn ON tn.id = r.to_org_id \
    LEFT JOIN user_profiles up ON up.user_id = r.requested_by \
    LEFT JOIN web_credentials w ON w.user_id = r.requested_by \
    LEFT JOIN google_identities g ON g.user_id = r.requested_by";

fn internal(e: sqlx::Error) -> ApiError {
    tracing::error!(error = %e, "project transfer request failed");
    (StatusCode::INTERNAL_SERVER_ERROR, "failed to handle the transfer request".to_string())
}

async fn load(pool: &sqlx::PgPool, id: Uuid) -> Result<TransferRequest, ApiError> {
    sqlx::query_as(&format!("SELECT {REQUEST_COLUMNS} WHERE r.id = $1"))
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(internal)?
        .ok_or((StatusCode::NOT_FOUND, "request not found".to_string()))
}

/// Records that `requester` wants `project_id` in `to_org`, for that workspace's owners to decide,
/// and tells them. One request is open per project at a time (409).
pub(crate) async fn create_request(
    pool: &sqlx::PgPool,
    project_id: Uuid,
    from_org: Uuid,
    to_org: Uuid,
    requester: Uuid,
) -> Result<TransferRequest, ApiError> {
    let id: Option<Uuid> = sqlx::query_scalar(
        "INSERT INTO project_transfer_requests (project_id, from_org_id, to_org_id, requested_by) VALUES ($1, $2, $3, $4) \
         ON CONFLICT (project_id) WHERE status = 'pending' DO NOTHING RETURNING id",
    )
    .bind(project_id)
    .bind(from_org)
    .bind(to_org)
    .bind(requester)
    .fetch_optional(pool)
    .await
    .map_err(internal)?;
    let id = id.ok_or((StatusCode::CONFLICT, "this project already has a move waiting on an owner".to_string()))?;
    let request = load(pool, id).await?;
    let owners: Vec<Uuid> = sqlx::query_scalar("SELECT user_id FROM memberships WHERE org_id = $1 AND status = 'active' AND role = $2 AND user_id <> $3")
        .bind(to_org)
        .bind(OWNER_ROLE)
        .bind(requester)
        .fetch_all(pool)
        .await
        .unwrap_or_default();
    for owner in owners {
        notify(pool, owner, "project.transfer_requested_title", "project.transfer_requested_body", &request, "/projects").await;
    }
    Ok(request)
}

async fn notify(pool: &sqlx::PgPool, user_id: Uuid, title_key: &str, body_key: &str, request: &TransferRequest, link: &str) {
    let code: Option<String> = sqlx::query_scalar("SELECT language FROM user_preferences WHERE user_id = $1").bind(user_id).fetch_optional(pool).await.ok().flatten();
    let locale = nomi_i18n::Locale::from_code_or_default(code.as_deref());
    let args = [("name", request.project_name.as_str()), ("who", request.requested_by_name.as_str()), ("from", request.from_org_name.as_str()), ("org", request.to_org_name.as_str())];
    let notice = crate::notifications::Notice {
        kind: crate::notifications::Kind::Project,
        title: locale.tf(title_key, &args),
        body: locale.tf(body_key, &args),
        link: Some(link.to_string()),
    };
    if let Err(e) = crate::notifications::send(pool, user_id, notice).await {
        tracing::warn!(error = %e, "couldn't send a project move notification");
    }
}

async fn is_owner(pool: &sqlx::PgPool, user_id: Uuid, org_id: Uuid) -> Result<bool, ApiError> {
    sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM memberships WHERE org_id = $1 AND user_id = $2 AND status = 'active' AND role = $3)")
        .bind(org_id)
        .bind(user_id)
        .bind(OWNER_ROLE)
        .fetch_one(pool)
        .await
        .map_err(internal)
}

#[derive(Serialize)]
pub struct MyTransferRequests {
    /// Waiting on the person, as the owner of the workspace the project would move into.
    pub incoming: Vec<TransferRequest>,
    /// Ones they asked for, newest first.
    pub outgoing: Vec<TransferRequest>,
}

/// `GET /api/transfer-requests`: moves waiting on the person, and the ones they asked for.
pub async fn mine(State(state): State<AppState>, AuthClaims(claims): AuthClaims) -> Result<Json<MyTransferRequests>, ApiError> {
    let incoming = sqlx::query_as(&format!(
        "SELECT {REQUEST_COLUMNS} WHERE r.status = 'pending' AND r.to_org_id IN \
         (SELECT org_id FROM memberships WHERE user_id = $1 AND status = 'active' AND role = $2) ORDER BY r.created_at DESC LIMIT $3"
    ))
    .bind(claims.sub)
    .bind(OWNER_ROLE)
    .bind(LIST_LIMIT)
    .fetch_all(&state.pool)
    .await
    .map_err(internal)?;
    let outgoing = sqlx::query_as(&format!("SELECT {REQUEST_COLUMNS} WHERE r.requested_by = $1 ORDER BY r.created_at DESC LIMIT $2"))
        .bind(claims.sub)
        .bind(LIST_LIMIT)
        .fetch_all(&state.pool)
        .await
        .map_err(internal)?;
    Ok(Json(MyTransferRequests { incoming, outgoing }))
}

/// `GET /api/projects/:id/transfer/requests`: a project's moves, newest first ("Waiting on the owner").
pub async fn for_project(State(state): State<AppState>, AuthClaims(claims): AuthClaims, Path(project_id): Path<Uuid>) -> Result<Json<Vec<TransferRequest>>, ApiError> {
    let can_open: bool = sqlx::query_scalar("SELECT can_open_project($1, $2)").bind(project_id).bind(claims.sub).fetch_one(&state.pool).await.map_err(internal)?;
    if !can_open {
        return Err((StatusCode::NOT_FOUND, "project not found".to_string()));
    }
    let requests = sqlx::query_as(&format!("SELECT {REQUEST_COLUMNS} WHERE r.project_id = $1 ORDER BY r.created_at DESC LIMIT $2"))
        .bind(project_id)
        .bind(LIST_LIMIT)
        .fetch_all(&state.pool)
        .await
        .map_err(internal)?;
    Ok(Json(requests))
}

/// Settles a pending request, once: `None` when it was already settled (or isn't there).
async fn settle(pool: &sqlx::PgPool, id: Uuid, status: &str, decider: Uuid) -> Result<Option<()>, ApiError> {
    let done = sqlx::query("UPDATE project_transfer_requests SET status = $2, decided_by = $3, decided_at = now() WHERE id = $1 AND status = 'pending'")
        .bind(id)
        .bind(status)
        .bind(decider)
        .execute(pool)
        .await
        .map_err(internal)?;
    Ok((done.rows_affected() > 0).then_some(()))
}

/// `DELETE /api/projects/:id/transfer/requests/:request_id`: takes a waiting move back, for the one
/// who asked or the project's owner and admins.
pub async fn cancel(State(state): State<AppState>, AuthClaims(claims): AuthClaims, Path((project_id, request_id)): Path<(Uuid, Uuid)>) -> Result<StatusCode, ApiError> {
    let request = load(&state.pool, request_id).await?;
    if request.project_id != project_id {
        return Err((StatusCode::NOT_FOUND, "request not found".to_string()));
    }
    if request.requested_by != claims.sub {
        super::deploy::require_admin(&state.pool, project_id, claims.sub).await?;
    }
    match settle(&state.pool, request_id, "cancelled", claims.sub).await? {
        Some(()) => Ok(StatusCode::NO_CONTENT),
        None => Err((StatusCode::CONFLICT, "the request was already decided".to_string())),
    }
}

/// `POST /api/transfer-requests/:id/decline`: the receiving workspace's owner says no.
pub async fn decline(State(state): State<AppState>, AuthClaims(claims): AuthClaims, Path(request_id): Path<Uuid>) -> Result<Json<TransferRequest>, ApiError> {
    let request = load(&state.pool, request_id).await?;
    if !is_owner(&state.pool, claims.sub, request.to_org_id).await? {
        return Err((StatusCode::NOT_FOUND, "request not found".to_string()));
    }
    if settle(&state.pool, request_id, "declined", claims.sub).await?.is_none() {
        return Err((StatusCode::CONFLICT, "the request was already decided".to_string()));
    }
    let request = load(&state.pool, request_id).await?;
    notify(&state.pool, request.requested_by, "project.transfer_declined_title", "project.transfer_declined_body", &request, &format!("/projects/{}/app", request.project_id)).await;
    Ok(Json(request))
}

/// `POST /api/transfer-requests/:id/approve`: the receiving workspace's owner takes the project;
/// it moves now, as if its owner had moved it, and becomes theirs.
pub async fn approve(State(state): State<AppState>, AuthClaims(claims): AuthClaims, Path(request_id): Path<Uuid>) -> Result<Json<TransferRequest>, ApiError> {
    let request = load(&state.pool, request_id).await?;
    if !is_owner(&state.pool, claims.sub, request.to_org_id).await? {
        return Err((StatusCode::NOT_FOUND, "request not found".to_string()));
    }
    // Claimed first, so a second tap (or the other owner) can't move it twice.
    if settle(&state.pool, request_id, "approved", claims.sub).await?.is_none() {
        return Err((StatusCode::CONFLICT, "the request was already decided".to_string()));
    }
    let reopen = |status: &'static str| {
        let pool = state.pool.clone();
        async move {
            let _ = sqlx::query("UPDATE project_transfer_requests SET status = $2, decided_by = NULL, decided_at = NULL WHERE id = $1").bind(request_id).bind(status).execute(&pool).await;
        }
    };
    let project: Option<(Uuid, Uuid)> = sqlx::query_as("SELECT org_id, session_id FROM projects WHERE id = $1").bind(request.project_id).fetch_optional(&state.pool).await.map_err(internal)?;
    let Some((current_org, session_id)) = project.filter(|(org, _)| *org == request.from_org_id) else {
        // Deleted, or moved by someone else since: there's nothing left to approve.
        let _ = sqlx::query("UPDATE project_transfer_requests SET status = 'cancelled' WHERE id = $1").bind(request_id).execute(&state.pool).await;
        return Err((StatusCode::CONFLICT, "the project is no longer where it was when this was asked".to_string()));
    };
    if let Err(e) = super::projects::move_project(&state, request.project_id, session_id, &request.project_name, current_org, request.to_org_id, claims.sub, request.requested_by).await {
        reopen("pending").await;
        return Err(e);
    }
    Ok(Json(load(&state.pool, request_id).await?))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_a_workspaces_owner_moves_into_it_without_asking() {
        assert!(!transfer_needs_approval(Some("owner")));
        // Admins and members of the target, and people outside it, ask its owner.
        assert!(transfer_needs_approval(Some("admin")));
        assert!(transfer_needs_approval(Some("member")));
        assert!(transfer_needs_approval(None));
    }
}
