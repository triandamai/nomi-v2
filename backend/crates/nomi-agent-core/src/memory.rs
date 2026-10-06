use sqlx::pool::PoolConnection;
use sqlx::{Postgres, PgPool};
use uuid::Uuid;

use crate::error::TurnError;
use nomi_embedding::EmbeddingProvider;
use nomi_llm::{ContentBlock, LlmMessage, LlmProvider, LlmRequest, LlmRole};

/// What kind of thing a memory is. Shown on the Memory page and given to the model with it.
pub const MEMORY_KINDS: [&str; 5] = ["preference", "person", "routine", "goal", "fact"];

#[derive(Debug, Clone, PartialEq)]
pub struct RetrievedMemory {
    pub id: Uuid,
    pub content: String,
    pub kind: String,
}

fn to_vector_literal(embedding: &[f32]) -> String {
    format!("[{}]", embedding.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(","))
}

/// Below this cosine similarity a memory has nothing to do with the message: it's left out
/// instead of being handed to the model just to fill the list. `MEMORY_MIN_SIMILARITY` adjusts
/// it for embedding models whose scores run higher or lower.
fn min_similarity() -> f64 {
    std::env::var("MEMORY_MIN_SIMILARITY").ok().and_then(|v| v.trim().parse().ok()).filter(|v: &f64| (0.0..1.0).contains(v)).unwrap_or(0.3)
}

/// Memories also have to be nearly as relevant as the best match (this share of its
/// similarity), so one strong match doesn't drag in loosely related ones.
const RELATIVE_SIMILARITY: f64 = 0.8;

/// The memories that matter for this message, best first. Relevance comes first; strength
/// (weight) breaks ties and lifts well-confirmed memories, without letting a strong but unrelated
/// memory crowd out a relevant one.
pub async fn retrieve_relevant_memories(
    conn: &mut PoolConnection<Postgres>,
    user_id: Uuid,
    query_embedding: &[f32],
    current_provider: &str,
    current_model: &str,
    limit: i64,
) -> Result<Vec<RetrievedMemory>, TurnError> {
    let literal = to_vector_literal(query_embedding);
    let rows: Vec<(Uuid, String, String, f64)> = sqlx::query_as(
        "SELECT id, content, kind, similarity FROM ( \
             SELECT id, content, kind, weight, (1 - (embedding <=> $4::vector)) AS similarity FROM memory_items \
             WHERE user_id = $1 AND embedding_provider = $2 AND embedding_model = $3 AND archived_at IS NULL \
         ) m WHERE similarity >= $6 \
         ORDER BY similarity * sqrt(weight) DESC \
         LIMIT $5",
    )
    .bind(user_id)
    .bind(current_provider)
    .bind(current_model)
    .bind(&literal)
    .bind(limit)
    .bind(min_similarity())
    .fetch_all(&mut **conn)
    .await?;

    let best = rows.iter().map(|r| r.3).fold(0.0, f64::max);
    Ok(rows
        .into_iter()
        .filter(|(_, _, _, similarity)| *similarity >= best * RELATIVE_SIMILARITY)
        .map(|(id, content, kind, _)| RetrievedMemory { id, content, kind })
        .collect())
}

