use std::env::var;
use std::sync::Arc;

use sqlx::PgPool;

use crate::embedding::{build_embedding_provider, EmbeddingConfig, EmbeddingProvider, EmbeddingProviderKind};
use crate::llm::{build_provider, LlmProvider, ModelConfig, ProviderKind};
use crate::settings;

fn llm_provider_kind_from_str(s: &str) -> ProviderKind {
    match s {
        "anthropic" => ProviderKind::Anthropic,
        "openai" => ProviderKind::OpenAi,
        "gemini" => ProviderKind::Gemini,
        "fake" => ProviderKind::Fake,
        other => panic!("unknown LLM provider: {other} (expected anthropic, openai, gemini, or fake)"),
    }
}

fn embedding_provider_kind_from_str(s: &str) -> EmbeddingProviderKind {
    match s {
        "openai" => EmbeddingProviderKind::OpenAi,
        "fake" => EmbeddingProviderKind::Fake,
        other => panic!("unknown embedding provider: {other} (expected openai or fake)"),
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

fn model_config_from_admin_model(settings_key: &[u8; 32], model: crate::settings::llm_models::AdminLlmModel) -> ModelConfig {
    let api_key = settings::crypto::decrypt(settings_key, &model.api_key_encrypted)
        .expect("failed to decrypt stored admin llm model api key");
    ModelConfig {
        provider: llm_provider_kind_from_str(&model.provider),
        model_id: model.model_id,
        api_key,
        base_url: model.base_url,
    }
}

pub async fn resolve_llm_model_config(pool: &PgPool, user_id: uuid::Uuid, settings_key: &[u8; 32]) -> ModelConfig {
    let selection = crate::settings::llm_models::get_user_llm_selection(pool, user_id)
        .await
        .expect("failed to query user_llm_selections");

    if let Some(row) = selection {
        if let Some(admin_model_id) = row.admin_model_id {
            if let Some(admin_model) = crate::settings::llm_models::get_admin_llm_model(pool, admin_model_id)
                .await
                .expect("failed to query admin_llm_models")
            {
                return model_config_from_admin_model(settings_key, admin_model);
            }
            // Referenced admin model no longer exists — fall through to the default below.
        } else if let Some(provider) = row.custom_provider {
            let api_key = settings::crypto::decrypt(
                settings_key,
                row.custom_api_key_encrypted.as_ref().expect("custom selection always carries a key"),
            )
            .expect("failed to decrypt stored custom api key");
            return ModelConfig {
                provider: llm_provider_kind_from_str(&provider),
                model_id: row.custom_model_id.expect("custom selection always carries a model_id"),
                api_key,
                base_url: row.custom_base_url,
            };
        }
    }

    if let Some(default_model) = crate::settings::llm_models::get_default_admin_llm_model(pool)
        .await
        .expect("failed to query admin_llm_models")
    {
        return model_config_from_admin_model(settings_key, default_model);
    }

    // No admin models configured at all yet (fresh install) — same env-var fallback the
    // function this replaced always used when nothing was configured.
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

    let embedding_config = match row {
        Some(row) => {
            let api_key = settings::crypto::decrypt(settings_key, &row.api_key_encrypted)
                .expect("failed to decrypt stored embedding api key");
            EmbeddingConfig {
                provider: embedding_provider_kind_from_str(&row.provider),
                model_id: row.model_id,
                api_key,
                base_url: row.base_url,
            }
        }
        None => {
            let provider = embedding_provider_kind_from_str(
                &std::env::var("EMBEDDING_PROVIDER").unwrap_or_else(|_| "openai".to_string()),
            );
            let (model_id, api_key) = match provider {
                EmbeddingProviderKind::Fake => (String::new(), String::new()),
                EmbeddingProviderKind::OpenAi => (
                    std::env::var("EMBEDDING_MODEL_ID").expect("EMBEDDING_MODEL_ID must be set"),
                    std::env::var("EMBEDDING_API_KEY").expect("EMBEDDING_API_KEY must be set"),
                ),
            };
            EmbeddingConfig { provider, model_id, api_key, base_url: std::env::var("EMBEDDING_BASE_URL").ok() }
        }
    };

    Arc::from(build_embedding_provider(embedding_config, http_client))
}
