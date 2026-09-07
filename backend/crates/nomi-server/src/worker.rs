use std::time::Duration;

use sqlx::PgPool;
use uuid::Uuid;

use crate::bootstrap::{build_embedding_provider_from_settings_or_env, build_llm_provider_for_user};
use nomi_realtime::{MqttPublisher, StreamEnvelope};
use nomi_turn::queue;

const NOTIFY_CHANNEL: &str = "turn_jobs_channel";
const APPROVAL_NOTIFY_CHANNEL: &str = "approval_resumes_channel";
const POLL_FALLBACK_INTERVAL: Duration = Duration::from_secs(5);

/// Runs the turn-processing worker loop forever: claims pending `turn_jobs` (via LISTEN/NOTIFY
/// with a polling fallback), runs each through `turn::process_turn`, and publishes the terminal
/// MQTT envelope. Shared by the standalone `worker` binary (`src/bin/worker.rs`) and, when
/// `RUN_WORKER_INLINE` isn't set to `false`, a background task spawned by the main server
/// binary (`src/main.rs`) — see that file for why embedding it there is opt-out rather than a
/// separate always-required process for local/single-instance use.
pub async fn run(pool: PgPool, mqtt: MqttPublisher, settings_key: [u8; 32], http_client: reqwest::Client, database_url: String, project_storage: nomi_storage::LocalFsStore) {
    let mut listener = match sqlx::postgres::PgListener::connect(&database_url).await {
        Ok(listener) => listener,
        Err(e) => {
            tracing::error!(error = %e, "worker: failed to connect LISTEN client; worker loop not started");
            return;
        }
    };
    if let Err(e) = listener.listen(NOTIFY_CHANNEL).await {
        tracing::error!(error = %e, "worker: failed to LISTEN on turn_jobs_channel; worker loop not started");
        return;
    }
    tracing::info!("worker: listening for new turn jobs");

    let mut approval_listener = match sqlx::postgres::PgListener::connect(&database_url).await {
        Ok(listener) => listener,
        Err(e) => {
            tracing::error!(error = %e, "worker: failed to connect approval-resume LISTEN client; worker loop not started");
            return;
        }
    };
    if let Err(e) = approval_listener.listen(APPROVAL_NOTIFY_CHANNEL).await {
        tracing::error!(error = %e, "worker: failed to LISTEN on approval_resumes_channel; worker loop not started");
        return;
    }

    let registry = crate::build_agent_registry(project_storage);

    loop {
        // Wake on NOTIFY from either channel, or on the fallback interval if a NOTIFY is ever
        // missed — either way, fall through to draining every currently-pending job on both
        // queues before waiting again.
        tokio::select! {
            _ = tokio::time::timeout(POLL_FALLBACK_INTERVAL, listener.recv()) => {}
            _ = tokio::time::timeout(POLL_FALLBACK_INTERVAL, approval_listener.recv()) => {}
        }

        loop {
            let claimed = match queue::claim_next(&pool).await {
                Ok(Some(job)) => job,
                Ok(None) => break,
                Err(e) => {
                    tracing::error!(error = %e, "worker: failed to claim next turn job");
                    break;
                }
            };

            let user_id: Result<Uuid, sqlx::Error> =
                sqlx::query_scalar("SELECT user_id FROM channel_identities WHERE id = $1")
                    .bind(claimed.sender_channel_identity_id)
                    .fetch_one(&pool)
                    .await;
            let user_id = match user_id {
                Ok(id) => id,
                Err(e) => {
                    tracing::error!(error = %e, job_id = %claimed.id, "worker: failed to resolve user_id for claimed job");
                    let _ = queue::mark_failed(&pool, claimed.id, &e.to_string()).await;
                    continue;
                }
            };

            let provider = build_llm_provider_for_user(&pool, user_id, &settings_key, http_client.clone()).await;
            let embedding_provider =
                build_embedding_provider_from_settings_or_env(&pool, &settings_key, http_client.clone()).await;

            let result = nomi_turn::process_turn(
                &pool,
                &mqtt,
                provider.as_ref(),
                embedding_provider.as_ref(),
                &registry,
                claimed.id,
                claimed.session_id,
                claimed.sender_channel_identity_id,
                user_id,
                &claimed.text,
            )
            .await;

            match result {
                Ok(outcome) => {
                    let _ = queue::mark_completed(&pool, claimed.id).await;
                    let _ = mqtt
                        .publish(
                            claimed.session_id,
                            &StreamEnvelope::TurnCompleted { turn_job_id: claimed.id, message_id: outcome.message_id.unwrap_or(Uuid::nil()) },
                        )
                        .await;
                    tracing::info!(job_id = %claimed.id, reply_len = outcome.reply.len(), "worker: turn job completed");
                }
                Err(e) => {
                    // process_turn already published TurnFailed and recorded the agent_events row
                    // internally (see turn::process_turn) — this only updates the job's own status.
                    let _ = queue::mark_failed(&pool, claimed.id, &e.to_string()).await;
                    tracing::warn!(job_id = %claimed.id, error = %e, "worker: turn job failed");
                }
            }
        }

        loop {
            let claimed = match nomi_turn::approval::claim_next(&pool).await {
                Ok(Some(job)) => job,
                Ok(None) => break,
                Err(e) => {
                    tracing::error!(error = %e, "worker: failed to claim next approval resume");
                    break;
                }
            };

            let session_id: Result<Uuid, sqlx::Error> = sqlx::query_scalar("SELECT session_id FROM messages WHERE id = $1")
                .bind(claimed.message_id)
                .fetch_one(&pool)
                .await;
            let session_id = match session_id {
                Ok(id) => id,
                Err(e) => {
                    tracing::error!(error = %e, resume_id = %claimed.id, "worker: failed to resolve session_id for approval resume");
                    let _ = nomi_turn::approval::mark_failed(&pool, claimed.id, &e.to_string()).await;
                    continue;
                }
            };
            let user_id: Result<Uuid, sqlx::Error> = sqlx::query_scalar(
                "SELECT ci.user_id FROM agent_sessions ags JOIN channel_identities ci ON ci.id = ags.sender_channel_identity_id \
                 WHERE ags.state->>'pending_approval_message_id' = $1",
            )
            .bind(claimed.message_id.to_string())
            .fetch_one(&pool)
            .await;
            let user_id = match user_id {
                Ok(id) => id,
                Err(e) => {
                    tracing::error!(error = %e, resume_id = %claimed.id, "worker: failed to resolve user_id for approval resume");
                    let _ = nomi_turn::approval::mark_failed(&pool, claimed.id, &e.to_string()).await;
                    continue;
                }
            };
            let _ = session_id; // resolved above only to fail fast with a clear error if the message/session no longer exists

            let provider = build_llm_provider_for_user(&pool, user_id, &settings_key, http_client.clone()).await;
            let embedding_provider = build_embedding_provider_from_settings_or_env(&pool, &settings_key, http_client.clone()).await;

            let result = nomi_turn::resume_paused_turn(
                &pool, &mqtt, provider.as_ref(), embedding_provider.as_ref(), &registry,
                claimed.message_id, &claimed.decision, claimed.remember,
            )
            .await;

            match result {
                Ok(outcome) => {
                    let _ = nomi_turn::approval::mark_completed(&pool, claimed.id).await;
                    let _ = mqtt
                        .publish(
                            outcome.session_id,
                            &StreamEnvelope::TurnCompleted { turn_job_id: Uuid::nil(), message_id: outcome.message_id.unwrap_or(Uuid::nil()) },
                        )
                        .await;
                    tracing::info!(resume_id = %claimed.id, "worker: approval resume completed");
                }
                Err(e) => {
                    let _ = nomi_turn::approval::mark_failed(&pool, claimed.id, &e.to_string()).await;
                    tracing::warn!(resume_id = %claimed.id, error = %e, "worker: approval resume failed");
                }
            }
        }
    }
}
