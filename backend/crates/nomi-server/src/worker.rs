use std::time::Duration;

use sqlx::PgPool;
use uuid::Uuid;

use crate::bootstrap::{build_embedding_provider_from_settings_or_env, build_llm_provider_for_user};
use nomi_realtime::{MqttPublisher, StreamEnvelope};
use nomi_turn::queue;

const NOTIFY_CHANNEL: &str = "turn_jobs_channel";
const POLL_FALLBACK_INTERVAL: Duration = Duration::from_secs(5);

/// Runs the turn-processing worker loop forever: claims pending `turn_jobs` (via LISTEN/NOTIFY
/// with a polling fallback), runs each through `turn::process_turn`, and publishes the terminal
/// MQTT envelope. Shared by the standalone `worker` binary (`src/bin/worker.rs`) and, when
/// `RUN_WORKER_INLINE` isn't set to `false`, a background task spawned by the main server
/// binary (`src/main.rs`) — see that file for why embedding it there is opt-out rather than a
/// separate always-required process for local/single-instance use.
pub async fn run(pool: PgPool, mqtt: MqttPublisher, settings_key: [u8; 32], http_client: reqwest::Client, database_url: String) {
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

    let registry = crate::build_agent_registry();

    loop {
        // Wake on NOTIFY, or on the fallback interval if a NOTIFY is ever missed — either way,
        // fall through to draining every currently-pending job before waiting again.
        let _ = tokio::time::timeout(POLL_FALLBACK_INTERVAL, listener.recv()).await;

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
                    // process_turn's TurnOutcome doesn't carry the persisted reply message's id
                    // (only its text) — the terminal envelope uses the turn_job_id as the
                    // correlation key; the WebSocket bridge looks up the actual message row by
                    // session_id once it sees this event, rather than needing message_id here.
                    let _ = mqtt
                        .publish(
                            claimed.session_id,
                            &StreamEnvelope::TurnCompleted { turn_job_id: claimed.id, message_id: Uuid::nil() },
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
    }
}
