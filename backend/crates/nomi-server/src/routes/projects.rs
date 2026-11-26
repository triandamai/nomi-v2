use axum::extract::{Path, Query, State};
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::app::AppState;
use nomi_turn::bootstrap::ensure_web_channel_identity;
use nomi_agent_coding::{guess_content_type, project_file_key, validate_path};
use nomi_auth::extractor::AuthClaims;
use nomi_turn::bootstrap::bootstrap_identity_and_session;

#[derive(Serialize, sqlx::FromRow)]
pub struct ProjectSummary {
    pub id: Uuid,
    pub session_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    /// Who started it (their display name, or the start of their email).
    pub created_by: String,
    pub open_issues: i64,
    #[serde(default)]
    pub stack: Option<String>,
    #[serde(default)]
    pub backend: Option<String>,
    #[serde(default)]
    pub shipyard_app: Option<String>,
    #[serde(default)]
    pub chats_count: i64,
    #[serde(default)]
    pub deploys_count: i64,
    #[serde(default)]
    pub files_count: i64,
    #[serde(default)]
    pub deploy_url: Option<String>,
    /// Questions and approvals the crew is waiting on in the project's chats.
    #[serde(default)]
    pub waiting_on_you: i64,
    /// `{ status: "ok" | "failed", error, at, number }` for its latest finished publish, if any
    /// (`number` is the version it published; `error` is null unless it failed).
    #[serde(default)]
    pub last_deploy: Option<serde_json::Value>,
    /// `up` or `down` while it is live in production (down when it crashed since its last
    /// publish, lately); null when it isn't published.
    #[serde(default)]
    pub live_status: Option<String>,
    /// `{ step, steps, doing }` while the crew is at work on it (see `project_build`); else null.
    #[serde(default)]
    #[sqlx(default)]
    pub build: Option<serde_json::Value>,
}

/// A crash this recent, with no publish since, has the app shown as down.
const CRASH_WINDOW_MINUTES: i32 = 30;
/// The longest project name.
pub const NAME_MAX_CHARS: usize = 80;
/// The longest project description.
pub const DESCRIPTION_MAX_CHARS: usize = 500;

/// The summary columns every project list shares; `p` is the project. `$2` is the crash window.
const SUMMARY_COLUMNS: &str = "p.id, p.session_id, p.name, p.description, p.status, p.created_at, p.updated_at, \
    COALESCE(NULLIF(up.display_name, ''), split_part(COALESCE(w.email, g.email, ''), '@', 1)) AS created_by, \
    (SELECT count(*) FROM project_issues i WHERE i.project_id = p.id AND i.status = 'open') AS open_issues, \
    p.stack, p.backend, p.shipyard_app, \
    (SELECT count(*) FROM sessions s WHERE s.id = p.session_id OR s.in_project_id = p.id) AS chats_count, \
    (SELECT count(*) FROM project_publishes pp WHERE pp.project_id = p.id) AS deploys_count, \
    (SELECT count(*) FROM project_files pf WHERE pf.project_id = p.id) AS files_count, \
    (SELECT pp.url FROM project_publishes pp WHERE pp.project_id = p.id AND pp.status = 'succeeded' ORDER BY pp.created_at DESC LIMIT 1) AS deploy_url, \
    (SELECT count(*) FROM sessions s WHERE (s.id = p.session_id OR s.in_project_id = p.id \
         OR s.id IN (SELECT i.session_id FROM project_issues i WHERE i.project_id = p.id)) \
       AND EXISTS (SELECT 1 FROM messages q WHERE q.session_id = s.id AND q.sender_channel_identity_id IS NULL \
                   AND jsonb_typeof(q.content_blocks) = 'array' AND q.content_blocks @> '[{\"kind\": \"questions\"}]'::jsonb \
                   AND NOT EXISTS (SELECT 1 FROM messages u WHERE u.session_id = s.id AND u.sender_channel_identity_id IS NOT NULL AND u.created_at > q.created_at))) \
    + (SELECT count(*) FROM agent_sessions a WHERE a.status IN ('active', 'awaiting_approval') \
         AND COALESCE((a.state->>'paused_for_approval')::boolean, false) \
         AND a.session_id IN (SELECT s.id FROM sessions s WHERE s.id = p.session_id OR s.in_project_id = p.id \
                              UNION SELECT i.session_id FROM project_issues i WHERE i.project_id = p.id)) AS waiting_on_you, \
    (SELECT jsonb_build_object('status', CASE pp.status WHEN 'succeeded' THEN 'ok' ELSE 'failed' END, 'at', COALESCE(pp.finished_at, pp.created_at), \
            'error', pp.error, 'number', COALESCE(pp.version, pp.files_version)) \
       FROM project_publishes pp WHERE pp.project_id = p.id AND pp.status IN ('succeeded', 'failed') \
       ORDER BY COALESCE(pp.finished_at, pp.created_at) DESC LIMIT 1) AS last_deploy, \
    (SELECT CASE WHEN EXISTS (SELECT 1 FROM shipyard_events e WHERE e.project_id = p.id AND e.type = 'service.crashed' \
                               AND COALESCE(e.environment, 'production') = 'production' \
                               AND e.created_at > COALESCE(lp.finished_at, lp.created_at) \
                               AND e.created_at > now() - make_interval(mins => $2)) \
                 THEN 'down' ELSE 'up' END \
       FROM project_publishes lp WHERE lp.project_id = p.id AND lp.environment = 'production' AND lp.status = 'succeeded' \
         AND p.shipyard_app IS NOT NULL ORDER BY lp.created_at DESC LIMIT 1) AS live_status";

