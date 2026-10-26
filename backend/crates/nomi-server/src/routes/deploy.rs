//! Publishing a project on Shipyard and looking after it once it's live: publish, the history
//! of publishes (each can be rolled back to), deployment and runtime logs, resource usage, the
//! events Shipyard sent, taking an environment offline, and deleting the whole project.
//!
//! Everyone who can open a project sees all of it; only its admins (whoever started it, people
//! made admin on it, and the organization's owners and admins) change anything.

use axum::body::Bytes;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::Json;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

use nomi_auth::extractor::AuthClaims;
use nomi_shipyard::{manifest, Shipyard, ShipyardError};

use crate::app::AppState;
use crate::project_ops::ShipyardAccess;
use crate::shipyard::{self as nomi_shipyard_side, EnsureError, ProjectTarget};

pub type ApiError = (StatusCode, String);

/// The environment a publish goes to unless another is named.
const DEFAULT_ENV: &str = "production";

pub(crate) fn internal<E: std::fmt::Display>(context: &'static str) -> impl Fn(E) -> ApiError {
    move |e| {
        tracing::error!(error = %e, "{context}");
        (StatusCode::INTERNAL_SERVER_ERROR, context.to_string())
    }
}

pub(crate) fn shipyard_error(e: ShipyardError) -> ApiError {
    match e {
        // Busy (a publish is running, the name is taken) is a conflict; anything else Shipyard
        // refused (a bad manifest, a missing secret, a plan or billing limit) can't be done as asked.
        ShipyardError::Rejected { status: 409, message, .. } => (StatusCode::CONFLICT, message),
        ShipyardError::Rejected { message, .. } => (StatusCode::UNPROCESSABLE_ENTITY, message),
        ShipyardError::NotFound(message) => (StatusCode::NOT_FOUND, message),
        ShipyardError::Unauthorized(message) => {
            tracing::error!(%message, "Shipyard refused Nomi's key");
            (StatusCode::BAD_GATEWAY, "Shipyard refused Nomi's key".to_string())
        }
        ShipyardError::Unavailable(message) => {
            tracing::warn!(%message, "Shipyard unavailable");
            (StatusCode::BAD_GATEWAY, "Shipyard couldn't be reached. Try again in a moment.".to_string())
        }
    }
}

pub(crate) fn ensure_error(e: EnsureError) -> ApiError {
    match e {
        EnsureError::Shipyard(e) => shipyard_error(e),
        EnsureError::Db(e) => internal("failed to prepare the project on Shipyard")(e),
    }
}

/// The person's role on the project, or 404 when they can't open it.
pub(crate) async fn project_role(pool: &sqlx::PgPool, project_id: Uuid, user_id: Uuid) -> Result<&'static str, ApiError> {
    let role: Option<bool> = sqlx::query_scalar(
        "SELECT (p.user_id = $2 \
                 OR EXISTS (SELECT 1 FROM project_members pm WHERE pm.project_id = p.id AND pm.user_id = $2 AND pm.role = 'admin') \
                 OR EXISTS (SELECT 1 FROM memberships m WHERE m.org_id = p.org_id AND m.user_id = $2 AND m.status = 'active' AND m.role IN ('owner', 'admin'))) \
         FROM projects p WHERE p.id = $1 AND can_open_project(p.id, $2)",
    )
    .bind(project_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .map_err(internal("failed to look up project"))?;
    match role {
        Some(true) => Ok("admin"),
        Some(false) => Ok("member"),
        None => Err((StatusCode::NOT_FOUND, "project not found".to_string())),
    }
}

pub(crate) async fn require_admin(pool: &sqlx::PgPool, project_id: Uuid, user_id: Uuid) -> Result<(), ApiError> {
    match project_role(pool, project_id, user_id).await? {
        "admin" => Ok(()),
        _ => Err((StatusCode::FORBIDDEN, "only the project's admins can do that".to_string())),
    }
}

pub(crate) fn client(state: &impl ShipyardAccess) -> Result<Shipyard<'_>, ApiError> {
    state
        .shipyard()
        .map(|config| Shipyard::new(state.http_client(), config))
        .ok_or((StatusCode::SERVICE_UNAVAILABLE, "publishing isn't set up on this Nomi".to_string()))
}

