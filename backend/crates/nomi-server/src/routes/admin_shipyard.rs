//! Admin → Shipyard apps: every app Shipyard holds for Nomi's organizations, which Nomi
//! project each belongs to, and reconnecting the ones no project points at (after a restore,
//! or a project that was recreated) to an existing project or a new one.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use futures_util::stream::{self, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use nomi_auth::extractor::AuthClaims;

use crate::app::AppState;
use crate::routes::deploy::{client, internal, shipyard_error, ApiError};

fn admin(claims: &nomi_auth::claims::Claims, level: &str) -> Result<(), ApiError> {
    crate::routes::admin_access::require_admin_owned(claims, "integrations", level)
}

#[derive(Serialize)]
pub struct LinkedProject {
    pub id: Uuid,
    pub name: String,
    pub session_id: Uuid,
}

#[derive(Serialize)]
pub struct ShipyardApp {
    pub org_id: Uuid,
    pub org_name: String,
    pub key: String,
    pub name: String,
    pub environments: Vec<String>,
    /// The Nomi project it belongs to; null when none does.
    pub project: Option<LinkedProject>,
}

#[derive(Serialize)]
pub struct UnlinkedProject {
    pub id: Uuid,
    pub name: String,
}

#[derive(Serialize)]
pub struct OrgProjects {
    pub org_id: Uuid,
    /// Projects in the organization not yet tied to a Shipyard app, to reconnect to.
    pub projects: Vec<UnlinkedProject>,
}

#[derive(Serialize)]
pub struct AppsOverview {
    pub apps: Vec<ShipyardApp>,
    pub candidates: Vec<OrgProjects>,
    /// Organizations Shipyard couldn't be asked about.
    pub errors: Vec<String>,
}

/// `GET /api/admin/shipyard/apps`
pub async fn list_apps(State(state): State<AppState>, AuthClaims(claims): AuthClaims) -> Result<Json<AppsOverview>, ApiError> {
    admin(&claims, "view")?;
    let client = client(&state)?;
    // Organizations that were ever sent to Shipyard, or have projects that might have been.
    let orgs: Vec<(Uuid, String)> = sqlx::query_as(
        "SELECT o.id, o.name FROM organizations o WHERE o.shipyard_synced_at IS NOT NULL \
            OR EXISTS (SELECT 1 FROM projects p WHERE p.org_id = o.id) ORDER BY o.name LIMIT 500",
    )
    .fetch_all(&state.pool)
    .await
    .map_err(internal("failed to list organizations"))?;

    let fetched: Vec<(Uuid, String, Result<Value, nomi_shipyard::ShipyardError>)> = stream::iter(orgs)
        .map(|(id, name)| {
            let client = &client;
            async move { (id, name, client.get_org(&id.to_string()).await) }
        })
        .buffer_unordered(8)
        .collect()
        .await;

    let linked: Vec<(Uuid, String, Uuid, String)> =
        sqlx::query_as("SELECT id, name, session_id, shipyard_app FROM projects WHERE shipyard_app IS NOT NULL")
            .fetch_all(&state.pool)
            .await
            .map_err(internal("failed to list projects"))?;

    let mut apps = Vec::new();
    let mut errors = Vec::new();
    let mut orgs_with_apps = Vec::new();
    for (org_id, org_name, result) in fetched {
        let data = match result {
            Ok(data) => data,
            Err(nomi_shipyard::ShipyardError::NotFound(_)) => continue,
            Err(e) => {
                errors.push(format!("{org_name}: {e}"));
                continue;
            }
        };
        for app in data.get("apps").and_then(Value::as_array).cloned().unwrap_or_default() {
            let key = app.get("key").and_then(Value::as_str).unwrap_or_default().to_string();
            if key.is_empty() {
                continue;
            }
            let project = linked
                .iter()
                .find(|(_, _, _, k)| *k == key)
                .map(|(id, name, session_id, _)| LinkedProject { id: *id, name: name.clone(), session_id: *session_id });
            apps.push(ShipyardApp {
                org_id,
                org_name: org_name.clone(),
                name: app.get("name").and_then(Value::as_str).unwrap_or(&key).to_string(),
                environments: app
                    .get("environments")
                    .and_then(Value::as_array)
                    .map(|e| e.iter().filter_map(Value::as_str).map(str::to_string).collect())
                    .unwrap_or_default(),
                key,
                project,
            });
        }
        orgs_with_apps.push(org_id);
    }
    // Orphans first, then by organization.
    apps.sort_by(|a, b| a.project.is_some().cmp(&b.project.is_some()).then(a.org_name.cmp(&b.org_name)).then(a.key.cmp(&b.key)));

    let mut candidates = Vec::new();
    for org_id in orgs_with_apps {
        let projects: Vec<(Uuid, String)> =
            sqlx::query_as("SELECT id, name FROM projects WHERE org_id = $1 AND shipyard_app IS NULL ORDER BY updated_at DESC")
                .bind(org_id)
                .fetch_all(&state.pool)
                .await
                .map_err(internal("failed to list projects"))?;
        candidates.push(OrgProjects { org_id, projects: projects.into_iter().map(|(id, name)| UnlinkedProject { id, name }).collect() });
    }
    Ok(Json(AppsOverview { apps, candidates, errors }))
}

