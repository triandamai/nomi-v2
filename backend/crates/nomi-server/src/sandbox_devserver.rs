//! Starts a project's dev server inside its sandbox: installs dependencies when they changed,
//! runs the dev command in the background, waits until it answers, and exposes its port.
//!
//! The reconciler runs this after every resume, so the dev server (which does not survive a
//! stop) is started fresh each time. The process id of the one started last, and the end of its
//! output, are saved on the project's sandbox row, so any Nomi process can kill it before the
//! next start and serve its log; the port is also freed by command, so a server nobody recorded
//! never blocks the new one.

use std::collections::BTreeMap;
use std::time::{Duration, Instant};

use nomi_sandbox::{PreviewUrl, ProcessHandle, SandboxError, SandboxId};
use uuid::Uuid;

use crate::sandbox_steps::ServerSteps;
use crate::sandbox_store as store;
use crate::sandbox_sync::{shell_quote, PROJECT_DIR};

/// The port every dev server listens on inside the sandbox.
pub const DEV_PORT: u16 = 5173;
/// How long the dev server gets to answer on its port before the start counts as failed.
pub const READY_TIMEOUT: Duration = Duration::from_secs(180);
/// How long to wait between readiness checks.
pub const READY_POLL_INTERVAL: Duration = Duration::from_secs(2);
/// How long one readiness check (a `curl`) may take.
const READY_CHECK_TIMEOUT: Duration = Duration::from_secs(10);
/// How long `npm install` may take (the longest a command may run on the provider).
const INSTALL_TIMEOUT: Duration = Duration::from_secs(600);
/// How long a quick check or cleanup command may take.
const QUICK_TIMEOUT: Duration = Duration::from_secs(15);
/// Most characters of the dev server's output kept per project (on its sandbox row).
pub const LOG_KEPT: usize = 8 * 1024;
/// The first HTTP status code that means the server answered (2xx to 4xx).
const FIRST_ANSWER_CODE: u16 = 200;
/// The last HTTP status code that counts as an answer (a 4xx page is still a running server; 5xx is not).
const LAST_ANSWER_CODE: u16 = 499;

/// The stack with no build step: its files are served as they are.
const STATIC_STACK: &str = "static";

/// The command that serves a stack in development on `DEV_PORT`: `npm run dev` for the Vite and Astro stacks. `--strictPort` makes Vite fail
/// rather than hop to the next port when a stale server still holds this one.
fn dev_command(stack: &str) -> String {
    if stack == STATIC_STACK {
        format!("npx --yes serve -l {DEV_PORT} .")
    } else {
        format!("npm run dev -- --host 0.0.0.0 --port {DEV_PORT} --strictPort")
    }
}
/// The files whose contents decide whether dependencies must be installed again.
const DEPENDENCY_FILES: &[&str] = &["package.json", "package-lock.json", "pnpm-lock.yaml", "yarn.lock", "bun.lockb"];

/// What the browser runtime sets for every preview.
const BASE_ENV: &[(&str, &str)] = &[("NOMI_PREVIEW", "1"), ("NODE_NO_WARNINGS", "1")];

