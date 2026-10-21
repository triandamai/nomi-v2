//! What a build actually showed when Koda finished: only checks that ran on the final files count.

use chrono::{DateTime, Duration, Utc};
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

use nomi_agent_coding::report::{build_report, BuildStatus, CheckOutcome, CheckSource};
use nomi_agent_core::Locale;

/// (session, project, when the build started)
async fn seed(pool: &PgPool, files_version: i64) -> (Uuid, Uuid, DateTime<Utc>) {
    let user_id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(pool).await.unwrap();
    let org_id: Uuid = sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Acme') RETURNING id").fetch_one(pool).await.unwrap();
    let session_id: Uuid = sqlx::query_scalar("INSERT INTO sessions (org_id, channel, chat_id) VALUES ($1, 'telegram', $2) RETURNING id")
        .bind(org_id)
        .bind(Uuid::new_v4().to_string())
        .fetch_one(pool)
        .await
        .unwrap();
    let project_id: Uuid = sqlx::query_scalar("INSERT INTO projects (user_id, session_id, name, files_version) VALUES ($1, $2, 'Tasks', $3) RETURNING id")
        .bind(user_id)
        .bind(session_id)
        .bind(files_version)
        .fetch_one(pool)
        .await
        .unwrap();
    let started = Utc::now() - Duration::minutes(10);
    // Written before this build: not one of its changes.
    sqlx::query("INSERT INTO project_files (project_id, path, size_bytes, updated_at) VALUES ($1, 'README.md', 10, $2)")
        .bind(project_id)
        .bind(started - Duration::hours(1))
        .execute(pool)
        .await
        .unwrap();
    (session_id, project_id, started)
}

async fn wrote(pool: &PgPool, project_id: Uuid, path: &str) {
    sqlx::query("INSERT INTO project_files (project_id, path, size_bytes) VALUES ($1, $2, 1)").bind(project_id).bind(path).execute(pool).await.unwrap();
}

/// A finished command: `script` is 'check', 'build' or 'install'.
async fn ran(pool: &PgPool, project_id: Uuid, script: &str, files_version: Option<i64>, status: &str, exit_code: Option<i32>, output: &str, at: DateTime<Utc>) {
    let args = if script == "install" { json!(["install"]) } else { json!(["run", script]) };
    sqlx::query(
        "INSERT INTO project_runs (project_id, command, args, status, exit_code, output, files_version, created_at, finished_at) \
         VALUES ($1, 'npm', $2, $3, $4, $5, $6, $7, $7)",
    )
    .bind(project_id)
    .bind(args)
    .bind(status)
    .bind(exit_code)
    .bind(output)
    .bind(files_version)
    .bind(at)
    .execute(pool)
    .await
    .unwrap();
}

