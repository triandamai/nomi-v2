use std::sync::Arc;

use nomi_orchestrator::embedding::{build_embedding_provider, EmbeddingConfig, EmbeddingProvider, EmbeddingProviderKind};
use nomi_orchestrator::llm::{build_provider, LlmProvider, ModelConfig, ProviderKind};

#[tokio::main]
async fn main() {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");

    let pool = sqlx::PgPool::connect(&database_url)
        .await
        .expect("failed to connect to database");
    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("failed to run migrations");

    let http_client = reqwest::Client::new();

    let llm_provider_kind = match std::env::var("LLM_PROVIDER").expect("LLM_PROVIDER must be set").as_str() {
        "anthropic" => ProviderKind::Anthropic,
        "openai" => ProviderKind::OpenAi,
        "gemini" => ProviderKind::Gemini,
        "fake" => ProviderKind::Fake,
        other => panic!("unknown LLM_PROVIDER: {other} (expected anthropic, openai, gemini, or fake)"),
    };
    let (llm_model_id, llm_api_key) = match llm_provider_kind {
        ProviderKind::Fake => (String::new(), String::new()),
        _ => (
            std::env::var("LLM_MODEL_ID").expect("LLM_MODEL_ID must be set"),
            std::env::var("LLM_API_KEY").expect("LLM_API_KEY must be set"),
        ),
    };
    let model_config = ModelConfig {
        provider: llm_provider_kind,
        model_id: llm_model_id,
        api_key: llm_api_key,
        base_url: std::env::var("LLM_BASE_URL").ok(),
    };
    let provider: Arc<dyn LlmProvider> = Arc::from(build_provider(model_config, http_client.clone()));

    let embedding_provider_kind = match std::env::var("EMBEDDING_PROVIDER").unwrap_or_else(|_| "openai".to_string()).as_str() {
        "openai" => EmbeddingProviderKind::OpenAi,
        "fake" => EmbeddingProviderKind::Fake,
        other => panic!("unknown EMBEDDING_PROVIDER: {other} (expected openai or fake)"),
    };
    let (embedding_model_id, embedding_api_key) = match embedding_provider_kind {
        EmbeddingProviderKind::Fake => (String::new(), String::new()),
        EmbeddingProviderKind::OpenAi => (
            std::env::var("EMBEDDING_MODEL_ID").expect("EMBEDDING_MODEL_ID must be set"),
            std::env::var("EMBEDDING_API_KEY").expect("EMBEDDING_API_KEY must be set"),
        ),
    };
    let embedding_config = EmbeddingConfig {
        provider: embedding_provider_kind,
        model_id: embedding_model_id,
        api_key: embedding_api_key,
        base_url: std::env::var("EMBEDDING_BASE_URL").ok(),
    };
    let embedding_provider: Arc<dyn EmbeddingProvider> =
        Arc::from(build_embedding_provider(embedding_config, http_client));

    let state = nomi_orchestrator::app::AppState { pool, jwt_secret, provider, embedding_provider };
    let app = nomi_orchestrator::app::build_router(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080")
        .await
        .expect("failed to bind to port 8080");
    axum::serve(listener, app).await.expect("server error");
}