#[derive(Deserialize)]
pub struct LinkRequest {
    pub org_id: Uuid,
    pub app: String,
    /// The project to tie it to; a new project (owned by the organization's owner) when left out.
    pub project_id: Option<Uuid>,
}

/// `POST /api/admin/shipyard/apps/link`: ties a Shipyard app to a Nomi project.
pub async fn link_app(State(state): State<AppState>, AuthClaims(claims): AuthClaims, Json(req): Json<LinkRequest>) -> Result<Json<LinkedProject>, ApiError> {
    admin(&claims, "write")?;
    let client = client(&state)?;
    // Confirm the app really is that organization's on Shipyard.
    let org = client.get_org(&req.org_id.to_string()).await.map_err(shipyard_error)?;
    let app = org
        .get("apps")
        .and_then(Value::as_array)
        .and_then(|apps| apps.iter().find(|a| a.get("key").and_then(Value::as_str) == Some(req.app.as_str())))
        .cloned()
        .ok_or((StatusCode::NOT_FOUND, "that app isn't in this organization on Shipyard".to_string()))?;
    let taken: Option<Uuid> = sqlx::query_scalar("SELECT id FROM projects WHERE shipyard_app = $1")
        .bind(&req.app)
        .fetch_optional(&state.pool)
        .await
        .map_err(internal("failed to link the app"))?;
    if taken.is_some_and(|id| Some(id) != req.project_id) {
        return Err((StatusCode::CONFLICT, "another project already uses that app".to_string()));
    }

    let project: (Uuid, String, Uuid) = match req.project_id {
        Some(project_id) => sqlx::query_as(
            "UPDATE projects SET shipyard_app = $3 WHERE id = $1 AND org_id = $2 AND (shipyard_app IS NULL OR shipyard_app = $3) \
             RETURNING id, name, session_id",
        )
        .bind(project_id)
        .bind(req.org_id)
        .bind(&req.app)
        .fetch_optional(&state.pool)
        .await
        .map_err(internal("failed to link the app"))?
        .ok_or((StatusCode::NOT_FOUND, "that project isn't in this organization, or already has an app".to_string()))?,
        None => {
            // A new project for the app, owned by the organization's (first) owner, with its own chat.
            let owner: Uuid = sqlx::query_scalar(
                "SELECT user_id FROM memberships WHERE org_id = $1 AND status = 'active' \
                 ORDER BY CASE role WHEN 'owner' THEN 0 WHEN 'admin' THEN 1 ELSE 2 END, created_at LIMIT 1",
            )
            .bind(req.org_id)
            .fetch_optional(&state.pool)
            .await
            .map_err(internal("failed to link the app"))?
            .ok_or((StatusCode::UNPROCESSABLE_ENTITY, "that organization has no members to own the project".to_string()))?;
            nomi_turn::bootstrap::ensure_web_channel_identity(&state.pool, owner).await.map_err(internal("failed to link the app"))?;
            let session = nomi_turn::bootstrap::bootstrap_identity_and_session(
                &state.pool,
                "web",
                "dm",
                &Uuid::new_v4().to_string(),
                &owner.to_string(),
                Some(req.org_id),
            )
            .await
            .map_err(internal("failed to link the app"))?;
            let name = app.get("name").and_then(Value::as_str).unwrap_or(&req.app).to_string();
            sqlx::query_as(
                "INSERT INTO projects (user_id, session_id, org_id, name, status, shipyard_app) VALUES ($1, $2, $3, $4, 'ready', $5) \
                 RETURNING id, name, session_id",
            )
            .bind(owner)
            .bind(session.session_id)
            .bind(req.org_id)
            .bind(name)
            .bind(&req.app)
            .fetch_one(&state.pool)
            .await
            .map_err(internal("failed to link the app"))?
        }
    };
    Ok(Json(LinkedProject { id: project.0, name: project.1, session_id: project.2 }))
}

