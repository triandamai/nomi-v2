use std::time::Duration;

use sqlx::PgPool;
use uuid::Uuid;

use crate::bootstrap::build_embedding_provider_from_settings_or_env;
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
/// Past the turn time limit, how long a 'processing' job may sit before it's counted as abandoned.
const ABANDONED_GRACE: std::time::Duration = std::time::Duration::from_secs(120);

pub async fn run(pool: PgPool, mqtt: MqttPublisher, s3: Option<nomi_storage::S3Config>, settings_key: [u8; 32], http_client: reqwest::Client, database_url: String, project_storage: nomi_storage::ProjectStore) {
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

    let registry = crate::build_agent_registry(&pool, project_storage.clone());
    let turn_registry = crate::build_agent_registry(&pool, project_storage.clone());
    let catalog = crate::build_tool_catalog(project_storage);
    let turns = std::sync::Arc::new(Turns {
        pool: pool.clone(),
        mqtt: mqtt.clone(),
        s3: s3.clone(),
        settings_key,
        http_client: http_client.clone(),
        registry: turn_registry,
        catalog: catalog.clone(),
    });
    let slots = Slots::default();
    let finished = std::sync::Arc::new(tokio::sync::Notify::new());

    loop {
        // Wake on NOTIFY from either channel, or on the fallback interval if a NOTIFY is ever
        // missed — either way, fall through to draining every currently-pending job on both
        // queues before waiting again.
        tokio::select! {
            _ = tokio::time::timeout(POLL_FALLBACK_INTERVAL, listener.recv()) => {}
            _ = tokio::time::timeout(POLL_FALLBACK_INTERVAL, approval_listener.recv()) => {}
            _ = finished.notified() => {}
        }

        // A turn can't legitimately run past its time limit, so anything still 'processing'
        // well after it was orphaned by a crash or restart.
        match queue::fail_abandoned(&pool, nomi_turn::turn_time_limit().await + ABANDONED_GRACE).await {
            Ok(0) => {}
            Ok(n) => tracing::warn!(count = n, "worker: failed turn jobs that no worker finished"),
            Err(e) => tracing::error!(error = %e, "worker: failed to clear abandoned turn jobs"),
        }

        // Turns from different chats run side by side, so one slow model call never holds up
        // everyone else's; claim_next never hands out a chat's next message while one of its
        // turns is still running, so each chat stays in order.
        loop {
            // Read each time, so a change in Admin → Configuration applies to the next claim.
            let Some(slot) = slots.try_take(turn_concurrency().await) else {
                break; // every slot busy: a finishing turn wakes the loop again
            };
            let claimed = match queue::claim_next(&pool).await {
                Ok(Some(job)) => job,
                Ok(None) => break,
                Err(e) => {
                    tracing::error!(error = %e, "worker: failed to claim next turn job");
                    break;
                }
            };
            let turns = turns.clone();
            let finished = finished.clone();
            tokio::spawn(async move {
                let _slot = slot;
                run_turn_job(&turns, claimed).await;
                finished.notify_one();
            });
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

            let provider = crate::bootstrap::build_llm_provider_in_session(&pool, user_id, session_id, &settings_key, http_client.clone(), crate::bootstrap::ModelPurpose::Chat).await;
            let embedding_provider = build_embedding_provider_from_settings_or_env(&pool, &settings_key, http_client.clone(), Some(user_id)).await;

            let result = nomi_turn::resume_paused_turn(
                &pool, &mqtt, s3.as_ref(), provider.as_ref(), embedding_provider.as_ref(), &registry, &catalog,
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
                    if let Some(message_id) = outcome.message_id {
                        crate::user_events::announce_chat_reply(&pool, &mqtt, user_id, outcome.session_id, message_id, &outcome.reply);
                    }
                    nomi_turn::activity::announce(&pool, &mqtt, outcome.session_id, user_id, nomi_turn::activity::Ended::Completed).await;
                    tracing::info!(resume_id = %claimed.id, "worker: approval resume completed");
                }
                Err(e) => {
                    let _ = nomi_turn::approval::mark_failed(&pool, claimed.id, &e.to_string()).await;
                    nomi_turn::activity::announce(&pool, &mqtt, session_id, user_id, nomi_turn::activity::Ended::Failed).await;
                    tracing::warn!(resume_id = %claimed.id, error = %e, "worker: approval resume failed");
                }
            }
        }
    }
}

/// What every running turn shares.
struct Turns {
    pool: PgPool,
    mqtt: MqttPublisher,
    s3: Option<nomi_storage::S3Config>,
    settings_key: [u8; 32],
    http_client: reqwest::Client,
    registry: nomi_agent_core::AgentRegistry,
    catalog: std::sync::Arc<nomi_agent_core::ToolCatalog>,
}

/// Turns that may run at once unless Admin → Configuration (or `TURN_CONCURRENCY`) says.
const DEFAULT_TURN_CONCURRENCY: u64 = 4;