pub async fn try_retrieve_memories(
    conn: &mut sqlx::pool::PoolConnection<sqlx::Postgres>,
    embedding_provider: &dyn nomi_embedding::EmbeddingProvider,
    user_id: uuid::Uuid,
    text: &str,
) -> Vec<RetrievedMemory> {
    const MEMORY_RETRIEVAL_LIMIT: i64 = 5;
    if text.trim().is_empty() {
        return Vec::new();
    }
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

/// A reply drew on these memories: they get a little stronger, and count as recently useful
/// (so they don't fade). A thumbs-down on the reply takes back more than this gives.
pub async fn mark_used(conn: &mut sqlx::PgConnection, memory_ids: &[Uuid]) -> Result<(), sqlx::Error> {
    if memory_ids.is_empty() {
        return Ok(());
    }
    sqlx::query("UPDATE memory_items SET weight = LEAST(weight * $2, $3), last_used_at = now() WHERE id = ANY($1)")
        .bind(memory_ids)
        .bind(USE_FACTOR)
        .bind(MAX_WEIGHT)
        .execute(conn)
        .await?;
    Ok(())
}

pub const MIN_WEIGHT: f64 = 0.1;
pub const MAX_WEIGHT: f64 = 5.0;
/// Each reply that uses a memory.
const USE_FACTOR: f64 = 1.02;
/// The same fact learned again.
const RELEARN_FACTOR: f64 = 1.1;

/// Room for the JSON answer. Reasoning models spend part of this on thinking before they
/// answer, so it can't be as tight as the answer itself.
const EXTRACTION_MAX_TOKENS: u32 = 1024;

/// A new fact this close (cosine distance) to one already stored is the same fact said again:
/// it strengthens the existing memory instead of adding a duplicate.
pub const DUPLICATE_DISTANCE: f64 = 0.08;

/// The width of `memory_items.embedding`.
const STORED_DIMENSIONS: usize = 1536;

/// How many known memories the extractor sees, so it can update or retire one instead of adding
/// a contradicting duplicate.
const KNOWN_FOR_EXTRACTION: i64 = 5;

/// Longest memory kept (they're meant to be one short line).
const MAX_MEMORY_CHARS: usize = 200;

/// What the extractor decided.
#[derive(Debug, Clone, PartialEq)]
pub enum MemoryChange {
    None,
    Add { kind: String, text: String },
    /// Known memory `target` (1-based, as listed to the model) changed or was corrected.
    Update { target: usize, kind: Option<String>, text: String },
    /// Known memory `target` is no longer true.
    Delete { target: usize },
}

fn clean_text(text: &str) -> Option<String> {
    let text = text.trim().trim_matches('"').trim();
    if text.is_empty() {
        return None;
    }
    Some(text.chars().take(MAX_MEMORY_CHARS).collect())
}

fn clean_kind(kind: Option<&str>) -> Option<String> {
    kind.map(|k| k.trim().to_lowercase()).filter(|k| MEMORY_KINDS.contains(&k.as_str()))
}

/// Reads the extractor's answer: the JSON it was asked for (possibly in a code fence), or a
/// plain-text fact from a model that ignored the format, or any spelling of "nothing".
pub fn parse_memory_change(answer: &str) -> MemoryChange {
    let trimmed = answer.trim();
    let json_text = match (trimmed.find('{'), trimmed.rfind('}')) {
        (Some(start), Some(end)) if end > start => Some(&trimmed[start..=end]),
        _ => None,
    };
    if let Some(value) = json_text.and_then(|t| serde_json::from_str::<serde_json::Value>(t).ok()) {
        let text = value.get("text").and_then(|v| v.as_str()).and_then(clean_text);
        let kind = clean_kind(value.get("kind").and_then(|v| v.as_str()));
        let target = value.get("target").and_then(|v| v.as_u64()).map(|n| n as usize).filter(|n| *n >= 1);
        return match value.get("action").and_then(|v| v.as_str()).unwrap_or("none") {
            "add" => text.map(|text| MemoryChange::Add { kind: kind.unwrap_or_else(|| "fact".to_string()), text }).unwrap_or(MemoryChange::None),
            "update" => match (target, text) {
                (Some(target), Some(text)) => MemoryChange::Update { target, kind, text },
                (None, Some(text)) => MemoryChange::Add { kind: kind.unwrap_or_else(|| "fact".to_string()), text },
                _ => MemoryChange::None,
            },
            "delete" => target.map(|target| MemoryChange::Delete { target }).unwrap_or(MemoryChange::None),
            _ => MemoryChange::None,
        };
    }
    match extracted_fact(trimmed) {
        Some(fact) => clean_text(&fact).map(|text| MemoryChange::Add { kind: "fact".to_string(), text }).unwrap_or(MemoryChange::None),
        None => MemoryChange::None,
    }
}

/// The extraction model's plain answer as a fact worth storing, or `None` for its "nothing to
/// remember" answer, which models write as NONE, "None.", "**NONE**" and the like.
fn extracted_fact(text: &str) -> Option<String> {
    let fact = text.trim();
    let bare: String = fact.chars().filter(|c| c.is_alphanumeric()).collect();
    if bare.is_empty() || bare.eq_ignore_ascii_case("none") {
        return None;
    }
    Some(fact.to_string())
}

/// After a reply: decides whether the exchange taught Nomi something lasting about the person,
/// and adds, updates or retires one memory accordingly.
pub async fn extract_and_store_memory(
    conn: &mut PoolConnection<Postgres>,
    provider: &dyn LlmProvider,
    embedding_provider: &dyn EmbeddingProvider,
    user_id: Uuid,
    user_text: &str,
    assistant_text: &str,
) {
    extract_and_store_memory_in(conn, provider, embedding_provider, user_id, None, user_text, assistant_text).await
}

/// [`extract_and_store_memory`], remembering which chat the memory came from (its latest message
/// from the person becomes the memory's source). Runs the model call inline; turns use
/// [`queue_learning`] instead so the reply never waits on it.
pub async fn extract_and_store_memory_in(
    conn: &mut PoolConnection<Postgres>,
    provider: &dyn LlmProvider,
    embedding_provider: &dyn EmbeddingProvider,
    user_id: Uuid,
    session_id: Option<Uuid>,
    user_text: &str,
    assistant_text: &str,
) {
    let Some((source_message_id, user_text)) = what_the_person_said(conn, session_id, user_text).await else {
        return;
    };
    learn(conn, provider, embedding_provider, user_id, source_message_id, &user_text, assistant_text).await
}

/// The person's own latest message in the chat, and its id. The caller's `user_text` is the
/// agent's last user turn, which after a tool call is a tool result (no text at all) and for a
/// delegated agent is its task brief, so the chat itself is the reliable source. `None` when
/// there's nothing they said to learn from.
async fn what_the_person_said(
    conn: &mut PoolConnection<Postgres>,
    session_id: Option<Uuid>,
    fallback: &str,
) -> Option<(Option<Uuid>, String)> {
    let latest: Option<(Uuid, String)> = match session_id {
        Some(session_id) => sqlx::query_as(
            "SELECT id, content FROM messages WHERE session_id = $1 AND sender_channel_identity_id IS NOT NULL \
             ORDER BY created_at DESC LIMIT 1",
        )
        .bind(session_id)
        .fetch_optional(&mut **conn)
        .await
        .ok()
        .flatten(),
        None => None,
    };
    let (source, text) = match latest {
        Some((id, content)) if !content.trim().is_empty() => (Some(id), content),
        _ => (None, fallback.to_string()),
    };
    (!text.trim().is_empty()).then_some((source, text))
}

/// After an agent answers, queues the exchange for the memory worker to learn from. Cheap: one
/// insert, so the reply goes out without waiting on a model call.
pub async fn queue_learning(
    conn: &mut PoolConnection<Postgres>,
    user_id: Uuid,
    session_id: Uuid,
    fallback_user_text: &str,
    answer: &str,
) {
    let Some((source_message_id, person_text)) = what_the_person_said(conn, Some(session_id), fallback_user_text).await else {
        return;
    };
    let queued = sqlx::query("INSERT INTO memory_jobs (user_id, source_message_id, person_text, answer) VALUES ($1, $2, $3, $4)")
        .bind(user_id)
        .bind(source_message_id)
        .bind(&person_text)
        .bind(answer)
        .execute(&mut **conn)
        .await;
    if let Err(e) = queued {
        tracing::warn!(error = %e, "memory: failed to queue the exchange to learn from");
    }
}

/// One queued exchange to learn from (see [`queue_learning`]).
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct LearningJob {
    pub id: Uuid,
    pub user_id: Uuid,
    pub source_message_id: Option<Uuid>,
    pub person_text: String,
    pub answer: String,
}

/// How often a job is tried before it's dropped (a model that keeps failing or timing out).
pub const LEARNING_ATTEMPTS: i32 = 3;

/// The oldest queued exchanges, counting this as an attempt for each.
pub async fn take_learning_jobs(pool: &sqlx::PgPool, limit: i64) -> Result<Vec<LearningJob>, sqlx::Error> {
    sqlx::query_as(
        "UPDATE memory_jobs SET attempts = attempts + 1 \
         WHERE id IN (SELECT id FROM memory_jobs WHERE attempts < $2 ORDER BY created_at LIMIT $1 FOR UPDATE SKIP LOCKED) \
         RETURNING id, user_id, source_message_id, person_text, answer",
    )
    .bind(limit)
    .bind(LEARNING_ATTEMPTS)
    .fetch_all(pool)
    .await
}

/// Learns from one queued exchange, then removes it from the queue.
pub async fn run_learning_job(
    pool: &sqlx::PgPool,
    provider: &dyn LlmProvider,
    embedding_provider: &dyn EmbeddingProvider,
    job: &LearningJob,
) -> Result<(), sqlx::Error> {
    let mut conn = pool.acquire().await?;
    learn(&mut conn, provider, embedding_provider, job.user_id, job.source_message_id, &job.person_text, &job.answer).await;
    sqlx::query("DELETE FROM memory_jobs WHERE id = $1").bind(job.id).execute(&mut *conn).await?;
    Ok(())
}

/// Clears jobs that ran out of attempts.
pub async fn drop_failed_learning_jobs(pool: &sqlx::PgPool) -> Result<u64, sqlx::Error> {
    Ok(sqlx::query("DELETE FROM memory_jobs WHERE attempts >= $1").bind(LEARNING_ATTEMPTS).execute(pool).await?.rows_affected())
}

/// Decides whether the exchange taught Nomi something lasting about the person, and adds,
/// updates or retires one memory accordingly.
async fn learn(
    conn: &mut PoolConnection<Postgres>,
    provider: &dyn LlmProvider,
    embedding_provider: &dyn EmbeddingProvider,
    user_id: Uuid,
    source_message_id: Option<Uuid>,
    user_text: &str,
    assistant_text: &str,
) {
    // What's already known near this topic, so the extractor can update instead of duplicate.
    let known: Vec<(Uuid, String, String)> = match embedding_provider.embed_for_query(user_text).await {
        Ok(query) if query.len() == STORED_DIMENSIONS => sqlx::query_as(
            "SELECT id, kind, content FROM memory_items \
             WHERE user_id = $1 AND embedding_provider = $2 AND embedding_model = $3 AND archived_at IS NULL \
               AND (1 - (embedding <=> $4::vector)) >= $6 \
             ORDER BY embedding <=> $4::vector LIMIT $5",
        )
        .bind(user_id)
        .bind(embedding_provider.provider_name())
        .bind(embedding_provider.model_id())
        .bind(to_vector_literal(&query))
        .bind(KNOWN_FOR_EXTRACTION)
        .bind(min_similarity())
        .fetch_all(&mut **conn)
        .await
        .unwrap_or_default(),
        _ => Vec::new(),
    };

    let mut prompt = String::new();
    if known.is_empty() {
        prompt.push_str("Known memories: none related.\n\n");
    } else {
        prompt.push_str("Known memories:\n");
        for (i, (_, kind, content)) in known.iter().enumerate() {
            prompt.push_str(&format!("{}. ({kind}) {content}\n", i + 1));
        }
        prompt.push('\n');
    }
    prompt.push_str(&format!("The person wrote:\n{}\n\nThe assistant replied:\n{}", user_text.trim(), assistant_text.trim()));

    let request = LlmRequest {
        system: Some(crate::prompts::MEMORY_EXTRACTION_SYSTEM_PROMPT.to_string()),
        messages: vec![LlmMessage { role: LlmRole::User, content: vec![ContentBlock::Text { text: prompt }] }],
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
    let answer = response.content.into_iter().find_map(|block| match block {
        ContentBlock::Text { text } => Some(text),
        _ => None,
    });
    if answer.as_deref().is_none_or(|a| a.trim().is_empty()) {
        tracing::warn!(stop_reason = ?response.stop_reason, "memory: the extractor gave no answer");
    }
    let change = answer.as_deref().map(parse_memory_change).unwrap_or(MemoryChange::None);

    let result = match change {
        MemoryChange::None => Ok(()),
        MemoryChange::Delete { target } => match known.get(target - 1) {
            Some((id, _, _)) => sqlx::query("UPDATE memory_items SET archived_at = now(), updated_at = now() WHERE id = $1")
                .bind(id)
                .execute(&mut **conn)
                .await
                .map(|_| ()),
            None => Ok(()),
        },
        MemoryChange::Add { kind, text } => store_new(conn, embedding_provider, user_id, &kind, &text, source_message_id, None).await,
        MemoryChange::Update { target, kind, text } => match known.get(target - 1) {
            Some((old_id, old_kind, _)) => {
                let kind = kind.unwrap_or_else(|| old_kind.clone());
                store_new(conn, embedding_provider, user_id, &kind, &text, source_message_id, Some(*old_id)).await
            }
            None => store_new(conn, embedding_provider, user_id, &kind.unwrap_or_else(|| "fact".to_string()), &text, source_message_id, None).await,
        },
    };
    if let Err(e) = result {
        tracing::warn!(error = %e, "memory: storing the extracted fact failed");
    }
}

/// Stores a new memory (or strengthens the identical one already there). When it `replaces` an
/// older memory, it takes over that memory's strength and the old one is retired.
async fn store_new(
    conn: &mut PoolConnection<Postgres>,
    embedding_provider: &dyn EmbeddingProvider,
    user_id: Uuid,
    kind: &str,
    text: &str,
    source_message_id: Option<Uuid>,
    replaces: Option<Uuid>,
) -> Result<(), sqlx::Error> {
    let embedding = match embedding_provider.embed(text).await {
        Ok(e) => e,
        Err(e) => {
            tracing::warn!(error = %e, provider = embedding_provider.provider_name(), "memory: embedding the extracted fact failed");
            return Ok(());
        }
    };
    if embedding.len() != STORED_DIMENSIONS {
        tracing::warn!(
            got = embedding.len(),
            expected = STORED_DIMENSIONS,
            model = embedding_provider.model_id(),
            "memory: the embedding model returns vectors of the wrong size, so memories can't be stored; pick a model that outputs 1536 dimensions",
        );
        return Ok(());
    }
    let literal = to_vector_literal(&embedding);

    let duplicate: Option<Uuid> = sqlx::query_scalar(
        "SELECT id FROM memory_items \
         WHERE user_id = $1 AND embedding_provider = $2 AND embedding_model = $3 AND archived_at IS NULL \
           AND (embedding <=> $4::vector) < $5 AND id IS DISTINCT FROM $6 \
         ORDER BY embedding <=> $4::vector LIMIT 1",
    )
    .bind(user_id)
    .bind(embedding_provider.provider_name())
    .bind(embedding_provider.model_id())
    .bind(&literal)
    .bind(DUPLICATE_DISTANCE)
    // The memory being replaced is close by nature ("lives in Jakarta" → "lives in Bandung").
    .bind(replaces)
    .fetch_optional(&mut **conn)
    .await?;
    if let Some(existing) = duplicate {
        sqlx::query("UPDATE memory_items SET weight = LEAST(weight * $2, $3), updated_at = now() WHERE id = $1")
            .bind(existing)
            .bind(RELEARN_FACTOR)
            .bind(MAX_WEIGHT)
            .execute(&mut **conn)
            .await?;
        return Ok(());
    }

    let mut tx = sqlx::Acquire::begin(&mut **conn).await?;
    let new_id: Uuid = sqlx::query_scalar(
        "INSERT INTO memory_items (user_id, content, kind, embedding, embedding_provider, embedding_model, source_message_id, weight) \
         VALUES ($1, $2, $3, $4::vector, $5, $6, $7, COALESCE((SELECT weight FROM memory_items WHERE id = $8), 1.0)) RETURNING id",
    )
    .bind(user_id)
    .bind(text)
    .bind(kind)
    .bind(&literal)
    .bind(embedding_provider.provider_name())
    .bind(embedding_provider.model_id())
    .bind(source_message_id)
    .bind(replaces)
    .fetch_one(&mut *tx)
    .await?;
    if let Some(old) = replaces {
        sqlx::query("UPDATE memory_items SET archived_at = now(), superseded_by = $2, updated_at = now() WHERE id = $1")
            .bind(old)
            .bind(new_id)
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await
}

#[cfg(test)]
mod tests {
    use super::{extracted_fact, parse_memory_change, MemoryChange};

    #[test]
    fn every_spelling_of_none_means_nothing_to_remember() {
        for text in ["NONE", "None.", "**NONE**", " none ", "\"NONE\"", ""] {
            assert_eq!(extracted_fact(text), None, "{text:?}");
        }
        assert_eq!(extracted_fact(" The user is vegetarian. "), Some("The user is vegetarian.".to_string()));
    }

    #[test]
    fn the_extractors_json_becomes_a_change() {
        assert_eq!(parse_memory_change(r#"{"action":"none"}"#), MemoryChange::None);
        assert_eq!(
            parse_memory_change("```json\n{\"action\":\"add\",\"kind\":\"preference\",\"text\":\"Vegetarian\"}\n```"),
            MemoryChange::Add { kind: "preference".into(), text: "Vegetarian".into() }
        );
        assert_eq!(
            parse_memory_change(r#"{"action":"update","target":2,"text":"Lives in Bandung"}"#),
            MemoryChange::Update { target: 2, kind: None, text: "Lives in Bandung".into() }
        );
        assert_eq!(parse_memory_change(r#"{"action":"delete","target":1}"#), MemoryChange::Delete { target: 1 });
        // An unknown kind becomes a plain fact; a missing target can't update anything.
        assert_eq!(
            parse_memory_change(r#"{"action":"add","kind":"vibe","text":"Likes jazz"}"#),
            MemoryChange::Add { kind: "fact".into(), text: "Likes jazz".into() }
        );
        assert_eq!(parse_memory_change(r#"{"action":"delete"}"#), MemoryChange::None);
    }

    #[test]
    fn a_plain_answer_still_counts() {
        assert_eq!(parse_memory_change("Vegetarian"), MemoryChange::Add { kind: "fact".into(), text: "Vegetarian".into() });
        assert_eq!(parse_memory_change("NONE"), MemoryChange::None);
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

impl ReinforcementSignal {
    /// A thumbs-up or thumbs-down on a reply.
    pub fn from_rating(rating: &str) -> Option<Self> {
        match rating {
            "up" => Some(Self::Positive),
            "down" => Some(Self::Negative),
            _ => None,
        }
    }

    fn factor(self) -> f64 {
        match self {
            Self::Positive => 1.2,
            Self::Negative => 0.8,
        }
    }
}

/// Strengthens (thumbs-up) or weakens (thumbs-down) every memory the reply drew on.
pub async fn reinforce(pool: &PgPool, reply_message_id: Uuid, signal: ReinforcementSignal) -> Result<(), ReinforcementError> {
    reinforce_change(pool, reply_message_id, None, Some(signal)).await
}

/// Applies a change of mind on a reply: from `previous` (none, up or down) to `current`. The old
/// rating's effect is taken back before the new one is applied, so flipping a thumb, or removing
/// it, never compounds.
pub async fn reinforce_change(
    pool: &PgPool,
    reply_message_id: Uuid,
    previous: Option<ReinforcementSignal>,
    current: Option<ReinforcementSignal>,
) -> Result<(), ReinforcementError> {
    let factor = current.map_or(1.0, ReinforcementSignal::factor) / previous.map_or(1.0, ReinforcementSignal::factor);
    if (factor - 1.0).abs() < f64::EPSILON {
        return Ok(());
    }
    sqlx::query(
        "UPDATE memory_items SET weight = LEAST(GREATEST(weight * $1, $3), $4) \
         WHERE id IN (SELECT memory_id FROM message_memory_usage WHERE message_id = $2)",
    )
    .bind(factor)
    .bind(reply_message_id)
    .bind(MIN_WEIGHT)
    .bind(MAX_WEIGHT)
    .execute(pool)
    .await?;
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum EditMemoryError {
    #[error("memory not found")]
    NotFound,
    #[error("a memory needs some text, at most {MAX_MEMORY_CHARS} characters")]
    InvalidText,
    #[error("unknown kind")]
    InvalidKind,
    #[error("couldn't embed the new text: {0}")]
    Embedding(String),
    #[error(transparent)]
    Db(#[from] sqlx::Error),
}

/// The person rewrote one of their memories. It's re-embedded so recall finds it by its new
/// words, and counts as confirmed (they just vouched for it).
pub async fn edit(
    pool: &PgPool,
    embedding_provider: &dyn EmbeddingProvider,
    user_id: Uuid,
    memory_id: Uuid,
    text: &str,
    kind: Option<&str>,
) -> Result<(), EditMemoryError> {
    let text = text.trim();
    if text.is_empty() || text.chars().count() > MAX_MEMORY_CHARS {
        return Err(EditMemoryError::InvalidText);
    }
    let kind = match kind {
        Some(k) => Some(clean_kind(Some(k)).ok_or(EditMemoryError::InvalidKind)?),
        None => None,
    };
    let owned: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM memory_items WHERE id = $1 AND user_id = $2)")
        .bind(memory_id)
        .bind(user_id)
        .fetch_one(pool)
        .await?;
    if !owned {
        return Err(EditMemoryError::NotFound);
    }
    let embedding = embedding_provider.embed(text).await.map_err(|e| EditMemoryError::Embedding(e.to_string()))?;
    if embedding.len() != STORED_DIMENSIONS {
        return Err(EditMemoryError::Embedding(format!("expected {STORED_DIMENSIONS} dimensions, got {}", embedding.len())));
    }
    sqlx::query(
        "UPDATE memory_items SET content = $3, kind = COALESCE($4, kind), embedding = $5::vector, \
             embedding_provider = $6, embedding_model = $7, confirmed_at = now(), updated_at = now(), \
             weight = GREATEST(weight, 1.0), archived_at = NULL \
         WHERE id = $1 AND user_id = $2",
    )
    .bind(memory_id)
    .bind(user_id)
    .bind(text)
    .bind(kind)
    .bind(to_vector_literal(&embedding))
    .bind(embedding_provider.provider_name())
    .bind(embedding_provider.model_id())
    .execute(pool)
    .await?;
    Ok(())
}

/// The person says a memory is still true: it stops fading and gets a little stronger.
/// Returns false when there's no such memory of theirs.
pub async fn confirm(pool: &PgPool, user_id: Uuid, memory_id: Uuid) -> Result<bool, sqlx::Error> {
    let updated = sqlx::query(
        "UPDATE memory_items SET confirmed_at = now(), weight = LEAST(weight * 1.1, $3) \
         WHERE id = $1 AND user_id = $2 AND archived_at IS NULL",
    )
    .bind(memory_id)
    .bind(user_id)
    .bind(MAX_WEIGHT)
    .execute(pool)
    .await?;
    Ok(updated.rows_affected() > 0)
}
