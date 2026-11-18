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
    /// The files changed since the live deploy, as they were when this was asked.
    pub files: Vec<String>,
    #[serde(skip)]
    ask_job_id: Option<Uuid>,
    #[serde(skip)]
    ask_text: Option<String>,
    #[serde(skip)]
    ask_at: Option<DateTime<Utc>>,
    /// The ask (turn job) this follows; null when it follows none.
    #[sqlx(skip)]
    pub ask: Option<RequestAsk>,
}

#[derive(Serialize)]
pub struct RequestAsk {
    pub job_id: Uuid,
    pub text: String,
    pub asked_at: DateTime<Utc>,
}

impl PublishRequestRow {
    /// Fills `ask` from the joined turn job.
    fn with_ask(mut self) -> Self {
        self.ask = match (self.ask_job_id, self.ask_text.clone(), self.ask_at) {
            (Some(job_id), Some(text), Some(asked_at)) => Some(RequestAsk { job_id, text, asked_at }),
            _ => None,
        };
        self
    }
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
                r.decided_at, r.publish_id, r.created_at, r.files, r.ask_job_id, tj.text AS ask_text, tj.created_at AS ask_at \
         FROM publish_requests r \
         LEFT JOIN user_profiles ap ON ap.user_id = r.requested_by \
         LEFT JOIN web_credentials aw ON aw.user_id = r.requested_by \
         LEFT JOIN google_identities ag ON ag.user_id = r.requested_by \
         LEFT JOIN user_profiles dp ON dp.user_id = r.decided_by \
         LEFT JOIN web_credentials dw ON dw.user_id = r.decided_by \
         LEFT JOIN google_identities dg ON dg.user_id = r.decided_by \
         LEFT JOIN turn_jobs tj ON tj.id = r.ask_job_id \
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
        .map(PublishRequestRow::with_ask)
}

async fn project_name(pool: &sqlx::PgPool, project_id: Uuid) -> String {
    sqlx::query_scalar("SELECT name FROM projects WHERE id = $1").bind(project_id).fetch_one(pool).await.unwrap_or_default()
}

#[derive(Deserialize, Default)]
pub struct AskRequest {
    pub note: Option<String>,
    pub environment: Option<String>,
    /// The ask (turn job) in the project's chat this follows; the asker's latest finished one
    /// when left out.
    pub job_id: Option<Uuid>,
}

/// The ask a request follows: `named` when it is a job of the project's chat, else the asker's
/// latest finished one there.
async fn ask_followed(pool: &sqlx::PgPool, project_id: Uuid, user_id: Uuid, named: Option<Uuid>) -> Result<Option<Uuid>, ApiError> {
    let found: Option<Uuid> = sqlx::query_scalar(
        "SELECT tj.id FROM turn_jobs tj \
         JOIN projects p ON p.session_id = tj.session_id AND p.id = $1 \
         JOIN channel_identities ci ON ci.id = tj.sender_channel_identity_id \
         WHERE ($3::uuid IS NOT NULL AND tj.id = $3) OR ($3::uuid IS NULL AND ci.user_id = $2 AND tj.status = 'completed') \
         ORDER BY tj.created_at DESC LIMIT 1",
    )
    .bind(project_id)
    .bind(user_id)
    .bind(named)
    .fetch_optional(pool)
    .await
    .map_err(internal("failed to find the ask"))?;
    if named.is_some() && found.is_none() {
        return Err((StatusCode::BAD_REQUEST, "that ask isn't in this project's chat".to_string()));
    }
    Ok(found)
}

/// The files changed since the live deploy (the newest successful publish to production; every
/// file before the first).
async fn files_since_live(pool: &sqlx::PgPool, project_id: Uuid) -> Result<Vec<String>, ApiError> {
    let live: Option<DateTime<Utc>> = sqlx::query_scalar(
        "SELECT created_at FROM project_publishes WHERE project_id = $1 AND environment = $2 AND status = 'succeeded' ORDER BY created_at DESC LIMIT 1",
    )
    .bind(project_id)
    .bind(DEFAULT_ENV)
    .fetch_optional(pool)
    .await
    .map_err(internal("failed to read the live deploy"))?;
    let written = crate::ask_built::project_files_written(pool, project_id).await.map_err(internal("failed to list the changed files"))?;
    Ok(crate::ask_built::changed_since(&written, live))
}

/// `POST /api/projects/:id/deploy/requests`, optionally `{ note, environment }`: asks the admins
/// to publish. Admins publish themselves (409); one open request per person (409).
pub async fn ask(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(project_id): Path<Uuid>,
    body: Option<Json<AskRequest>>,
) -> Result<(StatusCode, Json<PublishRequestRow>), ApiError> {
    if super::deploy::require_builder(&state.pool, project_id, claims.sub).await? == "admin" {
        return Err((StatusCode::CONFLICT, "admins publish the project themselves".to_string()));
    }
    let req = body.map(|Json(b)| b).unwrap_or_default();
    let environment = req.environment.unwrap_or_else(|| DEFAULT_ENV.to_string());
    valid_env(&environment)?;
    let note = req.note.map(|n| n.trim().to_string()).filter(|n| !n.is_empty());
    if note.as_ref().is_some_and(|n| n.chars().count() > MAX_NOTE_CHARS) {
        return Err((StatusCode::BAD_REQUEST, "note is too long".to_string()));
    }
    let ask_job_id = ask_followed(&state.pool, project_id, claims.sub, req.job_id).await?;
    let files = files_since_live(&state.pool, project_id).await?;
    let id: Option<Uuid> = sqlx::query_scalar(
        "INSERT INTO publish_requests (project_id, requested_by, environment, note, ask_job_id, files) VALUES ($1, $2, $3, $4, $5, $6) \
         ON CONFLICT (project_id, requested_by) WHERE status = 'pending' DO NOTHING RETURNING id",
    )
    .bind(project_id)
    .bind(claims.sub)
    .bind(&environment)
    .bind(&note)
    .bind(ask_job_id)
    .bind(&files)
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
    let requests = requests.into_iter().map(PublishRequestRow::with_ask).collect();
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