pub(crate) async fn target(state: &impl ShipyardAccess, project_id: Uuid) -> Result<ProjectTarget, ApiError> {
    nomi_shipyard_side::load_target(state.pool(), project_id)
        .await
        .map_err(internal("failed to load project"))?
        .ok_or((StatusCode::NOT_FOUND, "project not found".to_string()))
}

/// The Shipyard app of a project that was published before.
fn published_app(target: &ProjectTarget) -> Result<&str, ApiError> {
    target.shipyard_app.as_deref().ok_or((StatusCode::NOT_FOUND, "this project hasn't been published yet".to_string()))
}

fn valid_env(env: &str) -> Result<(), ApiError> {
    let ok = !env.is_empty()
        && env.len() <= 40
        && env.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        && !env.starts_with('-')
        && !env.ends_with('-');
    if ok {
        Ok(())
    } else {
        Err((StatusCode::BAD_REQUEST, "environment names are a-z, 0-9 and '-'".to_string()))
    }
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct PublishRow {
    pub id: Uuid,
    pub environment: String,
    pub status: String,
    pub url: Option<String>,
    pub error: Option<String>,
    pub files_version: i64,
    pub rollback_of: Option<Uuid>,
    /// Whether its source was kept, so it can be rolled back to.
    pub can_roll_back: bool,
    pub created_by: Option<String>,
    pub created_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
    /// Where a failed publish broke (`shipyard::PublishFailure`), when Shipyard said.
    pub failure: Option<Value>,
    /// The `files_version` of the publish this one rolled back to.
    pub rollback_of_version: Option<i64>,
    /// The crew member who started it through `publish_project`, when it wasn't the person.
    pub started_by_agent: Option<String>,
    /// The newest successful publish of its environment: what runs there now. Only the deploy
    /// overview works it out; a single publish says false.
    #[sqlx(default)]
    pub is_live: bool,
}

const PUBLISH_COLUMNS: &str = "pp.id, pp.environment, pp.status, pp.url, pp.error, pp.files_version, pp.rollback_of, \
     (pp.source_key IS NOT NULL) AS can_roll_back, \
     COALESCE(NULLIF(up.display_name, ''), split_part(COALESCE(w.email, g.email, ''), '@', 1)) AS created_by, \
     pp.created_at, pp.finished_at, pp.failure, rb.files_version AS rollback_of_version, pp.started_by_agent \
     FROM project_publishes pp \
     LEFT JOIN project_publishes rb ON rb.id = pp.rollback_of \
     LEFT JOIN user_profiles up ON up.user_id = pp.created_by \
     LEFT JOIN web_credentials w ON w.user_id = pp.created_by \
     LEFT JOIN google_identities g ON g.user_id = pp.created_by";

async fn publish_row(pool: &sqlx::PgPool, project_id: Uuid, publish_id: Uuid) -> Result<PublishRow, ApiError> {
    sqlx::query_as(&format!("SELECT {PUBLISH_COLUMNS} WHERE pp.project_id = $1 AND pp.id = $2"))
        .bind(project_id)
        .bind(publish_id)
        .fetch_optional(pool)
        .await
        .map_err(internal("failed to load publish"))?
        .ok_or((StatusCode::NOT_FOUND, "publish not found".to_string()))
}

#[derive(Serialize)]
pub struct DeployOverview {
    /// Whether this Nomi can publish at all.
    pub available: bool,
    pub role: &'static str,
    pub app: Option<String>,
    /// postgres, sqlite or none, read from the project's files.
    pub database: &'static str,
    /// The address of the newest successful publish to production.
    pub url: Option<String>,
    pub publishes: Vec<PublishRow>,
    /// What runs in production right now, from Shipyard; null when nothing does.
    pub live: Option<Value>,
}

/// The project's files, as Shipyard gets them (without excluded paths).
async fn project_files(state: &impl ShipyardAccess, project_id: Uuid) -> Result<Vec<(String, Vec<u8>)>, ApiError> {
    let paths: Vec<String> = sqlx::query_scalar("SELECT path FROM project_files WHERE project_id = $1 ORDER BY path")
        .bind(project_id)
        .fetch_all(state.pool())
        .await
        .map_err(internal("failed to list project files"))?;
    let mut files = Vec::with_capacity(paths.len());
    for path in paths {
        if nomi_shipyard::bundle::excluded(&path) {
            continue;
        }
        let key = nomi_agent_coding::project_file_key(project_id, &path);
        if let Some(bytes) = state.project_storage().get_bytes(&key).await.map_err(internal("failed to read project files"))? {
            files.push((path, bytes));
        }
    }
    Ok(files)
}

async fn database_kind(state: &AppState, project_id: Uuid) -> Result<manifest::Database, ApiError> {
    // Only the files that tell: the Drizzle config and package.json.
    let mut files = Vec::new();
    for path in ["drizzle.config.ts", "drizzle.config.js", "package.json"] {
        let key = nomi_agent_coding::project_file_key(project_id, path);
        if let Ok(Some(bytes)) = state.project_storage.get_bytes(&key).await {
            files.push((path.to_string(), bytes));
        }
    }
    Ok(manifest::detect_database(&files))
}

fn database_name(db: manifest::Database) -> &'static str {
    match db {
        manifest::Database::Postgres => "postgres",
        manifest::Database::Sqlite => "sqlite",
        manifest::Database::None => "none",
    }
}

