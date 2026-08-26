pub mod config;
pub mod openai;
pub mod types;
pub mod fake;

pub use config::{build_embedding_provider, EmbeddingConfig, EmbeddingProviderKind};
pub use types::EmbeddingError;

use async_trait::async_trait;

#[async_trait]
pub trait EmbeddingProvider: Send + Sync {
    async fn embed(&self, text: &str) -> Result<Vec<f32>, EmbeddingError>;
}