// ─── Webhook endpoints ───────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct Webhooks {
    pub endpoints: Value,
    /// Nomi's own endpoint: the one it registered (its secret is kept by Nomi, never shown), or
    /// one pointing at its `/hooks/shipyard` address.
    pub own_id: Option<String>,
}

/// `GET /api/admin/shipyard/webhooks`: every endpoint the integration has on Shipyard.
pub async fn list_webhooks(State(state): State<AppState>, AuthClaims(claims): AuthClaims) -> Result<Json<Webhooks>, ApiError> {
    admin(&claims, "view")?;
    let endpoints = client(&state)?.list_webhooks().await.map_err(shipyard_error)?;
    let registered = crate::shipyard::registered_webhook(&state.pool).await.map_err(internal("failed to read Shipyard settings"))?.map(|(id, _, _)| id);
    let own_id = crate::shipyard::own_endpoint(&endpoints, registered.as_deref()).and_then(|e| e.get("id")).and_then(Value::as_str).map(str::to_string);
    Ok(Json(Webhooks { endpoints, own_id }))
}

#[derive(Deserialize)]
pub struct NewWebhook {
    pub url: String,
    /// Exact types (`publish.succeeded`), prefixes (`deployment.*`) or `*`.
    pub events: Vec<String>,
    /// Only this Nomi organization's events.
    pub org: Option<Uuid>,
}

#[derive(Serialize)]
pub struct CreatedWebhook {
    pub id: String,
    /// The signing secret, shown once, for an endpoint someone else runs; `None` for Nomi's own
    /// (Nomi keeps that one to check what arrives).
    pub secret: Option<String>,
    pub own: bool,
}

/// An event filter Shipyard accepts: `*`, or dotted lowercase names ending in a name or `*`.
fn valid_event(event: &str) -> bool {
    if event == "*" {
        return true;
    }
    let parts: Vec<&str> = event.split('.').collect();
    let last = parts.len() - 1;
    parts.len() >= 2
        && parts.iter().enumerate().all(|(i, part)| {
            (i == last && *part == "*") || (!part.is_empty() && part.bytes().all(|b| b.is_ascii_lowercase() || b == b'_'))
        })
}

/// A web address Shipyard can deliver to: https, or http to this machine while developing.
fn valid_endpoint(url: &str) -> bool {
    match reqwest::Url::parse(url) {
        Ok(url) => match url.scheme() {
            "https" => url.host_str().is_some(),
            "http" => matches!(url.host_str(), Some("localhost" | "127.0.0.1")),
            _ => false,
        },
        Err(_) => false,
    }
}

