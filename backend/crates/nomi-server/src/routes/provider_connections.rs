//! Providers: the catalog people connect from, and their connections — the instance's (Admin →
//! Providers, area "models") and each person's own (Providers). Keys and header values go in and
//! never come back out: responses carry a masked key and header names only.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::app::AppState;
use crate::bootstrap::StoredHeader;
use nomi_auth::claims::Claims;
use nomi_auth::extractor::AuthClaims;
use nomi_llm::catalog::{self, AuthType, Category, ModelMetadata, ProviderDefinition};
use nomi_llm::{LlmError, ModelConfig};
use nomi_settings::{self as settings, provider_connections as connections, provider_connections::Scope};

type RouteError = (StatusCode, Json<Value>);

/// `{ "error": code }`, the codes the frontend turns into messages.
fn error(status: StatusCode, code: &str, detail: Option<String>) -> RouteError {
    (status, Json(json!({ "error": code, "detail": detail })))
}

fn server_error(what: &str, e: impl std::fmt::Display) -> RouteError {
    tracing::error!(error = %e, "{what}");
    error(StatusCode::INTERNAL_SERVER_ERROR, "server_error", None)
}

// ---------------------------------------------------------------------------------------------
// Catalog

#[derive(Serialize)]
pub struct ProviderResponse {
    id: &'static str,
    name: &'static str,
    description: &'static str,
    icon: String,
    has_logo: bool,
    category: Category,
    recommended: bool,
    auth_type: AuthType,
    kind: &'static str,
    default_base_url: Option<&'static str>,
    doc_url: Option<&'static str>,
    model_count: usize,
}

impl From<&'static ProviderDefinition> for ProviderResponse {
    fn from(p: &'static ProviderDefinition) -> Self {
        ProviderResponse {
            id: &p.id,
            name: &p.name,
            description: &p.description,
            icon: format!("/provider-logos/{}.svg", p.id),
            has_logo: p.has_logo,
            category: p.category,
            recommended: p.recommended,
            auth_type: p.auth_type,
            kind: p.kind.as_str(),
            default_base_url: p.default_base_url.as_deref(),
            doc_url: p.doc_url.as_deref(),
            model_count: p.models.len(),
        }
    }
}

/// `GET /api/llm/providers`: every provider that can be connected, without their models.
pub async fn list_providers(AuthClaims(_claims): AuthClaims) -> Json<Value> {
    let providers: Vec<ProviderResponse> = catalog::providers().iter().map(ProviderResponse::from).collect();
    Json(json!({ "providers": providers }))
}

/// `GET /api/llm/providers/:id/models`: a provider's models, with prices and what they read.
pub async fn list_provider_catalog_models(AuthClaims(_claims): AuthClaims, Path(id): Path<String>) -> Result<Json<Value>, RouteError> {
    let provider = catalog::provider(&id).ok_or_else(|| error(StatusCode::NOT_FOUND, "not_found", None))?;
    let models: &[ModelMetadata] = &provider.models;
    Ok(Json(json!({ "models": models })))
}

// ---------------------------------------------------------------------------------------------
// Connections

#[derive(Serialize)]
pub struct ConnectionResponse {
    id: Uuid,
    provider_id: String,
    label: String,
    auth_type: String,
    kind: String,
    base_url: Option<String>,
    header_names: Vec<String>,
    api_key_masked: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
    /// Admin models that come from it (instance connections only).
    models_in_use: Vec<String>,
}

#[derive(Deserialize)]
pub struct ConnectRequest {
    provider_id: String,
    #[serde(default)]
    api_key: Option<String>,
    #[serde(default)]
    label: Option<String>,
    #[serde(default)]
    base_url: Option<String>,
    #[serde(default)]
    headers: Vec<StoredHeader>,
}

#[derive(Deserialize)]
pub struct UpdateConnectionRequest {
    #[serde(default)]
    api_key: Option<String>,
    #[serde(default)]
    label: Option<String>,
    #[serde(default)]
    base_url: Option<String>,
    #[serde(default)]
    headers: Option<Vec<StoredHeader>>,
}

/// Headers Nomi sets itself, or that would let a custom header reach past the endpoint.
const RESERVED_HEADERS: [&str; 11] =
    ["authorization", "proxy-authorization", "host", "content-length", "content-type", "connection", "transfer-encoding", "te", "upgrade", "cookie", "expect"];

fn validate_headers(headers: &[StoredHeader]) -> Result<(), RouteError> {
    let invalid = |detail: &str| Err(error(StatusCode::BAD_REQUEST, "header_invalid", Some(detail.to_string())));
    if headers.len() > 20 {
        return invalid("at most 20 headers");
    }
    for header in headers {
        let name = header.name.trim();
        if name.is_empty() || name.len() > 64 || !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
            return invalid("header names use letters, digits, - and _");
        }
        if RESERVED_HEADERS.contains(&name.to_ascii_lowercase().as_str()) {
            return invalid("that header is set by Nomi");
        }
        if header.value.len() > 2048 || header.value.chars().any(|c| c.is_control()) {
            return invalid("header values are one line of text");
        }
    }
    Ok(())
}