/// How many turns may run at once.
async fn turn_concurrency() -> usize {
    let agent_loop = nomi_settings::config::current(&nomi_settings::config::AGENT_LOOP).await;
    agent_loop.number("turn_concurrency").unwrap_or(DEFAULT_TURN_CONCURRENCY) as usize
}

/// The turns running now, against a limit that can change between claims (a semaphore's can't
/// shrink while its permits are out).
#[derive(Clone, Default)]
struct Slots(std::sync::Arc<std::sync::atomic::AtomicUsize>);

/// One running turn's place; dropping it frees the place.
struct Slot(std::sync::Arc<std::sync::atomic::AtomicUsize>);

impl Slots {
    fn try_take(&self, limit: usize) -> Option<Slot> {
        use std::sync::atomic::Ordering;
        self.0.fetch_update(Ordering::AcqRel, Ordering::Acquire, |running| (running < limit).then_some(running + 1)).ok()?;
        Some(Slot(self.0.clone()))
    }
}

impl Drop for Slot {
    fn drop(&mut self) {
        self.0.fetch_sub(1, std::sync::atomic::Ordering::AcqRel);
    }
}

/// Runs one claimed turn and records how it ended.
async fn run_turn_job(turns: &Turns, claimed: queue::ClaimedJob) {
    let user_id: Result<Uuid, sqlx::Error> =
        sqlx::query_scalar("SELECT user_id FROM channel_identities WHERE id = $1")
            .bind(claimed.sender_channel_identity_id)
            .fetch_one(&turns.pool)
            .await;
    let user_id = match user_id {
        Ok(id) => id,
        Err(e) => {
            tracing::error!(error = %e, job_id = %claimed.id, "worker: failed to resolve user_id for claimed job");
            let _ = queue::mark_failed(&turns.pool, claimed.id, &e.to_string()).await;
            return;
        }
    };

    // The chat's own model when someone picked one in the chat, else the person's.
    let provider = crate::bootstrap::build_llm_provider_in_session(
        &turns.pool,
        user_id,
        claimed.session_id,
        &turns.settings_key,
        turns.http_client.clone(),
        crate::bootstrap::ModelPurpose::Chat,
    )
    .await;
    let embedding_provider =
        build_embedding_provider_from_settings_or_env(&turns.pool, &turns.settings_key, turns.http_client.clone(), Some(user_id)).await;

    let result = nomi_turn::process_turn(
        &turns.pool,
        &turns.mqtt,
        turns.s3.as_ref(),
        provider.as_ref(),
        embedding_provider.as_ref(),
        &turns.registry,
        &turns.catalog,
        claimed.id,
        claimed.session_id,
        claimed.sender_channel_identity_id,
        user_id,
        &claimed.text,
    )
    .await;

    match result {
        Ok(outcome) => {
            let _ = queue::mark_completed(&turns.pool, claimed.id).await;
            let _ = turns.mqtt
                .publish(
                    claimed.session_id,
                    &StreamEnvelope::TurnCompleted { turn_job_id: claimed.id, message_id: outcome.message_id.unwrap_or(Uuid::nil()) },
                )
                .await;
            if let Some(message_id) = outcome.message_id {
                crate::user_events::announce_chat_reply(&turns.pool, &turns.mqtt, user_id, claimed.session_id, message_id, &outcome.reply);
            }
            nomi_turn::activity::announce(&turns.pool, &turns.mqtt, claimed.session_id, user_id, nomi_turn::activity::Ended::Completed).await;
            tracing::info!(job_id = %claimed.id, reply_len = outcome.reply.len(), "worker: turn job completed");
        }
        Err(e) => {
            // process_turn already published TurnFailed and recorded the agent_events row
            // internally (see turn::process_turn) — this only updates the job's own status.
            let _ = queue::mark_failed(&turns.pool, claimed.id, &e.to_string()).await;
            nomi_turn::activity::announce(&turns.pool, &turns.mqtt, claimed.session_id, user_id, nomi_turn::activity::Ended::Failed).await;
            tracing::warn!(job_id = %claimed.id, error = %e, "worker: turn job failed");
        }
    }
}

#[cfg(test)]
mod slot_tests {
    use super::Slots;

    #[test]
    fn slots_follow_a_limit_that_changes_between_claims() {
        let slots = Slots::default();
        let first = slots.try_take(2).unwrap();
        let _second = slots.try_take(2).unwrap();
        assert!(slots.try_take(2).is_none(), "both places taken");
        // Lowered to 1: nothing new starts until the running ones finish.
        drop(first);
        assert!(slots.try_take(1).is_none());
        // Raised to 3: two more fit beside the one still running, and no more.
        let _third = slots.try_take(3).unwrap();
        let _fourth = slots.try_take(3).unwrap();
        assert!(slots.try_take(3).is_none());
    }
}
