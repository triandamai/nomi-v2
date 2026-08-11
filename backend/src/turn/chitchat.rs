use sqlx::pool::PoolConnection;
use sqlx::{Acquire, Postgres};
use uuid::Uuid;

use crate::embedding::EmbeddingProvider;
use crate::llm::{ContentBlock, LlmMessage, LlmProvider, LlmRequest, LlmRole};
use crate::realtime::{MqttPublisher, StreamEnvelope};

use super::memory::{self, RetrievedMemory};
use super::types::TurnError;

const CHITCHAT_SYSTEM_PROMPT: &str =
    "You are a helpful, friendly assistant chatting with the user. Keep replies concise.";
const CHITCHAT_HISTORY_LIMIT: i64 = 20;
const CHITCHAT_MAX_TOKENS: u32 = 1024;
const MEMORY_RETRIEVAL_LIMIT: i64 = 5;

pub async fn run_chitchat_turn(
    conn: &mut PoolConnection<Postgres>,
    mqtt: Option<(&MqttPublisher, Uuid)>,
    provider: &dyn LlmProvider,
    embedding_provider: &dyn EmbeddingProvider,
    session_id: Uuid,
    user_id: Uuid,
    text: &str,
) -> Result<String, TurnError> {
    let rows: Vec<(Option<Uuid>, String)> = sqlx::query_as(
        "SELECT sender_channel_identity_id, content FROM ( \
             SELECT sender_channel_identity_id, content, created_at FROM messages \
             WHERE session_id = $1 ORDER BY created_at DESC LIMIT $2 \
         ) recent ORDER BY created_at ASC",
    )
    .bind(session_id)
    .bind(CHITCHAT_HISTORY_LIMIT)
    .fetch_all(&mut **conn)
    .await?;

    let messages: Vec<LlmMessage> = rows
        .into_iter()
        .map(|(sender, content)| LlmMessage {
            role: if sender.is_some() { LlmRole::User } else { LlmRole::Assistant },
            content: vec![ContentBlock::Text { text: content }],
        })
        .collect();

    let memories = try_retrieve_memories(conn, embedding_provider, user_id, text).await;

    let system_prompt = if memories.is_empty() {
        CHITCHAT_SYSTEM_PROMPT.to_string()
    } else {
        let mut prompt = format!("{CHITCHAT_SYSTEM_PROMPT}\n\nRelevant things you know about this user:\n");
        for m in &memories {
            prompt.push_str(&format!("- {}\n", m.content));
        }
        prompt
    };

    let request = LlmRequest {
        system: Some(system_prompt),
        messages,
        tools: vec![],
        max_tokens: CHITCHAT_MAX_TOKENS,
    };

    // The LLM call happens outside any DB transaction: holding a transaction open across a
    // slow network round trip would needlessly extend how long this connection's locks are held.
    let stream = provider.complete_stream(request).await.map_err(TurnError::LlmCallFailed)?;
    let response = match mqtt {
        Some((publisher, turn_job_id)) => {
            use futures_util::StreamExt;
            // LlmEventStream requires 'static (it's boxed as `dyn Stream + Send` with no
            // lifetime), so this clones the publisher handle (cheap — it wraps rumqttc's
            // AsyncClient, itself a cheap handle clone) rather than capturing the `&MqttPublisher`
            // borrow, which would tie the `Then` adapter's concrete type to `mqtt`'s lifetime.
            let publisher = publisher.clone();
            let published = stream.then(move |event_result| {
                let publisher = publisher.clone();
                async move {
                    if let Ok(event) = &event_result {
                        let envelope = StreamEnvelope::Delta { turn_job_id, event: event.clone() };
                        // Best-effort: an MQTT publish failure never fails the turn (Global Constraints).
                        let _ = publisher.publish(session_id, &envelope).await;
                    }
                    event_result
                }
            });
            crate::llm::collect_stream(Box::pin(published)).await.map_err(TurnError::LlmCallFailed)?
        }
        None => crate::llm::collect_stream(stream).await.map_err(TurnError::LlmCallFailed)?,
    };

    let reply_text = response
        .content
        .into_iter()
        .find_map(|block| match block {
            ContentBlock::Text { text } => Some(text),
            _ => None,
        })
        .unwrap_or_default();

    let mut tx = conn.begin().await?;

    let reply_message_id: Uuid = sqlx::query_scalar(
        "INSERT INTO messages (session_id, sender_channel_identity_id, content) VALUES ($1, NULL, $2) RETURNING id",
    )
    .bind(session_id)
    .bind(&reply_text)
    .fetch_one(&mut *tx)
    .await?;

    sqlx::query("INSERT INTO agent_events (session_id, event_type, payload) VALUES ($1, 'ChitchatReply', $2)")
        .bind(session_id)
        .bind(serde_json::json!({
            "input_tokens": response.input_tokens,
            "output_tokens": response.output_tokens,
        }))
        .execute(&mut *tx)
        .await?;

    for m in &memories {
        sqlx::query("INSERT INTO message_memory_usage (message_id, memory_id) VALUES ($1, $2)")
            .bind(reply_message_id)
            .bind(m.id)
            .execute(&mut *tx)
            .await?;
    }

    tx.commit().await?;

    // Best-effort: this never changes the turn's outcome (Ok(reply_text) below is unaffected
    // by anything that happens here) — but it does run synchronously before this function
    // returns, adding one extraction LLM call plus one embedding call of latency to every
    // turn. No channel adapter calls this yet, so that cost isn't user-visible today; revisit
    // (e.g. spawn this as a detached task) before wiring a real channel to handle_inbound_message.
    memory::extract_and_store_memory(conn, provider, embedding_provider, user_id, text, &reply_text).await;

    Ok(reply_text)
}

async fn try_retrieve_memories(
    conn: &mut PoolConnection<Postgres>,
    embedding_provider: &dyn EmbeddingProvider,
    user_id: Uuid,
    text: &str,
) -> Vec<RetrievedMemory> {
    let embedding = match embedding_provider.embed(text).await {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };
    memory::retrieve_relevant_memories(conn, user_id, &embedding, MEMORY_RETRIEVAL_LIMIT)
        .await
        .unwrap_or_default()
}
