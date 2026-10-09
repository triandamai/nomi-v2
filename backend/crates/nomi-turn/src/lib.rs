pub mod approval;
pub mod bootstrap;
pub mod ingest;
pub mod lock;
pub mod queue;
pub mod routing;

pub use nomi_agent_core::TurnError;

use std::sync::Arc;

use sqlx::pool::PoolConnection;
use sqlx::{Acquire, PgPool, Postgres};
use uuid::Uuid;

use nomi_agent_core::AgentRegistry;
use nomi_embedding::EmbeddingProvider;
use nomi_llm::ContentBlock as LlmContentBlock;
use nomi_llm::{ContentBlock, LlmMessage, LlmProvider, LlmRole};
use nomi_realtime::{MqttPublisher, StreamEnvelope};

/// A chat's latest messages, sent in full; older ones reach agents as the chat's running summary.
const SUBAGENT_HISTORY_LIMIT: i64 = nomi_agent_core::working_memory::RECENT_MESSAGES;
/// Room for an agent's reply (thinking gets its own budget on top). A ceiling, not a target:
/// plans, drafts and tables need far more than a chat line.
const SUBAGENT_MAX_TOKENS: u32 = 8192;

#[derive(Debug, Clone, PartialEq)]
pub struct TurnOutcome {
    pub session_id: Uuid,
    pub reply: String,
    pub message_id: Option<Uuid>,
}

pub async fn handle_inbound_message(
    pool: &PgPool,
    s3: Option<&nomi_storage::S3Config>,
    provider: &dyn LlmProvider,
    embedding_provider: &dyn EmbeddingProvider,
    registry: &AgentRegistry,
    catalog: &Arc<nomi_agent_core::ToolCatalog>,
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

    let result = run_locked_turn(&mut conn, None, s3, provider, embedding_provider, registry, catalog, session_id, sender_channel_identity_id, user_id, text).await;

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
            settle_agent_phases(&mut conn, None, session_id).await;

            release_lock_ignoring_errors(&mut conn, session_id).await;
            Err(err)
        }
    }
}

/// Longest one turn may run (`TURN_TIME_LIMIT_SECS`, default 10 minutes). Long enough for a
/// many-step task on a slow model; short enough that a stuck one doesn't hold the queue.
pub fn turn_time_limit() -> std::time::Duration {
    let secs = std::env::var("TURN_TIME_LIMIT_SECS").ok().and_then(|v| v.trim().parse::<u64>().ok()).filter(|s| *s > 0).unwrap_or(600);
    std::time::Duration::from_secs(secs)
}

/// The worker's entry point (see backend/src/bin/worker.rs): processes an already-ingested
/// message (see nomi_turn::ingest::ingest_inbound_message) — bootstrap and the inbound
/// message insert have already happened, so this only acquires the session lock and runs
/// routing/dispatch, threading `mqtt`/`turn_job_id` through for live delta publishing.
#[allow(clippy::too_many_arguments)]
pub async fn process_turn(
    pool: &PgPool,
    mqtt: &MqttPublisher,
    s3: Option<&nomi_storage::S3Config>,
    provider: &dyn LlmProvider,
    embedding_provider: &dyn EmbeddingProvider,
    registry: &AgentRegistry,
    catalog: &Arc<nomi_agent_core::ToolCatalog>,
    turn_job_id: Uuid,
    session_id: Uuid,
    sender_channel_identity_id: Uuid,
    user_id: Uuid,
    text: &str,
) -> Result<TurnOutcome, TurnError> {
    let mut conn = lock::acquire_session_lock(pool, session_id).await?;

    // A stalled model or tool must never hold the chat (or the worker, which runs turns one at a
    // time) forever: past the limit the turn is dropped and fails like any other error below,
    // which also releases the session lock on this same connection.
    let limit = turn_time_limit();
    let result = match tokio::time::timeout(
        limit,
        run_locked_turn(
            &mut conn,
            Some((mqtt, turn_job_id)),
            s3,
            provider,
            embedding_provider,
            registry,
            catalog,
            session_id,
            sender_channel_identity_id,
            user_id,
            text,
        ),
    )
    .await
    {
        Ok(result) => result,
        Err(_) => Err(TurnError::TimedOut(limit)),
    };

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

            // Whatever was mid-step when it failed is idle now, or the chat and crew would show it
            // working forever.
            settle_agent_phases(&mut conn, Some(mqtt), session_id).await;
            // Best-effort: an MQTT publish failure never changes the turn's outcome.
            let explanation = post_failure_notice(&mut conn, mqtt, session_id, &err).await;
            let _ = mqtt
                .publish(session_id, &StreamEnvelope::TurnFailed { turn_job_id, error: explanation })
                .await;

            release_lock_ignoring_errors(&mut conn, session_id).await;
            Err(err)
        }
    }
}