/// `POST /api/admin/shipyard/webhooks`: registers an endpoint. Nomi's own address
/// (`APP_URL`/hooks/shipyard) keeps its secret here; any other gets it back once.
pub async fn create_webhook(State(state): State<AppState>, AuthClaims(claims): AuthClaims, Json(req): Json<NewWebhook>) -> Result<Json<CreatedWebhook>, ApiError> {
    admin(&claims, "manage")?;
    let url = req.url.trim().to_string();
    if !valid_endpoint(&url) {
        return Err((StatusCode::BAD_REQUEST, "use a public https address".to_string()));
    }
    let mut events: Vec<String> = req.events.iter().map(|e| e.trim().to_string()).filter(|e| !e.is_empty()).collect();
    events.sort();
    events.dedup();
    if events.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "pick at least one event".to_string()));
    }
    if let Some(bad) = events.iter().find(|e| !valid_event(e)) {
        return Err((StatusCode::BAD_REQUEST, format!("'{bad}' isn't an event type, a prefix like deployment.* or *")));
    }
    if events.iter().any(|e| e == "*") {
        events = vec!["*".to_string()];
    }
    let client = client(&state)?;
    let own = crate::public_url::app_url().is_some_and(|base| url.trim_end_matches('/') == format!("{base}{}", crate::shipyard::WEBHOOK_PATH));
    if own {
        // A second endpoint at Nomi's address would sign with a secret Nomi doesn't keep.
        let endpoints = client.list_webhooks().await.map_err(shipyard_error)?;
        let registered = crate::shipyard::registered_webhook(&state.pool).await.map_err(internal("failed to read Shipyard settings"))?.map(|(id, _, _)| id);
        if crate::shipyard::own_endpoint(&endpoints, registered.as_deref()).is_some() {
            return Err((StatusCode::CONFLICT, "Nomi's endpoint is already registered: give it a new secret or delete it first".to_string()));
        }
    }
    let (id, secret) = client.create_webhook_for(&url, &events, req.org.map(|o| o.to_string()).as_deref()).await.map_err(shipyard_error)?;
    if own {
        crate::shipyard::keep_webhook(&state.pool, &state.settings_key, &id, &url, &secret).await.map_err(internal("failed to keep the secret"))?;
        if let Err(e) = client.test_webhook(&id).await {
            tracing::warn!(error = ?e, "Shipyard's test event failed");
        }
        return Ok(Json(CreatedWebhook { id, secret: None, own }));
    }
    Ok(Json(CreatedWebhook { id, secret: Some(secret), own }))
}

#[cfg(test)]
mod webhook_tests {
    use super::*;

    #[test]
    fn event_filters_and_endpoints() {
        for ok in ["*", "deployment.*", "publish.succeeded", "deployment.step_failed"] {
            assert!(valid_event(ok), "{ok}");
        }
        for bad in ["", "deployment", "Deployment.*", "a..b", "*.x", "publish.succeeded;drop"] {
            assert!(!valid_event(bad), "{bad}");
        }
        assert!(valid_endpoint("https://nomi.example.com/hooks/shipyard"));
        assert!(valid_endpoint("http://localhost:3000/hooks/shipyard"));
        assert!(!valid_endpoint("http://nomi.example.com/hooks/shipyard"));
        assert!(!valid_endpoint("ftp://x"));
        assert!(!valid_endpoint("not a url"));
    }
}

#[derive(Deserialize)]
pub struct WebhookPatch {
    pub enabled: Option<bool>,
    pub events: Option<Vec<String>>,
}

/// `PATCH /api/admin/shipyard/webhooks/:id`: on or off, or which events it gets.
pub async fn update_webhook(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(id): Path<String>,
    Json(req): Json<WebhookPatch>,
) -> Result<Json<Value>, ApiError> {
    admin(&claims, "write")?;
    let mut patch = serde_json::Map::new();
    if let Some(enabled) = req.enabled {
        patch.insert("enabled".into(), Value::Bool(enabled));
    }
    if let Some(events) = req.events {
        if events.is_empty() {
            return Err((StatusCode::BAD_REQUEST, "pick at least one event".to_string()));
        }
        patch.insert("events".into(), serde_json::json!(events));
    }
    Ok(Json(client(&state)?.update_webhook(&id, &Value::Object(patch)).await.map_err(shipyard_error)?))
}

