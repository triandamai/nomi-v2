//! Admin → Tools: every tool an agent can be given, each with its on/off switch and, where it
//! has any, its settings. Saved keys are encrypted and never sent back to the browser.

use std::collections::{HashMap, HashSet};

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use nomi_agent_core::tools::{self as tool_settings, REQUIRED_TOOLS};
use nomi_agent_core::voice::{self, VoiceSettings};
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
    /// Belongs to one built-in agent and is part of how that agent works: shown, not switchable.
    pub agent_only: bool,
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

/// `speak`'s settings as the admin page shows them: never the key, only where it comes from.
async fn voice_view(state: &AppState, conn: &mut sqlx::PgConnection) -> Result<(Value, bool), ApiError> {
    let (settings, secrets) = voice::voice_settings_with(conn, &state.settings_key).await.map_err(db_error)?;
    let key_source = voice::key_source(&secrets);
    Ok((
        json!({
            "voice_id": settings.voice_id,
            "voice_name": settings.voice_name,
            "model_id": settings.model_id,
            "stability": settings.stability,
            "similarity_boost": settings.similarity_boost,
            "style": settings.style,
            "use_speaker_boost": settings.use_speaker_boost,
            "speed": settings.speed,
            "output_format": settings.output_format,
            "output_formats": voice::OUTPUT_FORMATS,
            "max_chars": settings.max_chars,
            "max_chars_limit": voice::MAX_CHARS_LIMIT,
            "key_source": key_source,
            "env_var": voice::ENV_API_KEY,
            "models": voice::known_models(),
        }),
        key_source != voice::KeySource::Missing,
    ))
}

#[tracing::instrument(skip(state, claims))]
pub async fn list_tools(State(state): State<AppState>, AuthClaims(claims): AuthClaims) -> Result<Json<ToolsResponse>, ApiError> {
    require_system_config_permission(&claims)?;
    let mut conn = state.pool.acquire().await.map_err(db_error)?;

    let rows: Vec<(String, bool)> = sqlx::query_as("SELECT name, enabled FROM tool_settings").fetch_all(&mut *conn).await.map_err(db_error)?;
    let enabled: HashMap<String, bool> = rows.into_iter().collect();
    let (search_settings, search_ready) = web_search_view(&state, &mut conn).await?;
    let (voice_settings, voice_ready) = voice_view(&state, &mut conn).await?;
    let row = tool_settings::load_row(&mut conn, web::READ_WEB_PAGE_TOOL_NAME).await.map_err(db_error)?;
    let read_settings: ReadPageSettings = serde_json::from_value(row.config).unwrap_or_default();

    let entry = |def: &nomi_llm::ToolDefinition, used_by: Vec<String>, agent_only: bool| {
        let name = def.name.clone();
        let required = REQUIRED_TOOLS.contains(&name.as_str());
        let (settings, ready) = match name.as_str() {
            web::WEB_SEARCH_TOOL_NAME => (Some(search_settings.clone()), search_ready),
            web::READ_WEB_PAGE_TOOL_NAME => (Some(json!(read_settings)), true),
            voice::SPEAK_TOOL_NAME => (Some(voice_settings.clone()), voice_ready),
            _ => (None, true),
        };
        ToolEntry {
            enabled: required || agent_only || enabled.get(&name).copied().unwrap_or(true),
            required,
            agent_only,
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
        .map(|d| entry(d, Vec::new(), false))
        .collect();
    groups.push(ToolGroup { key: "crew".into(), kind: "crew", label: "Crew".into(), tools: crew });

    // Each built-in agent's own tools, shown under it: part of how it works, so always on.
    for agent in registry.agents() {
        let tools: Vec<ToolEntry> = agent
            .tools()
            .iter()
            .filter(|d| !listed.contains(&d.name))
            .map(|d| entry(d, vec![agent.display_name().into_owned()], true))
            .collect();
        if !tools.is_empty() {
            groups.push(ToolGroup { key: agent.agent_type().into_owned(), kind: "agent", label: agent.display_name().into_owned(), tools });
        }
    }

    // The tools custom agents can be granted. Their switch decides whether custom agents may use
    // them; a built-in agent that has the same tool keeps it.
    let custom: Vec<ToolEntry> =
        state.tool_catalog.definitions().iter().filter(|d| listed.insert(d.name.clone())).map(|d| entry(d, Vec::new(), false)).collect();
    if !custom.is_empty() {
        groups.push(ToolGroup { key: "custom".into(), kind: "custom", label: "Custom agents".into(), tools: custom });
    }

    Ok(Json(ToolsResponse { groups }))
}

enum Switchable {
    Yes,
    /// One built-in agent's own tool.
    AgentOnly,
    Unknown,
}

/// Whether the page offers a switch for `name`: crew tools and the tools custom agents can be
/// granted do; a built-in agent's own tools are part of that agent and don't.
fn switchable(state: &AppState, name: &str) -> Switchable {
    let registry = crate::build_agent_registry(state.project_storage.clone());
    if nomi_agent_core::crew_tool_definitions(&registry).iter().any(|d| d.name == name) || state.tool_catalog.known_tool_names().contains(&name) {
        Switchable::Yes
    } else if registry.agents().iter().any(|a| a.tools().iter().any(|d| d.name == name)) {
        Switchable::AgentOnly
    } else {
        Switchable::Unknown
    }
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
    match switchable(&state, &name) {
        Switchable::Yes => {}
        Switchable::AgentOnly => return Err((StatusCode::BAD_REQUEST, "this tool belongs to its agent and can't be switched off")),
        Switchable::Unknown => return Err((StatusCode::NOT_FOUND, "no such tool")),
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
        voice::SPEAK_TOOL_NAME => {
            let req: VoiceSettingsRequest = serde_json::from_value(body).map_err(|_| (StatusCode::BAD_REQUEST, "invalid settings"))?;
            let voice_id = req.voice_id.trim().to_string();
            if voice_id.is_empty() || !voice_id.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') {
                return Err((StatusCode::BAD_REQUEST, "pick a voice"));
            }
            let model_id = req.model_id.trim().to_string();
            if model_id.is_empty() {
                return Err((StatusCode::BAD_REQUEST, "pick a model"));
            }
            let settings = VoiceSettings {
                voice_id,
                voice_name: req.voice_name.map(|n| n.trim().to_string()).filter(|n| !n.is_empty()),
                model_id,
                stability: req.stability,
                similarity_boost: req.similarity_boost,
                style: req.style,
                use_speaker_boost: req.use_speaker_boost,
                speed: req.speed,
                output_format: req.output_format,
                max_chars: req.max_chars,
            }
            .clamped();
            let new_key = req.api_key.map(|k| k.trim().to_string()).filter(|k| !k.is_empty());
            let secrets = if new_key.is_some() || req.clear_api_key {
                let mut secrets = std::collections::HashMap::new();
                if let Some(key) = new_key {
                    secrets.insert(voice::secret_name().to_string(), key);
                }
                Some(tool_settings::encrypt_secrets_with(&state.settings_key, &secrets))
            } else {
                None
            };
            let config = serde_json::to_value(&settings).map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "couldn't save settings"))?;
            tool_settings::save_settings(&mut conn, &name, &config, secrets).await.map_err(db_error)?;
        }
        _ => return Err((StatusCode::NOT_FOUND, "this tool has no settings")),
    }
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
pub struct VoiceSettingsRequest {
    pub voice_id: String,
    pub voice_name: Option<String>,
    pub model_id: String,
    pub stability: f64,
    pub similarity_boost: f64,
    pub style: f64,
    pub use_speaker_boost: bool,
    pub speed: f64,
    pub output_format: String,
    pub max_chars: u32,
    /// A new key. Blank keeps the saved one.
    pub api_key: Option<String>,
    /// Forget the saved key.
    #[serde(default)]
    pub clear_api_key: bool,
}

