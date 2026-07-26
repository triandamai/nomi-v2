use chrono::{DateTime, Utc};
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

const AGENT_EXPIRY_HOURS: i64 = 24;

#[derive(Debug, Clone, PartialEq)]
pub struct ActiveAgentSessionDetails {
    pub agent_type: String,
    pub last_activity_at: DateTime<Utc>,
}

pub async fn load_active_agent_session_details(
    conn: &mut PoolConnection<Postgres>,
    agent_session_id: Uuid,
) -> Result<ActiveAgentSessionDetails, TurnError> {
    let (agent_type, last_activity_at): (String, DateTime<Utc>) =
        sqlx::query_as("SELECT agent_type, last_activity_at FROM agent_sessions WHERE id = $1")
            .bind(agent_session_id)
            .fetch_one(&mut **conn)
            .await?;
    Ok(ActiveAgentSessionDetails { agent_type, last_activity_at })
}

pub fn is_stale(last_activity_at: DateTime<Utc>) -> bool {
    Utc::now() - last_activity_at > chrono::Duration::hours(AGENT_EXPIRY_HOURS)
}

pub async fn mark_expired(
    conn: &mut PoolConnection<Postgres>,
    agent_session_id: Uuid,
    session_id: Uuid,
    agent_type: &str,
) -> Result<(), TurnError> {
    sqlx::query("UPDATE agent_sessions SET status = 'expired', ended_at = now() WHERE id = $1")
        .bind(agent_session_id)
        .execute(&mut **conn)
        .await?;

    sqlx::query("INSERT INTO agent_events (session_id, agent_session_id, agent_type, event_type) VALUES ($1, $2, $3, 'AgentExpired')")
        .bind(session_id)
        .bind(agent_session_id)
        .bind(agent_type)
        .execute(&mut **conn)
        .await?;

    Ok(())
}

pub async fn spawn_agent_session(
    conn: &mut PoolConnection<Postgres>,
    session_id: Uuid,
    sender_channel_identity_id: Uuid,
    agent_type: &str,
) -> Result<Uuid, TurnError> {
    let agent_session_id: Uuid = sqlx::query_scalar(
        "INSERT INTO agent_sessions (session_id, sender_channel_identity_id, agent_type, status) VALUES ($1, $2, $3, 'active') RETURNING id",
    )
    .bind(session_id)
    .bind(sender_channel_identity_id)
    .bind(agent_type)
    .fetch_one(&mut **conn)
    .await?;

    sqlx::query("INSERT INTO agent_events (session_id, agent_session_id, agent_type, event_type) VALUES ($1, $2, $3, 'AgentSpawned')")
        .bind(session_id)
        .bind(agent_session_id)
        .bind(agent_type)
        .execute(&mut **conn)
        .await?;

    Ok(agent_session_id)
}

pub async fn complete_agent_session(
    conn: &mut PoolConnection<Postgres>,
    agent_session_id: Uuid,
    session_id: Uuid,
    agent_type: &str,
    status: &str,
    summary: &str,
) -> Result<(), TurnError> {
    sqlx::query("UPDATE agent_sessions SET status = $1, ended_at = now() WHERE id = $2")
        .bind(status)
        .bind(agent_session_id)
        .execute(&mut **conn)
        .await?;

    let event_type = if status == "cancelled" { "AgentCancelled" } else { "AgentCompleted" };

    sqlx::query("INSERT INTO agent_events (session_id, agent_session_id, agent_type, event_type, payload) VALUES ($1, $2, $3, $4, $5)")
        .bind(session_id)
        .bind(agent_session_id)
        .bind(agent_type)
        .bind(event_type)
        .bind(serde_json::json!({"summary": summary}))
        .execute(&mut **conn)
        .await?;

    Ok(())
}