/// `DELETE /api/admin/shipyard/webhooks/:id`
pub async fn delete_webhook(State(state): State<AppState>, AuthClaims(claims): AuthClaims, Path(id): Path<String>) -> Result<StatusCode, ApiError> {
    admin(&claims, "manage")?;
    client(&state)?.delete_webhook(&id).await.map_err(shipyard_error)?;
    sqlx::query("DELETE FROM shipyard_webhook WHERE endpoint_id = $1")
        .bind(&id)
        .execute(&state.pool)
        .await
        .map_err(internal("failed to forget the webhook"))?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Serialize)]
pub struct Rotated {
    /// The new secret, shown once — except for Nomi's own endpoint, whose secret Nomi keeps.
    pub secret: Option<String>,
}

/// `POST /api/admin/shipyard/webhooks/:id/rotate`: a new signing secret.
pub async fn rotate_webhook(State(state): State<AppState>, AuthClaims(claims): AuthClaims, Path(id): Path<String>) -> Result<Json<Rotated>, ApiError> {
    admin(&claims, "manage")?;
    let client = client(&state)?;
    // Nomi's own endpoint keeps its new secret (and from then on is Nomi's, even one set up by
    // hand); any other endpoint's secret is shown once to whoever runs it.
    let endpoints = client.list_webhooks().await.map_err(shipyard_error)?;
    let registered = crate::shipyard::registered_webhook(&state.pool).await.map_err(internal("failed to read Shipyard settings"))?.map(|(id, _, _)| id);
    let own = crate::shipyard::own_endpoint(&endpoints, registered.as_deref())
        .filter(|e| e.get("id").and_then(Value::as_str) == Some(id.as_str()))
        .and_then(|e| e.get("url"))
        .and_then(Value::as_str)
        .map(str::to_string);
    let secret = client.rotate_webhook_secret(&id).await.map_err(shipyard_error)?;
    if let Some(url) = own {
        crate::shipyard::keep_webhook(&state.pool, &state.settings_key, &id, &url, &secret).await.map_err(internal("failed to keep the new secret"))?;
        return Ok(Json(Rotated { secret: None }));
    }
    Ok(Json(Rotated { secret: Some(secret) }))
}

/// `POST /api/admin/shipyard/webhooks/:id/test`: Shipyard sends it a signed ping.
pub async fn test_webhook(State(state): State<AppState>, AuthClaims(claims): AuthClaims, Path(id): Path<String>) -> Result<Json<Value>, ApiError> {
    admin(&claims, "write")?;
    Ok(Json(client(&state)?.test_webhook(&id).await.map_err(shipyard_error)?))
}

#[derive(Deserialize)]
pub struct DeliveriesQuery {
    pub status: Option<String>,
}

/// `GET /api/admin/shipyard/webhooks/:id/deliveries`
pub async fn webhook_deliveries(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(id): Path<String>,
    Query(q): Query<DeliveriesQuery>,
) -> Result<Json<Value>, ApiError> {
    admin(&claims, "view")?;
    let status = q.status.filter(|s| !s.is_empty());
    Ok(Json(client(&state)?.webhook_deliveries(&id, status.as_deref(), 50).await.map_err(shipyard_error)?))
}

/// `POST /api/admin/shipyard/deliveries/:id/redeliver`
pub async fn redeliver(State(state): State<AppState>, AuthClaims(claims): AuthClaims, Path(id): Path<String>) -> Result<Json<Value>, ApiError> {
    admin(&claims, "write")?;
    Ok(Json(client(&state)?.redeliver(&id).await.map_err(shipyard_error)?))
}

// ─── Organizations ───────────────────────────────────────────────────────────

#[derive(Serialize, sqlx::FromRow)]
pub struct SyncedOrg {
    pub id: Uuid,
    pub name: String,
    pub synced_at: Option<chrono::DateTime<chrono::Utc>>,
    pub projects: i64,
    pub published: i64,
}

