//! Learns from chats and keeps working memory tidy: saves what people tell Nomi from the
//! exchanges turns queue (nomi_agent_core::memory::queue_learning), folds long chats into their
//! running summaries (nomi_agent_core::working_memory), rewrites each person's "recently" note
//! when one of their chats goes quiet (nomi_agent_core::recent_activity), and consolidates each
//! person's memories about once a day.

use std::time::Duration;

use sqlx::PgPool;

use crate::bootstrap::{build_embedding_provider_from_settings_or_env, build_llm_provider_for_user};
use nomi_agent_core::{memory, recent_activity, working_memory};

/// How often queued exchanges are checked: soon after a reply, without being on its path.
const LEARN_INTERVAL: Duration = Duration::from_secs(5);
/// How often summaries and consolidation run.
const SWEEP_INTERVAL: Duration = Duration::from_secs(60);
/// Queued exchanges learned from per pass.
const LEARNING_PER_PASS: i64 = 20;
/// One exchange's model and embedding calls; past this it's retried later (see LEARNING_ATTEMPTS).
const LEARNING_TIME_LIMIT: Duration = Duration::from_secs(60);
/// Chats summarized per sweep (each is one model call, on the chat owner's model and usage).
const SUMMARIES_PER_SWEEP: i64 = 5;
/// Quiet chats noted into their person's "recently" note per sweep (one model call each).
const NOTES_PER_SWEEP: i64 = 10;
/// People consolidated per sweep (no model calls).
const CONSOLIDATIONS_PER_SWEEP: i64 = 50;

pub async fn run(pool: PgPool, settings_key: [u8; 32], http_client: reqwest::Client) {
    tracing::info!(
        learn_every_secs = LEARN_INTERVAL.as_secs(),
        sweep_every_secs = SWEEP_INTERVAL.as_secs(),
        "memory worker: started"
    );
    let mut last_sweep: Option<std::time::Instant> = None;
    loop {
        // Each pass runs in its own task: a panic (a provider that can't be built, say) is logged
        // and the worker carries on, instead of the whole worker dying without a word.
        let (p, h) = (pool.clone(), http_client.clone());
        guarded("learning pass", async move { learn(&p, &settings_key, &h).await }).await;
        if last_sweep.is_none_or(|at| at.elapsed() >= SWEEP_INTERVAL) {
            let (p, h) = (pool.clone(), http_client.clone());
            guarded("sweep", async move { sweep(&p, &settings_key, &h).await }).await;
            last_sweep = Some(std::time::Instant::now());
        }
        tokio::time::sleep(LEARN_INTERVAL).await;
    }
}

/// Runs one pass in its own task and logs it if it panics.
async fn guarded(what: &'static str, pass: impl std::future::Future<Output = ()> + Send + 'static) {
    if let Err(e) = tokio::spawn(pass).await {
        if e.is_panic() {
            let panic = e.into_panic();
            let message = panic
                .downcast_ref::<&str>()
                .map(|m| m.to_string())
                .or_else(|| panic.downcast_ref::<String>().cloned())
                .unwrap_or_else(|| "unknown panic".to_string());
            tracing::error!(panic = %message, "memory worker: the {what} panicked; carrying on");
        } else {
            tracing::error!(error = %e, "memory worker: the {what} was cancelled");
        }
    }
}

/// Learns from the exchanges turns queued. Each runs on its person's own model and usage, with a
/// time limit; a failed or timed-out one stays queued for another try.
pub async fn learn(pool: &PgPool, settings_key: &[u8; 32], http_client: &reqwest::Client) {
    let jobs = match memory::take_learning_jobs(pool, LEARNING_PER_PASS).await {
        Ok(jobs) => jobs,
        Err(e) => {
            tracing::error!(error = %e, "memory worker: failed to read queued exchanges");
            return;
        }
    };
    if jobs.is_empty() {
        return;
    }
    tracing::info!(count = jobs.len(), "memory worker: picked up queued exchanges");
    for job in &jobs {
        tracing::info!(
            job_id = %job.id,
            user_id = %job.user_id,
            attempt = job.attempts,
            of = memory::LEARNING_ATTEMPTS,
            "memory worker: processing an exchange"
        );
        let provider = build_llm_provider_for_user(pool, job.user_id, settings_key, http_client.clone()).await;
        let embedding_provider = build_embedding_provider_from_settings_or_env(pool, settings_key, http_client.clone(), Some(job.user_id)).await;
        let started = chrono::Utc::now();
        let last_try = job.attempts >= memory::LEARNING_ATTEMPTS;
        match tokio::time::timeout(LEARNING_TIME_LIMIT, memory::run_learning_job(pool, provider.as_ref(), embedding_provider.as_ref(), job)).await {
            Ok(Ok(learned)) => {
                tracing::info!(job_id = %job.id, user_id = %job.user_id, ?learned, "memory worker: exchange done");
                if matches!(learned, memory::Learned::Added { .. } | memory::Learned::Updated { .. }) {
                    announce_saved(pool, job, started).await;
                }
            }
            Ok(Err(e)) => tracing::warn!(
                error = %e,
                job_id = %job.id,
                user_id = %job.user_id,
                attempt = job.attempts,
                last_try,
                "memory worker: learning from an exchange failed"
            ),
            Err(_) => tracing::warn!(
                job_id = %job.id,
                user_id = %job.user_id,
                attempt = job.attempts,
                last_try,
                limit_secs = LEARNING_TIME_LIMIT.as_secs(),
                "memory worker: learning from an exchange timed out"
            ),
        }
    }
    match memory::drop_failed_learning_jobs(pool).await {
        Ok(0) => {}
        Ok(n) => tracing::warn!(count = n, "memory worker: gave up on exchanges that kept failing"),
        Err(e) => tracing::error!(error = %e, "memory worker: failed to clear exchanges that kept failing"),
    }
}