const SUMMARY_JOINS: &str = "FROM projects p \
    LEFT JOIN user_profiles up ON up.user_id = p.user_id \
    LEFT JOIN web_credentials w ON w.user_id = p.user_id \
    LEFT JOIN google_identities g ON g.user_id = p.user_id";

#[derive(Debug, Deserialize)]
pub struct ListProjectsQuery {
    pub org_id: Option<Uuid>,
}

#[derive(Serialize)]
pub struct ListProjectsResponse {
    /// The organization shown; null until the person picks one for projects.
    pub org_id: Option<Uuid>,
    pub projects: Vec<ProjectSummary>,
    /// The domain published apps are addressed under (Admin → Configuration → Domains), so the
    /// page can say where a new project will live; null when none is set.
    pub base_domain: Option<String>,
}

fn internal(context: &'static str) -> impl Fn(sqlx::Error) -> (StatusCode, &'static str) {
    move |e| {
        tracing::error!(error = %e, "{context}");
        (StatusCode::INTERNAL_SERVER_ERROR, context)
    }
}

/// The organization the person picked for projects, while they're still an active member of it.
pub(crate) async fn project_org(pool: &sqlx::PgPool, user_id: Uuid) -> Result<Option<Uuid>, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT p.project_org_id FROM user_preferences p \
         JOIN memberships m ON m.org_id = p.project_org_id AND m.user_id = p.user_id AND m.status = 'active' \
         WHERE p.user_id = $1",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
}

pub(crate) async fn is_active_member(pool: &sqlx::PgPool, user_id: Uuid, org_id: Uuid) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM memberships WHERE org_id = $1 AND user_id = $2 AND status = 'active')")
        .bind(org_id)
        .bind(user_id)
        .fetch_one(pool)
        .await
}

/// The projects of one organization the person is in: the one asked for, else the one they
/// picked for projects. Nothing is listed until they've picked one.
#[tracing::instrument(skip(state, claims))]
pub async fn list_projects(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Query(query): Query<ListProjectsQuery>,
) -> Result<Json<ListProjectsResponse>, (StatusCode, &'static str)> {
    let org_id = match query.org_id {
        Some(org_id) => {
            if !is_active_member(&state.pool, claims.sub, org_id).await.map_err(internal("failed to list projects"))? {
                return Err((StatusCode::NOT_FOUND, "organization not found"));
            }
            Some(org_id)
        }
        None => project_org(&state.pool, claims.sub).await.map_err(internal("failed to list projects"))?,
    };
    let Some(org_id) = org_id else {
        return Ok(Json(ListProjectsResponse { org_id: None, projects: Vec::new(), base_domain: crate::configuration::apps_base_domain().await }));
    };

    let mut projects: Vec<ProjectSummary> =
        sqlx::query_as(&format!("SELECT {SUMMARY_COLUMNS} {SUMMARY_JOINS} WHERE p.org_id = $1 ORDER BY p.updated_at DESC"))
            .bind(org_id)
            .bind(CRASH_WINDOW_MINUTES)
            .fetch_all(&state.pool)
            .await
            .map_err(internal("failed to list projects"))?;
    if let Err(e) = super::project_build::attach(&state.pool, &mut projects).await {
        tracing::warn!(error = %e, "failed to load the projects' build progress");
    }

    Ok(Json(ListProjectsResponse { org_id: Some(org_id), projects, base_domain: crate::configuration::apps_base_domain().await }))
}

#[derive(Serialize)]
pub struct CreateProjectSessionResponse {
    pub session_id: Uuid,
    pub project_id: Uuid,
}

#[derive(Deserialize, Default)]
pub struct CreateProjectRequest {
    /// The organization the project belongs to; the one picked for projects when left out.
    pub org_id: Option<Uuid>,
}

/// The "+ Add new project" entry point: creates the project row *before* any conversation
/// happens, in the same request that creates the chat session, rather than waiting on the
/// planning agent to remember to call its own create_project tool mid-conversation. That
/// dependency was unreliable in practice (see the delegation-guardrail fix) — a session could
/// end up looking like a plain chat, with no project, even after the user explicitly asked to
/// start one from this button. Placeholder-named "New project"; the planning agent's
/// create_project renames this same row once it knows what's actually being built, rather than
/// inserting a second one (see create_project in nomi-agent-planning).
///
/// A project always belongs to an organization: until the person has picked one for projects,
/// this answers 409 and the app asks them to.
#[tracing::instrument(skip(state, claims, req))]
pub async fn create_project_session(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    req: Option<Json<CreateProjectRequest>>,
) -> Result<(StatusCode, Json<CreateProjectSessionResponse>), (StatusCode, &'static str)> {
    let req = req.map(|Json(req)| req).unwrap_or_default();
    let org_id = match req.org_id {
        Some(org_id) => {
            if !is_active_member(&state.pool, claims.sub, org_id).await.map_err(internal("failed to create project"))? {
                return Err((StatusCode::NOT_FOUND, "organization not found"));
            }
            org_id
        }
        None => project_org(&state.pool, claims.sub)
            .await
            .map_err(internal("failed to create project"))?
            .ok_or((StatusCode::CONFLICT, "choose an organization for projects first"))?,
    };

    let created = start_project(&state, claims.sub, claims.active_org_id, org_id, "New project").await?;
    Ok((StatusCode::CREATED, Json(created)))
}

