//! What a published site's Monitor says beyond Shipyard's traffic: where visits come from, the
//! forms sent, and whether it stayed up.
//!
//! - Publishing to production puts a small script in the site's HTML pages (`with_beacon`) that
//!   reports each arrival with its referrer, and each form sent, to `POST /api/hooks/sites/:id`.
//!   Moving between the site's own pages isn't an arrival, so it isn't reported.
//! - The worker opens each live app's address every few minutes (`probe_due`) and keeps whether
//!   it answered.
//! - `extras` adds `sources`, `form_sends` and `uptime` to the metrics answer: null where nothing
//!   was ever reported or looked at, never a made-up 0.

use std::time::Duration;

use axum::body::Bytes;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::PgPool;
use uuid::Uuid;

use nomi_shipyard::MetricsRange;

use crate::app::AppState;

/// The marker on the injected script, so a page is never given it twice.
const BEACON_MARKER: &str = "data-nomi-visits";
/// Where the site reports, under Nomi's public address.
const BEACON_PATH: &str = "/hooks/sites";
/// The largest report taken; a real one is well under this.
const MAX_REPORT_BYTES: usize = 2048;
/// The longest referrer host kept.
const MAX_HOST_LEN: usize = 253;
/// How often each live app's address is opened.
pub const PROBE_EVERY: Duration = Duration::from_secs(5 * 60);
/// How long the address has to answer.
const PROBE_TIMEOUT: Duration = Duration::from_secs(10);
/// How many apps one worker pass opens.
const PROBE_BATCH: i64 = 20;
/// Answers from this status up count as down.
const DOWN_FROM_STATUS: u16 = 500;
/// How long reports and looks are kept.
const RETENTION_DAYS: i32 = 30;
const HOUR_SECS: f64 = 3600.0;
const DAY_SECS: f64 = 24.0 * HOUR_SECS;
const WEEK_SECS: f64 = 7.0 * DAY_SECS;

/// The script put before `</head>`: reports the arrival (with where it came from) and each
/// form sent. A beacon whose body has no type carries no content type, so it needs no CORS
/// preflight and isn't taken for a cross-site form post by the web app that forwards it.
pub fn snippet(report_url: &str) -> String {
    let url = serde_json::to_string(report_url).unwrap_or_else(|_| "\"\"".to_string());
    format!(
        "<script {BEACON_MARKER}>(function(){{var u={url};function s(k){{try{{navigator.sendBeacon(u,new Blob([JSON.stringify({{k:k,r:document.referrer,h:location.hostname}})]))}}catch(e){{}}}}s('v');document.addEventListener('submit',function(){{s('f')}},true)}})();</script>"
    )
}

/// Where `project_id`'s site reports, when Nomi's public address is known.
pub fn report_url(project_id: Uuid) -> Option<String> {
    crate::public_url::app_url().map(|base| format!("{base}{BEACON_PATH}/{project_id}"))
}

/// Puts the script in every HTML page that has a `</head>` and doesn't have it yet (outside
/// `node_modules`). Returns how many pages were given it.
pub fn with_beacon(files: &mut [(String, Vec<u8>)], report_url: &str) -> usize {
    let script = snippet(report_url);
    let mut given = 0;
    for (path, bytes) in files.iter_mut() {
        if !path.ends_with(".html") || path.split('/').any(|part| part == "node_modules") {
            continue;
        }
        let Ok(text) = std::str::from_utf8(bytes) else {
            continue;
        };
        if text.contains(BEACON_MARKER) {
            continue;
        }
        let Some(at) = text.to_ascii_lowercase().find("</head>") else {
            continue;
        };
        let mut out = String::with_capacity(text.len() + script.len());
        out.push_str(&text[..at]);
        out.push_str(&script);
        out.push_str(&text[at..]);
        *bytes = out.into_bytes();
        given += 1;
    }
    given
}

#[derive(Deserialize)]
struct Report {
    /// `v` an arrival, `f` a form sent.
    k: String,
    #[serde(default)]
    r: String,
    #[serde(default)]
    h: String,
}

fn host_of(url: &str) -> Option<String> {
    let host = reqwest::Url::parse(url).ok()?.host_str()?.to_lowercase();
    (!host.is_empty() && host.len() <= MAX_HOST_LEN).then_some(host)
}

/// `POST /api/hooks/sites/:project_id`: a published site's report, unauthenticated (it's the
/// visitor's browser). Always 204, so a bad report teaches nothing.
pub async fn report(
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
    body: Bytes,
) -> StatusCode {
    if body.len() > MAX_REPORT_BYTES {
        return StatusCode::NO_CONTENT;
    }
    let Ok(report) = serde_json::from_slice::<Report>(&body) else {
        return StatusCode::NO_CONTENT;
    };
    let referrer = host_of(&report.r);
    let kind = match report.k.as_str() {
        "v" => {
            // A move between the site's own pages isn't an arrival.
            if referrer
                .as_deref()
                .is_some_and(|r| r.eq_ignore_ascii_case(report.h.trim()))
            {
                return StatusCode::NO_CONTENT;
            }
            "visit"
        }
        "f" => "form",
        _ => return StatusCode::NO_CONTENT,
    };
    let result = sqlx::query(
        "INSERT INTO site_hits (project_id, kind, referrer_host) \
         SELECT id, $2, $3 FROM projects WHERE id = $1 AND shipyard_app IS NOT NULL",
    )
    .bind(project_id)
    .bind(kind)
    .bind(if kind == "visit" { referrer } else { None })
    .execute(&state.pool)
    .await;
    if let Err(e) = result {
        tracing::warn!(error = %e, %project_id, "couldn't keep a site's report");
    }
    StatusCode::NO_CONTENT
}

