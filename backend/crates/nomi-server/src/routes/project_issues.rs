//! A project's issues: tasks, enhancements and bugs, each with a severity. Everyone who can open
//! the project can file, edit, close and reopen them; the person who filed one (or the project's
//! owner) can delete it.
//!
//! They sit on a board: two fixed columns every project has (Backlog, where new issues land, and
//! Ready to dev) and the columns people add after them. Moving an issue into Ready to dev queues it
//! for the crew (`issue_worker` picks it up); only the project's admins change the columns.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use nomi_auth::extractor::AuthClaims;

use crate::app::AppState;

type ApiError = (StatusCode, &'static str);

pub const KINDS: [&str; 3] = ["task", "enhancement", "bug"];
pub const SEVERITIES: [&str; 4] = ["low", "medium", "high", "critical"];
const MAX_TITLE_CHARS: usize = 200;
const MAX_BODY_CHARS: usize = 20_000;
/// The most points one issue can be estimated at.
const MAX_POINTS: i16 = 100;
/// Columns a board can have, the two fixed ones included.
const MAX_COLUMNS: i64 = 12;
const MAX_COLUMN_NAME_CHARS: usize = 40;
/// Below this gap between neighbours a column's issues are renumbered 1, 2, 3…
const MIN_POSITION_GAP: f64 = 1e-6;
/// The crew member an issue goes to unless someone picks another.
pub const DEFAULT_CREW_AGENT: &str = "coding";
/// What the project's open pages hear when its board changes (`project_events::changed`).
pub const ISSUES_CHANGED: &str = "issues.changed";
/// The fixed columns, by their `fixed` key.
pub const BACKLOG: &str = "backlog";
pub const READY: &str = "ready";
/// Positions of the fixed columns; the ones people add come after them.
const BACKLOG_POSITION: i32 = 0;
const READY_POSITION: i32 = 1;
const FIRST_ADDED_POSITION: i32 = 2;

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

async fn role(pool: &sqlx::PgPool, user_id: Uuid, project_id: Uuid) -> Result<&'static str, ApiError> {
    crate::routes::deploy::project_role(pool, project_id, user_id).await.map_err(|(status, _)| (status, "project not found"))
}

/// Someone who may change the project: anyone who can open it except a viewer.
async fn require_builder(pool: &sqlx::PgPool, user_id: Uuid, project_id: Uuid) -> Result<(), ApiError> {
    match role(pool, user_id, project_id).await? {
        "viewer" => Err((StatusCode::FORBIDDEN, "you can view this project but not change it")),
        _ => Ok(()),
    }
}

async fn require_admin(pool: &sqlx::PgPool, user_id: Uuid, project_id: Uuid) -> Result<(), ApiError> {
    match role(pool, user_id, project_id).await? {
        "admin" => Ok(()),
        _ => Err((StatusCode::FORBIDDEN, "only the project's admins can change its columns")),
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
    pub column_id: Option<Uuid>,
    pub position: f64,
    pub start_on: Option<NaiveDate>,
    pub due_on: Option<NaiveDate>,
    pub points: Option<i16>,
    /// The crew member working it (agent type, "coding"), or who will once it's picked up.
    pub crew_agent: String,
    /// Who handed it on to `crew_agent`, when it was handed on.
    pub assigned_by_agent: Option<String>,
    /// queued, working, done or failed; null until it reaches Ready to dev.
    pub crew_state: Option<String>,
    /// The issue's own chat, once the crew picked it up.
    pub session_id: Option<Uuid>,
}

const ISSUE_COLUMNS: &str = "i.number, i.title, i.body, i.kind, i.severity, i.status, \
     COALESCE(NULLIF(up.display_name, ''), split_part(COALESCE(w.email, g.email, ''), '@', 1)) AS created_by, \
     COALESCE(i.created_by = $2, false) AS mine, i.closed_at, i.created_at, i.updated_at, \
     i.column_id, i.position, i.start_on, i.due_on, i.points, i.crew_agent, i.assigned_by_agent, i.crew_state, i.session_id";

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
        Err((StatusCode::BAD_REQUEST, "kind must be task, enhancement or bug"))
    }
}

