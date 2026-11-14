//! The apps a person can connect (`GET /api/connections/catalog`) and their own MCP servers
//! (`/api/connections/mcp`).
//!
//! The catalog is kept here in code: only Google Workspace (when this server has Google sign-in
//! set up), GitHub and Hevy can be connected today; the rest are listed as coming. MCP servers
//! are stored only: nothing in the crew calls MCP tools yet, and only remote (http/https)
//! servers are accepted. Nomi never runs a local command for anyone.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::app::AppState;
use nomi_auth::extractor::AuthClaims;

/// One app in the catalog.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct CatalogApp {
    pub id: &'static str,
    pub name: &'static str,
    /// `work`, `building`, `health`, `money` or `home_media`.
    pub category: &'static str,
    /// The crew member (agent type) it's for.
    pub for_agent: &'static str,
    pub access: &'static [&'static str],
    /// Whether it can be connected on this server today.
    pub available: bool,
}

/// id, name, category, for_agent, access.
type Entry = (
    &'static str,
    &'static str,
    &'static str,
    &'static str,
    &'static [&'static str],
);

pub const GOOGLE_WORKSPACE: &str = "google_workspace";
/// Connectable regardless of server setup (a key or token the person makes).
const ALWAYS_AVAILABLE: [&str; 2] = ["github", "hevy"];

const CATALOG: [Entry; 16] = [
    (
        GOOGLE_WORKSPACE,
        "Google Workspace",
        "work",
        "workspace",
        &["read", "write", "ask"],
    ),
    ("notion", "Notion", "work", "workspace", &["read", "write"]),
    (
        "slack",
        "Slack",
        "work",
        "workspace",
        &["read", "write", "ask"],
    ),
    ("todoist", "Todoist", "work", "planning", &["read", "write"]),
    (
        "microsoft_365",
        "Microsoft 365",
        "work",
        "workspace",
        &["read", "write", "ask"],
    ),
    ("github", "GitHub", "building", "coding", &["write"]),
    ("linear", "Linear", "building", "coding", &["read", "write"]),
    ("vercel", "Vercel", "building", "coding", &["read", "write"]),
    ("hevy", "Hevy", "health", "health", &["read", "write"]),
    ("strava", "Strava", "health", "health", &["read"]),
    ("oura", "Oura", "health", "health", &["read"]),
    ("fitbit", "Fitbit", "health", "health", &["read"]),
    ("ynab", "YNAB", "money", "money", &["read", "write"]),
    ("wise", "Wise", "money", "money", &["read"]),
    (
        "spotify",
        "Spotify",
        "home_media",
        "chitchat",
        &["read", "write"],
    ),
    (
        "home_assistant",
        "Home Assistant",
        "home_media",
        "chitchat",
        &["read", "write", "ask"],
    ),
];

/// The catalog, with Google available only when `google_configured`.
pub fn catalog(google_configured: bool) -> Vec<CatalogApp> {
    CATALOG
        .iter()
        .map(|(id, name, category, for_agent, access)| CatalogApp {
            id,
            name,
            category,
            for_agent,
            access,
            available: ALWAYS_AVAILABLE.contains(id)
                || (*id == GOOGLE_WORKSPACE && google_configured),
        })
        .collect()
}

/// `GET /api/connections/catalog`
pub async fn get_catalog(AuthClaims(_claims): AuthClaims) -> Json<Vec<CatalogApp>> {
    let google = nomi_agent_workspace::connection::GoogleConfig::current()
        .await
        .is_some();
    Json(catalog(google))
}

/// The most MCP servers one person keeps.
pub const MAX_MCP_SERVERS: i64 = 20;
pub const MAX_MCP_NAME_CHARS: usize = 80;
pub const MAX_MCP_URL_CHARS: usize = 2048;

