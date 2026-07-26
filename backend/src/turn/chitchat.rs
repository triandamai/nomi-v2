use sqlx::pool::PoolConnection;
use sqlx::{Acquire, Postgres};
use uuid::Uuid;

use crate::llm::{ContentBlock, LlmMessage, LlmProvider, LlmRequest, LlmRole};

use super::types::TurnError;

const CHITCHAT_SYSTEM_PROMPT: &str =
    "You are a helpful, friendly assistant chatting with the user. Keep replies concise.";
const CHITCHAT_HISTORY_LIMIT: i64 = 20;
const CHITCHAT_MAX_TOKENS: u32 = 1024;

pub async fn run_chitchat_turn(
    conn: &mut PoolConnection<Postgres>,
    provider: &dyn LlmProvider,
    session_id: Uuid,
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

    let request = LlmRequest {
        system: Some(CHITCHAT_SYSTEM_PROMPT.to_string()),
        messages,
        tools: vec![],
        max_tokens: CHITCHAT_MAX_TOKENS,
    };

    // The LLM call happens outside any DB transaction: holding a transaction open across a
    // slow network round trip would needlessly extend how long this connection's locks are held.
    let response = provider.complete(request).await.map_err(TurnError::LlmCallFailed)?;

    let reply_text = response
        .content
        .into_iter()
        .find_map(|block| match block {
            ContentBlock::Text { text } => Some(text),
            _ => None,
        })
        .unwrap_or_default();

    let mut tx = conn.begin().await?;

    sqlx::query("INSERT INTO messages (session_id, sender_channel_identity_id, content) VALUES ($1, NULL, $2)")
        .bind(session_id)
        .bind(&reply_text)
        .execute(&mut *tx)
        .await?;

    sqlx::query("INSERT INTO agent_events (session_id, event_type, payload) VALUES ($1, 'ChitchatReply', $2)")
        .bind(session_id)
        .bind(serde_json::json!({
            "input_tokens": response.input_tokens,
            "output_tokens": response.output_tokens,
        }))
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

    Ok(reply_text)
}