/// `GET /api/admin/shipyard/orgs`: Nomi's organizations and whether each is on Shipyard.
pub async fn list_orgs(State(state): State<AppState>, AuthClaims(claims): AuthClaims) -> Result<Json<Vec<SyncedOrg>>, ApiError> {
    admin(&claims, "view")?;
    let orgs = sqlx::query_as(
        "SELECT o.id, o.name, o.shipyard_synced_at AS synced_at, \
                (SELECT COUNT(*) FROM projects p WHERE p.org_id = o.id) AS projects, \
                (SELECT COUNT(*) FROM projects p WHERE p.org_id = o.id AND p.shipyard_app IS NOT NULL) AS published \
         FROM organizations o \
         WHERE o.shipyard_synced_at IS NOT NULL OR EXISTS (SELECT 1 FROM projects p WHERE p.org_id = o.id) \
         ORDER BY o.shipyard_synced_at IS NULL, o.name LIMIT 500",
    )
    .fetch_all(&state.pool)
    .await
    .map_err(internal("failed to list organizations"))?;
    Ok(Json(orgs))
}

/// `POST /api/admin/shipyard/orgs/:id/sync`: creates the organization on Shipyard, or brings its name up to date.
pub async fn sync_org(State(state): State<AppState>, AuthClaims(claims): AuthClaims, Path(org_id): Path<Uuid>) -> Result<StatusCode, ApiError> {
    admin(&claims, "write")?;
    let client = client(&state)?;
    let name: String = sqlx::query_scalar("SELECT name FROM organizations WHERE id = $1")
        .bind(org_id)
        .fetch_optional(&state.pool)
        .await
        .map_err(internal("failed to load organization"))?
        .ok_or((StatusCode::NOT_FOUND, "organization not found".to_string()))?;
    client.put_org(&org_id.to_string(), &name).await.map_err(shipyard_error)?;
    sqlx::query("UPDATE organizations SET shipyard_synced_at = now() WHERE id = $1")
        .bind(org_id)
        .execute(&state.pool)
        .await
        .map_err(internal("failed to save organization"))?;
    Ok(StatusCode::NO_CONTENT)
}

/// `DELETE /api/admin/shipyard/orgs/:id`: removes the organization from Shipyard with every app
/// it has there. The Nomi organization and its projects stay; they just aren't published.
pub async fn remove_org(State(state): State<AppState>, AuthClaims(claims): AuthClaims, Path(org_id): Path<Uuid>) -> Result<StatusCode, ApiError> {
    admin(&claims, "manage")?;
    match client(&state)?.delete_org(&org_id.to_string()).await {
        Ok(_) | Err(nomi_shipyard::ShipyardError::NotFound(_)) => {}
        Err(e) => return Err(shipyard_error(e)),
    }
    let mut tx = state.pool.begin().await.map_err(internal("failed to update organization"))?;
    sqlx::query("UPDATE projects SET shipyard_app = NULL WHERE org_id = $1")
        .bind(org_id)
        .execute(&mut *tx)
        .await
        .map_err(internal("failed to update organization"))?;
    sqlx::query("UPDATE organizations SET shipyard_synced_at = NULL WHERE id = $1")
        .bind(org_id)
        .execute(&mut *tx)
        .await
        .map_err(internal("failed to update organization"))?;
    sqlx::query("DELETE FROM shipyard_event_cursors WHERE org_id = $1")
        .bind(org_id)
        .execute(&mut *tx)
        .await
        .map_err(internal("failed to update organization"))?;
    tx.commit().await.map_err(internal("failed to update organization"))?;
    Ok(StatusCode::NO_CONTENT)
}

// ─── Apps, environments, secrets ─────────────────────────────────────────────

/// `GET /api/admin/shipyard/orgs/:org/apps/:app`: its environments, services and dev database.
pub async fn get_app(State(state): State<AppState>, AuthClaims(claims): AuthClaims, Path((org, app)): Path<(Uuid, String)>) -> Result<Json<Value>, ApiError> {
    admin(&claims, "view")?;
    Ok(Json(client(&state)?.get_app(&org.to_string(), &app).await.map_err(shipyard_error)?))
}

