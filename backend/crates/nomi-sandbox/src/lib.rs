//! Nomi's provider-neutral view of a remote sandbox: a small machine that holds a project's
//! files, runs its commands and serves its preview. The server talks to the `SandboxProvider`
//! trait only; `FakeProvider` stands in for a real provider in tests.

use std::collections::BTreeMap;
use std::fmt;
use std::time::Duration;

use async_trait::async_trait;

pub mod boat;
pub mod error;
pub mod fake;
pub mod plan;
pub mod redact;

pub use boat::{BoatConfig, BoatProvider};
pub use error::SandboxError;
pub use fake::FakeProvider;

/// The provider's name for one sandbox.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SandboxId(pub String);

/// How big a sandbox to ask for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Size {
    Small,
    Default,
    Large,
}

impl Size {
    /// The size name the provider's API expects.
    pub fn as_api(self) -> &'static str {
        match self {
            Size::Small => "small",
            Size::Default => "default",
            Size::Large => "large",
        }
    }
}

/// What to create a sandbox with.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateSpec {
    pub size: Size,
    /// How long the sandbox may run before the provider stops it on its own.
    pub ttl: Ttl,
    /// Start from this snapshot instead of an empty machine.
    pub from_snapshot: Option<String>,
    /// Sending the same key twice gives back the same sandbox.
    pub idempotency_key: String,
    /// Labels stored on the sandbox, such as `nomi-project` set to the project id.
    pub labels: BTreeMap<String, String>,
}

/// What to resume a stopped sandbox with.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ResumeSpec {
    /// How long the resumed sandbox may run before the provider stops it on its own.
    pub ttl: Ttl,
}

/// How long a sandbox may run before the provider stops it by itself.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Ttl {
    /// Say nothing and take the provider's default (boat stops a sandbox after an hour).
    #[default]
    ProviderDefault,
    /// Never stop it by itself; the caller stops it.
    Never,
    /// Stop it after this many seconds.
    Seconds(u64),
}

/// Where a sandbox is in its life.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SandboxState {
    Starting,
    Running,
    Stopping,
    Stopped,
    Error,
    Gone,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SandboxInfo {
    pub id: SandboxId,
    pub state: SandboxState,
    /// Labels stored on the sandbox; empty when the provider didn't return any.
    pub labels: BTreeMap<String, String>,
}

/// The result of a command that ran to the end (or timed out, when `exit_code` is `None`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecOutput {
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
}

/// A command started in the background.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessHandle {
    pub process_id: String,
}

/// A background command's progress. Output is read from an offset; `*_next` is where to read from next.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessStatus {
    pub running: bool,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub stdout_next: u64,
    pub stderr_next: u64,
}

/// A public address for a sandbox port. The address itself is the secret, so it is never printed.
#[derive(Clone, PartialEq, Eq)]
pub struct PreviewUrl {
    pub url: String,
}

impl fmt::Debug for PreviewUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PreviewUrl")
            .field("url", &"[redacted]")
            .finish()
    }
}

#[async_trait]
pub trait SandboxProvider: Send + Sync {
    async fn create(&self, spec: CreateSpec) -> Result<SandboxInfo, SandboxError>;
    async fn get(&self, id: &SandboxId) -> Result<SandboxInfo, SandboxError>;
    async fn resume(&self, id: &SandboxId, spec: ResumeSpec) -> Result<SandboxInfo, SandboxError>;
    async fn stop(&self, id: &SandboxId, force: bool) -> Result<(), SandboxError>;
    async fn delete(&self, id: &SandboxId) -> Result<(), SandboxError>;
    async fn list(&self) -> Result<Vec<SandboxInfo>, SandboxError>;
    /// Run a command and wait for it, up to `timeout`.
    async fn exec(
        &self,
        id: &SandboxId,
        command: &str,
        timeout: Duration,
    ) -> Result<ExecOutput, SandboxError>;
    /// Start a command in the background; it does not survive a stop and resume.
    async fn exec_detached(
        &self,
        id: &SandboxId,
        command: &str,
    ) -> Result<ProcessHandle, SandboxError>;
    async fn process_status(
        &self,
        id: &SandboxId,
        handle: &ProcessHandle,
        stdout_from: u64,
        stderr_from: u64,
    ) -> Result<ProcessStatus, SandboxError>;
    async fn kill_process(
        &self,
        id: &SandboxId,
        handle: &ProcessHandle,
    ) -> Result<(), SandboxError>;
    async fn write_file(
        &self,
        id: &SandboxId,
        path: &str,
        bytes: &[u8],
    ) -> Result<(), SandboxError>;
    async fn delete_file(&self, id: &SandboxId, path: &str) -> Result<(), SandboxError>;
    /// Unpack a gzipped tar into `dest_dir`.
    async fn upload_archive(
        &self,
        id: &SandboxId,
        dest_dir: &str,
        tar_gz: Vec<u8>,
    ) -> Result<(), SandboxError>;
    async fn expose_port(&self, id: &SandboxId, port: u16) -> Result<PreviewUrl, SandboxError>;
    /// How long the provider says the sandbox has run (and billed) since it was created, or `None`
    /// when the provider can't say.
    async fn usage(&self, _id: &SandboxId) -> Result<Option<Duration>, SandboxError> {
        Ok(None)
    }
    /// Push a running sandbox's auto-stop out to `ttl` from now, so it keeps running while it is
    /// used. A provider that can't do this does nothing; a sandbox it then stops on its own is
    /// found by the caller's liveness check and started again.
    async fn extend_auto_stop(&self, _id: &SandboxId, _ttl: Ttl) -> Result<(), SandboxError> {
        Ok(())
    }
}
