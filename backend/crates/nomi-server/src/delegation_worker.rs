use std::time::Duration;

use sqlx::PgPool;
use uuid::Uuid;

use nomi_agent_core::LoopOutcome;
use nomi_llm::{ContentBlock, LlmMessage, LlmRole};
use nomi_realtime::{MqttPublisher, StreamEnvelope};

use crate::bootstrap::{build_embedding_provider_from_settings_or_env, build_llm_provider_for_user};

const NOTIFY_CHANNEL: &str = "agent_delegations_channel";
const POLL_FALLBACK_INTERVAL: Duration = Duration::from_secs(5);
const DELEGATED_MAX_TOKENS: u32 = 1024;

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
    let _ = sqlx::query("UPDATE agent_delegations SET status = 'failed', completed_at = now(), error = $2 WHERE id = $1")
        .bind(delegation_id)
        .bind(error)
        .execute(pool)
        .await;
    let _ = mqtt.publish(session_id, &StreamEnvelope::AgentDelegationUpdated { delegation_id }).await;
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
/// each through nomi_agent_core::run_agent_turn directly, phrases the result via the supervisor
/// agent, and delivers it. Deliberately does NOT take the conversational session's advisory
/// lock (see the design spec) — this must never block a user's live conversation.
pub async fn run(pool: PgPool, mqtt: MqttPublisher, settings_key: [u8; 32], http_client: reqwest::Client, database_url: String, s3: Option<nomi_storage::S3Config>) {
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

    let registry = crate::build_agent_registry(s3);

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

            let provider = build_llm_provider_for_user(&pool, claimed.user_id, &settings_key, http_client.clone()).await;
            let embedding_provider =
                build_embedding_provider_from_settings_or_env(&pool, &settings_key, http_client.clone()).await;

            let mut conn = match pool.acquire().await {
                Ok(conn) => conn,
                Err(e) => {
                    fail_and_notify(&pool, &mqtt, claimed.id, claimed.session_id, &e.to_string()).await;
                    continue;
                }
            };

            let messages = vec![LlmMessage { role: LlmRole::User, content: vec![ContentBlock::Text { text: claimed.task.clone() }] }];

            let outcome = nomi_agent_core::run_agent_turn(
                &mut conn,
                None,
                provider.as_ref(),
                embedding_provider.as_ref(),
                &registry,
                agent,
                claimed.session_id,
                claimed.session_id,
                claimed.user_id,
                messages,
                DELEGATED_MAX_TOKENS,
            )
            .await;

            match outcome {
                Ok(LoopOutcome::Reply { text, .. }) | Ok(LoopOutcome::Completed { summary: text, .. }) => {
                    let phrased = nomi_agent_supervisor::phrase_delegation_result(
                        provider.as_ref(),
                        &mut conn,
                        claimed.user_id,
                        &claimed.target_agent_type,
                        &claimed.task,
                        &text,
                    )
                    .await
                    .unwrap_or_else(|_| text.clone());

                    let _ = sqlx::query("INSERT INTO messages (session_id, sender_channel_identity_id, content) VALUES ($1, NULL, $2)")
                        .bind(claimed.session_id)
                        .bind(&phrased)
                        .execute(&mut *conn)
                        .await;

                    let _ = sqlx::query(
                        "UPDATE agent_delegations SET status = 'completed', completed_at = now(), result = $2 WHERE id = $1",
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
                Err(e) => {
                    tracing::warn!(delegation_id = %claimed.id, error = %e, "delegation worker: delegated turn failed");
                    let sorry = format!("I wasn't able to get an answer from the {} agent — {}.", claimed.target_agent_type, e);
                    let _ = sqlx::query("INSERT INTO messages (session_id, sender_channel_identity_id, content) VALUES ($1, NULL, $2)")
                        .bind(claimed.session_id)
                        .bind(&sorry)
                        .execute(&mut *conn)
                        .await;
                    fail_and_notify(&pool, &mqtt, claimed.id, claimed.session_id, &e.to_string()).await;
                }
            }
        }
    }
}