/// A new project named `name` in `org_id`, owned by `user_id`, with its own chat. Shared by
/// "+ Add new project", Duplicate and Remix.
pub(crate) async fn start_project(
    state: &AppState,
    user_id: Uuid,
    active_org_id: Uuid,
    org_id: Uuid,
    name: &str,
) -> Result<CreateProjectSessionResponse, (StatusCode, &'static str)> {
    nomi_usage::entitlements::check_new_project(&state.pool, user_id)
        .await
        .map_err(internal("failed to create project"))?
        .map_err(|hit| (StatusCode::PAYMENT_REQUIRED, hit.static_message()))?;

    ensure_web_channel_identity(&state.pool, user_id).await.map_err(|e| {
        tracing::error!(error = %e, "failed to resolve web identity");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to create project")
    })?;

    let chat_id = Uuid::new_v4().to_string();
    let bootstrap_result = bootstrap_identity_and_session(
        &state.pool,
        "web",
        "dm",
        &chat_id,
        &user_id.to_string(),
        Some(active_org_id),
    )
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "failed to create session");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to create project")
    })?;

    // Where the project runs is chosen once, here: projects that already exist keep theirs.
    let runtime = nomi_settings::config::new_project_runtime(user_id).await;
    let project_id: Uuid = sqlx::query_scalar(
        "INSERT INTO projects (user_id, session_id, org_id, name, runtime) VALUES ($1, $2, $3, $4, $5) RETURNING id",
    )
    .bind(user_id)
    .bind(bootstrap_result.session_id)
    .bind(org_id)
    .bind(name)
    .bind(runtime)
    .fetch_one(&state.pool)
    .await
    .map_err(internal("failed to create project"))?;

    Ok(CreateProjectSessionResponse { session_id: bootstrap_result.session_id, project_id })
}

#[derive(Serialize, sqlx::FromRow)]
pub struct ProjectFileSummary {
    pub path: String,
    pub content_type: String,
    pub size_bytes: i32,
}

#[derive(Serialize)]
pub struct ProjectDetailResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub plan: Option<String>,
    pub status: String,
    /// sveltekit, react, vue, astro or static (see nomi_agent_planning::STACKS).
    pub stack: String,
    /// Goes up on every file change; the preview copies changes in when it moves.
    pub files_version: i64,
    /// The version people see (one per crew reply that changed the files).
    pub version: i64,
    pub files: Vec<ProjectFileSummary>,
    /// The project's chat.
    pub session_id: Uuid,
    pub org_id: Uuid,
    pub org_name: String,
    pub open_issues: i64,
    /// `drizzle` or `relay`: where the app keeps its data, and so what the data sheet shows.
    pub backend: String,
    /// Set when the chat asked about is one of the project's issue chats rather than its own.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issue: Option<IssueChatRef>,
}

/// The issue a project's issue chat works on.
#[derive(Serialize, sqlx::FromRow)]
pub struct IssueChatRef {
    pub number: i32,
    pub title: String,
    pub crew_agent: String,
    pub crew_state: Option<String>,
    /// "open", or "closed" once it's in Done or Closed, which leaves the chat read-only.
    pub status: String,
    /// The fixed column it's in ("backlog", "ready", "done", "closed"); None in one people added.
    pub column: Option<String>,
}

/// The chat, organization, open issue count and backend of a project.
async fn project_context(pool: &sqlx::PgPool, project_id: Uuid) -> Result<(Uuid, Uuid, String, i64, String), sqlx::Error> {
    sqlx::query_as(
        "SELECT p.session_id, p.org_id, o.name, \
                (SELECT count(*) FROM project_issues i WHERE i.project_id = p.id AND i.status = 'open'), p.backend \
         FROM projects p JOIN organizations o ON o.id = p.org_id WHERE p.id = $1",
    )
    .bind(project_id)
    .fetch_one(pool)
    .await
}

pub(crate) async fn load_owned_project(
    pool: &sqlx::PgPool,
    project_id: Uuid,
    user_id: Uuid,
) -> Result<Option<(String, Option<String>, Option<String>, String, String, i64, i64)>, sqlx::Error> {
    sqlx::query_as(
        "SELECT name, description, plan, status, stack, files_version, version FROM projects WHERE id = $1 AND can_open_project(id, $2)",
    )
    .bind(project_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await
}

#[tracing::instrument(skip(state, claims))]
pub async fn get_project(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(project_id): Path<Uuid>,
) -> Result<Json<ProjectDetailResponse>, (StatusCode, &'static str)> {
    let row = load_owned_project(&state.pool, project_id, claims.sub).await.map_err(|e| {
        tracing::error!(error = %e, "failed to load project");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to load project")
    })?;
    let (name, description, plan, status, stack, files_version, version) = row.ok_or((StatusCode::NOT_FOUND, "project not found"))?;

    let files: Vec<ProjectFileSummary> = sqlx::query_as(
        "SELECT path, content_type, size_bytes FROM project_files WHERE project_id = $1 ORDER BY path",
    )
    .bind(project_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "failed to list project files");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to load project")
    })?;

    let (session_id, org_id, org_name, open_issues, backend) = project_context(&state.pool, project_id).await.map_err(|e| {
        tracing::error!(error = %e, "failed to load project context");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to load project")
    })?;

    Ok(Json(ProjectDetailResponse {
        id: project_id,
        name,
        description,
        plan,
        status,
        stack,
        files_version,
        version,
        files,
        session_id,
        org_id,
        org_name,
        open_issues,
        backend,
        issue: None,
    }))
}