/// A base URL Nomi may call: https on a public host (http and local hosts only where
/// NOMI_WEB_ALLOW_PRIVATE=1, i.e. development and tests), without a trailing slash.
async fn validate_base_url(raw: &str) -> Result<String, RouteError> {
    let invalid = |detail: String| error(StatusCode::BAD_REQUEST, "base_url_invalid", Some(detail));
    let url = reqwest::Url::parse(raw.trim()).map_err(|_| invalid("not a URL".to_string()))?;
    let allow_private = std::env::var("NOMI_WEB_ALLOW_PRIVATE").is_ok_and(|v| v == "1");
    match url.scheme() {
        "https" => {}
        "http" if allow_private => {}
        _ => return Err(invalid("use an https:// address".to_string())),
    }
    if !url.username().is_empty() || url.password().is_some() || url.query().is_some() || url.fragment().is_some() {
        return Err(invalid("no credentials, query or fragment in the address".to_string()));
    }
    let host = url.host_str().ok_or_else(|| invalid("no host".to_string()))?;
    nomi_agent_core::web::ensure_public_host(host, url.port_or_known_default().unwrap_or(443)).await.map_err(invalid)?;
    Ok(url.as_str().trim_end_matches('/').to_string())
}

/// Whether the provider accepts the settings, by listing its models (no tokens spent). A custom
/// endpoint without a model list is accepted; its models are typed in.
async fn verify(state: &AppState, config: ModelConfig, custom: bool) -> Result<(), RouteError> {
    match nomi_llm::list_provider_models(config, state.http_client.clone()).await {
        Ok(_) => Ok(()),
        Err(e) => match provider_status(&e) {
            Some(401 | 403) => Err(error(StatusCode::BAD_REQUEST, "key_rejected", Some(e.to_string()))),
            Some(404 | 405) if custom => Ok(()),
            _ => Err(error(StatusCode::BAD_GATEWAY, "unreachable", Some(e.to_string()))),
        },
    }
}

/// The HTTP status in a provider error ("openai returned 401 Unauthorized: …").
fn provider_status(e: &LlmError) -> Option<u16> {
    match e {
        LlmError::ProviderError(text) => text.split_once(" returned ")?.1.split_whitespace().next()?.parse().ok(),
        _ => None,
    }
}

fn encrypt_headers(state: &AppState, headers: &[StoredHeader]) -> Option<Vec<u8>> {
    (!headers.is_empty()).then(|| settings::crypto::encrypt(&state.settings_key, &serde_json::to_string(headers).expect("headers serialize")))
}

fn header_names(state: &AppState, row: &connections::ProviderConnectionRow) -> Vec<String> {
    let Some(encrypted) = &row.headers_encrypted else { return Vec::new() };
    settings::crypto::decrypt(&state.settings_key, encrypted)
        .ok()
        .and_then(|json| serde_json::from_str::<Vec<StoredHeader>>(&json).ok())
        .map(|headers| headers.into_iter().map(|h| h.name).collect())
        .unwrap_or_default()
}

async fn to_response(state: &AppState, row: connections::ProviderConnectionRow) -> Result<ConnectionResponse, RouteError> {
    let api_key_masked = match &row.api_key_encrypted {
        Some(encrypted) => Some(settings::mask_api_key(&settings::crypto::decrypt(&state.settings_key, encrypted).map_err(|e| server_error("failed to read a provider key", e))?)),
        None => None,
    };
    let models_in_use = if row.user_id.is_none() {
        connections::models_using(&state.pool, row.id).await.map_err(|e| server_error("failed to load models", e))?
    } else {
        Vec::new()
    };
    Ok(ConnectionResponse {
        header_names: header_names(state, &row),
        id: row.id,
        provider_id: row.provider_id,
        label: row.label,
        auth_type: row.auth_type,
        kind: row.kind,
        base_url: row.base_url,
        api_key_masked,
        created_at: row.created_at,
        models_in_use,
    })
}

async fn list(state: &AppState, scope: Scope) -> Result<Json<Value>, RouteError> {
    let rows = connections::list_connections(&state.pool, scope).await.map_err(|e| server_error("failed to list providers", e))?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        out.push(to_response(state, row).await?);
    }
    Ok(Json(json!({ "connections": out })))
}

