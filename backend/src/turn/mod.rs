pub mod bootstrap;
pub mod chitchat;
pub mod ingest;
pub mod lock;
pub mod memory;
pub mod money_agent;
pub mod queue;
pub mod routing;
pub mod subagent;
pub mod tools;
pub mod types;

pub use types::{TurnError, TurnOutcome};

use sqlx::pool::PoolConnection;
use sqlx::{Acquire, PgPool, Postgres};
use uuid::Uuid;

use crate::embedding::EmbeddingProvider;
use crate::llm::{ContentBlock, LlmMessage, LlmProvider, LlmRole};
use crate::realtime::{MqttPublisher, StreamEnvelope};

const SUBAGENT_HISTORY_LIMIT: i64 = 20;
const SUBAGENT_MAX_TOKENS: u32 = 1024;

enum RoutingOutcome {
    Continue(Uuid),
    NeedsClassification,
    FallbackToChitchat,
}

pub async fn handle_inbound_message(
    pool: &PgPool,
    provider: &dyn LlmProvider,
    embedding_provider: &dyn EmbeddingProvider,
    channel: &str,
    chat_type: &str,
    chat_id: &str,
    sender_channel_user_id: &str,
    text: &str,
    org_id_hint: Option<Uuid>,
) -> Result<TurnOutcome, TurnError> {
    let bootstrap::BootstrapResult { user_id, sender_channel_identity_id, session_id, .. } =
        bootstrap::bootstrap_identity_and_session(pool, channel, chat_type, chat_id, sender_channel_user_id, org_id_hint)
            .await?;

    let mut conn = lock::acquire_session_lock(pool, session_id).await?;

    lock::insert_inbound_message(&mut conn, session_id, sender_channel_identity_id, text).await?;

    let result = run_locked_turn(&mut conn, None, provider, embedding_provider, session_id, sender_channel_identity_id, user_id, text).await;

    match result {
        Ok(reply) => {
            release_lock_ignoring_errors(&mut conn, session_id).await;
            Ok(TurnOutcome { session_id, reply })
        }
        Err(err) => {
            let _ = sqlx::query(
                "INSERT INTO agent_events (session_id, event_type, payload) VALUES ($1, 'TurnFailed', $2)",
            )
            .bind(session_id)
            .bind(serde_json::json!({"error": err.to_string()}))
            .execute(&mut *conn)
            .await;

            release_lock_ignoring_errors(&mut conn, session_id).await;
            Err(err)
        }
    }
}

/// The worker's entry point (see backend/src/bin/worker.rs): processes an already-ingested
/// message (see turn::ingest::ingest_inbound_message) — bootstrap and the inbound message
/// insert have already happened, so this only acquires the session lock and runs routing/
/// dispatch, threading `mqtt`/`turn_job_id` through to chitchat for live delta publishing.
pub async fn process_turn(
    pool: &PgPool,
    mqtt: &MqttPublisher,
    provider: &dyn LlmProvider,
    embedding_provider: &dyn EmbeddingProvider,
    turn_job_id: Uuid,
    session_id: Uuid,
    sender_channel_identity_id: Uuid,
    user_id: Uuid,
    text: &str,
) -> Result<TurnOutcome, TurnError> {
    let mut conn = lock::acquire_session_lock(pool, session_id).await?;

    let result = run_locked_turn(
        &mut conn,
        Some((mqtt, turn_job_id)),
        provider,
        embedding_provider,
        session_id,
        sender_channel_identity_id,
        user_id,
        text,
    )
    .await;

    match result {
        Ok(reply) => {
            release_lock_ignoring_errors(&mut conn, session_id).await;
            Ok(TurnOutcome { session_id, reply })
        }
        Err(err) => {
            let _ = sqlx::query(
                "INSERT INTO agent_events (session_id, event_type, payload) VALUES ($1, 'TurnFailed', $2)",
            )
            .bind(session_id)
            .bind(serde_json::json!({"error": err.to_string()}))
            .execute(&mut *conn)
            .await;

            // Best-effort: an MQTT publish failure never changes the turn's outcome.
            let _ = mqtt
                .publish(session_id, &StreamEnvelope::TurnFailed { turn_job_id, error: err.to_string() })
                .await;

            release_lock_ignoring_errors(&mut conn, session_id).await;
            Err(err)
        }
    }
}