/// Resolves `agent_type` to a live `Arc<dyn SubAgent>` — a built-in agent from `registry` if
/// one matches, otherwise a `DynamicAgent` freshly built from the `dynamic_agents` table (its
/// row id, stringified, is what a dynamic agent's own `agent_type()` returns — see
/// `DynamicAgent::agent_type`). `None` when neither matches, e.g. a stale `agent_sessions` row
/// left over from a dynamic agent that's since been deleted... there is no delete in v1, so in
/// practice this only happens for a genuinely unrecognized `agent_type` string.
async fn resolve_agent(
    conn: &mut PoolConnection<Postgres>,
    registry: &AgentRegistry,
    catalog: &Arc<nomi_agent_core::ToolCatalog>,
    agent_type: &str,
) -> Option<Arc<dyn nomi_agent_core::SubAgent>> {
    if let Some(agent) = registry.find(agent_type) {
        return Some(agent);
    }
    let id: Uuid = agent_type.parse().ok()?;
    let row = nomi_agent_core::dynamic_agent::find_dynamic_agent_by_id(conn, id).await.ok()??;
    Some(Arc::new(nomi_agent_core::DynamicAgent::from_row(row, catalog.clone())) as Arc<dyn nomi_agent_core::SubAgent>)
}

/// Shared by handle_inbound_message and process_turn: routing/classification and agent
/// dispatch through `registry`, assuming the session lock is already held by the caller and
/// the inbound message has already been persisted (by the caller, before this runs).
#[allow(clippy::too_many_arguments)]
async fn run_locked_turn(
    conn: &mut PoolConnection<Postgres>,
    mqtt: Option<(&MqttPublisher, Uuid)>,
    s3: Option<&nomi_storage::S3Config>,
    provider: &dyn LlmProvider,
    embedding_provider: &dyn EmbeddingProvider,
    registry: &AgentRegistry,
    catalog: &Arc<nomi_agent_core::ToolCatalog>,
    session_id: Uuid,
    sender_channel_identity_id: Uuid,
    user_id: Uuid,
    text: &str,
) -> Result<(String, Option<Uuid>), TurnError> {
    // Attachments (files, voice notes) always go to the Files agent first: it reads them and
    // decides whether to answer itself or hand parts to Money, Reminders, Planning or Coding.
    // It runs without an agent session of its own, so an ongoing conversation with another agent
    // carries on afterwards.
    if nomi_agent_core::attachments::has_attachments(text) {
        if let Some(files) = registry.find(nomi_agent_core::attachments::FILES_AGENT_TYPE) {
            return run_subagent_turn(conn, mqtt, s3, provider, embedding_provider, registry, catalog, files, session_id, session_id, sender_channel_identity_id, user_id).await;
        }
    }

    let active = routing::find_active_agent_session(conn, session_id, sender_channel_identity_id).await?;

    enum RoutingOutcome {
        Continue { agent: Arc<dyn nomi_agent_core::SubAgent>, agent_session_id: Uuid },
        NeedsClassification,
    }

    let routing_outcome = match active {
        Some(agent_session_id) => {
            let details = routing::load_active_agent_session_details(conn, agent_session_id).await?;
            if routing::is_stale(details.last_activity_at) {
                routing::mark_expired(conn, mqtt, agent_session_id, session_id, &details.agent_type).await?;
                RoutingOutcome::NeedsClassification
            } else {
                match resolve_agent(conn, registry, catalog, &details.agent_type).await {
                    // The active session's agent_type isn't registered anymore (e.g. an
                    // agent crate was removed, or a dynamic agent row was somehow deleted) —
                    // fall back to classifying fresh, same as an unrecognized/stale session.
                    Some(agent) => RoutingOutcome::Continue { agent, agent_session_id },
                    None => RoutingOutcome::NeedsClassification,
                }
            }
        }
        None => RoutingOutcome::NeedsClassification,
    };

    match routing_outcome {
        RoutingOutcome::Continue { agent, agent_session_id } => {
            run_subagent_turn(conn, mqtt, s3, provider, embedding_provider, registry, catalog, agent, session_id, agent_session_id, sender_channel_identity_id, user_id).await
        }
        RoutingOutcome::NeedsClassification => {
            let agent = routing::classify_intent(conn, provider, registry, catalog, text).await;

            if agent.agent_type() == registry.default_agent().agent_type() {
                // The default agent (chitchat) never gets a persistent agent_sessions row —
                // matching today's behavior, where chitchat has no agent_session_id at all.
                // agent_session_id == session_id here purely as a stand-in for logging
                // (see nomi-agent-chitchat's own comment on this at its call site's origin).
                run_subagent_turn(conn, mqtt, s3, provider, embedding_provider, registry, catalog, agent, session_id, session_id, sender_channel_identity_id, user_id).await
            } else {
                let agent_session_id =
                    routing::spawn_agent_session(conn, mqtt, session_id, sender_channel_identity_id, agent.agent_type().as_ref(), agent.display_name().as_ref()).await?;
                run_subagent_turn(conn, mqtt, s3, provider, embedding_provider, registry, catalog, agent, session_id, agent_session_id, sender_channel_identity_id, user_id).await
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
async fn run_subagent_turn(
    conn: &mut PoolConnection<Postgres>,
    mqtt: Option<(&MqttPublisher, Uuid)>,
    s3: Option<&nomi_storage::S3Config>,
    provider: &dyn LlmProvider,
    embedding_provider: &dyn EmbeddingProvider,
    registry: &AgentRegistry,
    catalog: &Arc<nomi_agent_core::ToolCatalog>,
    agent: Arc<dyn nomi_agent_core::SubAgent>,
    session_id: Uuid,
    agent_session_id: Uuid,
    sender_channel_identity_id: Uuid,
    user_id: Uuid,
) -> Result<(String, Option<Uuid>), TurnError> {
    let mut messages = fetch_recent_messages(conn, session_id).await?;
    nomi_agent_core::attachments::expand_messages(conn, user_id, &mut messages).await;

    let outcome = nomi_agent_core::run_agent_turn(
        conn, mqtt, s3, provider, embedding_provider, registry, agent.as_ref(), session_id, agent_session_id, user_id, messages, SUBAGENT_MAX_TOKENS,
    )
    .await?;

    let handoff = HandOffContext { provider, embedding_provider, registry, catalog, s3, sender_channel_identity_id, user_id };
    follow_hand_offs(conn, mqtt, &handoff, session_id, agent, agent_session_id, outcome).await
}

/// What running a handed-off specialist needs, beyond the chat it runs in.
struct HandOffContext<'a> {
    provider: &'a dyn LlmProvider,
    embedding_provider: &'a dyn EmbeddingProvider,
    registry: &'a AgentRegistry,
    catalog: &'a Arc<nomi_agent_core::ToolCatalog>,
    s3: Option<&'a nomi_storage::S3Config>,
    sender_channel_identity_id: Uuid,
    user_id: Uuid,
}

/// How many times one turn may pass the conversation on. A hand-off past this is queued as a
/// background delegation instead, so agents can't bounce a request between them forever.
const MAX_HAND_OFFS: usize = 2;

/// When an agent hands the conversation to a specialist, the specialist runs now, in this turn,
/// with the conversation so far, and its answer is the turn's reply: one request, one answer, in
/// the voice of whoever did the work. The specialist keeps the conversation for follow-ups.
#[allow(clippy::too_many_arguments)]
async fn follow_hand_offs(
    conn: &mut PoolConnection<Postgres>,
    mqtt: Option<(&MqttPublisher, Uuid)>,
    ctx: &HandOffContext<'_>,
    session_id: Uuid,
    mut agent: Arc<dyn nomi_agent_core::SubAgent>,
    mut agent_session_id: Uuid,
    mut outcome: nomi_agent_core::LoopOutcome,
) -> Result<(String, Option<Uuid>), TurnError> {
    let mut hand_offs = 0;
    while let nomi_agent_core::LoopOutcome::HandOff { target_agent, task } = outcome.clone() {
        let target = if hand_offs < MAX_HAND_OFFS { resolve_agent(conn, ctx.registry, ctx.catalog, &target_agent).await } else { None };
        let Some(target) = target else {
            let locale = nomi_agent_core::user_locale(conn, ctx.user_id).await;
            let ack = nomi_agent_core::delegation::create_delegation(conn, mqtt, session_id, agent.agent_type().as_ref(), &target_agent, &task, ctx.user_id)
                .await
                .map(|_| locale.tf("turn.handed_off", &[("agent", &capitalize(&target_agent))]))
                .unwrap_or_else(|_| locale.t("engine.no_answer"));
            outcome = nomi_agent_core::LoopOutcome::Reply { text: ack, memory_ids_used: vec![], input_tokens: 0, output_tokens: 0 };
            break;
        };

        // The agent that handed off is done with this conversation; the default agent has no
        // session of its own to close.
        if agent_session_id != session_id {
            routing::complete_agent_session(conn, mqtt, agent_session_id, session_id, agent.agent_type().as_ref(), "completed", &format!("Handed to {target_agent}"))
                .await?;
        }
        agent_session_id = if target.is_default() {
            session_id
        } else {
            routing::spawn_agent_session(conn, mqtt, session_id, ctx.sender_channel_identity_id, target.agent_type().as_ref(), target.display_name().as_ref()).await?
        };

        let mut messages = fetch_recent_messages(conn, session_id).await?;
        nomi_agent_core::attachments::expand_messages(conn, ctx.user_id, &mut messages).await;
        add_hand_off_note(&mut messages, agent.display_name().as_ref(), &task);
        outcome = nomi_agent_core::run_agent_turn(
            conn,
            mqtt,
            ctx.s3,
            ctx.provider,
            ctx.embedding_provider,
            ctx.registry,
            target.as_ref(),
            session_id,
            agent_session_id,
            ctx.user_id,
            messages,
            SUBAGENT_MAX_TOKENS,
        )
        .await?;
        agent = target;
        hand_offs += 1;
    }

    finish_agent_turn(conn, mqtt, session_id, agent_session_id, agent.as_ref(), outcome).await
}

/// Tells the specialist what it was handed, on the user's latest message.
fn add_hand_off_note(messages: &mut Vec<LlmMessage>, from: &str, task: &str) {
    let note = format!("({from} handed this to you: {task})");
    match messages.last_mut() {
        Some(last) if last.role == LlmRole::User => last.content.push(ContentBlock::Text { text: note }),
        _ => messages.push(LlmMessage { role: LlmRole::User, content: vec![ContentBlock::Text { text: note }] }),
    }
}

fn capitalize(agent_type: &str) -> String {
    let mut chars = agent_type.chars();
    chars.next().map(|first| first.to_uppercase().chain(chars).collect()).unwrap_or_default()
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
    mqtt: Option<(&MqttPublisher, Uuid)>,
    session_id: Uuid,
    agent_session_id: Uuid,
    agent: &dyn nomi_agent_core::SubAgent,
    outcome: nomi_agent_core::LoopOutcome,
) -> Result<(String, Option<Uuid>), TurnError> {
    match outcome {
        nomi_agent_core::LoopOutcome::Reply { text: reply_text, memory_ids_used, input_tokens, output_tokens } => {
            let mut tx = conn.begin().await?;
            let reply_message_id: Uuid = sqlx::query_scalar(
                "INSERT INTO messages (session_id, sender_channel_identity_id, content, agent_display_name) VALUES ($1, NULL, $2, $3) RETURNING id",
            )
            .bind(session_id)
            .bind(&reply_text)
            .bind(agent.display_name().as_ref())
            .fetch_one(&mut *tx)
            .await?;
            for memory_id in &memory_ids_used {
                sqlx::query("INSERT INTO message_memory_usage (message_id, memory_id) VALUES ($1, $2)")
                    .bind(reply_message_id)
                    .bind(memory_id)
                    .execute(&mut *tx)
                    .await?;
            }
            nomi_agent_core::memory::mark_used(&mut *tx, &memory_ids_used).await?;
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
            .bind(agent.agent_type().as_ref())
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
                "INSERT INTO messages (session_id, sender_channel_identity_id, content, agent_display_name) VALUES ($1, NULL, $2, $3) RETURNING id",
            )
            .bind(session_id)
            .bind(&summary)
            .bind(agent.display_name().as_ref())
            .fetch_one(&mut *tx)
            .await?;
            tx.commit().await?;

            routing::complete_agent_session(conn, mqtt, agent_session_id, session_id, agent.agent_type().as_ref(), &status, &summary).await?;

            Ok((summary, Some(message_id)))
        }
        nomi_agent_core::LoopOutcome::AwaitingApproval { .. } => Ok(("Waiting for approval.".to_string(), None)),
        // follow_hand_offs runs the specialist before anything is finished.
        nomi_agent_core::LoopOutcome::HandOff { .. } => Ok((String::new(), None)),
        // Stopped by the user mid-turn: the supervisor already replied, so nothing is posted.
        nomi_agent_core::LoopOutcome::Cancelled => Ok((String::new(), None)),
    }
}

#[allow(clippy::too_many_arguments)]
pub async fn resume_paused_turn(
    pool: &PgPool,
    mqtt: &MqttPublisher,
    s3: Option<&nomi_storage::S3Config>,
    provider: &dyn LlmProvider,
    embedding_provider: &dyn EmbeddingProvider,
    registry: &AgentRegistry,
    catalog: &Arc<nomi_agent_core::ToolCatalog>,
    message_id: Uuid,
    decision: &str,
    remember: bool,
) -> Result<TurnOutcome, TurnError> {
    let session_id: Uuid = sqlx::query_scalar("SELECT session_id FROM messages WHERE id = $1")
        .bind(message_id)
        .fetch_one(pool)
        .await?;

    let mut conn = lock::acquire_session_lock(pool, session_id).await?;

    let result = resume_locked(&mut conn, mqtt, s3, provider, embedding_provider, registry, catalog, session_id, message_id, decision, remember).await;

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
            settle_agent_phases(&mut conn, Some(mqtt), session_id).await;
            let explanation = post_failure_notice(&mut conn, mqtt, session_id, &err).await;
            let _ = mqtt.publish(session_id, &StreamEnvelope::TurnFailed { turn_job_id: Uuid::nil(), error: explanation }).await;
            release_lock_ignoring_errors(&mut conn, session_id).await;
            Err(err)
        }
    }
}

