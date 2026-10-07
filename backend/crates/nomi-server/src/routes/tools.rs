//! Admin → Tools: every tool an agent can be given, each with its on/off switch and, where it
//! has any, its settings. Saved keys are encrypted and never sent back to the browser.

use std::collections::{HashMap, HashSet};

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use nomi_agent_core::tools::{self as tool_settings, REQUIRED_TOOLS};
use nomi_agent_core::web::{self, ReadPageSettings, SearchHit, SearchProvider, WebSearchSettings, MAX_RESULTS_LIMIT};
use nomi_auth::extractor::AuthClaims;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::app::AppState;
use crate::routes::settings::require_system_config_permission;

type ApiError = (StatusCode, &'static str);

fn db_error(e: sqlx::Error) -> ApiError {
    tracing::error!(error = %e, "tools: database error");
    (StatusCode::INTERNAL_SERVER_ERROR, "database error")
}

#[derive(Serialize)]
pub struct ToolEntry {
    pub name: String,
    pub description: String,
    pub enabled: bool,
    /// Can't be switched off: the crew needs it.
    pub required: bool,
    /// Has settings an admin can edit.
    pub configurable: bool,
    /// Usable as set up. Only `web_search` can be on but not ready (no key yet).
    pub ready: bool,
    /// Agents that have this tool (agent tools only).
    pub used_by: Vec<String>,
    pub settings: Option<Value>,
}

#[derive(Serialize)]
pub struct ToolGroup {
    /// `crew`, `custom`, or the agent type that owns the tools.
    pub key: String,
    pub kind: &'static str,
    pub label: String,
    pub tools: Vec<ToolEntry>,
}

#[derive(Serialize)]
pub struct ToolsResponse {
    pub groups: Vec<ToolGroup>,
}

/// `web_search`'s settings as the admin page shows them: never the keys, only where each comes from.
async fn web_search_view(state: &AppState, conn: &mut sqlx::PgConnection) -> Result<(Value, bool), ApiError> {
    let (settings, secrets) = web::search_settings_with(conn, &state.settings_key).await.map_err(db_error)?;
    let providers: Vec<Value> = SearchProvider::ALL
        .into_iter()
        .map(|p| {
            json!({
                "id": p.as_str(),
                "needs_key": p.needs_key(),
                "env_var": p.env_var(),
                "key_source": web::key_source(p, &settings, &secrets),
            })
        })
        .collect();
    let ready = web::setup_from(&settings, &secrets).is_some();
    Ok((
        json!({
            "provider": settings.provider,
            "max_results": settings.max_results,
            "max_results_limit": MAX_RESULTS_LIMIT,
            "base_url": settings.base_url,
            "providers": providers,
        }),
        ready,
    ))
}

#[tracing::instrument(skip(state, claims))]
pub async fn list_tools(State(state): State<AppState>, AuthClaims(claims): AuthClaims) -> Result<Json<ToolsResponse>, ApiError> {
    require_system_config_permission(&claims)?;
    let mut conn = state.pool.acquire().await.map_err(db_error)?;

    let rows: Vec<(String, bool)> = sqlx::query_as("SELECT name, enabled FROM tool_settings").fetch_all(&mut *conn).await.map_err(db_error)?;
    let enabled: HashMap<String, bool> = rows.into_iter().collect();
    let (search_settings, search_ready) = web_search_view(&state, &mut conn).await?;
    let row = tool_settings::load_row(&mut conn, web::READ_WEB_PAGE_TOOL_NAME).await.map_err(db_error)?;
    let read_settings: ReadPageSettings = serde_json::from_value(row.config).unwrap_or_default();

    let entry = |def: &nomi_llm::ToolDefinition, used_by: Vec<String>| {
        let name = def.name.clone();
        let required = REQUIRED_TOOLS.contains(&name.as_str());
        let (settings, ready) = match name.as_str() {
            web::WEB_SEARCH_TOOL_NAME => (Some(search_settings.clone()), search_ready),
            web::READ_WEB_PAGE_TOOL_NAME => (Some(json!(read_settings)), true),
            _ => (None, true),
        };
        ToolEntry {
            enabled: required || enabled.get(&name).copied().unwrap_or(true),
            required,
            configurable: settings.is_some(),
            ready,
            used_by,
            settings,
            description: def.description.clone(),
            name,
        }
    };

    let registry = crate::build_agent_registry(state.project_storage.clone());
    let mut listed: HashSet<String> = HashSet::new();
    let mut groups = Vec::new();

    let crew: Vec<ToolEntry> = nomi_agent_core::crew_tool_definitions(&registry)
        .iter()
        .filter(|d| listed.insert(d.name.clone()))
        .map(|d| entry(d, Vec::new()))
        .collect();
    groups.push(ToolGroup { key: "crew".into(), kind: "crew", label: "Crew".into(), tools: crew });

    // Who has each agent tool, so a tool shared by agents is listed once, under its first owner.
    let mut owners: HashMap<String, Vec<String>> = HashMap::new();
    for agent in registry.agents() {
        for def in agent.tools() {
            owners.entry(def.name).or_default().push(agent.display_name().into_owned());
        }
    }
    for agent in registry.agents() {
        let tools: Vec<ToolEntry> = agent
            .tools()
            .iter()
            .filter(|d| listed.insert(d.name.clone()))
            .map(|d| entry(d, owners.get(&d.name).cloned().unwrap_or_default()))
            .collect();
        if !tools.is_empty() {
            groups.push(ToolGroup { key: agent.agent_type().into_owned(), kind: "agent", label: agent.display_name().into_owned(), tools });
        }
    }

    let custom: Vec<ToolEntry> =
        state.tool_catalog.definitions().iter().filter(|d| listed.insert(d.name.clone())).map(|d| entry(d, Vec::new())).collect();
    if !custom.is_empty() {
        groups.push(ToolGroup { key: "custom".into(), kind: "custom", label: "Custom agents".into(), tools: custom });
    }

    Ok(Json(ToolsResponse { groups }))
}

/// Every tool name the page lists (switches and settings are only saved for these).
fn known_tool(state: &AppState, name: &str) -> bool {
    let registry = crate::build_agent_registry(state.project_storage.clone());
    nomi_agent_core::crew_tool_definitions(&registry).iter().any(|d| d.name == name)
        || registry.agents().iter().any(|a| a.tools().iter().any(|d| d.name == name))
        || state.tool_catalog.known_tool_names().contains(&name)
}

#[derive(Deserialize)]
pub struct SetEnabledRequest {
    pub enabled: bool,
}

#[tracing::instrument(skip(state, claims, req))]
pub async fn set_tool_enabled(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(name): Path<String>,
    Json(req): Json<SetEnabledRequest>,
) -> Result<StatusCode, ApiError> {
    require_system_config_permission(&claims)?;
    if !known_tool(&state, &name) {
        return Err((StatusCode::NOT_FOUND, "no such tool"));
    }
    if REQUIRED_TOOLS.contains(&name.as_str()) && !req.enabled {
        return Err((StatusCode::BAD_REQUEST, "this tool can't be switched off"));
    }
    let mut conn = state.pool.acquire().await.map_err(db_error)?;
    tool_settings::set_enabled(&mut conn, &name, req.enabled).await.map_err(db_error)?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
pub struct WebSearchSettingsRequest {
    pub provider: String,
    pub max_results: Option<u8>,
    pub base_url: Option<String>,
    /// A new key for `provider`. Blank keeps the saved one.
    pub api_key: Option<String>,
    /// Forget `provider`'s saved key.
    #[serde(default)]
    pub clear_api_key: bool,
}

#[derive(Deserialize)]
pub struct ReadPageSettingsRequest {
    pub max_chars: usize,
}

const MIN_PAGE_CHARS: usize = 1_000;
const MAX_PAGE_CHARS: usize = 50_000;

fn clean_base_url(raw: Option<String>) -> Result<Option<String>, ApiError> {
    let Some(url) = raw.map(|u| u.trim().trim_end_matches('/').to_string()).filter(|u| !u.is_empty()) else { return Ok(None) };
    match reqwest::Url::parse(&url) {
        Ok(parsed) if matches!(parsed.scheme(), "http" | "https") && parsed.host_str().is_some() => Ok(Some(url)),
        _ => Err((StatusCode::BAD_REQUEST, "the address must be a full http(s) URL")),
    }
}

#[tracing::instrument(skip(state, claims, body))]
pub async fn save_tool_settings(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(name): Path<String>,
    Json(body): Json<Value>,
) -> Result<StatusCode, ApiError> {
    require_system_config_permission(&claims)?;
    let mut conn = state.pool.acquire().await.map_err(db_error)?;
    match name.as_str() {
        web::WEB_SEARCH_TOOL_NAME => {
            let req: WebSearchSettingsRequest = serde_json::from_value(body).map_err(|_| (StatusCode::BAD_REQUEST, "invalid settings"))?;
            let provider = SearchProvider::parse(&req.provider).ok_or((StatusCode::BAD_REQUEST, "unknown search provider"))?;
            let settings = WebSearchSettings {
                provider,
                max_results: req.max_results.unwrap_or(5).clamp(1, MAX_RESULTS_LIMIT),
                base_url: clean_base_url(req.base_url)?,
            };
            let new_key = req.api_key.map(|k| k.trim().to_string()).filter(|k| !k.is_empty());
            let secrets = if new_key.is_some() || req.clear_api_key {
                let row = tool_settings::load_row(&mut conn, &name).await.map_err(db_error)?;
                let mut secrets = tool_settings::decrypt_secrets_with(&state.settings_key, row.secrets_encrypted.as_deref());
                match new_key {
                    Some(key) => secrets.insert(provider.as_str().to_string(), key),
                    None => secrets.remove(provider.as_str()),
                };
                Some(tool_settings::encrypt_secrets_with(&state.settings_key, &secrets))
            } else {
                None
            };
            let config = serde_json::to_value(&settings).map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "couldn't save settings"))?;
            tool_settings::save_settings(&mut conn, &name, &config, secrets).await.map_err(db_error)?;
        }
        web::READ_WEB_PAGE_TOOL_NAME => {
            let req: ReadPageSettingsRequest = serde_json::from_value(body).map_err(|_| (StatusCode::BAD_REQUEST, "invalid settings"))?;
            let settings = ReadPageSettings { max_chars: req.max_chars.clamp(MIN_PAGE_CHARS, MAX_PAGE_CHARS) };
            let config = serde_json::to_value(&settings).map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "couldn't save settings"))?;
            tool_settings::save_settings(&mut conn, &name, &config, None).await.map_err(db_error)?;
        }
        _ => return Err((StatusCode::NOT_FOUND, "this tool has no settings")),
    }
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
pub struct TestSearchRequest {
    pub query: Option<String>,
}

