pub mod bootstrap;
pub mod ingest;
pub mod lock;
pub mod queue;
pub mod routing;
pub mod types;

pub use types::TurnOutcome;
pub use crate::agent_core::TurnError;

use sqlx::pool::PoolConnection;
use sqlx::{Acquire, PgPool, Postgres};
use uuid::Uuid;

use crate::agent_core::{self, SubAgent};
use crate::embedding::EmbeddingProvider;
use crate::llm::{ContentBlock, LlmMessage, LlmProvider, LlmRole};
use crate::realtime::{MqttPublisher, StreamEnvelope};

const SUBAGENT_HISTORY_LIMIT: i64 = 20;
const SUBAGENT_MAX_TOKENS: u32 = 1024;
const CHITCHAT_MAX_TOKENS: u32 = 1024;

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
            } else if details.agent_type == nomi_agent_money::MONEY_AGENT_TYPE {
                RoutingOutcome::Continue(agent_session_id)
            } else {
                RoutingOutcome::FallbackToChitchat
            }
        }
        None => RoutingOutcome::NeedsClassification,
    };

    match routing_outcome {
        RoutingOutcome::Continue(agent_session_id) => {
            run_subagent_turn(conn, provider, embedding_provider, &nomi_agent_money::MoneyAgent, session_id, agent_session_id, user_id).await
        }
        RoutingOutcome::FallbackToChitchat => {
            run_chitchat_turn(conn, mqtt, provider, embedding_provider, session_id, user_id).await
        }
        RoutingOutcome::NeedsClassification => match routing::classify_intent(provider, text).await {
            routing::Intent::Money => {
                let agent_session_id = routing::spawn_agent_session(
                    conn,
                    session_id,
                    sender_channel_identity_id,
                    nomi_agent_money::MONEY_AGENT_TYPE,
                )
                .await?;
                run_subagent_turn(conn, provider, embedding_provider, &nomi_agent_money::MoneyAgent, session_id, agent_session_id, user_id).await
            }
            routing::Intent::Chitchat => {
                run_chitchat_turn(conn, mqtt, provider, embedding_provider, session_id, user_id).await
            }
        },
    }
}

async fn run_subagent_turn(
    conn: &mut PoolConnection<Postgres>,
    provider: &dyn LlmProvider,
    embedding_provider: &dyn EmbeddingProvider,
    agent: &dyn SubAgent,
    session_id: Uuid,
    agent_session_id: Uuid,
    user_id: Uuid,
) -> Result<String, TurnError> {
    let messages = fetch_recent_messages(conn, session_id).await?;

    let outcome = agent_core::run_agent_turn(
        conn,
        None, // mqtt: money_agent-style turns don't stream yet at this point in the migration —
              // Task 9 threads `mqtt` through from `process_turn` once run_locked_turn itself
              // is rewritten. Passing None here preserves today's exact behavior (no streaming
              // for subagent turns) until that task deliberately changes it.
        provider,
        embedding_provider,
        agent,
        session_id,
        agent_session_id,
        user_id,
        messages,
        SUBAGENT_MAX_TOKENS,
    )
    .await?;

    match outcome {
        agent_core::LoopOutcome::Reply(reply_text) => {
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
        agent_core::LoopOutcome::Completed { status, summary } => {
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

/// Temporary bridge (Task 8): chitchat is now `nomi_agent_core::run_agent_turn` driven by
/// `nomi_agent_chitchat::ChitchatAgent` instead of its own bespoke history-fetch/memory/
/// streaming code. It has no `agent_session_id` of its own (it isn't a stateful multi-turn
/// agent session like money_agent) — `session_id` is passed for both `session_id` and
/// `agent_session_id`; `agent_events` rows keyed to a "session" rather than an "agent session"
/// for chitchat's tool-call logging is harmless since chitchat has no tools, so
/// `log_tool_call` is never actually invoked for it. This whole function is rough on purpose:
/// Task 9 replaces all of `run_locked_turn` (including this exact call site) with the real
/// registry-driven version.
async fn run_chitchat_turn(
    conn: &mut PoolConnection<Postgres>,
    mqtt: Option<(&MqttPublisher, Uuid)>,
    provider: &dyn LlmProvider,
    embedding_provider: &dyn EmbeddingProvider,
    session_id: Uuid,
    user_id: Uuid,
) -> Result<String, TurnError> {
    let messages = fetch_recent_messages(conn, session_id).await?;

    let outcome = agent_core::run_agent_turn(
        conn,
        mqtt,
        provider,
        embedding_provider,
        &nomi_agent_chitchat::ChitchatAgent,
        session_id,
        session_id,
        user_id,
        messages,
        CHITCHAT_MAX_TOKENS,
    )
    .await?;

    let reply_text = match outcome {
        agent_core::LoopOutcome::Reply(text) => text,
        agent_core::LoopOutcome::Completed { summary, .. } => summary, // chitchat never completes; unreachable in practice
    };

    sqlx::query("INSERT INTO messages (session_id, sender_channel_identity_id, content) VALUES ($1, NULL, $2)")
        .bind(session_id)
        .bind(&reply_text)
        .execute(&mut **conn)
        .await?;

    Ok(reply_text)
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