#[allow(clippy::too_many_arguments)]
async fn resume_locked(
    conn: &mut PoolConnection<Postgres>,
    mqtt: &MqttPublisher,
    s3: Option<&nomi_storage::S3Config>,
    provider: &dyn LlmProvider,
    embedding_provider: &dyn EmbeddingProvider,
    registry: &AgentRegistry,
    catalog: &Arc<nomi_agent_core::ToolCatalog>,
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

    let agent = resolve_agent(conn, registry, catalog, &agent_type).await.ok_or(TurnError::ApprovalNoLongerPending)?;

    let tool_use_blocks: Vec<LlmContentBlock> =
        serde_json::from_value(state["tool_use_blocks"].clone()).map_err(|_| TurnError::ApprovalNoLongerPending)?;
    let messages: Vec<LlmMessage> =
        serde_json::from_value(state["messages"].clone()).map_err(|_| TurnError::ApprovalNoLongerPending)?;
    let pending_tool_use_id = state["pending_tool_use_id"].as_str().unwrap_or_default().to_string();
    let mut decided_tool_use_ids: Vec<(String, bool)> =
        serde_json::from_value(state["decided_tool_use_ids"].clone()).unwrap_or_default();

    if remember {
        if let Some(LlmContentBlock::ToolUse { name, input, .. }) =
            tool_use_blocks.iter().find(|b| matches!(b, LlmContentBlock::ToolUse { id, .. } if *id == pending_tool_use_id))
        {
            let path = input.get("path").and_then(|v| v.as_str());
            let rule_decision = if decision == "approve" { "allow" } else { "deny" };
            let _ = nomi_agent_core::permissions::remember_decision(conn, user_id, name, path, rule_decision).await;
        }
    }

    // Reused for identical calls later in this chat (check_tool_permission_in_session), so a
    // repeated or retried call doesn't ask again.
    if let Some(LlmContentBlock::ToolUse { name, input, .. }) =
        tool_use_blocks.iter().find(|b| matches!(b, LlmContentBlock::ToolUse { id, .. } if *id == pending_tool_use_id))
    {
        nomi_agent_core::permissions::record_session_decision(conn, session_id, user_id, name, input, decision == "approve").await?;
    }

    let new_status = if decision == "approve" { "approved" } else { "denied" };
    let _ = sqlx::query(
        "UPDATE messages SET content_blocks = jsonb_set(jsonb_set(content_blocks, '{0,status}', to_jsonb($1::text)), '{0,decided_at}', to_jsonb(now())) WHERE id = $2",
    )
    .bind(new_status)
    .bind(message_id)
    .execute(&mut **conn)
    .await;
    let _ = mqtt.publish(session_id, &StreamEnvelope::MessageUpdated { message_id }).await;

    // Clear the paused-state keys before resolving — resolving may pause again on a different
    // block in the same batch, in which case it writes fresh paused keys right back.
    let _ = sqlx::query(
        "UPDATE agent_sessions SET state = state - 'paused_for_approval' - 'pending_approval_message_id' - 'pending_tool_use_id' - 'tool_use_blocks' - 'messages' - 'decided_tool_use_ids' WHERE id = $1",
    )
    .bind(agent_session_id)
    .execute(&mut **conn)
    .await?;

    // Record the decision being resolved by THIS call, appended to the accumulator carried across
    // earlier resumes of the same batch — so every previously-decided block stays decided on the
    // next pre-scan pass and the batch converges instead of ping-ponging between cards.
    decided_tool_use_ids.push((pending_tool_use_id.clone(), decision == "approve"));

    let batch_outcome = nomi_agent_core::resolve_tool_batch(
        conn,
        Some((mqtt, Uuid::nil())),
        s3,
        registry,
        agent.as_ref(),
        session_id,
        agent_session_id,
        user_id,
        &tool_use_blocks,
        &messages,
        &decided_tool_use_ids,
    )
    .await?;

    let outcome = match batch_outcome {
        nomi_agent_core::ToolBatchOutcome::AwaitingApproval { .. } => return Ok(("Waiting for another approval.".to_string(), None)),
        nomi_agent_core::ToolBatchOutcome::Completed { status, summary } => nomi_agent_core::LoopOutcome::Completed { status, summary },
        nomi_agent_core::ToolBatchOutcome::HandOff { target_agent, task } => nomi_agent_core::LoopOutcome::HandOff { target_agent, task },
        nomi_agent_core::ToolBatchOutcome::Resolved(tool_results) => {
            let mut full_messages = messages;
            full_messages.push(LlmMessage { role: LlmRole::User, content: tool_results });

            nomi_agent_core::run_agent_turn(
                conn, Some((mqtt, Uuid::nil())), s3, provider, embedding_provider, registry, agent.as_ref(), session_id, agent_session_id, user_id,
                full_messages, SUBAGENT_MAX_TOKENS,
            )
            .await?
        }
    };
    if matches!(outcome, nomi_agent_core::LoopOutcome::AwaitingApproval { .. } | nomi_agent_core::LoopOutcome::Cancelled) {
        return finish_agent_turn(conn, Some((mqtt, Uuid::nil())), session_id, agent_session_id, agent.as_ref(), outcome).await;
    }

    let handoff = HandOffContext { provider, embedding_provider, registry, catalog, s3, sender_channel_identity_id, user_id };
    let (reply, reply_message_id) = follow_hand_offs(conn, Some((mqtt, Uuid::nil())), &handoff, session_id, agent, agent_session_id, outcome).await?;

    // A row the engine created only to hold this pause (default agent / delegated turn — see
    // resolve_tool_batch) has served its purpose once the turn finishes.
    sqlx::query("UPDATE agent_sessions SET status = 'completed', ended_at = now() WHERE id = $1 AND status = 'awaiting_approval'")
        .bind(agent_session_id)
        .execute(&mut **conn)
        .await?;

    // A delegated turn that paused for approval left its delegation 'processing' (the delegation
    // worker tags the paused state with its id); now that it has finished, close it out.
    if let Some(delegation_id) = state["delegation_id"].as_str().and_then(|id| id.parse::<Uuid>().ok()) {
        sqlx::query("UPDATE agent_delegations SET status = 'completed', completed_at = now(), result = $2 WHERE id = $1")
            .bind(delegation_id)
            .bind(&reply)
            .execute(&mut **conn)
            .await?;
        let _ = mqtt.publish(session_id, &StreamEnvelope::AgentDelegationUpdated { delegation_id }).await;
    }

    Ok((reply, reply_message_id))
}