async fn connect(state: &AppState, scope: Scope, by: Uuid, req: ConnectRequest) -> Result<(StatusCode, Json<ConnectionResponse>), RouteError> {
    let provider = catalog::provider(&req.provider_id).ok_or_else(|| error(StatusCode::BAD_REQUEST, "unknown_provider", None))?;
    let custom = provider.auth_type == AuthType::CustomEndpoint;
    let api_key = req.api_key.as_deref().map(str::trim).unwrap_or_default().to_string();
    let label = if custom { req.label.as_deref().map(str::trim).unwrap_or_default().to_string() } else { provider.name.clone() };
    if custom && (label.is_empty() || label.chars().count() > 60) {
        return Err(error(StatusCode::BAD_REQUEST, "label_required", None));
    }
    if !custom && api_key.is_empty() {
        return Err(error(StatusCode::BAD_REQUEST, "key_required", None));
    }
    let base_url = match req.base_url.as_deref().filter(|u| !u.trim().is_empty()) {
        Some(raw) => Some(validate_base_url(raw).await?),
        None if custom => return Err(error(StatusCode::BAD_REQUEST, "base_url_required", None)),
        None => None,
    };
    if custom {
        validate_headers(&req.headers)?;
    }
    let headers = if custom { req.headers } else { Vec::new() };

    let config = ModelConfig {
        provider: provider.kind,
        model_id: String::new(),
        api_key: api_key.clone(),
        base_url: base_url.clone().or_else(|| provider.default_base_url.clone()),
        extra_headers: headers.iter().map(|h| (h.name.trim().to_string(), h.value.clone())).collect(),
    };
    verify(state, config, custom).await?;

    let row = connections::create_connection(
        &state.pool,
        scope,
        connections::NewConnection {
            provider_id: &provider.id,
            label: &label,
            kind: provider.kind.as_str(),
            auth_type: if custom { "custom_endpoint" } else { "api_key" },
            api_key_encrypted: (!api_key.is_empty()).then(|| settings::crypto::encrypt(&state.settings_key, &api_key)),
            base_url: base_url.as_deref(),
            headers_encrypted: encrypt_headers(state, &headers),
            updated_by: by,
        },
    )
    .await
    .map_err(|e| match e {
        connections::CreateConnectionError::AlreadyConnected => error(StatusCode::CONFLICT, "already_connected", None),
        connections::CreateConnectionError::Database(e) => server_error("failed to save a provider", e),
    })?;
    Ok((StatusCode::CREATED, Json(to_response(state, row).await?)))
}

async fn update(state: &AppState, scope: Scope, by: Uuid, id: Uuid, req: UpdateConnectionRequest) -> Result<Json<ConnectionResponse>, RouteError> {
    let existing = connections::get_connection(&state.pool, scope, id)
        .await
        .map_err(|e| server_error("failed to load a provider", e))?
        .ok_or_else(|| error(StatusCode::NOT_FOUND, "not_found", None))?;
    let custom = existing.auth_type == "custom_endpoint";
    let api_key = req.api_key.as_deref().map(str::trim).filter(|k| !k.is_empty()).map(str::to_string);
    let label = if custom { req.label.as_deref().map(str::trim).filter(|l| !l.is_empty()).map(str::to_string) } else { None };
    let base_url = match req.base_url.as_deref().filter(|u| !u.trim().is_empty()) {
        Some(raw) => Some(validate_base_url(raw).await?),
        None => None,
    };
    let headers = if custom { req.headers } else { None };
    if let Some(headers) = &headers {
        validate_headers(headers)?;
    }

    // Check what the connection will be, before saving it.
    let mut config = crate::bootstrap::connection_model_config(&state.pool, &state.settings_key, scope, id, String::new())
        .await
        .ok_or_else(|| server_error("failed to read a provider", "unreadable connection"))?;
    if let Some(key) = &api_key {
        config.api_key = key.clone();
    }
    if let Some(url) = &base_url {
        config.base_url = Some(url.clone());
    }
    if let Some(headers) = &headers {
        config.extra_headers = headers.iter().map(|h| (h.name.trim().to_string(), h.value.clone())).collect();
    }
    if api_key.is_some() || base_url.is_some() || headers.is_some() {
        verify(state, config, custom).await?;
    }

    let row = connections::update_connection(
        &state.pool,
        scope,
        id,
        connections::ConnectionUpdate {
            label: label.as_deref(),
            api_key_encrypted: api_key.map(|k| settings::crypto::encrypt(&state.settings_key, &k)),
            base_url: base_url.as_deref(),
            headers_encrypted: headers.as_deref().map(|h| encrypt_headers(state, h).unwrap_or_else(|| settings::crypto::encrypt(&state.settings_key, "[]"))),
            updated_by: by,
        },
    )
    .await
    .map_err(|e| match &e {
        sqlx::Error::Database(db) if db.is_unique_violation() => error(StatusCode::CONFLICT, "already_connected", None),
        _ => server_error("failed to save a provider", e),
    })?
    .ok_or_else(|| error(StatusCode::NOT_FOUND, "not_found", None))?;
    Ok(Json(to_response(state, row).await?))
}

