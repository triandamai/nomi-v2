use async_trait::async_trait;

#[derive(Debug, thiserror::Error)]
pub enum EmbeddingError {
    #[error("provider returned an error response: {0}")]
    ProviderError(String),
    #[error("failed to parse provider response: {0}")]
    ParseError(String),
}

#[async_trait]
pub trait EmbeddingProvider: Send + Sync {
    async fn embed(&self, text: &str) -> Result<Vec<f32>, EmbeddingError>;
}
