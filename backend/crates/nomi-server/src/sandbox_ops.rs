//! The server's answer to what Koda asks of a project's sandbox (`nomi_agent_core::sandbox_ops`):
//! whether the project runs in one, and waking it. Waking only asks for the sandbox to be running
//! (`sandbox_store::want_running`); the sandbox worker does the starting, and this polls the row.

use std::sync::Arc;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use nomi_agent_core::sandbox_ops::{SandboxOps, SharedSandboxOps, WakeOutcome};
use sqlx::PgPool;
use uuid::Uuid;

use crate::sandbox_store::{self as store, State};
use crate::sandbox_worker::PROVIDER_NAME;

/// How often the row is looked at while waiting for the sandbox to come up.
const WAKE_POLL: Duration = Duration::from_millis(500);
/// What is said when the row errored without a recorded reason.
const UNKNOWN_FAILURE: &str = "no reason was recorded";

pub struct ServerSandboxOps {
    pool: PgPool,
    poll: Duration,
    /// Treat the admin switch as on without reading it. For tests.
    switch_on: bool,
    /// The longest any wait lasts, whatever the caller asks. For tests.
    wait_cap: Option<Duration>,
}

/// Why a wake is refused while the admin switch is off.
const SWITCH_OFF: &str = "the sandbox runtime is turned off";

impl ServerSandboxOps {
    pub fn new(pool: PgPool) -> Self {
        Self { pool, poll: WAKE_POLL, switch_on: false, wait_cap: None }
    }

    pub fn shared(pool: PgPool) -> SharedSandboxOps {
        Arc::new(Self::new(pool))
    }

    /// Behave as if the admin switch is on, and poll every `poll`, waiting at most `cap`. For tests.
    pub fn for_tests(mut self, poll: Duration, cap: Duration) -> Self {
        self.switch_on = true;
        self.poll = poll;
        self.wait_cap = Some(cap);
        self
    }
}

#[async_trait]
impl SandboxOps for ServerSandboxOps {
    async fn uses_sandbox(&self, project_id: Uuid) -> bool {
        if !self.switch_on && crate::configuration::sandbox_config().await.is_none() {
            return false;
        }
        let runtime: Result<Option<String>, _> = sqlx::query_scalar("SELECT runtime FROM projects WHERE id = $1").bind(project_id).fetch_optional(&self.pool).await;
        matches!(runtime, Ok(Some(runtime)) if runtime == "sandbox")
    }

    async fn wake_and_wait(&self, project_id: Uuid, timeout: Duration) -> WakeOutcome {
        // With the switch off nothing is asked for: the worker would only be winding sandboxes down.
        if !self.switch_on && crate::configuration::sandbox_config().await.is_none() {
            return WakeOutcome::Failed(SWITCH_OFF.to_string());
        }
        let timeout = self.wait_cap.map_or(timeout, |cap| cap.min(timeout));
        let asked = async {
            let mut conn = self.pool.acquire().await?;
            store::want_running(&mut conn, project_id, PROVIDER_NAME).await
        }
        .await;
        if let Err(e) = asked {
            return WakeOutcome::Failed(format!("database: {e}"));
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
