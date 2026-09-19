use std::sync::Arc;

use chrono::{DateTime, Utc};
use sqlx::pool::PoolConnection;
use sqlx::Postgres;
use uuid::Uuid;

use nomi_agent_core::{AgentRegistry, DynamicAgent, SubAgent, TurnError};
use nomi_agent_core::ToolCatalog;
use nomi_llm::{ContentBlock, LlmMessage, LlmProvider, LlmRequest, LlmRole};
use nomi_realtime::{MqttPublisher, StreamEnvelope};

// Generous for a single-word reply: some models don't reliably follow "reply with only one
// word" and add a short sentence around the label instead — find_by_intent_label's whole-word
// fallback handles that, but only if the label isn't truncated out of the response first.
const INTENT_CLASSIFICATION_MAX_TOKENS: u32 = 20;

/// Classifies `text` against every built-in agent in `registry` plus every currently-active
/// dynamic agent (fetched fresh from the DB, no cache), returning the matching agent — built-in
/// or dynamic — or the registry's default agent on no match, a parse failure, or an LLM error.
pub async fn classify_intent(
    conn: &mut PoolConnection<Postgres>,
    provider: &dyn LlmProvider,
    registry: &AgentRegistry,
    catalog: &Arc<ToolCatalog>,
    text: &str,
) -> Arc<dyn SubAgent> {
    let dynamic_rows = nomi_agent_core::dynamic_agent::fetch_active_dynamic_agents(conn).await.unwrap_or_default();
    let extra_labels: Vec<String> = dynamic_rows.iter().map(|r| r.intent_label.clone()).collect();
    let extra_options: Vec<String> =
        dynamic_rows.iter().map(|r| format!("{}: {}", r.intent_label, r.intent_description)).collect();

    let request = LlmRequest {
        system: Some(registry.classification_prompt_with_extra(&extra_labels, &extra_options)),
        messages: vec![LlmMessage { role: LlmRole::User, content: vec![ContentBlock::Text { text: text.to_string() }] }],
        tools: vec![],
        max_tokens: INTENT_CLASSIFICATION_MAX_TOKENS,
        enable_reasoning: false,
    };

    let response = match nomi_llm::complete(provider, request).await {
        Ok(r) => r,
        Err(_) => return registry.default_agent(),
    };

    let label_text = response
        .content
        .into_iter()
        .find_map(|block| match block {
            ContentBlock::Text { text } => Some(text),
            _ => None,
        })
        .unwrap_or_default();

    if let Some(agent) = registry.find_by_intent_label(&label_text) {
        return agent;
    }

    let normalized = label_text.trim().to_lowercase();
    let matched = dynamic_rows
        .iter()
        .find(|r| r.intent_label == normalized)
        .or_else(|| dynamic_rows.iter().find(|r| normalized.split(|c: char| !c.is_alphanumeric()).any(|w| w == r.intent_label)));

    match matched {
        Some(row) => Arc::new(DynamicAgent::from_row(row.clone(), catalog.clone())) as Arc<dyn SubAgent>,
        None => registry.default_agent(),
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
    mqtt: Option<(&MqttPublisher, Uuid)>,
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

    if let Some((publisher, _)) = mqtt {
        let envelope = StreamEnvelope::AgentSessionEnded { agent_session_id, session_id, reason: "expired".to_string() };
        let _ = publisher.publish(session_id, &envelope).await;
    }

    Ok(())
}

pub async fn spawn_agent_session(
    conn: &mut PoolConnection<Postgres>,
    mqtt: Option<(&MqttPublisher, Uuid)>,
    session_id: Uuid,
    sender_channel_identity_id: Uuid,
    agent_type: &str,
    agent_display_name: &str,
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

    if let Some((publisher, _)) = mqtt {
        // Best-effort, and only resolved when there's actually a publisher to send it to —
        // mirrors this codebase's existing "MQTT is optional infrastructure" convention.
        let row: Option<(Option<String>, String, String)> = sqlx::query_as(
            "SELECT wc.email, ci.channel, ci.channel_user_id \
             FROM channel_identities ci \
             LEFT JOIN web_credentials wc ON wc.user_id = ci.user_id \
             WHERE ci.id = $1",
        )
        .bind(sender_channel_identity_id)
        .fetch_optional(&mut **conn)
        .await
        .ok()
        .flatten();

        if let Some((email, channel, channel_user_id)) = row {
            let sender_label = email.unwrap_or_else(|| format!("{channel}:{channel_user_id}"));
            let envelope = StreamEnvelope::AgentSessionStarted {
                agent_session_id,
                session_id,
                agent_type: agent_type.to_string(),
                agent_display_name: agent_display_name.to_string(),
                channel,
                sender_label,
            };
            let _ = publisher.publish(session_id, &envelope).await;
        }
    }

    Ok(agent_session_id)
}

pub async fn complete_agent_session(
    conn: &mut PoolConnection<Postgres>,
    mqtt: Option<(&MqttPublisher, Uuid)>,
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

    if let Some((publisher, _)) = mqtt {
        let reason = if status == "cancelled" { "cancelled" } else { "completed" };
        let envelope = StreamEnvelope::AgentSessionEnded { agent_session_id, session_id, reason: reason.to_string() };
        let _ = publisher.publish(session_id, &envelope).await;
    }

    Ok(())
}