async fn fetch_recent_messages(
    conn: &mut PoolConnection<Postgres>,
    session_id: Uuid,
) -> Result<Vec<LlmMessage>, TurnError> {
    // Only what was actually said: shown thinking (🧠) and progress notes (💭) are a window
    // into the work, not replies, and fed back as replies they confuse the next agent.
    let rows: Vec<(Option<Uuid>, String, Uuid)> = sqlx::query_as(&format!(
        "SELECT sender_channel_identity_id, content, id FROM ( \
             SELECT id, sender_channel_identity_id, content, created_at FROM messages \
             WHERE session_id = $1 AND {} \
             ORDER BY created_at DESC, id DESC LIMIT $2 \
         ) recent ORDER BY created_at ASC, id ASC",
        nomi_agent_core::working_memory::SAID_IN_CHAT
    ))
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

/// The language of the person whose chat this is (English if it has no owner).
async fn session_owner_locale(conn: &mut PoolConnection<Postgres>, session_id: Uuid) -> nomi_agent_core::Locale {
    let owner: Option<Uuid> = sqlx::query_scalar("SELECT user_id FROM sessions WHERE id = $1")
        .bind(session_id)
        .fetch_optional(&mut **conn)
        .await
        .ok()
        .flatten();
    match owner {
        Some(user_id) => nomi_agent_core::user_locale(conn, user_id).await,
        None => nomi_agent_core::Locale::En,
    }
}

async fn release_lock_ignoring_errors(conn: &mut sqlx::pool::PoolConnection<sqlx::Postgres>, session_id: uuid::Uuid) {
    let _ = lock::release_session_lock(conn, session_id).await;
}

/// Tells the user in the chat why their message got no reply (out of credits, a rejected API
/// key, ...) and returns that explanation. Best-effort, like the rest of the failure path.
/// After a turn fails: every agent in the chat that was thinking, writing or calling a tool is
/// set back to waiting (and the chat is told), since the failure ended whatever it was doing.
async fn settle_agent_phases(conn: &mut PoolConnection<Postgres>, mqtt: Option<&MqttPublisher>, session_id: Uuid) {
    let settled: Vec<Uuid> = sqlx::query_scalar(
        "UPDATE agent_sessions SET current_phase = 'waiting', current_phase_detail = NULL \
         WHERE session_id = $1 AND current_phase <> 'waiting' RETURNING id",
    )
    .bind(session_id)
    .fetch_all(&mut **conn)
    .await
    .unwrap_or_default();
    let Some(mqtt) = mqtt else { return };
    for agent_session_id in settled {
        let envelope = StreamEnvelope::AgentPhaseChanged { agent_session_id, phase: "waiting".to_string(), detail: None };
        let _ = mqtt.publish(session_id, &envelope).await;
    }
}

async fn post_failure_notice(conn: &mut PoolConnection<Postgres>, mqtt: &MqttPublisher, session_id: Uuid, err: &TurnError) -> String {
    let locale = session_owner_locale(conn, session_id).await;
    let explanation = err.user_message_in(locale);
    let inserted: Result<Uuid, sqlx::Error> = sqlx::query_scalar(
        "INSERT INTO messages (session_id, sender_channel_identity_id, content, agent_display_name) VALUES ($1, NULL, $2, NULL) RETURNING id",
    )
    .bind(session_id)
    .bind(&explanation)
    .fetch_one(&mut **conn)
    .await;
    if let Ok(message_id) = inserted {
        let _ = mqtt.publish(session_id, &StreamEnvelope::MessageCreated { message_id }).await;
    }
    explanation
}