fn check_severity(severity: &str) -> Result<(), ApiError> {
    if SEVERITIES.contains(&severity) {
        Ok(())
    } else {
        Err((StatusCode::BAD_REQUEST, "severity must be low, medium, high or critical"))
    }
}

fn check_points(points: Option<i16>) -> Result<(), ApiError> {
    match points {
        Some(p) if !(1..=MAX_POINTS).contains(&p) => Err((StatusCode::BAD_REQUEST, "points must be between 1 and 100")),
        _ => Ok(()),
    }
}

fn check_dates(start_on: Option<NaiveDate>, due_on: Option<NaiveDate>) -> Result<(), ApiError> {
    match (start_on, due_on) {
        (Some(start), Some(due)) if start > due => Err((StatusCode::BAD_REQUEST, "the deadline can't be before the start date")),
        _ => Ok(()),
    }
}

/// Only the built-in crew take issues.
fn check_crew(agent: &str) -> Result<(), ApiError> {
    if nomi_agent_core::crew::crew_name(agent).is_some() {
        Ok(())
    } else {
        Err((StatusCode::BAD_REQUEST, "unknown crew member"))
    }
}

/// A database refusal of the start date and deadline (an edit set only one of them) as a 400.
fn dates_or_internal(context: &'static str) -> impl Fn(sqlx::Error) -> ApiError {
    move |e| {
        let dates = e.as_database_error().and_then(|d| d.constraint()) == Some("project_issues_dates_check");
        if dates {
            (StatusCode::BAD_REQUEST, "the deadline can't be before the start date")
        } else {
            internal(context)(e)
        }
    }
}

/// A field that can be set, cleared (`null`) or left out: `None` when left out.
fn present<'de, D, T>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(deserializer).map(Some)
}

#[derive(Deserialize)]
pub struct NewIssue {
    pub title: String,
    #[serde(default)]
    pub body: String,
    pub kind: String,
    pub severity: String,
    #[serde(default)]
    pub start_on: Option<NaiveDate>,
    #[serde(default)]
    pub due_on: Option<NaiveDate>,
    #[serde(default)]
    pub points: Option<i16>,
    /// The crew member it goes to (agent type); Koda when left out.
    #[serde(default)]
    pub crew_agent: Option<String>,
    /// "Work immediately": it goes straight to Ready to dev, queued for the crew, starting today
    /// unless it says otherwise.
    #[serde(default)]
    pub start_now: bool,
}

/// The project's two fixed columns, made the first time its board is used.
pub async fn ensure_fixed_columns<'e>(executor: impl sqlx::PgExecutor<'e>, project_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO project_issue_columns (project_id, name, position, fixed) \
         VALUES ($1, 'Backlog', $2, 'backlog'), ($1, 'Ready to dev', $3, 'ready') \
         ON CONFLICT (project_id, fixed) DO NOTHING",
    )
    .bind(project_id)
    .bind(BACKLOG_POSITION)
    .bind(READY_POSITION)
    .execute(executor)
    .await?;
    Ok(())
}

