//! A project's own domains: a hostname its admins own (shop.example.com) beside the app's
//! address under Nomi's apps domain. Nomi gives the records to set at the registrar (a CNAME to
//! the app's address, or A records to the same ingress for a bare domain, and a TXT proving
//! the domain is theirs), looks them up when asked and every few minutes (`recheck_due`), and
//! says which are found. Shipyard serves one address per environment, so HTTPS on the domain
//! stays `pending` until it can attach more.

use std::time::Duration;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use nomi_auth::extractor::AuthClaims;

use crate::app::AppState;
use crate::dns::{self, RecordKind};
use crate::routes::deploy::{internal, project_role, require_admin, ApiError};

/// The label under the domain that carries the TXT proof.
const VERIFY_LABEL: &str = "_nomi-verify";
/// What the TXT record's value starts with.
const TOKEN_PREFIX: &str = "nomi-verify=";
/// DNS's own limits on a whole name and on one label.
const HOSTNAME_MAX: usize = 253;
const LABEL_MAX: usize = 63;
/// A country's second-level names a bare domain sits under (warung.co.id is bare, like warung.id).
const SECOND_LEVELS: &[&str] = &[
    "co", "com", "net", "org", "ac", "go", "or", "web", "my", "sch", "biz", "gov", "edu", "id",
];
/// How long a country code top-level domain is.
const COUNTRY_TLD_LEN: usize = 2;
/// How often each domain is looked up again by the worker.
pub const RECHECK_EVERY: Duration = Duration::from_secs(5 * 60);
/// How many domains one worker pass looks up.
const RECHECK_BATCH: i64 = 20;

