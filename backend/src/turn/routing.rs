use sqlx::pool::PoolConnection;
use sqlx::Postgres;
use uuid::Uuid;

use super::types::TurnError;
use crate::llm::{ContentBlock, LlmMessage, LlmProvider, LlmRequest, LlmRole};

const INTENT_CLASSIFICATION_SYSTEM_PROMPT: &str =
    "Classify the user's message as exactly one of: chitchat, money. Reply with only that single word, nothing else.";
const INTENT_CLASSIFICATION_MAX_TOKENS: u32 = 10;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Intent {
    Chitchat,
    Money,
}

pub async fn classify_intent(provider: &dyn LlmProvider, text: &str) -> Intent {
    let request = LlmRequest {
        system: Some(INTENT_CLASSIFICATION_SYSTEM_PROMPT.to_string()),
        messages: vec![LlmMessage { role: LlmRole::User, content: vec![ContentBlock::Text { text: text.to_string() }] }],
        tools: vec![],
        max_tokens: INTENT_CLASSIFICATION_MAX_TOKENS,
    };

    let response = match provider.complete(request).await {
        Ok(r) => r,
        Err(_) => return Intent::Chitchat,
    };

    let text = response
        .content
        .into_iter()
        .find_map(|block| match block {
            ContentBlock::Text { text } => Some(text),
            _ => None,
        })
        .unwrap_or_default();

    match text.trim().to_lowercase().as_str() {
        "money" => Intent::Money,
        _ => Intent::Chitchat,
    }
}

pub async fn find_active_agent_session(
    conn: &mut PoolConnection<Postgres>,
    session_id: Uuid,
    sender_channel_identity_id: Uuid,
) -> Result<Option<Uuid>, TurnError> {
    let id: Option<Uuid> = sqlx::query_scalar(
        "SELECT id FROM agent_sessions WHERE session_id = $1 AND sender_channel_identity_id = $2 AND status = 'active'",
    )
    .bind(session_id)
    .bind(sender_channel_identity_id)
    .fetch_optional(&mut **conn)
    .await?;
    Ok(id)
}
