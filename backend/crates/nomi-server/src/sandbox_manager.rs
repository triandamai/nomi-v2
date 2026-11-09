//! Moves one project's sandbox toward what the project wants. `reconcile` looks at the row and
//! the provider, takes the next steps, and says whether it is done, should be tried again later,
//! or has failed. Every state change is written before the provider call that follows it, so a
//! crash in the middle picks the walk up again from the row.

use std::collections::{BTreeMap, HashMap};
use std::future::Future;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use nomi_sandbox::plan::{backoff, StartLimiter};
use nomi_sandbox::{CreateSpec, PreviewUrl, ResumeSpec, SandboxError, SandboxId, SandboxProvider, SandboxState, Ttl};
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
pub(crate) const RETRY_BASE: Duration = Duration::from_secs(5);
/// The longest wait between attempts.
pub(crate) const RETRY_CAP: Duration = Duration::from_secs(900);
/// Real failures in a row after which the row goes to `error` and the page falls back to the
/// browser. Waits and rate limits never count.
pub const MAX_ATTEMPTS: i32 = 5;
/// How long a row stays in `error` after `MAX_ATTEMPTS` failures before it is tried again.
pub const PROVIDER_COOLDOWN: Duration = Duration::from_secs(30 * 60);
/// How long a row may sit in `creating`, `starting` or `stopping` with no walk holding it before
/// the watchdog hands it back. Longer than the walk lease, so a live walk is never disturbed.
pub const STUCK_AFTER: Duration = Duration::from_secs(20 * 60);
/// How long a sandbox may sit stopped before it is deleted at the provider, which keeps stopped
/// disks for at least 14 days; the next start rebuilds from the project's files.
pub const EXPIRY_WARN: Duration = Duration::from_secs(10 * 24 * 60 * 60);
/// The label every sandbox carries, set to its project's id.
pub(crate) const PROJECT_LABEL: &str = "nomi-project";
/// How long the provider lets a sandbox run before stopping it by itself, asked on every create,
/// resume and extension. Nomi's idle sweep stops sandboxes long before; this bounds every leak
/// that doesn't depend on Nomi being healthy (an outage, a lost deletion, a dry-run reaper).
pub const SANDBOX_AUTO_STOP: Duration = Duration::from_secs(6 * 60 * 60);
/// `SANDBOX_AUTO_STOP` as the provider is asked for it.
pub const SANDBOX_TTL: Ttl = Ttl::Seconds(SANDBOX_AUTO_STOP.as_secs());
/// How close to its auto-stop a running sandbox gets before the liveness pass pushes the deadline
/// out again by `SANDBOX_AUTO_STOP`; many liveness intervals, so a few failed tries still land
/// well before it.
pub const AUTO_STOP_EXTEND_BEFORE: Duration = Duration::from_secs(2 * 60 * 60);
const _: () = assert!(SANDBOX_AUTO_STOP.as_secs() > 2 * AUTO_STOP_EXTEND_BEFORE.as_secs());
/// The wait after a failed auto-stop extension (or one whose deadline didn't move) before the
/// next try; it doubles with each further failure.
pub const AUTO_STOP_RETRY_BASE: Duration = Duration::from_secs(10 * 60);
/// The longest wait between extension tries: well under the extension margin, so a few tries
/// still land before the deadline, and far under the auto-stop itself.
pub const AUTO_STOP_RETRY_CAP: Duration = Duration::from_secs(40 * 60);
const _: () = assert!(AUTO_STOP_RETRY_CAP.as_secs() < AUTO_STOP_EXTEND_BEFORE.as_secs());
/// The shortest time between two restarts of a dev server that stopped answering in a live
/// sandbox, so one that keeps crashing isn't restarted on every liveness pass.
pub const DEVSERVER_RESTART_GAP: Duration = Duration::from_secs(10 * 60);
/// Recorded (`devserver_restart_reason`) when the liveness pass restarts a dev server.
const DEVSERVER_NOT_ANSWERING: &str = "the dev server stopped answering on its port in a running sandbox";
/// How often a walk moves its lease ahead. The lease (`sandbox_store::WALK_LEASE`) is more than
/// twice this, so one missed renewal never lets it run out under a live walk.
pub const LEASE_RENEW_EVERY: Duration = Duration::from_secs(60);
const _: () = assert!(store::WALK_LEASE.as_secs() > 2 * LEASE_RENEW_EVERY.as_secs());
/// The longest wait taken from a provider's retry hint; a larger one is cut to this, so the retry
/// time can always be computed.
pub const MAX_PROVIDER_WAIT: Duration = Duration::from_secs(60 * 60);
/// How long a running sandbox goes between asks of the provider whether it is still up.
pub const LIVENESS_INTERVAL: Duration = Duration::from_secs(2 * 60);
/// Most running sandboxes asked about in one liveness pass, which bounds the calls it makes.
pub const LIVENESS_BATCH: i64 = 25;
const _: () = assert!(AUTO_STOP_EXTEND_BEFORE.as_secs() > 10 * LIVENESS_INTERVAL.as_secs());

/// What went wrong in a project step.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StepError {
    /// The sandbox still runs commands; only the project's own setup failed (installing its
    /// dependencies, starting its dev server). The sandbox counts as running, without a preview
    /// and with this reason recorded, so Koda can run commands to fix it.
    Usable(String),
    /// Something outside the project failed (the provider, the database, a backend); try the
    /// step again later.
    Retry(String),
}

/// The time `wait` from now, with the wait cut to `MAX_PROVIDER_WAIT`: never a panic, whatever
/// the provider asked for.
pub fn retry_time(wait: Duration) -> DateTime<Utc> {
    let wait = chrono::Duration::from_std(wait.min(MAX_PROVIDER_WAIT)).unwrap_or_else(|_| chrono::Duration::zero());
    Utc::now().checked_add_signed(wait).unwrap_or_else(Utc::now)
}

