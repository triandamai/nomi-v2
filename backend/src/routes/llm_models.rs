use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::app::AppState;
use crate::auth::extractor::AuthClaims;
use crate::routes::settings::require_system_config_permission;
use crate::settings::{self, llm_models};

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
