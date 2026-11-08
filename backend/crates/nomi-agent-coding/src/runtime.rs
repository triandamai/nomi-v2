//! The bridge between Koda (on the server) and the project running in the user's browser
//! (WebContainer). The open project page checks in every few seconds (`check_in`), which both
//! marks it online and hands it any commands Koda queued; it posts each command's output back
//! (`finish_run`). Koda's run_command queues a command and waits for that output. When the
//! page opens a project it installs and checks it, and reports the result (`record_check`); a
//! failure goes to Koda as a new task, once per version of the files.

use std::time::{Duration, Instant};

use serde::Serialize;
use serde_json::{json, Value};
use sqlx::pool::PoolConnection;
use sqlx::{PgConnection, Postgres};
use uuid::Uuid;

use nomi_agent_core::prompts::{RUN_COMMAND_DESCRIPTION, SANDBOX_FAILED, SANDBOX_STARTING, SANDBOX_STOPPED_MID_RUN};
use nomi_agent_core::sandbox_ops::{NoSandboxOps, SandboxOps, WakeOutcome};
use nomi_llm::ToolDefinition;

use crate::files::owned_project_id;
use crate::CODING_AGENT_TYPE;

/// A page that checked in this recently counts as open.
pub const RUNNER_FRESH_SECS: i64 = 30;
/// How often the open page's check-in writes `runner_seen_at` at most; well inside RUNNER_FRESH_SECS,
/// so a skipped write (the row was busy) still leaves the page online.
pub const RUNNER_TOUCH_EVERY_SECS: i64 = 10;
pub const DEFAULT_TIMEOUT_SECS: u64 = 180;
const MAX_TIMEOUT_SECS: u64 = 600;
const POLL_EVERY: Duration = Duration::from_millis(750);
/// Most output kept per run, and how much of it Koda sees.
pub const OUTPUT_KEPT: usize = 64 * 1024;
const OUTPUT_SHOWN: usize = 12 * 1024;
/// How long a command waits for the project's sandbox to come up before Koda is told it is still
/// starting. A cold start creates or resumes the sandbox, copies the files in and starts the dev server.
pub const SANDBOX_START_WAIT: Duration = Duration::from_secs(90);
const PROGRAMS: &[&str] = &["npm", "npx", "node"];

pub const NOT_OPEN: &str = "The project isn't open in the user's browser right now, so commands can't run. Your files are \
     saved. Finish the work and complete the task: when the user opens the project, Nomi installs and checks it, and any \
     errors come back to you as a new task.";

pub fn run_command_tool_definition() -> ToolDefinition {
    ToolDefinition {
        name: "run_command".to_string(),
        description: RUN_COMMAND_DESCRIPTION.to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "project_id": {"type": "string"},
                "command": {"type": "string", "description": "e.g. 'npm run check'"},
                "timeout_seconds": {"type": "integer", "description": "Default 180, at most 600"}
            },
            "required": ["project_id", "command"]
        }),
    }
}

/// Splits a command line into program and arguments, allowing quotes but no shell operators.
pub fn parse_command(line: &str) -> Result<Vec<String>, String> {
    let line = line.trim();
    if line.chars().any(|c| matches!(c, '&' | '|' | ';' | '>' | '<' | '`' | '$' | '\n' | '\r')) {
        return Err("Run one command at a time, without shell operators (&&, |, ;, >, $).".to_string());
    }
    let mut words = Vec::new();
    let mut current = String::new();
    let mut quote: Option<char> = None;
    let mut started = false;
    for c in line.chars() {
        match (quote, c) {
            (Some(q), c) if c == q => quote = None,
            (Some(_), c) => current.push(c),
            (None, '\'' | '"') => {
                quote = Some(c);
                started = true;
            }
            (None, c) if c.is_whitespace() => {
                if started {
                    words.push(std::mem::take(&mut current));
                    started = false;
                }
            }
            (None, c) => {
                current.push(c);
                started = true;
            }
        }
    }
    if quote.is_some() {
        return Err("unclosed quote".to_string());
    }
    if started {
        words.push(current);
    }
    let Some(program) = words.first() else { return Err("command is empty".to_string()) };
    if !PROGRAMS.contains(&program.as_str()) {
        return Err(format!("only {} can run in the preview", PROGRAMS.join(", ")));
    }
    let args: Vec<&str> = words.iter().skip(1).map(String::as_str).collect();
    let starts_server = match program.as_str() {
        "npm" => matches!(args.as_slice(), ["start", ..] | ["run", "dev" | "start" | "preview", ..]),
        "npx" => matches!(args.first(), Some(&"vite") if !args.contains(&"build")),
        _ => args.contains(&"--watch"),
    };
    if starts_server {
        return Err("Nomi runs the dev server ('npm run dev') itself; don't start one. Edit files directly to hot-reload in the preview.".to_string());
    }
    if program == "npx" && args.windows(2).any(|w| w[0] == "drizzle-kit" && matches!(w[1], "push" | "migrate")) {
        return Err("drizzle-kit push/migrate need a database server. Run 'npm run db:generate'; the app applies migrations on start.".to_string());
    }
    Ok(words)
}