/// `GET /api/projects/:id/deploy`
pub async fn overview(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(project_id): Path<Uuid>,
) -> Result<Json<DeployOverview>, ApiError> {
    let role = project_role(&state.pool, project_id, claims.sub).await?;
    let target = target(&state, project_id).await?;
    let mut publishes: Vec<PublishRow> =
        sqlx::query_as(&format!("SELECT {PUBLISH_COLUMNS} WHERE pp.project_id = $1 ORDER BY pp.created_at DESC LIMIT 30"))
            .bind(project_id)
            .fetch_all(&state.pool)
            .await
            .map_err(internal("failed to list publishes"))?;
    mark_live(&mut publishes);
    let url = publishes
        .iter()
        .find(|p| p.environment == DEFAULT_ENV && p.status == "succeeded" && p.url.is_some())
        .and_then(|p| p.url.clone());
    let live = match (client(&state), target.shipyard_app.as_deref()) {
        (Ok(client), Some(app)) => client.get_env(&target.org_key(), app, DEFAULT_ENV).await.ok(),
        _ => None,
    };
    Ok(Json(DeployOverview {
        available: state.shipyard.is_some(),
        role,
        app: target.shipyard_app.clone(),
        database: database_name(database_kind(&state, project_id).await?),
        url,
        publishes,
        live,
    }))
}

/// Marks the newest successful publish of each environment as live (`publishes` is newest first).
fn mark_live(publishes: &mut [PublishRow]) {
    let mut seen = std::collections::HashSet::new();
    for publish in publishes.iter_mut() {
        if publish.status == "succeeded" && seen.insert(publish.environment.clone()) {
            publish.is_live = true;
        }
    }
}

#[derive(Deserialize, Default)]
pub struct PublishRequest {
    pub environment: Option<String>,
}

/// Sends `source` with `manifest` and records the publish. The row is kept even when Shipyard
/// refuses it, with Shipyard's words, so the history shows what happened.
async fn send_publish(
    state: &impl ShipyardAccess,
    target: &ProjectTarget,
    app: &str,
    environment: &str,
    manifest: Value,
    source: Vec<u8>,
    rollback_of: Option<Uuid>,
    user_id: Uuid,
    agent: Option<&str>,
) -> Result<PublishRow, ApiError> {
    let id = Uuid::new_v4();
    let source_key = format!("publishes/{}/{id}.zip", target.id);
    state
        .project_storage()
        .put_bytes(&source_key, source.clone(), "application/zip")
        .await
        .map_err(internal("failed to keep the published source"))?;
    sqlx::query(
        "INSERT INTO project_publishes (id, project_id, environment, manifest, source_key, files_version, rollback_of, created_by, started_by_agent) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
    )
    .bind(id)
    .bind(target.id)
    .bind(environment)
    .bind(&manifest)
    .bind(&source_key)
    .bind(target.files_version)
    .bind(rollback_of)
    .bind(user_id)
    .bind(agent)
    .execute(state.pool())
    .await
    .map_err(internal("failed to record the publish"))?;

    let client = client(state)?;
    match client.publish(&target.org_key(), app, environment, &manifest, Some(source)).await {
        Ok(accepted) => {
            sqlx::query("UPDATE project_publishes SET shipyard_publish_id = $2, url = $3 WHERE id = $1")
                .bind(id)
                .bind(&accepted.publish_id)
                .bind(&accepted.url)
                .execute(state.pool())
                .await
                .map_err(internal("failed to record the publish"))?;
        }
        Err(e) => {
            sqlx::query("UPDATE project_publishes SET status = 'failed', error = $2, finished_at = now() WHERE id = $1")
                .bind(id)
                .bind(e.to_string())
                .execute(state.pool())
                .await
                .map_err(internal("failed to record the publish"))?;
            return Err(shipyard_error(e));
        }
    }
    publish_row(state.pool(), target.id, id).await
}

