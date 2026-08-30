use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::app::AppState;
use nomi_auth::extractor::AuthClaims;
use crate::routes::settings::require_system_config_permission;
use nomi_settings::{self as settings, llm_models};

pub(crate) const ALLOWED_PROVIDERS: [&str; 4] = ["anthropic", "openai", "gemini", "fake"];

#[derive(Serialize)]
pub struct AdminLlmModelResponse {
    pub id: Uuid,
    pub label: String,
    pub provider: String,
    pub model_id: String,
    pub base_url: Option<String>,
    pub is_default: bool,
    pub api_key_masked: String,
}

fn to_response(
    model: llm_models::AdminLlmModel,
    settings_key: &[u8; 32],
) -> Result<AdminLlmModelResponse, (StatusCode, &'static str)> {
    let api_key = settings::crypto::decrypt(settings_key, &model.api_key_encrypted)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to decrypt stored api key"))?;
    Ok(AdminLlmModelResponse {
        id: model.id,
        label: model.label,
        provider: model.provider,
        model_id: model.model_id,
        base_url: model.base_url,
        is_default: model.is_default,
        api_key_masked: settings::mask_api_key(&api_key),
    })
}

pub async fn list_admin_models(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
) -> Result<Json<Vec<AdminLlmModelResponse>>, (StatusCode, &'static str)> {
    require_system_config_permission(&claims)?;
    let models = llm_models::list_admin_llm_models(&state.pool)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to load models"))?;
    let responses = models.into_iter().map(|m| to_response(m, &state.settings_key)).collect::<Result<Vec<_>, _>>()?;
    Ok(Json(responses))
}

#[derive(Deserialize)]
pub struct CreateAdminModelRequest {
    pub label: String,
    pub provider: String,
    pub model_id: String,
    pub api_key: String,
    pub base_url: Option<String>,
}

pub async fn create_admin_model(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Json(req): Json<CreateAdminModelRequest>,
) -> Result<(StatusCode, Json<AdminLlmModelResponse>), (StatusCode, &'static str)> {
    require_system_config_permission(&claims)?;
    if !ALLOWED_PROVIDERS.contains(&req.provider.as_str()) {
        return Err((StatusCode::BAD_REQUEST, "unknown provider (expected anthropic, openai, gemini, or fake)"));
    }
    let is_fake = req.provider == "fake";
    if !is_fake && req.model_id.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "model_id is required for a non-fake provider"));
    }
    if !is_fake && req.api_key.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "api_key is required for a non-fake provider"));
    }
    if req.label.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "label is required"));
    }

    let model = llm_models::create_admin_llm_model(
        &state.pool,
        llm_models::NewAdminLlmModel {
            label: &req.label,
            provider: &req.provider,
            model_id: &req.model_id,
            api_key_encrypted: settings::crypto::encrypt(&state.settings_key, &req.api_key),
            base_url: req.base_url.as_deref(),
            updated_by: claims.sub,
        },
    )
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to create model"))?;

    Ok((StatusCode::CREATED, Json(to_response(model, &state.settings_key)?)))
}

#[derive(Deserialize)]
pub struct UpdateAdminModelRequest {
    pub label: String,
    pub provider: String,
    pub model_id: String,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
}

pub async fn update_admin_model(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateAdminModelRequest>,
) -> Result<Json<AdminLlmModelResponse>, (StatusCode, &'static str)> {
    require_system_config_permission(&claims)?;
    if !ALLOWED_PROVIDERS.contains(&req.provider.as_str()) {
        return Err((StatusCode::BAD_REQUEST, "unknown provider (expected anthropic, openai, gemini, or fake)"));
    }
    let is_fake = req.provider == "fake";
    if !is_fake && req.model_id.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "model_id is required for a non-fake provider"));
    }
    if req.label.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "label is required"));
    }

    let api_key_encrypted = match req.api_key.as_deref() {
        Some(key) if !key.is_empty() => Some(settings::crypto::encrypt(&state.settings_key, key)),
        _ => None,
    };

    let model = llm_models::update_admin_llm_model(
        &state.pool,
        id,
        llm_models::UpdateAdminLlmModel {
            label: &req.label,
            provider: &req.provider,
            model_id: &req.model_id,
            api_key_encrypted,
            base_url: req.base_url.as_deref(),
            updated_by: claims.sub,
        },
    )
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to update model"))?
    .ok_or((StatusCode::NOT_FOUND, "model not found"))?;

    Ok(Json(to_response(model, &state.settings_key)?))
}

