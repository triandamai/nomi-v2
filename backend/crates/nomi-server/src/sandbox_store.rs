//! The per-project sandbox rows: what each project wants its sandbox to be doing (`desired`),
//! what the provider last reported (`state`), and the retry bookkeeping the reconciler uses to
//! move one toward the other.

use chrono::{DateTime, Duration, Utc};
use sqlx::{PgConnection, Row};
use uuid::Uuid;

/// How long a claimed row is held back from other claimers, in seconds. It must outlast a whole
/// walk (a start can wait two minutes for the sandbox, then sync); the walk's outcome replaces it
/// with the real retry time.
const WALK_LEASE_SECS: i64 = 600;

/// What a project wants its sandbox to be doing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Desired {
    Running,
    Stopped,
}

impl Desired {
    pub fn as_str(self) -> &'static str {
        match self {
            Desired::Running => "running",
            Desired::Stopped => "stopped",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "running" => Some(Desired::Running),
            "stopped" => Some(Desired::Stopped),
            _ => None,
        }
    }
}

/// What the provider last reported about a project's sandbox.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    None,
    Creating,
    Starting,
    Running,
    Stopping,
    Stopped,
    Error,
    Gone,
}

impl State {
    pub fn as_str(self) -> &'static str {
        match self {
            State::None => "none",
            State::Creating => "creating",
            State::Starting => "starting",
            State::Running => "running",
            State::Stopping => "stopping",
            State::Stopped => "stopped",
            State::Error => "error",
            State::Gone => "gone",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "none" => Some(State::None),
            "creating" => Some(State::Creating),
            "starting" => Some(State::Starting),
            "running" => Some(State::Running),
            "stopping" => Some(State::Stopping),
            "stopped" => Some(State::Stopped),
            "error" => Some(State::Error),
            "gone" => Some(State::Gone),
            _ => None,
        }
    }

    /// Moving here ends any retrying: the sandbox is where it was last told to be.
    fn is_settled(self) -> bool {
        matches!(self, State::Running | State::Stopped | State::None)
    }
}

#[derive(Debug, Clone)]
pub struct SandboxRow {
    pub project_id: Uuid,
    pub provider: String,
    pub sandbox_id: Option<String>,
    pub desired: Desired,
    pub state: State,
    pub synced_version: i64,
    pub preview_url: Option<String>,
    pub attempts: i32,
}

fn decode<T>(what: &str, value: &str, parsed: Option<T>) -> Result<T, sqlx::Error> {
    parsed.ok_or_else(|| sqlx::Error::Decode(format!("unknown sandbox {what} {value:?}").into()))
}

/// Ask for the project's sandbox to be running, creating its row the first time.
pub async fn want_running(conn: &mut PgConnection, project_id: Uuid, provider: &str) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO project_sandboxes (project_id, provider, desired) VALUES ($1, $2, 'running')
         ON CONFLICT (project_id) DO UPDATE SET desired = 'running', updated_at = now()",
    )
    .bind(project_id)
    .bind(provider)
    .execute(conn)
    .await?;
    Ok(())
}

/// Ask for the project's sandbox to be stopped. Does nothing for a project with no sandbox.
pub async fn want_stopped(conn: &mut PgConnection, project_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE project_sandboxes SET desired = 'stopped', updated_at = now() WHERE project_id = $1")
        .bind(project_id)
        .execute(conn)
        .await?;
    Ok(())
}

/// Rows whose state hasn't reached what they want (or is mid-change) and whose retry time has
/// come, locked with `FOR UPDATE SKIP LOCKED` so two reconcilers never take the same one. The
/// locks last until the transaction ends, so the caller must run this inside a transaction and
/// keep it open while it works the rows. Claimed rows are leased: their `retry_at` moves
/// `WALK_LEASE_SECS` ahead so a later claim skips them until the walk records its outcome.
pub async fn claim_due(conn: &mut PgConnection, limit: i64) -> Result<Vec<SandboxRow>, sqlx::Error> {
    let rows = sqlx::query(
        "SELECT project_id, provider, sandbox_id, desired, state, synced_version, preview_url, attempts
         FROM project_sandboxes
         WHERE (desired = 'running' AND state <> 'running'
                OR desired = 'stopped' AND state NOT IN ('stopped', 'none', 'gone')
                OR state IN ('creating', 'starting', 'stopping'))
           AND (retry_at IS NULL OR retry_at <= now())
         ORDER BY updated_at
         LIMIT $1
         FOR UPDATE SKIP LOCKED",
    )
    .bind(limit)
    .fetch_all(&mut *conn)
    .await?;
    let claimed = rows.iter().map(row_from).collect::<Result<Vec<_>, _>>()?;
    let ids: Vec<Uuid> = claimed.iter().map(|r| r.project_id).collect();
    sqlx::query("UPDATE project_sandboxes SET retry_at = $2 WHERE project_id = ANY($1)")
        .bind(&ids)
        .bind(Utc::now() + Duration::seconds(WALK_LEASE_SECS))
        .execute(conn)
        .await?;
    Ok(claimed)
}

