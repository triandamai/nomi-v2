use std::env::var;
use std::sync::Arc;

use nomi_orchestrator::embedding::{
    build_embedding_provider, EmbeddingConfig, EmbeddingProvider, EmbeddingProviderKind,
};
use nomi_orchestrator::llm::{build_provider, LlmProvider, ModelConfig, ProviderKind};
use nomi_orchestrator::settings;

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

async fn build_llm_provider_from_settings_or_env(
    pool: &sqlx::PgPool,
    settings_key: &[u8; 32],
    http_client: reqwest::Client,
) -> Arc<dyn LlmProvider> {
    let row = settings::get_settings(pool, "llm").await.expect("failed to query provider_settings");

    let model_config = match row {
        Some(row) => {
            let api_key = settings::crypto::decrypt(settings_key, &row.api_key_encrypted)
                .expect("failed to decrypt stored llm api key");
            ModelConfig {
                provider: llm_provider_kind_from_str(&row.provider),
                model_id: row.model_id,
                api_key,
                base_url: row.base_url,
            }
        }
        None => {
            let provider =
                llm_provider_kind_from_str(&std::env::var("LLM_PROVIDER").expect("LLM_PROVIDER must be set"));
            let (model_id, api_key) = match provider {
                ProviderKind::Fake => (String::new(), String::new()),
                _ => (
                    std::env::var("LLM_MODEL_ID").expect("LLM_MODEL_ID must be set"),
                    std::env::var("LLM_API_KEY").expect("LLM_API_KEY must be set"),
                ),
            };
            ModelConfig { provider, model_id, api_key, base_url: std::env::var("LLM_BASE_URL").ok() }
        }
    };

    Arc::from(build_provider(model_config, http_client))
}

async fn build_embedding_provider_from_settings_or_env(
    pool: &sqlx::PgPool,
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

#[tokio::main]
async fn main() {
    let database_url = var("DATABASE_URL").expect("DATABASE_URL must be set");
    let jwt_secret = var("JWT_SECRET").expect("JWT_SECRET must be set");
    let settings_key = settings::crypto::parse_key(
        &var("SETTINGS_ENCRYPTION_KEY").expect("SETTINGS_ENCRYPTION_KEY must be set"),
    )
    .expect("SETTINGS_ENCRYPTION_KEY must be 64 hex characters (32 bytes)");

    let pool = sqlx::PgPool::connect(&database_url).await.expect("failed to connect to database");
    sqlx::migrate!().run(&pool).await.expect("failed to run migrations");

    let http_client = reqwest::Client::new();

    let provider = build_llm_provider_from_settings_or_env(&pool, &settings_key, http_client.clone()).await;
    let embedding_provider =
        build_embedding_provider_from_settings_or_env(&pool, &settings_key, http_client.clone()).await;

    let state = nomi_orchestrator::app::AppState {
        pool,
        jwt_secret,
        provider: Arc::new(tokio::sync::RwLock::new(provider)),
        embedding_provider: Arc::new(tokio::sync::RwLock::new(embedding_provider)),
        http_client,
        settings_key,
    };
    let app = nomi_orchestrator::app::build_router(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.expect("failed to bind to port 8080");
    axum::serve(listener, app).await.expect("server error");
}