async fn load_owned_project_by_session(
    pool: &sqlx::PgPool,
    session_id: Uuid,
    user_id: Uuid,
) -> Result<Option<(Uuid, String, Option<String>, Option<String>, String, String, i64, i64)>, sqlx::Error> {
    // The project's own chat or one of its issue chats (chat_project). A chat can accumulate more
    // than one project if the user asks to build several things in it over time; the most recent
    // wins, so the workspace page always matches what the conversation last started building.
    sqlx::query_as(
        "SELECT id, name, description, plan, status, stack, files_version, version FROM projects \
         WHERE id = chat_project($1) AND can_open_project(id, $2)",
    )
    .bind(session_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await
}

/// The project workspace page is keyed by session_id (a chat session may or may not have a
/// project attached yet) — this is how it finds out. A 404 here is a normal, expected state
/// (the user hasn't described anything to build yet), not an error.
#[tracing::instrument(skip(state, claims))]
pub async fn get_project_by_session(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(chat): Path<Uuid>,
) -> Result<Json<ProjectDetailResponse>, (StatusCode, &'static str)> {
    let row = load_owned_project_by_session(&state.pool, chat, claims.sub).await.map_err(|e| {
        tracing::error!(error = %e, "failed to load project by session");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to load project")
    })?;
    let (project_id, name, description, plan, status, stack, files_version, version) = row.ok_or((StatusCode::NOT_FOUND, "no project for this session"))?;

    let files: Vec<ProjectFileSummary> = sqlx::query_as(
        "SELECT path, content_type, size_bytes FROM project_files WHERE project_id = $1 ORDER BY path",
    )
    .bind(project_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "failed to list project files");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to load project")
    })?;

    let (session_id, org_id, org_name, open_issues, backend) = project_context(&state.pool, project_id).await.map_err(|e| {
        tracing::error!(error = %e, "failed to load project context");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to load project")
    })?;

    let issue: Option<IssueChatRef> = sqlx::query_as(
        "SELECT i.number, i.title, i.crew_agent, i.crew_state, i.status, c.fixed AS column \
         FROM project_issues i LEFT JOIN project_issue_columns c ON c.id = i.column_id \
         WHERE i.project_id = $1 AND i.session_id = $2",
    )
    .bind(project_id)
    .bind(chat)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "failed to load the chat's issue");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to load project")
    })?;

    Ok(Json(ProjectDetailResponse {
        id: project_id,
        name,
        description,
        plan,
        status,
        stack,
        files_version,
        version,
        files,
        session_id,
        org_id,
        org_name,
        open_issues,
        backend,
        issue,
    }))
}

#[tracing::instrument(skip(state, claims))]
pub async fn get_project_file(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path((project_id, path)): Path<(Uuid, String)>,
) -> Result<Response, (StatusCode, String)> {
    validate_path(&path).map_err(|e| (StatusCode::BAD_REQUEST, e))?;

    if load_owned_project(&state.pool, project_id, claims.sub).await.map_err(|e| {
        tracing::error!(error = %e, "failed to load project");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to load project".to_string())
    })?.is_none() {
        return Err((StatusCode::NOT_FOUND, "project not found".to_string()));
    }

    let content = state.project_storage.get_bytes(&project_file_key(project_id, &path)).await.map_err(|e| {
        tracing::error!(error = %e, "failed to read project file");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to read file".to_string())
    })?;
    let content = content.ok_or((StatusCode::NOT_FOUND, "file not found".to_string()))?;

    Ok(([(header::CONTENT_TYPE, guess_content_type(&path))], content).into_response())
}

/// How [`PutFileRequest::content`] is written when the file isn't text.
const BASE64_ENCODING: &str = "base64";

#[derive(serde::Deserialize)]
pub struct PutFileRequest {
    pub content: String,
    /// `"base64"` for a binary file (an image, a font); left out for text.
    #[serde(default)]
    pub encoding: Option<String>,
}

/// A refused file save: plain text as ever, except a full storage (402), which carries
/// `{"error":"storage_full"}` for the app to read.
fn put_file_error((status, message): (StatusCode, String)) -> axum::response::Response {
    use axum::response::IntoResponse;
    if status == StatusCode::PAYMENT_REQUIRED {
        return (status, Json(serde_json::json!({ "error": "storage_full", "message": message }))).into_response();
    }
    (status, message).into_response()
}

#[tracing::instrument(skip(state, claims, req))]
pub async fn put_project_file(
    state: State<AppState>,
    claims: AuthClaims,
    path: Path<(Uuid, String)>,
    req: Json<PutFileRequest>,
) -> Result<StatusCode, axum::response::Response> {
    save_project_file(state, claims, path, req).await.map_err(put_file_error)
}

