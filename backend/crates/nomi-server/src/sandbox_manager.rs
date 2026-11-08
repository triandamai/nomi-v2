//! Moves one project's sandbox toward what the project wants. `reconcile` looks at the row and
//! the provider, takes the next steps, and says whether it is done, should be tried again later,
//! or has failed. Every state change is written before the provider call that follows it, so a
//! crash in the middle picks the walk up again from the row.

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use nomi_sandbox::plan::{backoff, StartLimiter};
use nomi_sandbox::{CreateSpec, PreviewUrl, ResumeSpec, SandboxError, SandboxId, SandboxProvider, SandboxState};
use sqlx::PgPool;
use uuid::Uuid;

use crate::configuration::SandboxConfig;
use crate::sandbox_store::{self as store, Desired, SandboxRow, State};

/// How long a resumed or created sandbox may take to report running before the attempt is retried.
const RESUME_READY_TIMEOUT: Duration = Duration::from_secs(120);
/// How long to wait between checks while a sandbox is coming up.
const READY_POLL_INTERVAL: Duration = Duration::from_secs(1);
/// How long to wait before looking again at a sandbox that is still stopping.
const STOPPING_RECHECK: Duration = Duration::from_secs(5);
/// The first wait after a failed attempt; it doubles with each further failure.
const RETRY_BASE: Duration = Duration::from_secs(5);
/// The longest wait between attempts.
const RETRY_CAP: Duration = Duration::from_secs(900);
/// The label every sandbox carries, set to its project's id.
const PROJECT_LABEL: &str = "nomi-project";

/// What reconciling a row came to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReconcileOutcome {
    /// The sandbox is where the project wants it.
    Settled,
    /// Not there yet; try again at this time.
    RetryAt(DateTime<Utc>),
    /// The provider said no and trying again won't help. The row is held back, not dropped.
    Failed(String),
}

/// The project-specific work done once a sandbox is up. Syncing copies the project's files in and
/// returns the files version it copied; starting the preview runs the dev server and returns the
/// address it is served at.
#[async_trait]
pub trait ProjectSteps: Send + Sync {
    async fn sync(&self, project_id: Uuid, sandbox: &SandboxId) -> Result<i64, String>;
    async fn start_preview(&self, project_id: Uuid, sandbox: &SandboxId) -> Result<PreviewUrl, String>;
}

pub struct SandboxManager {
    provider: Arc<dyn SandboxProvider>,
    limiter: Mutex<StartLimiter>,
    pool: PgPool,
    steps: Arc<dyn ProjectSteps>,
    cfg: SandboxConfig,
}

/// An early exit from the walk: the outcome to report.
type Exit = ReconcileOutcome;

fn db<T>(result: Result<T, sqlx::Error>) -> Result<T, Exit> {
    result.map_err(|e| ReconcileOutcome::Failed(format!("database: {e}")))
}

impl SandboxManager {
    pub fn new(provider: Arc<dyn SandboxProvider>, steps: Arc<dyn ProjectSteps>, pool: PgPool, cfg: SandboxConfig) -> Self {
        let limiter = Mutex::new(StartLimiter::new(cfg.starts_per_minute, cfg.starts_per_hour));
        Self { provider, limiter, pool, steps, cfg }
    }

    /// Take the row toward what it wants. The row passed in only says which project to work on;
    /// the current row is read again so a stale copy never decides anything.
    pub async fn reconcile(&self, row: SandboxRow) -> ReconcileOutcome {
        let outcome = match self.walk(row.project_id).await {
            Ok(outcome) | Err(outcome) => outcome,
        };
        // The walk is over, whichever way it ended: let the next claim take the row.
        let _ = sqlx::query("UPDATE project_sandboxes SET leased_until = NULL WHERE project_id = $1").bind(row.project_id).execute(&self.pool).await;
        if outcome == ReconcileOutcome::Settled {
            // A settled row has nothing left to wait for.
            let _ = sqlx::query("UPDATE project_sandboxes SET retry_at = NULL WHERE project_id = $1").bind(row.project_id).execute(&self.pool).await;
        }
        outcome
    }

    async fn walk(&self, project_id: Uuid) -> Result<ReconcileOutcome, Exit> {
        let row = {
            let mut conn = db(self.pool.acquire().await)?;
            db(store::get(&mut conn, project_id).await)?
        };
        let Some(row) = row else { return Ok(ReconcileOutcome::Settled) };
        match row.desired {
            Desired::Running => self.bring_up(row).await,
            Desired::Stopped => self.bring_down(row).await,
        }
    }