async fn fixed_column(conn: &mut sqlx::PgConnection, project_id: Uuid, fixed: &str) -> Result<Uuid, sqlx::Error> {
    ensure_fixed_columns(&mut *conn, project_id).await?;
    sqlx::query_scalar("SELECT id FROM project_issue_columns WHERE project_id = $1 AND fixed = $2")
        .bind(project_id)
        .bind(fixed)
        .fetch_one(&mut *conn)
        .await
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
    check_points(req.points)?;
    let crew_agent = req.crew_agent.as_deref().unwrap_or(DEFAULT_CREW_AGENT);
    check_crew(crew_agent)?;
    let start_on = if req.start_now { req.start_on.or_else(|| Some(Utc::now().date_naive())) } else { req.start_on };
    check_dates(start_on, req.due_on)?;

    let mut tx = state.pool.begin().await.map_err(internal("failed to file issue"))?;
    // Holding the project row makes two people filing at once take turns for the next number.
    sqlx::query("SELECT id FROM projects WHERE id = $1 FOR UPDATE")
        .bind(project_id)
        .execute(&mut *tx)
        .await
        .map_err(internal("failed to file issue"))?;
    // A new issue lands at the bottom of Backlog, or of Ready to dev, queued, when it's to start now.
    let column = fixed_column(&mut tx, project_id, if req.start_now { READY } else { BACKLOG })
        .await
        .map_err(internal("failed to file issue"))?;
    let number: i32 = sqlx::query_scalar(
        "INSERT INTO project_issues (project_id, number, title, body, kind, severity, created_by, column_id, position, start_on, due_on, points, crew_agent, crew_state, queued_by) \
         VALUES ($1, (SELECT COALESCE(max(number), 0) + 1 FROM project_issues WHERE project_id = $1), $2, $3, $4, $5, $6, $7, \
                 (SELECT COALESCE(max(position), 0) + 1 FROM project_issues WHERE column_id = $7), $8, $9, $10, $11, \
                 CASE WHEN $12 THEN 'queued' END, CASE WHEN $12 THEN $6 END) \
         RETURNING number",
    )
    .bind(project_id)
    .bind(&title)
    .bind(&body)
    .bind(&req.kind)
    .bind(&req.severity)
    .bind(claims.sub)
    .bind(column)
    .bind(start_on)
    .bind(req.due_on)
    .bind(req.points)
    .bind(crew_agent)
    .bind(req.start_now)
    .fetch_one(&mut *tx)
    .await
    .map_err(internal("failed to file issue"))?;
    tx.commit().await.map_err(internal("failed to file issue"))?;
    crate::project_events::changed(project_id, ISSUES_CHANGED);

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
    #[serde(default, deserialize_with = "present")]
    pub start_on: Option<Option<NaiveDate>>,
    #[serde(default, deserialize_with = "present")]
    pub due_on: Option<Option<NaiveDate>>,
    #[serde(default, deserialize_with = "present")]
    pub points: Option<Option<i16>>,
    /// Who works it next (agent type). Changing it doesn't move work already started.
    pub crew_agent: Option<String>,
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
    check_points(req.points.flatten())?;
    check_dates(req.start_on.flatten(), req.due_on.flatten())?;
    if let Some(agent) = &req.crew_agent {
        check_crew(agent)?;
    }

    let updated = sqlx::query(
        "UPDATE project_issues SET \
            title = COALESCE($3, title), body = COALESCE($4, body), kind = COALESCE($5, kind), \
            severity = COALESCE($6, severity), \
            closed_at = CASE WHEN $7::text IS NULL OR $7 = status THEN closed_at WHEN $7 = 'closed' THEN now() ELSE NULL END, \
            status = COALESCE($7, status), \
            start_on = CASE WHEN $8 THEN $9 ELSE start_on END, \
            due_on = CASE WHEN $10 THEN $11 ELSE due_on END, \
            points = CASE WHEN $12 THEN $13 ELSE points END, \
            crew_agent = COALESCE($14, crew_agent), \
            assigned_by_agent = CASE WHEN $14::text IS NULL THEN assigned_by_agent ELSE NULL END, \
            updated_at = now() \
         WHERE project_id = $1 AND number = $2",
    )
    .bind(project_id)
    .bind(number)
    .bind(title)
    .bind(body)
    .bind(&req.kind)
    .bind(&req.severity)
    .bind(&req.status)
    .bind(req.start_on.is_some())
    .bind(req.start_on.flatten())
    .bind(req.due_on.is_some())
    .bind(req.due_on.flatten())
    .bind(req.points.is_some())
    .bind(req.points.flatten())
    .bind(&req.crew_agent)
    .execute(&state.pool)
    .await
    .map_err(dates_or_internal("failed to update issue"))?;
    if updated.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "issue not found"));
    }
    crate::project_events::changed(project_id, ISSUES_CHANGED);
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
            crate::project_events::changed(project_id, ISSUES_CHANGED);
            Ok(StatusCode::NO_CONTENT)
        }
    }
}

