use std::env::var;
use std::sync::Arc;

use sqlx::PgPool;

use nomi_embedding::{build_embedding_provider, EmbeddingConfig, EmbeddingProvider, EmbeddingProviderKind};
use nomi_llm::media::{MediaRouter, MediaSupport};
use nomi_llm::{build_provider, LlmProvider, ModelConfig, ProviderKind};
use nomi_settings as settings;
use nomi_usage::{MeteredEmbeddingProvider, MeteredProvider, ModelTag};

/// None for a name this build doesn't know (written by a newer version): callers fall back
/// rather than fail the person's turn.
fn llm_provider_kind_from_str(s: &str) -> Option<ProviderKind> {
    let kind = ProviderKind::parse(s);
    if kind.is_none() {
        tracing::error!(provider = s, "unknown LLM provider; falling back");
    }
    kind
}

fn embedding_provider_kind_from_str(s: &str) -> Result<EmbeddingProviderKind, String> {
    match s {
        "openai" => Ok(EmbeddingProviderKind::OpenAi),
        "gemini" => Ok(EmbeddingProviderKind::Gemini),
        "cohere" => Ok(EmbeddingProviderKind::Cohere),
        "fake" => Ok(EmbeddingProviderKind::Fake),
        other => Err(format!("unknown embedding provider: {other} (expected openai, gemini, cohere, or fake)")),
    }
}

/// What a model is wanted for: most work runs on the person's chat model; Koda builds projects
/// with the coding model (theirs, or the one an admin picked).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelPurpose {
    Chat,
    Coding,
}

/// The purpose an agent's runs are for.
pub fn purpose_for_agent(agent_type: &str) -> ModelPurpose {
    if agent_type == nomi_agent_coding::CODING_AGENT_TYPE {
        ModelPurpose::Coding
    } else {
        ModelPurpose::Chat
    }
}

pub async fn build_llm_provider_for_user(
    pool: &PgPool,
    user_id: uuid::Uuid,
    settings_key: &[u8; 32],
    http_client: reqwest::Client,
) -> Arc<dyn LlmProvider> {
    build_llm_provider_for(pool, user_id, settings_key, http_client, ModelPurpose::Chat).await
}

pub async fn build_llm_provider_for(
    pool: &PgPool,
    user_id: uuid::Uuid,
    settings_key: &[u8; 32],
    http_client: reqwest::Client,
    purpose: ModelPurpose,
) -> Arc<dyn LlmProvider> {
    build_llm_provider(pool, user_id, None, settings_key, http_client, purpose).await
}

/// [`build_llm_provider_for`] for a turn in `session_id`: it runs on the model picked for that
/// chat, when there is one it can use.
pub async fn build_llm_provider_in_session(
    pool: &PgPool,
    user_id: uuid::Uuid,
    session_id: uuid::Uuid,
    settings_key: &[u8; 32],
    http_client: reqwest::Client,
    purpose: ModelPurpose,
) -> Arc<dyn LlmProvider> {
    build_llm_provider(pool, user_id, Some(session_id), settings_key, http_client, purpose).await
}