/// `DELETE /api/admin/shipyard/orgs/:org/apps/:app`: terminates the app everywhere.
pub async fn delete_app(State(state): State<AppState>, AuthClaims(claims): AuthClaims, Path((org, app)): Path<(Uuid, String)>) -> Result<StatusCode, ApiError> {
    admin(&claims, "manage")?;
    match client(&state)?.delete_app(&org.to_string(), &app).await {
        Ok(_) | Err(nomi_shipyard::ShipyardError::NotFound(_)) => {}
        Err(e) => return Err(shipyard_error(e)),
    }
    sqlx::query("UPDATE projects SET shipyard_app = NULL WHERE org_id = $1 AND shipyard_app = $2")
        .bind(org)
        .bind(&app)
        .execute(&state.pool)
        .await
        .map_err(internal("failed to update project"))?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
pub struct DuplicateRequest {
    pub to: String,
    #[serde(default = "yes")]
    pub copy_secrets: bool,
}

fn yes() -> bool {
    true
}

/// Environment names: what Shipyard accepts in a URL.
fn valid_env(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 40
        && name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        && !name.starts_with('-')
        && !name.ends_with('-')
}

/// `POST /api/admin/shipyard/orgs/:org/apps/:app/envs/:env/duplicate`: a staging or preview
/// copy of the environment, with its own empty database, secrets and URL.
pub async fn duplicate_env(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path((org, app, env)): Path<(Uuid, String, String)>,
    Json(req): Json<DuplicateRequest>,
) -> Result<Json<Value>, ApiError> {
    admin(&claims, "write")?;
    let to = req.to.trim().to_lowercase();
    if !valid_env(&to) {
        return Err((StatusCode::BAD_REQUEST, "use lowercase letters, numbers and dashes for the environment name".to_string()));
    }
    Ok(Json(client(&state)?.duplicate_env(&org.to_string(), &app, &env, &to, req.copy_secrets).await.map_err(shipyard_error)?))
}

/// `DELETE /api/admin/shipyard/orgs/:org/apps/:app/envs/:env`: takes one environment down.
pub async fn delete_env(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path((org, app, env)): Path<(Uuid, String, String)>,
) -> Result<StatusCode, ApiError> {
    admin(&claims, "manage")?;
    client(&state)?.delete_env(&org.to_string(), &app, &env).await.map_err(shipyard_error)?;
    let project: Option<Uuid> = sqlx::query_scalar("SELECT id FROM projects WHERE shipyard_app = $1").bind(&app).fetch_optional(&state.pool).await.ok().flatten();
    if let Some(project_id) = project {
        crate::routes::showcase::hide_when_offline(&state.pool, project_id, &env).await;
    }
    Ok(StatusCode::NO_CONTENT)
}

/// `GET /api/admin/shipyard/orgs/:org/apps/:app/secrets`: names, refs and where they're used. Never values.
pub async fn list_secrets(State(state): State<AppState>, AuthClaims(claims): AuthClaims, Path((org, app)): Path<(Uuid, String)>) -> Result<Json<Value>, ApiError> {
    admin(&claims, "view")?;
    Ok(Json(client(&state)?.list_secrets(&org.to_string(), &app).await.map_err(shipyard_error)?))
}

#[derive(Deserialize)]
pub struct SecretRequest {
    /// Left out to have Shipyard generate one.
    pub value: Option<String>,
    /// Left out for every environment.
    pub environment: Option<String>,
}

/// Secret names as environment variables: `A-Z0-9_`, not starting with a digit.
fn valid_secret_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 128
        && name.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
        && !name.starts_with(|c: char| c.is_ascii_digit())
}