/// Run `walk` while holding its lease: `renew` moves the lease `lease` ahead and is called at
/// once and then every `every`. A renewal that fails leaves the walk running only while the
/// lease last written is sure to outlast the next try; past that the walk is dropped (`None`),
/// so a walk never goes on after its lease could have run out and another worker taken the row.
/// The walk writes every state before the provider call that follows it, so dropping it between
/// two awaits is a crash the next claim recovers from.
pub async fn hold_lease<T, R, RF>(walk: impl Future<Output = T>, mut renew: R, every: Duration, lease: Duration) -> Option<T>
where
    R: FnMut() -> RF,
    RF: Future<Output = bool>,
{
    tokio::pin!(walk);
    // The claim has just written a lease, so the first renewal is due one period from now, not at
    // once: a renewal started with every walk could land after a quick walk's release.
    let mut ticker = tokio::time::interval_at(tokio::time::Instant::now() + every, every);
    ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    // The claim that handed the walk over has just written a lease.
    let mut held_until = Instant::now() + lease;
    // A renewal runs beside the walk, never instead of it: it may be waiting for something the
    // walk holds (a pool connection, the row lock of its transaction), which only the walk
    // running on can free.
    let mut renewing: Option<(Instant, std::pin::Pin<Box<tokio::time::Timeout<RF>>>)> = None;
    loop {
        tokio::select! {
            biased;
            out = &mut walk => return Some(out),
            renewed = async { renewing.as_mut().expect("guarded by the branch condition").1.as_mut().await }, if renewing.is_some() => {
                let asked = renewing.take().map_or_else(Instant::now, |(asked, _)| asked);
                if renewed.unwrap_or(false) {
                    held_until = asked + lease;
                } else if Instant::now() + every >= held_until {
                    return None;
                }
            }
            _ = ticker.tick(), if renewing.is_none() => {
                renewing = Some((Instant::now(), Box::pin(tokio::time::timeout(every, renew()))));
            }
        }
    }
}

/// The `provider_error_kind` of a row that went to `error` after too many failures in a row,
/// when no single provider call is to blame.
const ATTEMPTS_EXHAUSTED: &str = "attempts_exhausted";
/// Why a walk ended when its lease could no longer be renewed (for logs).
const LEASE_LOST: &str = "the walk's lease could not be renewed";

/// What kind of provider failure this is, for logs. The kind only, never the message: messages
/// can carry URLs and tokens.
fn error_kind(err: &SandboxError) -> &'static str {
    match err {
        SandboxError::RateLimited { .. } => "rate_limited",
        SandboxError::OutOfCapacity { .. } => "out_of_capacity",
        SandboxError::BillingRequired => "billing_required",
        SandboxError::SnapshotExpired => "snapshot_expired",
        SandboxError::NotFound => "not_found",
        SandboxError::Refused(_) => "refused",
        SandboxError::Unavailable(_) => "unavailable",
        SandboxError::Other(_) => "other",
    }
}

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
    async fn start_preview(&self, project_id: Uuid, sandbox: &SandboxId) -> Result<PreviewUrl, StepError>;
    /// Install the project's dependencies again if its dependency files changed since the last
    /// install. Run after a sync into a sandbox that is already running. Returns whether an
    /// install ran: the dev server is then started again, so it picks up what changed.
    async fn update_dependencies(&self, _project_id: Uuid, _sandbox: &SandboxId) -> Result<bool, StepError> {
        Ok(false)
    }
    /// Whether the project's dev server answers on its port (the readiness check, once). The
    /// liveness pass asks this of running sandboxes that have a preview.
    async fn dev_server_answers(&self, _project_id: Uuid, _sandbox: &SandboxId) -> Result<bool, String> {
        Ok(true)
    }
}

pub struct SandboxManager {
    provider: Arc<dyn SandboxProvider>,
    limiter: Mutex<StartLimiter>,
    pool: PgPool,
    steps: Arc<dyn ProjectSteps>,
    cfg: SandboxConfig,
    /// The last state each project in flight was seen in, and when, so a transition can say what
    /// it came from and how long that took. Settled projects are dropped from it.
    seen: Mutex<HashMap<Uuid, (State, Instant)>>,
    /// The admin switch is off: the walk only brings sandboxes down, never up.
    stop_only: AtomicBool,
    /// How often a walk renews its lease (`LEASE_RENEW_EVERY`; tests shorten it).
    lease_renew_every: Duration,
}

/// An early exit from the walk: the outcome to report.
type Exit = ReconcileOutcome;

fn db<T>(result: Result<T, sqlx::Error>) -> Result<T, Exit> {
    result.map_err(|e| ReconcileOutcome::Failed(format!("database: {e}")))
}

impl SandboxManager {
    /// The provider the manager talks to.
    pub fn provider(&self) -> &Arc<dyn SandboxProvider> {
        &self.provider
    }

    pub fn new(provider: Arc<dyn SandboxProvider>, steps: Arc<dyn ProjectSteps>, pool: PgPool, cfg: SandboxConfig) -> Self {
        let limiter = Mutex::new(StartLimiter::new(cfg.starts_per_minute, cfg.starts_per_hour));
        Self { provider, limiter, pool, steps, cfg, seen: Mutex::default(), stop_only: AtomicBool::new(false), lease_renew_every: LEASE_RENEW_EVERY }
    }

    /// Renew walk leases this often instead of `LEASE_RENEW_EVERY`. For tests.
    pub fn with_lease_renewal(mut self, every: Duration) -> Self {
        self.lease_renew_every = every;
        self
    }

