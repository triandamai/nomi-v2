use super::fake::FakeEmbeddingProvider;
use super::openai::OpenAiEmbeddingProvider;
use super::EmbeddingProvider;

#[derive(Debug, Clone, PartialEq)]
pub enum EmbeddingProviderKind {
    OpenAi,
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
        EmbeddingProviderKind::Fake => Box::new(FakeEmbeddingProvider),
    }
}