async fn save_project_file(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path((project_id, path)): Path<(Uuid, String)>,
    Json(req): Json<PutFileRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    validate_path(&path).map_err(|e| (StatusCode::BAD_REQUEST, e))?;

    if load_owned_project(&state.pool, project_id, claims.sub).await.map_err(|e| {
        tracing::error!(error = %e, "failed to load project");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to load project".to_string())
    })?.is_none() {
        return Err((StatusCode::NOT_FOUND, "project not found".to_string()));
    }
    crate::routes::deploy::require_builder(&state.pool, project_id, claims.sub).await.map_err(|(code, why)| (code, why))?;

    let mut conn = state.pool.acquire().await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let storage_refusal = |hit: nomi_usage::entitlements::LimitHit| (StatusCode::PAYMENT_REQUIRED, hit.message());
    let storage_error = |e: sqlx::Error| {
        tracing::error!(error = %e, "failed to check storage");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to save file".to_string())
    };
    match req.encoding.as_deref() {
        None => {
            nomi_usage::entitlements::check_file_write(&mut conn, project_id, &path, req.content.len() as i64)
                .await
                .map_err(storage_error)?
                .map_err(storage_refusal)?;
            nomi_agent_coding::files::save_file(&mut conn, &state.project_storage, project_id, &path, &req.content).await
        }
        Some(BASE64_ENCODING) => {
            use base64::Engine as _;
            let bytes = base64::engine::general_purpose::STANDARD
                .decode(req.content.as_bytes())
                .map_err(|_| (StatusCode::BAD_REQUEST, "content isn't valid base64".to_string()))?;
            nomi_usage::entitlements::check_file_write(&mut conn, project_id, &path, bytes.len() as i64)
                .await
                .map_err(storage_error)?
                .map_err(storage_refusal)?;
            nomi_agent_coding::files::save_bytes(&mut conn, &state.project_storage, project_id, &path, &bytes).await
        }
        Some(other) => return Err((StatusCode::BAD_REQUEST, format!("unknown encoding {other}"))),
    }
    .map_err(|e| (StatusCode::BAD_REQUEST, e))?;
    nomi_agent_coding::files::bump_files_version_by(&mut conn, project_id, claims.sub).await.map_err(|e| {
        tracing::error!(error = %e, "failed to bump project files version");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to save file".to_string())
    })?;

    Ok(StatusCode::OK)
}

#[tracing::instrument(skip(state, claims))]
pub async fn delete_project_file(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path((project_id, path)): Path<(Uuid, String)>,
) -> Result<StatusCode, (StatusCode, String)> {
    validate_path(&path).map_err(|e| (StatusCode::BAD_REQUEST, e))?;

    if load_owned_project(&state.pool, project_id, claims.sub).await.map_err(|e| {
        tracing::error!(error = %e, "failed to load project");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to load project".to_string())
    })?.is_none() {
        return Err((StatusCode::NOT_FOUND, "project not found".to_string()));
    }
    crate::routes::deploy::require_builder(&state.pool, project_id, claims.sub).await.map_err(|(code, why)| (code, why))?;

    let mut conn = state.pool.acquire().await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    nomi_agent_coding::files::remove_file(&mut conn, &state.project_storage, project_id, &path).await.map_err(|e| {
        tracing::error!(error = %e, "failed to delete project file");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to delete file".to_string())
    })?;
    nomi_agent_coding::files::bump_files_version_by(&mut conn, project_id, claims.sub).await.map_err(|e| {
        tracing::error!(error = %e, "failed to bump project files version");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to delete file".to_string())
    })?;

    Ok(StatusCode::OK)
}

/// Shared by both preview routes (Step 2) — one with just `:id` (defaults to `index.html`), one
/// with `:id/*path`. axum's `Path` tuple extractor requires the capture count to match the route
/// pattern exactly, so a single handler can't use `Path<(Uuid, Option<String>)>` to cover a route
/// that has no second capture at all; two thin wrappers around this shared function is the fix.
async fn render_preview(state: &AppState, user_id: Uuid, project_id: Uuid, path: String) -> Result<Response, (StatusCode, String)> {
    if load_owned_project(&state.pool, project_id, user_id).await.map_err(|e| {
        tracing::error!(error = %e, "failed to load project");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to load project".to_string())
    })?.is_none() {
        return Err((StatusCode::NOT_FOUND, "project not found".to_string()));
    }

    let content = state.project_storage.get_bytes(&project_file_key(project_id, &path)).await.map_err(|e| {
        tracing::error!(error = %e, "failed to read project file for preview");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to load preview".to_string())
    })?;
    let content = content.ok_or((StatusCode::NOT_FOUND, "file not found".to_string()))?;

    Ok(([(header::CONTENT_TYPE, guess_content_type(&path))], content).into_response())
}

#[tracing::instrument(skip(state, claims))]
pub async fn preview_project_index(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(project_id): Path<Uuid>,
) -> Result<Response, (StatusCode, String)> {
    render_preview(&state, claims.sub, project_id, "index.html".to_string()).await
}

#[tracing::instrument(skip(state, claims))]
pub async fn preview_project_file(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path((project_id, path)): Path<(Uuid, String)>,
) -> Result<Response, (StatusCode, String)> {
    render_preview(&state, claims.sub, project_id, path).await
}

