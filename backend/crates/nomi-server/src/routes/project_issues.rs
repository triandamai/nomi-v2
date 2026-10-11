//! A project's issues: feature requests, enhancements and bugs, each with a severity. Everyone
//! who can open the project can file, edit, close and reopen them; the person who filed one (or
//! the project's owner) can delete it.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use nomi_auth::extractor::AuthClaims;

use crate::app::AppState;

type ApiError = (StatusCode, &'static str);

pub const KINDS: [&str; 3] = ["feature", "enhancement", "bug"];
pub const SEVERITIES: [&str; 4] = ["low", "medium", "high", "critical"];
const MAX_TITLE_CHARS: usize = 200;
const MAX_BODY_CHARS: usize = 20_000;

fn internal(context: &'static str) -> impl Fn(sqlx::Error) -> ApiError {
    move |e| {
        tracing::error!(error = %e, "{context}");
        (StatusCode::INTERNAL_SERVER_ERROR, context)
    }
}

async fn require_project(pool: &sqlx::PgPool, user_id: Uuid, project_id: Uuid) -> Result<(), ApiError> {
    let can: bool = sqlx::query_scalar("SELECT can_open_project($1, $2)")
        .bind(project_id)
        .bind(user_id)
        .fetch_one(pool)
        .await
        .map_err(internal("failed to look up project"))?;
    if can {
        Ok(())
    } else {
        Err((StatusCode::NOT_FOUND, "project not found"))
    }
}

#[derive(Serialize, sqlx::FromRow)]
pub struct Issue {
    pub number: i32,
    pub title: String,
    pub body: String,
    pub kind: String,
    pub severity: String,
    pub status: String,
    pub created_by: Option<String>,
    pub mine: bool,
    pub closed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

const ISSUE_COLUMNS: &str = "i.number, i.title, i.body, i.kind, i.severity, i.status, \
     COALESCE(NULLIF(up.display_name, ''), split_part(COALESCE(w.email, g.email, ''), '@', 1)) AS created_by, \
     COALESCE(i.created_by = $2, false) AS mine, i.closed_at, i.created_at, i.updated_at";

const ISSUE_JOINS: &str = "FROM project_issues i \
     LEFT JOIN user_profiles up ON up.user_id = i.created_by \
     LEFT JOIN web_credentials w ON w.user_id = i.created_by \
     LEFT JOIN google_identities g ON g.user_id = i.created_by";

#[derive(Serialize, sqlx::FromRow)]
pub struct IssueCounts {
    pub open: i64,
    pub closed: i64,
}

#[derive(Serialize)]
pub struct IssueList {
    pub issues: Vec<Issue>,
    pub counts: IssueCounts,
}

#[derive(Deserialize)]
pub struct IssueQuery {
    /// "open" (default), "closed" or "all".
    pub status: Option<String>,
    pub kind: Option<String>,
    pub severity: Option<String>,
}

/// `GET /api/projects/:id/issues`: newest first, filtered by status, kind and severity.
pub async fn list_issues(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(project_id): Path<Uuid>,
    Query(query): Query<IssueQuery>,
) -> Result<Json<IssueList>, ApiError> {
    require_project(&state.pool, claims.sub, project_id).await?;
    let status = query.status.as_deref().unwrap_or("open");
    if !["open", "closed", "all"].contains(&status) {
        return Err((StatusCode::BAD_REQUEST, "status must be open, closed or all"));
    }
    let kind = query.kind.as_deref().filter(|k| !k.is_empty());
    if kind.is_some_and(|k| !KINDS.contains(&k)) {
        return Err((StatusCode::BAD_REQUEST, "unknown kind"));
    }
    let severity = query.severity.as_deref().filter(|s| !s.is_empty());
    if severity.is_some_and(|s| !SEVERITIES.contains(&s)) {
        return Err((StatusCode::BAD_REQUEST, "unknown severity"));
    }

    let issues: Vec<Issue> = sqlx::query_as(&format!(
        "SELECT {ISSUE_COLUMNS} {ISSUE_JOINS} \
         WHERE i.project_id = $1 AND ($3 = 'all' OR i.status = $3) \
           AND ($4::text IS NULL OR i.kind = $4) AND ($5::text IS NULL OR i.severity = $5) \
         ORDER BY i.number DESC"
    ))
    .bind(project_id)
    .bind(claims.sub)
    .bind(status)
    .bind(kind)
    .bind(severity)
    .fetch_all(&state.pool)
    .await
    .map_err(internal("failed to list issues"))?;
    let counts: IssueCounts = sqlx::query_as(
        "SELECT count(*) FILTER (WHERE status = 'open') AS open, count(*) FILTER (WHERE status = 'closed') AS closed \
         FROM project_issues WHERE project_id = $1 AND ($2::text IS NULL OR kind = $2) AND ($3::text IS NULL OR severity = $3)",
    )
    .bind(project_id)
    .bind(kind)
    .bind(severity)
    .fetch_one(&state.pool)
    .await
    .map_err(internal("failed to list issues"))?;
    Ok(Json(IssueList { issues, counts }))
}

async fn load_issue(pool: &sqlx::PgPool, user_id: Uuid, project_id: Uuid, number: i32) -> Result<Issue, ApiError> {
    sqlx::query_as(&format!("SELECT {ISSUE_COLUMNS} {ISSUE_JOINS} WHERE i.project_id = $1 AND i.number = $3"))
        .bind(project_id)
        .bind(user_id)
        .bind(number)
        .fetch_optional(pool)
        .await
        .map_err(internal("failed to load issue"))?
        .ok_or((StatusCode::NOT_FOUND, "issue not found"))
}

/// `GET /api/projects/:id/issues/:number`.
pub async fn get_issue(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path((project_id, number)): Path<(Uuid, i32)>,
) -> Result<Json<Issue>, ApiError> {
    require_project(&state.pool, claims.sub, project_id).await?;
    Ok(Json(load_issue(&state.pool, claims.sub, project_id, number).await?))
}

fn clean_title(title: &str) -> Result<String, ApiError> {
    let title = title.trim();
    if title.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "title must not be empty"));
    }
    if title.chars().count() > MAX_TITLE_CHARS {
        return Err((StatusCode::BAD_REQUEST, "title is too long"));
    }
    Ok(title.to_string())
}