/// Shared by handle_inbound_message and process_turn: routing/classification and
/// chitchat/subagent dispatch, assuming the session lock is already held by the caller and
/// the inbound message has already been persisted (by the caller, before this runs).
async fn run_locked_turn(
    conn: &mut PoolConnection<Postgres>,
    mqtt: Option<(&MqttPublisher, Uuid)>,
    provider: &dyn LlmProvider,
    embedding_provider: &dyn EmbeddingProvider,
    session_id: Uuid,
    sender_channel_identity_id: Uuid,
    user_id: Uuid,
    text: &str,
) -> Result<String, TurnError> {
    let active = routing::find_active_agent_session(conn, session_id, sender_channel_identity_id).await?;

    let routing_outcome = match active {
        Some(agent_session_id) => {
            let details = routing::load_active_agent_session_details(conn, agent_session_id).await?;
            if routing::is_stale(details.last_activity_at) {
                routing::mark_expired(conn, agent_session_id, session_id, &details.agent_type).await?;
                RoutingOutcome::NeedsClassification
            } else if details.agent_type == money_agent::MONEY_AGENT_TYPE {
                RoutingOutcome::Continue(agent_session_id)
            } else {
                RoutingOutcome::FallbackToChitchat
            }
        }
        None => RoutingOutcome::NeedsClassification,
    };

    match routing_outcome {
        RoutingOutcome::Continue(agent_session_id) => {
            run_subagent_turn(conn, provider, &money_agent::MoneyAgent, session_id, agent_session_id, user_id).await
        }
        RoutingOutcome::FallbackToChitchat => {
            chitchat::run_chitchat_turn(conn, mqtt, provider, embedding_provider, session_id, user_id, text).await
        }
        RoutingOutcome::NeedsClassification => match routing::classify_intent(provider, text).await {
            routing::Intent::Money => {
                let agent_session_id = routing::spawn_agent_session(
                    conn,
                    session_id,
                    sender_channel_identity_id,
                    money_agent::MONEY_AGENT_TYPE,
                )
                .await?;
                run_subagent_turn(conn, provider, &money_agent::MoneyAgent, session_id, agent_session_id, user_id).await
            }
            routing::Intent::Chitchat => {
                chitchat::run_chitchat_turn(conn, mqtt, provider, embedding_provider, session_id, user_id, text).await
            }
        },
    }
}

async fn run_subagent_turn(
    conn: &mut PoolConnection<Postgres>,
    provider: &dyn LlmProvider,
    agent: &dyn subagent::SubAgent,
    session_id: Uuid,
    agent_session_id: Uuid,
    user_id: Uuid,
) -> Result<String, TurnError> {
    let messages = fetch_recent_messages(conn, session_id).await?;

    let outcome = tools::run_tool_calling_loop(
        conn,
        provider,
        agent,
        session_id,
        agent_session_id,
        user_id,
        messages,
        SUBAGENT_MAX_TOKENS,
    )
    .await?;

    match outcome {
        tools::LoopOutcome::Reply(reply_text) => {
            let mut tx = conn.begin().await?;
            sqlx::query("INSERT INTO messages (session_id, sender_channel_identity_id, content) VALUES ($1, NULL, $2)")
                .bind(session_id)
                .bind(&reply_text)
                .execute(&mut *tx)
                .await?;
            sqlx::query("UPDATE agent_sessions SET last_activity_at = now() WHERE id = $1")
                .bind(agent_session_id)
                .execute(&mut *tx)
                .await?;
            tx.commit().await?;
            Ok(reply_text)
        }
        tools::LoopOutcome::Completed { status, summary } => {
            let mut tx = conn.begin().await?;
            sqlx::query("INSERT INTO messages (session_id, sender_channel_identity_id, content) VALUES ($1, NULL, $2)")
                .bind(session_id)
                .bind(&summary)
                .execute(&mut *tx)
                .await?;
            tx.commit().await?;

            routing::complete_agent_session(conn, agent_session_id, session_id, agent.agent_type(), &status, &summary).await?;

            Ok(summary)
        }
    }
}

async fn fetch_recent_messages(
    conn: &mut PoolConnection<Postgres>,
    session_id: Uuid,
) -> Result<Vec<LlmMessage>, TurnError> {
    let rows: Vec<(Option<Uuid>, String)> = sqlx::query_as(
        "SELECT sender_channel_identity_id, content FROM ( \
             SELECT sender_channel_identity_id, content, created_at FROM messages \
             WHERE session_id = $1 ORDER BY created_at DESC LIMIT $2 \
         ) recent ORDER BY created_at ASC",
    )
    .bind(session_id)
    .bind(SUBAGENT_HISTORY_LIMIT)
    .fetch_all(&mut **conn)
    .await?;

    Ok(rows
        .into_iter()
        .map(|(sender, content)| LlmMessage {
            role: if sender.is_some() { LlmRole::User } else { LlmRole::Assistant },
            content: vec![ContentBlock::Text { text: content }],
        })
        .collect())
}

async fn release_lock_ignoring_errors(conn: &mut sqlx::pool::PoolConnection<sqlx::Postgres>, session_id: uuid::Uuid) {
    let _ = lock::release_session_lock(conn, session_id).await;
}