pub async fn delete_admin_model(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, &'static str)> {
    require_system_config_permission(&claims)?;
    llm_models::delete_admin_llm_model(&state.pool, id).await.map_err(|e| match e {
        llm_models::DeleteAdminLlmModelError::NotFound => (StatusCode::NOT_FOUND, "model not found"),
        llm_models::DeleteAdminLlmModelError::IsDefault => {
            (StatusCode::BAD_REQUEST, "cannot delete the current default; set a different default first")
        }
        llm_models::DeleteAdminLlmModelError::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, "failed to delete model"),
    })?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn set_default_admin_model(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, &'static str)> {
    require_system_config_permission(&claims)?;
    llm_models::set_default_admin_llm_model(&state.pool, id).await.map_err(|e| match e {
        llm_models::SetDefaultAdminLlmModelError::NotFound => (StatusCode::NOT_FOUND, "model not found"),
        llm_models::SetDefaultAdminLlmModelError::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, "failed to set default"),
    })?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
pub struct FetchProviderModelsRequest {
    pub provider: String,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    /// When `api_key` is left blank (editing an existing model without changing its key),
    /// reuse the stored, decrypted key for this admin model row instead of requiring the
    /// caller to re-paste it just to browse the model list.
    pub existing_model_id: Option<Uuid>,
}

#[derive(Serialize)]
pub struct FetchedModel {
    pub id: String,
    pub label: Option<String>,
}

#[derive(Serialize)]
pub struct FetchProviderModelsResponse {
    pub models: Vec<FetchedModel>,
}

#[tracing::instrument(skip(state, claims, req))]
pub async fn fetch_provider_models(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Json(req): Json<FetchProviderModelsRequest>,
) -> Result<Json<FetchProviderModelsResponse>, (StatusCode, String)> {
    require_system_config_permission(&claims).map_err(|(status, msg)| (status, msg.to_string()))?;

    let provider_kind = provider_kind_from_str(&req.provider)
        .ok_or((StatusCode::BAD_REQUEST, "unknown provider (expected anthropic, openai, gemini, or fake)".to_string()))?;
    let is_fake = req.provider == "fake";

    let api_key = match req.api_key.as_deref() {
        Some(key) if !key.is_empty() => key.to_string(),
        _ if is_fake => String::new(),
        _ => {
            let existing_id = req
                .existing_model_id
                .ok_or((StatusCode::BAD_REQUEST, "api_key is required to fetch models for a new entry".to_string()))?;
            let model = llm_models::get_admin_llm_model(&state.pool, existing_id)
                .await
                .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to load existing model".to_string()))?
                .ok_or((StatusCode::NOT_FOUND, "model not found".to_string()))?;
            settings::crypto::decrypt(&state.settings_key, &model.api_key_encrypted)
                .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to decrypt stored api key".to_string()))?
        }
    };

    let config = nomi_llm::ModelConfig { provider: provider_kind, model_id: String::new(), api_key, base_url: req.base_url.clone() };
    let models = nomi_llm::list_provider_models(config, state.http_client.clone())
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;

    Ok(Json(FetchProviderModelsResponse {
        models: models.into_iter().map(|m| FetchedModel { id: m.id, label: m.label }).collect(),
    }))
}

#[derive(Serialize)]
pub struct UserModelOption {
    pub id: Uuid,
    pub label: String,
    pub provider: String,
    pub model_id: String,
}

#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum UserSelectionResponse {
    Admin { admin_model_id: Uuid },
    Custom { label: String, provider: String, model_id: String, api_key_masked: String, base_url: Option<String> },
}

#[derive(Serialize)]
pub struct UserModelsResponse {
    pub admin_models: Vec<UserModelOption>,
    pub selection: Option<UserSelectionResponse>,
}