/// `PUT /api/admin/shipyard/orgs/:org/apps/:app/secrets/:name`: sets a secret. The value goes
/// straight to Shipyard; Nomi doesn't keep it and no agent sees it.
pub async fn set_secret(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path((org, app, name)): Path<(Uuid, String, String)>,
    Json(req): Json<SecretRequest>,
) -> Result<Json<Value>, ApiError> {
    admin(&claims, "manage")?;
    if !valid_secret_name(&name) {
        return Err((StatusCode::BAD_REQUEST, "secret names use A-Z, 0-9 and _, like an environment variable".to_string()));
    }
    let value = req.value.filter(|v| !v.is_empty());
    let environment = req.environment.map(|e| e.trim().to_string()).filter(|e| !e.is_empty());
    let saved = client(&state)?
        .set_secret(&org.to_string(), &app, &name, value.as_deref(), environment.as_deref())
        .await
        .map_err(shipyard_error)?;
    Ok(Json(saved))
}

#[derive(Deserialize)]
pub struct SecretScope {
    pub environment: Option<String>,
}

/// `DELETE /api/admin/shipyard/orgs/:org/apps/:app/secrets/:name`
pub async fn delete_secret(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path((org, app, name)): Path<(Uuid, String, String)>,
    Query(scope): Query<SecretScope>,
) -> Result<StatusCode, ApiError> {
    admin(&claims, "manage")?;
    let environment = scope.environment.filter(|e| !e.is_empty());
    client(&state)?.delete_secret(&org.to_string(), &app, &name, environment.as_deref()).await.map_err(shipyard_error)?;
    Ok(StatusCode::NO_CONTENT)
}

// ─── Events ──────────────────────────────────────────────────────────────────

#[derive(Serialize, sqlx::FromRow)]
pub struct ReceivedEvent {
    pub id: String,
    #[sqlx(rename = "type")]
    #[serde(rename = "type")]
    pub kind: String,
    pub project_id: Option<Uuid>,
    pub project_name: Option<String>,
    pub environment: Option<String>,
    pub service: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub received_at: chrono::DateTime<chrono::Utc>,
}

/// How many events a page of the feed holds.
const EVENTS_PER_PAGE: i64 = 20;

#[derive(Serialize)]
pub struct EventFeed {
    pub events: Vec<ReceivedEvent>,
    pub page: i64,
    pub pages: i64,
    pub total: i64,
    /// What Shipyard can send.
    pub types: Value,
}

#[derive(Deserialize)]
pub struct EventPage {
    pub page: Option<i64>,
}

/// `GET /api/admin/shipyard/events?page=`: the events Nomi received, newest first, a page at a time.
pub async fn list_events(State(state): State<AppState>, AuthClaims(claims): AuthClaims, Query(q): Query<EventPage>) -> Result<Json<EventFeed>, ApiError> {
    admin(&claims, "view")?;
    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM shipyard_events").fetch_one(&state.pool).await.map_err(internal("failed to count events"))?;
    let pages = ((total + EVENTS_PER_PAGE - 1) / EVENTS_PER_PAGE).max(1);
    let page = q.page.unwrap_or(1).clamp(1, pages);
    let events = sqlx::query_as(
        "SELECT e.id, e.type, e.project_id, p.name AS project_name, e.environment, e.service, e.created_at, e.received_at \
         FROM shipyard_events e LEFT JOIN projects p ON p.id = e.project_id ORDER BY e.received_at DESC, e.id LIMIT $1 OFFSET $2",
    )
    .bind(EVENTS_PER_PAGE)
    .bind((page - 1) * EVENTS_PER_PAGE)
    .fetch_all(&state.pool)
    .await
    .map_err(internal("failed to list events"))?;
    let types = match &state.shipyard {
        Some(_) => client(&state)?.event_types().await.unwrap_or(Value::Array(vec![])),
        None => Value::Array(vec![]),
    };
    Ok(Json(EventFeed { events, page, pages, total, types }))
}

/// `POST /api/admin/shipyard/events/catch-up`: pulls events no webhook delivered, now.
pub async fn catch_up(State(state): State<AppState>, AuthClaims(claims): AuthClaims) -> Result<Json<crate::shipyard::CatchUp>, ApiError> {
    admin(&claims, "write")?;
    let client = client(&state)?;
    Ok(Json(crate::shipyard::catch_up(&state.pool, &client).await.map_err(internal("failed to catch up on events"))?))
}