async fn build_llm_provider(
    pool: &PgPool,
    user_id: uuid::Uuid,
    session_id: Option<uuid::Uuid>,
    settings_key: &[u8; 32],
    http_client: reqwest::Client,
    purpose: ModelPurpose,
) -> Arc<dyn LlmProvider> {
    let (mut model_config, mut tag, mut support) = resolve_for_purpose(pool, user_id, session_id, settings_key, purpose).await;

    // Nomi's own models count toward the person's monthly allowance. Past it, the key they saved
    // answers instead; with none, nothing does.
    let mut over_quota = false;
    if !tag.own_key {
        match crate::quota::usage(pool, user_id).await {
            Ok(usage) => {
                let own = if usage.standing() == crate::quota::Standing::UsedUp { saved_own_key(pool, user_id, settings_key).await } else { None };
                crate::quota::alert(pool, user_id, &usage, own.as_ref().map(|(_, tag, _)| tag.label.as_str())).await;
                if usage.standing() == crate::quota::Standing::UsedUp {
                    match own {
                        Some(own) => {
                            (model_config, tag, support) = own;
                            over_quota = true;
                        }
                        None => return crate::quota::blocked(),
                    }
                }
            }
            // A failed check never stops anyone.
            Err(e) => tracing::warn!(error = %e, %user_id, "failed to check the token quota"),
        }
    }
    let primary_admin_id = tag.admin_model_id;
    let provider: Arc<dyn LlmProvider> = Arc::from(build_provider(model_config, http_client.clone()));
    // Every call made for this person counts toward their usage (Billing & usage).
    let primary: Arc<dyn LlmProvider> = Arc::new(MeteredProvider::new(provider, pool.clone(), user_id, tag));

    // Files their own model can't open (an image for a text-only model, a video for Claude) go
    // to the files model an admin picked, also counted toward their usage.
    // Past the allowance, Nomi's files model doesn't step in either.
    let files = match settings::llm_models::get_files_admin_llm_model(pool).await {
        Ok(Some(_)) if over_quota => None,
        Ok(Some(model)) if Some(model.id) != primary_admin_id => {
            let files_support = media_support_for(&model.provider, &model.model_id, model.media_inputs.as_deref());
            let (id, label) = (model.id, model.label.clone());
            admin_model_config(pool, settings_key, model).await.map(|config| {
                let tag = tag_for(&config, Some(id), false, label);
                let provider: Arc<dyn LlmProvider> = Arc::from(build_provider(config, http_client));
                (Arc::new(MeteredProvider::new(provider, pool.clone(), user_id, tag)) as Arc<dyn LlmProvider>, files_support)
            })
        }
        Ok(_) => None,
        Err(e) => {
            tracing::error!(error = %e, "failed to load the files model");
            None
        }
    };
    Arc::new(MediaRouter::new(primary, support, files))
}

/// Whether `media_type` files can be opened for this person: by their own model, or by the
/// files model an admin picked.
pub async fn can_open_for_user(pool: &PgPool, user_id: uuid::Uuid, settings_key: &[u8; 32], media_type: &str) -> bool {
    let (_, _, support) = resolve_llm_model_with_media(pool, user_id, settings_key).await;
    if support.allows(media_type) {
        return true;
    }
    match settings::llm_models::get_files_admin_llm_model(pool).await {
        Ok(Some(model)) => media_support_for(&model.provider, &model.model_id, model.media_inputs.as_deref()).allows(media_type),
        _ => false,
    }
}

/// The API key the person saved (Models → Your own key), even while they use one of Nomi's
/// models: what Nomi falls back to when their allowance runs out.
async fn saved_own_key(pool: &PgPool, user_id: uuid::Uuid, settings_key: &[u8; 32]) -> Option<(ModelConfig, ModelTag, MediaSupport)> {
    let row = settings::llm_models::get_user_llm_selection(pool, user_id).await.ok()??;
    own_model(pool, user_id, settings_key, row).await
}

/// The person's own model: a key saved the old way (Models → Your own key), else a model from
/// one of their connected providers. Never billed to their plan.
async fn own_model(
    pool: &PgPool,
    user_id: uuid::Uuid,
    settings_key: &[u8; 32],
    row: settings::llm_models::UserLlmSelectionRow,
) -> Option<(ModelConfig, ModelTag, MediaSupport)> {
    let model_id = row.custom_model_id?;
    let config = match (&row.custom_api_key_encrypted, row.custom_connection_id) {
        (Some(encrypted), _) => {
            let api_key = settings::crypto::decrypt(settings_key, encrypted)
                .map_err(|e| tracing::error!(%user_id, error = %e, "failed to decrypt custom llm api key; falling back"))
                .ok()?;
            ModelConfig { provider: llm_provider_kind_from_str(row.custom_provider.as_deref()?)?, model_id, api_key, base_url: row.custom_base_url, extra_headers: vec![] }
        }
        (None, Some(connection_id)) => connection_model_config(pool, settings_key, settings::provider_connections::Scope::User(user_id), connection_id, model_id).await?,
        (None, None) => return None,
    };
    let tag = tag_for(&config, None, true, row.custom_label.unwrap_or_else(|| config.model_id.clone()));
    let support = MediaSupport::infer(&config.provider, &config.model_id);
    Some((config, tag, support))
}

