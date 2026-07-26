use sqlx::pool::PoolConnection;
use sqlx::Postgres;
use uuid::Uuid;

use super::types::TurnError;
use crate::embedding::EmbeddingProvider;
use crate::llm::{ContentBlock, LlmMessage, LlmProvider, LlmRequest, LlmRole};

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
    limit: i64,
) -> Result<Vec<RetrievedMemory>, TurnError> {
    let literal = to_vector_literal(query_embedding);
    let rows: Vec<(Uuid, String)> = sqlx::query_as(
        "SELECT id, content FROM memory_items \
         WHERE user_id = $1 \
         ORDER BY weight * (1 - (embedding <=> $2::vector)) DESC \
         LIMIT $3",
    )
    .bind(user_id)
    .bind(&literal)
    .bind(limit)
    .fetch_all(&mut **conn)
    .await?;

    Ok(rows.into_iter().map(|(id, content)| RetrievedMemory { id, content }).collect())
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

    let response = match provider.complete(request).await {
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
    let _ = sqlx::query("INSERT INTO memory_items (user_id, content, embedding) VALUES ($1, $2, $3::vector)")
        .bind(user_id)
        .bind(&fact)
        .bind(&literal)
        .execute(&mut **conn)
        .await;
}
