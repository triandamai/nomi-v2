//! A project's web address on Shipyard: every environment gets `<name>.<apps-domain>` on its
//! first publish, and its admins can pick another free name. Names are shared across the whole
//! platform, so Shipyard says whether one is free and suggests free ones nearby.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

use nomi_auth::extractor::AuthClaims;
use nomi_shipyard::ShipyardError;

use crate::app::AppState;
use crate::routes::deploy::{client, internal, project_role, require_admin, shipyard_error, target, ApiError};

const ENV: &str = "production";
/// Shipyard's own limit on a name.
const NAME_MAX: usize = 63;

#[derive(Serialize)]
pub struct ProjectDomain {
    /// `None` until the first publish gives production an address.
    pub hostname: Option<String>,
    pub subdomain: Option<String>,
    pub url: Option<String>,
    pub can_edit: bool,
}

fn text(value: &Value, key: &str) -> Option<String> {
    value.get(key).and_then(Value::as_str).map(str::to_string)
}

/// `GET /api/projects/:id/domain`
pub async fn get(State(state): State<AppState>, AuthClaims(claims): AuthClaims, Path(project_id): Path<Uuid>) -> Result<Json<ProjectDomain>, ApiError> {
    let role = project_role(&state.pool, project_id, claims.sub).await?;
    let target = target(&state, project_id).await?;
    let can_edit = role == "admin";
    let Some(app) = target.shipyard_app.as_deref() else {
        return Ok(Json(ProjectDomain { hostname: None, subdomain: None, url: None, can_edit }));
    };
    let domain = match client(&state)?.get_domain(&target.org_key(), app, ENV).await {
        Ok(domain) => domain,
        // The app or its production isn't there yet: no address.
        Err(ShipyardError::NotFound(_)) => Value::Null,
        Err(e) => return Err(shipyard_error(e)),
    };
    let mut hostname = text(&domain, "hostname");
    let mut subdomain = text(&domain, "subdomain");
    let mut url = text(&domain, "url");
    if subdomain.as_deref().is_some_and(|s| s.ends_with(nomi_shipyard::manifest::BACKEND_SUFFIX)) {
        hostname = None;
        subdomain = None;
        url = None;
        if let Ok(env_data) = client(&state)?.get_env(&target.org_key(), app, ENV).await {
            if let Some(web_url) = env_data
                .get("services")
                .and_then(Value::as_array)
                .and_then(|services| {
                    services.iter().find(|s| s.get("name").and_then(Value::as_str) == Some("web")).and_then(|s| {
                        s.get("urls").and_then(Value::as_array).and_then(|u| u.first()).and_then(Value::as_str)
                    })
                })
            {
                if let Ok(parsed) = reqwest::Url::parse(web_url) {
                    if let Some(host) = parsed.host_str() {
                        let sub = host.split('.').next().unwrap_or(host);
                        hostname = Some(host.to_string());
                        subdomain = Some(sub.to_string());
                        url = Some(web_url.to_string());
                    }
                }
            }
        }
    }
    Ok(Json(ProjectDomain { hostname, subdomain, url, can_edit }))
}

#[derive(Deserialize)]
pub struct CheckQuery {
    pub subdomain: String,
}

/// The name as Shipyard reads it: trimmed and lowercased.
fn clean(subdomain: &str) -> Result<String, ApiError> {
    let name = subdomain.trim().to_lowercase();
    if name.is_empty() || name.len() > NAME_MAX {
        return Err((StatusCode::BAD_REQUEST, format!("a name is 3 to {NAME_MAX} characters")));
    }
    Ok(name)
}

/// `GET /api/projects/:id/domain/check?subdomain=`: Shipyard's answer as it is, `available`
/// with `reason`, `message` and `suggestions` when it isn't. The project's own current name
/// counts as available.
pub async fn check(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(project_id): Path<Uuid>,
    Query(q): Query<CheckQuery>,
) -> Result<Json<Value>, ApiError> {
    project_role(&state.pool, project_id, claims.sub).await?;
    let name = clean(&q.subdomain)?;
    let target = target(&state, project_id).await?;
    let app_env = target.shipyard_app.as_deref().map(|app| (app, ENV));
    Ok(Json(client(&state)?.check_domain(&target.org_key(), &name, app_env).await.map_err(shipyard_error)?))
}

#[derive(Deserialize)]
pub struct ChangeRequest {
    pub subdomain: String,
}

/// `PUT /api/projects/:id/domain`: the project's admins move production to another name. When
/// Shipyard refuses (taken a moment ago, invalid, nothing published yet) the answer is `409`
/// with why and free names to pick from instead.
pub async fn change(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(project_id): Path<Uuid>,
    Json(req): Json<ChangeRequest>,
) -> Result<Response, ApiError> {
    require_admin(&state.pool, project_id, claims.sub).await?;
    let name = clean(&req.subdomain)?;
    let target = target(&state, project_id).await?;
    let Some(app) = target.shipyard_app.as_deref() else {
        return Err((StatusCode::CONFLICT, "publish the app first: it gets an address on its first publish".to_string()));
    };
    let shipyard = client(&state)?;
    let changed = match shipyard.set_domain(&target.org_key(), app, ENV, &name).await {
        Ok(changed) => changed,
        Err(ShipyardError::Rejected { status: 409, message, .. }) => {
            // Why, and what's free instead (the refusal itself doesn't list them for Nomi).
            let mut answer = shipyard.check_domain(&target.org_key(), &name, Some((app, ENV))).await.unwrap_or_else(|_| json!({}));
            if let Value::Object(map) = &mut answer {
                map.insert("available".into(), Value::Bool(false));
                map.insert("message".into(), Value::String(message));
            }
            return Ok((StatusCode::CONFLICT, Json(answer)).into_response());
        }
        Err(e) => return Err(shipyard_error(e)),
    };
    // Everything that shows the address follows: the latest publish's address (the top bar and
    // the showcase read it) and a showcased app's stored one.
    if let Some(url) = changed.get("url").and_then(Value::as_str) {
        let latest: Option<Uuid> = sqlx::query_scalar(
            "SELECT id FROM project_publishes WHERE project_id = $1 AND environment = $2 AND status = 'succeeded' \
             ORDER BY created_at DESC LIMIT 1",
        )
        .bind(project_id)
        .bind(ENV)
        .fetch_optional(&state.pool)
        .await
        .map_err(internal("failed to update the address"))?;
        if let Some(latest) = latest {
            sqlx::query("UPDATE project_publishes SET url = $2 WHERE id = $1").bind(latest).bind(url).execute(&state.pool).await.map_err(internal("failed to update the address"))?;
        }
        sqlx::query("UPDATE projects SET showcase_url = $2 WHERE id = $1 AND showcase_url IS NOT NULL")
            .bind(project_id)
            .bind(url)
            .execute(&state.pool)
            .await
            .map_err(internal("failed to update the address"))?;
    }
    crate::project_events::changed(project_id, "domain.changed");
    Ok(Json(changed).into_response())
}
