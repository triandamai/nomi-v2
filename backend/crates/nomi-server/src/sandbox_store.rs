//! The per-project sandbox rows: what each project wants its sandbox to be doing (`desired`),
//! what the provider last reported (`state`), and the retry bookkeeping the reconciler uses to
//! move one toward the other.

use std::time::Duration as StdDuration;

use chrono::{DateTime, Utc};
use nomi_sandbox::redact::redact_urls;
use sqlx::{PgConnection, Row};
use uuid::Uuid;

/// How long a claim, or a renewal of it, holds a row back from other claimers. A walk renews it
/// every `sandbox_manager::LEASE_RENEW_EVERY` for as long as it runs, and gives up the walk if it
/// can't (see `sandbox_manager::hold_lease`), so no single step has to fit inside it.
pub const WALK_LEASE: StdDuration = StdDuration::from_secs(5 * 60);

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
    /// Why the last attempt failed; on a running row with no preview, why the preview failed.
    pub last_error: Option<String>,
}

fn decode<T>(what: &str, value: &str, parsed: Option<T>) -> Result<T, sqlx::Error> {
    parsed.ok_or_else(|| sqlx::Error::Decode(format!("unknown sandbox {what} {value:?}").into()))
}

/// Ask for the project's sandbox to be running, creating its row the first time. Every call is a
/// sign of use (the page's heartbeat, a start, Koda waking it), so it also moves `last_active_at`,
/// which keeps the idle sweep away. Flipping a settled
/// row (none, stopped, running) from stopped also drops any leftover backoff so the person's open
/// isn't delayed; a repeat call, an errored row and an in-flight row keeps its `retry_at`; the walk lease is `leased_until`, which these never touch.
pub async fn want_running(conn: &mut PgConnection, project_id: Uuid, provider: &str) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO project_sandboxes (project_id, provider, desired, last_active_at) VALUES ($1, $2, 'running', now())
         ON CONFLICT (project_id) DO UPDATE SET desired = 'running', updated_at = now(), last_active_at = now(),
             retry_at = CASE WHEN project_sandboxes.desired = 'stopped' AND project_sandboxes.state IN ('none', 'stopped', 'running') THEN NULL ELSE project_sandboxes.retry_at END",
    )
    .bind(project_id)
    .bind(provider)
    .execute(conn)
    .await?;
    Ok(())
}

/// Ask for the project's sandbox to be stopped, clearing backoff on a settled row like
/// `want_running`, and on a row in `error`: its cooldown only holds back starts, and a sandbox
/// that may still be up must not wait it out before it is stopped. Does nothing for a project
/// with no sandbox.
pub async fn want_stopped(conn: &mut PgConnection, project_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE project_sandboxes SET desired = 'stopped', updated_at = now(),
             retry_at = CASE WHEN desired = 'running' AND state IN ('none', 'stopped', 'running', 'error') THEN NULL ELSE retry_at END
         WHERE project_id = $1",
    )
        .bind(project_id)
        .execute(conn)
        .await?;
    Ok(())
}