/// `POST /api/projects/:id/deploy/publish`: zips the project (adding a Dockerfile for its stack
/// when it has none), builds the manifest (database when the app keeps data) and publishes.
pub async fn publish(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(project_id): Path<Uuid>,
    body: Option<Json<PublishRequest>>,
) -> Result<(StatusCode, Json<PublishRow>), ApiError> {
    require_admin(&state.pool, project_id, claims.sub).await?;
    let environment = body.and_then(|Json(b)| b.environment).unwrap_or_else(|| DEFAULT_ENV.to_string());
    let row = start_publish(&state, project_id, claims.sub, &environment, None).await?;
    Ok((StatusCode::ACCEPTED, Json(row)))
}

/// Publishes the project's current files to `environment` as `user_id` (whose right to was
/// checked by the caller): the Deploy sheet's Publish, and agents' `publish_project`.
/// `agent` is the crew member publishing for `user_id` through `publish_project`, if any.
pub(crate) async fn start_publish(
    state: &impl ShipyardAccess,
    project_id: Uuid,
    user_id: Uuid,
    environment: &str,
    agent: Option<&str>,
) -> Result<PublishRow, ApiError> {
    valid_env(environment)?;
    let client = client(state)?;
    let running: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM project_publishes WHERE project_id = $1 AND environment = $2 AND status = 'running' \
         AND created_at > now() - interval '30 minutes')",
    )
    .bind(project_id)
    .bind(environment)
    .fetch_one(state.pool())
    .await
    .map_err(internal("failed to check publishes"))?;
    if running {
        return Err((StatusCode::CONFLICT, "a publish is already running".to_string()));
    }

    let target = target(state, project_id).await?;
    let mut files = project_files(state, project_id).await?;
    if !files.iter().any(|(p, _)| p == "package.json" || p == "index.html") {
        return Err((StatusCode::UNPROCESSABLE_ENTITY, "there's nothing to publish yet".to_string()));
    }
    let database = manifest::detect_database(&files);
    if !files.iter().any(|(p, _)| p == "Dockerfile") {
        let dockerfile = manifest::dockerfile(&target.stack, &files);
        files.push(("Dockerfile".to_string(), dockerfile.into_bytes()));
    }
    let secrets = crate::routes::project_secrets::names(state.pool(), project_id).await?;
    let mut manifest = manifest::build(database, &secrets);
    manifest::label(&mut manifest, &project_id.to_string(), &target.stack, target.files_version);
    let source = nomi_shipyard::bundle::zip(&files).map_err(internal("failed to pack the project"))?;

    let app = nomi_shipyard_side::ensure_app(state.pool(), &client, &target).await.map_err(ensure_error)?;
    send_publish(state, &target, &app, environment, manifest, source, None, user_id, agent).await
}