#[derive(Serialize)]
pub struct TestSearchResponse {
    pub ok: bool,
    pub provider: SearchProvider,
    pub results: Vec<SearchHit>,
    pub error: Option<String>,
}

/// Runs one search with the saved settings, so an admin can check the key works.
#[tracing::instrument(skip(state, claims, req))]
pub async fn test_web_search(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Json(req): Json<TestSearchRequest>,
) -> Result<Json<TestSearchResponse>, ApiError> {
    require_system_config_permission(&claims)?;
    let mut conn = state.pool.acquire().await.map_err(db_error)?;
    let (settings, secrets) = web::search_settings_with(&mut conn, &state.settings_key).await.map_err(db_error)?;
    drop(conn);
    let Some(setup) = web::setup_from(&settings, &secrets) else {
        let error = if settings.provider.needs_key() { "no API key saved for this provider" } else { "no SearXNG address saved" };
        return Ok(Json(TestSearchResponse { ok: false, provider: settings.provider, results: Vec::new(), error: Some(error.to_string()) }));
    };
    let query = req.query.map(|q| q.trim().to_string()).filter(|q| !q.is_empty()).unwrap_or_else(|| "latest news".to_string());
    Ok(Json(match web::search(&setup, &query).await {
        Ok(results) => TestSearchResponse { ok: true, provider: setup.provider, results, error: None },
        Err(error) => TestSearchResponse { ok: false, provider: setup.provider, results: Vec::new(), error: Some(error) },
    }))
}
