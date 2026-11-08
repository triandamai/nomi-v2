//! The project-specific work done once a sandbox is up: copying the project's files in
//! (`sandbox_sync`) and starting its dev server (not built yet).

use std::sync::Arc;

use async_trait::async_trait;
use nomi_sandbox::{PreviewUrl, SandboxId, SandboxProvider};
use nomi_storage::ProjectStore;
use sqlx::PgPool;
use uuid::Uuid;

use crate::sandbox_manager::ProjectSteps;
use crate::sandbox_sync::PassHook;

/// Runs the steps the reconciler asks for against the project's stored files and the provider.
pub struct ServerSteps {
    pub(crate) pool: PgPool,
    pub(crate) storage: ProjectStore,
    pub(crate) provider: Arc<dyn SandboxProvider>,
    pub(crate) pass_hook: Option<PassHook>,
}

impl ServerSteps {
    pub fn new(pool: PgPool, storage: ProjectStore, provider: Arc<dyn SandboxProvider>) -> Self {
        Self { pool, storage, provider, pass_hook: None }
    }

    /// Run `hook` with the pass number after each sync pass has copied its files, before the
    /// check for changes made meanwhile. For tests that change files mid-sync.
    pub fn with_pass_hook(mut self, hook: PassHook) -> Self {
        self.pass_hook = Some(hook);
        self
    }
}

#[async_trait]
impl ProjectSteps for ServerSteps {
    async fn sync(&self, project_id: Uuid, sandbox: &SandboxId) -> Result<i64, String> {
        self.sync_project(project_id, sandbox).await.map_err(|e| e.to_string())
    }

    async fn start_preview(&self, _project_id: Uuid, _sandbox: &SandboxId) -> Result<PreviewUrl, String> {
        Err("dev server not implemented yet".to_string())
    }
}