/// The source a referrer's host is shown as: the big ones by name, the rest by host.
pub fn source_name(host: Option<&str>) -> String {
    let Some(host) = host
        .map(|h| h.trim_start_matches("www.").to_lowercase())
        .filter(|h| !h.is_empty())
    else {
        return "Direct".to_string();
    };
    let is = |names: &[&str]| {
        names
            .iter()
            .any(|n| host == *n || host.ends_with(&format!(".{n}")))
    };
    let named = if is(&["instagram.com"]) {
        "Instagram"
    } else if host.starts_with("google.") || host.contains(".google.") {
        "Google search"
    } else if is(&["whatsapp.com", "whatsapp.net", "wa.me"]) {
        "WhatsApp"
    } else if is(&["facebook.com", "fb.com", "fb.me"]) {
        "Facebook"
    } else if is(&["t.co", "twitter.com", "x.com"]) {
        "X"
    } else if is(&["tiktok.com"]) {
        "TikTok"
    } else if is(&["linkedin.com", "lnkd.in"]) {
        "LinkedIn"
    } else if is(&["bing.com"]) {
        "Bing search"
    } else if is(&["duckduckgo.com"]) {
        "DuckDuckGo"
    } else {
        return host;
    };
    named.to_string()
}

#[derive(Serialize, Debug, PartialEq)]
pub struct Source {
    pub name: String,
    pub share: f64,
}

/// Visits grouped by source, largest share first.
pub fn group_sources(counts: &[(Option<String>, i64)]) -> Vec<Source> {
    let total: i64 = counts.iter().map(|(_, n)| n).sum();
    if total == 0 {
        return Vec::new();
    }
    let mut grouped: Vec<(String, i64)> = Vec::new();
    for (host, n) in counts {
        let name = source_name(host.as_deref());
        match grouped.iter_mut().find(|(g, _)| *g == name) {
            Some((_, sum)) => *sum += n,
            None => grouped.push((name, *n)),
        }
    }
    grouped.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    grouped
        .into_iter()
        .map(|(name, n)| Source {
            name,
            share: n as f64 / total as f64,
        })
        .collect()
}

fn range_secs(range: MetricsRange) -> f64 {
    match range {
        MetricsRange::Hour => HOUR_SECS,
        MetricsRange::Day => DAY_SECS,
        MetricsRange::Week => WEEK_SECS,
    }
}

/// `sources`, `form_sends` and `uptime` over `range`, as Nomi measured them.
pub async fn extras(
    pool: &PgPool,
    project_id: Uuid,
    range: MetricsRange,
) -> Result<Value, sqlx::Error> {
    let secs = range_secs(range);
    let reported: bool =
        sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM site_hits WHERE project_id = $1)")
            .bind(project_id)
            .fetch_one(pool)
            .await?;
    let (sources, form_sends) = if reported {
        let counts: Vec<(Option<String>, i64)> = sqlx::query_as(
            "SELECT referrer_host, count(*) FROM site_hits WHERE project_id = $1 AND kind = 'visit' \
             AND at > now() - make_interval(secs => $2) GROUP BY referrer_host",
        )
        .bind(project_id)
        .bind(secs)
        .fetch_all(pool)
        .await?;
        let forms: i64 = sqlx::query_scalar("SELECT count(*) FROM site_hits WHERE project_id = $1 AND kind = 'form' AND at > now() - make_interval(secs => $2)")
            .bind(project_id)
            .bind(secs)
            .fetch_one(pool)
            .await?;
        (json!(group_sources(&counts)), json!(forms))
    } else {
        (Value::Null, Value::Null)
    };
    let (up, total, since): (i64, i64, Option<chrono::DateTime<chrono::Utc>>) = sqlx::query_as(
        "SELECT count(*) FILTER (WHERE up AND checked_at > now() - make_interval(secs => $2)), \
                count(*) FILTER (WHERE checked_at > now() - make_interval(secs => $2)), min(checked_at) \
         FROM site_uptime_checks WHERE project_id = $1",
    )
    .bind(project_id)
    .bind(secs)
    .fetch_one(pool)
    .await?;
    let uptime = if total > 0 {
        json!({ "share": up as f64 / total as f64, "since": since })
    } else {
        Value::Null
    };
    Ok(json!({ "sources": sources, "form_sends": form_sends, "uptime": uptime }))
}