#[derive(Serialize)]
pub struct TransferOrgSummary {
    pub id: Uuid,
    pub name: String,
    pub is_personal: bool,
    pub owner_id: Option<Uuid>,
    pub owner_name: Option<String>,
    pub my_role: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SearchTransferOrgsQuery {
    pub q: Option<String>,
}

#[tracing::instrument(skip(state, claims))]
pub async fn search_transfer_orgs(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(project_id): Path<Uuid>,
    Query(query): Query<SearchTransferOrgsQuery>,
) -> Result<Json<Vec<TransferOrgSummary>>, (StatusCode, String)> {
    let current_project: (Uuid, Uuid) = sqlx::query_as(
        "SELECT p.user_id, p.org_id FROM projects p WHERE p.id = $1 AND can_open_project(p.id, $2)"
    )
    .bind(project_id)
    .bind(claims.sub)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or((StatusCode::NOT_FOUND, "project not found".to_string()))?;

    let is_admin: bool = current_project.0 == claims.sub || sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS (SELECT 1 FROM memberships WHERE org_id = $1 AND user_id = $2 AND status = 'active' AND role IN ('owner', 'admin'))"
    )
    .bind(current_project.1)
    .bind(claims.sub)
    .fetch_one(&state.pool)
    .await
    .unwrap_or(false);

    if !is_admin {
        return Err((StatusCode::FORBIDDEN, "only the project owner or admins can transfer this project".to_string()));
    }

    let q = query.q.as_deref().unwrap_or("").trim();
    let current_org_id = current_project.1;

