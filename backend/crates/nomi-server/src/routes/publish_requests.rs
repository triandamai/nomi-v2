//! Asking to publish: someone who works on a project but isn't one of its admins asks; the admins
//! see the queue and approve (which publishes, as the admin who approved) or decline, and the
//! person who asked can take it back. Every change goes out on the project's stream and as a
//! notification (the admins hear of a request, the person who asked of the answer).

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use nomi_auth::extractor::AuthClaims;

use super::deploy::{begin_publish, internal, project_role, valid_env, ApiError, Started};
use crate::app::AppState;
use crate::notifications::{self, Kind, Notice};

/// What the project's open pages hear when the queue changes (`project_events::changed`).
const REQUESTS_CHANGED: &str = "publish_request.changed";
/// The environment a request is for unless it names another.
const DEFAULT_ENV: &str = "production";
/// The longest note a request carries, in characters.
const MAX_NOTE_CHARS: usize = 500;

#[derive(Serialize, sqlx::FromRow)]
pub struct PublishRequestRow {
    pub id: Uuid,
    pub environment: String,
    pub note: Option<String>,
    pub status: String,
    pub requested_by: Uuid,
    pub requested_by_name: String,
    pub decided_by_name: Option<String>,
    pub decided_at: Option<DateTime<Utc>>,
    pub publish_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

/// A person's name as the project shows it, from the profile, web and Google rows joined as
/// `{alias}p`, `{alias}w` and `{alias}g`: their display name, else their address before the @.
fn name_of(alias: &str) -> String {
    format!("COALESCE(NULLIF({alias}p.display_name, ''), split_part(COALESCE({alias}w.email, {alias}g.email, ''), '@', 1))")
}

fn select_requests(filter: &str) -> String {
    format!(
        "SELECT r.id, r.environment, r.note, r.status, r.requested_by, {asker} AS requested_by_name, \
                CASE WHEN r.decided_by IS NULL THEN NULL ELSE {decider} END AS decided_by_name, \
                r.decided_at, r.publish_id, r.created_at \
         FROM publish_requests r \
         LEFT JOIN user_profiles ap ON ap.user_id = r.requested_by \
         LEFT JOIN web_credentials aw ON aw.user_id = r.requested_by \
         LEFT JOIN google_identities ag ON ag.user_id = r.requested_by \
         LEFT JOIN user_profiles dp ON dp.user_id = r.decided_by \
         LEFT JOIN web_credentials dw ON dw.user_id = r.decided_by \
         LEFT JOIN google_identities dg ON dg.user_id = r.decided_by \
         WHERE {filter}",
        asker = name_of("a"),
        decider = name_of("d"),
    )
}

async fn one(pool: &sqlx::PgPool, project_id: Uuid, request_id: Uuid) -> Result<PublishRequestRow, ApiError> {
    sqlx::query_as(&select_requests("r.id = $1 AND r.project_id = $2"))
        .bind(request_id)
        .bind(project_id)
        .fetch_optional(pool)
        .await
        .map_err(internal("failed to load the request"))?
        .ok_or((StatusCode::NOT_FOUND, "request not found".to_string()))
}

async fn project_name(pool: &sqlx::PgPool, project_id: Uuid) -> String {
    sqlx::query_scalar("SELECT name FROM projects WHERE id = $1").bind(project_id).fetch_one(pool).await.unwrap_or_default()
}

#[derive(Deserialize, Default)]
pub struct AskRequest {
    pub note: Option<String>,
    pub environment: Option<String>,
}

/// `POST /api/projects/:id/deploy/requests`, optionally `{ note, environment }`: asks the admins
/// to publish. Admins publish themselves (409); one open request per person (409).
pub async fn ask(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(project_id): Path<Uuid>,
    body: Option<Json<AskRequest>>,
) -> Result<(StatusCode, Json<PublishRequestRow>), ApiError> {
    if project_role(&state.pool, project_id, claims.sub).await? == "admin" {
        return Err((StatusCode::CONFLICT, "admins publish the project themselves".to_string()));
    }
    let req = body.map(|Json(b)| b).unwrap_or_default();
    let environment = req.environment.unwrap_or_else(|| DEFAULT_ENV.to_string());
    valid_env(&environment)?;
    let note = req.note.map(|n| n.trim().to_string()).filter(|n| !n.is_empty());
    if note.as_ref().is_some_and(|n| n.chars().count() > MAX_NOTE_CHARS) {
        return Err((StatusCode::BAD_REQUEST, "note is too long".to_string()));
    }
    let id: Option<Uuid> = sqlx::query_scalar(
        "INSERT INTO publish_requests (project_id, requested_by, environment, note) VALUES ($1, $2, $3, $4) \
         ON CONFLICT (project_id, requested_by) WHERE status = 'pending' DO NOTHING RETURNING id",
    )
    .bind(project_id)
    .bind(claims.sub)
    .bind(&environment)
    .bind(&note)
    .fetch_optional(&state.pool)
    .await
    .map_err(internal("failed to ask"))?;
    let Some(id) = id else {
        return Err((StatusCode::CONFLICT, "you've already asked; it's waiting for an admin".to_string()));
    };
    let row = one(&state.pool, project_id, id).await?;

    let locale = crate::shipyard::person_locale(&state.pool, claims.sub).await;
    let detail = match &row.note {
        Some(note) => locale.tf("deploy.request_asked_note", &[("who", &row.requested_by_name), ("note", note)]),
        None => locale.tf("deploy.request_asked", &[("who", &row.requested_by_name)]),
    };
    crate::shipyard::notify_admins(&state.pool, project_id, "publish_requested", &detail).await;
    crate::project_events::changed(project_id, REQUESTS_CHANGED);
    Ok((StatusCode::CREATED, Json(row)))
}

#[derive(Serialize)]
pub struct RequestList {
    pub requests: Vec<PublishRequestRow>,
}

/// `GET /api/projects/:id/deploy/requests`: the project's requests, open ones first, then the
/// rest newest first. Everyone who can open the project sees them.
pub async fn list(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(project_id): Path<Uuid>,
) -> Result<Json<RequestList>, ApiError> {
    project_role(&state.pool, project_id, claims.sub).await?;
    let requests: Vec<PublishRequestRow> = sqlx::query_as(&format!("{} ORDER BY (r.status = 'pending') DESC, r.created_at DESC", select_requests("r.project_id = $1")))
        .bind(project_id)
        .fetch_all(&state.pool)
        .await
        .map_err(internal("failed to list requests"))?;
    Ok(Json(RequestList { requests }))
}

#[derive(Deserialize, Clone, Copy, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    Approve,
    Decline,
    Cancel,
}