pub async fn runner_online(conn: &mut PgConnection, project_id: Uuid) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar("SELECT COALESCE(runner_seen_at > now() - make_interval(secs => $2), false) FROM projects WHERE id = $1")
        .bind(project_id)
        .bind(RUNNER_FRESH_SECS as f64)
        .fetch_one(&mut *conn)
        .await
}

/// The end of `output`, at most `max` bytes, cut at a character boundary.
pub fn tail(output: &str, max: usize) -> &str {
    if output.len() <= max {
        return output;
    }
    let mut start = output.len() - max;
    while !output.is_char_boundary(start) {
        start += 1;
    }
    &output[start..]
}

pub async fn run_command(conn: &mut PoolConnection<Postgres>, user_id: Uuid, input: Value) -> Result<String, String> {
    run_command_with(conn, &NoSandboxOps, user_id, input).await
}

/// `run_command`, where a project that runs in a sandbox wakes it and waits for it instead of
/// needing its page open in the browser; the run is queued and waited for the same way.
pub async fn run_command_with(conn: &mut PoolConnection<Postgres>, sandbox: &dyn SandboxOps, user_id: Uuid, input: Value) -> Result<String, String> {
    let project_id = owned_project_id(conn, user_id, &input).await?;
    let line = input.get("command").and_then(|v| v.as_str()).ok_or("command is required")?;
    let words = parse_command(line)?;
    let timeout = input.get("timeout_seconds").and_then(|v| v.as_u64()).unwrap_or(DEFAULT_TIMEOUT_SECS).clamp(10, MAX_TIMEOUT_SECS);
    let in_sandbox = sandbox.uses_sandbox(project_id).await;
    if in_sandbox {
        match sandbox.wake_and_wait(project_id, SANDBOX_START_WAIT).await {
            WakeOutcome::Running => {}
            WakeOutcome::Starting => return Ok(SANDBOX_STARTING.to_string()),
            WakeOutcome::Failed(why) => return Ok(SANDBOX_FAILED.replace("{why}", &why)),
        }
    } else if !runner_online(conn, project_id).await.map_err(|e| e.to_string())? {
        return Ok(NOT_OPEN.to_string());
    }
    let run_id: Uuid = sqlx::query_scalar(
        "INSERT INTO project_runs (project_id, command, args, files_version) \
         SELECT $1, $2, $3, files_version FROM projects WHERE id = $1 RETURNING id",
    )
        .bind(project_id)
        .bind(&words[0])
        .bind(json!(words[1..]))
        .fetch_one(&mut **conn)
        .await
        .map_err(|e| e.to_string())?;

    let started = Instant::now();
    loop {
        tokio::time::sleep(POLL_EVERY).await;
        // The page's check-in decides whether a browser project is online; a sandbox project is
        // online while its sandbox runs, so a closed tab doesn't end a command there.
        let (status, exit_code, output, online): (String, Option<i32>, Option<String>, bool) = sqlx::query_as(
            "SELECT r.status, r.exit_code, r.output, \
                 CASE WHEN $3 THEN COALESCE((SELECT s.state = 'running' FROM project_sandboxes s WHERE s.project_id = p.id), false) \
                 ELSE COALESCE(p.runner_seen_at > now() - make_interval(secs => $2), false) END \
             FROM project_runs r JOIN projects p ON p.id = r.project_id WHERE r.id = $1",
        )
        .bind(run_id)
        .bind((RUNNER_FRESH_SECS + 15) as f64)
        .bind(in_sandbox)
        .fetch_one(&mut **conn)
        .await
        .map_err(|e| e.to_string())?;
        match status.as_str() {
            "done" | "failed" => {
                let output = output.unwrap_or_default();
                let shown = tail(&output, OUTPUT_SHOWN);
                let cut = if shown.len() < output.len() { "… (earlier output cut)\n" } else { "" };
                return Ok(format!("$ {line}\nexit code {}\n{cut}{shown}", exit_code.map_or("?".to_string(), |c| c.to_string())));
            }
            _ if !online || started.elapsed() > Duration::from_secs(timeout) => {
                sqlx::query("UPDATE project_runs SET status = 'timed_out', finished_at = now() WHERE id = $1 AND status IN ('pending', 'running')")
                    .bind(run_id)
                    .execute(&mut **conn)
                    .await
                    .map_err(|e| e.to_string())?;
                return Ok(if online {
                    format!("$ {line}\nstopped waiting after {timeout}s with no result.")
                } else if in_sandbox {
                    format!("$ {line}\n{SANDBOX_STOPPED_MID_RUN}")
                } else {
                    format!("$ {line}\nThe user closed the project before it finished. {NOT_OPEN}")
                });
            }
            _ => {}
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ClaimedRun {
    pub id: Uuid,
    pub command: String,
    pub args: Vec<String>,
}

/// Updates `runner_seen_at` to mark the project page online.
/// Throttled so it only writes if `runner_seen_at` is NULL or older than RUNNER_TOUCH_EVERY_SECS.
/// When another transaction holds the project row (saving files, filing an issue) it skips the
/// write rather than waiting on the lock: the next check-in writes it, well before the page counts
/// as offline.
pub async fn touch_runner(conn: &mut PgConnection, project_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE projects SET runner_seen_at = now() WHERE id = ( \
             SELECT id FROM projects WHERE id = $1 \
             AND (runner_seen_at IS NULL OR runner_seen_at < now() - make_interval(secs => $2)) \
             FOR UPDATE SKIP LOCKED)",
    )
    .bind(project_id)
    .bind(RUNNER_TOUCH_EVERY_SECS as f64)
    .execute(conn)
    .await?;
    Ok(())
}

/// Reads the current files version of the project without locking the row.
pub async fn files_version(conn: &mut PgConnection, project_id: Uuid) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar("SELECT files_version FROM projects WHERE id = $1")
        .bind(project_id)
        .fetch_one(conn)
        .await
}

/// Claims any pending commands for the project from `project_runs`.
pub async fn claim_runs(conn: &mut PgConnection, project_id: Uuid) -> Result<Vec<ClaimedRun>, sqlx::Error> {
    let mut rows: Vec<(Uuid, String, Value, chrono::DateTime<chrono::Utc>)> = sqlx::query_as(
        "UPDATE project_runs SET status = 'running', started_at = now() WHERE id IN ( \
             SELECT id FROM project_runs WHERE project_id = $1 AND status = 'pending' ORDER BY created_at FOR UPDATE SKIP LOCKED) \
         RETURNING id, command, args, created_at",
    )
    .bind(project_id)
    .fetch_all(&mut *conn)
    .await?;
    rows.sort_by_key(|r| r.3);
    Ok(rows
        .into_iter()
        .map(|(id, command, args, _)| ClaimedRun {
            id,
            command,
            args: args.as_array().map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect()).unwrap_or_default(),
        })
        .collect())
}

