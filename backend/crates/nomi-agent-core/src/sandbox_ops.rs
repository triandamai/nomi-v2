//! What agents can ask of a project's server-side sandbox (see nomi-server's `sandbox_manager`):
//! whether the project runs in one, and to wake it and wait until it is up. The server provides
//! it (it holds the sandbox rows and the admin switch); agents only see this trait, so an agent
//! crate never reaches the sandbox store. A Nomi without sandboxes uses `NoSandboxOps`, and every
//! project keeps running its commands in the user's browser.

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use uuid::Uuid;

/// How waking a project's sandbox ended.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WakeOutcome {
    /// The sandbox is up and can run commands.
    Running,
    /// It is still starting after the wait.
    Starting,
    /// It can't start; the reason.
    Failed(String),
}

#[async_trait]
pub trait SandboxOps: Send + Sync {
    /// Whether the project's commands run in a sandbox: the project asked for the sandbox runtime
    /// and the admin switch is on.
    async fn uses_sandbox(&self, project_id: Uuid) -> bool;

    /// Asks for the project's sandbox to be running and waits up to `timeout` for it.
    async fn wake_and_wait(&self, project_id: Uuid, timeout: Duration) -> WakeOutcome;
}

/// For tests and setups without sandboxes: no project uses one.
pub struct NoSandboxOps;

#[async_trait]
impl SandboxOps for NoSandboxOps {
    async fn uses_sandbox(&self, _project_id: Uuid) -> bool {
        false
    }

    async fn wake_and_wait(&self, _project_id: Uuid, _timeout: Duration) -> WakeOutcome {
        WakeOutcome::Failed(crate::prompts::SANDBOX_NOT_AVAILABLE.to_string())
    }
}

pub type SharedSandboxOps = Arc<dyn SandboxOps>;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn without_sandboxes_no_project_uses_one() {
        let ops = NoSandboxOps;
        assert!(!ops.uses_sandbox(Uuid::nil()).await);
        assert!(matches!(ops.wake_and_wait(Uuid::nil(), Duration::ZERO).await, WakeOutcome::Failed(_)));
    }
}