    /// Take the row toward what it wants. The row passed in only says which project to work on;
    /// the current row is read again so a stale copy never decides anything. The walk's lease is
    /// renewed for as long as it runs (see `hold_lease`).
    pub async fn reconcile(&self, row: SandboxRow) -> ReconcileOutcome {
        let project_id = row.project_id;
        let pool = self.pool.clone();
        let renew = move || {
            let pool = pool.clone();
            async move {
                match pool.acquire().await {
                    Ok(mut conn) => store::renew_lease(&mut conn, project_id).await.is_ok(),
                    Err(_) => false,
                }
            }
        };
        let walked = hold_lease(self.walk(project_id), renew, self.lease_renew_every, store::WALK_LEASE).await;
        let outcome = match walked {
            Some(Ok(outcome) | Err(outcome)) => outcome,
            // The lease could not be kept: leave the row to the lease running out.
            None => return ReconcileOutcome::Failed(LEASE_LOST.to_string()),
        };
        // The walk is over, whichever way it ended: let the next claim take the row.
        if let Ok(mut conn) = self.pool.acquire().await {
            let _ = store::release(&mut conn, project_id).await;
        }
        match &outcome {
            // A settled row has nothing left to wait for.
            ReconcileOutcome::Settled => {
                let _ = sqlx::query("UPDATE project_sandboxes SET retry_at = NULL WHERE project_id = $1").bind(project_id).execute(&self.pool).await;
            }
            // Every failure leaves the row held back, even one (a database error) that wrote no
            // retry time of its own, so the worker never claims it again on its next poll.
            ReconcileOutcome::Failed(_) => {
                let _ = sqlx::query("UPDATE project_sandboxes SET retry_at = $2 WHERE project_id = $1 AND (retry_at IS NULL OR retry_at <= now())")
                    .bind(project_id)
                    .bind(retry_time(RETRY_BASE))
                    .execute(&self.pool)
                    .await;
            }
            ReconcileOutcome::RetryAt(_) => {}
        }
        outcome
    }

    async fn walk(&self, project_id: Uuid) -> Result<ReconcileOutcome, Exit> {
        let row = {
            let mut conn = db(self.pool.acquire().await)?;
            db(store::get(&mut conn, project_id).await)?
        };
        let Some(row) = row else { return Ok(ReconcileOutcome::Settled) };
        // Where the walk starts from, for the first transition's `from`.
        self.seen.lock().unwrap_or_else(|e| e.into_inner()).entry(project_id).or_insert((row.state, Instant::now()));
        match row.desired {
            Desired::Running if self.stop_only.load(Ordering::SeqCst) => Ok(ReconcileOutcome::Settled),
            Desired::Running => self.bring_up(row).await,
            Desired::Stopped => self.bring_down(row).await,
        }
    }

    /// Turn stop-only mode on or off. While on, a row that wants to run is left as it is: no
    /// create, no resume, no attempt counted.
    pub fn set_stop_only(&self, on: bool) {
        self.stop_only.store(on, Ordering::SeqCst);
    }

    /// Text for a log line, with the API key and any URL's secrets removed.
    pub fn redact(&self, text: &str) -> String {
        nomi_sandbox::redact::redact(text, &self.cfg.api_key)
    }

    /// Tell the project's open pages the sandbox is now in `to`, and log the transition. Called
    /// once the change is saved (after the commit, where the change was made in a transaction) so
    /// a page that reloads on the event sees it. Every state this module writes goes through
    /// here, so this is the one place transitions are logged: the project, the sandbox when the
    /// row has one, where it came from, how long it was there, and the kind of provider failure
    /// when there was one. Never the key, the preview address or an error message.
    async fn announce(&self, project_id: Uuid, to: State, error_kind: Option<&'static str>) {
        let (from, elapsed_ms) = {
            let mut seen = self.seen.lock().unwrap_or_else(|e| e.into_inner());
            let previous = seen.get(&project_id).copied();
            if matches!(to, State::None | State::Stopped | State::Gone) {
                seen.remove(&project_id);
            } else {
                seen.insert(project_id, (to, Instant::now()));
            }
            previous.map_or((None, None), |(state, at)| (Some(state), Some(at.elapsed().as_millis() as u64)))
        };
        let sandbox_id: Option<String> = sqlx::query_scalar("SELECT sandbox_id FROM project_sandboxes WHERE project_id = $1").bind(project_id).fetch_optional(&self.pool).await.ok().flatten();
        tracing::info!(
            %project_id,
            sandbox_id = sandbox_id.as_deref().unwrap_or(""),
            from = from.map_or("unknown", State::as_str),
            to = to.as_str(),
            elapsed_ms = elapsed_ms.unwrap_or(0),
            provider_error_kind = error_kind.unwrap_or(""),
            "sandbox: state changed"
        );
        crate::project_events::sandbox_changed(project_id, to.phase());
    }

    async fn set_state(&self, project_id: Uuid, state: State) -> Result<(), Exit> {
        let mut conn = db(self.pool.acquire().await)?;
        db(store::set_state(&mut conn, project_id, state, None).await)?;
        self.announce(project_id, state, None).await;
        Ok(())
    }

    /// Hold the row back until `at`, saying why.
    async fn retry_at(&self, project_id: Uuid, at: DateTime<Utc>, why: &str) -> Exit {
        let result = async {
            let mut tx = self.pool.begin().await?;
            let attempts = store::record_attempt_failure(&mut tx, project_id, at, why).await?;
            let cooled = attempts >= MAX_ATTEMPTS;
            if cooled {
                store::enter_cooldown(&mut tx, project_id, Self::cooldown_end(), why).await?;
            }
            tx.commit().await?;
            Ok::<bool, sqlx::Error>(cooled)
        }
        .await;
        match result {
            Ok(true) => {
                self.stop_after_cooldown(project_id).await;
                self.announce(project_id, State::Error, Some(ATTEMPTS_EXHAUSTED)).await;
                ReconcileOutcome::Failed(why.to_string())
            }
            Ok(false) => ReconcileOutcome::RetryAt(at),
            Err(e) => ReconcileOutcome::Failed(format!("database: {e}")),
        }
    }