/// The admin switch is off: every sandbox that is wanted running is asked to stop, so the ones
/// that are up wind down and none is created or resumed. Returns how many rows changed. Like
/// `want_stopped`, the backoff of a settled or errored row is dropped.
pub async fn want_all_stopped(conn: &mut PgConnection) -> Result<u64, sqlx::Error> {
    let done = sqlx::query(
        "UPDATE project_sandboxes SET desired = 'stopped', updated_at = now(),
             retry_at = CASE WHEN state IN ('none', 'stopped', 'running', 'error') THEN NULL ELSE retry_at END
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
/// `WALK_LEASE` ahead so a later claim skips them until the walk records its outcome.
///
/// A running sandbox is also due when the project's files moved past what it holds
/// (`synced_version < projects.files_version`), so edits reach it, and when it runs without a
/// preview and no failure is recorded (the person asked to retry it). Such a row is left alone
/// while one of the project's commands is running in it (started within `run_lease`); the
/// command claim in turn waits for the walk's lease (`sandbox_runs::claim_next`), so a sync or
/// an install never overlaps a command.
pub async fn claim_due(conn: &mut PgConnection, limit: i64) -> Result<Vec<SandboxRow>, sqlx::Error> {
    claim_due_in(conn, limit, false).await
}

/// The columns `row_from` reads.
const ROW_COLUMNS: &str = "project_id, provider, sandbox_id, desired, state, synced_version, preview_url, attempts, last_error";

/// `claim_due`, optionally only the rows that want to be stopped (the admin switch is off): a row
/// that asks to be running between the switch going off and the claim is left alone.
pub async fn claim_due_in(conn: &mut PgConnection, limit: i64, stop_only: bool) -> Result<Vec<SandboxRow>, sqlx::Error> {
    let rows = sqlx::query(&format!(
        "SELECT {ROW_COLUMNS}
         FROM project_sandboxes s
         WHERE (desired = 'running' AND state <> 'running'
                OR desired = 'stopped' AND state NOT IN ('stopped', 'none', 'gone')
                OR state IN ('creating', 'starting', 'stopping')
                OR desired = 'running' AND state = 'running' AND sandbox_id IS NOT NULL
                   AND (synced_version < (SELECT p.files_version FROM projects p WHERE p.id = s.project_id)
                        OR preview_url IS NULL AND last_error IS NULL)
                   AND NOT EXISTS (SELECT 1 FROM project_runs r WHERE r.project_id = s.project_id AND r.status = 'running'
                                   AND r.started_at >= now() - make_interval(secs => $3)))
           AND (NOT $2 OR desired = 'stopped')
           AND (retry_at IS NULL OR retry_at <= now())
           AND (leased_until IS NULL OR leased_until <= now())
         ORDER BY updated_at
         LIMIT $1
         FOR UPDATE SKIP LOCKED"
    ))
    .bind(limit)
    .bind(stop_only)
    .bind(crate::sandbox_runs::RUN_LEASE.as_secs_f64())
    .fetch_all(&mut *conn)
    .await?;
    let claimed = rows.iter().map(row_from).collect::<Result<Vec<_>, _>>()?;
    let ids: Vec<Uuid> = claimed.iter().map(|r| r.project_id).collect();
    sqlx::query("UPDATE project_sandboxes SET leased_until = now() + make_interval(secs => $2) WHERE project_id = ANY($1)")
        .bind(&ids)
        .bind(WALK_LEASE.as_secs_f64())
        .execute(conn)
        .await?;
    Ok(claimed)
}

/// Let go of the walk lease taken by `claim_due`; the walk is over.
pub async fn release(conn: &mut PgConnection, project_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE project_sandboxes SET leased_until = NULL WHERE project_id = $1").bind(project_id).execute(conn).await?;
    Ok(())
}

/// Move a walk's lease another `WALK_LEASE` ahead; the walk is still going. Only a lease that is
/// still held is moved: a renewal still in flight when the walk ends must not bring back a lease
/// that `release` has already let go of.
pub async fn renew_lease(conn: &mut PgConnection, project_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE project_sandboxes SET leased_until = now() + make_interval(secs => $2) WHERE project_id = $1 AND leased_until IS NOT NULL")
        .bind(project_id)
        .bind(WALK_LEASE.as_secs_f64())
        .execute(conn)
        .await?;
    Ok(())
}

/// The person asked to try the preview of a running sandbox again: forget its recorded failure
/// so the next claim walks it (see `claim_due_in`). Does nothing to a row that has a preview or
/// isn't running.
pub async fn retry_preview(conn: &mut PgConnection, project_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE project_sandboxes SET last_error = NULL, retry_at = NULL, updated_at = now() WHERE project_id = $1 AND state = 'running' AND preview_url IS NULL")
        .bind(project_id)
        .execute(conn)
        .await?;
    Ok(())
}

/// A running sandbox the liveness pass looks at, and what else is due for it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AliveDue {
    pub project_id: Uuid,
    pub sandbox_id: String,
    /// It has a preview, so its dev server should answer.
    pub has_preview: bool,
    /// Its recorded auto-stop deadline is unknown or closer than the extension margin, and no
    /// backoff from a failed extension holds it.
    pub extend_auto_stop: bool,
    /// No dev server restart within the restart gap, so a dead one may be restarted now.
    pub may_restart_dev_server: bool,
}

/// Running sandboxes not asked about for `every_secs` (oldest first, at most `limit`) and not
/// being walked: the liveness pass asks the provider whether they are still up. Each says whether
/// its auto-stop is due within `extend_before_secs` and whether its dev server was restarted
/// within `restart_gap_secs`.
pub async fn alive_due(conn: &mut PgConnection, every_secs: f64, limit: i64, extend_before_secs: f64, restart_gap_secs: f64) -> Result<Vec<AliveDue>, sqlx::Error> {
    let rows: Vec<(Uuid, String, bool, bool, bool)> = sqlx::query_as(
        "SELECT project_id, sandbox_id, preview_url IS NOT NULL,
                (auto_stop_at IS NULL OR auto_stop_at < now() + make_interval(secs => $3))
                    AND (auto_stop_retry_at IS NULL OR auto_stop_retry_at <= now()),
                devserver_restarted_at IS NULL OR devserver_restarted_at < now() - make_interval(secs => $4)
         FROM project_sandboxes
         WHERE state = 'running' AND sandbox_id IS NOT NULL
           AND (leased_until IS NULL OR leased_until <= now())
           AND (alive_checked_at IS NULL OR alive_checked_at < now() - make_interval(secs => $1))
         ORDER BY alive_checked_at NULLS FIRST
         LIMIT $2",
    )
    .bind(every_secs)
    .bind(limit)
    .bind(extend_before_secs)
    .bind(restart_gap_secs)
    .fetch_all(conn)
    .await?;
    Ok(rows
        .into_iter()
        .map(|(project_id, sandbox_id, has_preview, extend_auto_stop, may_restart_dev_server)| AliveDue { project_id, sandbox_id, has_preview, extend_auto_stop, may_restart_dev_server })
        .collect())
}

/// The provider will stop this sandbox by itself at `deadline` (as it reported, or as asked on
/// create, resume or an extension): record it and forget any extension backoff, so the liveness
/// pass extends it before it comes.
pub async fn note_auto_stop(conn: &mut PgConnection, project_id: Uuid, sandbox_id: &str, deadline: DateTime<Utc>) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE project_sandboxes SET auto_stop_at = $3, auto_stop_failures = 0, auto_stop_retry_at = NULL WHERE project_id = $1 AND sandbox_id = $2")
        .bind(project_id)
        .bind(sandbox_id)
        .bind(deadline)
        .execute(conn)
        .await?;
    Ok(())
}