/// What a model takes besides text: what an admin listed, or a guess from its id.
pub fn media_support_for(provider: &str, model_id: &str, listed: Option<&[String]>) -> MediaSupport {
    let Some(kind) = llm_provider_kind_from_str(provider) else { return MediaSupport::NONE };
    match listed {
        Some(kinds) => MediaSupport::from_list(&kind, kinds),
        None => MediaSupport::infer(&kind, model_id),
    }
}

fn provider_name(kind: &ProviderKind) -> &'static str {
    kind.as_str()
}

fn tag_for(config: &ModelConfig, admin_model_id: Option<uuid::Uuid>, own_key: bool, label: String) -> ModelTag {
    ModelTag { admin_model_id, own_key, label, provider: provider_name(&config.provider).to_string(), model_id: config.model_id.clone() }
}

/// A custom endpoint's extra header, as stored (encrypted JSON) on its connection.
#[derive(serde::Serialize, serde::Deserialize)]
pub struct StoredHeader {
    pub name: String,
    pub value: String,
}

/// What runs `model_id` through a connection in `scope`: its kind, key, base URL and headers.
/// None (logged) when the connection isn't in that scope or its secrets can't be read.
pub(crate) async fn connection_model_config(
    pool: &PgPool,
    settings_key: &[u8; 32],
    scope: settings::provider_connections::Scope,
    connection_id: uuid::Uuid,
    model_id: String,
) -> Option<ModelConfig> {
    let connection = settings::provider_connections::get_connection(pool, scope, connection_id)
        .await
        .map_err(|e| tracing::error!(%connection_id, error = %e, "failed to load a provider connection"))
        .ok()??;
    let decrypt = |encrypted: &Vec<u8>| {
        settings::crypto::decrypt(settings_key, encrypted)
            .map_err(|e| tracing::error!(%connection_id, error = %e, "failed to decrypt a provider connection; falling back"))
            .ok()
    };
    let api_key = match &connection.api_key_encrypted {
        Some(encrypted) => decrypt(encrypted)?,
        None => String::new(),
    };
    let extra_headers = match &connection.headers_encrypted {
        Some(encrypted) => serde_json::from_str::<Vec<StoredHeader>>(&decrypt(encrypted)?)
            .map_err(|e| tracing::error!(%connection_id, error = %e, "unreadable provider connection headers"))
            .ok()?
            .into_iter()
            .map(|h| (h.name, h.value))
            .collect(),
        None => Vec::new(),
    };
    // A catalog provider's address comes from the catalog unless it was changed when connecting.
    let base_url = connection.base_url.or_else(|| nomi_llm::catalog::provider(&connection.provider_id).and_then(|p| p.default_base_url.clone()));
    Some(ModelConfig { provider: llm_provider_kind_from_str(&connection.kind)?, model_id, api_key, base_url, extra_headers })
}

/// How to run an admin model: its own key when it still has one (saved before providers were
/// connected), else its connection's.
async fn admin_model_config(pool: &PgPool, settings_key: &[u8; 32], model: settings::llm_models::AdminLlmModel) -> Option<ModelConfig> {
    match (&model.api_key_encrypted, model.connection_id) {
        (Some(encrypted), _) => match settings::crypto::decrypt(settings_key, encrypted) {
            Ok(api_key) => Some(ModelConfig {
                provider: llm_provider_kind_from_str(&model.provider)?,
                model_id: model.model_id,
                api_key,
                base_url: model.base_url,
                extra_headers: vec![],
            }),
            Err(e) => {
                tracing::error!(model_id = %model.id, error = %e, "failed to decrypt admin llm model api key; falling back");
                None
            }
        },
        (None, Some(connection_id)) => connection_model_config(pool, settings_key, settings::provider_connections::Scope::Admin, connection_id, model.model_id).await,
        (None, None) => None,
    }
}

