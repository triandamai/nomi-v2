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

use nomi_llm::ToolDefinition;

use crate::files::owned_project_id;
use crate::CODING_AGENT_TYPE;

/// A page that checked in this recently counts as open.
pub const RUNNER_FRESH_SECS: i64 = 30;
const DEFAULT_TIMEOUT_SECS: u64 = 180;
const MAX_TIMEOUT_SECS: u64 = 600;
const POLL_EVERY: Duration = Duration::from_millis(750);
/// Most output kept per run, and how much of it Koda sees.
const OUTPUT_KEPT: usize = 64 * 1024;
const OUTPUT_SHOWN: usize = 12 * 1024;
const PROGRAMS: &[&str] = &["npm", "npx", "node"];

pub const NOT_OPEN: &str = "The project isn't open in the user's browser right now, so commands can't run. Your files are \
     saved. Finish the work and complete the task: when the user opens the project, Nomi installs and checks it, and any \
     errors come back to you as a new task.";

pub fn run_command_tool_definition() -> ToolDefinition {
    ToolDefinition {
        name: "run_command".to_string(),
        description: "Run one command in the project's preview (Node in the user's browser) and get its output: \
            'npm install', 'npm install zod', 'npm run check', 'npm run build', 'npm run db:generate', 'npx ...', 'node ...'. \
            One command at a time, no shell operators. Don't start the dev server; Nomi runs it."
            .to_string(),
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
        return Err("Nomi runs the dev server itself; don't start one. Use 'npm run check' or 'npm run build' to test.".to_string());
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
    let project_id = owned_project_id(conn, user_id, &input).await?;
    let line = input.get("command").and_then(|v| v.as_str()).ok_or("command is required")?;
    let words = parse_command(line)?;
    let timeout = input.get("timeout_seconds").and_then(|v| v.as_u64()).unwrap_or(DEFAULT_TIMEOUT_SECS).clamp(10, MAX_TIMEOUT_SECS);
    if !runner_online(conn, project_id).await.map_err(|e| e.to_string())? {
        return Ok(NOT_OPEN.to_string());
    }
    let run_id: Uuid = sqlx::query_scalar("INSERT INTO project_runs (project_id, command, args) VALUES ($1, $2, $3) RETURNING id")
        .bind(project_id)
        .bind(&words[0])
        .bind(json!(words[1..]))
        .fetch_one(&mut **conn)
        .await
        .map_err(|e| e.to_string())?;

    let started = Instant::now();
    loop {
        tokio::time::sleep(POLL_EVERY).await;
        let (status, exit_code, output, online): (String, Option<i32>, Option<String>, bool) = sqlx::query_as(
            "SELECT r.status, r.exit_code, r.output, COALESCE(p.runner_seen_at > now() - make_interval(secs => $2), false) \
             FROM project_runs r JOIN projects p ON p.id = r.project_id WHERE r.id = $1",
        )
        .bind(run_id)
        .bind((RUNNER_FRESH_SECS + 15) as f64)
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

/// The open page checking in: marks it online and hands it the commands waiting to run.
/// Returns the files version and the claimed commands.
pub async fn check_in(conn: &mut PgConnection, project_id: Uuid) -> Result<(i64, Vec<ClaimedRun>), sqlx::Error> {
    let version: i64 = sqlx::query_scalar("UPDATE projects SET runner_seen_at = now() WHERE id = $1 RETURNING files_version")
        .bind(project_id)
        .fetch_one(&mut *conn)
        .await?;
    let mut rows: Vec<(Uuid, String, Value, chrono::DateTime<chrono::Utc>)> = sqlx::query_as(
        "UPDATE project_runs SET status = 'running', started_at = now() WHERE id IN ( \
             SELECT id FROM project_runs WHERE project_id = $1 AND status = 'pending' ORDER BY created_at FOR UPDATE SKIP LOCKED) \
         RETURNING id, command, args, created_at",
    )
    .bind(project_id)
    .fetch_all(&mut *conn)
    .await?;
    rows.sort_by_key(|r| r.3);
    let runs = rows
        .into_iter()
        .map(|(id, command, args, _)| ClaimedRun {
            id,
            command,
            args: args.as_array().map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect()).unwrap_or_default(),
        })
        .collect();
    Ok((version, runs))
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
}