/// Failed auto-stop extensions in a row for this sandbox.
pub async fn auto_stop_failures(conn: &mut PgConnection, project_id: Uuid, sandbox_id: &str) -> Result<i32, sqlx::Error> {
    Ok(sqlx::query_scalar("SELECT auto_stop_failures FROM project_sandboxes WHERE project_id = $1 AND sandbox_id = $2")
        .bind(project_id)
        .bind(sandbox_id)
        .fetch_optional(conn)
        .await?
        .unwrap_or(0))
}

/// An auto-stop extension failed (or the provider's deadline didn't move): count it, hold the
/// next try back until `retry_at`, and save the deadline the provider reported, if it did.
pub async fn auto_stop_extension_failed(conn: &mut PgConnection, project_id: Uuid, sandbox_id: &str, retry_at: DateTime<Utc>, reported: Option<DateTime<Utc>>) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE project_sandboxes SET auto_stop_failures = auto_stop_failures + 1, auto_stop_retry_at = $3, auto_stop_at = COALESCE($4, auto_stop_at)
         WHERE project_id = $1 AND sandbox_id = $2",
    )
    .bind(project_id)
    .bind(sandbox_id)
    .bind(retry_at)
    .bind(reported)
    .execute(conn)
    .await?;
    Ok(())
}

