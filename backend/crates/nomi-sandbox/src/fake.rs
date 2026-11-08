//! An in-memory `SandboxProvider` for tests. It follows the real provider's rules: files stay
//! across stop and resume, background processes do not, and a resumed sandbox gets a new preview host.

use std::collections::BTreeMap;
use std::sync::{Mutex, MutexGuard};
use std::time::Duration;

use async_trait::async_trait;

use crate::{
    CreateSpec, ExecOutput, PreviewUrl, ProcessHandle, ProcessStatus, ResumeSpec, SandboxError,
    SandboxId, SandboxInfo, SandboxProvider, SandboxState,
};

/// Exit code the fake gives every command that finishes.
const EXIT_OK: i32 = 0;
/// Suffix of the file `upload_archive` leaves in the destination directory.
const ARCHIVE_FILE: &str = ".upload.tar.gz";

struct Process {
    running: bool,
}

struct Sandbox {
    state: SandboxState,
    files: BTreeMap<String, Vec<u8>>,
    processes: BTreeMap<String, Process>,
    /// Bumped on every resume so the preview host changes.
    generation: u64,
}

#[derive(Default)]
struct Inner {
    sandboxes: BTreeMap<SandboxId, Sandbox>,
    by_key: BTreeMap<String, SandboxId>,
    calls: Vec<String>,
    queued_failures: Vec<SandboxError>,
    next_id: u64,
    next_process: u64,
}

/// In-memory provider that records calls and can be told to fail.
#[derive(Default)]
pub struct FakeProvider {
    inner: Mutex<Inner>,
}

impl FakeProvider {
    pub fn new() -> Self {
        Self::default()
    }

    fn lock(&self) -> MutexGuard<'_, Inner> {
        self.inner.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// Make the next call (any method) fail with `err`; later calls work again.
    pub fn fail_next(&self, err: SandboxError) {
        self.lock().queued_failures.push(err);
    }

    pub fn set_state(&self, id: &SandboxId, state: SandboxState) {
        if let Some(sb) = self.lock().sandboxes.get_mut(id) {
            sb.state = state;
        }
    }

    /// The files currently held for a sandbox, by path.
    pub fn files(&self, id: &SandboxId) -> BTreeMap<String, Vec<u8>> {
        self.lock()
            .sandboxes
            .get(id)
            .map(|s| s.files.clone())
            .unwrap_or_default()
    }

    /// Every call made so far, as `method` or `method arg`, in order.
    pub fn calls(&self) -> Vec<String> {
        self.lock().calls.clone()
    }

    /// Records the call and returns the queued failure, if any.
    fn enter(&self, call: String) -> Result<MutexGuard<'_, Inner>, SandboxError> {
        let mut g = self.lock();
        g.calls.push(call);
        if g.queued_failures.is_empty() {
            Ok(g)
        } else {
            Err(g.queued_failures.remove(0))
        }
    }
}

impl Inner {
    fn sandbox(&mut self, id: &SandboxId) -> Result<&mut Sandbox, SandboxError> {
        self.sandboxes.get_mut(id).ok_or(SandboxError::NotFound)
    }

    fn running(&mut self, id: &SandboxId) -> Result<&mut Sandbox, SandboxError> {
        let sb = self.sandbox(id)?;
        if sb.state == SandboxState::Running {
            Ok(sb)
        } else {
            Err(SandboxError::Refused(format!("sandbox is {:?}", sb.state)))
        }
    }
}

#[async_trait]
impl SandboxProvider for FakeProvider {
    async fn create(&self, spec: CreateSpec) -> Result<SandboxInfo, SandboxError> {
        let mut g = self.enter(format!("create {}", spec.idempotency_key))?;
        if let Some(id) = g.by_key.get(&spec.idempotency_key).cloned() {
            let state = g.sandbox(&id)?.state;
            return Ok(SandboxInfo { id, state });
        }
        g.next_id += 1;
        let id = SandboxId(format!("fake-{}", g.next_id));
        g.by_key.insert(spec.idempotency_key, id.clone());
        g.sandboxes.insert(
            id.clone(),
            Sandbox {
                state: SandboxState::Running,
                files: BTreeMap::new(),
                processes: BTreeMap::new(),
                generation: 0,
            },
        );
        Ok(SandboxInfo {
            id,
            state: SandboxState::Running,
        })
    }

    async fn get(&self, id: &SandboxId) -> Result<SandboxInfo, SandboxError> {
        let mut g = self.enter(format!("get {}", id.0))?;
        let state = g.sandbox(id)?.state;
        Ok(SandboxInfo {
            id: id.clone(),
            state,
        })
    }

