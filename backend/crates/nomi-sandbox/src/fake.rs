//! An in-memory `SandboxProvider` for tests. It follows the real provider's rules: files stay
//! across stop and resume, background processes do not, and a resumed sandbox gets a new preview host.

use std::collections::BTreeMap;
use std::sync::{Mutex, MutexGuard};
use std::time::Duration;

use async_trait::async_trait;

use crate::{
    CreateSpec, ExecOutput, PreviewUrl, ProcessHandle, ProcessStatus, ResumeSpec, SandboxError,
    SandboxId, SandboxInfo, SandboxProvider, SandboxState, Ttl,
};

/// Exit code the fake gives every command that finishes.
const EXIT_OK: i32 = 0;
/// Suffix of the file `upload_archive` leaves in the destination directory.
const ARCHIVE_FILE: &str = ".upload.tar.gz";

struct Process {
    running: bool,
    /// Status checks made so far.
    polls: u32,
    exit_code: Option<i32>,
}

struct Sandbox {
    state: SandboxState,
    files: BTreeMap<String, Vec<u8>>,
    processes: BTreeMap<String, Process>,
    /// Bumped on every resume so the preview host changes.
    generation: u64,
    labels: BTreeMap<String, String>,
}

#[derive(Default)]
struct Inner {
    sandboxes: BTreeMap<SandboxId, Sandbox>,
    by_key: BTreeMap<String, SandboxId>,
    calls: Vec<String>,
    /// The auto-stop asked for by each create and resume, in order.
    ttls: Vec<Ttl>,
    /// Each `extend_auto_stop` asked for, in order.
    extensions: Vec<(SandboxId, Ttl)>,
    queued_failures: Vec<SandboxError>,
    /// (text a command must contain, what it answers); the latest matching rule wins.
    exec_rules: Vec<(String, ExecOutput)>,
    /// When set, a background process exits with the code after this many status checks.
    process_exit: Option<(u32, i32)>,
    /// What every background process reports as its output so far.
    process_logs: (String, String),
    /// What `usage` reports per sandbox.
    usage: BTreeMap<SandboxId, Duration>,
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

    /// Answer every later `exec` whose command contains `contains` with `output`. The latest
    /// matching rule wins; commands no rule matches succeed with no output.
    pub fn on_exec(&self, contains: &str, output: ExecOutput) {
        self.lock().exec_rules.push((contains.to_string(), output));
    }

    /// What background processes report as their output from now on.
    pub fn set_process_logs(&self, stdout: &str, stderr: &str) {
        self.lock().process_logs = (stdout.to_string(), stderr.to_string());
    }

    /// Make every background process exit with `exit_code` once `polls` status checks have seen
    /// it running (by default a process runs until it is killed).
    pub fn exit_processes_after(&self, polls: u32, exit_code: i32) {
        self.lock().process_exit = Some((polls, exit_code));
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

    /// Make `usage` report `used` for the sandbox.
    pub fn set_usage(&self, id: &SandboxId, used: Duration) {
        self.lock().usage.insert(id.clone(), used);
    }

    /// Every call made so far, as `method` or `method arg`, in order.
    pub fn calls(&self) -> Vec<String> {
        self.lock().calls.clone()
    }

    /// The auto-stop each create and resume asked for, in order.
    pub fn ttls(&self) -> Vec<Ttl> {
        self.lock().ttls.clone()
    }

    /// Each auto-stop extension asked for, in order.
    pub fn extensions(&self) -> Vec<(SandboxId, Ttl)> {
        self.lock().extensions.clone()
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
        g.ttls.push(spec.ttl);
        if let Some(id) = g.by_key.get(&spec.idempotency_key).cloned() {
            let state = g.sandbox(&id)?.state;
            let labels = g.sandbox(&id)?.labels.clone();
            return Ok(SandboxInfo { id, state, labels });
        }
        g.next_id += 1;
        let id = SandboxId(format!("fake-{}", g.next_id));
        g.by_key.insert(spec.idempotency_key.clone(), id.clone());
        g.sandboxes.insert(
            id.clone(),
            Sandbox {
                state: SandboxState::Running,
                files: BTreeMap::new(),
                processes: BTreeMap::new(),
                generation: 0,
                labels: spec.labels.clone(),
            },
        );
        Ok(SandboxInfo {
            id,
            state: SandboxState::Running,
            labels: spec.labels,
        })
    }

    async fn get(&self, id: &SandboxId) -> Result<SandboxInfo, SandboxError> {
        let mut g = self.enter(format!("get {}", id.0))?;
        let sb = g.sandbox(id)?;
        Ok(SandboxInfo {
            id: id.clone(),
            state: sb.state,
            labels: sb.labels.clone(),
        })
    }

    async fn resume(&self, id: &SandboxId, spec: ResumeSpec) -> Result<SandboxInfo, SandboxError> {
        let mut g = self.enter(format!("resume {}", id.0))?;
        g.ttls.push(spec.ttl);
        let sb = g.sandbox(id)?;
        sb.state = SandboxState::Running;
        sb.generation += 1;
        Ok(SandboxInfo {
            id: id.clone(),
            state: SandboxState::Running,
            labels: sb.labels.clone(),
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
        g.by_key.retain(|_, v| v != id);
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
                labels: s.labels.clone(),
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
        let scripted = g
            .exec_rules
            .iter()
            .rev()
            .find(|(needle, _)| command.contains(needle.as_str()))
            .map(|(_, out)| out.clone());
        Ok(scripted.unwrap_or(ExecOutput {
            exit_code: Some(EXIT_OK),
            stdout: String::new(),
            stderr: String::new(),
        }))
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
            .insert(process_id.clone(), Process { running: true, polls: 0, exit_code: None });
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
        let (full_out, full_err) = g.process_logs.clone();
        let process_exit = g.process_exit;
        let stdout = full_out.get(stdout_from as usize..).unwrap_or_default().to_string();
        let stderr = full_err.get(stderr_from as usize..).unwrap_or_default().to_string();
        let p = g
            .sandbox(id)?
            .processes
            .get_mut(&handle.process_id)
            .ok_or(SandboxError::NotFound)?;
        if let Some((after, code)) = process_exit {
            if p.running && p.polls >= after {
                p.running = false;
                p.exit_code = Some(code);
            }
            p.polls += 1;
        }
        Ok(ProcessStatus {
            running: p.running,
            exit_code: if p.running { None } else { Some(p.exit_code.unwrap_or(EXIT_OK)) },
            stdout_next: full_out.len() as u64,
            stderr_next: full_err.len() as u64,
            stdout,
            stderr,
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

    async fn extend_auto_stop(&self, id: &SandboxId, ttl: Ttl) -> Result<(), SandboxError> {
        let mut g = self.enter(format!("extend_auto_stop {}", id.0))?;
        g.sandbox(id)?;
        g.extensions.push((id.clone(), ttl));
        Ok(())
    }

    async fn usage(&self, id: &SandboxId) -> Result<Option<Duration>, SandboxError> {
        let mut g = self.enter(format!("usage {}", id.0))?;
        g.sandbox(id)?;
        Ok(g.usage.get(id).copied())
    }
}
