//! What an ask changed in the preview: the files written since a moment (the turn's start, or the
//! live deploy) and what the turn's recorded test run said. Koda's turn in a project's chat that
//! ends with changed files is announced on the chat's stream as `AskBuilt`; a publish request
//! carries the files changed since the live deploy.

use chrono::{DateTime, Utc};
use nomi_realtime::{StreamEnvelope, TestsOutcome};
use uuid::Uuid;

/// The most file paths one announcement or request carries.
pub const MAX_FILES_LISTED: usize = 200;

/// A command Koda ran in a project, as `project_runs` keeps it.
pub struct RecordedRun {
    pub command: String,
    pub args: Vec<String>,
    pub status: String,
}

/// Programs that are a test runner on their own.
const TEST_RUNNERS: [&str; 5] = ["vitest", "jest", "pytest", "mocha", "playwright"];
/// The word a package manager or build tool is given to run tests (`npm test`, `cargo test`).
const TEST_WORD: &str = "test";
/// A script named `test:unit`, `test:e2e`.
const TEST_SCRIPT_PREFIX: &str = "test:";

fn is_test_run(run: &RecordedRun) -> bool {
    let program = run.command.rsplit('/').next().unwrap_or(&run.command);
    TEST_RUNNERS.contains(&program) || run.args.iter().any(|a| a == TEST_WORD || a.starts_with(TEST_SCRIPT_PREFIX) || TEST_RUNNERS.contains(&a.as_str()))
}

/// What the turn's test runs said, oldest run first: the last finished one decides, and a turn
/// that ran none says nothing (null), never "passed".
pub fn tests_outcome(runs: &[RecordedRun]) -> Option<TestsOutcome> {
    runs.iter().rev().filter(|r| is_test_run(r)).find_map(|r| match r.status.as_str() {
        "done" => Some(TestsOutcome::Passed),
        "failed" | "timed_out" => Some(TestsOutcome::Failed),
        _ => None,
    })
}

/// The paths of files (path, last written) written after `since`, in path order; every file when
/// there is no `since`. At most [`MAX_FILES_LISTED`].
pub fn changed_since(files: &[(String, DateTime<Utc>)], since: Option<DateTime<Utc>>) -> Vec<String> {
    let mut paths: Vec<String> = files.iter().filter(|(_, at)| since.is_none_or(|since| *at > since)).map(|(path, _)| path.clone()).collect();
    paths.sort();
    paths.truncate(MAX_FILES_LISTED);
    paths
}

/// Every file of the project with when it was last written.
pub async fn project_files_written(pool: &sqlx::PgPool, project_id: Uuid) -> Result<Vec<(String, DateTime<Utc>)>, sqlx::Error> {
    sqlx::query_as("SELECT path, updated_at FROM project_files WHERE project_id = $1").bind(project_id).fetch_all(pool).await
}

/// Tells the chat's devices that an ask finished and changed files in the project of this chat:
/// the files written since the turn was claimed, and its recorded test run. Nothing is said for a
/// chat without a project or a turn that wrote nothing. Best effort, like the other announcements.
pub async fn announce(pool: &sqlx::PgPool, mqtt: &nomi_realtime::MqttPublisher, job_id: Uuid, session_id: Uuid) {
    match built_by(pool, job_id, session_id).await {
        Ok(Some((files, tests))) => {
            if let Err(e) = mqtt.publish(session_id, &StreamEnvelope::AskBuilt { job_id, files, tests }).await {
                tracing::debug!(error = %e, %session_id, "couldn't announce a built ask");
            }
        }
        Ok(None) => {}
        Err(e) => tracing::warn!(error = %e, %job_id, "couldn't tell what an ask built"),
    }
}

async fn built_by(pool: &sqlx::PgPool, job_id: Uuid, session_id: Uuid) -> Result<Option<(Vec<String>, Option<TestsOutcome>)>, sqlx::Error> {
    let project: Option<Uuid> = sqlx::query_scalar("SELECT id FROM projects WHERE session_id = $1 LIMIT 1").bind(session_id).fetch_optional(pool).await?;
    let started: Option<Option<DateTime<Utc>>> = sqlx::query_scalar("SELECT claimed_at FROM turn_jobs WHERE id = $1").bind(job_id).fetch_optional(pool).await?;
    let (Some(project_id), Some(Some(started))) = (project, started) else { return Ok(None) };
    let files = changed_since(&project_files_written(pool, project_id).await?, Some(started));
    if files.is_empty() {
        return Ok(None);
    }
    let rows: Vec<(String, sqlx::types::Json<Vec<String>>, String)> =
        sqlx::query_as("SELECT command, args, status FROM project_runs WHERE project_id = $1 AND created_at >= $2 ORDER BY created_at")
            .bind(project_id)
            .bind(started)
            .fetch_all(pool)
            .await?;
    let runs: Vec<RecordedRun> = rows.into_iter().map(|(command, args, status)| RecordedRun { command, args: args.0, status }).collect();
    Ok(Some((files, tests_outcome(&runs))))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    fn run(command: &str, args: &[&str], status: &str) -> RecordedRun {
        RecordedRun { command: command.to_string(), args: args.iter().map(|a| a.to_string()).collect(), status: status.to_string() }
    }

    #[test]
    fn a_turn_that_ran_no_tests_says_nothing() {
        assert_eq!(tests_outcome(&[]), None);
        assert_eq!(tests_outcome(&[run("npm", &["run", "build"], "done")]), None);
    }

    #[test]
    fn the_last_finished_test_run_decides() {
        assert_eq!(tests_outcome(&[run("npm", &["test"], "done")]), Some(TestsOutcome::Passed));
        assert_eq!(tests_outcome(&[run("npx", &["vitest", "run"], "failed")]), Some(TestsOutcome::Failed));
        assert_eq!(tests_outcome(&[run("pnpm", &["test:unit"], "timed_out")]), Some(TestsOutcome::Failed));
        // Fixed after a failure.
        assert_eq!(tests_outcome(&[run("npm", &["test"], "failed"), run("npm", &["test"], "done")]), Some(TestsOutcome::Passed));
        // A run still going doesn't decide, and neither does a non-test command after it.
        assert_eq!(tests_outcome(&[run("npm", &["test"], "failed"), run("npm", &["test"], "running"), run("npm", &["run", "build"], "done")]), Some(TestsOutcome::Failed));
        assert_eq!(tests_outcome(&[run("pytest", &[], "done")]), Some(TestsOutcome::Passed));
    }

    #[test]
    fn files_changed_since_a_moment_are_listed_by_path() {
        let live = Utc::now();
        let files = vec![
            ("src/b.ts".to_string(), live + Duration::seconds(5)),
            ("src/old.ts".to_string(), live - Duration::seconds(5)),
            ("src/a.ts".to_string(), live + Duration::seconds(1)),
        ];
        assert_eq!(changed_since(&files, Some(live)), ["src/a.ts", "src/b.ts"]);
        assert_eq!(changed_since(&files, None).len(), 3);
        assert!(changed_since(&files, Some(live + Duration::hours(1))).is_empty());
    }

    #[test]
    fn the_list_is_capped() {
        let now = Utc::now();
        let files: Vec<_> = (0..MAX_FILES_LISTED + 10).map(|i| (format!("f{i:04}"), now)).collect();
        assert_eq!(changed_since(&files, None).len(), MAX_FILES_LISTED);
    }
}
