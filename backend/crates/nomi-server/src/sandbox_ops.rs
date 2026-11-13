//! The server's answer to what Koda asks of a project's sandbox (`nomi_agent_core::sandbox_ops`):
//! whether the project runs in one, and waking it. Waking only asks for the sandbox to be running
//! (`sandbox_store::want_running`); the sandbox worker does the starting, and this polls the row.
//!
//! Both questions go through `routes::project_sandbox::availability`, the same check the page's
//! start and heartbeat use: a sandbox over its owner's allowance, cooling down, or switched off is
//! never woken for Koda, and Koda's commands go to the browser instead.

use std::sync::Arc;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use nomi_agent_core::prompts::SANDBOX_UNAVAILABLE_NOW;
use nomi_agent_core::sandbox_ops::{SandboxOps, SharedSandboxOps, WakeOutcome};
use sqlx::PgPool;
use uuid::Uuid;

use crate::configuration::{ReaperMode, Rollout, SandboxConfig};
use crate::routes::project_sandbox::{availability, RUNTIME_SANDBOX};
use crate::sandbox_store::{self as store, State};
use crate::sandbox_worker::PROVIDER_NAME;

/// How often the row is looked at while waiting for the sandbox to come up.
const WAKE_POLL: Duration = Duration::from_millis(500);
/// What is said when the row errored without a recorded reason.
const UNKNOWN_FAILURE: &str = "no reason was recorded";
/// The allowance the test config gives: far more minutes than any test uses.
const TEST_MINUTES: u32 = 1_000_000;
/// The start limits of the test config: never in the way.
const TEST_STARTS: u32 = 1_000;

pub struct ServerSandboxOps {
    pool: PgPool,
    poll: Duration,
    /// Use this config instead of the saved one (the admin switch is then on). For tests.
    config: Option<SandboxConfig>,
    /// The longest any wait lasts, whatever the caller asks. For tests.
    wait_cap: Option<Duration>,
}

impl ServerSandboxOps {
    pub fn new(pool: PgPool) -> Self {
        Self { pool, poll: WAKE_POLL, config: None, wait_cap: None }
    }

    pub fn shared(pool: PgPool) -> SharedSandboxOps {
        Arc::new(Self::new(pool))
    }

    /// Behave as if the admin switch is on for everyone with a generous allowance, and poll every
    /// `poll`, waiting at most `cap`. For tests.
    pub fn for_tests(mut self, poll: Duration, cap: Duration) -> Self {
        self.config = Some(SandboxConfig {
            api_base: String::new(),
            api_key: String::new(),
            size: nomi_sandbox::Size::Small,
            idle_stop_secs: crate::configuration::DEFAULT_IDLE_STOP_SECS,
            auto_stop_secs: crate::configuration::DEFAULT_AUTO_STOP_SECS,
            rollout: Rollout::All,
            starts_per_minute: TEST_STARTS,
            starts_per_hour: TEST_STARTS,
            monthly_minutes_per_user: TEST_MINUTES,
            reaper_mode: ReaperMode::DryRun,
        });
        self.poll = poll;
        self.wait_cap = Some(cap);
        self
    }

    /// Use this config (switch on) instead of the saved one. For tests.
    pub fn with_config(mut self, config: SandboxConfig) -> Self {
        self.config = Some(config);
        self
    }

    async fn config(&self) -> Option<SandboxConfig> {
        match &self.config {
            Some(config) => Some(config.clone()),
            None => crate::configuration::sandbox_config().await,
        }
    }

    /// Whether the project runs in a sandbox that may be used for its owner (Koda acts for the
    /// owner) right now.
    async fn usable(&self, conn: &mut sqlx::PgConnection, project_id: Uuid) -> Result<bool, sqlx::Error> {
        let Some(config) = self.config().await else { return Ok(false) };
        let found = availability(conn, project_id, None, Some(&config)).await?;
        Ok(found.is_some_and(|a| a.runtime == RUNTIME_SANDBOX && a.available))
    }
}

#[async_trait]
impl SandboxOps for ServerSandboxOps {
    /// A sandbox project whose sandbox may be used, and that the person isn't running in their
    /// browser right now (the browser runtime's check-in marks the runner online; the sandbox
    /// page's never does): then the browser page claims Koda's commands, as before.
    async fn uses_sandbox(&self, project_id: Uuid) -> bool {
        let decided = async {
            let mut conn = self.pool.acquire().await?;
            if !self.usable(&mut conn, project_id).await? {
                return Ok(false);
            }
            Ok::<bool, sqlx::Error>(!nomi_agent_coding::runtime::runner_online(&mut conn, project_id).await?)
        }
        .await;
        decided.unwrap_or(false)
    }

    async fn wake_and_wait(&self, project_id: Uuid, timeout: Duration) -> WakeOutcome {
        let timeout = self.wait_cap.map_or(timeout, |cap| cap.min(timeout));
        let asked = async {
            let mut conn = self.pool.acquire().await?;
            // Never start (or bill) a sandbox the page itself wouldn't be allowed to start.
            if !self.usable(&mut conn, project_id).await? {
                return Ok(false);
            }
            // Waking is a sign of use: it moves `last_active_at`, so the idle sweep leaves the
            // sandbox alone while it starts and Koda's command is queued.
            store::want_running(&mut conn, project_id, PROVIDER_NAME).await?;
            Ok::<bool, sqlx::Error>(true)
        }
        .await;
        match asked {
            Ok(true) => {}
            Ok(false) => return WakeOutcome::Failed(SANDBOX_UNAVAILABLE_NOW.to_string()),
            Err(e) => return WakeOutcome::Failed(format!("database: {e}")),
        }
        let started = Instant::now();
        loop {
            let row: Result<Option<(String, Option<String>)>, _> =
                sqlx::query_as("SELECT state, last_error FROM project_sandboxes WHERE project_id = $1").bind(project_id).fetch_optional(&self.pool).await;
            match row {
                Ok(Some((state, why))) => match State::parse(&state) {
                    Some(State::Running) => return WakeOutcome::Running,
                    Some(State::Error) => return WakeOutcome::Failed(why.unwrap_or_else(|| UNKNOWN_FAILURE.to_string())),
                    _ => {}
                },
                Ok(None) => {}
                Err(e) => return WakeOutcome::Failed(format!("database: {e}")),
            }
            if started.elapsed() >= timeout {
                return WakeOutcome::Starting;
            }
            tokio::time::sleep(self.poll).await;
        }
    }
}