#[derive(Serialize, sqlx::FromRow)]
pub struct BoardColumn {
    pub id: Uuid,
    pub name: String,
    pub position: i32,
    /// "backlog" or "ready" for the fixed columns, which can't be renamed, moved or deleted.
    pub fixed: Option<String>,
    #[sqlx(skip)]
    pub issues: Vec<Issue>,
}

#[derive(Serialize)]
pub struct Board {
    pub columns: Vec<BoardColumn>,
}

async fn load_board(pool: &sqlx::PgPool, user_id: Uuid, project_id: Uuid) -> Result<Board, ApiError> {
    ensure_fixed_columns(pool, project_id).await.map_err(internal("failed to load the board"))?;
    let mut columns: Vec<BoardColumn> =
        sqlx::query_as("SELECT id, name, position, fixed FROM project_issue_columns WHERE project_id = $1 ORDER BY position, created_at")
            .bind(project_id)
            .fetch_all(pool)
            .await
            .map_err(internal("failed to load the board"))?;
    let issues: Vec<Issue> = sqlx::query_as(&format!(
        "SELECT {ISSUE_COLUMNS} {ISSUE_JOINS} WHERE i.project_id = $1 AND i.status = 'open' ORDER BY i.position, i.number"
    ))
    .bind(project_id)
    .bind(user_id)
    .fetch_all(pool)
    .await
    .map_err(internal("failed to load the board"))?;
    let backlog = columns.iter().position(|c| c.fixed.as_deref() == Some(BACKLOG));
    for issue in issues {
        // An issue whose column was deleted under it shows in Backlog.
        let at = issue.column_id.and_then(|id| columns.iter().position(|c| c.id == id)).or(backlog);
        if let Some(at) = at {
            columns[at].issues.push(issue);
        }
    }
    Ok(Board { columns })
}

/// `GET /api/projects/:id/board`: the columns in order, each with its open issues in order.
pub async fn get_board(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(project_id): Path<Uuid>,
) -> Result<Json<Board>, ApiError> {
    require_project(&state.pool, claims.sub, project_id).await?;
    Ok(Json(load_board(&state.pool, claims.sub, project_id).await?))
}

fn clean_column_name(name: &str) -> Result<String, ApiError> {
    let name = name.trim();
    if name.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "name must not be empty"));
    }
    if name.chars().count() > MAX_COLUMN_NAME_CHARS {
        return Err((StatusCode::BAD_REQUEST, "name is too long"));
    }
    Ok(name.to_string())
}

#[derive(Deserialize)]
pub struct NewColumn {
    pub name: String,
}

/// `POST /api/projects/:id/board/columns`: a column after the last one.
pub async fn add_column(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(project_id): Path<Uuid>,
    Json(req): Json<NewColumn>,
) -> Result<(StatusCode, Json<Board>), ApiError> {
    require_admin(&state.pool, claims.sub, project_id).await?;
    let name = clean_column_name(&req.name)?;
    ensure_fixed_columns(&state.pool, project_id).await.map_err(internal("failed to add the column"))?;
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM project_issue_columns WHERE project_id = $1")
        .bind(project_id)
        .fetch_one(&state.pool)
        .await
        .map_err(internal("failed to add the column"))?;
    if count >= MAX_COLUMNS {
        return Err((StatusCode::CONFLICT, "the board has as many columns as it can"));
    }
    sqlx::query(
        "INSERT INTO project_issue_columns (project_id, name, position) \
         VALUES ($1, $2, GREATEST($3, (SELECT max(position) + 1 FROM project_issue_columns WHERE project_id = $1)))",
    )
    .bind(project_id)
    .bind(&name)
    .bind(FIRST_ADDED_POSITION)
    .execute(&state.pool)
    .await
    .map_err(internal("failed to add the column"))?;
    crate::project_events::changed(project_id, ISSUES_CHANGED);
    Ok((StatusCode::CREATED, Json(load_board(&state.pool, claims.sub, project_id).await?)))
}

