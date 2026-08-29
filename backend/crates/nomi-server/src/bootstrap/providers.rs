use std::env::var;
use std::sync::Arc;

use sqlx::PgPool;

use nomi_embedding::{build_embedding_provider, EmbeddingConfig, EmbeddingProvider, EmbeddingProviderKind};
use nomi_llm::{build_provider, LlmProvider, ModelConfig, ProviderKind};
use nomi_settings as settings;

fn llm_provider_kind_from_str(s: &str) -> ProviderKind {
    match s {
        "anthropic" => ProviderKind::Anthropic,
        "openai" => ProviderKind::OpenAi,
        "gemini" => ProviderKind::Gemini,
        "fake" => ProviderKind::Fake,
        other => panic!("unknown LLM provider: {other} (expected anthropic, openai, gemini, or fake)"),
    }
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

pub async fn build_llm_provider_for_user(
    pool: &PgPool,
    user_id: uuid::Uuid,
    settings_key: &[u8; 32],
    http_client: reqwest::Client,
) -> Arc<dyn LlmProvider> {
    let model_config = resolve_llm_model_config(pool, user_id, settings_key).await;
    Arc::from(build_provider(model_config, http_client))
}

fn model_config_from_admin_model(
    settings_key: &[u8; 32],
    model: settings::llm_models::AdminLlmModel,
) -> Option<ModelConfig> {
    match settings::crypto::decrypt(settings_key, &model.api_key_encrypted) {
        Ok(api_key) => Some(ModelConfig {
            provider: llm_provider_kind_from_str(&model.provider),
            model_id: model.model_id,
            api_key,
            base_url: model.base_url,
        }),
        Err(e) => {
            tracing::error!(
                model_id = %model.id,
                error = %e,
                "failed to decrypt admin llm model api key; falling back"
            );
            None
        }
    }
}

pub async fn resolve_llm_model_config(pool: &PgPool, user_id: uuid::Uuid, settings_key: &[u8; 32]) -> ModelConfig {
    let selection = settings::llm_models::get_user_llm_selection(pool, user_id)
        .await
        .expect("failed to query user_llm_selections");

    if let Some(row) = selection {
        if let Some(admin_model_id) = row.admin_model_id {
            if let Some(admin_model) = settings::llm_models::get_admin_llm_model(pool, admin_model_id)
                .await
                .expect("failed to query admin_llm_models")
            {
                if let Some(config) = model_config_from_admin_model(settings_key, admin_model) {
                    return config;
                }
                // Decryption failed (e.g. a rotated/mismatched SETTINGS_ENCRYPTION_KEY) —
                // fall through to the admin default below.
            }
            // Referenced admin model no longer exists — fall through to the default below.
        } else if let Some(provider) = row.custom_provider {
            let decrypted = settings::crypto::decrypt(
                settings_key,
                row.custom_api_key_encrypted.as_ref().expect("custom selection always carries a key"),
            );
            match decrypted {
                Ok(api_key) => {
                    return ModelConfig {
                        provider: llm_provider_kind_from_str(&provider),
                        model_id: row.custom_model_id.expect("custom selection always carries a model_id"),
                        api_key,
                        base_url: row.custom_base_url,
                    };
                }
                Err(e) => {
                    tracing::error!(
                        user_id = %user_id,
                        error = %e,
                        "failed to decrypt custom llm api key; falling back to admin default"
                    );
                    // Fall through to the admin default below.
                }
            }
        }
    }

    if let Some(default_model) = settings::llm_models::get_default_admin_llm_model(pool)
        .await
        .expect("failed to query admin_llm_models")
    {
        if let Some(config) = model_config_from_admin_model(settings_key, default_model) {
            return config;
        }
        // Decryption failed — fall through to the env-var fallback below.
    }

    // No admin models configured at all yet (fresh install), or the default model's key
    // couldn't be decrypted — same env-var fallback the function this replaced always used.
    let provider = llm_provider_kind_from_str(&var("LLM_PROVIDER").expect("LLM_PROVIDER must be set"));
    let (model_id, api_key) = match provider {
        ProviderKind::Fake => (String::new(), String::new()),
        _ => (
            var("LLM_MODEL_ID").expect("LLM_MODEL_ID must be set"),
            var("LLM_API_KEY").expect("LLM_API_KEY must be set"),
        ),
    };
    ModelConfig { provider, model_id, api_key, base_url: var("LLM_BASE_URL").ok() }
}

pub async fn build_embedding_provider_from_settings_or_env(
    pool: &PgPool,
    settings_key: &[u8; 32],
    http_client: reqwest::Client,
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
                _ => (
                    std::env::var("EMBEDDING_MODEL_ID").expect("EMBEDDING_MODEL_ID must be set"),
                    std::env::var("EMBEDDING_API_KEY").expect("EMBEDDING_API_KEY must be set"),
                ),
            };
            EmbeddingConfig { provider, model_id, api_key, base_url: std::env::var("EMBEDDING_BASE_URL").ok() }
        }
    };

    Arc::from(build_embedding_provider(embedding_config, http_client))
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