/// Something new was remembered from the person's message: the chat shows it where it happened
/// (the message's `saved_memories`), so its devices fetch the message again.
async fn announce_saved(pool: &PgPool, job: &memory::LearningJob, since: chrono::DateTime<chrono::Utc>) {
    let Some(message_id) = job.source_message_id else { return };
    let session: Option<uuid::Uuid> = sqlx::query_scalar(
        "SELECT m.session_id FROM messages m \
         WHERE m.id = $1 AND EXISTS (SELECT 1 FROM memory_items mi WHERE mi.source_message_id = m.id AND mi.created_at >= $2)",
    )
    .bind(message_id)
    .bind(since)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();
    if let Some(session_id) = session {
        crate::chat_events::message_updated(job.user_id, session_id, message_id);
    }
}

/// One pass: summaries, then "recently" notes, then consolidation. Every failure is logged and skipped.
pub async fn sweep(pool: &PgPool, settings_key: &[u8; 32], http_client: &reqwest::Client) {
    match working_memory::sessions_to_summarize(pool, SUMMARIES_PER_SWEEP).await {
        Ok(sessions) => {
            for (session_id, user_id) in sessions {
                let provider = build_llm_provider_for_user(pool, user_id, settings_key, http_client.clone()).await;
                match working_memory::summarize_session(pool, provider.as_ref(), session_id).await {
                    Ok(true) => tracing::info!(%session_id, "memory worker: chat summary updated"),
                    Ok(false) => {}
                    Err(e) => tracing::warn!(error = %e, %session_id, "memory worker: chat summary failed"),
                }
            }
        }
        Err(e) => tracing::error!(error = %e, "memory worker: failed to find chats to summarize"),
    }

    note_recent_activity(pool, settings_key, http_client).await;

    match working_memory::people_to_consolidate(pool, CONSOLIDATIONS_PER_SWEEP).await {
        Ok(people) => {
            for user_id in people {
                match working_memory::consolidate(pool, user_id).await {
                    Ok(tidied) if tidied != Default::default() => {
                        tracing::info!(%user_id, merged = tidied.merged, faded = tidied.faded, archived = tidied.archived, "memory worker: memories tidied")
                    }
                    Ok(_) => {}
                    Err(e) => tracing::warn!(error = %e, %user_id, "memory worker: consolidation failed"),
                }
            }
        }
        Err(e) => tracing::error!(error = %e, "memory worker: failed to find memories to tidy"),
    }
}

/// Folds chats that went quiet into their people's "recently" notes, on each person's own model.
/// A failed one isn't marked noted, so the next sweep tries it again.
pub async fn note_recent_activity(pool: &PgPool, settings_key: &[u8; 32], http_client: &reqwest::Client) {
    let chats = match recent_activity::chats_gone_quiet(pool, NOTES_PER_SWEEP).await {
        Ok(chats) => chats,
        Err(e) => {
            tracing::error!(error = %e, "memory worker: failed to find quiet chats to note");
            return;
        }
    };
    if !chats.is_empty() {
        tracing::info!(count = chats.len(), "memory worker: noting what people did in chats that went quiet");
    }
    for (session_id, user_id) in chats {
        let provider = build_llm_provider_for_user(pool, user_id, settings_key, http_client.clone()).await;
        match tokio::time::timeout(LEARNING_TIME_LIMIT, recent_activity::note_chat(pool, provider.as_ref(), session_id, user_id)).await {
            Ok(Ok(recent_activity::Noted::Updated { messages, chars })) => {
                tracing::info!(%session_id, %user_id, messages, chars, "memory worker: recent activity noted")
            }
            Ok(Ok(recent_activity::Noted::Nothing)) => {}
            Ok(Ok(recent_activity::Noted::ModelFailed(error))) => {
                tracing::warn!(%session_id, %user_id, %error, "memory worker: noting recent activity failed; trying again next sweep")
            }
            Ok(Err(e)) => tracing::warn!(error = %e, %session_id, %user_id, "memory worker: noting recent activity failed"),
            Err(_) => tracing::warn!(%session_id, %user_id, limit_secs = LEARNING_TIME_LIMIT.as_secs(), "memory worker: noting recent activity timed out"),
        }
    }
}