pub async fn get_user_models(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
) -> Result<Json<UserModelsResponse>, (StatusCode, &'static str)> {
    let admin_models = llm_models::list_admin_llm_models(&state.pool)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to load models"))?
        .into_iter()
        .map(|m| UserModelOption { id: m.id, label: m.label, provider: m.provider, model_id: m.model_id })
        .collect();

    let selection_row = llm_models::get_user_llm_selection(&state.pool, claims.sub)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to load selection"))?;

    let selection = match selection_row {
        Some(row) => {
            if let Some(admin_model_id) = row.admin_model_id {
                Some(UserSelectionResponse::Admin { admin_model_id })
            } else if let Some(provider) = row.custom_provider {
                let api_key = settings::crypto::decrypt(
                    &state.settings_key,
                    row.custom_api_key_encrypted.as_deref().unwrap_or_default(),
                )
                .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to decrypt stored api key"))?;
                Some(UserSelectionResponse::Custom {
                    label: row.custom_label.unwrap_or_default(),
                    provider,
                    model_id: row.custom_model_id.unwrap_or_default(),
                    api_key_masked: settings::mask_api_key(&api_key),
                    base_url: row.custom_base_url,
                })
            } else {
                None
            }
        }
        None => None,
    };

    Ok(Json(UserModelsResponse { admin_models, selection }))
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SelectionRequest {
    Admin { admin_model_id: Uuid },
    Custom { label: String, provider: String, model_id: String, api_key: String, base_url: Option<String> },
}

fn provider_kind_from_str(s: &str) -> Option<nomi_llm::ProviderKind> {
    match s {
        "anthropic" => Some(nomi_llm::ProviderKind::Anthropic),
        "openai" => Some(nomi_llm::ProviderKind::OpenAi),
        "gemini" => Some(nomi_llm::ProviderKind::Gemini),
        "fake" => Some(nomi_llm::ProviderKind::Fake),
        _ => None,
    }
}

/// Unlike every other handler in this file, errors here carry a dynamic `String` rather than
/// `&'static str` — the BYOK validation failure needs to surface the provider's actual error
/// text (spec requirement: "the provider's actual error message... surfaced inline on the
/// form"), which structurally cannot be a `&'static str`.
pub async fn put_user_selection(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Json(req): Json<SelectionRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    match req {
        SelectionRequest::Admin { admin_model_id } => {
            let exists = llm_models::get_admin_llm_model(&state.pool, admin_model_id)
                .await
                .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to look up model".to_string()))?
                .is_some();
            if !exists {
                return Err((StatusCode::NOT_FOUND, "model not found".to_string()));
            }
            llm_models::set_user_llm_selection_admin(&state.pool, claims.sub, admin_model_id)
                .await
                .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to save selection".to_string()))?;
        }
        SelectionRequest::Custom { label, provider, model_id, api_key, base_url } => {
            if !ALLOWED_PROVIDERS.contains(&provider.as_str()) {
                return Err((
                    StatusCode::BAD_REQUEST,
                    "unknown provider (expected anthropic, openai, gemini, or fake)".to_string(),
                ));
            }
            let is_fake = provider == "fake";
            if !is_fake && model_id.trim().is_empty() {
                return Err((StatusCode::BAD_REQUEST, "model_id is required for a non-fake provider".to_string()));
            }
            if !is_fake && api_key.trim().is_empty() {
                return Err((StatusCode::BAD_REQUEST, "api_key is required for a non-fake provider".to_string()));
            }

            let provider_kind = provider_kind_from_str(&provider)
                .ok_or((StatusCode::BAD_REQUEST, "unknown provider".to_string()))?;
            let model_config = nomi_llm::ModelConfig {
                provider: provider_kind,
                model_id: model_id.clone(),
                api_key: api_key.clone(),
                base_url: base_url.clone(),
            };
            nomi_llm::validate_model_config(model_config, state.http_client.clone())
                .await
                .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

            llm_models::set_user_llm_selection_custom(
                &state.pool,
                claims.sub,
                llm_models::CustomLlmSelection {
                    label: &label,
                    provider: &provider,
                    model_id: &model_id,
                    api_key_encrypted: settings::crypto::encrypt(&state.settings_key, &api_key),
                    base_url: base_url.as_deref(),
                },
            )
            .await
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to save selection".to_string()))?;
        }
    }
    Ok(StatusCode::NO_CONTENT)
}
