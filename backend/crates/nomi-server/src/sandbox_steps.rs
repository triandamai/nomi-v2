//! The project-specific work done once a sandbox is up: copying the project's files in
//! (`sandbox_sync`), installing its dependencies and starting its dev server
//! (`sandbox_devserver`).

use std::sync::Arc;

use async_trait::async_trait;
use nomi_sandbox::redact::redact_urls;
use nomi_sandbox::{PreviewUrl, SandboxId, SandboxProvider};
use nomi_storage::ProjectStore;
use sqlx::PgPool;
use uuid::Uuid;

use crate::sandbox_devserver::{DevServerError, DevServerTiming};
use crate::sandbox_manager::{ProjectSteps, StepError};
use crate::sandbox_sync::PassHook;

/// Runs the steps the reconciler asks for against the project's stored files and the provider.
pub struct ServerSteps {
    pub(crate) pool: PgPool,
    pub(crate) storage: ProjectStore,
    pub(crate) provider: Arc<dyn SandboxProvider>,
    pub(crate) pass_hook: Option<PassHook>,
    pub(crate) devserver_timing: DevServerTiming,
    /// Relay to read a project's dev backend from; the project's own (or the shared one) when unset.
    pub(crate) relay_config: Option<nomi_relay_client::RelayConfig>,
    /// Opens the sealed Relay keys of projects that have their own backend.
    pub(crate) settings_key: Option<[u8; 32]>,
    pub(crate) http_client: reqwest::Client,
    /// Wakes a project's sleeping Relay before its dev server starts; not woken when unset.
    pub(crate) shipyard: Option<nomi_shipyard::ShipyardConfig>,
    /// How long a command from Koda may run in the sandbox.
    pub(crate) run_timeout: std::time::Duration,
}

impl ServerSteps {
    pub fn new(pool: PgPool, storage: ProjectStore, provider: Arc<dyn SandboxProvider>) -> Self {
        Self {
            pool,
            storage,
            provider,
            pass_hook: None,
            devserver_timing: DevServerTiming::default(),
            relay_config: None,
            settings_key: None,
            http_client: reqwest::Client::new(),
            shipyard: None,
            run_timeout: crate::sandbox_runs::DEFAULT_RUN_TIMEOUT,
        }
    }

    /// Run `hook` with the pass number after each sync pass has copied its files, before the
    /// check for changes made meanwhile. For tests that change files mid-sync.
    pub fn with_pass_hook(mut self, hook: PassHook) -> Self {
        self.pass_hook = Some(hook);
        self
    }

    /// How long to wait for a dev server and how often to check. For tests.
    pub fn with_devserver_timing(mut self, timing: DevServerTiming) -> Self {
        self.devserver_timing = timing;
        self
    }

    /// How long a command from Koda may run; longer than one `exec` allows runs it detached.
    pub fn with_run_timeout(mut self, timeout: std::time::Duration) -> Self {
        self.run_timeout = timeout;
        self
    }

    /// Read every project's dev backend through this Relay. For tests.
    pub fn with_relay_config(mut self, config: nomi_relay_client::RelayConfig) -> Self {
        self.relay_config = Some(config);
        self
    }

    /// Wake projects' sleeping Relay backends through this Shipyard.
    pub fn with_shipyard(mut self, config: Option<nomi_shipyard::ShipyardConfig>) -> Self {
        self.shipyard = config;
        self
    }

    /// The key that opens projects' sealed Relay keys, and the client used to reach Relay.
    pub fn with_relay_access(mut self, settings_key: Option<[u8; 32]>, http_client: reqwest::Client) -> Self {
        self.settings_key = settings_key;
        self.http_client = http_client;
        self
    }
}

#[async_trait]
impl ProjectSteps for ServerSteps {
    async fn sync(&self, project_id: Uuid, sandbox: &SandboxId) -> Result<i64, String> {
        self.sync_project(project_id, sandbox).await.map_err(|e| redact_urls(&e.to_string()))
    }

    async fn start_preview(&self, project_id: Uuid, sandbox: &SandboxId) -> Result<PreviewUrl, StepError> {
        let version = self.current_version(project_id).await.map_err(StepError::Retry)?;
        let started = self.ensure_dev_server(project_id, sandbox).await;
        if let Some(version) = version {
            self.record_check(project_id, version, &started).await;
        }
        started.map_err(step_error)
    }

    async fn update_dependencies(&self, project_id: Uuid, sandbox: &SandboxId) -> Result<(), StepError> {
        let version = self.current_version(project_id).await.map_err(StepError::Retry)?;
        let installed = self.reinstall_if_changed(project_id, sandbox).await;
        if let (Some(version), Err(DevServerError::Install(_))) = (version, &installed) {
            self.record_check(project_id, version, &installed).await;
        }
        installed.map_err(step_error)
    }
}

/// A dev server failure as the reconciler sees it: the project's own install or dev server
/// failing leaves a sandbox that still runs commands (`Usable`), so Koda can fix it; anything
/// else is retried. Step errors are logged and saved as `last_error`, and storage or Relay
/// errors may hold URLs, so the text is redacted.
pub(crate) fn step_error(err: DevServerError) -> StepError {
    let usable = matches!(err, DevServerError::Install(_) | DevServerError::NotReady(_));
    let why = redact_urls(&err.to_string());
    if usable {
        StepError::Usable(why)
    } else {
        StepError::Retry(why)
    }
}

impl ServerSteps {
    async fn current_version(&self, project_id: Uuid) -> Result<Option<i64>, String> {
        sqlx::query_scalar("SELECT files_version FROM projects WHERE id = $1").bind(project_id).fetch_optional(&self.pool).await.map_err(|e| e.to_string())
    }

    /// The check the browser page runs on opening a project is `npm install`, and the dev server
    /// start has just run it, so its outcome is the check's: record it once per `files_version`,
    /// and when it failed hand it to Koda as the page's check does (see
    /// `routes::project_runtime::record_check_for_koda`). A dev server that doesn't come up is a
    /// failed check too: with no page running the project, nobody else would see it.
    async fn record_check<T>(&self, project_id: Uuid, version: i64, started: &Result<T, DevServerError>) {
        let (ok, output) = match started {
            Ok(_) => (true, String::new()),
            Err(e @ (DevServerError::Install(_) | DevServerError::NotReady(_))) => (false, redact_urls(&e.to_string())),
            Err(_) => return,
        };
        let recorded = async {
            let (owner, checked): (Uuid, Option<i64>) =
                sqlx::query_as("SELECT user_id, checked_version FROM projects WHERE id = $1").bind(project_id).fetch_one(&self.pool).await.map_err(|e| e.to_string())?;
            if checked == Some(version) {
                return Ok(());
            }
            let mut conn = self.pool.acquire().await.map_err(|e| e.to_string())?;
            crate::routes::project_runtime::record_check_for_koda(&mut conn, project_id, owner, version, ok, &output).await.map(|_| ())
        }
        .await;
        if let Err(why) = recorded {
            tracing::warn!(error = %why, %project_id, "sandbox: recording the first check failed");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_projects_own_setup_failing_leaves_a_usable_sandbox() {
        assert!(matches!(step_error(DevServerError::Install("npm install exited with 1".into())), StepError::Usable(why) if why.contains("npm install")));
        assert!(matches!(step_error(DevServerError::NotReady("no answer".into())), StepError::Usable(_)));
        assert!(matches!(step_error(DevServerError::Backend("relay down".into())), StepError::Retry(_)));
        assert!(matches!(step_error(DevServerError::Sandbox(nomi_sandbox::SandboxError::NotFound)), StepError::Retry(_)));
    }
}