    async fn set_state(&self, project_id: Uuid, state: State) -> Result<(), Exit> {
        let mut conn = db(self.pool.acquire().await)?;
        db(store::set_state(&mut conn, project_id, state, None).await)?;
        announce(project_id, state);
        Ok(())
    }

    /// Hold the row back until `at`, saying why.
    async fn retry_at(&self, project_id: Uuid, at: DateTime<Utc>, why: &str) -> Exit {
        let result = async {
            let mut conn = self.pool.acquire().await?;
            store::record_attempt_failure(&mut conn, project_id, at, why).await
        }
        .await;
        match result {
            Ok(()) => ReconcileOutcome::RetryAt(at),
            Err(e) => ReconcileOutcome::Failed(format!("database: {e}")),
        }
    }

    /// Wait until `at` without counting a failure.
    async fn defer(&self, project_id: Uuid, at: DateTime<Utc>) -> Exit {
        let result = async {
            let mut conn = self.pool.acquire().await?;
            store::defer(&mut conn, project_id, at).await
        }
        .await;
        match result {
            Ok(()) => ReconcileOutcome::RetryAt(at),
            Err(e) => ReconcileOutcome::Failed(format!("database: {e}")),
        }
    }

    /// How long to wait after a failure: what the provider asked for, or a growing delay.
    fn wait_after(attempts: i32, err: Option<&SandboxError>) -> Duration {
        err.and_then(SandboxError::retry_after).unwrap_or_else(|| backoff(attempts.max(0) as u32, RETRY_BASE, RETRY_CAP))
    }

    /// A provider call failed: retry later if that can help, otherwise mark the row as errored
    /// (still held back, so the worker doesn't spin on it).
    async fn provider_failed(&self, row: &SandboxRow, err: SandboxError) -> Exit {
        let at = Utc::now() + Self::wait_after(row.attempts, Some(&err));
        let why = err.to_string();
        if err.is_retryable() {
            return self.defer(row.project_id, at).await;
        }
        // The state, the reason and the retry time are saved together, and the pages told only
        // after: a page that reloads on the event must see the error message too.
        let project_id = row.project_id;
        let saved = async {
            let mut tx = self.pool.begin().await?;
            store::set_state(&mut tx, project_id, State::Error, Some(&why)).await?;
            store::record_attempt_failure(&mut tx, project_id, at, &why).await?;
            tx.commit().await
        }
        .await;
        match saved {
            Ok(()) => {
                announce(project_id, State::Error);
                ReconcileOutcome::Failed(why)
            }
            Err(e) => ReconcileOutcome::Failed(format!("database: {e}")),
        }
    }

    /// A project step failed; the sandbox is fine, so try the step again later.
    async fn step_failed(&self, row: &SandboxRow, why: String) -> Exit {
        let at = Utc::now() + Self::wait_after(row.attempts, None);
        self.retry_at(row.project_id, at, &why).await
    }

    /// Wait for the next start the limiter allows. Creating and resuming both count as starts.
    async fn take_start(&self, row: &SandboxRow) -> Result<(), Exit> {
        let verdict = self.limiter.lock().unwrap_or_else(|e| e.into_inner()).try_acquire(Instant::now());
        match verdict {
            Ok(()) => Ok(()),
            Err(wait) => Err(self.defer(row.project_id, Utc::now() + wait).await),
        }
    }

    /// Forget the provider's sandbox: it is gone, or can't be started again.
    async fn forget(&self, project_id: Uuid) -> Result<(), Exit> {
        self.forget_keeping(project_id, 0).await
    }

    /// Like `forget`, but the row keeps `attempts` failed attempts (reaching `none` would reset
    /// them), so a sandbox that keeps dying while it starts backs off further each time.
    async fn forget_keeping(&self, project_id: Uuid, attempts: i32) -> Result<(), Exit> {
        let result = async {
            let mut tx = self.pool.begin().await?;
            sqlx::query(
                "UPDATE project_sandboxes SET sandbox_id = NULL, synced_version = 0, synced_manifest = '{}'::jsonb, deps_hash = NULL, preview_url = NULL WHERE project_id = $1",
            )
            .bind(project_id)
            .execute(&mut *tx)
            .await?;
            store::set_state(&mut tx, project_id, State::None, None).await?;
            sqlx::query("UPDATE project_sandboxes SET attempts = $2 WHERE project_id = $1").bind(project_id).bind(attempts).execute(&mut *tx).await?;
            if close_session(&mut tx, project_id).await? == 0 {
                // Nothing was open, so a sandbox that never ran leaves no trace in the count the
                // next key is built from. A zero-length closed session advances it.
                sqlx::query("INSERT INTO sandbox_sessions (project_id, started_at, stopped_at) VALUES ($1, now(), now())")
                    .bind(project_id)
                    .execute(&mut *tx)
                    .await?;
            }
            tx.commit().await
        }
        .await;
        db(result)?;
        announce(project_id, State::None);
        Ok(())
    }

