use sqlx::pool::PoolConnection;
use sqlx::{Acquire, Postgres};
use uuid::Uuid;

use crate::error::TurnError;

pub async fn get_current_personality(conn: &mut PoolConnection<Postgres>, user_id: Uuid) -> Option<String> {
    sqlx::query_scalar("SELECT description FROM user_personality WHERE user_id = $1")
        .bind(user_id)
        .fetch_optional(&mut **conn)
        .await
        .ok()
        .flatten()
}

/// Upserts `user_personality` and records a `PersonalityChanged` `agent_events` row with
/// `{old_description, new_description}` (`old_description` is `null` on a user's first-ever
/// change) — both in one transaction. The literal `"personality"` agent_type below is
/// `nomi-agent-personality`'s `PERSONALITY_AGENT_TYPE`, duplicated here because
/// `nomi-agent-core` sits below `nomi-agent-personality` in the dependency graph (the same
/// convention every other agent_type string in this crate already follows).
pub async fn set_personality(
    conn: &mut PoolConnection<Postgres>,
    session_id: Uuid,
    agent_session_id: Uuid,
    user_id: Uuid,
    new_description: &str,
) -> Result<(), TurnError> {
    let mut tx = conn.begin().await?;

    let old_description: Option<String> =
        sqlx::query_scalar("SELECT description FROM user_personality WHERE user_id = $1")
            .bind(user_id)
            .fetch_optional(&mut *tx)
            .await?;

    sqlx::query(
        "INSERT INTO user_personality (user_id, description, updated_at) VALUES ($1, $2, now()) \
         ON CONFLICT (user_id) DO UPDATE SET description = $2, updated_at = now()",
    )
    .bind(user_id)
    .bind(new_description)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        "INSERT INTO agent_events (session_id, agent_session_id, agent_type, event_type, payload) \
         VALUES ($1, $2, 'personality', 'PersonalityChanged', $3)",
    )
    .bind(session_id)
    .bind(agent_session_id)
    .bind(serde_json::json!({"old_description": old_description, "new_description": new_description}))
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(())
}
