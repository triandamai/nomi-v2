//! The per-project sandbox rows: what each project wants its sandbox to be doing (`desired`),
//! what the provider last reported (`state`), and the retry bookkeeping the reconciler uses to
//! move one toward the other.

use chrono::{DateTime, Duration, Utc};
use nomi_sandbox::redact::redact_urls;
use sqlx::{PgConnection, Row};
use uuid::Uuid;

/// How long a claimed row is held back from other claimers, in seconds. It must outlast a whole
/// walk (a start can wait two minutes for the sandbox, then sync); the walk's outcome replaces it
/// with the real retry time.
const WALK_LEASE_SECS: i64 = 600;

/// No sandbox exists (or it was dropped); opening the project will start one.
pub const PHASE_IDLE: &str = "idle";
/// The sandbox is being created or resumed.
pub const PHASE_WAKING: &str = "waking";
/// The sandbox is up and serving the preview.
pub const PHASE_RUNNING: &str = "running";
/// The sandbox is being stopped.
pub const PHASE_STOPPING: &str = "stopping";
/// The sandbox is stopped and can be resumed.
pub const PHASE_STOPPED: &str = "stopped";
/// The provider failed; the row is backing off before it tries again.
pub const PHASE_ERROR: &str = "error";

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

    /// The stable word the project page and its realtime events use for this state.
    pub fn phase(self) -> &'static str {
        match self {
            State::None | State::Gone => PHASE_IDLE,
            State::Creating | State::Starting => PHASE_WAKING,
            State::Running => PHASE_RUNNING,
            State::Stopping => PHASE_STOPPING,
            State::Stopped => PHASE_STOPPED,
            State::Error => PHASE_ERROR,
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

/// Ask for the project's sandbox to be running, creating its row the first time. Flipping a settled
/// row (none, stopped, running) from stopped also drops any leftover backoff so the person's open
/// isn't delayed; a repeat call, an errored row and an in-flight row keeps its `retry_at`; the walk lease is `leased_until`, which these never touch.
pub async fn want_running(conn: &mut PgConnection, project_id: Uuid, provider: &str) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO project_sandboxes (project_id, provider, desired) VALUES ($1, $2, 'running')
         ON CONFLICT (project_id) DO UPDATE SET desired = 'running', updated_at = now(),
             retry_at = CASE WHEN project_sandboxes.desired = 'stopped' AND project_sandboxes.state IN ('none', 'stopped', 'running') THEN NULL ELSE project_sandboxes.retry_at END",
    )
    .bind(project_id)
    .bind(provider)
    .execute(conn)
    .await?;
    Ok(())
}

/// Ask for the project's sandbox to be stopped, clearing backoff on a settled row like
/// `want_running`. Does nothing for a project with no sandbox.
pub async fn want_stopped(conn: &mut PgConnection, project_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE project_sandboxes SET desired = 'stopped', updated_at = now(),
             retry_at = CASE WHEN desired = 'running' AND state IN ('none', 'stopped', 'running') THEN NULL ELSE retry_at END
         WHERE project_id = $1",
    )
        .bind(project_id)
        .execute(conn)
        .await?;
    Ok(())
}

/// The admin switch is off: every sandbox that is wanted running is asked to stop, so the ones
/// that are up wind down and none is created or resumed. Returns how many rows changed. Like
/// `want_stopped`, a settled row's backoff is dropped.
pub async fn want_all_stopped(conn: &mut PgConnection) -> Result<u64, sqlx::Error> {
    let done = sqlx::query(
        "UPDATE project_sandboxes SET desired = 'stopped', updated_at = now(),
             retry_at = CASE WHEN state IN ('none', 'stopped', 'running') THEN NULL ELSE retry_at END
         WHERE desired = 'running'",
    )
    .execute(conn)
    .await?;
    Ok(done.rows_affected())
}

/// Rows whose state hasn't reached what they want (or is mid-change) and whose retry time has
/// come, locked with `FOR UPDATE SKIP LOCKED` so two reconcilers never take the same one. The
/// locks last until the transaction ends, so the caller must run this inside a transaction and
/// keep it open while it works the rows. Claimed rows are leased: their `leased_until` moves
/// `WALK_LEASE_SECS` ahead so a later claim skips them until the walk records its outcome.
pub async fn claim_due(conn: &mut PgConnection, limit: i64) -> Result<Vec<SandboxRow>, sqlx::Error> {
    let rows = sqlx::query(
        "SELECT project_id, provider, sandbox_id, desired, state, synced_version, preview_url, attempts
         FROM project_sandboxes
         WHERE (desired = 'running' AND state <> 'running'
                OR desired = 'stopped' AND state NOT IN ('stopped', 'none', 'gone')
                OR state IN ('creating', 'starting', 'stopping'))
           AND (retry_at IS NULL OR retry_at <= now())
           AND (leased_until IS NULL OR leased_until <= now())
         ORDER BY updated_at
         LIMIT $1
         FOR UPDATE SKIP LOCKED",
    )
    .bind(limit)
    .fetch_all(&mut *conn)
    .await?;
    let claimed = rows.iter().map(row_from).collect::<Result<Vec<_>, _>>()?;
    let ids: Vec<Uuid> = claimed.iter().map(|r| r.project_id).collect();
    sqlx::query("UPDATE project_sandboxes SET leased_until = $2 WHERE project_id = ANY($1)")
        .bind(&ids)
        .bind(Utc::now() + Duration::seconds(WALK_LEASE_SECS))
        .execute(conn)
        .await?;
    Ok(claimed)
}