    /// The number of sandboxes this project has had running. A create that is retried before the
    /// sandbox ever ran gets the same number (and so the same key, and the same sandbox); once a
    /// sandbox has run, the next one gets a new number.
    async fn generation(&self, project_id: Uuid) -> Result<i64, Exit> {
        db(sqlx::query_scalar("SELECT count(*) FROM sandbox_sessions WHERE project_id = $1").bind(project_id).fetch_one(&self.pool).await)
    }

    async fn create(&self, row: &SandboxRow) -> Result<SandboxId, Exit> {
        self.take_start(row).await?;
        self.set_state(row.project_id, State::Creating).await?;
        let n = self.generation(row.project_id).await?;
        let spec = CreateSpec {
            size: self.cfg.size,
            ttl_seconds: None,
            from_snapshot: None,
            idempotency_key: format!("nomi-project-{}-{n}", row.project_id),
            labels: BTreeMap::from([(PROJECT_LABEL.to_string(), row.project_id.to_string())]),
        };
        let info = match self.provider.create(spec).await {
            Ok(info) => info,
            Err(e) => return Err(self.provider_failed(row, e).await),
        };
        // Another walk may have recorded a sandbox since this one began; theirs stands.
        let written = db(sqlx::query("UPDATE project_sandboxes SET sandbox_id = $2 WHERE project_id = $1 AND (sandbox_id IS NULL OR sandbox_id = $2)")
            .bind(row.project_id)
            .bind(&info.id.0)
            .execute(&self.pool)
            .await)?;
        if written.rows_affected() == 0 {
            let _ = self.provider.delete(&info.id).await;
            return Err(self.defer(row.project_id, Utc::now() + STOPPING_RECHECK).await);
        }
        Ok(info.id)
    }

    /// Poll the provider until the sandbox reports running, or the wait runs out.
    async fn wait_ready(&self, row: &SandboxRow, id: &SandboxId) -> Result<(), Exit> {
        let started = Instant::now();
        loop {
            match self.provider.get(id).await {
                Ok(info) if info.state == SandboxState::Running => return Ok(()),
                Ok(info) if matches!(info.state, SandboxState::Gone | SandboxState::Error) => {
                    self.forget_keeping(row.project_id, row.attempts).await?;
                    let at = Utc::now() + Self::wait_after(row.attempts, None);
                    return Err(self.retry_at(row.project_id, at, "sandbox went away while starting").await);
                }
                Ok(_) => {}
                Err(e) => return Err(self.provider_failed(row, e).await),
            }
            if started.elapsed() >= RESUME_READY_TIMEOUT {
                let at = Utc::now() + Self::wait_after(row.attempts, None);
                return Err(self.retry_at(row.project_id, at, "sandbox did not become ready in time").await);
            }
            tokio::time::sleep(READY_POLL_INTERVAL).await;
        }
    }

    /// The provider's sandbox is gone. One that never got past starting keeps its attempt count
    /// and backs off, so a sandbox that keeps dying doesn't loop; one that had been healthy is
    /// recreated at once with a fresh count.
    async fn vanished(&self, row: &SandboxRow) -> Result<ReconcileOutcome, Exit> {
        if matches!(row.state, State::Creating | State::Starting) {
            self.forget_keeping(row.project_id, row.attempts).await?;
            let at = Utc::now() + Self::wait_after(row.attempts + 1, None);
            return Ok(self.retry_at(row.project_id, at, "sandbox went away while starting").await);
        }
        self.forget(row.project_id).await?;
        Ok(self.defer(row.project_id, Utc::now()).await)
    }

