use chrono::{DateTime, Utc};
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

/// Upserts `user_personality`, appends a new row to `user_personality_versions`, and records a
/// `PersonalityChanged` `agent_events` row with `{old_description, new_description, new_version}`
/// (`old_description` is `null` on a user's first-ever change) — all in one transaction.
/// `session_id`/`agent_session_id` are `None` for changes with no chat turn behind them (e.g. a
/// rollback triggered from the web UI) — `agent_events.session_id`/`.agent_session_id` are
/// already nullable columns for exactly this reason (see chitchat's own NULL-sentinel handling
/// in `nomi-turn`). The literal `"personality"` agent_type below is `nomi-agent-personality`'s
/// `PERSONALITY_AGENT_TYPE`, duplicated here because `nomi-agent-core` sits below
/// `nomi-agent-personality` in the dependency graph (the same convention every other agent_type
/// string in this crate already follows).
pub async fn set_personality(
    conn: &mut PoolConnection<Postgres>,
    session_id: Option<Uuid>,
    agent_session_id: Option<Uuid>,
    user_id: Uuid,
    new_description: &str,
) -> Result<(), TurnError> {
    let mut tx = conn.begin().await?;

    let old_description: Option<String> =
        sqlx::query_scalar("SELECT description FROM user_personality WHERE user_id = $1")
            .bind(user_id)
            .fetch_optional(&mut *tx)
            .await?;

    let max_version: Option<i32> =
        sqlx::query_scalar("SELECT MAX(version) FROM user_personality_versions WHERE user_id = $1")
            .bind(user_id)
            .fetch_one(&mut *tx)
            .await?;
    let new_version = max_version.unwrap_or(0) + 1;

    sqlx::query("INSERT INTO user_personality_versions (user_id, version, description) VALUES ($1, $2, $3)")
        .bind(user_id)
        .bind(new_version)
        .bind(new_description)
        .execute(&mut *tx)
        .await?;

    sqlx::query(
        "INSERT INTO user_personality (user_id, description, current_version, updated_at) \
         VALUES ($1, $2, $3, now()) \
         ON CONFLICT (user_id) DO UPDATE SET description = $2, current_version = $3, updated_at = now()",
    )
    .bind(user_id)
    .bind(new_description)
    .bind(new_version)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        "INSERT INTO agent_events (session_id, agent_session_id, agent_type, event_type, payload) \
         VALUES ($1, $2, 'personality', 'PersonalityChanged', $3)",
    )
    .bind(session_id)
    .bind(agent_session_id)
    .bind(serde_json::json!({
        "old_description": old_description,
        "new_description": new_description,
        "new_version": new_version,
    }))
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(())
}

#[derive(Debug, Clone, PartialEq)]
pub struct PersonalityVersion {
    pub version: i32,
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub is_current: bool,
}

/// Returns up to `limit` versions for `user_id`, most recent first. `is_current` is computed in
/// SQL via a join against `user_personality.current_version` — one query, no per-row lookup.
pub async fn list_versions(
    conn: &mut PoolConnection<Postgres>,
    user_id: Uuid,
    limit: i64,
) -> Result<Vec<PersonalityVersion>, TurnError> {
    let rows: Vec<(i32, String, DateTime<Utc>, bool)> = sqlx::query_as(
        "SELECT v.version, v.description, v.created_at, v.version = up.current_version AS is_current \
         FROM user_personality_versions v \
         JOIN user_personality up ON up.user_id = v.user_id \
         WHERE v.user_id = $1 \
         ORDER BY v.version DESC \
         LIMIT $2",
    )
    .bind(user_id)
    .bind(limit)
    .fetch_all(&mut **conn)
    .await?;

    Ok(rows
        .into_iter()
        .map(|(version, description, created_at, is_current)| PersonalityVersion {
            version,
            description,
            created_at,
            is_current,
        })
        .collect())
}

#[derive(Debug, thiserror::Error)]
pub enum RollbackError {
    #[error(transparent)]
    Db(#[from] sqlx::Error),
    #[error("personality version {0} not found for this user")]
    VersionNotFound(i32),
    #[error(transparent)]
    SetPersonality(#[from] TurnError),
}

/// Restores `user_id`'s personality to `target_version`'s description by creating a *new*
/// version with that description — never mutates or deletes `target_version`'s own row. The
/// lookup runs outside `set_personality`'s own transaction: version rows are immutable once
/// written, so a concurrent write racing this read can only mean the description copied forward
/// is still a genuine historical value, never a torn read.
pub async fn rollback_to_version(
    conn: &mut PoolConnection<Postgres>,
    session_id: Option<Uuid>,
    agent_session_id: Option<Uuid>,
    user_id: Uuid,
    target_version: i32,
) -> Result<(), RollbackError> {
    let target_description: Option<String> = sqlx::query_scalar(
        "SELECT description FROM user_personality_versions WHERE user_id = $1 AND version = $2",
    )
    .bind(user_id)
    .bind(target_version)
    .fetch_optional(&mut **conn)
    .await?;

    let target_description = target_description.ok_or(RollbackError::VersionNotFound(target_version))?;

    set_personality(conn, session_id, agent_session_id, user_id, &target_description).await?;

    Ok(())
}
