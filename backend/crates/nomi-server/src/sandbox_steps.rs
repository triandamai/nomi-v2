//! The project-specific work done once a sandbox is up: copying the project's files in
//! (`sandbox_sync`) and starting its dev server (not built yet).

use std::sync::Arc;

use async_trait::async_trait;
use nomi_sandbox::{PreviewUrl, SandboxId, SandboxProvider};
use nomi_storage::ProjectStore;
use sqlx::PgPool;
use uuid::Uuid;

use crate::sandbox_devserver::{DevServerError, DevServerTiming};
use crate::sandbox_manager::ProjectSteps;
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
        self.sync_project(project_id, sandbox).await.map_err(|e| e.to_string())
    }

    async fn start_preview(&self, project_id: Uuid, sandbox: &SandboxId) -> Result<PreviewUrl, String> {
        let version: Option<i64> = sqlx::query_scalar("SELECT files_version FROM projects WHERE id = $1").bind(project_id).fetch_optional(&self.pool).await.map_err(|e| e.to_string())?;
        let started = self.ensure_dev_server(project_id, sandbox).await;
        if let Some(version) = version {
            self.record_first_check(project_id, version, &started).await;
        }
        started.map_err(|e| e.to_string())
    }
}

impl ServerSteps {
    /// The check the browser page runs on opening a project is `npm install`, and the dev server
    /// start has just run it, so its outcome is the check's: record it once per `files_version`,
    /// and when the install failed hand it to Koda as the page's check does (see
    /// `routes::project_runtime::record_check_for_koda`). Other start failures (the dev server
    /// not answering) are not a check result, as in the browser.
    async fn record_first_check(&self, project_id: Uuid, version: i64, started: &Result<PreviewUrl, DevServerError>) {
        let (ok, output) = match started {
            Ok(_) => (true, String::new()),
            Err(DevServerError::Install(why)) => (false, why.clone()),
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