    async fn bring_up(&self, row: SandboxRow) -> Result<ReconcileOutcome, Exit> {
        let project_id = row.project_id;
        let mut id = row.sandbox_id.clone().map(SandboxId);
        // A sandbox we already have: find out where it is.
        if let Some(existing) = id.clone() {
            match self.provider.get(&existing).await {
                Ok(info) => match info.state {
                    SandboxState::Running if row.state == State::Running => return Ok(ReconcileOutcome::Settled),
                    SandboxState::Gone | SandboxState::Error => {
                        return self.vanished(&row).await;
                    }
                    SandboxState::Stopping => {
                        return Ok(self.defer(project_id, Utc::now() + STOPPING_RECHECK).await);
                    }
                    SandboxState::Stopped => {
                        self.take_start(&row).await?;
                        self.set_state(project_id, State::Starting).await?;
                        match self.provider.resume(&existing, ResumeSpec::default()).await {
                            Ok(_) => {}
                            Err(SandboxError::SnapshotExpired) => id = self.start_over(&existing, &row).await?,
                            Err(e) => return Ok(self.provider_failed(&row, e).await),
                        }
                    }
                    SandboxState::Starting | SandboxState::Running => {}
                },
                Err(SandboxError::NotFound) => return self.vanished(&row).await,
                Err(SandboxError::SnapshotExpired) => id = self.start_over(&existing, &row).await?,
                Err(e) => return Ok(self.provider_failed(&row, e).await),
            }
        }
        let id = match id {
            Some(id) => id,
            None => self.create(&row).await?,
        };
        self.set_state(project_id, State::Starting).await?;
        self.wait_ready(&row, &id).await?;

        let version = match self.steps.sync(project_id, &id).await {
            Ok(version) => version,
            Err(why) => return Ok(self.step_failed(&row, why).await),
        };
        let preview = match self.steps.start_preview(project_id, &id).await {
            Ok(preview) => preview,
            Err(why) => return Ok(self.step_failed(&row, why).await),
        };
        let size = self.cfg.size.as_api();
        let result = async {
            let mut tx = self.pool.begin().await?;
            sqlx::query("UPDATE project_sandboxes SET synced_version = $2, preview_url = $3, last_active_at = now() WHERE project_id = $1")
                .bind(project_id)
                .bind(version)
                .bind(&preview.url)
                .execute(&mut *tx)
                .await?;
            store::set_state(&mut tx, project_id, State::Running, None).await?;
            // The unique index keeps a racing walk from opening a second session.
            sqlx::query(
                "INSERT INTO sandbox_sessions (project_id, size) VALUES ($1, $2)
                 ON CONFLICT (project_id) WHERE stopped_at IS NULL DO NOTHING",
            )
            .bind(project_id)
            .bind(size)
            .execute(&mut *tx)
            .await?;
            tx.commit().await
        }
        .await;
        db(result)?;
        announce(project_id, State::Running);
        Ok(ReconcileOutcome::Settled)
    }

    /// The sandbox can't be started again (its snapshot expired): drop it and make a new one.
    async fn start_over(&self, old: &SandboxId, row: &SandboxRow) -> Result<Option<SandboxId>, Exit> {
        // Best effort: the old one is useless either way, and a failure here must not stop the new one.
        let _ = self.provider.delete(old).await;
        self.forget(row.project_id).await?;
        Ok(Some(self.create(row).await?))
    }

    async fn bring_down(&self, row: SandboxRow) -> Result<ReconcileOutcome, Exit> {
        let project_id = row.project_id;
        let Some(raw) = row.sandbox_id.clone() else {
            self.forget(project_id).await?;
            return Ok(ReconcileOutcome::Settled);
        };
        self.set_state(project_id, State::Stopping).await?;
        match self.provider.stop(&SandboxId(raw), false).await {
            Ok(()) => {}
            Err(SandboxError::NotFound) => {
                self.forget(project_id).await?;
                return Ok(ReconcileOutcome::Settled);
            }
            // Refused or unreachable: try the same polite stop again later rather than forcing it.
            Err(SandboxError::Refused(why)) => {
                let at = Utc::now() + Self::wait_after(row.attempts, None);
                return Ok(self.retry_at(project_id, at, &format!("stop refused: {why}")).await);
            }
            Err(e) => return Ok(self.provider_failed(&row, e).await),
        }
        let result = async {
            let mut tx = self.pool.begin().await?;
            store::set_state(&mut tx, project_id, State::Stopped, None).await?;
            close_session(&mut tx, project_id).await?;
            tx.commit().await
        }
        .await;
        db(result)?;
        announce(project_id, State::Stopped);
        Ok(ReconcileOutcome::Settled)
    }
}

/// Tell the project's open pages the sandbox is now in `state`. Called once the change is saved
/// (after the commit, where the change was made in a transaction) so a page that reloads on the
/// event sees it. Every state this module writes goes through here.
fn announce(project_id: Uuid, state: State) {
    crate::project_events::sandbox_changed(project_id, state.phase());
}

/// End the project's open metering session, if it has one.
async fn close_session(conn: &mut sqlx::PgConnection, project_id: Uuid) -> Result<u64, sqlx::Error> {
    let done = sqlx::query("UPDATE sandbox_sessions SET stopped_at = now() WHERE project_id = $1 AND stopped_at IS NULL")
        .bind(project_id)
        .execute(conn)
        .await?;
    Ok(done.rows_affected())
}
