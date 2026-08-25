use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};

use crate::app::AppState;
use crate::auth::claims::Claims;
use crate::auth::extractor::AuthClaims;
use crate::settings;

#[derive(Serialize)]
pub struct ProviderSettingsResponse {
    pub provider: String,
    pub model_id: String,
    pub base_url: Option<String>,
    pub api_key_masked: String,
}

#[derive(Deserialize)]
pub struct UpdateProviderSettingsRequest {
    pub provider: String,
    pub model_id: String,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
}

pub(crate) fn require_system_config_permission(claims: &Claims) -> Result<(), (StatusCode, &'static str)> {
    if claims.has_permission("admin", "system_config", "manage") {
        Ok(())
    } else {
        Err((StatusCode::FORBIDDEN, "not authorized to manage system settings"))
    }
}

#[tracing::instrument(skip(state))]
async fn load_settings_response(
    state: &AppState,
    setting_type: &str,
) -> Result<Json<ProviderSettingsResponse>, (StatusCode, &'static str)> {
    let row = settings::get_settings(&state.pool, setting_type)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "failed to load settings");
            (StatusCode::INTERNAL_SERVER_ERROR, "failed to load settings")
        })?
        .ok_or((StatusCode::NOT_FOUND, "settings not configured"))?;

    let api_key = settings::crypto::decrypt(&state.settings_key, &row.api_key_encrypted)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to decrypt stored api key"))?;

    tracing::debug!(provider = %row.provider, model_id = %row.model_id, "loaded provider settings");
    Ok(Json(ProviderSettingsResponse {
        provider: row.provider,
        model_id: row.model_id,
        base_url: row.base_url,
        api_key_masked: settings::mask_api_key(&api_key),
    }))
}

#[tracing::instrument(skip(state, claims))]
pub async fn get_embedding_settings(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
) -> Result<Json<ProviderSettingsResponse>, (StatusCode, &'static str)> {
    require_system_config_permission(&claims)?;
    load_settings_response(&state, "embedding").await
}

async fn resolve_api_key(
    state: &AppState,
    setting_type: &str,
    submitted: Option<&str>,
    is_fake: bool,
) -> Result<String, (StatusCode, &'static str)> {
    if let Some(key) = submitted {
        if !key.is_empty() {
            return Ok(key.to_string());
        }
    }
    let existing = settings::get_settings(&state.pool, setting_type)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to load existing settings"))?;
    match existing {
        Some(row) => settings::crypto::decrypt(&state.settings_key, &row.api_key_encrypted)
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to decrypt stored api key")),
        None if is_fake => Ok(String::new()),
        None => Err((StatusCode::BAD_REQUEST, "api_key is required for a new non-fake provider configuration")),
    }
}

#[tracing::instrument(skip(state, claims, req))]
pub async fn put_embedding_settings(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Json(req): Json<UpdateProviderSettingsRequest>,
) -> Result<Json<ProviderSettingsResponse>, (StatusCode, &'static str)> {
    require_system_config_permission(&claims)?;
    tracing::info!(
        provider = %req.provider,
        model_id = %req.model_id,
        has_api_key = req.api_key.is_some(),
        updated_by = %claims.sub,
        "updating embedding provider settings"
    );

    if !["openai", "fake"].contains(&req.provider.as_str()) {
        return Err((StatusCode::BAD_REQUEST, "unknown provider (expected openai or fake)"));
    }
    let is_fake = req.provider == "fake";
    if !is_fake && req.model_id.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "model_id is required for a non-fake provider"));
    }

    let api_key = resolve_api_key(&state, "embedding", req.api_key.as_deref(), is_fake).await?;
    if !is_fake && api_key.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "api_key is required for a non-fake provider"));
    }

    settings::upsert_settings(
        &state.pool,
        settings::UpsertInput {
            setting_type: "embedding",
            provider: &req.provider,
            model_id: &req.model_id,
            api_key_encrypted: settings::crypto::encrypt(&state.settings_key, &api_key),
            base_url: req.base_url.as_deref(),
            updated_by: claims.sub,
        },
    )
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to save settings"))?;

    Ok(Json(ProviderSettingsResponse {
        provider: req.provider,
        model_id: req.model_id,
        base_url: req.base_url,
        api_key_masked: settings::mask_api_key(&api_key),
    }))
}
