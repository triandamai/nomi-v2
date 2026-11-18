//! Admin → Projects and Sandboxes. Every project with its live sandbox's phase, how many sandboxes
//! it has had and its issues; every sandbox record (`sandbox_instances`) in whatever state; and a
//! project's issues, read only, for an admin who isn't one of its people.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use chrono::{DateTime, Utc};
use nomi_auth::extractor::AuthClaims;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::app::AppState;
use crate::routes::admin_access::require_admin;
use crate::sandbox_store::{State as SandboxState, PHASE_IDLE};

/// The admin area these pages belong to.
pub const AREA: &str = "projects";
/// The most rows one page of either list holds.
const LIST_LIMIT: i64 = 500;

type Refused = (StatusCode, &'static str);

fn failed(what: &'static str) -> impl Fn(sqlx::Error) -> Refused {
    move |e| {
        tracing::error!(error = %e, "admin projects: {what}");
        (StatusCode::INTERNAL_SERVER_ERROR, what)
    }
}

/// A sandbox's provider state as the project page names it: idle, waking, running, stopping,
/// stopped or error.
pub fn phase_of(state: &str) -> &'static str {
    SandboxState::parse(state).map(SandboxState::phase).unwrap_or(PHASE_IDLE)
}

const OWNER_EMAIL: &str = "COALESCE(w.email, g.email)";

