//! A project's web address on Shipyard: every environment gets `<name>.<apps-domain>` on its
//! first publish, and its admins can pick another free name. The apps domain is Nomi's own
//! (Admin → Configuration → Domains), sent to Shipyard as full hostnames; unset, it's Shipyard's.
//! Names are shared across the whole platform, so Shipyard says whether one is free and suggests
//! free ones nearby.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

use nomi_auth::extractor::AuthClaims;
use nomi_shipyard::{DomainName, ShipyardError};

use crate::app::AppState;
use crate::routes::deploy::{client, internal, project_role, require_admin, shipyard_error, target, ApiError};
use crate::shipyard::{first_label, service_host};

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
    /// Nomi's apps base domain (Admin → Configuration → Domains); `None` leaves it to Shipyard.
    pub base_domain: Option<String>,
    /// Where the address moves on the next publish, when it isn't under the base domain yet.
    pub pending_hostname: Option<String>,
}

/// `<label>.<base>` when `hostname` isn't already under `base`: where the next publish moves it.
fn pending_hostname(hostname: Option<&str>, base: Option<&str>) -> Option<String> {
    let (hostname, base) = (hostname?, base?);
    let wanted = format!("{}.{base}", first_label(hostname));
    (hostname != wanted).then_some(wanted)
}

fn text(value: &Value, key: &str) -> Option<String> {
    value.get(key).and_then(Value::as_str).map(str::to_string)
}

/// `GET /api/projects/:id/domain`
pub async fn get(State(state): State<AppState>, AuthClaims(claims): AuthClaims, Path(project_id): Path<Uuid>) -> Result<Json<ProjectDomain>, ApiError> {
    let role = project_role(&state.pool, project_id, claims.sub).await?;
    let target = target(&state, project_id).await?;
    let can_edit = role == "admin";
    let base_domain = crate::configuration::apps_base_domain().await;
    let Some(app) = target.shipyard_app.as_deref() else {
        return Ok(Json(ProjectDomain { hostname: None, subdomain: None, url: None, can_edit, base_domain, pending_hostname: None }));
    };
    let domain = match client(&state)?.get_domain(&target.org_key(), app, ENV).await {
        Ok(domain) => domain,
        // The app or its production isn't there yet: no address.
        Err(ShipyardError::NotFound(_)) => Value::Null,
        Err(e) => return Err(shipyard_error(e)),
    };
    let mut hostname = text(&domain, "hostname");
    let mut url = text(&domain, "url");
    // The environment's managed address can be the backend's; the app's own is the web service's.
    if hostname.as_deref().is_some_and(|h| first_label(h).ends_with(nomi_shipyard::manifest::BACKEND_SUFFIX)) {
        hostname = service_host(&client(&state)?, &target.org_key(), app, ENV, nomi_shipyard::manifest::WEB_SERVICE).await;
        url = hostname.as_ref().map(|h| format!("https://{h}"));
    }
    // The part admins pick: the first label, whichever domain it sits under.
    let subdomain = hostname.as_deref().map(|h| first_label(h).to_string());
    let pending_hostname = pending_hostname(hostname.as_deref(), base_domain.as_deref());
    Ok(Json(ProjectDomain { hostname, subdomain, url, can_edit, base_domain, pending_hostname }))
}

#[derive(Deserialize)]
pub struct CheckQuery {
    pub subdomain: String,
}

pub fn env_suffix() -> Option<&'static str> {
    let raw = std::env::var("ENVI")
        .or_else(|_| std::env::var("ENVIRONMENT"))
        .or_else(|_| std::env::var("APP_ENV"))
        .ok()?;
    let val = raw.trim().to_lowercase();
    if val == "production" || val == "prod" {
        None
    } else {
        Some("-staging")
    }
}

/// Applies the environment domain suffix (e.g. `-staging` for non-prod/staging) if not already present.
pub fn apply_domain_suffix(name: &str) -> String {
    if let Some(suffix) = env_suffix() {
        if !name.ends_with(suffix) {
            return format!("{name}{suffix}");
        }
    }
    name.to_string()
}

