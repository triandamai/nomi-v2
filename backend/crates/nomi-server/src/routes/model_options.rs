//! The chat's model selector: what it offers (Nomi's models by provider, then models from the
//! person's own providers), which of those each person shows (Manage models), and the model a
//! chat runs on when someone picks one in it.

use std::collections::HashMap;
use std::time::Duration;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::app::AppState;
use crate::routes::llm_models::provider_display_name;
use nomi_auth::extractor::AuthClaims;
use nomi_llm::catalog;
use nomi_settings::model_choices::{self, ModelKey};
use nomi_settings::provider_connections::{self as connections, ProviderConnectionRow, Scope};
use nomi_settings::llm_models;

type RouteError = (StatusCode, Json<Value>);

fn error(status: StatusCode, code: &str) -> RouteError {
    (status, Json(json!({ "error": code })))
}

fn server_error(what: &str, e: impl std::fmt::Display) -> RouteError {
    tracing::error!(error = %e, "{what}");
    error(StatusCode::INTERNAL_SERVER_ERROR, "server_error")
}

/// How long a custom endpoint gets to list its models before the selector shows it as unreachable.
const LIVE_LIST_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Serialize)]
pub struct ModelOption {
    key: String,
    label: String,
    model_id: String,
    free: bool,
    enabled: bool,
}

#[derive(Serialize)]
pub struct ModelGroup {
    id: String,
    source: &'static str,
    provider_id: String,
    has_logo: bool,
    label: String,
    connection_id: Option<Uuid>,
    error: Option<&'static str>,
    models: Vec<ModelOption>,
}

/// A model as the selector's trigger shows it.
#[derive(Serialize)]
pub struct ModelChoice {
    key: String,
    label: String,
    model_id: String,
    provider_id: String,
    provider_label: String,
    has_logo: bool,
    /// Picked for this chat, rather than the person's default.
    chat_only: bool,
}

fn has_logo(provider_id: &str) -> bool {
    catalog::provider(provider_id).is_some_and(|p| p.has_logo)
}

/// The catalog id (for the logo) of a provider stored by its kind.
fn catalog_id_for_kind(kind: &str) -> String {
    match kind {
        "gemini" => "google".to_string(),
        "openai_compatible" => "custom".to_string(),
        other => other.to_string(),
    }
}

/// Where one of Nomi's models runs: its instance connection's provider and name, else its kind's.
fn admin_provider(model: &llm_models::AdminLlmModel, shared: &HashMap<Uuid, ProviderConnectionRow>) -> (String, String) {
    match model.connection_id.and_then(|id| shared.get(&id)) {
        Some(connection) => (connection.provider_id.clone(), connection.label.clone()),
        None => (catalog_id_for_kind(&model.provider), provider_display_name(&model.provider)),
    }
}

/// A connection's name as the selector shows it: the provider's own name for catalog providers.
fn connection_name(connection: &ProviderConnectionRow) -> String {
    match catalog::provider(&connection.provider_id) {
        Some(provider) if connection.auth_type != "custom_endpoint" => provider.name.clone(),
        _ => connection.label.clone(),
    }
}

fn catalog_model_name(provider_id: &str, model_id: &str) -> Option<String> {
    catalog::provider(provider_id)?.models.iter().find(|m| m.id == model_id).map(|m| m.name.clone())
}

async fn shared_connections(state: &AppState) -> Result<HashMap<Uuid, ProviderConnectionRow>, RouteError> {
    Ok(connections::list_connections(&state.pool, Scope::Admin)
        .await
        .map_err(|e| server_error("failed to load providers", e))?
        .into_iter()
        .map(|c| (c.id, c))
        .collect())
}

fn admin_choice(model: &llm_models::AdminLlmModel, shared: &HashMap<Uuid, ProviderConnectionRow>, chat_only: bool) -> ModelChoice {
    let (provider_id, provider_label) = admin_provider(model, shared);
    ModelChoice {
        key: ModelKey::Admin(model.id).to_string(),
        label: model.label.clone(),
        model_id: model.model_id.clone(),
        has_logo: has_logo(&provider_id),
        provider_id,
        provider_label,
        chat_only,
    }
}

fn connection_choice(connection: &ProviderConnectionRow, model_id: &str, label: Option<String>, chat_only: bool) -> ModelChoice {
    ModelChoice {
        key: ModelKey::Connection { connection_id: connection.id, model_id: model_id.to_string() }.to_string(),
        label: label.or_else(|| catalog_model_name(&connection.provider_id, model_id)).unwrap_or_else(|| model_id.to_string()),
        model_id: model_id.to_string(),
        has_logo: has_logo(&connection.provider_id),
        provider_id: connection.provider_id.clone(),
        provider_label: connection_name(connection),
        chat_only,
    }
}