/// `POST /api/projects/:id/deploy/publishes/:publish_id/rollback`: publishes that publish's
/// source and manifest again, to its environment.
pub async fn rollback(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path((project_id, publish_id)): Path<(Uuid, Uuid)>,
) -> Result<(StatusCode, Json<PublishRow>), ApiError> {
    require_admin(&state.pool, project_id, claims.sub).await?;
    let client = client(&state)?;
    let old: Option<(String, Value, Option<String>)> = sqlx::query_as(
        "SELECT environment, manifest, source_key FROM project_publishes WHERE project_id = $1 AND id = $2",
    )
    .bind(project_id)
    .bind(publish_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(internal("failed to load publish"))?;
    let (environment, mut manifest, source_key) = old.ok_or((StatusCode::NOT_FOUND, "publish not found".to_string()))?;
    // The code goes back; the secrets stay as they are now.
    manifest::bind_secrets(&mut manifest, &crate::routes::project_secrets::names(&state.pool, project_id).await?);
    let source_key = source_key.ok_or((StatusCode::GONE, "that publish's source wasn't kept".to_string()))?;
    let source = state
        .project_storage
        .get_bytes(&source_key)
        .await
        .map_err(internal("failed to read the published source"))?
        .ok_or((StatusCode::GONE, "that publish's source wasn't kept".to_string()))?;
    let target = target(&state, project_id).await?;
    let app = nomi_shipyard_side::ensure_app(&state.pool, &client, &target).await.map_err(ensure_error)?;
    let row = send_publish(&state, &target, &app, &environment, manifest, source, Some(publish_id), claims.sub, None).await?;
    Ok((StatusCode::ACCEPTED, Json(row)))
}

#[derive(Serialize)]
pub struct PublishDetail {
    #[serde(flatten)]
    pub publish: PublishRow,
    /// `[{ service, deployment_id, status }]` from Shipyard.
    pub deployments: Vec<Value>,
}

/// `GET /api/projects/:id/deploy/publishes/:publish_id`: the publish, asking Shipyard for its
/// state while it's still running (in case a webhook went missing).
pub async fn get_publish(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path((project_id, publish_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<PublishDetail>, ApiError> {
    project_role(&state.pool, project_id, claims.sub).await?;
    let target = target(&state, project_id).await?;
    let (environment, shipyard_id): (String, Option<String>) =
        sqlx::query_as("SELECT environment, shipyard_publish_id FROM project_publishes WHERE project_id = $1 AND id = $2")
            .bind(project_id)
            .bind(publish_id)
            .fetch_optional(&state.pool)
            .await
            .map_err(internal("failed to load publish"))?
            .ok_or((StatusCode::NOT_FOUND, "publish not found".to_string()))?;
    let mut deployments = Vec::new();
    if let (Some(shipyard_id), Some(app), Ok(client)) = (shipyard_id, target.shipyard_app.as_deref(), client(&state)) {
        if let Ok(remote) = client.get_publish(&target.org_key(), app, &environment, &shipyard_id).await {
            let list = remote
                .get("deployments")
                .or_else(|| remote.get("services"))
                .and_then(Value::as_array);
            if let Some(list) = list {
                deployments = list
                    .iter()
                    .filter_map(|d| match d {
                        Value::Object(_) => {
                            let mut obj = d.clone();
                            if let Value::Object(ref mut map) = obj {
                                if let Some(id) = map.get("id").cloned() {
                                    map.entry("deployment_id".to_string()).or_insert(id);
                                } else if let Some(dep_id) = map.get("deployment_id").cloned() {
                                    map.entry("id".to_string()).or_insert(dep_id);
                                }
                                map.entry("service".to_string()).or_insert_with(|| json!("web"));
                            }
                            Some(obj)
                        }
                        Value::String(s) => Some(json!({
                            "service": "web",
                            "id": s,
                            "deployment_id": s,
                            "status": "unknown"
                        })),
                        _ => None,
                    })
                    .collect();
            }
            let status = remote.get("status").and_then(Value::as_str).unwrap_or("running");
            if status == "succeeded" || status == "failed" {
                let url = remote.get("urls").and_then(Value::as_array).and_then(|u| u.first()).and_then(Value::as_str);
                let error = match remote.get("error") {
                    Some(Value::String(s)) => Some(s.clone()),
                    Some(Value::Null) | None => None,
                    Some(other) => Some(other.to_string()),
                };
                sqlx::query(
                    "UPDATE project_publishes SET status = $2, url = COALESCE($3, url), error = $4, \
                     finished_at = COALESCE(finished_at, now()) WHERE id = $1 AND status = 'running'",
                )
                .bind(publish_id)
                .bind(status)
                .bind(url)
                .bind(error)
                .execute(&state.pool)
                .await
                .map_err(internal("failed to update publish"))?;
            }
        }
    }
    // Fallback: If no deployments were listed on the publish, query environment's last deployments
    if deployments.is_empty() {
        if let (Some(app), Ok(client)) = (target.shipyard_app.as_deref(), client(&state)) {
            if let Ok(env_data) = client.get_env(&target.org_key(), app, &environment).await {
                if let Some(services) = env_data.get("services").and_then(Value::as_array) {
                    for s in services {
                        let name = s.get("name").and_then(Value::as_str).unwrap_or("web");
                        if let Some(last_dep) = s.get("last_deployment").filter(|d| !d.is_null()) {
                            let dep_id = last_dep.get("id").or_else(|| last_dep.get("deployment_id"));
                            if let Some(id) = dep_id {
                                deployments.push(json!({
                                    "service": name,
                                    "id": id,
                                    "deployment_id": id,
                                    "status": last_dep.get("status").unwrap_or(&json!("succeeded"))
                                }));
                            }
                        }
                    }
                }
            }
        }
    }
    let publish = publish_row(&state.pool, project_id, publish_id).await?;
    Ok(Json(PublishDetail { publish, deployments }))
}

#[derive(Deserialize)]
pub struct EnvQuery {
    pub environment: Option<String>,
}

fn env_of(query: &EnvQuery) -> Result<String, ApiError> {
    let env = query.environment.clone().unwrap_or_else(|| DEFAULT_ENV.to_string());
    valid_env(&env)?;
    Ok(env)
}

/// `GET /api/projects/:id/deploy/deployments/:deployment_id`: steps and build/deploy log.
pub async fn get_deployment(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path((project_id, deployment_id)): Path<(Uuid, String)>,
    Query(query): Query<EnvQuery>,
) -> Result<Json<Value>, ApiError> {
    project_role(&state.pool, project_id, claims.sub).await?;
    let target = target(&state, project_id).await?;
    let app = published_app(&target)?;
    let env = env_of(&query)?;
    let deployment = client(&state)?
        .get_deployment(&target.org_key(), app, &env, &deployment_id)
        .await
        .map_err(shipyard_error)?;
    Ok(Json(deployment))
}

#[derive(Deserialize)]
pub struct LogsQuery {
    pub environment: Option<String>,
    pub service: Option<String>,
    pub tail: Option<u32>,
    pub since: Option<String>,
}

/// `GET /api/projects/:id/deploy/logs`: what the running app printed.
pub async fn runtime_logs(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(project_id): Path<Uuid>,
    Query(query): Query<LogsQuery>,
) -> Result<Json<Value>, ApiError> {
    project_role(&state.pool, project_id, claims.sub).await?;
    let target = target(&state, project_id).await?;
    let app = published_app(&target)?;
    let env = env_of(&EnvQuery { environment: query.environment.clone() })?;
    let service = query.service.clone().unwrap_or_else(|| "web".to_string());
    let logs = client(&state)?
        .service_logs(&target.org_key(), app, &env, &service, query.tail.unwrap_or(300), query.since.as_deref())
        .await
        .map_err(shipyard_error)?;
    Ok(Json(logs))
}

/// `GET /api/projects/:id/deploy/usage`: CPU, memory, network and disk per container.
pub async fn usage(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(project_id): Path<Uuid>,
    Query(query): Query<EnvQuery>,
) -> Result<Json<Value>, ApiError> {
    project_role(&state.pool, project_id, claims.sub).await?;
    let target = target(&state, project_id).await?;
    let app = published_app(&target)?;
    let env = env_of(&query)?;
    let usage = client(&state)?.usage(&target.org_key(), app, &env).await.map_err(shipyard_error)?;
    Ok(Json(usage))
}

#[derive(Serialize, sqlx::FromRow)]
pub struct StoredEvent {
    pub id: String,
    #[serde(rename = "type")]
    #[sqlx(rename = "type")]
    pub kind: String,
    pub environment: Option<String>,
    pub service: Option<String>,
    pub data: Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct EventsQuery {
    pub limit: Option<i64>,
}

/// `GET /api/projects/:id/deploy/events`: what Shipyard reported, newest first.
pub async fn events(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(project_id): Path<Uuid>,
    Query(query): Query<EventsQuery>,
) -> Result<Json<Vec<StoredEvent>>, ApiError> {
    project_role(&state.pool, project_id, claims.sub).await?;
    let events = sqlx::query_as(
        "SELECT id, type, environment, service, data, created_at FROM shipyard_events \
         WHERE project_id = $1 ORDER BY created_at DESC LIMIT $2",
    )
    .bind(project_id)
    .bind(query.limit.unwrap_or(100).clamp(1, 500))
    .fetch_all(&state.pool)
    .await
    .map_err(internal("failed to list events"))?;
    Ok(Json(events))
}

/// `DELETE /api/projects/:id/deploy/environments/:env`: takes the environment offline, with
/// its database and data. The project and its publishes stay; publishing brings it back.
pub async fn take_offline(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path((project_id, environment)): Path<(Uuid, String)>,
) -> Result<StatusCode, ApiError> {
    require_admin(&state.pool, project_id, claims.sub).await?;
    valid_env(&environment)?;
    let target = target(&state, project_id).await?;
    let app = published_app(&target)?;
    client(&state)?.delete_env(&target.org_key(), app, &environment).await.map_err(shipyard_error)?;
    crate::routes::showcase::hide_when_offline(&state.pool, project_id, &environment).await;
    Ok(StatusCode::NO_CONTENT)
}

/// `DELETE /api/projects/:id`: terminates the project. Its Shipyard app goes everywhere (every
/// environment, data, the dev database), then its files and Nomi's record of it. The chat stays.
pub async fn delete_project(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(project_id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    require_admin(&state.pool, project_id, claims.sub).await?;
    let target = target(&state, project_id).await?;
    if let Some(app) = target.shipyard_app.as_deref() {
        match client(&state)?.delete_app(&target.org_key(), app).await {
            Ok(_) | Err(ShipyardError::NotFound(_)) => {}
            Err(e) => return Err(shipyard_error(e)),
        }
    }
    for prefix in [project_id.to_string(), format!("publishes/{project_id}")] {
        if let Err(e) = state.project_storage.delete_prefix(&prefix).await {
            tracing::warn!(error = %e, %prefix, "couldn't delete project storage");
        }
    }
    sqlx::query("DELETE FROM projects WHERE id = $1")
        .bind(project_id)
        .execute(&state.pool)
        .await
        .map_err(internal("failed to delete project"))?;
    Ok(StatusCode::NO_CONTENT)
}

/// `GET /api/projects/:id/ws`: the project's live updates (see `project_events`), for anyone
/// who can open the project.
pub async fn project_stream(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(project_id): Path<Uuid>,
    ws: axum::extract::ws::WebSocketUpgrade,
) -> Result<axum::response::Response, ApiError> {
    project_role(&state.pool, project_id, claims.sub).await?;
    let topic = crate::project_events::topic(project_id);
    let (host, port) = (state.mqtt_broker_host.clone(), state.mqtt_broker_port);
    Ok(ws.on_upgrade(move |socket| crate::routes::sessions::relay_topic(socket, topic, host, port)))
}

/// `POST /api/hooks/shipyard`: Shipyard's events. Signature first, then stored once and applied.
pub async fn webhook(State(state): State<AppState>, headers: HeaderMap, body: Bytes) -> StatusCode {
    let Some(config) = state.shipyard.as_ref() else { return StatusCode::NOT_FOUND };
    let secrets = nomi_shipyard_side::webhook_secrets(&state.pool, config, &state.settings_key).await;
    if secrets.is_empty() {
        tracing::warn!("a Shipyard webhook arrived but Nomi has no secret to check it with");
        return StatusCode::SERVICE_UNAVAILABLE;
    }
    let signature = headers.get("shipyard-signature").and_then(|v| v.to_str().ok()).unwrap_or_default();
    let now = Utc::now().timestamp();
    if !secrets.iter().any(|secret| nomi_shipyard::signature::verify(secret, signature, &body, now)) {
        tracing::warn!("a Shipyard webhook's signature didn't match any secret Nomi has");
        return StatusCode::UNAUTHORIZED;
    }
    let event: nomi_shipyard_side::Event = match serde_json::from_slice(&body) {
        Ok(event) => event,
        // Signed but not an event Nomi understands (e.g. `ping`): accepted, nothing to do.
        Err(_) => return StatusCode::NO_CONTENT,
    };
    // Stored before answering, so a failure here makes Shipyard deliver it again; what it
    // changes (and the notifications, which send email) happens after the reply.
    let project_id = match nomi_shipyard_side::store_event(&state.pool, &event).await {
        Ok(Some(project_id)) => project_id,
        Ok(None) => return StatusCode::NO_CONTENT,
        Err(e) => {
            tracing::error!(error = %e, event = %event.id, "failed to record a Shipyard event");
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
    };
    let pool = state.pool.clone();
    let http = state.http_client.clone();
    let config = config.clone();
    tokio::spawn(async move {
        let client = Shipyard::new(&http, &config);
        if let Err(e) = nomi_shipyard_side::apply_event(&pool, Some(&client), project_id, &event).await {
            tracing::error!(error = %e, event = %event.id, "failed to apply a Shipyard event");
        }
    });
    StatusCode::NO_CONTENT
}

#[derive(Serialize)]
pub struct AdminShipyard {
    pub configured: bool,
    pub can_reveal: bool,
    /// Nomi's endpoint on Shipyard, as Shipyard lists it.
    pub webhook_url: Option<String>,
    /// Whether Shipyard has an endpoint for Nomi; `None` when Shipyard couldn't be asked.
    pub webhook_registered: Option<bool>,
    /// Whether that endpoint is switched on.
    pub webhook_enabled: Option<bool>,
    /// Where "Register" would point Shipyard (`APP_URL` + /hooks/shipyard); `None` until the
    /// public address is set.
    pub webhook_target: Option<String>,
    /// SHIPYARD_WEBHOOK_SECRET is set (its endpoint was set up by hand).
    pub webhook_from_env: bool,
    pub registered_at: Option<DateTime<Utc>>,
    pub last_event_at: Option<DateTime<Utc>>,
}

async fn admin_status(state: &AppState) -> Result<AdminShipyard, ApiError> {
    let registered = nomi_shipyard_side::registered_webhook(&state.pool).await.map_err(internal("failed to read Shipyard settings"))?;
    let last_event_at: Option<DateTime<Utc>> = sqlx::query_scalar("SELECT max(received_at) FROM shipyard_events")
        .fetch_one(&state.pool)
        .await
        .map_err(internal("failed to read Shipyard settings"))?;
    // Shipyard's own list decides whether the endpoint exists: it may have been deleted there,
    // or never registered while SHIPYARD_WEBHOOK_SECRET was set.
    let listed = match client(state) {
        Ok(client) => match client.list_webhooks().await {
            Ok(endpoints) => Some(endpoints),
            Err(e) => {
                tracing::warn!(error = ?e, "couldn't list Shipyard's webhooks");
                None
            }
        },
        Err(_) => None,
    };
    let own = listed.as_ref().and_then(|endpoints| nomi_shipyard_side::own_endpoint(endpoints, registered.as_ref().map(|(id, _, _)| id.as_str())));
    Ok(AdminShipyard {
        configured: state.shipyard.is_some(),
        can_reveal: state.shipyard.as_ref().is_some_and(|c| c.reveal_key.is_some()),
        webhook_url: own.and_then(|e| e.get("url")).and_then(Value::as_str).map(str::to_string),
        webhook_registered: listed.as_ref().map(|_| own.is_some()),
        webhook_enabled: own.map(|e| e.get("enabled").and_then(Value::as_bool).unwrap_or(true)),
        webhook_target: crate::public_url::app_url().map(|url| format!("{url}{}", nomi_shipyard_side::WEBHOOK_PATH)),
        webhook_from_env: state.shipyard.as_ref().is_some_and(|c| c.webhook_secret.is_some()),
        registered_at: registered.map(|(_, _, at)| at),
        last_event_at,
    })
}

/// `GET /api/admin/shipyard`
pub async fn admin_get(State(state): State<AppState>, AuthClaims(claims): AuthClaims) -> Result<Json<AdminShipyard>, ApiError> {
    crate::routes::admin_access::require_admin(&claims, "integrations", "view").map_err(|(s, m)| (s, m.to_string()))?;
    Ok(Json(admin_status(&state).await?))
}

/// `POST /api/admin/shipyard/webhook`: registers Nomi's webhook endpoint at `APP_URL` again
/// (a new secret), then asks Shipyard to send it a test event.
pub async fn admin_register_webhook(State(state): State<AppState>, AuthClaims(claims): AuthClaims) -> Result<Json<AdminShipyard>, ApiError> {
    crate::routes::admin_access::require_admin(&claims, "integrations", "manage").map_err(|(s, m)| (s, m.to_string()))?;
    let client = client(&state)?;
    let app_url = crate::public_url::app_url()
        .ok_or((StatusCode::UNPROCESSABLE_ENTITY, "set APP_URL to Nomi's public address first".to_string()))?;
    nomi_shipyard_side::register_webhook(&state.pool, &client, &state.settings_key, &app_url).await.map_err(ensure_error)?;
    if let Ok(Some((id, _, _))) = nomi_shipyard_side::registered_webhook(&state.pool).await {
        if let Err(e) = client.test_webhook(&id).await {
            tracing::warn!(error = %e, "Shipyard's test event failed");
        }
    }
    Ok(Json(admin_status(&state).await?))
}