fn clean_body(body: &str) -> Result<String, ApiError> {
    if body.chars().count() > MAX_BODY_CHARS {
        return Err((StatusCode::BAD_REQUEST, "description is too long"));
    }
    Ok(body.trim().to_string())
}

fn check_kind(kind: &str) -> Result<(), ApiError> {
    if KINDS.contains(&kind) {
        Ok(())
    } else {
        Err((StatusCode::BAD_REQUEST, "kind must be feature, enhancement or bug"))
    }
}

fn check_severity(severity: &str) -> Result<(), ApiError> {
    if SEVERITIES.contains(&severity) {
        Ok(())
    } else {
        Err((StatusCode::BAD_REQUEST, "severity must be low, medium, high or critical"))
    }
}

#[derive(Deserialize)]
pub struct NewIssue {
    pub title: String,
    #[serde(default)]
    pub body: String,
    pub kind: String,
    pub severity: String,
}

/// `POST /api/projects/:id/issues`: files one, numbered after the project's last.
pub async fn create_issue(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(project_id): Path<Uuid>,
    Json(req): Json<NewIssue>,
) -> Result<(StatusCode, Json<Issue>), ApiError> {
    require_project(&state.pool, claims.sub, project_id).await?;
    let title = clean_title(&req.title)?;
    let body = clean_body(&req.body)?;
    check_kind(&req.kind)?;
    check_severity(&req.severity)?;

    let mut tx = state.pool.begin().await.map_err(internal("failed to file issue"))?;
    // Holding the project row makes two people filing at once take turns for the next number.
    sqlx::query("SELECT id FROM projects WHERE id = $1 FOR UPDATE")
        .bind(project_id)
        .execute(&mut *tx)
        .await
        .map_err(internal("failed to file issue"))?;
    let number: i32 = sqlx::query_scalar(
        "INSERT INTO project_issues (project_id, number, title, body, kind, severity, created_by) \
         VALUES ($1, (SELECT COALESCE(max(number), 0) + 1 FROM project_issues WHERE project_id = $1), $2, $3, $4, $5, $6) \
         RETURNING number",
    )
    .bind(project_id)
    .bind(&title)
    .bind(&body)
    .bind(&req.kind)
    .bind(&req.severity)
    .bind(claims.sub)
    .fetch_one(&mut *tx)
    .await
    .map_err(internal("failed to file issue"))?;
    tx.commit().await.map_err(internal("failed to file issue"))?;

    Ok((StatusCode::CREATED, Json(load_issue(&state.pool, claims.sub, project_id, number).await?)))
}