#[derive(Debug, thiserror::Error)]
pub enum DevServerError {
    #[error("database: {0}")]
    Database(#[from] sqlx::Error),
    #[error("sandbox: {0}")]
    Sandbox(#[from] SandboxError),
    #[error("the project's backend isn't reachable: {0}")]
    Backend(String),
    #[error("installing dependencies failed: {0}")]
    Install(String),
    #[error("the dev server didn't come up: {0}")]
    NotReady(String),
}

/// How long to wait for the dev server and how often to look. Tests shorten both.
#[derive(Debug, Clone, Copy)]
pub struct DevServerTiming {
    pub ready_timeout: Duration,
    pub poll_interval: Duration,
}

impl Default for DevServerTiming {
    fn default() -> Self {
        Self { ready_timeout: READY_TIMEOUT, poll_interval: READY_POLL_INTERVAL }
    }
}

/// Add `text` to a log kept in memory, keeping only its last `LOG_KEPT` bytes.
fn append_log(log: &mut String, text: &str) {
    log.push_str(text);
    if log.len() > LOG_KEPT {
        let mut cut = log.len() - LOG_KEPT;
        while !log.is_char_boundary(cut) {
            cut += 1;
        }
        log.drain(..cut);
    }
}

/// The environment of the dev server: what the browser runtime passes, plus the Relay variables
/// when the project has a backend (the same names `previewEnv` builds in the frontend).
pub(crate) fn preview_env(access: Option<&crate::relay_ops::PreviewAccess>) -> Vec<(&'static str, String)> {
    let mut env: Vec<(&'static str, String)> = BASE_ENV.iter().map(|(k, v)| (*k, v.to_string())).collect();
    if let Some(access) = access {
        env.push(("PUBLIC_NOMI_URL", access.url.clone()));
        env.push(("PUBLIC_NOMI_KEY", access.anon_key.clone()));
        env.push(("NOMI_URL", access.url.clone()));
        env.push(("NOMI_ANON_KEY", access.anon_key.clone()));
        if let Some(key) = &access.service_key {
            env.push(("NOMI_SERVICE_KEY", key.clone()));
        }
    }
    env
}

/// The detached command line: the provider has no environment field, so the variables go in front.
fn start_command(env: &[(&str, String)], stack: &str) -> String {
    let vars: Vec<String> = env.iter().map(|(k, v)| format!("{k}={}", shell_quote(v))).collect();
    format!("cd {PROJECT_DIR} && exec env {} {}", vars.join(" "), dev_command(stack))
}

/// True when a `curl` status code means the server is answering.
fn answered(stdout: &str) -> bool {
    stdout.trim().trim_matches('\'').parse::<u16>().map(|c| (FIRST_ANSWER_CODE..=LAST_ANSWER_CODE).contains(&c)).unwrap_or(false)
}

impl ServerSteps {
    /// Install what the project needs, start its dev server, wait until it answers and return the
    /// address its port is served at.
    pub async fn ensure_dev_server(&self, project_id: Uuid, sandbox: &SandboxId) -> Result<PreviewUrl, DevServerError> {
        let (stack, on_relay, owner): (String, bool, Uuid) =
            sqlx::query_as("SELECT stack, backend = 'relay', user_id FROM projects WHERE id = $1").bind(project_id).fetch_one(&self.pool).await?;
        let access = if on_relay {
            self.wake_backend(project_id).await?;
            Some(self.relay_access(project_id, owner).await?)
        } else {
            None
        };
        if stack != STATIC_STACK {
            self.install_if_needed(project_id, sandbox).await?;
        }
        self.stop_previous(project_id, sandbox).await?;
        let handle = self.provider.exec_detached(sandbox, &start_command(&preview_env(access.as_ref()), &stack)).await?;
        {
            let mut conn = self.pool.acquire().await?;
            store::record_dev_server(&mut conn, project_id, &sandbox.0, Some(&handle.process_id)).await?;
        }
        self.wait_until_ready(project_id, sandbox, &handle).await?;
        Ok(self.provider.expose_port(sandbox, DEV_PORT).await?)
    }

    /// Wake the project's own Relay if it sleeps, and wait for it (the same wait the browser's
    /// preview does), so a sleeping or still-starting backend doesn't fail the start.
    async fn wake_backend(&self, project_id: Uuid) -> Result<(), DevServerError> {
        if let (Some(shipyard), Some(key)) = (self.shipyard.as_ref(), self.settings_key.as_ref()) {
            let deps = crate::relay_backend::Deps { pool: &self.pool, http: &self.http_client, shipyard, settings_key: key };
            crate::relay_backend::ensure_awake(&deps, project_id).await.map_err(DevServerError::Backend)?;
        }
        Ok(())
    }

    /// The dev backend's address and keys, as the owner's browser gets them (the service key is
    /// the project's own app key, which the owner already holds).
    pub(crate) async fn relay_access(&self, project_id: Uuid, owner: Uuid) -> Result<crate::relay_ops::PreviewAccess, DevServerError> {
        let config = match &self.relay_config {
            Some(config) => Some(config.clone()),
            None => crate::project_relays::for_request(&self.pool, self.settings_key.as_ref(), project_id, crate::relay_backend::DEV)
                .await
                .map_err(|(_, why)| DevServerError::Backend(why))?,
        };
        crate::relay_ops::preview_access(&self.pool, &self.http_client, config.as_ref(), project_id, owner).await.map_err(|(_, why)| DevServerError::Backend(why))
    }

    /// A stamp of the dependency files' versions; it changes when any of them is edited.
    async fn dependency_stamp(&self, project_id: Uuid) -> Result<String, DevServerError> {
        let manifest = crate::routes::project_runtime::load_manifest(&self.pool, project_id).await?;
        let found: BTreeMap<&str, String> = manifest
            .files
            .iter()
            .filter(|f| DEPENDENCY_FILES.contains(&f.path.as_str()))
            .map(|f| (f.path.as_str(), f.updated_at.to_rfc3339_opts(chrono::SecondsFormat::Micros, true)))
            .collect();
        Ok(found.iter().map(|(path, at)| format!("{path}={at}")).collect::<Vec<_>>().join(";"))
    }

    /// Install again when the dependency files changed since the last install (or the modules
    /// are missing); nothing for a stack with no build step. For a sandbox that is already
    /// running, after its files were synced.
    pub(crate) async fn reinstall_if_changed(&self, project_id: Uuid, sandbox: &SandboxId) -> Result<(), DevServerError> {
        let stack: String = sqlx::query_scalar("SELECT stack FROM projects WHERE id = $1").bind(project_id).fetch_one(&self.pool).await?;
        if stack == STATIC_STACK {
            return Ok(());
        }
        self.install_if_needed(project_id, sandbox).await
    }

    async fn install_if_needed(&self, project_id: Uuid, sandbox: &SandboxId) -> Result<(), DevServerError> {
        let stamp = self.dependency_stamp(project_id).await?;
        let stored: Option<String> =
            sqlx::query_scalar("SELECT deps_hash FROM project_sandboxes WHERE project_id = $1 AND sandbox_id = $2").bind(project_id).bind(&sandbox.0).fetch_optional(&self.pool).await?.flatten();
        let has_modules = self.provider.exec(sandbox, &format!("test -d {PROJECT_DIR}/node_modules"), QUICK_TIMEOUT).await?.exit_code == Some(0);
        if has_modules && stored.as_deref() == Some(stamp.as_str()) {
            return Ok(());
        }
        let done = self.provider.exec(sandbox, &format!("cd {PROJECT_DIR} && npm install"), INSTALL_TIMEOUT).await?;
        if done.exit_code != Some(0) {
            let why = match done.exit_code {
                Some(code) => format!("npm install exited with {code}: {}", tail(&format!("{}{}", done.stdout, done.stderr))),
                None => "npm install timed out".to_string(),
            };
            return Err(DevServerError::Install(why));
        }
        sqlx::query("UPDATE project_sandboxes SET deps_hash = $3 WHERE project_id = $1 AND sandbox_id = $2").bind(project_id).bind(&sandbox.0).bind(&stamp).execute(&self.pool).await?;
        Ok(())
    }

    /// Kill the dev server started earlier (whichever Nomi process started it), and free the
    /// port for any that was never recorded.
    async fn stop_previous(&self, project_id: Uuid, sandbox: &SandboxId) -> Result<(), DevServerError> {
        let previous = {
            let mut conn = self.pool.acquire().await?;
            store::dev_server_process(&mut conn, project_id, &sandbox.0).await?
        };
        if let Some(process_id) = previous {
            if let Err(e) = self.provider.kill_process(sandbox, &ProcessHandle { process_id }).await {
                tracing::debug!(error = %e, "sandbox dev server: the previous process was already gone");
            }
        }
        let free = format!("(command -v fuser >/dev/null 2>&1 && fuser -k {DEV_PORT}/tcp) || true");
        if let Err(e) = self.provider.exec(sandbox, &free, QUICK_TIMEOUT).await {
            tracing::debug!(error = %e, "sandbox dev server: couldn't free the dev port");
        }
        Ok(())
    }

    /// Poll until the dev server answers on its port; on failure the error carries its output.
    async fn wait_until_ready(&self, project_id: Uuid, sandbox: &SandboxId, handle: &ProcessHandle) -> Result<(), DevServerError> {
        let check = format!("curl -s -o /dev/null -w '%{{http_code}}' http://localhost:{DEV_PORT}/");
        let started = Instant::now();
        let (mut stdout_next, mut stderr_next) = (0, 0);
        let mut log = String::new();
        loop {
            let status = self.provider.process_status(sandbox, handle, stdout_next, stderr_next).await?;
            stdout_next = status.stdout_next;
            stderr_next = status.stderr_next;
            let printed = format!("{}{}", status.stdout, status.stderr);
            append_log(&mut log, &printed);
            {
                let mut conn = self.pool.acquire().await?;
                store::append_log(&mut conn, project_id, &printed, LOG_KEPT).await?;
            }
            if let Ok(out) = self.provider.exec(sandbox, &check, READY_CHECK_TIMEOUT).await {
                if answered(&out.stdout) {
                    return Ok(());
                }
            }
            if !status.running {
                return Err(DevServerError::NotReady(format!("it exited with {:?}. {log}", status.exit_code)));
            }
            if started.elapsed() >= self.devserver_timing.ready_timeout {
                return Err(DevServerError::NotReady(format!("no answer on port {DEV_PORT} after {}s. {log}", self.devserver_timing.ready_timeout.as_secs())));
            }
            tokio::time::sleep(self.devserver_timing.poll_interval).await;
        }
    }
}

/// The end of some command output, bounded by `LOG_KEPT`.
fn tail(text: &str) -> String {
    let mut log = String::new();
    append_log(&mut log, text);
    log
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::relay_ops::PreviewAccess;

    /// Every variable the sandbox may ever be given: the preview flags and the project's own
    /// dev-backend address and keys. Nothing of Nomi's own (JWT secret, database URL, settings
    /// key, LLM or Shipyard keys, the boat key) is on this list or read from the process.
    const ALLOWED_NAMES: &[&str] = &["NOMI_PREVIEW", "NODE_NO_WARNINGS", "PUBLIC_NOMI_URL", "PUBLIC_NOMI_KEY", "NOMI_URL", "NOMI_ANON_KEY", "NOMI_SERVICE_KEY"];

    fn access() -> PreviewAccess {
        PreviewAccess { url: "https://todo-dev.api.relay.test".into(), anon_key: "anon'; id #".into(), service_key: Some("svc $(id)".into()) }
    }

    #[test]
    fn the_sandbox_environment_holds_only_preview_and_project_backend_variables() {
        for env in [preview_env(None), preview_env(Some(&access()))] {
            for (name, _) in &env {
                assert!(ALLOWED_NAMES.contains(name), "{name} must not reach the sandbox");
            }
        }
        let names: Vec<&str> = preview_env(None).iter().map(|(k, _)| *k).collect();
        assert_eq!(names, vec!["NOMI_PREVIEW", "NODE_NO_WARNINGS"]);
    }

    #[test]
    fn environment_values_are_shell_quoted_in_the_start_command() {
        let line = start_command(&preview_env(Some(&access())), "vite");
        assert!(line.contains(r#"NOMI_ANON_KEY='anon'\''; id #'"#), "{line}");
        assert!(line.contains("NOMI_SERVICE_KEY='svc $(id)'"), "{line}");
        assert!(line.starts_with(&format!("cd {PROJECT_DIR} && exec env ")), "{line}");
    }
}
