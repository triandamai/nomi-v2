//! The showcase: apps built with Nomi that their owners chose to show. The page is public; a
//! project's admins turn it on or off (with one line about the app) from its settings. Only a
//! project that is live in production can be shown, and taking production offline takes it off.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use nomi_auth::extractor::AuthClaims;

use crate::app::AppState;
use crate::routes::deploy::{internal, project_role, require_admin, ApiError};

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

/// `GET /api/showcase`: the apps on show, newest first. No sign-in needed.
pub async fn list(State(state): State<AppState>) -> Result<Json<Vec<ShowcaseApp>>, ApiError> {
    let apps: Vec<ShowcaseApp> = sqlx::query_as(&format!(
        "SELECT id, name, blurb, url, stack, team, showcased_at FROM ( \
             SELECT p.id, p.name, p.showcase_blurb AS blurb, {LIVE_URL} AS url, p.stack, \
                    CASE WHEN o.is_personal THEN NULL ELSE o.name END AS team, \
                    COALESCE(p.showcased_at, p.created_at) AS showcased_at \
             FROM projects p JOIN organizations o ON o.id = p.org_id \
             WHERE p.showcase AND p.shipyard_app IS NOT NULL \
         ) shown WHERE url IS NOT NULL ORDER BY showcased_at DESC LIMIT {PAGE_SIZE}"
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
        "SELECT p.showcase, p.showcase_blurb, CASE WHEN p.shipyard_app IS NULL THEN NULL ELSE {LIVE_URL} END \
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
    if req.showcase && read(&state, project_id, true).await?.url.is_none() {
        return Err((StatusCode::CONFLICT, "publish the app first: only live apps can be shown".to_string()));
    }
    sqlx::query(
        "UPDATE projects SET showcase = $2, showcase_blurb = $3, \
         showcased_at = CASE WHEN $2 AND NOT showcase THEN now() WHEN $2 THEN showcased_at ELSE NULL END \
         WHERE id = $1",
    )
    .bind(project_id)
    .bind(req.showcase)
    .bind(&blurb)
    .execute(&state.pool)
    .await
    .map_err(internal("failed to save the showcase setting"))?;
    Ok(Json(read(&state, project_id, true).await?))
}

/// Production went offline: the app leaves the showcase (its owners can show it again once it's
/// published).
pub async fn hide_when_offline(pool: &sqlx::PgPool, project_id: Uuid, environment: &str) {
    if environment != "production" {
        return;
    }
    if let Err(e) = sqlx::query("UPDATE projects SET showcase = false, showcased_at = NULL WHERE id = $1").bind(project_id).execute(pool).await {
        tracing::warn!(error = %e, %project_id, "couldn't take an offline app off the showcase");
    }
}