/// The person's default: their pick under Models, else Nomi's default model.
async fn default_choice(state: &AppState, user_id: Uuid, shared: &HashMap<Uuid, ProviderConnectionRow>) -> Result<Option<ModelChoice>, RouteError> {
    let selection = llm_models::get_user_llm_selection(&state.pool, user_id).await.map_err(|e| server_error("failed to load the selection", e))?;
    if let Some(row) = selection {
        if let Some(id) = row.admin_model_id {
            if let Some(model) = llm_models::get_admin_llm_model(&state.pool, id).await.map_err(|e| server_error("failed to load a model", e))?.filter(|m| m.enabled) {
                return Ok(Some(admin_choice(&model, shared, false)));
            }
        } else if let (Some(connection_id), Some(model_id)) = (row.custom_connection_id, row.custom_model_id.as_deref()) {
            if let Some(connection) = connections::get_connection(&state.pool, Scope::User(user_id), connection_id).await.map_err(|e| server_error("failed to load a provider", e))? {
                return Ok(Some(connection_choice(&connection, model_id, row.custom_label.clone(), false)));
            }
        }
    }
    Ok(llm_models::get_default_admin_llm_model(&state.pool)
        .await
        .map_err(|e| server_error("failed to load the default model", e))?
        .map(|model| admin_choice(&model, shared, false)))
}

/// A custom endpoint's models, listed live (it has no catalog entry). None when it can't be reached.
async fn live_models(state: &AppState, user_id: Uuid, connection: &ProviderConnectionRow) -> Option<Vec<String>> {
    let config = crate::bootstrap::connection_model_config(&state.pool, &state.settings_key, Scope::User(user_id), connection.id, String::new()).await?;
    match tokio::time::timeout(LIVE_LIST_TIMEOUT, nomi_llm::list_provider_models(config, state.http_client.clone())).await {
        Ok(Ok(models)) => Some(models.into_iter().map(|m| m.id).collect()),
        Ok(Err(e)) => {
            tracing::warn!(connection_id = %connection.id, error = %e, "couldn't list a custom provider's models");
            None
        }
        Err(_) => None,
    }
}

/// `GET /api/llm/model-options`: every model the person can pick, by provider, and their default.
pub async fn list_options(State(state): State<AppState>, AuthClaims(claims): AuthClaims) -> Result<Json<Value>, RouteError> {
    let user_id = claims.sub;
    let visibility = model_choices::model_visibility(&state.pool, user_id).await.map_err(|e| server_error("failed to load model visibility", e))?;
    let shared = shared_connections(&state).await?;

    // Nomi's models, under the provider each runs on, in the order admins added them.
    let mut groups: Vec<ModelGroup> = Vec::new();
    let models = llm_models::list_admin_llm_models(&state.pool).await.map_err(|e| server_error("failed to load models", e))?;
    for model in models.iter().filter(|m| m.enabled) {
        let (provider_id, provider_label) = admin_provider(model, &shared);
        let key = ModelKey::Admin(model.id).to_string();
        let option = ModelOption {
            enabled: visibility.get(&key).copied().unwrap_or(true),
            key,
            label: model.label.clone(),
            model_id: model.model_id.clone(),
            free: false,
        };
        let id = format!("nomi:{provider_label}");
        match groups.iter_mut().find(|g| g.id == id) {
            Some(group) => group.models.push(option),
            None => groups.push(ModelGroup {
                id,
                source: "nomi",
                has_logo: has_logo(&provider_id),
                provider_id,
                label: provider_label,
                connection_id: None,
                error: None,
                models: vec![option],
            }),
        }
    }

    // Then the person's own providers: the catalog's models, or a custom endpoint's live list.
    let own = connections::list_connections(&state.pool, Scope::User(user_id)).await.map_err(|e| server_error("failed to load providers", e))?;
    for connection in own {
        let option = |model_id: &str, label: String, free: bool| {
            let key = ModelKey::Connection { connection_id: connection.id, model_id: model_id.to_string() }.to_string();
            ModelOption { enabled: visibility.get(&key).copied().unwrap_or(false), key, label, model_id: model_id.to_string(), free }
        };
        let (models, error) = match catalog::provider(&connection.provider_id).filter(|_| connection.auth_type != "custom_endpoint") {
            Some(provider) => (
                provider.models.iter().map(|m| option(&m.id, m.name.clone(), m.cost.input == Some(0.0) && m.cost.output == Some(0.0))).collect(),
                None,
            ),
            None => match live_models(&state, user_id, &connection).await {
                Some(ids) => (ids.iter().map(|id| option(id, id.clone(), false)).collect(), None),
                None => (Vec::new(), Some("unreachable")),
            },
        };
        groups.push(ModelGroup {
            id: format!("conn:{}", connection.id),
            source: "connection",
            provider_id: connection.provider_id.clone(),
            has_logo: has_logo(&connection.provider_id),
            label: connection_name(&connection),
            connection_id: Some(connection.id),
            error,
            models,
        });
    }

    let current = default_choice(&state, user_id, &shared).await?;
    Ok(Json(json!({ "groups": groups, "current": current })))
}