/// Polls for the current files version and any claimed commands without taking row locks on `projects`.
pub async fn poll(conn: &mut PgConnection, project_id: Uuid) -> Result<(i64, Vec<ClaimedRun>), sqlx::Error> {
    let version = files_version(conn, project_id).await?;
    let runs = claim_runs(conn, project_id).await?;
    Ok((version, runs))
}

/// The open page checking in: marks it online (throttled) and hands it the commands waiting to run.
/// Returns the files version and the claimed commands.
pub async fn check_in(conn: &mut PgConnection, project_id: Uuid) -> Result<(i64, Vec<ClaimedRun>), sqlx::Error> {
    touch_runner(conn, project_id).await?;
    poll(conn, project_id).await
}

/// A command's result from the page. False when there was no such running command.
pub async fn finish_run(conn: &mut PgConnection, project_id: Uuid, run_id: Uuid, exit_code: Option<i32>, output: &str) -> Result<bool, sqlx::Error> {
    let status = if exit_code == Some(0) { "done" } else { "failed" };
    let updated = sqlx::query(
        "UPDATE project_runs SET status = $3, exit_code = $4, output = $5, finished_at = now() \
         WHERE id = $1 AND project_id = $2 AND status = 'running'",
    )
    .bind(run_id)
    .bind(project_id)
    .bind(status)
    .bind(exit_code)
    .bind(tail(output, OUTPUT_KEPT))
    .execute(&mut *conn)
    .await?
    .rows_affected();
    Ok(updated > 0)
}