async fn resolve_for_purpose(
    pool: &PgPool,
    user_id: uuid::Uuid,
    session_id: Option<uuid::Uuid>,
    settings_key: &[u8; 32],
    purpose: ModelPurpose,
) -> (ModelConfig, ModelTag, MediaSupport) {
    // A model picked in the chat is what everything in that chat runs on, Koda's building included.
    if let Some(session_id) = session_id {
        if let Some(resolved) = session_model(pool, user_id, session_id, settings_key).await {
            return resolved;
        }
    }
    if purpose == ModelPurpose::Coding {
        if let Some(resolved) = resolve_coding_model(pool, user_id, settings_key).await {
            return resolved;
        }
    }
    resolve_llm_model_with_media(pool, user_id, settings_key).await
}

/// The model a person's work for `purpose` runs on.
pub async fn resolve_llm_model_config_for(pool: &PgPool, user_id: uuid::Uuid, settings_key: &[u8; 32], purpose: ModelPurpose) -> ModelConfig {
    resolve_for_purpose(pool, user_id, None, settings_key, purpose).await.0
}

/// The model this person's work for `purpose` in `session_id` runs on.
pub async fn resolve_llm_model_config_in_session_for(
    pool: &PgPool,
    user_id: uuid::Uuid,
    session_id: uuid::Uuid,
    settings_key: &[u8; 32],
    purpose: ModelPurpose,
) -> ModelConfig {
    resolve_for_purpose(pool, user_id, Some(session_id), settings_key, purpose).await.0
}

/// The model this person's chat turn in `session_id` runs on, and how its usage is recorded.
pub async fn resolve_llm_model_in_session(pool: &PgPool, user_id: uuid::Uuid, session_id: uuid::Uuid, settings_key: &[u8; 32]) -> (ModelConfig, ModelTag) {
    let (config, tag, _) = resolve_for_purpose(pool, user_id, Some(session_id), settings_key, ModelPurpose::Chat).await;
    (config, tag)
}

/// The model picked for the chat in the chat itself, when it can run this person's turn: one of
/// Nomi's models that's still switched on, or a model from the picker's own provider — which
/// only ever runs the picker's own turns, never a collaborator's in a shared chat.
async fn session_model(pool: &PgPool, user_id: uuid::Uuid, session_id: uuid::Uuid, settings_key: &[u8; 32]) -> Option<(ModelConfig, ModelTag, MediaSupport)> {
    use settings::model_choices::ModelKey;
    let picked = settings::model_choices::get_session_model(pool, session_id)
        .await
        .map_err(|e| tracing::error!(%session_id, error = %e, "failed to load the chat's model"))
        .ok()??;
    match picked.key {
        ModelKey::Admin(id) => {
            let model = settings::llm_models::get_admin_llm_model(pool, id).await.ok().flatten().filter(|m| m.enabled)?;
            let label = model.label.clone();
            let support = media_support_for(&model.provider, &model.model_id, model.media_inputs.as_deref());
            let config = admin_model_config(pool, settings_key, model).await?;
            let tag = tag_for(&config, Some(id), false, label);
            Some((config, tag, support))
        }
        ModelKey::Connection { connection_id, model_id } => {
            if picked.set_by != Some(user_id) {
                return None;
            }
            let config = connection_model_config(pool, settings_key, settings::provider_connections::Scope::User(user_id), connection_id, model_id).await?;
            let tag = tag_for(&config, None, true, picked.label.unwrap_or_else(|| config.model_id.clone()));
            let support = MediaSupport::infer(&config.provider, &config.model_id);
            Some((config, tag, support))
        }
    }
}

pub async fn resolve_llm_model_config(pool: &PgPool, user_id: uuid::Uuid, settings_key: &[u8; 32]) -> ModelConfig {
    resolve_llm_model(pool, user_id, settings_key).await.0
}

/// The person's model, and how its usage is recorded.
pub async fn resolve_llm_model(pool: &PgPool, user_id: uuid::Uuid, settings_key: &[u8; 32]) -> (ModelConfig, ModelTag) {
    let (config, tag, _) = resolve_llm_model_with_media(pool, user_id, settings_key).await;
    (config, tag)
}