#[derive(Serialize)]
pub struct VoiceOptionsResponse {
    pub voices: Vec<voice::VoiceOption>,
    pub models: Vec<voice::ModelOption>,
}

/// `GET /api/admin/tools/speak/voices`: the voices and models the saved key's account can use.
#[tracing::instrument(skip(state, claims))]
pub async fn voice_options(State(state): State<AppState>, AuthClaims(claims): AuthClaims) -> Result<Json<VoiceOptionsResponse>, (StatusCode, String)> {
    require_system_config_permission(&claims).map_err(|(s, m)| (s, m.to_string()))?;
    let mut conn = state.pool.acquire().await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let (_, secrets) = voice::voice_settings_with(&mut conn, &state.settings_key).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    drop(conn);
    let key = voice::api_key(&secrets).ok_or((StatusCode::CONFLICT, "save an ElevenLabs API key first".to_string()))?;
    let (voices, models) = voice::list_options(&key).await.map_err(|e| (StatusCode::BAD_GATEWAY, e))?;
    Ok(Json(VoiceOptionsResponse { voices, models }))
}

#[derive(Deserialize)]
pub struct TestVoiceRequest {
    pub text: Option<String>,
}

/// `POST /api/admin/tools/speak/test`: says a line with the saved settings and returns the
/// recording, so an admin hears the voice before the crew uses it.
#[tracing::instrument(skip(state, claims, req))]
pub async fn test_voice(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Json(req): Json<TestVoiceRequest>,
) -> Result<axum::response::Response, (StatusCode, String)> {
    use axum::response::IntoResponse;
    require_system_config_permission(&claims).map_err(|(s, m)| (s, m.to_string()))?;
    let mut conn = state.pool.acquire().await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let (settings, secrets) = voice::voice_settings_with(&mut conn, &state.settings_key).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    drop(conn);
    let key = voice::api_key(&secrets).ok_or((StatusCode::CONFLICT, "save an ElevenLabs API key first".to_string()))?;
    let text = req.text.map(|t| t.trim().chars().take(300).collect::<String>()).filter(|t| !t.is_empty());
    let text = text.unwrap_or_else(|| "Hi, I'm Nomi. This is how I sound when I read something out loud for you.".to_string());
    let audio = voice::synthesize(&settings, &key, &text).await.map_err(|e| (StatusCode::BAD_GATEWAY, e))?;
    Ok(([(axum::http::header::CONTENT_TYPE, "audio/mpeg")], audio).into_response())
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
