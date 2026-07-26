pub mod bootstrap;
pub mod chitchat;
pub mod lock;
pub mod memory;
pub mod money_agent;
pub mod routing;
pub mod subagent;
pub mod tools;
pub mod types;

pub use types::{TurnError, TurnOutcome};

use sqlx::PgPool;

use crate::embedding::EmbeddingProvider;
use crate::llm::LlmProvider;

pub async fn handle_inbound_message(
    pool: &PgPool,
    provider: &dyn LlmProvider,
    embedding_provider: &dyn EmbeddingProvider,
    channel: &str,
    chat_type: &str,
    chat_id: &str,
    sender_channel_user_id: &str,
    text: &str,
) -> Result<TurnOutcome, TurnError> {
    let bootstrap::BootstrapResult { user_id, sender_channel_identity_id, session_id, .. } =
        bootstrap::bootstrap_identity_and_session(pool, channel, chat_type, chat_id, sender_channel_user_id)
            .await?;

    let mut conn = lock::acquire_session_lock(pool, session_id).await?;

    lock::insert_inbound_message(&mut conn, session_id, sender_channel_identity_id, text).await?;

    // Sub-agent routing is scaffolded but not yet implemented: nothing populates
    // agent_sessions in this plan, so every turn falls through to chitchat. See
    // docs/superpowers/specs/2026-07-26-orchestrator-turn-loop-design.md §2 step 5.
    let _active_agent_session_id =
        routing::find_active_agent_session(&mut conn, session_id, sender_channel_identity_id).await?;

    let result = chitchat::run_chitchat_turn(&mut conn, provider, embedding_provider, session_id, user_id, text).await;

    match result {
        Ok(reply) => {
            release_lock_ignoring_errors(&mut conn, session_id).await;
            Ok(TurnOutcome { session_id, reply })
        }
        Err(err) => {
            // Best-effort: a failed event write here must never mask the original error.
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

async fn release_lock_ignoring_errors(conn: &mut sqlx::pool::PoolConnection<sqlx::Postgres>, session_id: uuid::Uuid) {
    let _ = lock::release_session_lock(conn, session_id).await;
}