#[derive(Serialize, sqlx::FromRow)]
pub struct McpServer {
    pub id: Uuid,
    pub name: String,
    pub url: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct McpServers {
    pub servers: Vec<McpServer>,
    /// Whether the crew can call MCP tools yet (not yet: servers are kept for when it can).
    pub callable: bool,
}

/// Nothing in the crew calls MCP tools yet.
const MCP_CALLABLE: bool = false;

type ApiError = (StatusCode, Json<serde_json::Value>);

fn api_error(status: StatusCode, code: &str) -> ApiError {
    (status, Json(serde_json::json!({ "error": code })))
}

fn server_error<E: std::fmt::Display>(e: E) -> ApiError {
    tracing::error!(error = %e, "mcp servers request failed");
    api_error(StatusCode::INTERNAL_SERVER_ERROR, "server_error")
}

/// `GET /api/connections/mcp`
pub async fn list_mcp(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
) -> Result<Json<McpServers>, ApiError> {
    let servers = sqlx::query_as(
        "SELECT id, name, url, created_at FROM mcp_servers WHERE user_id = $1 ORDER BY created_at",
    )
    .bind(claims.sub)
    .fetch_all(&state.pool)
    .await
    .map_err(server_error)?;
    Ok(Json(McpServers {
        servers,
        callable: MCP_CALLABLE,
    }))
}

#[derive(Deserialize)]
pub struct AddMcpServer {
    pub name: String,
    pub url: Option<String>,
    /// A local command: refused (`400 command_not_supported`), never stored or run.
    pub command: Option<String>,
}

/// Why an MCP server address isn't accepted, as the error code the page words.
pub fn check_mcp_url(url: &str) -> Result<String, &'static str> {
    let url = url.trim();
    if url.is_empty() {
        return Err("missing_url");
    }
    if url.chars().count() > MAX_MCP_URL_CHARS {
        return Err("invalid_url");
    }
    let parsed = reqwest::Url::parse(url).map_err(|_| "invalid_url")?;
    if !matches!(parsed.scheme(), "http" | "https") || parsed.host_str().is_none_or(str::is_empty) {
        return Err("invalid_url");
    }
    Ok(parsed.to_string())
}

/// `POST /api/connections/mcp` with `{ name, url }` → `201` with the server. `400 missing_name`,
/// `missing_url`, `invalid_url` (http or https only), `command_not_supported`; `409 already_added`,
/// `too_many`.
pub async fn add_mcp(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Json(req): Json<AddMcpServer>,
) -> Result<(StatusCode, Json<McpServer>), ApiError> {
    if req.command.as_deref().is_some_and(|c| !c.trim().is_empty()) {
        return Err(api_error(StatusCode::BAD_REQUEST, "command_not_supported"));
    }
    let name = req.name.trim();
    if name.is_empty() || name.chars().count() > MAX_MCP_NAME_CHARS {
        return Err(api_error(StatusCode::BAD_REQUEST, "missing_name"));
    }
    let url = check_mcp_url(req.url.as_deref().unwrap_or_default())
        .map_err(|code| api_error(StatusCode::BAD_REQUEST, code))?;
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM mcp_servers WHERE user_id = $1")
        .bind(claims.sub)
        .fetch_one(&state.pool)
        .await
        .map_err(server_error)?;
    if count >= MAX_MCP_SERVERS {
        return Err(api_error(StatusCode::CONFLICT, "too_many"));
    }
    let added: Option<McpServer> = sqlx::query_as(
        "INSERT INTO mcp_servers (user_id, name, url) VALUES ($1, $2, $3) ON CONFLICT (user_id, url) DO NOTHING \
         RETURNING id, name, url, created_at",
    )
    .bind(claims.sub)
    .bind(name)
    .bind(&url)
    .fetch_optional(&state.pool)
    .await
    .map_err(server_error)?;
    let added = added.ok_or_else(|| api_error(StatusCode::CONFLICT, "already_added"))?;
    Ok((StatusCode::CREATED, Json(added)))
}

/// `DELETE /api/connections/mcp/:id` → `204`; someone else's (or a missing) one is `404`.
pub async fn delete_mcp(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    let removed = sqlx::query("DELETE FROM mcp_servers WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(claims.sub)
        .execute(&state.pool)
        .await
        .map_err(server_error)?
        .rows_affected();
    if removed == 0 {
        return Err(api_error(StatusCode::NOT_FOUND, "not_found"));
    }
    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_what_connects_today_is_available() {
        let without_google: Vec<_> = catalog(false)
            .into_iter()
            .filter(|a| a.available)
            .map(|a| a.id)
            .collect();
        assert_eq!(without_google, vec!["github", "hevy"]);
        assert!(catalog(true)
            .iter()
            .any(|a| a.id == GOOGLE_WORKSPACE && a.available));
    }

    #[test]
    fn only_remote_servers_are_accepted() {
        assert_eq!(
            check_mcp_url(" https://mcp.example.com/sse "),
            Ok("https://mcp.example.com/sse".to_string())
        );
        assert_eq!(check_mcp_url(""), Err("missing_url"));
        assert_eq!(check_mcp_url("file:///etc/passwd"), Err("invalid_url"));
        assert_eq!(check_mcp_url("npx some-server"), Err("invalid_url"));
    }
}