#[derive(Deserialize)]
pub struct VisibilityChange {
    key: String,
    enabled: bool,
}

#[derive(Deserialize)]
pub struct VisibilityRequest {
    changes: Vec<VisibilityChange>,
}

/// `PUT /api/llm/model-options`: show or hide models in the person's selector.
pub async fn set_visibility(State(state): State<AppState>, AuthClaims(claims): AuthClaims, Json(req): Json<VisibilityRequest>) -> Result<StatusCode, RouteError> {
    if req.changes.len() > 2000 {
        return Err(error(StatusCode::BAD_REQUEST, "too_many_changes"));
    }
    let changes = req
        .changes
        .into_iter()
        .map(|c| ModelKey::parse(&c.key).map(|key| (key.to_string(), c.enabled)))
        .collect::<Option<Vec<_>>>()
        .ok_or_else(|| error(StatusCode::BAD_REQUEST, "invalid_key"))?;
    model_choices::set_model_visibility(&state.pool, claims.sub, &changes).await.map_err(|e| server_error("failed to save model visibility", e))?;
    Ok(StatusCode::NO_CONTENT)
}

async fn authorize(state: &AppState, user_id: Uuid, session_id: Uuid) -> Result<(), RouteError> {
    crate::routes::sessions::authorize_session_access(&state.pool, user_id, session_id).await.map_err(|(status, _)| {
        error(status, if status == StatusCode::NOT_FOUND { "not_found" } else { "server_error" })
    })
}

/// The model a chat runs on for this person: the one picked in the chat when it can run their
/// turns (see bootstrap::providers::session_model), else their default.
async fn chat_choice(state: &AppState, user_id: Uuid, session_id: Uuid) -> Result<Option<ModelChoice>, RouteError> {
    let shared = shared_connections(state).await?;
    let picked = model_choices::get_session_model(&state.pool, session_id).await.map_err(|e| server_error("failed to load the chat's model", e))?;
    match picked {
        Some(model_choices::SessionModel { key: ModelKey::Admin(id), .. }) => {
            if let Some(model) = llm_models::get_admin_llm_model(&state.pool, id).await.map_err(|e| server_error("failed to load a model", e))?.filter(|m| m.enabled) {
                return Ok(Some(admin_choice(&model, &shared, true)));
            }
        }
        Some(model_choices::SessionModel { key: ModelKey::Connection { connection_id, model_id }, label, set_by }) if set_by == Some(user_id) => {
            if let Some(connection) = connections::get_connection(&state.pool, Scope::User(user_id), connection_id).await.map_err(|e| server_error("failed to load a provider", e))? {
                return Ok(Some(connection_choice(&connection, &model_id, label, true)));
            }
        }
        _ => {}
    }
    default_choice(state, user_id, &shared).await
}

/// `GET /api/sessions/:id/model`.
pub async fn get_chat_model(State(state): State<AppState>, AuthClaims(claims): AuthClaims, Path(session_id): Path<Uuid>) -> Result<Json<Value>, RouteError> {
    authorize(&state, claims.sub, session_id).await?;
    Ok(Json(json!({ "model": chat_choice(&state, claims.sub, session_id).await? })))
}

#[derive(Deserialize)]
pub struct ChatModelRequest {
    key: String,
}

/// `PUT /api/sessions/:id/model`: run this chat on another model.
pub async fn set_chat_model(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(session_id): Path<Uuid>,
    Json(req): Json<ChatModelRequest>,
) -> Result<Json<Value>, RouteError> {
    authorize(&state, claims.sub, session_id).await?;
    let key = ModelKey::parse(req.key.trim()).ok_or_else(|| error(StatusCode::BAD_REQUEST, "invalid_key"))?;
    let label = match &key {
        ModelKey::Admin(id) => {
            llm_models::get_admin_llm_model(&state.pool, *id)
                .await
                .map_err(|e| server_error("failed to load a model", e))?
                .filter(|m| m.enabled)
                .ok_or_else(|| error(StatusCode::NOT_FOUND, "not_found"))?;
            None
        }
        ModelKey::Connection { connection_id, model_id } => {
            // Only the person's own providers: a pick never spends someone else's key.
            let connection = connections::get_connection(&state.pool, Scope::User(claims.sub), *connection_id)
                .await
                .map_err(|e| server_error("failed to load a provider", e))?
                .ok_or_else(|| error(StatusCode::NOT_FOUND, "not_found"))?;
            catalog_model_name(&connection.provider_id, model_id)
        }
    };
    model_choices::set_session_model(&state.pool, session_id, &key, label.as_deref(), claims.sub).await.map_err(|e| server_error("failed to save the chat's model", e))?;
    Ok(Json(json!({ "model": chat_choice(&state, claims.sub, session_id).await? })))
}
