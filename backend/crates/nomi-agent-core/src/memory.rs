use sqlx::pool::PoolConnection;
use sqlx::{Postgres, PgPool};
use uuid::Uuid;

use crate::error::TurnError;
use nomi_embedding::EmbeddingProvider;
use nomi_llm::{ContentBlock, LlmMessage, LlmProvider, LlmRequest, LlmRole};

#[derive(Debug, Clone, PartialEq)]
pub struct RetrievedMemory {
    pub id: Uuid,
    pub content: String,
}

fn to_vector_literal(embedding: &[f32]) -> String {
    format!("[{}]", embedding.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(","))
}

pub async fn retrieve_relevant_memories(
    conn: &mut PoolConnection<Postgres>,
    user_id: Uuid,
    query_embedding: &[f32],
    current_provider: &str,
    current_model: &str,
    limit: i64,
) -> Result<Vec<RetrievedMemory>, TurnError> {
    let literal = to_vector_literal(query_embedding);
    let rows: Vec<(Uuid, String)> = sqlx::query_as(
        "SELECT id, content FROM memory_items \
         WHERE user_id = $1 AND embedding_provider = $2 AND embedding_model = $3 \
         ORDER BY weight * (1 - (embedding <=> $4::vector)) DESC \
         LIMIT $5",
    )
    .bind(user_id)
    .bind(current_provider)
    .bind(current_model)
    .bind(&literal)
    .bind(limit)
    .fetch_all(&mut **conn)
    .await?;

    Ok(rows.into_iter().map(|(id, content)| RetrievedMemory { id, content }).collect())
}

pub async fn try_retrieve_memories(
    conn: &mut sqlx::pool::PoolConnection<sqlx::Postgres>,
    embedding_provider: &dyn nomi_embedding::EmbeddingProvider,
    user_id: uuid::Uuid,
    text: &str,
) -> Vec<RetrievedMemory> {
    const MEMORY_RETRIEVAL_LIMIT: i64 = 5;
    let embedding = match embedding_provider.embed_for_query(text).await {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };
    retrieve_relevant_memories(
        conn,
        user_id,
        &embedding,
        embedding_provider.provider_name(),
        embedding_provider.model_id(),
        MEMORY_RETRIEVAL_LIMIT,
    )
    .await
    .unwrap_or_default()
}

const EXTRACTION_SYSTEM_PROMPT: &str =
    "Extract at most one durable fact worth remembering long-term from this exchange, or say NONE if nothing is worth storing.";
const EXTRACTION_MAX_TOKENS: u32 = 128;

pub async fn extract_and_store_memory(
    conn: &mut PoolConnection<Postgres>,
    provider: &dyn LlmProvider,
    embedding_provider: &dyn EmbeddingProvider,
    user_id: Uuid,
    user_text: &str,
    assistant_text: &str,
) {
    let request = LlmRequest {
        system: Some(EXTRACTION_SYSTEM_PROMPT.to_string()),
        messages: vec![
            LlmMessage { role: LlmRole::User, content: vec![ContentBlock::Text { text: user_text.to_string() }] },
            LlmMessage { role: LlmRole::Assistant, content: vec![ContentBlock::Text { text: assistant_text.to_string() }] },
        ],
        tools: vec![],
        max_tokens: EXTRACTION_MAX_TOKENS,
    };

    let response = match nomi_llm::complete(provider, request).await {
        Ok(r) => r,
        Err(_) => return,
    };

    let extracted = response.content.into_iter().find_map(|block| match block {
        ContentBlock::Text { text } => Some(text),
        _ => None,
    });

    let fact = match extracted {
        Some(f) if f.trim() != "NONE" && !f.trim().is_empty() => f.trim().to_string(),
        _ => return,
    };

    let embedding = match embedding_provider.embed(&fact).await {
        Ok(e) => e,
        Err(_) => return,
    };

    let literal = to_vector_literal(&embedding);
    let _ = sqlx::query(
        "INSERT INTO memory_items (user_id, content, embedding, embedding_provider, embedding_model) \
         VALUES ($1, $2, $3::vector, $4, $5)",
    )
    .bind(user_id)
    .bind(&fact)
    .bind(&literal)
    .bind(embedding_provider.provider_name())
    .bind(embedding_provider.model_id())
    .execute(&mut **conn)
    .await;
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ReinforcementSignal {
    Positive,
    Negative,
}

#[derive(Debug, thiserror::Error)]
pub enum ReinforcementError {
    #[error(transparent)]
    Db(#[from] sqlx::Error),
}

pub async fn reinforce(
    pool: &PgPool,
    reply_message_id: Uuid,
    signal: ReinforcementSignal,
) -> Result<(), ReinforcementError> {
    let factor: f64 = match signal {
        ReinforcementSignal::Positive => 1.2,
        ReinforcementSignal::Negative => 0.8,
    };

    sqlx::query(
        "UPDATE memory_items SET weight = LEAST(GREATEST(weight * $1, 0.1), 5.0) \
         WHERE id IN (SELECT memory_id FROM message_memory_usage WHERE message_id = $2)",
    )
    .bind(factor)
    .bind(reply_message_id)
    .execute(pool)
    .await?;

    Ok(())
}