fn row_from(r: &sqlx::postgres::PgRow) -> Result<SandboxRow, sqlx::Error> {
    let desired: String = r.try_get("desired")?;
    let state: String = r.try_get("state")?;
    Ok(SandboxRow {
        project_id: r.try_get("project_id")?,
        provider: r.try_get("provider")?,
        sandbox_id: r.try_get("sandbox_id")?,
        desired: decode("desired", &desired, Desired::parse(&desired))?,
        state: decode("state", &state, State::parse(&state))?,
        synced_version: r.try_get("synced_version")?,
        preview_url: r.try_get("preview_url")?,
        attempts: r.try_get("attempts")?,
    })
}

/// One project's row as it is now, or `None` if it has no sandbox.
pub async fn get(conn: &mut PgConnection, project_id: Uuid) -> Result<Option<SandboxRow>, sqlx::Error> {
    let row = sqlx::query(
        "SELECT project_id, provider, sandbox_id, desired, state, synced_version, preview_url, attempts
         FROM project_sandboxes WHERE project_id = $1",
    )
    .bind(project_id)
    .fetch_optional(conn)
    .await?;
    row.as_ref().map(row_from).transpose()
}

/// Record the state the provider reported, with an error message if it gave one. Reaching a
/// settled state (running, stopped, none) resets the retry bookkeeping.
pub async fn set_state(conn: &mut PgConnection, project_id: Uuid, state: State, error: Option<&str>) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE project_sandboxes SET state = $2, last_error = $3, updated_at = now(),
                attempts = CASE WHEN $4 THEN 0 ELSE attempts END,
                retry_at = CASE WHEN $4 THEN NULL ELSE retry_at END
         WHERE project_id = $1",
    )
    .bind(project_id)
    .bind(state.as_str())
    .bind(error)
    .bind(state.is_settled())
    .execute(conn)
    .await?;
    Ok(())
}

/// A failed attempt: count it, say why, and hold the row back until `retry_at`.
pub async fn record_attempt_failure(conn: &mut PgConnection, project_id: Uuid, retry_at: DateTime<Utc>, error: &str) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE project_sandboxes SET attempts = attempts + 1, retry_at = $2, last_error = $3, updated_at = now()
         WHERE project_id = $1",
    )
    .bind(project_id)
    .bind(retry_at)
    .bind(error)
    .execute(conn)
    .await?;
    Ok(())
}

/// Hold the row back until `retry_at` without counting a failure: a wait, not a fault.
pub async fn defer(conn: &mut PgConnection, project_id: Uuid, retry_at: DateTime<Utc>) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE project_sandboxes SET retry_at = $2, updated_at = now() WHERE project_id = $1")
        .bind(project_id)
        .bind(retry_at)
        .execute(conn)
        .await?;
    Ok(())
}

/// Projects whose sandbox should be running but whose page hasn't checked in for `idle_secs`
/// and which have no command waiting or running.
pub async fn idle_candidates(conn: &mut PgConnection, idle_secs: u64) -> Result<Vec<Uuid>, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT s.project_id FROM project_sandboxes s JOIN projects p ON p.id = s.project_id
         WHERE s.desired = 'running'
           AND (p.runner_seen_at IS NULL OR p.runner_seen_at < now() - make_interval(secs => $1))
           AND NOT EXISTS (SELECT 1 FROM project_runs r WHERE r.project_id = s.project_id AND r.status IN ('pending', 'running'))",
    )
    .bind(idle_secs as f64)
    .fetch_all(conn)
    .await
}
