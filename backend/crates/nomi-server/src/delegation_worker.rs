use std::time::Duration;

use sqlx::PgPool;
use uuid::Uuid;

use nomi_agent_core::LoopOutcome;
use nomi_llm::{ContentBlock, LlmMessage, LlmRole};
use nomi_realtime::{MqttPublisher, StreamEnvelope};

use crate::bootstrap::build_embedding_provider_from_settings_or_env;

const NOTIFY_CHANNEL: &str = "agent_delegations_channel";
const POLL_FALLBACK_INTERVAL: Duration = Duration::from_secs(5);
/// Room for an agent's reply (thinking gets its own budget on top). A ceiling, not a target:
/// plans, drafts and tables need far more than a chat line.
const DELEGATED_MAX_TOKENS: u32 = 8192;

struct ClaimedDelegation {
    id: Uuid,
    session_id: Uuid,
    user_id: Uuid,
    target_agent_type: String,
    task: String,
}

async fn claim_next(pool: &PgPool) -> Result<Option<ClaimedDelegation>, sqlx::Error> {
    let row: Option<(Uuid, Uuid, Uuid, String, String)> = sqlx::query_as(
        "WITH claimed AS ( \
             SELECT id FROM agent_delegations \
             WHERE status = 'pending' \
             ORDER BY created_at \
             FOR UPDATE SKIP LOCKED \
             LIMIT 1 \
         ) \
         UPDATE agent_delegations SET status = 'processing', claimed_at = now() \
         WHERE id IN (SELECT id FROM claimed) \
         RETURNING id, session_id, user_id, target_agent_type, task",
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|(id, session_id, user_id, target_agent_type, task)| ClaimedDelegation {
        id,
        session_id,
        user_id,
        target_agent_type,
        task,
    }))
}

async fn fail_and_notify(pool: &PgPool, mqtt: &MqttPublisher, delegation_id: Uuid, session_id: Uuid, error: &str) {
    let _ = sqlx::query("UPDATE agent_delegations SET status = 'failed', completed_at = now(), error = $2 WHERE id = $1 AND status <> 'cancelled'")
        .bind(delegation_id)
        .bind(error)
        .execute(pool)
        .await;
    let _ = mqtt.publish(session_id, &StreamEnvelope::AgentDelegationUpdated { delegation_id }).await;
}

/// Posts a message into the chat and tells the open chat about it right away.
async fn post(conn: &mut sqlx::PgConnection, mqtt: &MqttPublisher, session_id: Uuid, author: &str, text: &str) {
    let message_id: Option<Uuid> = sqlx::query_scalar(
        "INSERT INTO messages (session_id, sender_channel_identity_id, content, agent_display_name) VALUES ($1, NULL, $2, $3) RETURNING id",
    )
    .bind(session_id)
    .bind(text)
    .bind(author)
    .fetch_one(conn)
    .await
    .ok();
    if let Some(message_id) = message_id {
        let _ = mqtt.publish(session_id, &StreamEnvelope::MessageCreated { message_id }).await;
    }
}

/// Planning's delegation task is always formatted "Project <uuid>: ..." (see
/// nomi-agent-planning's system prompt) — this is the only place that convention needs parsing,
/// since it's just used to know which project to mark ready, a UI/status nicety. A malformed or
/// missing UUID here is silently ignored, not an error: the project simply stays "building" and
/// nothing about the delegation itself fails over a status cosmetic.
fn extract_project_id(task: &str) -> Option<Uuid> {
    task.strip_prefix("Project ")?.split_once(':').map(|(id, _)| id.trim()).and_then(|id| id.parse().ok())
}