/// The dev server of this running sandbox stopped answering: drop its preview (with no failure
/// recorded, so `claim_due_in` walks the row and starts the preview again) and record when and
/// why. Only if the row still records this sandbox running with a preview and no walk holds it.
/// Returns whether it changed.
pub async fn dev_server_down(conn: &mut PgConnection, project_id: Uuid, sandbox_id: &str, reason: &str) -> Result<bool, sqlx::Error> {
    let done = sqlx::query(
        "UPDATE project_sandboxes SET preview_url = NULL, last_error = NULL, retry_at = NULL, devserver_restarted_at = now(), devserver_restart_reason = $3, updated_at = now()
         WHERE project_id = $1 AND sandbox_id = $2 AND state = 'running' AND preview_url IS NOT NULL AND (leased_until IS NULL OR leased_until <= now())",
    )
    .bind(project_id)
    .bind(sandbox_id)
    .bind(reason)
    .execute(conn)
    .await?;
    Ok(done.rows_affected() > 0)
}

/// The provider says this running sandbox is still up.
pub async fn mark_alive(conn: &mut PgConnection, project_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE project_sandboxes SET alive_checked_at = now() WHERE project_id = $1").bind(project_id).execute(conn).await?;
    Ok(())
}

/// The provider says this sandbox is no longer running: the row becomes `stopped` (no preview) so
/// the next claim walks it again (resumes it, or finds it gone and makes a new one). Only if the
/// row still records this sandbox as running and no walk holds it. Returns whether it changed.
pub async fn mark_dead(conn: &mut PgConnection, project_id: Uuid, sandbox_id: &str) -> Result<bool, sqlx::Error> {
    let done = sqlx::query(
        "UPDATE project_sandboxes SET state = 'stopped', preview_url = NULL, attempts = 0, retry_at = NULL, alive_checked_at = now(), updated_at = now()
         WHERE project_id = $1 AND sandbox_id = $2 AND state = 'running' AND (leased_until IS NULL OR leased_until <= now())",
    )
    .bind(project_id)
    .bind(sandbox_id)
    .execute(conn)
    .await?;
    Ok(done.rows_affected() > 0)
}

/// Add to the end of the dev server's saved output, keeping at most `max_chars` characters.
pub async fn append_log(conn: &mut PgConnection, project_id: Uuid, text: &str, max_chars: usize) -> Result<(), sqlx::Error> {
    if text.is_empty() {
        return Ok(());
    }
    sqlx::query("UPDATE project_sandboxes SET log_tail = right(log_tail || $2, $3) WHERE project_id = $1")
        .bind(project_id)
        .bind(text)
        .bind(i32::try_from(max_chars).unwrap_or(i32::MAX))
        .execute(conn)
        .await?;
    Ok(())
}

/// A new dev server starts in this sandbox: forget the last one's output and remember the new
/// one's process, so the start after it can kill it (from any Nomi process).
pub async fn record_dev_server(conn: &mut PgConnection, project_id: Uuid, sandbox_id: &str, process_id: Option<&str>) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE project_sandboxes SET log_tail = '', devserver_process = $3 WHERE project_id = $1 AND sandbox_id = $2")
        .bind(project_id)
        .bind(sandbox_id)
        .bind(process_id)
        .execute(conn)
        .await?;
    Ok(())
}

/// The dev server process started last in this sandbox, if one was recorded.
pub async fn dev_server_process(conn: &mut PgConnection, project_id: Uuid, sandbox_id: &str) -> Result<Option<String>, sqlx::Error> {
    Ok(sqlx::query_scalar("SELECT devserver_process FROM project_sandboxes WHERE project_id = $1 AND sandbox_id = $2")
        .bind(project_id)
        .bind(sandbox_id)
        .fetch_optional(conn)
        .await?
        .flatten())
}