async fn disconnect(state: &AppState, scope: Scope, id: Uuid) -> Result<StatusCode, RouteError> {
    connections::delete_connection(&state.pool, scope, id).await.map_err(|e| match e {
        connections::DeleteConnectionError::NotFound => error(StatusCode::NOT_FOUND, "not_found", None),
        connections::DeleteConnectionError::InUse(models) => (StatusCode::CONFLICT, Json(json!({ "error": "in_use", "models": models }))),
        connections::DeleteConnectionError::Database(e) => server_error("failed to remove a provider", e),
    })?;
    Ok(StatusCode::NO_CONTENT)
}

/// What the provider serves right now, using the connection's own key.
async fn live_models(state: &AppState, scope: Scope, id: Uuid) -> Result<Json<Value>, RouteError> {
    let config = crate::bootstrap::connection_model_config(&state.pool, &state.settings_key, scope, id, String::new())
        .await
        .ok_or_else(|| error(StatusCode::NOT_FOUND, "not_found", None))?;
    let models = nomi_llm::list_provider_models(config, state.http_client.clone()).await.map_err(|e| match provider_status(&e) {
        Some(401 | 403) => error(StatusCode::BAD_REQUEST, "key_rejected", Some(e.to_string())),
        _ => error(StatusCode::BAD_GATEWAY, "unreachable", Some(e.to_string())),
    })?;
    Ok(Json(json!({ "models": models })))
}

fn require_admin(claims: &Claims, action: &str) -> Result<(), RouteError> {
    crate::routes::admin_access::require_admin(claims, "models", action).map_err(|(status, message)| error(status, "forbidden", Some(message.to_string())))
}

// Personal: /api/llm/connections

pub async fn list_own(State(state): State<AppState>, AuthClaims(claims): AuthClaims) -> Result<Json<Value>, RouteError> {
    list(&state, Scope::User(claims.sub)).await
}

pub async fn connect_own(State(state): State<AppState>, AuthClaims(claims): AuthClaims, Json(req): Json<ConnectRequest>) -> Result<(StatusCode, Json<ConnectionResponse>), RouteError> {
    connect(&state, Scope::User(claims.sub), claims.sub, req).await
}

pub async fn update_own(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateConnectionRequest>,
) -> Result<Json<ConnectionResponse>, RouteError> {
    update(&state, Scope::User(claims.sub), claims.sub, id, req).await
}

pub async fn disconnect_own(State(state): State<AppState>, AuthClaims(claims): AuthClaims, Path(id): Path<Uuid>) -> Result<StatusCode, RouteError> {
    disconnect(&state, Scope::User(claims.sub), id).await
}

pub async fn own_models(State(state): State<AppState>, AuthClaims(claims): AuthClaims, Path(id): Path<Uuid>) -> Result<Json<Value>, RouteError> {
    live_models(&state, Scope::User(claims.sub), id).await
}

// Instance: /api/admin/settings/llm/connections

pub async fn list_shared(State(state): State<AppState>, AuthClaims(claims): AuthClaims) -> Result<Json<Value>, RouteError> {
    require_admin(&claims, "view")?;
    list(&state, Scope::Admin).await
}

pub async fn connect_shared(State(state): State<AppState>, AuthClaims(claims): AuthClaims, Json(req): Json<ConnectRequest>) -> Result<(StatusCode, Json<ConnectionResponse>), RouteError> {
    require_admin(&claims, "write")?;
    connect(&state, Scope::Admin, claims.sub, req).await
}

pub async fn update_shared(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateConnectionRequest>,
) -> Result<Json<ConnectionResponse>, RouteError> {
    require_admin(&claims, "write")?;
    update(&state, Scope::Admin, claims.sub, id, req).await
}

pub async fn disconnect_shared(State(state): State<AppState>, AuthClaims(claims): AuthClaims, Path(id): Path<Uuid>) -> Result<StatusCode, RouteError> {
    require_admin(&claims, "manage")?;
    disconnect(&state, Scope::Admin, id).await
}

pub async fn shared_models(State(state): State<AppState>, AuthClaims(claims): AuthClaims, Path(id): Path<Uuid>) -> Result<Json<Value>, RouteError> {
    require_admin(&claims, "view")?;
    live_models(&state, Scope::Admin, id).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_status_out_of_a_provider_error() {
        assert_eq!(provider_status(&LlmError::ProviderError("openai returned 401 Unauthorized: bad key".to_string())), Some(401));
        assert_eq!(provider_status(&LlmError::ProviderError("provider returned 404 Not Found: ".to_string())), Some(404));
        assert_eq!(provider_status(&LlmError::ProviderError("something else".to_string())), None);
    }
}
