use super::cohere::CohereEmbeddingProvider;
use super::fake::FakeEmbeddingProvider;
use super::gemini::GeminiEmbeddingProvider;
use super::openai::OpenAiEmbeddingProvider;
use super::EmbeddingProvider;

#[derive(Debug, Clone, PartialEq)]
pub enum EmbeddingProviderKind {
    OpenAi,
    Gemini,
    Cohere,
    Fake,
}

#[derive(Debug, Clone)]
pub struct EmbeddingConfig {
    pub provider: EmbeddingProviderKind,
    pub model_id: String,
    pub api_key: String,
    pub base_url: Option<String>,
}

pub fn build_embedding_provider(config: EmbeddingConfig, http_client: reqwest::Client) -> Box<dyn EmbeddingProvider> {
    match config.provider {
        EmbeddingProviderKind::OpenAi => {
            let base_url = config.base_url.unwrap_or_else(OpenAiEmbeddingProvider::default_base_url);
            Box::new(OpenAiEmbeddingProvider::new(http_client, config.api_key, config.model_id, base_url))
        }
        EmbeddingProviderKind::Gemini => {
            let base_url = config.base_url.unwrap_or_else(GeminiEmbeddingProvider::default_base_url);
            Box::new(GeminiEmbeddingProvider::new(http_client, config.api_key, config.model_id, base_url))
        }
        EmbeddingProviderKind::Cohere => {
            let base_url = config.base_url.unwrap_or_else(CohereEmbeddingProvider::default_base_url);
            Box::new(CohereEmbeddingProvider::new(http_client, config.api_key, config.model_id, base_url))
        }
        EmbeddingProviderKind::Fake => Box::new(FakeEmbeddingProvider),
    }
}
