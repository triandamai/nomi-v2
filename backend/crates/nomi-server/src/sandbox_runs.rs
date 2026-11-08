//! Runs Koda's queued commands inside a project's sandbox. For a browser project the open page
//! claims them from `project_runs`; for a sandbox project this does, one at a time in the order
//! they were queued, in the project directory, and finishes each with its exit code and output.
//!
//! A run claimed by a worker that then died is left `running`. After `RUN_LEASE` it is queued
//! again once (the marker `REQUEUED_NOTE` in its otherwise empty `output` records that, so no
//! column is needed); abandoned a second time it is `timed_out`, so a command that kills the
//! worker cannot loop forever.

use std::time::{Duration, Instant};

use nomi_agent_coding::runtime::{claim_runs, finish_run, parse_command, ClaimedRun, OUTPUT_KEPT};
use nomi_sandbox::{ExecOutput, SandboxId};
use uuid::Uuid;

use crate::sandbox_devserver::preview_env;
use crate::sandbox_steps::ServerSteps;
use crate::sandbox_sync::{shell_quote, PROJECT_DIR};

/// How long a run may stay `running` before its worker is presumed dead.
pub const RUN_LEASE: Duration = Duration::from_secs(15 * 60);
/// The longest one `exec` may run on the provider; a longer run goes the detached way.
pub const MAX_EXEC: Duration = Duration::from_secs(600);
/// How long a run gets when the request carried no timeout (the same as the tool's own wait).
pub const DEFAULT_RUN_TIMEOUT: Duration = Duration::from_secs(nomi_agent_coding::runtime::DEFAULT_TIMEOUT_SECS);
/// Left in the output of a run that was queued again after its worker died.
pub const REQUEUED_NOTE: &str = "(queued again: the worker running it stopped)";
/// Left in the output of a run that was abandoned twice.
const ABANDONED_NOTE: &str = "(gave up: the worker running it stopped twice)";
/// Added to the output of a command that ran out of time.
const STOPPED_NOTE: &str = "\n(stopped: took too long)";

/// The command line for the sandbox shell: into the project directory, with the preview
/// environment, then the program and its quoted arguments.
fn command_line(words: &[String], env: &[(&str, String)]) -> String {
    let vars: Vec<String> = env.iter().map(|(k, v)| format!("{k}={}", shell_quote(v))).collect();
    let args: Vec<String> = words[1..].iter().map(|w| shell_quote(w)).collect();
    format!("cd {PROJECT_DIR} && env {} {} {}", vars.join(" "), words[0], args.join(" "))
}

/// Check the stored words against the allow-list again, as a line the page's rules accept. The
/// queue is written by Nomi's own tool, but whatever is in it is checked before it reaches a shell.
fn validate(run: &ClaimedRun) -> Result<Vec<String>, String> {
    let mut words = vec![run.command.clone()];
    words.extend(run.args.iter().cloned());
    let mut quoted = Vec::with_capacity(words.len());
    for w in &words {
        quoted.push(match (w.contains('\''), w.contains('"')) {
            (true, true) => return Err("an argument with both kinds of quote isn't run".to_string()),
            (true, false) => format!("\"{w}\""),
            (false, _) if w.is_empty() || w.contains(char::is_whitespace) => format!("'{w}'"),
            _ => w.clone(),
        });
    }
    let parsed = parse_command(&quoted.join(" "))?;
    if parsed != words {
        return Err("the command doesn't survive being read back, so it isn't run".to_string());
    }
    Ok(words)
}

fn combined(out: &ExecOutput) -> String {
    format!("{}{}", out.stdout, out.stderr)
}

impl ServerSteps {
    /// Run the project's waiting commands in its sandbox, oldest first, until none are left.
    pub async fn execute_pending(&self, project_id: Uuid, sandbox: &SandboxId) {
        if let Err(e) = self.settle_abandoned(project_id).await {
            tracing::warn!(%project_id, error = %e, "sandbox runs: looking for abandoned runs failed");
            return;
        }
        let env = self.run_env(project_id).await;
        loop {
            let claimed = {
                let Ok(mut conn) = self.pool.acquire().await else { return };
                match claim_runs(&mut conn, project_id).await {
                    Ok(runs) => runs,
                    Err(e) => {
                        tracing::warn!(%project_id, error = %e, "sandbox runs: claiming runs failed");
                        return;
                    }
                }
            };
            if claimed.is_empty() {
                return;
            }
            for run in claimed {
                let (exit_code, output) = self.run_one(sandbox, &run, &env).await;
                match self.pool.acquire().await {
                    Ok(mut conn) => {
                        if let Err(e) = finish_run(&mut conn, project_id, run.id, exit_code, &output).await {
                            tracing::warn!(%project_id, run = %run.id, error = %e, "sandbox runs: saving the result failed");
                        }
                    }
                    Err(e) => tracing::warn!(%project_id, run = %run.id, error = %e, "sandbox runs: saving the result failed"),
                }
            }
        }
    }

