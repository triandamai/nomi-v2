//! The engine side of stopping agents: a running turn checks between steps whether the user
//! has asked to stop it since it began. The requests themselves are written by
//! `nomi-agent-supervisor`'s stop module.

use chrono::{DateTime, Utc};
use sqlx::pool::PoolConnection;
use sqlx::Postgres;
use uuid::Uuid;

/// The database's clock, so a turn's start time compares cleanly with `requested_at`
/// (`clock_timestamp()`, not `now()`, which is frozen for the length of a transaction).
pub async fn database_clock(conn: &mut PoolConnection<Postgres>) -> Result<DateTime<Utc>, sqlx::Error> {
    sqlx::query_scalar("SELECT clock_timestamp()").fetch_one(&mut **conn).await
}

/// Whether a stop covering this turn was requested at or after `since`. A failed check reads
/// as "not stopped": a stop is a user convenience, never a reason to fail the turn.
pub async fn is_stop_requested(
    conn: &mut PoolConnection<Postgres>,
    user_id: Uuid,
    session_id: Uuid,
    agent_type: &str,
    since: DateTime<Utc>,
) -> bool {
    sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM agent_stop_requests \
         WHERE user_id = $1 AND (session_id IS NULL OR session_id = $2) \
           AND (agent_type IS NULL OR agent_type = $3) \
           AND (exempt_agent_type IS NULL OR exempt_agent_type <> $3) AND requested_at >= $4)",
    )
    .bind(user_id)
    .bind(session_id)
    .bind(agent_type)
    .bind(since)
    .fetch_one(&mut **conn)
    .await
    .unwrap_or(false)
}
