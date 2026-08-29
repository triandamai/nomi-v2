pub mod config;
pub mod openai;
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
