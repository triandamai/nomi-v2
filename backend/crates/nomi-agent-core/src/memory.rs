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

/// Room for the one-line fact. Reasoning models spend part of this on thinking before they
/// answer, so it can't be as tight as the fact itself.
const EXTRACTION_MAX_TOKENS: u32 = 1024;

/// A new fact this close (cosine distance) to one already stored is the same fact said again:
/// it strengthens the existing memory instead of adding a duplicate.
const DUPLICATE_DISTANCE: f64 = 0.08;

/// The width of `memory_items.embedding`.
const STORED_DIMENSIONS: usize = 1536;

/// The extraction model's answer as a fact worth storing, or `None` for its "nothing to
/// remember" answer, which models write as NONE, "None.", "**NONE**" and the like.
fn extracted_fact(text: &str) -> Option<String> {
    let fact = text.trim();
    let bare: String = fact.chars().filter(|c| c.is_alphanumeric()).collect();
    if bare.is_empty() || bare.eq_ignore_ascii_case("none") {
        return None;
    }
    Some(fact.to_string())
}

pub async fn extract_and_store_memory(
    conn: &mut PoolConnection<Postgres>,
    provider: &dyn LlmProvider,
    embedding_provider: &dyn EmbeddingProvider,
    user_id: Uuid,
    user_text: &str,
    assistant_text: &str,
) {
    let request = LlmRequest {
        system: Some(crate::prompts::MEMORY_EXTRACTION_SYSTEM_PROMPT.to_string()),
        messages: vec![
            LlmMessage { role: LlmRole::User, content: vec![ContentBlock::Text { text: user_text.to_string() }] },
            LlmMessage { role: LlmRole::Assistant, content: vec![ContentBlock::Text { text: assistant_text.to_string() }] },
        ],
        tools: vec![],
        max_tokens: EXTRACTION_MAX_TOKENS,
        enable_reasoning: false,
        reasoning_effort: Default::default(),
    };

    // Memory is best-effort and never fails the turn, but every way it can fail is logged:
    // these used to be swallowed silently, which left the memory page empty with no clue why.
    let response = match nomi_llm::complete(provider, request).await {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!(error = %e, "memory: fact extraction call failed");
            return;
        }
    };

    let extracted = response.content.into_iter().find_map(|block| match block {
        ContentBlock::Text { text } => Some(text),
        _ => None,
    });

    let Some(fact) = extracted.as_deref().and_then(extracted_fact) else {
        return;
    };

    let embedding = match embedding_provider.embed(&fact).await {
        Ok(e) => e,
        Err(e) => {
            tracing::warn!(error = %e, provider = embedding_provider.provider_name(), "memory: embedding the extracted fact failed");
            return;
        }
    };
    if embedding.len() != STORED_DIMENSIONS {
        tracing::warn!(
            got = embedding.len(),
            expected = STORED_DIMENSIONS,
            model = embedding_provider.model_id(),
            "memory: the embedding model returns vectors of the wrong size, so memories can't be stored; pick a model that outputs 1536 dimensions",
        );
        return;
    }

    let literal = to_vector_literal(&embedding);
    let duplicate: Result<Option<Uuid>, sqlx::Error> = sqlx::query_scalar(
        "SELECT id FROM memory_items \
         WHERE user_id = $1 AND embedding_provider = $2 AND embedding_model = $3 \
           AND (embedding <=> $4::vector) < $5 \
         ORDER BY embedding <=> $4::vector LIMIT 1",
    )
    .bind(user_id)
    .bind(embedding_provider.provider_name())
    .bind(embedding_provider.model_id())
    .bind(&literal)
    .bind(DUPLICATE_DISTANCE)
    .fetch_optional(&mut **conn)
    .await;
    if let Ok(Some(existing)) = duplicate {
        let _ = sqlx::query("UPDATE memory_items SET weight = LEAST(weight * 1.1, 5.0), updated_at = now() WHERE id = $1")
            .bind(existing)
            .execute(&mut **conn)
            .await;
        return;
    }

    let inserted = sqlx::query(
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
    if let Err(e) = inserted {
        tracing::warn!(error = %e, "memory: storing the extracted fact failed");
    }
}

#[cfg(test)]
mod tests {
    use super::extracted_fact;

    #[test]
    fn every_spelling_of_none_means_nothing_to_remember() {
        for text in ["NONE", "None.", "**NONE**", " none ", "\"NONE\"", ""] {
            assert_eq!(extracted_fact(text), None, "{text:?}");
        }
        assert_eq!(extracted_fact(" The user is vegetarian. "), Some("The user is vegetarian.".to_string()));
    }
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