/// Fills in what Shipyard's own answer doesn't carry.
pub fn merge(metrics: &mut Value, extras: Value) {
    let (Value::Object(target), Value::Object(extras)) = (metrics, extras) else {
        return;
    };
    for (key, value) in extras {
        if target.get(&key).is_none_or(Value::is_null) {
            target.insert(key, value);
        }
    }
}

/// The worker's pass: opens the address of each live app not looked at in the last
/// `PROBE_EVERY`, and keeps whether it answered. Returns how many it opened.
pub async fn probe_due(pool: &PgPool, http: &reqwest::Client) -> usize {
    let due: Vec<(Uuid, String)> = match sqlx::query_as(
        "SELECT p.id, lp.url FROM projects p \
         JOIN LATERAL (SELECT url FROM project_publishes WHERE project_id = p.id AND environment = 'production' \
                       AND status = 'succeeded' AND url IS NOT NULL ORDER BY created_at DESC LIMIT 1) lp ON true \
         WHERE p.shipyard_app IS NOT NULL \
           AND NOT EXISTS (SELECT 1 FROM site_uptime_checks c WHERE c.project_id = p.id AND c.checked_at > now() - make_interval(secs => $1)) \
         LIMIT $2",
    )
    .bind(PROBE_EVERY.as_secs_f64())
    .bind(PROBE_BATCH)
    .fetch_all(pool)
    .await
    {
        Ok(due) => due,
        Err(e) => {
            tracing::warn!(error = %e, "uptime: couldn't list the apps due a look");
            return 0;
        }
    };
    for (project_id, url) in &due {
        let status = match http.get(url).timeout(PROBE_TIMEOUT).send().await {
            Ok(response) => Some(response.status().as_u16()),
            Err(_) => None,
        };
        let up = status.is_some_and(|s| s < DOWN_FROM_STATUS);
        if let Err(e) = sqlx::query(
            "INSERT INTO site_uptime_checks (project_id, up, status) VALUES ($1, $2, $3)",
        )
        .bind(project_id)
        .bind(up)
        .bind(status.map(i32::from))
        .execute(pool)
        .await
        {
            tracing::warn!(error = %e, %project_id, "uptime: couldn't keep a look");
        }
    }
    let _ = sqlx::query(
        "DELETE FROM site_uptime_checks WHERE checked_at < now() - make_interval(days => $1)",
    )
    .bind(RETENTION_DAYS)
    .execute(pool)
    .await;
    let _ = sqlx::query("DELETE FROM site_hits WHERE at < now() - make_interval(days => $1)")
        .bind(RETENTION_DAYS)
        .execute(pool)
        .await;
    due.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn referrers_group_into_sources() {
        assert_eq!(source_name(None), "Direct");
        assert_eq!(source_name(Some("l.instagram.com")), "Instagram");
        assert_eq!(source_name(Some("www.google.co.id")), "Google search");
        assert_eq!(source_name(Some("wa.me")), "WhatsApp");
        assert_eq!(source_name(Some("m.facebook.com")), "Facebook");
        assert_eq!(source_name(Some("www.kompas.com")), "kompas.com");
        let grouped = group_sources(&[
            (Some("l.instagram.com".into()), 2),
            (Some("instagram.com".into()), 1),
            (None, 1),
        ]);
        assert_eq!(
            grouped,
            vec![
                Source {
                    name: "Instagram".into(),
                    share: 0.75
                },
                Source {
                    name: "Direct".into(),
                    share: 0.25
                }
            ]
        );
        assert!(group_sources(&[]).is_empty());
    }

    #[test]
    fn pages_get_the_script_once() {
        let mut files = vec![
            (
                "index.html".to_string(),
                b"<html><HEAD><title>a</title></HEAD><body></body></html>".to_vec(),
            ),
            ("about/index.html".to_string(), b"<p>no head</p>".to_vec()),
            (
                "node_modules/x/index.html".to_string(),
                b"<head></head>".to_vec(),
            ),
            ("app.js".to_string(), b"</head>".to_vec()),
        ];
        assert_eq!(
            with_beacon(&mut files, "https://nomi.test/hooks/sites/1"),
            1
        );
        let page = String::from_utf8(files[0].1.clone()).unwrap();
        assert!(
            page.contains("\"https://nomi.test/hooks/sites/1\"")
                && page.ends_with("</HEAD><body></body></html>"),
            "{page}"
        );
        assert_eq!(
            with_beacon(&mut files, "https://nomi.test/hooks/sites/1"),
            0,
            "never twice"
        );
    }

    #[test]
    fn shipyards_own_numbers_win() {
        let mut metrics = json!({ "totals": {}, "uptime": { "share": 1.0 } });
        merge(
            &mut metrics,
            json!({ "sources": [], "uptime": null, "form_sends": 2 }),
        );
        assert_eq!(
            metrics,
            json!({ "totals": {}, "uptime": { "share": 1.0 }, "sources": [], "form_sends": 2 })
        );
    }
}
