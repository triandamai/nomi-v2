//! Admin → Configuration: attachments, file storage, Google, email and the agent loop, saved in
//! Nomi's database instead of the server's environment (which stays the fallback). Seeing it takes
//! the `configuration` area; changing it takes `manage`, since it holds secrets.

use std::collections::BTreeMap;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};

use nomi_auth::catalog::{MANAGE, VIEW};
use nomi_auth::extractor::AuthClaims;
use nomi_settings::config::{self, ConfigError, FieldView, SectionDef};

use crate::app::AppState;
use crate::routes::admin_access::require_admin_owned;
use crate::routes::deploy::ApiError;

const AREA: &str = "configuration";

#[derive(Serialize)]
pub struct SectionView {
    pub name: &'static str,
    pub fields: Vec<FieldView>,
}

#[derive(Serialize)]
pub struct ConfigurationView {
    pub sections: Vec<SectionView>,
    pub can_manage: bool,
}

fn env(name: &str) -> Option<String> {
    std::env::var(name).ok()
}

async fn section_view(state: &AppState, def: &'static SectionDef) -> Result<SectionView, ApiError> {
    let fields = config::view(&state.pool, Some(&state.settings_key), def, env)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("couldn't read the configuration: {e}")))?;
    Ok(SectionView { name: def.name, fields })
}

/// `GET /api/admin/configuration`: every section, with where each value comes from.
pub async fn get(State(state): State<AppState>, AuthClaims(claims): AuthClaims) -> Result<Json<ConfigurationView>, ApiError> {
    require_admin_owned(&claims, AREA, VIEW)?;
    let mut sections = Vec::new();
    for def in config::SECTIONS {
        sections.push(section_view(&state, def).await?);
    }
    Ok(Json(ConfigurationView { sections, can_manage: claims.has_permission("admin", AREA, MANAGE) }))
}

#[derive(Deserialize)]
pub struct SaveRequest {
    /// Field → value. An empty value goes back to the environment; an empty secret keeps the
    /// saved one.
    pub values: BTreeMap<String, String>,
}

/// What a section needs as a whole, beyond each field being valid on its own.
async fn check_whole(state: &AppState, def: &'static SectionDef, values: &BTreeMap<String, String>) -> Result<(), ApiError> {
    if def.name != config::STORAGE.name {
        return Ok(());
    }
    let saved = config::load(&state.pool, Some(&state.settings_key), def)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("couldn't read the configuration: {e}")))?
        .with_env(env);
    let after = |field: &str| match values.get(field).map(|v| v.trim()) {
        Some("") if field != "secret_access_key" => None,
        Some("") | None => saved.get(field),
        Some(value) => Some(value.to_string()),
    };
    if after("bucket").is_some() && (after("access_key_id").is_none() || after("secret_access_key").is_none()) {
        return Err((StatusCode::UNPROCESSABLE_ENTITY, "a bucket needs an access key ID and a secret access key".to_string()));
    }
    Ok(())
}

/// `PUT /api/admin/configuration/:section`: saves the section's changed fields, all or nothing.
pub async fn save(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(section): Path<String>,
    Json(req): Json<SaveRequest>,
) -> Result<Json<SectionView>, ApiError> {
    require_admin_owned(&claims, AREA, MANAGE)?;
    let def = config::section_named(&section).ok_or((StatusCode::NOT_FOUND, format!("there's no {section} section")))?;
    check_whole(&state, def, &req.values).await?;
    config::save(&state.pool, &state.settings_key, def, &req.values, claims.sub).await.map_err(|e| match e {
        ConfigError::Invalid(why) => (StatusCode::UNPROCESSABLE_ENTITY, why),
        ConfigError::NoKey => (StatusCode::SERVICE_UNAVAILABLE, e.to_string()),
        ConfigError::Db(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("couldn't save the configuration: {e}")),
    })?;
    tracing::info!(user = %claims.sub, section = def.name, fields = ?req.values.keys().collect::<Vec<_>>(), "configuration saved");
    Ok(Json(section_view(&state, def).await?))
}