/// The model Koda builds with for this person, when it isn't simply their chat model: the one
/// of Nomi's models they picked for coding, else the coding model an admin picked. `None` means
/// use their chat model.
async fn resolve_coding_model(pool: &PgPool, user_id: uuid::Uuid, settings_key: &[u8; 32]) -> Option<(ModelConfig, ModelTag, MediaSupport)> {
    use settings::llm_models::CodingChoice;
    let choice = settings::llm_models::get_user_coding_choice(pool, user_id).await.unwrap_or_else(|e| {
        tracing::error!(error = %e, %user_id, "failed to load the coding model choice");
        CodingChoice::Default
    });
    let picked = match choice {
        CodingChoice::SameAsChat => return None,
        CodingChoice::Admin(id) => settings::llm_models::get_admin_llm_model(pool, id).await.ok().flatten().filter(|m| m.enabled),
        CodingChoice::Default => None,
    };
    // Their pick was deleted, or they left it to Nomi: the admin's coding model.
    let model = match picked {
        Some(model) => model,
        None => settings::llm_models::get_coding_admin_llm_model(pool).await.ok().flatten()?,
    };
    let (id, label) = (model.id, model.label.clone());
    let support = media_support_for(&model.provider, &model.model_id, model.media_inputs.as_deref());
    let config = admin_model_config(pool, settings_key, model).await?;
    let tag = tag_for(&config, Some(id), false, label);
    Some((config, tag, support))
}

/// [`resolve_llm_model`], plus what that model can open besides text.
async fn resolve_llm_model_with_media(pool: &PgPool, user_id: uuid::Uuid, settings_key: &[u8; 32]) -> (ModelConfig, ModelTag, MediaSupport) {
    let selection = settings::llm_models::get_user_llm_selection(pool, user_id)
        .await
        .expect("failed to query user_llm_selections");

    if let Some(row) = selection {
        if let Some(admin_model_id) = row.admin_model_id {
            // A model an admin switched off falls back to the default.
            if let Some(admin_model) = settings::llm_models::get_admin_llm_model(pool, admin_model_id)
                .await
                .expect("failed to query admin_llm_models")
                .filter(|m| m.enabled)
            {
                let (id, label) = (admin_model.id, admin_model.label.clone());
                let support = media_support_for(&admin_model.provider, &admin_model.model_id, admin_model.media_inputs.as_deref());
                if let Some(config) = admin_model_config(pool, settings_key, admin_model).await {
                    let tag = tag_for(&config, Some(id), false, label);
                    return (config, tag, support);
                }
                // Decryption failed (e.g. a rotated/mismatched SETTINGS_ENCRYPTION_KEY) —
                // fall through to the admin default below.
            }
            // Referenced admin model no longer exists — fall through to the default below.
        } else if let Some(resolved) = own_model(pool, user_id, settings_key, row).await {
            return resolved;
        }
        // Their own key couldn't be used (unreadable, or the connection is gone): the default below.
    }

    if let Some(default_model) = settings::llm_models::get_default_admin_llm_model(pool)
        .await
        .expect("failed to query admin_llm_models")
    {
        let (id, label) = (default_model.id, default_model.label.clone());
        let support = media_support_for(&default_model.provider, &default_model.model_id, default_model.media_inputs.as_deref());
        if let Some(config) = admin_model_config(pool, settings_key, default_model).await {
            let tag = tag_for(&config, Some(id), false, label);
            return (config, tag, support);
        }
        // Decryption failed — fall through to the env-var fallback below.
    }

    // No admin models configured at all yet (fresh install), or the default model's key
    // couldn't be decrypted — same env-var fallback the function this replaced always used.
    let provider = llm_provider_kind_from_str(&var("LLM_PROVIDER").expect("LLM_PROVIDER must be set")).expect("LLM_PROVIDER must name a known provider");
    let (model_id, api_key) = match provider {
        ProviderKind::Fake => (String::new(), String::new()),
        _ => (
            var("LLM_MODEL_ID").expect("LLM_MODEL_ID must be set"),
            var("LLM_API_KEY").expect("LLM_API_KEY must be set"),
        ),
    };
    let config = ModelConfig { provider, model_id, api_key, base_url: var("LLM_BASE_URL").ok(), extra_headers: vec![] };
    let tag = tag_for(&config, None, false, if config.model_id.is_empty() { provider_name(&config.provider).to_string() } else { config.model_id.clone() });
    let support = MediaSupport::infer(&config.provider, &config.model_id);
    (config, tag, support)
}