    fn cooldown_end() -> DateTime<Utc> {
        Utc::now() + chrono::Duration::from_std(PROVIDER_COOLDOWN).expect("the cooldown fits a chrono duration")
    }

    /// The row went to `error` for its cooldown: stop its sandbox, if it has one, so it doesn't
    /// keep running (and billing) unmetered while nothing can use it, and close any metering
    /// session. Best effort: a sandbox that can't be stopped now is stopped when the row is.
    async fn stop_after_cooldown(&self, project_id: Uuid) {
        let sandbox_id: Option<String> =
            sqlx::query_scalar("SELECT sandbox_id FROM project_sandboxes WHERE project_id = $1").bind(project_id).fetch_optional(&self.pool).await.ok().flatten().flatten();
        if let Some(id) = sandbox_id {
            match self.provider.stop(&SandboxId(id), false).await {
                Ok(()) | Err(SandboxError::NotFound) => {}
                Err(e) => tracing::warn!(%project_id, provider_error_kind = error_kind(&e), "sandbox: stopping a sandbox entering its cooldown failed"),
            }
        }
        if let Ok(mut conn) = self.pool.acquire().await {
            if let Err(e) = close_session(&mut conn, project_id).await {
                tracing::warn!(%project_id, error = %e, "sandbox: closing the session of a sandbox entering its cooldown failed");
            }
        }
    }

    /// The files version the project is at now.
    async fn files_version(&self, project_id: Uuid) -> Result<i64, Exit> {
        db(sqlx::query_scalar("SELECT files_version FROM projects WHERE id = $1").bind(project_id).fetch_one(&self.pool).await)
    }