/// The dev server's saved output; empty when it never printed anything (or there is no sandbox).
pub async fn log_tail(conn: &mut PgConnection, project_id: Uuid) -> Result<String, sqlx::Error> {
    Ok(sqlx::query_scalar("SELECT log_tail FROM project_sandboxes WHERE project_id = $1").bind(project_id).fetch_optional(conn).await?.unwrap_or_default())
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
        last_error: r.try_get("last_error")?,
    })
}

/// One project's row as it is now, or `None` if it has no sandbox.
pub async fn get(conn: &mut PgConnection, project_id: Uuid) -> Result<Option<SandboxRow>, sqlx::Error> {
    let row = sqlx::query(&format!("SELECT {ROW_COLUMNS} FROM project_sandboxes WHERE project_id = $1"))
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

/// Projects whose sandbox should be running but that nobody has used for `idle_secs`: no
/// heartbeat from the sandbox page and no wake from Koda (`last_active_at`), no command waiting
/// or running, and none finished within `idle_secs` either, so a sandbox Koda woke with no page
/// open stays up while it works and for the same grace after its last command. Only the
/// sandbox's own commands count (`ran_in_sandbox`), and a waiting one only while no browser page
/// is open to claim it: commands the browser runtime runs never keep the sandbox awake.
pub async fn idle_candidates(conn: &mut PgConnection, idle_secs: u64) -> Result<Vec<Uuid>, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT s.project_id FROM project_sandboxes s
         WHERE s.desired = 'running'
           AND (s.last_active_at IS NULL OR s.last_active_at < now() - make_interval(secs => $1))
           AND NOT EXISTS (SELECT 1 FROM project_runs r JOIN projects p ON p.id = r.project_id WHERE r.project_id = s.project_id
                           AND (r.status = 'pending' AND NOT COALESCE(p.runner_seen_at > now() - make_interval(secs => $2), false)
                                OR r.ran_in_sandbox AND (r.status = 'running' OR r.finished_at >= now() - make_interval(secs => $1))))",
    )
    .bind(idle_secs as f64)
    .bind(crate::sandbox_runs::BROWSER_ONLINE_SECS)
    .fetch_all(conn)
    .await
}

/// What the project page shows about a project's sandbox.
#[derive(Debug, Clone)]
pub struct Status {
    pub desired: Desired,
    pub state: State,
    pub preview_url: Option<String>,
    pub last_error: Option<String>,
    /// Set while the row is held back, so a row in `error` with a future time is cooling down.
    pub retry_at: Option<DateTime<Utc>>,
}

/// Touch the last_active_at timestamp while the user is actively viewing/connected.
pub async fn touch_active(conn: &mut PgConnection, project_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE project_sandboxes SET last_active_at = now() WHERE project_id = $1 AND desired = 'running'")
        .bind(project_id)
        .execute(conn)
        .await?;
    Ok(())
}

/// The project's sandbox as the page should see it, or `None` if it has no sandbox.
pub async fn status(conn: &mut PgConnection, project_id: Uuid) -> Result<Option<Status>, sqlx::Error> {
    let row = sqlx::query("SELECT desired, state, preview_url, last_error, retry_at FROM project_sandboxes WHERE project_id = $1")
        .bind(project_id)
        .fetch_optional(conn)
        .await?;
    row.map(|r| {
        let desired: String = r.try_get("desired")?;
        let state: String = r.try_get("state")?;
        Ok(Status {
            desired: decode("desired", &desired, Desired::parse(&desired))?,
            state: decode("state", &state, State::parse(&state))?,
            preview_url: r.try_get("preview_url")?,
            last_error: r.try_get("last_error")?,
            retry_at: r.try_get("retry_at")?,
        })
    })
    .transpose()
}