#[derive(Deserialize)]
pub struct IssueChange {
    pub title: Option<String>,
    pub body: Option<String>,
    pub kind: Option<String>,
    pub severity: Option<String>,
    /// "open" or "closed".
    pub status: Option<String>,
}

/// `PATCH /api/projects/:id/issues/:number`: edits it, or closes or reopens it.
pub async fn update_issue(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path((project_id, number)): Path<(Uuid, i32)>,
    Json(req): Json<IssueChange>,
) -> Result<Json<Issue>, ApiError> {
    require_project(&state.pool, claims.sub, project_id).await?;
    let title = req.title.as_deref().map(clean_title).transpose()?;
    let body = req.body.as_deref().map(clean_body).transpose()?;
    if let Some(kind) = &req.kind {
        check_kind(kind)?;
    }
    if let Some(severity) = &req.severity {
        check_severity(severity)?;
    }
    if req.status.as_deref().is_some_and(|s| s != "open" && s != "closed") {
        return Err((StatusCode::BAD_REQUEST, "status must be open or closed"));
    }

    let updated = sqlx::query(
        "UPDATE project_issues SET \
            title = COALESCE($3, title), body = COALESCE($4, body), kind = COALESCE($5, kind), \
            severity = COALESCE($6, severity), \
            closed_at = CASE WHEN $7::text IS NULL OR $7 = status THEN closed_at WHEN $7 = 'closed' THEN now() ELSE NULL END, \
            status = COALESCE($7, status), updated_at = now() \
         WHERE project_id = $1 AND number = $2",
    )
    .bind(project_id)
    .bind(number)
    .bind(title)
    .bind(body)
    .bind(&req.kind)
    .bind(&req.severity)
    .bind(&req.status)
    .execute(&state.pool)
    .await
    .map_err(internal("failed to update issue"))?;
    if updated.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "issue not found"));
    }
    Ok(Json(load_issue(&state.pool, claims.sub, project_id, number).await?))
}

/// `DELETE /api/projects/:id/issues/:number`: only whoever filed it, or the project's owner.
pub async fn delete_issue(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path((project_id, number)): Path<(Uuid, i32)>,
) -> Result<StatusCode, ApiError> {
    require_project(&state.pool, claims.sub, project_id).await?;
    let allowed: Option<bool> = sqlx::query_scalar(
        "SELECT COALESCE(i.created_by = $3, false) OR p.user_id = $3 FROM project_issues i JOIN projects p ON p.id = i.project_id \
         WHERE i.project_id = $1 AND i.number = $2",
    )
    .bind(project_id)
    .bind(number)
    .bind(claims.sub)
    .fetch_optional(&state.pool)
    .await
    .map_err(internal("failed to delete issue"))?;
    match allowed {
        None => Err((StatusCode::NOT_FOUND, "issue not found")),
        Some(false) => Err((StatusCode::FORBIDDEN, "only whoever filed it or the project's owner can delete it")),
        Some(true) => {
            sqlx::query("DELETE FROM project_issues WHERE project_id = $1 AND number = $2")
                .bind(project_id)
                .bind(number)
                .execute(&state.pool)
                .await
                .map_err(internal("failed to delete issue"))?;
            Ok(StatusCode::NO_CONTENT)
        }
    }
}
