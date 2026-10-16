//! Admin → Shipyard apps: every app Shipyard holds for Nomi's organizations, which Nomi
//! project each belongs to, and reconnecting the ones no project points at (after a restore,
//! or a project that was recreated) to an existing project or a new one.

use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use futures_util::stream::{self, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use nomi_auth::extractor::AuthClaims;

use crate::app::AppState;
use crate::routes::deploy::{client, internal, shipyard_error, ApiError};
use crate::routes::settings::require_system_config_permission;

fn admin(claims: &nomi_auth::claims::Claims) -> Result<(), ApiError> {
    require_system_config_permission(claims).map_err(|(s, m)| (s, m.to_string()))
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
    admin(&claims)?;
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
    admin(&claims)?;
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
