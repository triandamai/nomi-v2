//! Learns from chats and keeps working memory tidy: saves what people tell Nomi from the
//! exchanges turns queue (nomi_agent_core::memory::queue_learning), folds long chats into their
//! running summaries (nomi_agent_core::working_memory), and consolidates each person's
//! memories about once a day.

use std::time::Duration;

use sqlx::PgPool;

use crate::bootstrap::{build_embedding_provider_from_settings_or_env, build_llm_provider_for_user};
use nomi_agent_core::{memory, working_memory};

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
/// People consolidated per sweep (no model calls).
const CONSOLIDATIONS_PER_SWEEP: i64 = 50;

pub async fn run(pool: PgPool, settings_key: [u8; 32], http_client: reqwest::Client) {
    tracing::info!("memory worker: started");
    let mut last_sweep: Option<std::time::Instant> = None;
    loop {
        learn(&pool, &settings_key, &http_client).await;
        if last_sweep.is_none_or(|at| at.elapsed() >= SWEEP_INTERVAL) {
            sweep(&pool, &settings_key, &http_client).await;
            last_sweep = Some(std::time::Instant::now());
        }
        tokio::time::sleep(LEARN_INTERVAL).await;
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
    let embedding_provider = build_embedding_provider_from_settings_or_env(pool, settings_key, http_client.clone()).await;
    for job in &jobs {
        let provider = build_llm_provider_for_user(pool, job.user_id, settings_key, http_client.clone()).await;
        match tokio::time::timeout(LEARNING_TIME_LIMIT, memory::run_learning_job(pool, provider.as_ref(), embedding_provider.as_ref(), job)).await {
            Ok(Ok(())) => {}
            Ok(Err(e)) => tracing::warn!(error = %e, job_id = %job.id, "memory worker: learning from an exchange failed"),
            Err(_) => tracing::warn!(job_id = %job.id, "memory worker: learning from an exchange timed out"),
        }
    }
    match memory::drop_failed_learning_jobs(pool).await {
        Ok(0) => {}
        Ok(n) => tracing::warn!(count = n, "memory worker: gave up on exchanges that kept failing"),
        Err(e) => tracing::error!(error = %e, "memory worker: failed to clear exchanges that kept failing"),
    }
}

/// One pass: summaries first, then consolidation. Every failure is logged and skipped.
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