    /// Queue again the runs left `running` past the lease, once; time out those already queued again.
    async fn settle_abandoned(&self, project_id: Uuid) -> Result<(), sqlx::Error> {
        let lease = RUN_LEASE.as_secs() as f64;
        sqlx::query("UPDATE project_runs SET status = 'timed_out', finished_at = now(), output = $3 WHERE project_id = $1 AND status = 'running' AND output IS NOT NULL AND started_at < now() - make_interval(secs => $2)")
            .bind(project_id)
            .bind(lease)
            .bind(ABANDONED_NOTE)
            .execute(&self.pool)
            .await?;
        sqlx::query("UPDATE project_runs SET status = 'pending', started_at = NULL, output = $3 WHERE project_id = $1 AND status = 'running' AND output IS NULL AND started_at < now() - make_interval(secs => $2)")
            .bind(project_id)
            .bind(lease)
            .bind(REQUEUED_NOTE)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// What the browser runtime passes to the commands it runs: the preview variables, plus the
    /// Relay ones when the project has a backend that answers (a sleeping one is not woken here).
    async fn run_env(&self, project_id: Uuid) -> Vec<(&'static str, String)> {
        let found: Option<(bool, Uuid)> = sqlx::query_as("SELECT backend = 'relay', user_id FROM projects WHERE id = $1").bind(project_id).fetch_optional(&self.pool).await.ok().flatten();
        let access = match found {
            Some((true, owner)) => self.relay_access(project_id, owner).await.map_err(|e| tracing::debug!(%project_id, error = %e, "sandbox runs: no backend keys for the command")).ok(),
            _ => None,
        };
        preview_env(access.as_ref())
    }

    /// Run one command to its end. The exit code is `None` when it never got one.
    async fn run_one(&self, sandbox: &SandboxId, run: &ClaimedRun, env: &[(&'static str, String)]) -> (Option<i32>, String) {
        let words = match validate(run) {
            Ok(words) => words,
            Err(why) => return (None, why),
        };
        let line = command_line(&words, env);
        let result = if self.run_timeout <= MAX_EXEC { self.provider.exec(sandbox, &line, self.run_timeout).await.map(|out| (out.exit_code, combined(&out))) } else { self.run_detached(sandbox, &line).await };
        match result {
            Ok((None, output)) => (None, format!("{output}{STOPPED_NOTE}")),
            Ok(done) => done,
            Err(e) => (None, format!("the sandbox couldn't run it: {e}")),
        }
    }

    /// Start the command in the background and poll it until it exits or runs out of time.
    async fn run_detached(&self, sandbox: &SandboxId, line: &str) -> Result<(Option<i32>, String), nomi_sandbox::SandboxError> {
        let handle = self.provider.exec_detached(sandbox, line).await?;
        let started = Instant::now();
        let (mut stdout_next, mut stderr_next) = (0, 0);
        let (mut stdout, mut stderr) = (String::new(), String::new());
        loop {
            let status = self.provider.process_status(sandbox, &handle, stdout_next, stderr_next).await?;
            stdout.push_str(&status.stdout);
            stderr.push_str(&status.stderr);
            stdout_next = status.stdout_next;
            stderr_next = status.stderr_next;
            keep_end(&mut stdout);
            keep_end(&mut stderr);
            if !status.running {
                return Ok((status.exit_code, format!("{stdout}{stderr}")));
            }
            if started.elapsed() >= self.run_timeout {
                if let Err(e) = self.provider.kill_process(sandbox, &handle).await {
                    tracing::debug!(error = %e, "sandbox runs: the timed-out process was already gone");
                }
                return Ok((None, format!("{stdout}{stderr}")));
            }
            tokio::time::sleep(self.devserver_timing.poll_interval).await;
        }
    }
}

/// Keep only the end of a growing output, so a chatty command can't fill memory.
fn keep_end(text: &mut String) {
    if text.len() > OUTPUT_KEPT {
        let mut cut = text.len() - OUTPUT_KEPT;
        while !text.is_char_boundary(cut) {
            cut += 1;
        }
        text.drain(..cut);
    }
}