    /// A step on a running sandbox failed in a way worth retrying: count it and hold the row back
    /// (growing waits, at most `RETRY_CAP`), without ever moving it out of `running`, so its
    /// commands keep running and the preview stays up.
    async fn retry_running(&self, row: &SandboxRow, why: String) -> Exit {
        let at = Utc::now() + Self::wait_after(row.attempts, None);
        let result = async {
            let mut conn = self.pool.acquire().await?;
            store::record_attempt_failure(&mut conn, row.project_id, at, &why).await
        }
        .await;
        match result {
            Ok(_) => ReconcileOutcome::RetryAt(at),
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

    /// How long to wait after a failure: what the provider asked for (at most
    /// `MAX_PROVIDER_WAIT`), or a growing delay.
    fn wait_after(attempts: i32, err: Option<&SandboxError>) -> Duration {
        err.and_then(SandboxError::retry_after).map_or_else(|| backoff(attempts.max(0) as u32, RETRY_BASE, RETRY_CAP), |wait| wait.min(MAX_PROVIDER_WAIT))
    }

    /// A provider call failed: retry later if that can help, otherwise mark the row as errored
    /// (still held back, so the worker doesn't spin on it).
    async fn provider_failed(&self, row: &SandboxRow, err: SandboxError) -> Exit {
        let at = retry_time(Self::wait_after(row.attempts, Some(&err)));
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
            let cooled = store::record_attempt_failure(&mut tx, project_id, at, &why).await? >= MAX_ATTEMPTS;
            if cooled {
                store::enter_cooldown(&mut tx, project_id, Self::cooldown_end(), &why).await?;
            }
            tx.commit().await?;
            Ok::<bool, sqlx::Error>(cooled)
        }
        .await;
        match saved {
            Ok(cooled) => {
                if cooled {
                    self.stop_after_cooldown(project_id).await;
                }
                self.announce(project_id, State::Error, Some(error_kind(&err))).await;
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
            Err(wait) => Err(self.defer(row.project_id, retry_time(wait)).await),
        }
    }

    /// Forget the provider's sandbox: it is gone, or can't be started again.
    async fn forget(&self, project_id: Uuid) -> Result<(), Exit> {
        self.forget_inner(project_id, 0, false).await
    }

    /// Queue a sandbox for deletion at the provider, when a delete just now didn't work.
    async fn enqueue_deletion(&self, project_id: Uuid, id: &SandboxId) {
        let queued = sqlx::query("INSERT INTO sandbox_deletions (provider, sandbox_id, project_id) VALUES ($1, $2, $3) ON CONFLICT (sandbox_id) DO NOTHING")
            .bind(crate::sandbox_worker::PROVIDER_NAME)
            .bind(&id.0)
            .bind(project_id)
            .execute(&self.pool)
            .await;
        if let Err(e) = queued {
            tracing::warn!(%project_id, error = %e, "sandbox: queueing a sandbox for deletion failed");
        }
    }

    /// Delete a sandbox we no longer want; if the provider can't do it now, queue it for the worker.
    async fn delete_or_enqueue(&self, project_id: Uuid, id: &SandboxId) {
        match self.provider.delete(id).await {
            Ok(()) | Err(SandboxError::NotFound) => {}
            Err(_) => self.enqueue_deletion(project_id, id).await,
        }
    }

    /// Like `forget`, but the row keeps `attempts` failed attempts (reaching `none` would reset
    /// them), so a sandbox that keeps dying while it starts backs off further each time.
    /// `enqueue` is for a sandbox that still exists at the provider (in its `error` state): its
    /// id goes to the deletion outbox in the same transaction, so it isn't leaked.
    async fn forget_inner(&self, project_id: Uuid, attempts: i32, enqueue: bool) -> Result<(), Exit> {
        let result = async {
            let mut tx = self.pool.begin().await?;
            if enqueue {
                sqlx::query("INSERT INTO sandbox_deletions (provider, sandbox_id, project_id) SELECT provider, sandbox_id, project_id FROM project_sandboxes WHERE project_id = $1 AND sandbox_id IS NOT NULL ON CONFLICT (sandbox_id) DO NOTHING")
                    .bind(project_id)
                    .execute(&mut *tx)
                    .await?;
            }
            sqlx::query(
                "UPDATE project_sandboxes SET sandbox_id = NULL, synced_version = 0, synced_manifest = '{}'::jsonb, deps_hash = NULL, preview_url = NULL, devserver_process = NULL, auto_stop_at = NULL, auto_stop_failures = 0, auto_stop_retry_at = NULL WHERE project_id = $1",
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
        self.announce(project_id, State::None, None).await;
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
            ttl: SANDBOX_TTL,
            from_snapshot: None,
            idempotency_key: format!("nomi-project-{}-{n}", row.project_id),
            labels: BTreeMap::from([(PROJECT_LABEL.to_string(), row.project_id.to_string())]),
        };
        let info = match self.provider.create(spec).await {
            Ok(info) => info,
            Err(e) => return Err(self.provider_failed(row, e).await),
        };
        // Another walk may have recorded a sandbox since this one began; theirs stands.
        let written = db(sqlx::query(
            "UPDATE project_sandboxes SET sandbox_id = $2, auto_stop_at = now() + make_interval(secs => $3), auto_stop_failures = 0, auto_stop_retry_at = NULL WHERE project_id = $1 AND (sandbox_id IS NULL OR sandbox_id = $2)",
        )
            .bind(row.project_id)
            .bind(&info.id.0)
            .bind(SANDBOX_AUTO_STOP.as_secs_f64())
            .execute(&self.pool)
            .await)?;
        if written.rows_affected() == 0 {
            self.delete_or_enqueue(row.project_id, &info.id).await;
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
                    self.forget_inner(row.project_id, row.attempts, info.state == SandboxState::Error).await?;
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
    /// `still_exists` is true for a sandbox in the provider's `error` state: it is deleted later.
    async fn vanished(&self, row: &SandboxRow, still_exists: bool) -> Result<ReconcileOutcome, Exit> {
        if matches!(row.state, State::Creating | State::Starting) {
            self.forget_inner(row.project_id, row.attempts, still_exists).await?;
            let at = Utc::now() + Self::wait_after(row.attempts + 1, None);
            return Ok(self.retry_at(row.project_id, at, "sandbox went away while starting").await);
        }
        self.forget_inner(row.project_id, 0, still_exists).await?;
        Ok(self.defer(row.project_id, Utc::now()).await)
    }

    async fn bring_up(&self, row: SandboxRow) -> Result<ReconcileOutcome, Exit> {
        let project_id = row.project_id;
        let mut id = row.sandbox_id.clone().map(SandboxId);
        // A sandbox we already have: find out where it is.
        if let Some(existing) = id.clone() {
            match self.provider.get(&existing).await {
                Ok(info) => match info.state {
                    SandboxState::Running if row.state == State::Running => return self.refresh(&row, &existing).await,
                    SandboxState::Gone | SandboxState::Error => {
                        return self.vanished(&row, info.state == SandboxState::Error).await;
                    }
                    SandboxState::Stopping => {
                        return Ok(self.defer(project_id, Utc::now() + STOPPING_RECHECK).await);
                    }
                    SandboxState::Stopped => {
                        self.take_start(&row).await?;
                        self.set_state(project_id, State::Starting).await?;
                        match self.provider.resume(&existing, ResumeSpec { ttl: SANDBOX_TTL }).await {
                            Ok(_) => {
                                let mut conn = db(self.pool.acquire().await)?;
                                db(store::note_auto_stop(&mut conn, project_id, &existing.0, Self::assumed_auto_stop()).await)?;
                            }
                            Err(SandboxError::SnapshotExpired) => id = self.start_over(&existing, &row).await?,
                            Err(e) => return Ok(self.provider_failed(&row, e).await),
                        }
                    }
                    SandboxState::Starting | SandboxState::Running => {}
                },
                Err(SandboxError::NotFound) => return self.vanished(&row, false).await,
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
        // A project whose own setup fails (its install, its dev server) still gets a running
        // sandbox, without a preview and with the reason saved, so Koda can run the commands
        // that fix it (the failure has gone to Koda as a failed check).
        let (preview, failure) = match self.steps.start_preview(project_id, &id).await {
            Ok(preview) => (Some(preview.url), None),
            Err(StepError::Usable(why)) => (None, Some(why)),
            Err(StepError::Retry(why)) => return Ok(self.step_failed(&row, why).await),
        };
        let size = self.cfg.size.as_api();
        let result = async {
            let mut tx = self.pool.begin().await?;
            sqlx::query("UPDATE project_sandboxes SET synced_version = $2, preview_url = $3, alive_checked_at = now() WHERE project_id = $1")
                .bind(project_id)
                .bind(version)
                .bind(&preview)
                .execute(&mut *tx)
                .await?;
            store::set_state(&mut tx, project_id, State::Running, failure.as_deref()).await?;
            // The unique index keeps a racing walk from opening a second session.
            sqlx::query(
                "INSERT INTO sandbox_sessions (project_id, user_id, size) VALUES ($1, (SELECT user_id FROM projects WHERE id = $1), $2)
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
        self.announce(project_id, State::Running, None).await;
        Ok(ReconcileOutcome::Settled)
    }

    /// A running sandbox whose project moved on: copy the changed files in, install again if the
    /// dependency files changed, and start the preview if it has none (it failed before, and the
    /// files that changed may be Koda's fix, the person asked to retry, or the liveness pass found
    /// its dev server dead). When an install ran, the dev server is started again (killed and
    /// started, and its new address saved): upgraded or removed packages, a plugin or a changed
    /// dev script need a fresh server. Settled when there is nothing to do. A failure of the project's own setup is recorded on the row, which stays
    /// running; anything else is retried later, still running.
    async fn refresh(&self, row: &SandboxRow, id: &SandboxId) -> Result<ReconcileOutcome, Exit> {
        let project_id = row.project_id;
        let behind = row.synced_version < self.files_version(project_id).await?;
        let retry_preview = row.preview_url.is_none() && row.last_error.is_none();
        if !behind && !retry_preview {
            return Ok(ReconcileOutcome::Settled);
        }
        let mut version = row.synced_version;
        let mut failure = None;
        let mut restart = false;
        if behind {
            version = match self.steps.sync(project_id, id).await {
                Ok(version) => version,
                Err(why) => return Ok(self.retry_running(row, why).await),
            };
            match self.steps.update_dependencies(project_id, id).await {
                Ok(installed) => restart = installed,
                Err(StepError::Usable(why)) => failure = Some(why),
                Err(StepError::Retry(why)) => return Ok(self.retry_running(row, why).await),
            }
        }
        let mut preview = row.preview_url.clone();
        if (preview.is_none() || restart) && failure.is_none() {
            match self.steps.start_preview(project_id, id).await {
                Ok(url) => preview = Some(url.url),
                // The old server was killed for the start: there is no preview now.
                Err(StepError::Usable(why)) => {
                    preview = None;
                    failure = Some(why);
                }
                Err(StepError::Retry(why)) => {
                    if restart {
                        // The old server may be gone, and the install that asked for the restart
                        // won't run again: without a preview the retry starts the server again.
                        let cleared = sqlx::query("UPDATE project_sandboxes SET preview_url = NULL WHERE project_id = $1 AND sandbox_id = $2").bind(project_id).bind(&id.0).execute(&self.pool).await;
                        db(cleared)?;
                    }
                    return Ok(self.retry_running(row, why).await);
                }
            }
        }
        let result = async {
            let mut tx = self.pool.begin().await?;
            sqlx::query("UPDATE project_sandboxes SET synced_version = $3, preview_url = $4 WHERE project_id = $1 AND sandbox_id = $2")
                .bind(project_id)
                .bind(&id.0)
                .bind(version)
                .bind(&preview)
                .execute(&mut *tx)
                .await?;
            store::set_state(&mut tx, project_id, State::Running, failure.as_deref()).await?;
            tx.commit().await
        }
        .await;
        db(result)?;
        self.announce(project_id, State::Running, None).await;
        Ok(ReconcileOutcome::Settled)
    }

    /// Ask the provider whether the running sandboxes not asked about for `LIVENESS_INTERVAL` are
    /// still up (at most `LIVENESS_BATCH` per pass, oldest first). One that stopped by itself,
    /// crashed or is gone has its row moved back to `stopped` with its metering session closed,
    /// so the next claim walks it again: resumes it, or finds it gone and makes a new one.
    ///
    /// For one that is up, the pass also pushes its auto-stop out again once it is within
    /// `AUTO_STOP_EXTEND_BEFORE`, and asks its dev server whether it answers: one that doesn't is
    /// restarted through the preview start (at most once per `DEVSERVER_RESTART_GAP`, the reason
    /// recorded). Returns how many sandboxes were found down.
    pub async fn check_alive(&self) -> usize {
        let due = async {
            let mut conn = self.pool.acquire().await?;
            store::alive_due(&mut conn, LIVENESS_INTERVAL.as_secs_f64(), LIVENESS_BATCH, AUTO_STOP_EXTEND_BEFORE.as_secs_f64(), DEVSERVER_RESTART_GAP.as_secs_f64()).await
        }
        .await;
        let due = match due {
            Ok(due) => due,
            Err(e) => {
                tracing::warn!(error = %e, "sandbox liveness: looking for running sandboxes failed");
                return 0;
            }
        };
        let mut down = 0;
        for item in due {
            let project_id = item.project_id;
            let sandbox_id = item.sandbox_id.clone();
            let id = SandboxId(sandbox_id.clone());
            // `Some(alive)` when the provider answered; `None` when it couldn't say.
            let known = match self.provider.get(&id).await {
                Ok(info) => Some(matches!(info.state, SandboxState::Running | SandboxState::Starting)),
                Err(SandboxError::NotFound) => Some(false),
                Err(e) => {
                    // Unknown: ask again next interval rather than hammering the provider.
                    tracing::warn!(%project_id, provider_error_kind = error_kind(&e), "sandbox liveness: asking the provider failed");
                    None
                }
            };
            let alive = known.unwrap_or(true);
            let result = async {
                let mut tx = self.pool.begin().await?;
                let moved = if alive {
                    store::mark_alive(&mut tx, project_id).await?;
                    false
                } else if store::mark_dead(&mut tx, project_id, &sandbox_id).await? {
                    close_session(&mut tx, project_id).await?;
                    true
                } else {
                    false
                };
                tx.commit().await?;
                Ok::<bool, sqlx::Error>(moved)
            }
            .await;
            match result {
                Ok(true) => {
                    tracing::warn!(%project_id, "sandbox liveness: a running sandbox is down, walking it again");
                    self.announce(project_id, State::Stopped, None).await;
                    down += 1;
                }
                Ok(false) => {}
                Err(e) => tracing::warn!(%project_id, error = %e, "sandbox liveness: saving the result failed"),
            }
            if known != Some(true) {
                continue;
            }
            if item.extend_auto_stop {
                self.extend_auto_stop(project_id, &id).await;
            }
            if item.has_preview {
                self.check_dev_server(&item, &id).await;
            }
        }
        down
    }

    /// The auto-stop deadline assumed when the provider reports none: `SANDBOX_AUTO_STOP` from now.
    fn assumed_auto_stop() -> DateTime<Utc> {
        Utc::now() + chrono::Duration::from_std(SANDBOX_AUTO_STOP).expect("the auto-stop fits a chrono duration")
    }

    /// Push a running sandbox's auto-stop `SANDBOX_AUTO_STOP` out from now and record the
    /// deadline the provider then reports (assumed only when it reports none). An extension
    /// through boat's resume may count as a start (the contract counts every resume and doesn't
    /// say whether resuming a running sandbox is one), so it takes a start from the limiter and
    /// waits for a later pass when none is free. A failure, or a reported deadline that didn't
    /// move past the margin, is backed off (`AUTO_STOP_RETRY_BASE` doubling up to
    /// `AUTO_STOP_RETRY_CAP`), so a provider that refuses is asked a few times before the
    /// deadline, not on every pass; should the deadline still pass, the sandbox stops and the
    /// liveness pass resumes it.
    async fn extend_auto_stop(&self, project_id: Uuid, id: &SandboxId) {
        let slot = self.limiter.lock().unwrap_or_else(|e| e.into_inner()).try_acquire(Instant::now());
        if slot.is_err() {
            tracing::debug!(%project_id, "sandbox liveness: no start free to extend the auto-stop, trying next pass");
            return;
        }
        let reported = match self.provider.extend_auto_stop(id, SANDBOX_TTL).await {
            Ok(reported) => reported.and_then(|raw| DateTime::parse_from_rfc3339(&raw).ok()).map(|d| d.with_timezone(&Utc)),
            Err(e) => {
                tracing::warn!(%project_id, provider_error_kind = error_kind(&e), "sandbox liveness: extending the auto-stop failed; backing off");
                self.extension_failed(project_id, id, None).await;
                return;
            }
        };
        let margin = chrono::Duration::from_std(AUTO_STOP_EXTEND_BEFORE).expect("the margin fits a chrono duration");
        match reported {
            Some(deadline) if deadline <= Utc::now() + margin => {
                tracing::warn!(%project_id, "sandbox liveness: the provider took the extension but its auto-stop didn't move; backing off");
                self.extension_failed(project_id, id, Some(deadline)).await;
            }
            _ => {
                let deadline = reported.unwrap_or_else(Self::assumed_auto_stop);
                let noted = async {
                    let mut conn = self.pool.acquire().await?;
                    store::note_auto_stop(&mut conn, project_id, &id.0, deadline).await
                }
                .await;
                if let Err(e) = noted {
                    tracing::warn!(%project_id, error = %e, "sandbox liveness: saving the new auto-stop failed");
                }
            }
        }
    }

    /// Count a failed extension and hold the next try back, saving the provider's deadline if it
    /// reported one.
    async fn extension_failed(&self, project_id: Uuid, id: &SandboxId, reported: Option<DateTime<Utc>>) {
        let saved = async {
            let mut conn = self.pool.acquire().await?;
            let failures = store::auto_stop_failures(&mut conn, project_id, &id.0).await?;
            let wait = backoff(failures.max(0) as u32, AUTO_STOP_RETRY_BASE, AUTO_STOP_RETRY_CAP);
            store::auto_stop_extension_failed(&mut conn, project_id, &id.0, retry_time(wait), reported).await
        }
        .await;
        if let Err(e) = saved {
            tracing::warn!(%project_id, error = %e, "sandbox liveness: saving the extension backoff failed");
        }
    }

    /// Ask the dev server of a live sandbox whether it answers. One that doesn't loses its
    /// preview, with the reason recorded, so the next claim walks the row and starts it again
    /// (`refresh`, which kills the old process first); not again within `DEVSERVER_RESTART_GAP`.
    async fn check_dev_server(&self, item: &store::AliveDue, id: &SandboxId) {
        let project_id = item.project_id;
        match self.steps.dev_server_answers(project_id, id).await {
            Ok(true) => {}
            Ok(false) if !item.may_restart_dev_server => {
                tracing::warn!(%project_id, "sandbox liveness: the dev server isn't answering; restarted recently, so waiting");
            }
            Ok(false) => {
                let marked = async {
                    let mut conn = self.pool.acquire().await?;
                    store::dev_server_down(&mut conn, project_id, &item.sandbox_id, DEVSERVER_NOT_ANSWERING).await
                }
                .await;
                match marked {
                    Ok(true) => {
                        tracing::warn!(%project_id, reason = DEVSERVER_NOT_ANSWERING, "sandbox liveness: restarting the dev server");
                        self.announce(project_id, State::Running, None).await;
                    }
                    Ok(false) => {}
                    Err(e) => tracing::warn!(%project_id, error = %e, "sandbox liveness: saving the dev server restart failed"),
                }
            }
            Err(why) => tracing::warn!(%project_id, why = %self.redact(&why), "sandbox liveness: asking the dev server failed"),
        }
    }

    /// The sandbox can't be started again (its snapshot expired): drop it and make a new one.
    async fn start_over(&self, old: &SandboxId, row: &SandboxRow) -> Result<Option<SandboxId>, Exit> {
        // Best effort: the old one is useless either way, and a failure here must not stop the new one.
        self.delete_or_enqueue(row.project_id, old).await;
        self.forget(row.project_id).await?;
        Ok(Some(self.create(row).await?))
    }

    /// Hand back rows that have sat mid-change with nobody walking them for longer than
    /// `STUCK_AFTER`; they are due at once and no attempt is counted. Returns how many.
    pub async fn unstick(&self) -> usize {
        let moved = async {
            let mut conn = self.pool.acquire().await?;
            store::release_stuck(&mut conn, STUCK_AFTER.as_secs() as i64).await
        }
        .await;
        match moved {
            Ok(ids) => {
                for id in &ids {
                    tracing::warn!(project_id = %id, "sandbox watchdog: row was stuck, handing it back");
                }
                ids.len()
            }
            Err(e) => {
                tracing::warn!(error = %e, "sandbox watchdog failed");
                0
            }
        }
    }

    /// Delete the provider copy of sandboxes that have been stopped longer than `EXPIRY_WARN`
    /// and mark their rows `gone`, so the next start builds a fresh one from the project's
    /// files. A running or starting sandbox is never touched. Returns how many were expired.
    pub async fn expire_stopped(&self) -> usize {
        let due = async {
            let mut conn = self.pool.acquire().await?;
            store::long_stopped(&mut conn, EXPIRY_WARN.as_secs() as i64).await
        }
        .await;
        let due = match due {
            Ok(due) => due,
            Err(e) => {
                tracing::warn!(error = %e, "sandbox expiry: looking for old sandboxes failed");
                return 0;
            }
        };
        let mut expired = 0;
        for (project_id, sandbox_id) in due {
            match self.provider.delete(&SandboxId(sandbox_id.clone())).await {
                Ok(()) | Err(SandboxError::NotFound) => {}
                Err(e) => {
                    tracing::warn!(%project_id, error = %e, "sandbox expiry: deleting at the provider failed");
                    continue;
                }
            }
            let marked = async {
                let mut conn = self.pool.acquire().await?;
                store::mark_expired(&mut conn, project_id, &sandbox_id).await
            }
            .await;
            match marked {
                Ok(true) => {
                    self.announce(project_id, State::Gone, None).await;
                    expired += 1;
                }
                Ok(false) => {}
                Err(e) => tracing::warn!(%project_id, error = %e, "sandbox expiry: marking the row failed"),
            }
        }
        expired
    }

    async fn bring_down(&self, row: SandboxRow) -> Result<ReconcileOutcome, Exit> {
        let project_id = row.project_id;
        let Some(raw) = row.sandbox_id.clone() else {
            self.forget(project_id).await?;
            return Ok(ReconcileOutcome::Settled);
        };
        // A row in `error` may hold a sandbox its cooldown already stopped, or one that is gone.
        if row.state == State::Error {
            match self.provider.get(&SandboxId(raw.clone())).await {
                Ok(info) if info.state == SandboxState::Stopped => return self.mark_stopped(project_id).await,
                Ok(info) if info.state == SandboxState::Gone => {
                    self.forget(project_id).await?;
                    return Ok(ReconcileOutcome::Settled);
                }
                Err(SandboxError::NotFound) => {
                    self.forget(project_id).await?;
                    return Ok(ReconcileOutcome::Settled);
                }
                _ => {}
            }
        }
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
        self.mark_stopped(project_id).await
    }

    /// The sandbox is stopped at the provider: record it and close the metering session.
    async fn mark_stopped(&self, project_id: Uuid) -> Result<ReconcileOutcome, Exit> {
        let result = async {
            let mut tx = self.pool.begin().await?;
            store::set_state(&mut tx, project_id, State::Stopped, None).await?;
            close_session(&mut tx, project_id).await?;
            tx.commit().await
        }
        .await;
        db(result)?;
        self.announce(project_id, State::Stopped, None).await;
        Ok(ReconcileOutcome::Settled)
    }
}

/// End the project's open metering session, if it has one.
async fn close_session(conn: &mut sqlx::PgConnection, project_id: Uuid) -> Result<u64, sqlx::Error> {
    let done = sqlx::query("UPDATE sandbox_sessions SET stopped_at = now() WHERE project_id = $1 AND stopped_at IS NULL")
        .bind(project_id)
        .execute(conn)
        .await?;
    Ok(done.rows_affected())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A renewal interval short enough for a test.
    const EVERY: Duration = Duration::from_millis(10);
    /// A lease a few renewals long.
    const LEASE: Duration = Duration::from_millis(35);

    #[test]
    fn a_huge_provider_wait_is_capped_and_never_panics() {
        let huge = SandboxError::RateLimited { retry_after: Duration::MAX };
        assert_eq!(SandboxManager::wait_after(0, Some(&huge)), MAX_PROVIDER_WAIT);
        let at = retry_time(Duration::MAX);
        assert!(at <= Utc::now() + chrono::Duration::from_std(MAX_PROVIDER_WAIT).unwrap());
    }

    #[test]
    fn sandboxes_take_a_long_finite_auto_stop() {
        assert_eq!(SANDBOX_TTL, Ttl::Seconds(6 * 60 * 60));
    }

    #[tokio::test]
    async fn a_walk_that_keeps_its_lease_runs_to_the_end() {
        let renewals = std::sync::atomic::AtomicUsize::new(0);
        let walk = async {
            tokio::time::sleep(LEASE * 3).await;
            "done"
        };
        let out = hold_lease(
            walk,
            || {
                renewals.fetch_add(1, Ordering::SeqCst);
                async { true }
            },
            EVERY,
            LEASE,
        )
        .await;
        assert_eq!(out, Some("done"));
        assert!(renewals.load(Ordering::SeqCst) >= 3, "the lease was renewed while the walk ran");
    }

    /// A renewal that has to wait for something the walk holds (a pool connection, the row lock
    /// of its transaction) must not stop the walk from running: the walk ends, frees what it
    /// held, and the renewal goes through.
    #[tokio::test]
    async fn the_walk_keeps_running_while_a_renewal_waits_on_it() {
        let held = std::sync::Arc::new(tokio::sync::Semaphore::new(1));
        let walk = {
            let held = held.clone();
            async move {
                let _held = held.acquire().await.unwrap();
                tokio::time::sleep(LEASE * 2 / 3).await;
                "done"
            }
        };
        let out = hold_lease(
            walk,
            || {
                let held = held.clone();
                async move { held.acquire().await.is_ok() }
            },
            EVERY,
            LEASE,
        )
        .await;
        assert_eq!(out, Some("done"));
    }

    #[tokio::test]
    async fn a_walk_that_loses_its_lease_is_dropped_before_the_lease_runs_out() {
        let started = Instant::now();
        let out = hold_lease(std::future::pending::<()>(), || async { false }, EVERY, LEASE).await;
        assert_eq!(out, None);
        assert!(started.elapsed() < LEASE, "dropped before the last lease written could run out: {:?}", started.elapsed());
    }
}
