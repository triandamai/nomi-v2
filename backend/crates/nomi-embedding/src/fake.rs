use async_trait::async_trait;

use super::{EmbeddingError, EmbeddingProvider};

pub struct FakeEmbeddingProvider;

#[async_trait]
impl EmbeddingProvider for FakeEmbeddingProvider {
    async fn embed(&self, _text: &str) -> Result<Vec<f32>, EmbeddingError> {
        Ok(vec![0.0; 1536])
    }

    fn provider_name(&self) -> &'static str {
        "fake"
    }

    fn model_id(&self) -> &str {
        ""
    }
}
