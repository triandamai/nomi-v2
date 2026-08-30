pub mod config;
pub mod openai;
pub mod cohere;
pub mod gemini;
pub mod types;
pub mod fake;

pub use config::{build_embedding_provider, EmbeddingConfig, EmbeddingProviderKind};
pub use types::EmbeddingError;

use async_trait::async_trait;

#[async_trait]
pub trait EmbeddingProvider: Send + Sync {
    /// Embed text for storage — this is the "document" side of an asymmetric model.
    async fn embed(&self, text: &str) -> Result<Vec<f32>, EmbeddingError>;

    /// Embed text for a similarity query. Providers whose models are asymmetric (a
    /// different request shape for text you're storing vs. text you're searching with)
    /// override this; everyone else inherits the default, which just calls `embed`.
    async fn embed_for_query(&self, text: &str) -> Result<Vec<f32>, EmbeddingError> {
        self.embed(text).await
    }

    /// A short, fixed identifier for which provider this is (e.g. "openai", "gemini") —
    /// used to tag stored memories so a provider switch doesn't compare embeddings from
    /// incompatible vector spaces.
    fn provider_name(&self) -> &'static str;

    /// The specific model this instance is configured with.
    fn model_id(&self) -> &str;
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct EmbeddingModelSummary {
    pub id: String,
    pub label: Option<String>,
}

/// Calls the provider's real model-listing API using the given key, filtered to
/// embedding-capable models where the provider's API supports that, without persisting
/// anything — used by the admin UI to populate a model picker instead of a free-text field.
pub async fn list_embedding_provider_models(
    config: EmbeddingConfig,
    http_client: reqwest::Client,
) -> Result<Vec<EmbeddingModelSummary>, EmbeddingError> {
    let base_url = config.base_url.clone().unwrap_or_else(|| match config.provider {
        EmbeddingProviderKind::OpenAi => openai::OpenAiEmbeddingProvider::default_base_url(),
        EmbeddingProviderKind::Gemini => gemini::GeminiEmbeddingProvider::default_base_url(),
        EmbeddingProviderKind::Cohere => cohere::CohereEmbeddingProvider::default_base_url(),
        EmbeddingProviderKind::Fake => String::new(),
    });
    match config.provider {
        EmbeddingProviderKind::OpenAi => openai::list_models(&http_client, &config.api_key, &base_url).await,
        EmbeddingProviderKind::Gemini => gemini::list_models(&http_client, &config.api_key, &base_url).await,
        EmbeddingProviderKind::Cohere => cohere::list_models(&http_client, &config.api_key, &base_url).await,
        EmbeddingProviderKind::Fake => Ok(vec![EmbeddingModelSummary {
            id: "fake-embedding-model".to_string(),
            label: Some("Fake Embedding Model (dev/testing)".to_string()),
        }]),
    }
}
