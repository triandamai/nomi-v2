pub mod approval;
pub mod bootstrap;
pub mod ingest;
pub mod lock;
pub mod queue;
pub mod routing;

pub use nomi_agent_core::TurnError;

use sqlx::pool::PoolConnection;
use sqlx::{Acquire, PgPool, Postgres};
use uuid::Uuid;

use nomi_agent_core::AgentRegistry;
use nomi_embedding::EmbeddingProvider;
use nomi_llm::ContentBlock as LlmContentBlock;
use nomi_llm::{ContentBlock, LlmMessage, LlmProvider, LlmRole};
use nomi_realtime::{MqttPublisher, StreamEnvelope};

const SUBAGENT_HISTORY_LIMIT: i64 = 20;
const SUBAGENT_MAX_TOKENS: u32 = 1024;

#[derive(Debug, Clone, PartialEq)]
pub struct TurnOutcome {
    pub session_id: Uuid,
    pub reply: String,
    pub message_id: Option<Uuid>,
}

pub async fn handle_inbound_message(
    pool: &PgPool,
    provider: &dyn LlmProvider,
    embedding_provider: &dyn EmbeddingProvider,
    registry: &AgentRegistry,
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

    let result = run_locked_turn(&mut conn, None, provider, embedding_provider, registry, session_id, sender_channel_identity_id, user_id, text).await;

    match result {
        Ok((reply, message_id)) => {
            release_lock_ignoring_errors(&mut conn, session_id).await;
            Ok(TurnOutcome { session_id, reply, message_id })
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
/// message (see nomi_turn::ingest::ingest_inbound_message) — bootstrap and the inbound
/// message insert have already happened, so this only acquires the session lock and runs
/// routing/dispatch, threading `mqtt`/`turn_job_id` through for live delta publishing.
#[allow(clippy::too_many_arguments)]
pub async fn process_turn(
    pool: &PgPool,
    mqtt: &MqttPublisher,
    provider: &dyn LlmProvider,
    embedding_provider: &dyn EmbeddingProvider,
    registry: &AgentRegistry,
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
        registry,
        session_id,
        sender_channel_identity_id,
        user_id,
        text,
    )
    .await;

    match result {
        Ok((reply, message_id)) => {
            release_lock_ignoring_errors(&mut conn, session_id).await;
            Ok(TurnOutcome { session_id, reply, message_id })
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

/// Shared by handle_inbound_message and process_turn: routing/classification and agent
/// dispatch through `registry`, assuming the session lock is already held by the caller and
/// the inbound message has already been persisted (by the caller, before this runs).
#[allow(clippy::too_many_arguments)]
async fn run_locked_turn(
    conn: &mut PoolConnection<Postgres>,
    mqtt: Option<(&MqttPublisher, Uuid)>,
    provider: &dyn LlmProvider,
    embedding_provider: &dyn EmbeddingProvider,
    registry: &AgentRegistry,
    session_id: Uuid,
    sender_channel_identity_id: Uuid,
    user_id: Uuid,
    text: &str,
) -> Result<(String, Option<Uuid>), TurnError> {
    let active = routing::find_active_agent_session(conn, session_id, sender_channel_identity_id).await?;

    enum RoutingOutcome<'a> {
        Continue { agent: &'a dyn nomi_agent_core::SubAgent, agent_session_id: Uuid },
        NeedsClassification,
    }

    let routing_outcome = match active {
        Some(agent_session_id) => {
            let details = routing::load_active_agent_session_details(conn, agent_session_id).await?;
            if routing::is_stale(details.last_activity_at) {
                routing::mark_expired(conn, agent_session_id, session_id, &details.agent_type).await?;
                RoutingOutcome::NeedsClassification
            } else {
                match registry.find(&details.agent_type) {
                    Some(agent) => RoutingOutcome::Continue { agent, agent_session_id },
                    // The active session's agent_type isn't registered anymore (e.g. an
                    // agent crate was removed) — fall back to classifying fresh, same as
                    // an unrecognized/stale session.
                    None => RoutingOutcome::NeedsClassification,
                }
            }
        }
        None => RoutingOutcome::NeedsClassification,
    };

    match routing_outcome {
        RoutingOutcome::Continue { agent, agent_session_id } => {
            run_subagent_turn(conn, mqtt, provider, embedding_provider, registry, agent, session_id, agent_session_id, user_id).await
        }
        RoutingOutcome::NeedsClassification => {
            let agent = routing::classify_intent(provider, registry, text).await;

            if agent.agent_type() == registry.default_agent().agent_type() {
                // The default agent (chitchat) never gets a persistent agent_sessions row —
                // matching today's behavior, where chitchat has no agent_session_id at all.
                // agent_session_id == session_id here purely as a stand-in for logging
                // (see nomi-agent-chitchat's own comment on this at its call site's origin).
                run_subagent_turn(conn, mqtt, provider, embedding_provider, registry, agent, session_id, session_id, user_id).await
            } else {
                let agent_session_id =
                    routing::spawn_agent_session(conn, session_id, sender_channel_identity_id, agent.agent_type()).await?;
                run_subagent_turn(conn, mqtt, provider, embedding_provider, registry, agent, session_id, agent_session_id, user_id).await
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
async fn run_subagent_turn(
    conn: &mut PoolConnection<Postgres>,
    mqtt: Option<(&MqttPublisher, Uuid)>,
    provider: &dyn LlmProvider,
    embedding_provider: &dyn EmbeddingProvider,
    registry: &AgentRegistry,
    agent: &dyn nomi_agent_core::SubAgent,
    session_id: Uuid,
    agent_session_id: Uuid,
    user_id: Uuid,
) -> Result<(String, Option<Uuid>), TurnError> {
    let messages = fetch_recent_messages(conn, session_id).await?;

    let outcome = nomi_agent_core::run_agent_turn(
        conn, mqtt, provider, embedding_provider, registry, agent, session_id, agent_session_id, user_id, messages, SUBAGENT_MAX_TOKENS,
    )
    .await?;

    finish_agent_turn(conn, session_id, agent_session_id, agent, outcome).await
}

/// Persists a `LoopOutcome` (insert the final reply / completion message, record bookkeeping
/// events, update `last_activity_at`) and returns the reply text plus the persisted message's
/// id (`None` for `AwaitingApproval`, which already inserted its own approval message inside
/// `resolve_tool_batch` — nothing more to persist here). Shared by the fresh-turn path
/// (`run_subagent_turn`) and the resume path (`resume_paused_turn`) below, since both end up
/// with a `LoopOutcome` to finish the same way.
#[allow(clippy::too_many_arguments)]
async fn finish_agent_turn(
    conn: &mut PoolConnection<Postgres>,
    session_id: Uuid,
    agent_session_id: Uuid,
    agent: &dyn nomi_agent_core::SubAgent,
    outcome: nomi_agent_core::LoopOutcome,
) -> Result<(String, Option<Uuid>), TurnError> {
    match outcome {
        nomi_agent_core::LoopOutcome::Reply { text: reply_text, memory_ids_used, input_tokens, output_tokens } => {
            let mut tx = conn.begin().await?;
            let reply_message_id: Uuid = sqlx::query_scalar(
                "INSERT INTO messages (session_id, sender_channel_identity_id, content) VALUES ($1, NULL, $2) RETURNING id",
            )
            .bind(session_id)
            .bind(&reply_text)
            .fetch_one(&mut *tx)
            .await?;
            for memory_id in &memory_ids_used {
                sqlx::query("INSERT INTO message_memory_usage (message_id, memory_id) VALUES ($1, $2)")
                    .bind(reply_message_id)
                    .bind(memory_id)
                    .execute(&mut *tx)
                    .await?;
            }
            // The default agent (chitchat) has no real agent_sessions row — run_locked_turn
            // reuses session_id as agent_session_id for it as a sentinel (see its call site).
            // agent_events.agent_session_id has a foreign key into agent_sessions, so binding
            // that sentinel directly would fail every default-agent reply; NULL it out instead.
            let agent_session_id_for_event = if agent_session_id == session_id { None } else { Some(agent_session_id) };
            sqlx::query(
                "INSERT INTO agent_events (session_id, agent_session_id, agent_type, event_type, payload) VALUES ($1, $2, $3, 'AgentReplied', $4)",
            )
            .bind(session_id)
            .bind(agent_session_id_for_event)
            .bind(agent.agent_type())
            .bind(serde_json::json!({"input_tokens": input_tokens, "output_tokens": output_tokens}))
            .execute(&mut *tx)
            .await?;
            sqlx::query("UPDATE agent_sessions SET last_activity_at = now() WHERE id = $1")
                .bind(agent_session_id)
                .execute(&mut *tx)
                .await?;
            tx.commit().await?;
            Ok((reply_text, Some(reply_message_id)))
        }
        nomi_agent_core::LoopOutcome::Completed { status, summary } => {
            let mut tx = conn.begin().await?;
            let message_id: Uuid = sqlx::query_scalar(
                "INSERT INTO messages (session_id, sender_channel_identity_id, content) VALUES ($1, NULL, $2) RETURNING id",
            )
            .bind(session_id)
            .bind(&summary)
            .fetch_one(&mut *tx)
            .await?;
            tx.commit().await?;

            routing::complete_agent_session(conn, agent_session_id, session_id, agent.agent_type(), &status, &summary).await?;

            Ok((summary, Some(message_id)))
        }
        nomi_agent_core::LoopOutcome::AwaitingApproval { .. } => Ok(("Waiting for approval.".to_string(), None)),
    }
}

#[allow(clippy::too_many_arguments)]
pub async fn resume_paused_turn(
    pool: &PgPool,
    mqtt: &MqttPublisher,
    provider: &dyn LlmProvider,
    embedding_provider: &dyn EmbeddingProvider,
    registry: &AgentRegistry,
    message_id: Uuid,
    decision: &str,
    remember: bool,
) -> Result<TurnOutcome, TurnError> {
    let session_id: Uuid = sqlx::query_scalar("SELECT session_id FROM messages WHERE id = $1")
        .bind(message_id)
        .fetch_one(pool)
        .await?;

    let mut conn = lock::acquire_session_lock(pool, session_id).await?;

    let result = resume_locked(&mut conn, mqtt, provider, embedding_provider, registry, session_id, message_id, decision, remember).await;

    match result {
        Ok((reply, resumed_message_id)) => {
            release_lock_ignoring_errors(&mut conn, session_id).await;
            Ok(TurnOutcome { session_id, reply, message_id: resumed_message_id })
        }
        Err(err) => {
            let _ = sqlx::query("INSERT INTO agent_events (session_id, event_type, payload) VALUES ($1, 'TurnFailed', $2)")
                .bind(session_id)
                .bind(serde_json::json!({"error": err.to_string()}))
                .execute(&mut *conn)
                .await;
            let _ = mqtt.publish(session_id, &StreamEnvelope::TurnFailed { turn_job_id: Uuid::nil(), error: err.to_string() }).await;
            release_lock_ignoring_errors(&mut conn, session_id).await;
            Err(err)
        }
    }
}

#[allow(clippy::too_many_arguments)]
async fn resume_locked(
    conn: &mut PoolConnection<Postgres>,
    mqtt: &MqttPublisher,
    provider: &dyn LlmProvider,
    embedding_provider: &dyn EmbeddingProvider,
    registry: &AgentRegistry,
    session_id: Uuid,
    message_id: Uuid,
    decision: &str,
    remember: bool,
) -> Result<(String, Option<Uuid>), TurnError> {
    let row: Option<(Uuid, String, serde_json::Value, Uuid)> = sqlx::query_as(
        "SELECT id, agent_type, state, sender_channel_identity_id FROM agent_sessions \
         WHERE state->>'pending_approval_message_id' = $1 AND (state->>'paused_for_approval')::boolean = true",
    )
    .bind(message_id.to_string())
    .fetch_optional(&mut **conn)
    .await?;

    let Some((agent_session_id, agent_type, state, sender_channel_identity_id)) = row else {
        return Err(TurnError::ApprovalNoLongerPending);
    };

    let user_id: Uuid = sqlx::query_scalar("SELECT user_id FROM channel_identities WHERE id = $1")
        .bind(sender_channel_identity_id)
        .fetch_one(&mut **conn)
        .await?;

    let agent = registry.find(&agent_type).ok_or(TurnError::ApprovalNoLongerPending)?;

    let tool_use_blocks: Vec<LlmContentBlock> =
        serde_json::from_value(state["tool_use_blocks"].clone()).map_err(|_| TurnError::ApprovalNoLongerPending)?;
    let messages: Vec<LlmMessage> =
        serde_json::from_value(state["messages"].clone()).map_err(|_| TurnError::ApprovalNoLongerPending)?;
    let pending_tool_use_id = state["pending_tool_use_id"].as_str().unwrap_or_default().to_string();

    if remember {
        if let Some(LlmContentBlock::ToolUse { name, input, .. }) =
            tool_use_blocks.iter().find(|b| matches!(b, LlmContentBlock::ToolUse { id, .. } if *id == pending_tool_use_id))
        {
            let path = input.get("path").and_then(|v| v.as_str());
            let rule_decision = if decision == "approve" { "allow" } else { "deny" };
            let _ = nomi_agent_core::permissions::remember_decision(conn, user_id, name, path, rule_decision).await;
        }
    }

    // Clear the paused-state keys before resolving — resolving may pause again on a different
    // block in the same batch, in which case it writes fresh paused keys right back.
    let _ = sqlx::query(
        "UPDATE agent_sessions SET state = state - 'paused_for_approval' - 'pending_approval_message_id' - 'pending_tool_use_id' - 'tool_use_blocks' - 'messages' WHERE id = $1",
    )
    .bind(agent_session_id)
    .execute(&mut **conn)
    .await?;

    let batch_outcome = nomi_agent_core::resolve_tool_batch(
        conn,
        Some((mqtt, Uuid::nil())),
        registry,
        agent,
        session_id,
        agent_session_id,
        user_id,
        &tool_use_blocks,
        &messages,
        Some((pending_tool_use_id.as_str(), decision == "approve")),
    )
    .await?;

    match batch_outcome {
        nomi_agent_core::ToolBatchOutcome::AwaitingApproval { .. } => Ok(("Waiting for another approval.".to_string(), None)),
        nomi_agent_core::ToolBatchOutcome::Completed { status, summary } => {
            finish_agent_turn(conn, session_id, agent_session_id, agent, nomi_agent_core::LoopOutcome::Completed { status, summary }).await
        }
        nomi_agent_core::ToolBatchOutcome::Resolved(tool_results) => {
            let mut full_messages = messages;
            full_messages.push(LlmMessage { role: LlmRole::User, content: tool_results });

            let outcome = nomi_agent_core::run_agent_turn(
                conn, Some((mqtt, Uuid::nil())), provider, embedding_provider, registry, agent, session_id, agent_session_id, user_id,
                full_messages, SUBAGENT_MAX_TOKENS,
            )
            .await?;

            finish_agent_turn(conn, session_id, agent_session_id, agent, outcome).await
        }
    }
}

async fn fetch_recent_messages(
    conn: &mut PoolConnection<Postgres>,
    session_id: Uuid,
) -> Result<Vec<LlmMessage>, TurnError> {
    let rows: Vec<(Option<Uuid>, String, Uuid)> = sqlx::query_as(
        "SELECT sender_channel_identity_id, content, id FROM ( \
             SELECT id, sender_channel_identity_id, content, created_at FROM messages \
             WHERE session_id = $1 ORDER BY created_at DESC, id DESC LIMIT $2 \
         ) recent ORDER BY created_at ASC, id ASC",
    )
    .bind(session_id)
    .bind(SUBAGENT_HISTORY_LIMIT)
    .fetch_all(&mut **conn)
    .await?;

    Ok(rows
        .into_iter()
        .map(|(sender, content, _id)| LlmMessage {
            role: if sender.is_some() { LlmRole::User } else { LlmRole::Assistant },
            content: vec![ContentBlock::Text { text: content }],
        })
        .collect())
}

async fn release_lock_ignoring_errors(conn: &mut sqlx::pool::PoolConnection<sqlx::Postgres>, session_id: uuid::Uuid) {
    let _ = lock::release_session_lock(conn, session_id).await;
}
