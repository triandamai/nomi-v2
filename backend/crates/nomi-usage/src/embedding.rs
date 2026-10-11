//! Embedding usage: every text Nomi embeds (saving a memory, looking memories up) is metered by
//! [`MeteredEmbeddingProvider`], for Admin → Usage.

use std::sync::Arc;

use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use nomi_embedding::{estimate_tokens, Embedded, EmbeddingError, EmbeddingProvider};

/// An embedding provider that records the tokens of each call, for `user_id` when known.
pub struct MeteredEmbeddingProvider {
    inner: Arc<dyn EmbeddingProvider>,
    pool: PgPool,
    user_id: Option<Uuid>,
}

impl MeteredEmbeddingProvider {
    pub fn new(inner: Arc<dyn EmbeddingProvider>, pool: PgPool, user_id: Option<Uuid>) -> Self {
        Self { inner, pool, user_id }
    }

    async fn counted(&self, text: &str, for_query: bool) -> Result<Embedded, EmbeddingError> {
        let embedded = self.inner.embed_counted(text, for_query).await?;
        let (tokens, estimated) = match embedded.tokens {
            Some(tokens) => (tokens, false),
            None => (estimate_tokens(text), true),
        };
        // A failed write never fails the embedding.
        if let Err(e) = record(&self.pool, self.user_id, self.inner.provider_name(), self.inner.model_id(), tokens, estimated).await {
            tracing::warn!(error = %e, "failed to record embedding usage");
        }
        Ok(embedded)
    }
}

#[async_trait]
impl EmbeddingProvider for MeteredEmbeddingProvider {
    async fn embed(&self, text: &str) -> Result<Vec<f32>, EmbeddingError> {
        Ok(self.counted(text, false).await?.vector)
    }

    async fn embed_for_query(&self, text: &str) -> Result<Vec<f32>, EmbeddingError> {
        Ok(self.counted(text, true).await?.vector)
    }

    async fn embed_counted(&self, text: &str, for_query: bool) -> Result<Embedded, EmbeddingError> {
        self.counted(text, for_query).await
    }

    fn provider_name(&self) -> &'static str {
        self.inner.provider_name()
    }

    fn model_id(&self) -> &str {
        self.inner.model_id()
    }
}

/// Records one embedding call.
pub async fn record(pool: &PgPool, user_id: Option<Uuid>, provider: &str, model_id: &str, tokens: u32, estimated: bool) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO embedding_usage (user_id, provider, model_id, tokens, estimated) VALUES ($1, $2, $3, $4, $5)")
        .bind(user_id)
        .bind(provider)
        .bind(model_id)
        .bind(tokens.min(i32::MAX as u32) as i32)
        .bind(estimated)
        .execute(pool)
        .await?;
    Ok(())
}