/// What to hand Koda after a failed check: the chat it belongs to and the task.
#[derive(Debug, PartialEq)]
pub struct FixRequest {
    pub session_id: Uuid,
    pub task: String,
}

/// Records the install-and-check the page ran on opening the project. When it failed, Koda isn't
/// already working on the project, and this version wasn't handed over before, returns the task
/// to give him.
pub async fn record_check(conn: &mut PgConnection, project_id: Uuid, files_version: i64, ok: bool, output: &str) -> Result<Option<FixRequest>, sqlx::Error> {
    let output = tail(output, OUTPUT_KEPT);
    let (session_id, current_version, already_requested): (Uuid, i64, Option<i64>) = sqlx::query_as(
        "UPDATE projects SET checked_version = $2, check_ok = $3, check_output = $4 WHERE id = $1 \
         RETURNING session_id, files_version, fix_requested_version",
    )
    .bind(project_id)
    .bind(files_version)
    .bind(ok)
    .bind(output)
    .fetch_one(&mut *conn)
    .await?;
    // A check of files that already changed since is stale: Koda may have fixed it.
    if ok || files_version != current_version || already_requested == Some(files_version) {
        return Ok(None);
    }
    let busy: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM agent_delegations WHERE session_id = $1 AND target_agent_type = $2 AND status IN ('pending', 'processing'))",
    )
    .bind(session_id)
    .bind(CODING_AGENT_TYPE)
    .fetch_one(&mut *conn)
    .await?;
    if busy {
        return Ok(None);
    }
    sqlx::query("UPDATE projects SET fix_requested_version = $2 WHERE id = $1").bind(project_id).bind(files_version).execute(&mut *conn).await?;
    Ok(Some(FixRequest {
        session_id,
        task: format!(
            "Project {project_id}: When the user opened the project, installing and checking it failed. Find the cause, fix it, \
             then run 'npm run check' again until it passes. The output:\n```\n{}\n```",
            tail(output, 8 * 1024)
        ),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn commands_split_like_a_shell_without_its_operators() {
        assert_eq!(parse_command("npm install zod").unwrap(), ["npm", "install", "zod"]);
        assert_eq!(parse_command("node -e 'console.log(1 + 1)'").unwrap(), ["node", "-e", "console.log(1 + 1)"]);
        assert_eq!(parse_command("  npx  tsc   --noEmit ").unwrap(), ["npx", "tsc", "--noEmit"]);
        assert!(parse_command("npm install && npm run build").is_err());
        assert!(parse_command("cat /etc/passwd").is_err());
        assert!(parse_command("rm -rf /").is_err());
        assert!(parse_command("npm run check > out.txt").is_err());
        assert!(parse_command("node -e 'unterminated").is_err());
        assert!(parse_command("").is_err());
    }

    #[test]
    fn dev_servers_and_database_pushes_are_refused() {
        assert!(parse_command("npm run dev").is_err());
        assert!(parse_command("npm start").is_err());
        assert!(parse_command("npx vite").is_err());
        assert!(parse_command("npx drizzle-kit push").is_err());
        assert!(parse_command("npx vite build").is_ok());
        assert!(parse_command("npm run build").is_ok());
    }

    #[test]
    fn tail_keeps_the_end_on_a_character_boundary() {
        assert_eq!(tail("hello", 10), "hello");
        assert_eq!(tail("hello world", 5), "world");
        assert_eq!(tail("ééé", 3), "é");
    }

    /// Answers `wake_and_wait` with `outcome` and counts the calls.
    struct FakeSandbox {
        sandboxed: bool,
        outcome: WakeOutcome,
        woken: std::sync::Mutex<Vec<Duration>>,
    }

    impl FakeSandbox {
        fn new(sandboxed: bool, outcome: WakeOutcome) -> Self {
            Self { sandboxed, outcome, woken: Default::default() }
        }
    }

    #[async_trait::async_trait]
    impl SandboxOps for FakeSandbox {
        async fn uses_sandbox(&self, _project_id: Uuid) -> bool {
            self.sandboxed
        }
        async fn wake_and_wait(&self, _project_id: Uuid, timeout: Duration) -> WakeOutcome {
            self.woken.lock().unwrap().push(timeout);
            self.outcome.clone()
        }
    }

    async fn seed(pool: &sqlx::PgPool) -> (Uuid, Uuid) {
        let user_id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(pool).await.unwrap();
        let org_id: Uuid = sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Acme') RETURNING id").fetch_one(pool).await.unwrap();
        let session_id: Uuid = sqlx::query_scalar("INSERT INTO sessions (org_id, channel, chat_id) VALUES ($1, 'telegram', $2) RETURNING id")
            .bind(org_id)
            .bind(Uuid::new_v4().to_string())
            .fetch_one(pool)
            .await
            .unwrap();
        let project_id: Uuid = sqlx::query_scalar("INSERT INTO projects (user_id, session_id, name, runtime) VALUES ($1, $2, 'Tasks', 'sandbox') RETURNING id")
            .bind(user_id)
            .bind(session_id)
            .fetch_one(pool)
            .await
            .unwrap();
        (user_id, project_id)
    }

    fn input(project_id: Uuid) -> Value {
        json!({ "project_id": project_id.to_string(), "command": "npm run check", "timeout_seconds": 10 })
    }

    #[sqlx::test(migrations = "../../migrations")]
    async fn sandbox_project_wakes_a_stopped_sandbox_and_runs_the_command(pool: sqlx::PgPool) {
        let (user_id, project_id) = seed(&pool).await;
        sqlx::query("INSERT INTO project_sandboxes (project_id, provider, desired, state) VALUES ($1, 'boat', 'running', 'running')").bind(project_id).execute(&pool).await.unwrap();
        // The sandbox worker answers the queued run; no browser page ever checks in.
        let worker_pool = pool.clone();
        let worker = tokio::spawn(async move {
            loop {
                let id: Option<Uuid> = sqlx::query_scalar("UPDATE project_runs SET status = 'done', exit_code = 0, output = 'ok' WHERE status = 'pending' RETURNING id")
                    .fetch_optional(&worker_pool)
                    .await
                    .unwrap();
                if id.is_some() {
                    return;
                }
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
        });
        let ops = FakeSandbox::new(true, WakeOutcome::Running);
        let mut conn = pool.acquire().await.unwrap();
        let out = run_command_with(&mut conn, &ops, user_id, input(project_id)).await.unwrap();
        worker.await.unwrap();
        assert_eq!(out, "$ npm run check\nexit code 0\nok");
        assert_eq!(*ops.woken.lock().unwrap(), [SANDBOX_START_WAIT]);
    }

    #[sqlx::test(migrations = "../../migrations")]
    async fn start_wait_expiry_returns_the_starting_message_not_not_open(pool: sqlx::PgPool) {
        let (user_id, project_id) = seed(&pool).await;
        let mut conn = pool.acquire().await.unwrap();
        let out = run_command_with(&mut conn, &FakeSandbox::new(true, WakeOutcome::Starting), user_id, input(project_id)).await.unwrap();
        assert_eq!(out, SANDBOX_STARTING);
        assert_ne!(out, NOT_OPEN);
        let failed = run_command_with(&mut conn, &FakeSandbox::new(true, WakeOutcome::Failed("no capacity".into())), user_id, input(project_id)).await.unwrap();
        assert!(failed.contains("no capacity") && failed != NOT_OPEN, "{failed}");
        let queued: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM project_runs").fetch_one(&pool).await.unwrap();
        assert_eq!(queued, 0, "nothing was queued for a sandbox that isn't up");
    }

    #[sqlx::test(migrations = "../../migrations")]
    async fn browser_projects_behave_exactly_as_before(pool: sqlx::PgPool) {
        let (user_id, project_id) = seed(&pool).await;
        sqlx::query("UPDATE projects SET runtime = 'browser' WHERE id = $1").bind(project_id).execute(&pool).await.unwrap();
        let ops = FakeSandbox::new(false, WakeOutcome::Running);
        let mut conn = pool.acquire().await.unwrap();
        let out = run_command_with(&mut conn, &ops, user_id, input(project_id)).await.unwrap();
        assert_eq!(out, NOT_OPEN);
        assert!(ops.woken.lock().unwrap().is_empty(), "a browser project never wakes a sandbox");
        assert_eq!(run_command(&mut conn, user_id, input(project_id)).await.unwrap(), NOT_OPEN);
    }
}
