//! The showcase: apps built with Nomi that their owners chose to show. The page is public; a
//! project's admins turn it on or off (with one line about the app) from its settings. Only a
//! project that is live in production can be shown, and taking production offline takes it off.
//!
//! The address shown is the one Shipyard reports for the project's own production environment
//! when the showcase is turned on (kept in `showcase_url`), never a publish record's: a publish
//! records the address Shipyard named when it started, which isn't checked against the app.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use nomi_auth::extractor::AuthClaims;

use crate::app::AppState;
use crate::routes::deploy::{client, internal, project_role, require_admin, shipyard_error, ApiError};

/// The longest line about an app.
pub const BLURB_MAX: usize = 140;
/// How many apps the page shows.
const PAGE_SIZE: i64 = 60;

/// The address of the project's newest successful production publish, if it is still published.
const LIVE_URL: &str = "(SELECT pp.url FROM project_publishes pp \
     WHERE pp.project_id = p.id AND pp.environment = 'production' AND pp.status = 'succeeded' AND pp.url IS NOT NULL \
     ORDER BY pp.created_at DESC LIMIT 1)";

#[derive(Serialize, sqlx::FromRow)]
pub struct ShowcaseApp {
    pub id: Uuid,
    pub name: String,
    pub blurb: Option<String>,
    pub url: String,
    pub stack: String,
    /// The team that built it; left out for a personal workspace, which would name the person.
    pub team: Option<String>,
    pub showcased_at: DateTime<Utc>,
}

/// `GET /api/showcase`: the apps on show, newest first. No sign-in needed. An address is shown
/// once: for the project Shipyard confirmed it for, else the one that showed it first (entries
/// from before `showcase_url` fall back to their publish record, which could name another app).
pub async fn list(State(state): State<AppState>) -> Result<Json<Vec<ShowcaseApp>>, ApiError> {
    let apps: Vec<ShowcaseApp> = sqlx::query_as(&format!(
        "SELECT id, name, blurb, url, stack, team, showcased_at FROM ( \
             SELECT DISTINCT ON (url) * FROM ( \
                 SELECT p.id, p.name, p.showcase_blurb AS blurb, COALESCE(p.showcase_url, {LIVE_URL}) AS url, p.stack, \
                        CASE WHEN o.is_personal THEN NULL ELSE o.name END AS team, \
                        COALESCE(p.showcased_at, p.created_at) AS showcased_at, p.showcase_url IS NOT NULL AS checked \
                 FROM projects p JOIN organizations o ON o.id = p.org_id \
                 WHERE p.showcase AND p.shipyard_app IS NOT NULL \
             ) candidates WHERE url IS NOT NULL ORDER BY url, checked DESC, showcased_at \
         ) shown ORDER BY showcased_at DESC LIMIT {PAGE_SIZE}"
    ))
    .fetch_all(&state.pool)
    .await
    .map_err(internal("failed to load the showcase"))?;
    Ok(Json(apps))
}

#[derive(Serialize)]
pub struct ProjectShowcase {
    pub showcase: bool,
    pub blurb: Option<String>,
    /// Where it's live; it can only be shown while it is.
    pub url: Option<String>,
    pub can_edit: bool,
}

async fn read(state: &AppState, project_id: Uuid, can_edit: bool) -> Result<ProjectShowcase, ApiError> {
    let (showcase, blurb, url): (bool, Option<String>, Option<String>) = sqlx::query_as(&format!(
        "SELECT p.showcase, p.showcase_blurb, \
                CASE WHEN p.shipyard_app IS NULL THEN NULL WHEN p.showcase THEN COALESCE(p.showcase_url, {LIVE_URL}) ELSE {LIVE_URL} END \
         FROM projects p WHERE p.id = $1"
    ))
    .bind(project_id)
    .fetch_one(&state.pool)
    .await
    .map_err(internal("failed to read the showcase setting"))?;
    Ok(ProjectShowcase { showcase: showcase && url.is_some(), blurb, url, can_edit })
}

/// `GET /api/projects/:id/showcase`
pub async fn get(State(state): State<AppState>, AuthClaims(claims): AuthClaims, Path(project_id): Path<Uuid>) -> Result<Json<ProjectShowcase>, ApiError> {
    let role = project_role(&state.pool, project_id, claims.sub).await?;
    Ok(Json(read(&state, project_id, role == "admin").await?))
}

#[derive(Deserialize)]
pub struct ShowcaseRequest {
    pub showcase: bool,
    pub blurb: Option<String>,
}