#[derive(Deserialize)]
pub struct ResolveRequest {
    pub action: Action,
}

/// `POST /api/projects/:id/deploy/requests/:request_id` with `{ action }`: an admin approves
/// (publishing as themselves; the request stays open if the publish can't start) or declines; the
/// person who asked cancels. Only an open request changes (409 otherwise).
pub async fn resolve(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path((project_id, request_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<ResolveRequest>,
) -> Result<Json<PublishRequestRow>, ApiError> {
    let role = project_role(&state.pool, project_id, claims.sub).await?;
    let request = one(&state.pool, project_id, request_id).await?;
    let allowed = match req.action {
        Action::Approve | Action::Decline => role == "admin",
        Action::Cancel => request.requested_by == claims.sub,
    };
    if !allowed {
        return Err((StatusCode::FORBIDDEN, "only the project's admins can do that".to_string()));
    }
    if request.status != "pending" {
        return Err((StatusCode::CONFLICT, "this request was already answered".to_string()));
    }

    let publish_id = if req.action == Action::Approve {
        match begin_publish(&state, project_id, claims.sub, &request.environment, None).await? {
            Started::Publishing(row) => Some(row.id),
            // The app's backend is set up first; the publish follows by itself.
            Started::Preparing(_) => None,
        }
    } else {
        None
    };
    let status = match req.action {
        Action::Approve => "approved",
        Action::Decline => "declined",
        Action::Cancel => "cancelled",
    };
    let changed = sqlx::query(
        "UPDATE publish_requests SET status = $3, decided_by = $4, decided_at = now(), publish_id = $5 \
         WHERE id = $1 AND project_id = $2 AND status = 'pending'",
    )
    .bind(request_id)
    .bind(project_id)
    .bind(status)
    .bind(claims.sub)
    .bind(publish_id)
    .execute(&state.pool)
    .await
    .map_err(internal("failed to answer the request"))?;
    if changed.rows_affected() == 0 {
        return Err((StatusCode::CONFLICT, "this request was already answered".to_string()));
    }
    let row = one(&state.pool, project_id, request_id).await?;

    if req.action != Action::Cancel {
        let locale = crate::shipyard::person_locale(&state.pool, request.requested_by).await;
        let name = project_name(&state.pool, project_id).await;
        let who = row.decided_by_name.clone().unwrap_or_default();
        let (title, body) = if req.action == Action::Approve {
            ("deploy.request_approved_title", "deploy.request_approved_body")
        } else {
            ("deploy.request_declined_title", "deploy.request_declined_body")
        };
        let notice = Notice {
            kind: Kind::Deploy,
            title: locale.tf(title, &[("name", &name)]),
            body: locale.tf(body, &[("who", &who)]),
            link: Some(format!("/projects/{project_id}/app?sheet=deploy")),
        };
        if let Err(e) = notifications::send(&state.pool, request.requested_by, notice).await {
            tracing::warn!(error = %e, "couldn't tell the person who asked to publish");
        }
    }
    crate::project_events::changed(project_id, REQUESTS_CHANGED);
    Ok(Json(row))
}