const FOUND: &str = "found";
const WAITING: &str = "waiting";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct DnsRecord {
    #[serde(rename = "type")]
    pub kind: String,
    pub name: String,
    pub value: String,
    pub status: String,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct CustomDomain {
    pub hostname: String,
    #[sqlx(json)]
    pub records: Vec<DnsRecord>,
    pub checked_at: Option<DateTime<Utc>>,
    pub verified_at: Option<DateTime<Utc>>,
    #[sqlx(rename = "is_primary")]
    pub primary: bool,
    pub https: String,
    pub created_at: DateTime<Utc>,
}

/// A domain's token, its records as last kept, and when it was verified.
type StoredDomain = (
    String,
    sqlx::types::Json<Vec<DnsRecord>>,
    Option<DateTime<Utc>>,
);

const COLUMNS: &str = "hostname, records, checked_at, verified_at, is_primary, https, created_at";

/// The hostname as DNS reads it: lowercased, without a scheme, path, port or trailing dot.
fn clean_hostname(raw: &str) -> Result<String, ApiError> {
    let invalid = || {
        (
            StatusCode::BAD_REQUEST,
            "that isn't a domain name (like shop.example.com)".to_string(),
        )
    };
    let mut host = raw.trim().to_lowercase();
    for scheme in ["https://", "http://"] {
        if let Some(rest) = host.strip_prefix(scheme) {
            host = rest.to_string();
        }
    }
    let host = host.split(['/', '?', '#']).next().unwrap_or_default();
    let host = host
        .split(':')
        .next()
        .unwrap_or_default()
        .trim_end_matches('.')
        .to_string();
    if host.is_empty() || host.len() > HOSTNAME_MAX || host.parse::<std::net::IpAddr>().is_ok() {
        return Err(invalid());
    }
    let labels: Vec<&str> = host.split('.').collect();
    if labels.len() < 2 {
        return Err(invalid());
    }
    let label_ok = |l: &&str| {
        !l.is_empty()
            && l.len() <= LABEL_MAX
            && !l.starts_with('-')
            && !l.ends_with('-')
            && l.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
    };
    if !labels.iter().all(label_ok)
        || labels
            .last()
            .is_some_and(|tld| tld.chars().all(|c| c.is_ascii_digit()))
    {
        return Err(invalid());
    }
    Ok(host)
}

/// A bare domain (example.com, warung.co.id) can't hold a CNAME: it points A records instead.
fn is_apex(host: &str) -> bool {
    let labels: Vec<&str> = host.split('.').collect();
    match labels.as_slice() {
        [_, _] => true,
        [_, second, tld] => tld.len() == COUNTRY_TLD_LEN && SECOND_LEVELS.contains(second),
        _ => false,
    }
}

fn record(kind: RecordKind, name: &str, value: &str) -> DnsRecord {
    DnsRecord {
        kind: kind.as_str().to_string(),
        name: name.to_string(),
        value: value.to_string(),
        status: WAITING.to_string(),
    }
}

/// The app's own address (its latest live production publish), which the domain points at.
async fn app_hostname(pool: &PgPool, project_id: Uuid) -> Result<Option<String>, sqlx::Error> {
    let url: Option<String> = sqlx::query_scalar(
        "SELECT url FROM project_publishes WHERE project_id = $1 AND environment = 'production' AND status = 'succeeded' \
         AND url IS NOT NULL ORDER BY created_at DESC LIMIT 1",
    )
    .bind(project_id)
    .fetch_optional(pool)
    .await?;
    Ok(url
        .and_then(|u| reqwest::Url::parse(&u).ok())
        .and_then(|u| u.host_str().map(str::to_string)))
}

/// What the registrar should hold for `host`, pointing at `app_host` (whose own addresses are
/// the ingress a bare domain's A records name).
async fn expected(host: &str, token: &str, app_host: &str) -> Vec<DnsRecord> {
    let mut records = Vec::new();
    let ingress = dns::resolver().lookup(app_host, RecordKind::A).await;
    if is_apex(host) && !ingress.is_empty() {
        records.extend(ingress.iter().map(|ip| record(RecordKind::A, host, ip)));
    } else {
        records.push(record(RecordKind::Cname, host, app_host));
    }
    records.push(record(
        RecordKind::Txt,
        &format!("{VERIFY_LABEL}.{host}"),
        &format!("{TOKEN_PREFIX}{token}"),
    ));
    records
}

/// Looks each record up and marks it found or waiting. A CNAME also counts as found when the
/// name already reaches the app's addresses some other way (a registrar's ALIAS or flattening).
async fn look(records: &mut [DnsRecord], app_host: Option<&str>) {
    let resolver = dns::resolver();
    for r in records.iter_mut() {
        let found = match r.kind.as_str() {
            "A" => resolver
                .lookup(&r.name, RecordKind::A)
                .await
                .contains(&r.value),
            "CNAME" => {
                let cname = resolver.lookup(&r.name, RecordKind::Cname).await;
                if cname.iter().any(|v| v.eq_ignore_ascii_case(&r.value)) {
                    true
                } else if let Some(app_host) = app_host {
                    let ingress = resolver.lookup(app_host, RecordKind::A).await;
                    let here = resolver.lookup(&r.name, RecordKind::A).await;
                    !here.is_empty() && here.iter().all(|ip| ingress.contains(ip))
                } else {
                    false
                }
            }
            "TXT" => resolver
                .lookup(&r.name, RecordKind::Txt)
                .await
                .iter()
                .any(|v| v.trim_matches('"') == r.value),
            _ => false,
        };
        r.status = if found { FOUND } else { WAITING }.to_string();
    }
}

/// Looks `hostname`'s records up now and keeps what was found. `None` when the domain isn't
/// on `project_id`.
pub(crate) async fn check_one(
    pool: &PgPool,
    project_id: Uuid,
    hostname: &str,
) -> Result<Option<CustomDomain>, sqlx::Error> {
    let row: Option<StoredDomain> =
        sqlx::query_as("SELECT token, records, verified_at FROM project_domains WHERE project_id = $1 AND hostname = $2")
            .bind(project_id)
            .bind(hostname)
            .fetch_optional(pool)
            .await?;
    let Some((token, stored, was_verified)) = row else {
        return Ok(None);
    };
    let app_host = app_hostname(pool, project_id).await?;
    // The app's address can move (renamed, published again): the records follow it.
    let mut records = match &app_host {
        Some(app_host) => expected(hostname, &token, app_host).await,
        None => stored.0,
    };
    look(&mut records, app_host.as_deref()).await;
    let all_found = !records.is_empty() && records.iter().all(|r| r.status == FOUND);
    let domain: CustomDomain = sqlx::query_as(&format!(
        "UPDATE project_domains SET records = $3, checked_at = now(), \
         verified_at = CASE WHEN $4 THEN COALESCE(verified_at, now()) ELSE NULL END \
         WHERE project_id = $1 AND hostname = $2 RETURNING {COLUMNS}"
    ))
    .bind(project_id)
    .bind(hostname)
    .bind(sqlx::types::Json(&records))
    .bind(all_found)
    .fetch_one(pool)
    .await?;
    if was_verified.is_some() != all_found {
        crate::project_events::changed(project_id, "domain.checked");
    }
    Ok(Some(domain))
}

/// The worker's pass: looks up again the domains last looked at more than `RECHECK_EVERY` ago,
/// oldest first. Returns how many it looked at.
pub async fn recheck_due(pool: &PgPool) -> usize {
    let due: Vec<(Uuid, String)> = match sqlx::query_as(
        "SELECT project_id, hostname FROM project_domains \
         WHERE checked_at IS NULL OR checked_at < now() - make_interval(secs => $1) \
         ORDER BY checked_at NULLS FIRST LIMIT $2",
    )
    .bind(RECHECK_EVERY.as_secs_f64())
    .bind(RECHECK_BATCH)
    .fetch_all(pool)
    .await
    {
        Ok(due) => due,
        Err(e) => {
            tracing::warn!(error = %e, "domains: couldn't list the ones due a look");
            return 0;
        }
    };
    for (project_id, hostname) in &due {
        if let Err(e) = check_one(pool, *project_id, hostname).await {
            tracing::warn!(error = %e, %hostname, "domains: look failed");
        }
    }
    due.len()
}

fn not_found() -> ApiError {
    (
        StatusCode::NOT_FOUND,
        "that domain isn't on this project".to_string(),
    )
}

/// `GET /api/projects/:id/domains`: the primary first, then as added.
pub async fn list(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(project_id): Path<Uuid>,
) -> Result<Json<Vec<CustomDomain>>, ApiError> {
    project_role(&state.pool, project_id, claims.sub).await?;
    let domains = sqlx::query_as(&format!("SELECT {COLUMNS} FROM project_domains WHERE project_id = $1 ORDER BY is_primary DESC, created_at"))
        .bind(project_id)
        .fetch_all(&state.pool)
        .await
        .map_err(internal("failed to list domains"))?;
    Ok(Json(domains))
}

#[derive(Deserialize)]
pub struct AddDomain {
    pub hostname: String,
}

/// `POST /api/projects/:id/domains {hostname}` → 201, looked up once already. The app must be
/// live (the records point at its address); a hostname is on one project only.
pub async fn add(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(project_id): Path<Uuid>,
    Json(req): Json<AddDomain>,
) -> Result<(StatusCode, Json<CustomDomain>), ApiError> {
    require_admin(&state.pool, project_id, claims.sub).await?;
    // A domain of the project's own is the project owner's plan to allow, whoever of its admins asks.
    if let Some(owner_plan) = nomi_usage::entitlements::for_project_owner(&state.pool, project_id)
        .await
        .map_err(internal("failed to add the domain"))?
    {
        if !owner_plan.custom_domain {
            return Err((
                StatusCode::PAYMENT_REQUIRED,
                nomi_usage::entitlements::LimitHit::CustomDomain.message(),
            ));
        }
    }
    let hostname = clean_hostname(&req.hostname)?;
    if let Some(base) = crate::configuration::apps_base_domain().await {
        if hostname == base || hostname.ends_with(&format!(".{base}")) {
            return Err((
                StatusCode::BAD_REQUEST,
                "that's under Nomi's own apps domain: change the app's address instead".to_string(),
            ));
        }
    }
    let app_host = app_hostname(&state.pool, project_id)
        .await
        .map_err(internal("failed to add the domain"))?;
    let Some(app_host) = app_host else {
        return Err((
            StatusCode::CONFLICT,
            "publish the app first: the domain points at its address".to_string(),
        ));
    };
    if hostname == app_host {
        return Err((
            StatusCode::BAD_REQUEST,
            "that's the app's own address already".to_string(),
        ));
    }
    let token = Uuid::new_v4().simple().to_string();
    let records = expected(&hostname, &token, &app_host).await;
    let inserted = sqlx::query(
        "INSERT INTO project_domains (project_id, hostname, token, records, created_by) VALUES ($1, $2, $3, $4, $5) \
         ON CONFLICT (hostname) DO NOTHING",
    )
    .bind(project_id)
    .bind(&hostname)
    .bind(&token)
    .bind(sqlx::types::Json(&records))
    .bind(claims.sub)
    .execute(&state.pool)
    .await
    .map_err(internal("failed to add the domain"))?;
    if inserted.rows_affected() == 0 {
        return Err((
            StatusCode::CONFLICT,
            "that domain is already on a project".to_string(),
        ));
    }
    let domain = check_one(&state.pool, project_id, &hostname)
        .await
        .map_err(internal("failed to check the domain"))?
        .ok_or_else(not_found)?;
    crate::project_events::changed(project_id, "domain.added");
    Ok((StatusCode::CREATED, Json(domain)))
}

/// `POST /api/projects/:id/domains/:hostname/check`: Check now.
pub async fn check(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path((project_id, hostname)): Path<(Uuid, String)>,
) -> Result<Json<CustomDomain>, ApiError> {
    project_role(&state.pool, project_id, claims.sub).await?;
    let domain = check_one(&state.pool, project_id, &hostname.to_lowercase())
        .await
        .map_err(internal("failed to check the domain"))?;
    domain.map(Json).ok_or_else(not_found)
}

/// `PUT /api/projects/:id/domains/:hostname/primary`: the address the app is known by. Only a
/// domain whose records are all found.
pub async fn make_primary(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path((project_id, hostname)): Path<(Uuid, String)>,
) -> Result<Json<CustomDomain>, ApiError> {
    require_admin(&state.pool, project_id, claims.sub).await?;
    let hostname = hostname.to_lowercase();
    let verified: Option<Option<DateTime<Utc>>> = sqlx::query_scalar(
        "SELECT verified_at FROM project_domains WHERE project_id = $1 AND hostname = $2",
    )
    .bind(project_id)
    .bind(&hostname)
    .fetch_optional(&state.pool)
    .await
    .map_err(internal("failed to update the domain"))?;
    match verified {
        None => return Err(not_found()),
        Some(None) => {
            return Err((
                StatusCode::CONFLICT,
                "its records aren't all found yet".to_string(),
            ))
        }
        Some(Some(_)) => {}
    }
    let mut tx = state
        .pool
        .begin()
        .await
        .map_err(internal("failed to update the domain"))?;
    sqlx::query("UPDATE project_domains SET is_primary = (hostname = $2) WHERE project_id = $1")
        .bind(project_id)
        .bind(&hostname)
        .execute(&mut *tx)
        .await
        .map_err(internal("failed to update the domain"))?;
    let domain = sqlx::query_as(&format!(
        "SELECT {COLUMNS} FROM project_domains WHERE project_id = $1 AND hostname = $2"
    ))
    .bind(project_id)
    .bind(&hostname)
    .fetch_one(&mut *tx)
    .await
    .map_err(internal("failed to update the domain"))?;
    tx.commit()
        .await
        .map_err(internal("failed to update the domain"))?;
    crate::project_events::changed(project_id, "domain.primary");
    Ok(Json(domain))
}

/// `DELETE /api/projects/:id/domains/:hostname` → 204.
pub async fn remove(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path((project_id, hostname)): Path<(Uuid, String)>,
) -> Result<StatusCode, ApiError> {
    require_admin(&state.pool, project_id, claims.sub).await?;
    let done = sqlx::query("DELETE FROM project_domains WHERE project_id = $1 AND hostname = $2")
        .bind(project_id)
        .bind(hostname.to_lowercase())
        .execute(&state.pool)
        .await
        .map_err(internal("failed to remove the domain"))?;
    if done.rows_affected() == 0 {
        return Err(not_found());
    }
    crate::project_events::changed(project_id, "domain.removed");
    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hostnames_are_cleaned_and_checked() {
        assert_eq!(
            clean_hostname(" HTTPS://Shop.Example.com/menu ").unwrap(),
            "shop.example.com"
        );
        assert_eq!(clean_hostname("example.com.").unwrap(), "example.com");
        assert_eq!(clean_hostname("example.com:443").unwrap(), "example.com");
        for bad in [
            "",
            "localhost",
            "10.0.0.1",
            "-a.com",
            "a_b.com",
            "a..com",
            "example.123",
        ] {
            assert!(clean_hostname(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn bare_domains_point_a_records() {
        assert!(is_apex("example.com"));
        assert!(is_apex("warungkopi.co.id"));
        assert!(!is_apex("shop.example.com"));
        assert!(!is_apex("www.warungkopi.co.id"));
    }
}