    async fn resume(&self, id: &SandboxId, _spec: ResumeSpec) -> Result<SandboxInfo, SandboxError> {
        let mut g = self.enter(format!("resume {}", id.0))?;
        let sb = g.sandbox(id)?;
        sb.state = SandboxState::Running;
        sb.generation += 1;
        Ok(SandboxInfo {
            id: id.clone(),
            state: SandboxState::Running,
        })
    }

    async fn stop(&self, id: &SandboxId, force: bool) -> Result<(), SandboxError> {
        let mut g = self.enter(format!("stop {} force={force}", id.0))?;
        let sb = g.sandbox(id)?;
        sb.state = SandboxState::Stopped;
        sb.processes.values_mut().for_each(|p| p.running = false);
        Ok(())
    }

    async fn delete(&self, id: &SandboxId) -> Result<(), SandboxError> {
        let mut g = self.enter(format!("delete {}", id.0))?;
        g.sandboxes
            .remove(id)
            .map(|_| ())
            .ok_or(SandboxError::NotFound)
    }

    async fn list(&self) -> Result<Vec<SandboxInfo>, SandboxError> {
        let g = self.enter("list".into())?;
        Ok(g.sandboxes
            .iter()
            .map(|(id, s)| SandboxInfo {
                id: id.clone(),
                state: s.state,
            })
            .collect())
    }

    async fn exec(
        &self,
        id: &SandboxId,
        command: &str,
        _timeout: Duration,
    ) -> Result<ExecOutput, SandboxError> {
        let mut g = self.enter(format!("exec {} {command}", id.0))?;
        g.running(id)?;
        Ok(ExecOutput {
            exit_code: Some(EXIT_OK),
            stdout: String::new(),
            stderr: String::new(),
        })
    }

    async fn exec_detached(
        &self,
        id: &SandboxId,
        command: &str,
    ) -> Result<ProcessHandle, SandboxError> {
        let mut g = self.enter(format!("exec_detached {} {command}", id.0))?;
        g.running(id)?;
        g.next_process += 1;
        let process_id = format!("proc-{}", g.next_process);
        g.sandbox(id)?
            .processes
            .insert(process_id.clone(), Process { running: true });
        Ok(ProcessHandle { process_id })
    }

    async fn process_status(
        &self,
        id: &SandboxId,
        handle: &ProcessHandle,
        stdout_from: u64,
        stderr_from: u64,
    ) -> Result<ProcessStatus, SandboxError> {
        let mut g = self.enter(format!("process_status {} {}", id.0, handle.process_id))?;
        let p = g
            .sandbox(id)?
            .processes
            .get(&handle.process_id)
            .ok_or(SandboxError::NotFound)?;
        Ok(ProcessStatus {
            running: p.running,
            exit_code: if p.running { None } else { Some(EXIT_OK) },
            stdout: String::new(),
            stderr: String::new(),
            stdout_next: stdout_from,
            stderr_next: stderr_from,
        })
    }

    async fn kill_process(
        &self,
        id: &SandboxId,
        handle: &ProcessHandle,
    ) -> Result<(), SandboxError> {
        let mut g = self.enter(format!("kill_process {} {}", id.0, handle.process_id))?;
        let p = g
            .sandbox(id)?
            .processes
            .get_mut(&handle.process_id)
            .ok_or(SandboxError::NotFound)?;
        p.running = false;
        Ok(())
    }

    async fn write_file(
        &self,
        id: &SandboxId,
        path: &str,
        bytes: &[u8],
    ) -> Result<(), SandboxError> {
        let mut g = self.enter(format!("write_file {} {path}", id.0))?;
        g.running(id)?
            .files
            .insert(path.to_string(), bytes.to_vec());
        Ok(())
    }

    async fn delete_file(&self, id: &SandboxId, path: &str) -> Result<(), SandboxError> {
        let mut g = self.enter(format!("delete_file {} {path}", id.0))?;
        g.running(id)?
            .files
            .remove(path)
            .map(|_| ())
            .ok_or(SandboxError::NotFound)
    }

    async fn upload_archive(
        &self,
        id: &SandboxId,
        dest_dir: &str,
        tar_gz: Vec<u8>,
    ) -> Result<(), SandboxError> {
        let mut g = self.enter(format!("upload_archive {} {dest_dir}", id.0))?;
        let path = format!("{}/{ARCHIVE_FILE}", dest_dir.trim_end_matches('/'));
        g.running(id)?.files.insert(path, tar_gz);
        Ok(())
    }

    async fn expose_port(&self, id: &SandboxId, port: u16) -> Result<PreviewUrl, SandboxError> {
        let mut g = self.enter(format!("expose_port {} {port}", id.0))?;
        let sb = g.running(id)?;
        Ok(PreviewUrl {
            url: format!(
                "https://{}-{port}-g{}.preview.fake.test",
                id.0, sb.generation
            ),
        })
    }
}