#[derive(Deserialize)]
pub struct ColumnChange {
    pub name: Option<String>,
    /// Where it goes among the columns people added: 0 is right after Ready to dev.
    pub index: Option<usize>,
}

async fn added_column(pool: &sqlx::PgPool, project_id: Uuid, column_id: Uuid) -> Result<(), ApiError> {
    let fixed: Option<Option<String>> = sqlx::query_scalar("SELECT fixed FROM project_issue_columns WHERE id = $1 AND project_id = $2")
        .bind(column_id)
        .bind(project_id)
        .fetch_optional(pool)
        .await
        .map_err(internal("failed to look up the column"))?;
    match fixed {
        None => Err((StatusCode::NOT_FOUND, "column not found")),
        Some(Some(_)) => Err((StatusCode::CONFLICT, "Backlog and Ready to dev can't be renamed, moved or deleted")),
        Some(None) => Ok(()),
    }
}

/// `PATCH /api/projects/:id/board/columns/:column`: renames or reorders a column people added.
pub async fn update_column(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path((project_id, column_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<ColumnChange>,
) -> Result<Json<Board>, ApiError> {
    require_admin(&state.pool, claims.sub, project_id).await?;
    added_column(&state.pool, project_id, column_id).await?;
    let mut tx = state.pool.begin().await.map_err(internal("failed to change the column"))?;
    if let Some(name) = req.name.as_deref() {
        sqlx::query("UPDATE project_issue_columns SET name = $2 WHERE id = $1")
            .bind(column_id)
            .bind(clean_column_name(name)?)
            .execute(&mut *tx)
            .await
            .map_err(internal("failed to change the column"))?;
    }
    if let Some(index) = req.index {
        let mut order: Vec<Uuid> = sqlx::query_scalar(
            "SELECT id FROM project_issue_columns WHERE project_id = $1 AND fixed IS NULL ORDER BY position, created_at FOR UPDATE",
        )
        .bind(project_id)
        .fetch_all(&mut *tx)
        .await
        .map_err(internal("failed to change the column"))?;
        order.retain(|id| *id != column_id);
        order.insert(index.min(order.len()), column_id);
        for (at, id) in order.iter().enumerate() {
            sqlx::query("UPDATE project_issue_columns SET position = $2 WHERE id = $1")
                .bind(id)
                .bind(FIRST_ADDED_POSITION + at as i32)
                .execute(&mut *tx)
                .await
                .map_err(internal("failed to change the column"))?;
        }
    }
    tx.commit().await.map_err(internal("failed to change the column"))?;
    crate::project_events::changed(project_id, ISSUES_CHANGED);
    Ok(Json(load_board(&state.pool, claims.sub, project_id).await?))
}

/// `DELETE /api/projects/:id/board/columns/:column`: its issues go back to Backlog.
pub async fn delete_column(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path((project_id, column_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<Board>, ApiError> {
    require_admin(&state.pool, claims.sub, project_id).await?;
    added_column(&state.pool, project_id, column_id).await?;
    let mut tx = state.pool.begin().await.map_err(internal("failed to delete the column"))?;
    let backlog = fixed_column(&mut tx, project_id, BACKLOG).await.map_err(internal("failed to delete the column"))?;
    sqlx::query(
        "UPDATE project_issues SET column_id = $2, \
            position = (SELECT COALESCE(max(position), 0) FROM project_issues WHERE column_id = $2) + position \
         WHERE column_id = $1",
    )
    .bind(column_id)
    .bind(backlog)
    .execute(&mut *tx)
    .await
    .map_err(internal("failed to delete the column"))?;
    sqlx::query("DELETE FROM project_issue_columns WHERE id = $1")
        .bind(column_id)
        .execute(&mut *tx)
        .await
        .map_err(internal("failed to delete the column"))?;
    tx.commit().await.map_err(internal("failed to delete the column"))?;
    crate::project_events::changed(project_id, ISSUES_CHANGED);
    Ok(Json(load_board(&state.pool, claims.sub, project_id).await?))
}

/// Where an issue goes when it lands at `index` among `others` (the column's positions, in
/// order, without it): between its new neighbours. `None` when they're too close and the
/// column needs renumbering first.
pub fn position_at(others: &[f64], index: usize) -> Option<f64> {
    let index = index.min(others.len());
    let before = index.checked_sub(1).map(|i| others[i]);
    let after = others.get(index).copied();
    match (before, after) {
        (None, None) => Some(1.0),
        (Some(b), None) => Some(b + 1.0),
        (None, Some(a)) => Some(a - 1.0),
        (Some(b), Some(a)) if a - b > MIN_POSITION_GAP => Some((a + b) / 2.0),
        _ => None,
    }
}

/// What the crew does with an issue that moves from one column to another: queued when it
/// enters Ready to dev (unless it's already queued or being worked), no longer queued when it
/// leaves before it was picked up. Work already started carries on; `None` keeps it as it is.
pub fn crew_state_after_move(state: Option<&str>, into_ready: bool, out_of_ready: bool) -> Option<Option<&'static str>> {
    match state {
        Some("queued") if out_of_ready => Some(None),
        Some("queued") | Some("working") => None,
        _ if into_ready => Some(Some("queued")),
        _ => None,
    }
}

#[derive(Deserialize)]
pub struct IssueMove {
    pub column_id: Uuid,
    /// Where it goes in that column: 0 is the top.
    pub index: usize,
}

/// The open issues' positions in a column, in order, without `skip`.
async fn column_positions(conn: &mut sqlx::PgConnection, column_id: Uuid, skip: Uuid) -> Result<Vec<f64>, sqlx::Error> {
    sqlx::query_scalar("SELECT position FROM project_issues WHERE column_id = $1 AND id <> $2 AND status = 'open' ORDER BY position, number")
        .bind(column_id)
        .bind(skip)
        .fetch_all(conn)
        .await
}

/// `POST /api/projects/:id/issues/:number/move`: to a column, at a place in it. Into Ready to dev
/// queues it for the crew.
pub async fn move_issue(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path((project_id, number)): Path<(Uuid, i32)>,
    Json(req): Json<IssueMove>,
) -> Result<Json<Issue>, ApiError> {
    require_builder(&state.pool, claims.sub, project_id).await?;
    let mut tx = state.pool.begin().await.map_err(internal("failed to move the issue"))?;
    let target: Option<Option<String>> = sqlx::query_scalar("SELECT fixed FROM project_issue_columns WHERE id = $1 AND project_id = $2")
        .bind(req.column_id)
        .bind(project_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(internal("failed to move the issue"))?;
    let Some(target_fixed) = target else {
        return Err((StatusCode::NOT_FOUND, "column not found"));
    };
    let current: Option<(Uuid, Option<String>, Option<String>)> = sqlx::query_as(
        "SELECT i.id, i.crew_state, c.fixed FROM project_issues i LEFT JOIN project_issue_columns c ON c.id = i.column_id \
         WHERE i.project_id = $1 AND i.number = $2 FOR UPDATE OF i",
    )
    .bind(project_id)
    .bind(number)
    .fetch_optional(&mut *tx)
    .await
    .map_err(internal("failed to move the issue"))?;
    let Some((issue_id, crew_state, from_fixed)) = current else {
        return Err((StatusCode::NOT_FOUND, "issue not found"));
    };

    let mut positions = column_positions(&mut tx, req.column_id, issue_id).await.map_err(internal("failed to move the issue"))?;
    let position = match position_at(&positions, req.index) {
        Some(p) => p,
        None => {
            // Too close to fit between: number the column 1, 2, 3… and try again.
            sqlx::query(
                "UPDATE project_issues i SET position = r.n FROM \
                 (SELECT id, row_number() OVER (ORDER BY position, number) AS n FROM project_issues WHERE column_id = $1 AND id <> $2) r \
                 WHERE i.id = r.id",
            )
            .bind(req.column_id)
            .bind(issue_id)
            .execute(&mut *tx)
            .await
            .map_err(internal("failed to move the issue"))?;
            positions = column_positions(&mut tx, req.column_id, issue_id).await.map_err(internal("failed to move the issue"))?;
            position_at(&positions, req.index).unwrap_or(0.0)
        }
    };

    let into_ready = target_fixed.as_deref() == Some(READY) && from_fixed.as_deref() != Some(READY);
    let out_of_ready = from_fixed.as_deref() == Some(READY) && target_fixed.as_deref() != Some(READY);
    let next_state = crew_state_after_move(crew_state.as_deref(), into_ready, out_of_ready);
    sqlx::query(
        "UPDATE project_issues SET column_id = $2, position = $3, updated_at = now(), \
            crew_state = CASE WHEN $4 THEN $5 ELSE crew_state END, \
            queued_by = CASE WHEN $4 AND $5 = 'queued' THEN $6 ELSE queued_by END \
         WHERE id = $1",
    )
    .bind(issue_id)
    .bind(req.column_id)
    .bind(position)
    .bind(next_state.is_some())
    .bind(next_state.flatten())
    .bind(claims.sub)
    .execute(&mut *tx)
    .await
    .map_err(internal("failed to move the issue"))?;
    tx.commit().await.map_err(internal("failed to move the issue"))?;
    if next_state == Some(Some("queued")) {
        tracing::info!(%project_id, issue = number, "issue board: queued for the crew");
    }
    crate::project_events::changed(project_id, ISSUES_CHANGED);
    Ok(Json(load_issue(&state.pool, claims.sub, project_id, number).await?))
}

/// `POST /api/projects/:id/issues/:number/retry`: queues an issue the crew couldn't finish again.
pub async fn retry_issue(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path((project_id, number)): Path<(Uuid, i32)>,
) -> Result<Json<Issue>, ApiError> {
    require_builder(&state.pool, claims.sub, project_id).await?;
    let retried = sqlx::query(
        "UPDATE project_issues i SET crew_state = 'queued', queued_by = $3, updated_at = now() \
         FROM project_issue_columns c \
         WHERE c.id = i.column_id AND c.fixed = 'ready' AND i.project_id = $1 AND i.number = $2 AND i.crew_state = 'failed'",
    )
    .bind(project_id)
    .bind(number)
    .bind(claims.sub)
    .execute(&state.pool)
    .await
    .map_err(internal("failed to retry the issue"))?;
    if retried.rows_affected() == 0 {
        return Err((StatusCode::CONFLICT, "only an issue the crew couldn't finish in Ready to dev can be tried again"));
    }
    crate::project_events::changed(project_id, ISSUES_CHANGED);
    Ok(Json(load_issue(&state.pool, claims.sub, project_id, number).await?))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_moved_issue_lands_between_its_new_neighbours() {
        assert_eq!(position_at(&[], 0), Some(1.0));
        assert_eq!(position_at(&[1.0, 2.0], 0), Some(0.0));
        assert_eq!(position_at(&[1.0, 2.0], 1), Some(1.5));
        assert_eq!(position_at(&[1.0, 2.0], 9), Some(3.0));
        assert_eq!(position_at(&[1.0, 1.0 + MIN_POSITION_GAP / 2.0], 1), None);
    }

    #[test]
    fn ready_to_dev_queues_and_leaving_it_unqueues_unless_work_started() {
        assert_eq!(crew_state_after_move(None, true, false), Some(Some("queued")));
        assert_eq!(crew_state_after_move(Some("done"), true, false), Some(Some("queued")));
        assert_eq!(crew_state_after_move(Some("failed"), true, false), Some(Some("queued")));
        assert_eq!(crew_state_after_move(Some("queued"), false, true), Some(None));
        assert_eq!(crew_state_after_move(Some("working"), false, true), None);
        assert_eq!(crew_state_after_move(Some("done"), false, true), None);
        assert_eq!(crew_state_after_move(None, false, false), None);
    }
}