/// Runs the delegation-processing worker loop forever: claims pending `agent_delegations` (via
/// LISTEN/NOTIFY with a polling fallback, mirroring worker.rs's turn_jobs loop exactly), runs
/// each through nomi_agent_core::run_agent_turn directly, and posts the agent's own answer.
/// Deliberately does NOT take the conversational session's advisory lock (see the design spec) — this must never block a user's live conversation.
pub async fn run(pool: PgPool, mqtt: MqttPublisher, s3: Option<nomi_storage::S3Config>, settings_key: [u8; 32], http_client: reqwest::Client, database_url: String, project_storage: nomi_storage::ProjectStore) {
    let mut listener = match sqlx::postgres::PgListener::connect(&database_url).await {
        Ok(listener) => listener,
        Err(e) => {
            tracing::error!(error = %e, "delegation worker: failed to connect LISTEN client; not started");
            return;
        }
    };
    if let Err(e) = listener.listen(NOTIFY_CHANNEL).await {
        tracing::error!(error = %e, "delegation worker: failed to LISTEN on agent_delegations_channel; not started");
        return;
    }
    tracing::info!("delegation worker: listening for new agent delegations");

    let registry = crate::build_agent_registry(project_storage);

    loop {
        let _ = tokio::time::timeout(POLL_FALLBACK_INTERVAL, listener.recv()).await;

        loop {
            let claimed = match claim_next(&pool).await {
                Ok(Some(job)) => job,
                Ok(None) => break,
                Err(e) => {
                    tracing::error!(error = %e, "delegation worker: failed to claim next delegation");
                    break;
                }
            };

            let Some(agent) = registry.find(&claimed.target_agent_type) else {
                tracing::warn!(delegation_id = %claimed.id, target = %claimed.target_agent_type, "delegation worker: unknown target agent");
                fail_and_notify(&pool, &mqtt, claimed.id, claimed.session_id, "unknown target agent").await;
                continue;
            };

            // Koda builds with the coding model; everyone else with the person's chat model.
            let purpose = crate::bootstrap::purpose_for_agent(&claimed.target_agent_type);
            let provider = crate::bootstrap::build_llm_provider_for(&pool, claimed.user_id, &settings_key, http_client.clone(), purpose).await;
            let embedding_provider =
                build_embedding_provider_from_settings_or_env(&pool, &settings_key, http_client.clone()).await;

            let mut conn = match pool.acquire().await {
                Ok(conn) => conn,
                Err(e) => {
                    fail_and_notify(&pool, &mqtt, claimed.id, claimed.session_id, &e.to_string()).await;
                    continue;
                }
            };

            let locale = nomi_agent_core::user_locale(&mut conn, claimed.user_id).await;
            let started_message = nomi_agent_supervisor::phrase_delegation_started(
                provider.as_ref(),
                &mut conn,
                claimed.user_id,
                &claimed.target_agent_type,
                &claimed.task,
            )
            .await
            .unwrap_or_else(|_| locale.tf("turn.working_with", &[("agent", &claimed.target_agent_type)]));

            post(&mut conn, &mqtt, claimed.session_id, "Supervisor", &started_message).await;

            let _ = mqtt.publish(claimed.session_id, &StreamEnvelope::AgentDelegationUpdated { delegation_id: claimed.id }).await;

            let mut messages = vec![LlmMessage { role: LlmRole::User, content: vec![ContentBlock::Text { text: claimed.task.clone() }] }];
            // A task can carry the user's files (the Files agent passes their tags along).
            nomi_agent_core::attachments::expand_messages(&mut conn, claimed.user_id, &mut messages).await;

            let outcome = nomi_agent_core::run_agent_turn(
                &mut conn,
                Some((&mqtt, claimed.id)),
                s3.as_ref(),
                provider.as_ref(),
                embedding_provider.as_ref(),
                &registry,
                agent.as_ref(),
                claimed.session_id,
                claimed.session_id,
                claimed.user_id,
                messages,
                DELEGATED_MAX_TOKENS,
            )
            .await;

            // The delegated run streamed its progress into the chat under the delegation's id;
            // close it so the chat stops showing it as working.
            let _ = mqtt
                .publish(claimed.session_id, &StreamEnvelope::TurnCompleted { turn_job_id: claimed.id, message_id: Uuid::nil() })
                .await;

            match outcome {
                Ok(LoopOutcome::Reply { text, .. }) | Ok(LoopOutcome::Completed { summary: text, .. }) => {
                    // The agent's own answer, whole: rewording it through a short supervisor
                    // completion cut long plans and lists off mid-sentence.
                    if !text.trim().is_empty() {
                        post(&mut conn, &mqtt, claimed.session_id, agent.display_name().as_ref(), &text).await;
                    }

                    let _ = sqlx::query(
                        "UPDATE agent_delegations SET status = 'completed', completed_at = now(), result = $2 WHERE id = $1 AND status = 'processing'",
                    )
                    .bind(claimed.id)
                    .bind(&text)
                    .execute(&pool)
                    .await;

                    if claimed.target_agent_type == nomi_agent_coding::CODING_AGENT_TYPE {
                        if let Some(project_id) = extract_project_id(&claimed.task) {
                            let _ = sqlx::query("UPDATE projects SET status = 'ready' WHERE id = $1 AND status = 'building'")
                                .bind(project_id)
                                .execute(&pool)
                                .await;
                        }
                    }

                    let _ = mqtt.publish(claimed.session_id, &StreamEnvelope::AgentDelegationUpdated { delegation_id: claimed.id }).await;

                    tracing::info!(delegation_id = %claimed.id, "delegation worker: delegation completed");
                }
                // Nobody is waiting in a live turn here: the specialist it handed to is queued
                // as its own background delegation, and this one is done.
                Ok(LoopOutcome::HandOff { target_agent, task }) => {
                    let _ = nomi_agent_core::delegation::create_delegation(
                        &mut conn,
                        Some((&mqtt, claimed.id)),
                        claimed.session_id,
                        &claimed.target_agent_type,
                        &target_agent,
                        &task,
                        claimed.user_id,
                    )
                    .await;
                    let _ = sqlx::query(
                        "UPDATE agent_delegations SET status = 'completed', completed_at = now(), result = $2 WHERE id = $1 AND status = 'processing'",
                    )
                    .bind(claimed.id)
                    .bind(format!("Handed to {target_agent}"))
                    .execute(&pool)
                    .await;
                    let _ = mqtt.publish(claimed.session_id, &StreamEnvelope::AgentDelegationUpdated { delegation_id: claimed.id }).await;
                }
                // The engine parked the paused turn in its own agent_sessions row; the approval
                // worker resumes it from there (nomi_turn::resume_paused_turn). Tag that row with
                // this delegation so the resume can mark the delegation completed once the
                // delegated agent finishes. Until then it stays 'processing', since it isn't
                // done yet.
                Ok(LoopOutcome::AwaitingApproval { message_id }) => {
                    tracing::info!(delegation_id = %claimed.id, %message_id, "delegation worker: delegated turn is awaiting tool approval");
                    let _ = sqlx::query(
                        "UPDATE agent_sessions SET state = state || jsonb_build_object('delegation_id', $1::text) \
                         WHERE state->>'pending_approval_message_id' = $2",
                    )
                    .bind(claimed.id.to_string())
                    .bind(message_id.to_string())
                    .execute(&pool)
                    .await;
                    let notice = format!(
                        "The {} agent is waiting on your approval for a tool call before it can continue.",
                        claimed.target_agent_type
                    );
                    post(&mut conn, &mqtt, claimed.session_id, "Supervisor", &notice).await;
                    let _ = mqtt.publish(claimed.session_id, &StreamEnvelope::AgentDelegationUpdated { delegation_id: claimed.id }).await;
                }
                // Stopped by the user (nomi-agent-supervisor's stop). The supervisor already marked
                // the delegation cancelled and told the user; this only refreshes the crew panel.
                Ok(LoopOutcome::Cancelled) => {
                    tracing::info!(delegation_id = %claimed.id, "delegation worker: delegated turn was stopped by the user");
                    let _ = sqlx::query(
                        "UPDATE agent_delegations SET status = 'cancelled', completed_at = now() WHERE id = $1 AND status = 'processing'",
                    )
                    .bind(claimed.id)
                    .execute(&pool)
                    .await;
                    let _ = mqtt.publish(claimed.session_id, &StreamEnvelope::AgentDelegationUpdated { delegation_id: claimed.id }).await;
                }
                Err(e) => {
                    tracing::warn!(delegation_id = %claimed.id, error = %e, "delegation worker: delegated turn failed");
                    let sorry = locale.tf("turn.agent_failed", &[("agent", &claimed.target_agent_type), ("reason", &e.user_message_in(locale))]);
                    post(&mut conn, &mqtt, claimed.session_id, "Supervisor", &sorry).await;
                    fail_and_notify(&pool, &mqtt, claimed.id, claimed.session_id, &e.to_string()).await;
                }
            }
        }
    }
}
