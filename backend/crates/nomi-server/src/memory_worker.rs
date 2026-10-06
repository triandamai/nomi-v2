//! Keeps working memory tidy (nomi_agent_core::working_memory): folds long chats into their
//! running summaries, and consolidates each person's memories about once a day.

use std::time::Duration;

use sqlx::PgPool;

use crate::bootstrap::build_llm_provider_for_user;
use nomi_agent_core::working_memory;

const POLL_INTERVAL: Duration = Duration::from_secs(60);
/// Chats summarized per sweep (each is one model call, on the chat owner's model and usage).
const SUMMARIES_PER_SWEEP: i64 = 5;
/// People consolidated per sweep (no model calls).
const CONSOLIDATIONS_PER_SWEEP: i64 = 50;

pub async fn run(pool: PgPool, settings_key: [u8; 32], http_client: reqwest::Client) {
    tracing::info!("memory worker: started");
    loop {
        sweep(&pool, &settings_key, &http_client).await;
        tokio::time::sleep(POLL_INTERVAL).await;
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
