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

/// Why storage can't be saved as `changes` asks, if it can't: a bucket needs both keys, whether
/// they're saved here or come from the environment. `saved` answers what's saved for a field;
/// `env` what the environment has for it (a field that's cleared falls back to the environment).
fn storage_problem(saved: impl Fn(&str) -> Option<String>, env: impl Fn(&str) -> Option<String>, changes: &BTreeMap<String, String>) -> Option<&'static str> {
    let after = |field: &str| match changes.get(field).map(|v| v.trim()) {
        Some("") if field != "secret_access_key" => env(field),
        Some("") | None => saved(field).or_else(|| env(field)),
        Some(value) => Some(value.to_string()),
    };
    (after("bucket").is_some() && (after("access_key_id").is_none() || after("secret_access_key").is_none()))
        .then_some("a bucket needs an access key ID and a secret access key")
}

/// What a section needs as a whole, beyond each field being valid on its own.
async fn check_whole(state: &AppState, def: &'static SectionDef, values: &BTreeMap<String, String>) -> Result<(), ApiError> {
    if def.name != config::STORAGE.name {
        return Ok(());
    }
    let saved = config::load(&state.pool, Some(&state.settings_key), def)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("couldn't read the configuration: {e}")))?
        .with_env(|_| None);
    let from_env = |field: &str| def.fields.iter().find(|f| f.name == field).and_then(|f| f.env.and_then(env)).filter(|v| !v.trim().is_empty());
    match storage_problem(|field| saved.get(field), from_env, values) {
        Some(why) => Err((StatusCode::UNPROCESSABLE_ENTITY, why.to_string())),
        None => Ok(()),
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    fn changes(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
        pairs.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect()
    }

    fn keys(field: &str) -> Option<String> {
        matches!(field, "access_key_id" | "secret_access_key").then(|| format!("{field}-value"))
    }

    #[test]
    fn a_bucket_needs_both_keys_from_here_or_the_environment() {
        let nothing = |_: &str| None;
        assert!(storage_problem(nothing, nothing, &changes(&[("bucket", "files")])).is_some());
        assert!(storage_problem(nothing, nothing, &changes(&[("bucket", "files"), ("access_key_id", "id")])).is_some());
        assert!(storage_problem(nothing, nothing, &changes(&[("bucket", "files"), ("access_key_id", "id"), ("secret_access_key", "s")])).is_none());
        assert!(storage_problem(keys, nothing, &changes(&[("bucket", "files")])).is_none(), "keys saved before");
        assert!(storage_problem(nothing, keys, &changes(&[("bucket", "files")])).is_none(), "keys in the environment");
        assert!(storage_problem(nothing, nothing, &changes(&[("region", "eu-west-1")])).is_none(), "no bucket, nothing needed");
    }

    #[test]
    fn a_cleared_key_falls_back_to_the_environment_and_an_empty_secret_keeps_the_saved_one() {
        let nothing = |_: &str| None;
        let cleared = changes(&[("bucket", "files"), ("access_key_id", ""), ("secret_access_key", "")]);
        assert!(storage_problem(keys, nothing, &cleared).is_some(), "the saved ID is gone and the environment has none");
        assert!(storage_problem(keys, keys, &cleared).is_none(), "the environment's ID takes over");
    }
}