/// Let go of the walk lease taken by `claim_due`; the walk is over.
pub async fn release(conn: &mut PgConnection, project_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE project_sandboxes SET leased_until = NULL WHERE project_id = $1").bind(project_id).execute(conn).await?;
    Ok(())
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
///
/// Every saved error goes through `redact_urls` (here and in the two writers below): project
/// members read `last_error`, so a preview token or signed URL must never be stored in it.
pub async fn set_state(conn: &mut PgConnection, project_id: Uuid, state: State, error: Option<&str>) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE project_sandboxes SET state = $2, last_error = $3, updated_at = now(),
                attempts = CASE WHEN $4 THEN 0 ELSE attempts END,
                retry_at = CASE WHEN $4 THEN NULL ELSE retry_at END
         WHERE project_id = $1",
    )
    .bind(project_id)
    .bind(state.as_str())
    .bind(error.map(redact_urls))
    .bind(state.is_settled())
    .execute(conn)
    .await?;
    Ok(())
}

/// A failed attempt: count it, say why, and hold the row back until `retry_at`. Returns how many
/// failed attempts the row now has.
pub async fn record_attempt_failure(conn: &mut PgConnection, project_id: Uuid, retry_at: DateTime<Utc>, error: &str) -> Result<i32, sqlx::Error> {
    sqlx::query_scalar(
        "UPDATE project_sandboxes SET attempts = attempts + 1, retry_at = $2, last_error = $3, updated_at = now()
         WHERE project_id = $1 RETURNING attempts",
    )
    .bind(project_id)
    .bind(retry_at)
    .bind(redact_urls(error))
    .fetch_one(conn)
    .await
}

/// Too many real failures in a row: put the row in `error` and hold it back until `until`, with
/// a fresh attempt count for when the cooldown is over. The page falls back to the browser while
/// the cooldown lasts.
pub async fn enter_cooldown(conn: &mut PgConnection, project_id: Uuid, until: DateTime<Utc>, error: &str) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE project_sandboxes SET state = 'error', attempts = 0, retry_at = $2, last_error = $3, updated_at = now() WHERE project_id = $1")
        .bind(project_id)
        .bind(until)
        .bind(redact_urls(error))
        .execute(conn)
        .await?;
    Ok(())
}

/// Hand back rows stuck in `creating`, `starting` or `stopping` for longer than `stuck_secs`
/// with no live walk lease: they become due at once, without counting an attempt, and the lease
/// is cleared. Returns the projects moved.
pub async fn release_stuck(conn: &mut PgConnection, stuck_secs: i64) -> Result<Vec<Uuid>, sqlx::Error> {
    sqlx::query_scalar(
        "UPDATE project_sandboxes SET retry_at = now(), leased_until = NULL, updated_at = now()
         WHERE state IN ('creating', 'starting', 'stopping')
           AND updated_at < now() - make_interval(secs => $1)
           AND (leased_until IS NULL OR leased_until <= now())
         RETURNING project_id",
    )
    .bind(stuck_secs as f64)
    .fetch_all(conn)
    .await
}

/// Projects whose sandbox has sat stopped (and wanted stopped) for longer than `secs`.
pub async fn long_stopped(conn: &mut PgConnection, secs: i64) -> Result<Vec<(Uuid, String)>, sqlx::Error> {
    sqlx::query_as(
        "SELECT project_id, sandbox_id FROM project_sandboxes
         WHERE state = 'stopped' AND desired = 'stopped' AND sandbox_id IS NOT NULL
           AND updated_at < now() - make_interval(secs => $1)
           AND (leased_until IS NULL OR leased_until <= now())",
    )
    .bind(secs as f64)
    .fetch_all(conn)
    .await
}

/// The provider's copy of a long-stopped sandbox is deleted: mark the row `gone` and drop what
/// was synced, so the next start builds a new sandbox from the project's files. Does nothing if
/// the row has moved on since it was read. Returns whether the row was changed.
pub async fn mark_expired(conn: &mut PgConnection, project_id: Uuid, sandbox_id: &str) -> Result<bool, sqlx::Error> {
    let done = sqlx::query(
        "UPDATE project_sandboxes SET state = 'gone', sandbox_id = NULL, synced_version = 0, synced_manifest = '{}'::jsonb, deps_hash = NULL, preview_url = NULL, updated_at = now()
         WHERE project_id = $1 AND state = 'stopped' AND desired = 'stopped' AND sandbox_id = $2",
    )
    .bind(project_id)
    .bind(sandbox_id)
    .execute(conn)
    .await?;
    Ok(done.rows_affected() > 0)
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

/// What the project page shows about a project's sandbox.
#[derive(Debug, Clone)]
pub struct Status {
    pub state: State,
    pub preview_url: Option<String>,
    pub last_error: Option<String>,
    /// Set while the row is held back, so a row in `error` with a future time is cooling down.
    pub retry_at: Option<DateTime<Utc>>,
}

/// The project's sandbox as the page should see it, or `None` if it has no sandbox.
pub async fn status(conn: &mut PgConnection, project_id: Uuid) -> Result<Option<Status>, sqlx::Error> {
    let row = sqlx::query("SELECT state, preview_url, last_error, retry_at FROM project_sandboxes WHERE project_id = $1")
        .bind(project_id)
        .fetch_optional(conn)
        .await?;
    row.map(|r| {
        let state: String = r.try_get("state")?;
        Ok(Status {
            state: decode("state", &state, State::parse(&state))?,
            preview_url: r.try_get("preview_url")?,
            last_error: r.try_get("last_error")?,
            retry_at: r.try_get("retry_at")?,
        })
    })
    .transpose()
}