/// The name as Shipyard reads it: trimmed and lowercased, one label (the domain is Nomi's).
fn clean(subdomain: &str) -> Result<String, ApiError> {
    let raw = subdomain.trim().to_lowercase();
    if raw.is_empty() {
        return Err((StatusCode::BAD_REQUEST, format!("a name is 3 to {NAME_MAX} characters")));
    }
    if raw.contains('.') {
        return Err((StatusCode::BAD_REQUEST, "a name can't contain dots".to_string()));
    }
    let name = apply_domain_suffix(&raw);
    if name.len() < 3 || name.len() > NAME_MAX {
        return Err((StatusCode::BAD_REQUEST, format!("a name is 3 to {NAME_MAX} characters")));
    }
    Ok(name)
}

/// What Shipyard is asked about for `name`: `<name>.<base>` under Nomi's apps base domain, else
/// the name alone under Shipyard's.
fn wanted<'a>(name: &'a str, hostname: &'a Option<String>) -> DomainName<'a> {
    match hostname {
        Some(full) => DomainName::Hostname(full.as_str()),
        None => DomainName::Subdomain(name),
    }
}

async fn full_hostname(name: &str) -> Option<String> {
    crate::configuration::apps_base_domain().await.map(|base| format!("{name}.{base}"))
}

/// Shipyard's answer with `subdomain` filled in as the first label, which is what Nomi edits
/// (Shipyard leaves it empty for a hostname outside its own apps domain).
fn with_label(mut answer: Value) -> Value {
    if let Some(label) = answer.get("hostname").and_then(Value::as_str).map(|h| first_label(h).to_string()) {
        if let Value::Object(map) = &mut answer {
            map.insert("subdomain".into(), Value::String(label));
        }
    }
    answer
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
    let hostname = full_hostname(&name).await;
    let target = target(&state, project_id).await?;
    let app_env = target.shipyard_app.as_deref().map(|app| (app, ENV));
    let answer = client(&state)?.check_domain(&target.org_key(), wanted(&name, &hostname), app_env).await.map_err(shipyard_error)?;
    Ok(Json(with_label(answer)))
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
    let hostname = full_hostname(&name).await;
    let target = target(&state, project_id).await?;
    let Some(app) = target.shipyard_app.as_deref() else {
        return Err((StatusCode::CONFLICT, "publish the app first: it gets an address on its first publish".to_string()));
    };
    let shipyard = client(&state)?;
    let changed = match shipyard.set_domain(&target.org_key(), app, ENV, wanted(&name, &hostname)).await {
        Ok(changed) => changed,
        Err(ShipyardError::Rejected { status: 409, message, .. }) => {
            // Why, and what's free instead (the refusal itself doesn't list them for Nomi).
            let mut answer = shipyard.check_domain(&target.org_key(), wanted(&name, &hostname), Some((app, ENV))).await.map(with_label).unwrap_or_else(|_| json!({}));
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_address_outside_the_base_domain_moves_under_it_keeping_its_name() {
        assert_eq!(pending_hostname(Some("todo.apps.old.dev"), Some("nomi.app")), Some("todo.nomi.app".to_string()));
        assert_eq!(pending_hostname(Some("todo.nomi.app"), Some("nomi.app")), None, "already there");
        assert_eq!(pending_hostname(Some("todo.apps.old.dev"), None), None, "no base domain set");
        assert_eq!(pending_hostname(None, Some("nomi.app")), None, "not published yet");
    }

    #[test]
    fn domain_suffix_in_staging_and_prod() {
        std::env::set_var("ENVI", "production");
        assert_eq!(apply_domain_suffix("my-app"), "my-app");

        std::env::set_var("ENVI", "prod");
        assert_eq!(apply_domain_suffix("my-app"), "my-app");

        std::env::set_var("ENVI", "staging");
        assert_eq!(apply_domain_suffix("my-app"), "my-app-staging");
        // No duplicate suffix
        assert_eq!(apply_domain_suffix("my-app-staging"), "my-app-staging");

        std::env::set_var("ENVI", "dev");
        assert_eq!(apply_domain_suffix("my-app"), "my-app-staging");

        std::env::remove_var("ENVI");
        assert_eq!(apply_domain_suffix("my-app"), "my-app");
    }
}