/// `PUT /api/projects/:id/showcase`: the project's admins show it or take it off.
pub async fn put(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(project_id): Path<Uuid>,
    Json(req): Json<ShowcaseRequest>,
) -> Result<Json<ProjectShowcase>, ApiError> {
    require_admin(&state.pool, project_id, claims.sub).await?;
    let blurb = req.blurb.map(|b| b.split_whitespace().collect::<Vec<_>>().join(" ")).filter(|b| !b.is_empty());
    if blurb.as_ref().is_some_and(|b| b.chars().count() > BLURB_MAX) {
        return Err((StatusCode::BAD_REQUEST, format!("keep the description to {BLURB_MAX} characters")));
    }
    // Turning it on asks Shipyard where the project's own production answers, every time.
    let url = if req.showcase {
        if read(&state, project_id, true).await?.url.is_none() {
            return Err((StatusCode::CONFLICT, "publish the app first: only live apps can be shown".to_string()));
        }
        Some(verified_url(&state, project_id).await?)
    } else {
        None
    };
    sqlx::query(
        "UPDATE projects SET showcase = $2, showcase_blurb = $3, showcase_url = $4, \
         showcased_at = CASE WHEN $2 AND NOT showcase THEN now() WHEN $2 THEN showcased_at ELSE NULL END \
         WHERE id = $1",
    )
    .bind(project_id)
    .bind(req.showcase)
    .bind(&blurb)
    .bind(&url)
    .execute(&state.pool)
    .await
    .map_err(internal("failed to save the showcase setting"))?;
    Ok(Json(read(&state, project_id, true).await?))
}

/// The address the project's own production environment answers on, as Shipyard reports it now,
/// and not already shown for another project.
async fn verified_url(state: &AppState, project_id: Uuid) -> Result<String, ApiError> {
    let target = crate::shipyard::load_target(&state.pool, project_id)
        .await
        .map_err(internal("failed to load project"))?
        .ok_or((StatusCode::NOT_FOUND, "project not found".to_string()))?;
    let app = target.shipyard_app.as_deref().ok_or((StatusCode::CONFLICT, "publish the app first: only live apps can be shown".to_string()))?;
    let env = client(state)?.get_env(&target.org_key(), app, "production").await.map_err(shipyard_error)?;
    let url = live_web_url(&env).ok_or((StatusCode::CONFLICT, "Shipyard says the app isn't live in production right now".to_string()))?;
    let other: Option<String> = sqlx::query_scalar("SELECT name FROM projects WHERE showcase AND id <> $1 AND showcase_url = $2")
        .bind(project_id)
        .bind(&url)
        .fetch_optional(&state.pool)
        .await
        .map_err(internal("failed to check the showcase"))?;
    if let Some(other) = other {
        return Err((StatusCode::CONFLICT, format!("{url} is already shown for {other}")));
    }
    // Shipyard has been seen reporting another app's domain for an environment. When the address
    // isn't the one this project's own publish named, and another project's publish named it,
    // it belongs to that project.
    let owner: Option<String> = sqlx::query_scalar(&format!(
        "SELECT o.name FROM projects o WHERE o.id <> $1 AND o.shipyard_app IS NOT NULL \
           AND (SELECT pp.url FROM project_publishes pp WHERE pp.project_id = o.id AND pp.environment = 'production' \
                AND pp.status = 'succeeded' AND pp.url IS NOT NULL ORDER BY pp.created_at DESC LIMIT 1) = $2 \
           AND (SELECT {own} FROM projects p WHERE p.id = $1) IS DISTINCT FROM $2 \
         LIMIT 1",
        own = LIVE_URL
    ))
    .bind(project_id)
    .bind(&url)
    .fetch_optional(&state.pool)
    .await
    .map_err(internal("failed to check the showcase"))?;
    if let Some(owner) = owner {
        return Err((
            StatusCode::CONFLICT,
            format!("Shipyard reports {url} for this app, but that's {owner}'s address. Check the app's domains in Shipyard, then try again."),
        ));
    }
    Ok(url)
}

/// The web address of an environment from Shipyard's `GET .../envs/{env}`: the first service
/// with a URL, preferring one that's live (the database has none).
pub fn live_web_url(env: &serde_json::Value) -> Option<String> {
    let services = env.get("services")?.as_array()?;
    let url_of = |s: &serde_json::Value| s.get("urls").and_then(|u| u.as_array()).and_then(|u| u.first()).and_then(|u| u.as_str()).map(str::to_string);
    services
        .iter()
        .find(|s| s.get("live").and_then(|l| l.as_bool()) == Some(true) && url_of(s).is_some())
        .or_else(|| services.iter().find(|s| url_of(s).is_some()))
        .and_then(url_of)
}

/// Production went offline: the app leaves the showcase (its owners can show it again once it's
/// published).
pub async fn hide_when_offline(pool: &sqlx::PgPool, project_id: Uuid, environment: &str) {
    if environment != "production" {
        return;
    }
    if let Err(e) = sqlx::query("UPDATE projects SET showcase = false, showcased_at = NULL, showcase_url = NULL WHERE id = $1").bind(project_id).execute(pool).await {
        tracing::warn!(error = %e, %project_id, "couldn't take an offline app off the showcase");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn the_web_address_comes_from_the_live_service() {
        let env = json!({ "services": [
            { "name": "db", "type": "database", "urls": [] },
            { "name": "old", "urls": ["https://old.example"], "live": false },
            { "name": "web", "urls": ["https://todo.apps.example", "https://todo.example.com"], "live": true },
        ] });
        assert_eq!(live_web_url(&env).as_deref(), Some("https://todo.apps.example"));
        assert_eq!(live_web_url(&json!({ "services": [{ "name": "web", "urls": ["https://a.example"] }] })).as_deref(), Some("https://a.example"));
        assert_eq!(live_web_url(&json!({ "services": [{ "name": "db", "urls": [] }] })), None);
        assert_eq!(live_web_url(&json!({})), None);
    }
}