    let orgs: Vec<(Uuid, String, bool, Option<String>)> = if !q.is_empty() {
        let pattern = format!("%{q}%");
        sqlx::query_as(
            "SELECT o.id, o.name, o.is_personal, m.role \
             FROM organizations o \
             LEFT JOIN memberships m ON m.org_id = o.id AND m.user_id = $2 AND m.status = 'active' \
             WHERE o.name ILIKE $1 AND o.id != $3 AND o.is_personal = false AND o.deleted_at IS NULL \
             ORDER BY (m.role IS NOT NULL) DESC, o.name ASC \
             LIMIT 20",
        )
        .bind(&pattern)
        .bind(claims.sub)
        .bind(current_org_id)
        .fetch_all(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    } else {
        sqlx::query_as(
            "SELECT o.id, o.name, o.is_personal, m.role \
             FROM organizations o \
             JOIN memberships m ON m.org_id = o.id AND m.user_id = $1 AND m.status = 'active' \
             WHERE o.id != $2 AND o.is_personal = false \
             ORDER BY m.created_at ASC \
             LIMIT 20",
        )
        .bind(claims.sub)
        .bind(current_org_id)
        .fetch_all(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    };

    let mut summaries = Vec::new();
    for (org_id, name, is_personal, my_role) in orgs {
        let owner: Option<(Uuid, String)> = sqlx::query_as(
            "SELECT u.id, COALESCE(NULLIF(up.display_name, ''), split_part(COALESCE(w.email, g.email, ''), '@', 1), 'Owner') AS name \
             FROM memberships m \
             JOIN users u ON u.id = m.user_id \
             LEFT JOIN user_profiles up ON up.user_id = u.id \
             LEFT JOIN web_credentials w ON w.user_id = u.id \
             LEFT JOIN google_identities g ON g.user_id = u.id \
             WHERE m.org_id = $1 AND m.status = 'active' \
             ORDER BY (m.role = 'owner') DESC, (m.role = 'admin') DESC, m.created_at ASC \
             LIMIT 1",
        )
        .bind(org_id)
        .fetch_optional(&state.pool)
        .await
        .unwrap_or(None);

        summaries.push(TransferOrgSummary {
            id: org_id,
            name,
            is_personal,
            owner_id: owner.as_ref().map(|o| o.0),
            owner_name: owner.map(|o| o.1),
            my_role,
        });
    }

    Ok(Json(summaries))
}

#[derive(Debug, Deserialize)]
pub struct TransferProjectRequest {
    pub target_org_id: Uuid,
    /// The project's name as the person typed it to confirm; it has to match.
    pub confirm_name: String,
}

#[tracing::instrument(skip(state, claims))]
pub async fn transfer_project(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(project_id): Path<Uuid>,
    Json(req): Json<TransferProjectRequest>,
) -> Result<Response, (StatusCode, String)> {
    let project_row: Option<(Uuid, Uuid, Uuid, String)> = sqlx::query_as(
        "SELECT p.user_id, p.org_id, p.session_id, p.name FROM projects p WHERE p.id = $1 AND can_open_project(p.id, $2)"
    )
    .bind(project_id)
    .bind(claims.sub)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let (current_owner_id, current_org_id, session_id, project_name) = project_row.ok_or((StatusCode::NOT_FOUND, "project not found".to_string()))?;

    let is_admin: bool = current_owner_id == claims.sub || sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS (SELECT 1 FROM memberships WHERE org_id = $1 AND user_id = $2 AND status = 'active' AND role IN ('owner', 'admin'))"
    )
    .bind(current_org_id)
    .bind(claims.sub)
    .fetch_one(&state.pool)
    .await
    .unwrap_or(false);

    if !is_admin {
        return Err((StatusCode::FORBIDDEN, "only the project owner or admins can transfer this project".to_string()));
    }

    if req.target_org_id == current_org_id {
        return Err((StatusCode::BAD_REQUEST, "project is already in this organization".to_string()));
    }
    if req.confirm_name.trim() != project_name.trim() {
        return Err((StatusCode::BAD_REQUEST, "type the project's name to confirm the transfer".to_string()));
    }

    let target_org_exists: bool = sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM organizations WHERE id = $1 AND deleted_at IS NULL)")
        .bind(req.target_org_id)
        .fetch_one(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if !target_org_exists {
        return Err((StatusCode::NOT_FOUND, "target organization not found".to_string()));
    }

    let target_owner: Option<Uuid> = sqlx::query_scalar(
        "SELECT m.user_id FROM memberships m \
         WHERE m.org_id = $1 AND m.status = 'active' \
         ORDER BY (m.role = 'owner') DESC, (m.role = 'admin') DESC, m.created_at ASC \
         LIMIT 1"
    )
    .bind(req.target_org_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let target_owner_id = target_owner.ok_or((StatusCode::BAD_REQUEST, "target organization has no active members or owner".to_string()))?;

    // A workspace the person doesn't own isn't theirs to put a project in: the owner decides.
    let role: Option<String> = sqlx::query_scalar("SELECT role FROM memberships WHERE org_id = $1 AND user_id = $2 AND status = 'active'")
        .bind(req.target_org_id)
        .bind(claims.sub)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    if super::project_transfers::transfer_needs_approval(role.as_deref()) {
        let request = super::project_transfers::create_request(&state.pool, project_id, current_org_id, req.target_org_id, claims.sub).await?;
        return Ok((StatusCode::ACCEPTED, Json(request)).into_response());
    }

    move_project(&state, project_id, session_id, &project_name, current_org_id, req.target_org_id, target_owner_id, claims.sub).await?;
    Ok(StatusCode::OK.into_response())
}

/// Moves the project (and its chat, and its Shipyard app) into `target_org_id`, under
/// `target_owner_id`; `mover` is who asked, and isn't told. The move itself, whether it was
/// allowed outright or approved by the receiving workspace's owner.
#[allow(clippy::too_many_arguments)]
pub(crate) async fn move_project(
    state: &AppState,
    project_id: Uuid,
    session_id: Uuid,
    project_name: &str,
    current_org_id: Uuid,
    target_org_id: Uuid,
    target_owner_id: Uuid,
    mover: Uuid,
) -> Result<(), (StatusCode, String)> {
    // A published project's Shipyard app moves first, so the two stay in step: when Shipyard
    // refuses (a publish is running, the destination's plan can't hold it), nothing moves here.
    move_shipyard_app(state, project_id, current_org_id, target_org_id).await?;

    let mut tx = state.pool.begin().await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    sqlx::query("UPDATE projects SET org_id = $2, user_id = $3, updated_at = now() WHERE id = $1")
        .bind(project_id)
        .bind(target_org_id)
        .bind(target_owner_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    sqlx::query("UPDATE sessions SET org_id = $2, user_id = $3 WHERE id = $1")
        .bind(session_id)
        .bind(target_org_id)
        .bind(target_owner_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    sqlx::query(
        "DELETE FROM project_members WHERE project_id = $1 AND user_id NOT IN (SELECT user_id FROM memberships WHERE org_id = $2 AND status = 'active')"
    )
    .bind(project_id)
    .bind(target_org_id)
    .execute(&mut *tx)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    tx.commit().await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    crate::project_events::changed(project_id, "app.transferred");
    notify_transfer(&state.pool, project_id, project_name, current_org_id, target_org_id, mover).await;
    Ok(())
}

/// Moves the project's app on Shipyard to the destination organization (created there first if
/// it hasn't published anything yet). A project that was never published has nothing to move.
async fn move_shipyard_app(state: &AppState, project_id: Uuid, from_org: Uuid, to_org: Uuid) -> Result<(), (StatusCode, String)> {
    let internal = |e: sqlx::Error| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string());
    let app: Option<String> = sqlx::query_scalar("SELECT shipyard_app FROM projects WHERE id = $1")
        .bind(project_id)
        .fetch_one(&state.pool)
        .await
        .map_err(internal)?;
    let Some(app) = app else { return Ok(()) };
    let shipyard = crate::routes::deploy::client(state).map_err(|_| {
        (StatusCode::SERVICE_UNAVAILABLE, "this project is published on Shipyard, which isn't set up here, so it can't be moved".to_string())
    })?;
    let to_name: String = sqlx::query_scalar("SELECT name FROM organizations WHERE id = $1")
        .bind(to_org)
        .fetch_one(&state.pool)
        .await
        .map_err(internal)?;
    let quota = crate::shipyard::org_quota_patch(&state.pool, to_org).await.map_err(internal)?;
    shipyard.put_org_with_quota(&to_org.to_string(), &to_name, Some(&quota)).await.map_err(crate::routes::deploy::shipyard_error)?;
    sqlx::query("UPDATE organizations SET shipyard_synced_at = now() WHERE id = $1").bind(to_org).execute(&state.pool).await.map_err(internal)?;
    match shipyard.transfer_app(&from_org.to_string(), &app, &to_org.to_string(), None).await {
        Ok(moved) => {
            tracing::info!(%project_id, %app, %from_org, %to_org, redeploying = %moved.get("redeploying").cloned().unwrap_or_default(), "moved the Shipyard app");
            Ok(())
        }
        // A publish or deployment is running, or the destination already has an app with this
        // key: the transfer is cancelled, never renamed on the way.
        Err(nomi_shipyard::ShipyardError::Rejected { status: 409, message, .. }) => {
            Err((StatusCode::CONFLICT, format!("Shipyard can't move the app ({message}), so the project wasn't moved")))
        }
        // The destination's plan can't hold the app, or its billing doesn't allow deploys.
        Err(nomi_shipyard::ShipyardError::Rejected { status: 403, message, .. }) => {
            Err((StatusCode::FORBIDDEN, format!("Shipyard won't move the app to this organization: {message}")))
        }
        Err(e) => Err(crate::routes::deploy::shipyard_error(e)),
    }
}

/// Tells the receiving organization's owners (its admins when it has no owner) that the project
/// is theirs now, in their inbox and by email. The person who moved it isn't told.
async fn notify_transfer(pool: &sqlx::PgPool, project_id: Uuid, project_name: &str, from_org: Uuid, to_org: Uuid, mover: Uuid) {
    let names: Result<(String, String, String), sqlx::Error> = sqlx::query_as(
        "SELECT (SELECT name FROM organizations WHERE id = $1), (SELECT name FROM organizations WHERE id = $2), \
                (SELECT COALESCE(NULLIF(up.display_name, ''), split_part(COALESCE(w.email, g.email, ''), '@', 1), 'Someone') \
                 FROM users u LEFT JOIN user_profiles up ON up.user_id = u.id LEFT JOIN web_credentials w ON w.user_id = u.id \
                 LEFT JOIN google_identities g ON g.user_id = u.id WHERE u.id = $3)",
    )
    .bind(from_org)
    .bind(to_org)
    .bind(mover)
    .fetch_one(pool)
    .await;
    let (from, org, who) = match names {
        Ok(names) => names,
        Err(e) => {
            tracing::warn!(error = %e, %project_id, "couldn't tell the new organization about a transferred project");
            return;
        }
    };
    let owners: Vec<Uuid> = sqlx::query_scalar(
        "SELECT user_id FROM memberships WHERE org_id = $1 AND status = 'active' AND user_id <> $2 \
           AND role = CASE WHEN EXISTS (SELECT 1 FROM memberships WHERE org_id = $1 AND status = 'active' AND role = 'owner') \
                           THEN 'owner' ELSE 'admin' END",
    )
    .bind(to_org)
    .bind(mover)
    .fetch_all(pool)
    .await
    .unwrap_or_default();
    for user_id in owners {
        let code: Option<String> = sqlx::query_scalar("SELECT language FROM user_preferences WHERE user_id = $1")
            .bind(user_id)
            .fetch_optional(pool)
            .await
            .ok()
            .flatten();
        let locale = nomi_i18n::Locale::from_code_or_default(code.as_deref());
        let notice = crate::notifications::Notice {
            kind: crate::notifications::Kind::Project,
            title: locale.tf("project.transferred_title", &[("name", project_name), ("org", &org)]),
            body: locale.tf("project.transferred_body", &[("who", &who), ("name", project_name), ("from", &from), ("org", &org)]),
            link: Some(format!("/projects/{project_id}/app")),
        };
        if let Err(e) = crate::notifications::send(pool, user_id, notice).await {
            tracing::warn!(error = %e, "couldn't send a project transfer notification");
        }
    }
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn a_full_project_storage_is_a_402_with_a_coded_json_body() {
        use http_body_util::BodyExt;
        let full = put_file_error((StatusCode::PAYMENT_REQUIRED, "it's full".to_string()));
        assert_eq!(full.status(), StatusCode::PAYMENT_REQUIRED);
        let body: serde_json::Value = serde_json::from_slice(&full.into_body().collect().await.unwrap().to_bytes()).unwrap();
        assert_eq!(body["error"], "storage_full");
        let other = put_file_error((StatusCode::BAD_REQUEST, "bad".to_string()));
        assert_eq!(other.status(), StatusCode::BAD_REQUEST);
        assert_eq!(&other.into_body().collect().await.unwrap().to_bytes()[..], b"bad");
    }

    use super::*;
    use uuid::Uuid;

    #[test]
    fn deserializes_transfer_request() {
        let org_id = Uuid::new_v4();
        let payload = format!(r#"{{"target_org_id":"{org_id}","confirm_name":"Todo"}}"#);
        let req: TransferProjectRequest = serde_json::from_str(&payload).expect("valid request");
        assert_eq!(req.target_org_id, org_id);

        let invalid = r#"{"target_org_id":"not-a-uuid","confirm_name":"Todo"}"#;
        assert!(serde_json::from_str::<TransferProjectRequest>(invalid).is_err());
    }

    #[test]
    fn deserializes_search_transfer_query() {
        let query_with_q: SearchTransferOrgsQuery =
            serde_json::from_value(serde_json::json!({"q": "acme"})).expect("valid query");
        assert_eq!(query_with_q.q.as_deref(), Some("acme"));

        let empty_query: SearchTransferOrgsQuery =
            serde_json::from_value(serde_json::json!({})).expect("empty query");
        assert_eq!(empty_query.q, None);
    }

    #[test]
    fn serializes_transfer_org_summary_as_spec() {
        let org_id = Uuid::new_v4();
        let owner_id = Uuid::new_v4();
        let summary = TransferOrgSummary {
            id: org_id,
            name: "Acme Corp".to_string(),
            is_personal: false,
            owner_id: Some(owner_id),
            owner_name: Some("Alice".to_string()),
            my_role: Some("admin".to_string()),
        };

        let json = serde_json::to_value(&summary).expect("serialization succeeds");
        assert_eq!(json["id"], org_id.to_string());
        assert_eq!(json["name"], "Acme Corp");
        assert_eq!(json["is_personal"], false);
        assert_eq!(json["owner_id"], owner_id.to_string());
        assert_eq!(json["owner_name"], "Alice");
        assert_eq!(json["my_role"], "admin");
    }
}