#[derive(Deserialize)]
pub struct ProjectQuery {
    /// Part of a project's name or its owner's email.
    pub q: Option<String>,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct AdminProject {
    pub id: Uuid,
    pub name: String,
    pub owner_email: Option<String>,
    pub workspace: Option<String>,
    pub runtime: String,
    pub created_at: DateTime<Utc>,
    /// The live sandbox's state, when the project has one.
    #[serde(skip)]
    pub sandbox_state: Option<String>,
    #[sqlx(skip)]
    pub sandbox_phase: Option<&'static str>,
    pub sandbox_records: i64,
    pub open_issues: i64,
    pub issues: i64,
}

#[derive(Serialize)]
pub struct ProjectsResponse {
    pub projects: Vec<AdminProject>,
}

/// `GET /api/admin/projects?q=`: newest first.
pub async fn list(State(state): State<AppState>, AuthClaims(claims): AuthClaims, Query(query): Query<ProjectQuery>) -> Result<Json<ProjectsResponse>, Refused> {
    require_admin(&claims, AREA, "view")?;
    let search = query.q.map(|q| q.trim().to_string()).filter(|q| !q.is_empty());
    let mut projects: Vec<AdminProject> = sqlx::query_as(&format!(
        "SELECT p.id, p.name, {OWNER_EMAIL} AS owner_email, o.name AS workspace, p.runtime, p.created_at, \
                s.state AS sandbox_state, \
                (SELECT count(*) FROM sandbox_instances i WHERE i.project_id = p.id) AS sandbox_records, \
                (SELECT count(*) FROM project_issues pi WHERE pi.project_id = p.id AND pi.status = 'open') AS open_issues, \
                (SELECT count(*) FROM project_issues pi WHERE pi.project_id = p.id) AS issues \
         FROM projects p \
         LEFT JOIN organizations o ON o.id = p.org_id \
         LEFT JOIN project_sandboxes s ON s.project_id = p.id \
         LEFT JOIN web_credentials w ON w.user_id = p.user_id \
         LEFT JOIN google_identities g ON g.user_id = p.user_id \
         WHERE $1::text IS NULL OR p.name ILIKE '%' || $1 || '%' OR {OWNER_EMAIL} ILIKE '%' || $1 || '%' \
         ORDER BY p.created_at DESC LIMIT $2"
    ))
    .bind(search)
    .bind(LIST_LIMIT)
    .fetch_all(&state.pool)
    .await
    .map_err(failed("failed to list projects"))?;
    for project in &mut projects {
        project.sandbox_phase = project.sandbox_state.as_deref().map(phase_of);
    }
    Ok(Json(ProjectsResponse { projects }))
}

#[derive(Deserialize)]
pub struct SandboxQuery {
    pub project: Option<Uuid>,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct AdminSandbox {
    pub id: Uuid,
    pub project_id: Uuid,
    pub project_name: String,
    pub provider: String,
    pub sandbox_id: String,
    pub state: String,
    #[sqlx(skip)]
    pub phase: &'static str,
    /// The project's live sandbox; the others ended.
    pub current: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
}

#[derive(Serialize)]
pub struct SandboxesResponse {
    pub sandboxes: Vec<AdminSandbox>,
}

/// `GET /api/admin/sandboxes?project=`: every sandbox record, newest first, one project's when asked.
pub async fn sandboxes(State(state): State<AppState>, AuthClaims(claims): AuthClaims, Query(query): Query<SandboxQuery>) -> Result<Json<SandboxesResponse>, Refused> {
    require_admin(&claims, AREA, "view")?;
    let mut sandboxes: Vec<AdminSandbox> = sqlx::query_as(
        "SELECT i.id, i.project_id, p.name AS project_name, i.provider, i.sandbox_id, i.state, i.ended_at IS NULL AS current, \
                i.created_at, i.updated_at, i.ended_at \
         FROM sandbox_instances i JOIN projects p ON p.id = i.project_id \
         WHERE $1::uuid IS NULL OR i.project_id = $1 \
         ORDER BY i.created_at DESC, i.id DESC LIMIT $2",
    )
    .bind(query.project)
    .bind(LIST_LIMIT)
    .fetch_all(&state.pool)
    .await
    .map_err(failed("failed to list sandboxes"))?;
    for sandbox in &mut sandboxes {
        sandbox.phase = phase_of(&sandbox.state);
    }
    Ok(Json(SandboxesResponse { sandboxes }))
}

#[derive(Serialize, sqlx::FromRow)]
pub struct IssueProject {
    pub id: Uuid,
    pub name: String,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct AdminIssue {
    pub number: i32,
    pub title: String,
    pub kind: String,
    pub severity: String,
    pub status: String,
    pub column_name: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
}

#[derive(Serialize)]
pub struct IssuesResponse {
    pub project: IssueProject,
    pub issues: Vec<AdminIssue>,
}

/// `GET /api/admin/projects/:id/issues`: open and closed, newest first.
pub async fn issues(State(state): State<AppState>, AuthClaims(claims): AuthClaims, Path(project_id): Path<Uuid>) -> Result<Json<IssuesResponse>, Refused> {
    require_admin(&claims, AREA, "view")?;
    let project: IssueProject = sqlx::query_as("SELECT id, name FROM projects WHERE id = $1")
        .bind(project_id)
        .fetch_optional(&state.pool)
        .await
        .map_err(failed("failed to read the project"))?
        .ok_or((StatusCode::NOT_FOUND, "no such project"))?;
    let issues: Vec<AdminIssue> = sqlx::query_as(
        "SELECT i.number, i.title, i.kind, i.severity, i.status, c.name AS column_name, i.created_at, i.updated_at, i.closed_at \
         FROM project_issues i LEFT JOIN project_issue_columns c ON c.id = i.column_id \
         WHERE i.project_id = $1 ORDER BY i.created_at DESC, i.number DESC LIMIT $2",
    )
    .bind(project_id)
    .bind(LIST_LIMIT)
    .fetch_all(&state.pool)
    .await
    .map_err(failed("failed to list issues"))?;
    Ok(Json(IssuesResponse { project, issues }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_sandbox_state_reads_as_the_project_pages_phase() {
        assert_eq!(phase_of("running"), "running");
        assert_eq!(phase_of("creating"), "waking");
        assert_eq!(phase_of("starting"), "waking");
        assert_eq!(phase_of("stopping"), "stopping");
        assert_eq!(phase_of("stopped"), "stopped");
        assert_eq!(phase_of("gone"), "idle");
        assert_eq!(phase_of("none"), "idle");
        assert_eq!(phase_of("error"), "error");
        assert_eq!(phase_of("something new"), "idle");
    }
}