/// The embedding model Nomi uses, metered for `user_id` (Admin → Usage).
pub async fn build_embedding_provider_from_settings_or_env(
    pool: &PgPool,
    settings_key: &[u8; 32],
    http_client: reqwest::Client,
    user_id: Option<uuid::Uuid>,
) -> Arc<dyn EmbeddingProvider> {
    let row = settings::get_settings(pool, "embedding").await.expect("failed to query provider_settings");

    let from_row = row.and_then(|row| {
        let api_key = match settings::crypto::decrypt(settings_key, &row.api_key_encrypted) {
            Ok(key) => key,
            Err(e) => {
                tracing::error!(error = %e, "failed to decrypt stored embedding api key; falling back to env vars");
                return None;
            }
        };
        let provider = match embedding_provider_kind_from_str(&row.provider) {
            Ok(provider) => provider,
            Err(e) => {
                tracing::error!(error = %e, "stored embedding provider settings are invalid; falling back to env vars");
                return None;
            }
        };
        Some(EmbeddingConfig { provider, model_id: row.model_id, api_key, base_url: row.base_url })
    });

    let embedding_config = match from_row {
        Some(config) => config,
        None => {
            let provider = embedding_provider_kind_from_str(
                &std::env::var("EMBEDDING_PROVIDER").unwrap_or_else(|_| "openai".to_string()),
            )
            .expect("EMBEDDING_PROVIDER env var must be a known provider");
            let (model_id, api_key) = match provider {
                EmbeddingProviderKind::Fake => (String::new(), String::new()),
                _ => {
                    let model_id = std::env::var("EMBEDDING_MODEL_ID").expect("EMBEDDING_MODEL_ID must be set");
                    if model_id.trim().is_empty() {
                        panic!("EMBEDDING_MODEL_ID must not be empty");
                    }
                    let api_key = std::env::var("EMBEDDING_API_KEY").expect("EMBEDDING_API_KEY must be set");
                    (model_id, api_key)
                }
            };
            EmbeddingConfig { provider, model_id, api_key, base_url: std::env::var("EMBEDDING_BASE_URL").ok() }
        }
    };

    let provider: Arc<dyn EmbeddingProvider> = Arc::from(build_embedding_provider(embedding_config, http_client));
    Arc::new(MeteredEmbeddingProvider::new(provider, pool.clone(), user_id))
}

/// The embedding provider and model Nomi uses now: Admin → Embeddings, else the environment.
pub async fn current_embedding_model(pool: &PgPool) -> Option<(String, String)> {
    match settings::get_settings(pool, "embedding").await {
        Ok(Some(row)) => Some((row.provider, row.model_id)),
        _ => {
            let provider = std::env::var("EMBEDDING_PROVIDER").unwrap_or_else(|_| "openai".to_string());
            let model_id = if provider == "fake" { String::new() } else { std::env::var("EMBEDDING_MODEL_ID").ok()? };
            Some((provider, model_id))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedding_provider_kind_from_str_accepts_all_known_providers() {
        assert_eq!(embedding_provider_kind_from_str("openai"), Ok(EmbeddingProviderKind::OpenAi));
        assert_eq!(embedding_provider_kind_from_str("gemini"), Ok(EmbeddingProviderKind::Gemini));
        assert_eq!(embedding_provider_kind_from_str("cohere"), Ok(EmbeddingProviderKind::Cohere));
        assert_eq!(embedding_provider_kind_from_str("fake"), Ok(EmbeddingProviderKind::Fake));
    }

    #[test]
    fn embedding_provider_kind_from_str_rejects_unknown_providers() {
        assert!(embedding_provider_kind_from_str("not-a-real-provider").is_err());
    }
}