#[sqlx::test(migrations = "../../migrations")]
async fn checks_that_passed_on_the_final_files_verify_the_build(pool: PgPool) {
    let (session_id, project_id, started) = seed(&pool, 3).await;
    wrote(&pool, project_id, "src/routes/+page.svelte").await;
    wrote(&pool, project_id, "package.json").await;
    sqlx::query("INSERT INTO messages (session_id, content, content_blocks, agent_display_name) VALUES ($1, 'deleted', $2, 'Koda')")
        .bind(session_id)
        .bind(json!([{ "kind": "file_delete", "project_id": project_id, "path": "src/old.ts", "previous_content": null }]))
        .execute(&pool)
        .await
        .unwrap();
    let now = Utc::now();
    ran(&pool, project_id, "install", Some(3), "done", Some(0), "added 120 packages", now).await;
    // An earlier failure, fixed since: the latest run is what counts.
    ran(&pool, project_id, "check", Some(2), "failed", Some(1), "Type error", now - Duration::minutes(2)).await;
    ran(&pool, project_id, "check", Some(3), "done", Some(0), "0 errors", now).await;
    ran(&pool, project_id, "build", Some(3), "done", Some(0), "built in 2s", now).await;

    let mut conn = pool.acquire().await.unwrap();
    let report = build_report(&mut conn, session_id, project_id, started).await.unwrap();
    assert_eq!(report.status, BuildStatus::Verified);
    assert_eq!(report.files_version, 3);
    assert_eq!(report.changed_files, vec!["package.json", "src/routes/+page.svelte"]);
    assert_eq!(report.deleted_files, vec!["src/old.ts"]);
    let outcomes: Vec<_> = report.checks.iter().map(|c| (c.command.as_str(), c.outcome, c.exit_code)).collect();
    assert_eq!(outcomes, vec![("npm run check", CheckOutcome::Passed, Some(0)), ("npm run build", CheckOutcome::Passed, Some(0))]);
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_failing_check_on_the_final_files_means_it_is_not_working_yet(pool: PgPool) {
    let (session_id, project_id, started) = seed(&pool, 3).await;
    let now = Utc::now();
    ran(&pool, project_id, "check", Some(3), "done", Some(0), "0 errors", now).await;
    ran(&pool, project_id, "build", Some(3), "failed", Some(1), "x\nerror: Could not resolve 'zod'", now).await;

    let mut conn = pool.acquire().await.unwrap();
    let report = build_report(&mut conn, session_id, project_id, started).await.unwrap();
    assert_eq!(report.status, BuildStatus::Failing);
    let build = &report.checks[1];
    assert_eq!((build.outcome, build.exit_code), (CheckOutcome::Failed, Some(1)));
    assert!(build.output_tail.as_deref().unwrap().ends_with("Could not resolve 'zod'"));
}

#[sqlx::test(migrations = "../../migrations")]
async fn checks_on_older_files_or_before_the_build_are_not_evidence(pool: PgPool) {
    let (session_id, project_id, started) = seed(&pool, 3).await;
    // Passed, then the files changed again.
    ran(&pool, project_id, "check", Some(2), "done", Some(0), "0 errors", Utc::now()).await;
    // Passed on these very files, but before this build started (and from before versions were recorded).
    ran(&pool, project_id, "build", None, "done", Some(0), "built", started - Duration::minutes(1)).await;

    let mut conn = pool.acquire().await.unwrap();
    let report = build_report(&mut conn, session_id, project_id, started).await.unwrap();
    assert_eq!(report.status, BuildStatus::Unverified);
    let outcomes: Vec<_> = report.checks.iter().map(|c| c.outcome).collect();
    assert_eq!(outcomes, vec![CheckOutcome::Stale, CheckOutcome::NotRun]);
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_command_that_never_finished_is_not_a_pass(pool: PgPool) {
    let (session_id, project_id, started) = seed(&pool, 1).await;
    ran(&pool, project_id, "check", Some(1), "timed_out", None, "", Utc::now()).await;
    ran(&pool, project_id, "build", Some(1), "done", Some(0), "built", Utc::now()).await;
    let mut conn = pool.acquire().await.unwrap();
    let report = build_report(&mut conn, session_id, project_id, started).await.unwrap();
    assert_eq!(report.status, BuildStatus::Unverified);
    assert_eq!(report.checks[0].outcome, CheckOutcome::TimedOut);
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_previews_own_check_of_the_final_files_counts(pool: PgPool) {
    let (session_id, project_id, started) = seed(&pool, 4).await;
    sqlx::query("UPDATE projects SET checked_version = 4, check_ok = true, check_output = '0 errors' WHERE id = $1")
        .bind(project_id)
        .execute(&pool)
        .await
        .unwrap();
    ran(&pool, project_id, "build", Some(4), "done", Some(0), "built", Utc::now()).await;
    let mut conn = pool.acquire().await.unwrap();
    let report = build_report(&mut conn, session_id, project_id, started).await.unwrap();
    assert_eq!(report.status, BuildStatus::Verified);
    assert_eq!((report.checks[0].outcome, report.checks[0].source), (CheckOutcome::Passed, CheckSource::Preview));
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_report_reads_as_one_line_in_the_persons_language(pool: PgPool) {
    let (session_id, project_id, started) = seed(&pool, 2).await;
    ran(&pool, project_id, "check", Some(2), "done", Some(0), "ok", Utc::now()).await;
    let mut conn = pool.acquire().await.unwrap();
    let report = build_report(&mut conn, session_id, project_id, started).await.unwrap();
    let line = report.summary_line(Locale::En);
    assert!(line.contains("npm run check passed") && line.contains("npm run build didn't run"), "{line}");
    assert!(!report.summary_line(Locale::Id).contains("didn't run"));
}

async fn delegation(pool: &PgPool, session_id: Uuid, project_id: Uuid, claimed_at: DateTime<Utc>) -> Uuid {
    let user_id: Uuid = sqlx::query_scalar("SELECT user_id FROM projects WHERE id = $1").bind(project_id).fetch_one(pool).await.unwrap();
    sqlx::query_scalar(
        "INSERT INTO agent_delegations (session_id, user_id, requesting_agent_type, target_agent_type, task, status, claimed_at) \
         VALUES ($1, $2, 'planning', 'coding', $3, 'processing', $4) RETURNING id",
    )
    .bind(session_id)
    .bind(user_id)
    .bind(format!("Project {project_id}: build plan version 1"))
    .bind(claimed_at)
    .fetch_one(pool)
    .await
    .unwrap()
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_finished_build_keeps_its_report_and_is_ready_only_if_nothing_failed(pool: PgPool) {
    use nomi_agent_coding::report::finish_build;

    let (session_id, project_id, started) = seed(&pool, 2).await;
    sqlx::query("UPDATE projects SET status = 'building' WHERE id = $1").bind(project_id).execute(&pool).await.unwrap();
    ran(&pool, project_id, "build", Some(2), "failed", Some(1), "error", Utc::now()).await;
    let failing = delegation(&pool, session_id, project_id, started).await;
    let mut conn = pool.acquire().await.unwrap();
    let report = finish_build(&mut conn, failing, session_id, project_id).await.unwrap();
    assert_eq!(report.status, BuildStatus::Failing);
    let (status, saved): (String, serde_json::Value) =
        sqlx::query_as("SELECT p.status, d.report FROM projects p, agent_delegations d WHERE p.id = $1 AND d.id = $2")
            .bind(project_id)
            .bind(failing)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(status, "building", "a failing build isn't ready");
    assert_eq!(saved["status"], "failing");
    assert_eq!(saved["checks"][1]["outcome"], "failed");

    // Koda fixes it in a later run: both checks pass on the new files.
    sqlx::query("UPDATE projects SET files_version = 3 WHERE id = $1").bind(project_id).execute(&pool).await.unwrap();
    let fixing = delegation(&pool, session_id, project_id, Utc::now()).await;
    ran(&pool, project_id, "check", Some(3), "done", Some(0), "ok", Utc::now()).await;
    ran(&pool, project_id, "build", Some(3), "done", Some(0), "ok", Utc::now()).await;
    let report = finish_build(&mut conn, fixing, session_id, project_id).await.unwrap();
    assert_eq!(report.status, BuildStatus::Verified);
    let status: String = sqlx::query_scalar("SELECT status FROM projects WHERE id = $1").bind(project_id).fetch_one(&pool).await.unwrap();
    assert_eq!(status, "ready");
}
