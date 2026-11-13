//! Koda's commands for sandbox projects: run in the sandbox one at a time, finished with the
//! exit code and output, abandoned ones requeued once.

mod support;

use std::sync::Arc;
use std::time::Duration;

use nomi_agent_coding::runtime::{run_command_with, OUTPUT_KEPT};
use nomi_agent_core::prompts::SANDBOX_STARTING;
use nomi_sandbox::{CreateSpec, ExecOutput, FakeProvider, SandboxId, SandboxProvider, Size};
use nomi_server::sandbox_devserver::DevServerTiming;
use nomi_server::sandbox_runs::{claim_next, RUN_LEASE, REQUEUED_NOTE};
use nomi_server::sandbox_manager::ProjectSteps;
use nomi_server::sandbox_ops::ServerSandboxOps;
use nomi_server::sandbox_steps::ServerSteps;
use nomi_server::sandbox_store as store;
use serde_json::json;
use sqlx::PgPool;
use support::App;
use uuid::Uuid;

const PROVIDER: &str = "boat";
const FAST_POLL: Duration = Duration::from_millis(5);
/// Longer than one `exec` may run, so the command goes the detached way.
const LONG_RUN: Duration = Duration::from_secs(700);
/// Seconds past the lease.
const PAST_LEASE_SECS: f64 = RUN_LEASE.as_secs() as f64 + 60.0;

struct Fixture {
    pool: PgPool,
    provider: Arc<FakeProvider>,
    project: Uuid,
    sandbox: SandboxId,
}

fn out(code: i32, stdout: &str, stderr: &str) -> ExecOutput {
    ExecOutput { exit_code: Some(code), stdout: stdout.to_string(), stderr: stderr.to_string() }
}

async fn fixture(pool: PgPool) -> Fixture {
    let app = App::new(pool.clone());
    let owner = app.person(&format!("owner-{}@runs.test", Uuid::new_v4())).await;
    let project = app.project(&owner, "runs").await.0;
    sqlx::query("UPDATE projects SET runtime = 'sandbox' WHERE id = $1").bind(project).execute(&pool).await.unwrap();
    let provider = Arc::new(FakeProvider::new());
    let sandbox = provider
        .create(CreateSpec { size: Size::Small, ttl: nomi_sandbox::Ttl::ProviderDefault, from_snapshot: None, idempotency_key: "k".into(), labels: Default::default() })
        .await
        .unwrap()
        .id;
    let mut conn = pool.acquire().await.unwrap();
    store::want_running(&mut conn, project, PROVIDER).await.unwrap();
    drop(conn);
    sqlx::query("UPDATE project_sandboxes SET sandbox_id = $2, state = 'running' WHERE project_id = $1").bind(project).bind(&sandbox.0).execute(&pool).await.unwrap();
    Fixture { pool, provider, project, sandbox }
}

impl Fixture {
    fn steps(&self) -> ServerSteps {
        ServerSteps::new(self.pool.clone(), nomi_test_support::test_project_storage(), self.provider.clone()).with_devserver_timing(DevServerTiming { ready_timeout: Duration::from_secs(5), poll_interval: FAST_POLL })
    }

    async fn queue(&self, command: &str, args: &[&str], age_secs: i64) -> Uuid {
        sqlx::query_scalar("INSERT INTO project_runs (project_id, command, args, created_at) VALUES ($1, $2, $3, now() - make_interval(secs => $4)) RETURNING id")
            .bind(self.project)
            .bind(command)
            .bind(json!(args))
            .bind(age_secs as f64)
            .fetch_one(&self.pool)
            .await
            .unwrap()
    }

    async fn run(&self, id: Uuid) -> (String, Option<i32>, Option<String>) {
        sqlx::query_as("SELECT status, exit_code, output FROM project_runs WHERE id = $1").bind(id).fetch_one(&self.pool).await.unwrap()
    }

    fn execs(&self) -> Vec<String> {
        self.provider.calls().into_iter().filter(|c| c.starts_with("exec ")).collect()
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn runs_execute_in_order_and_finish_with_exit_code_and_output(pool: PgPool) {
    let f = fixture(pool).await;
    // Queued newest-first on purpose: created_at decides the order.
    let third = f.queue("node", &["c.js"], 10).await;
    let second = f.queue("npm", &["run", "build"], 20).await;
    let first = f.queue("npm", &["install"], 30).await;
    f.provider.on_exec("'install'", out(0, "installed", ""));
    f.provider.on_exec("'build'", out(1, "", "boom"));
    f.steps().execute_pending(f.project, &f.sandbox).await;
    let execs = f.execs();
    assert_eq!(execs.len(), 3, "{execs:?}");
    assert!(execs[0].contains("cd /home/user/project && ") && execs[0].contains("npm 'install'"), "{}", execs[0]);
    assert!(execs[1].contains("'build'") && execs[2].contains("c.js"));
    assert_eq!(f.run(first).await, ("done".into(), Some(0), Some("installed".into())));
    assert_eq!(f.run(second).await, ("failed".into(), Some(1), Some("boom".into())));
    assert_eq!(f.run(third).await.0, "done");
    assert!(execs[0].contains("NOMI_PREVIEW='1'"), "runs get the preview environment");
}

#[sqlx::test(migrations = "../../migrations")]
async fn output_is_tailed_to_output_kept(pool: PgPool) {
    let f = fixture(pool).await;
    let id = f.queue("npm", &["test"], 1).await;
    let big = format!("{}END", "x".repeat(OUTPUT_KEPT + 1000));
    f.provider.on_exec("'test'", out(0, &big, ""));
    f.steps().execute_pending(f.project, &f.sandbox).await;
    let (_, _, output) = f.run(id).await;
    let output = output.unwrap();
    assert_eq!(output.len(), OUTPUT_KEPT);
    assert!(output.ends_with("END"));
}

#[sqlx::test(migrations = "../../migrations")]
async fn long_command_uses_detached_and_polls_until_exit(pool: PgPool) {
    let f = fixture(pool).await;
    let id = f.queue("npm", &["run", "build"], 1).await;
    f.provider.set_process_logs("built ok", "a warning");
    f.provider.exit_processes_after(2, 0);
    f.steps().with_run_timeout(LONG_RUN).execute_pending(f.project, &f.sandbox).await;
    let calls = f.provider.calls();
    assert!(calls.iter().any(|c| c.starts_with("exec_detached") && c.contains("'build'")), "{calls:?}");
    assert!(calls.iter().filter(|c| c.starts_with("process_status")).count() >= 3);
    assert!(f.execs().is_empty(), "nothing ran through exec");
    let (status, code, output) = f.run(id).await;
    assert_eq!((status.as_str(), code), ("done", Some(0)));
    let output = output.unwrap();
    assert!(output.contains("built ok") && output.contains("a warning"), "{output}");
}

#[sqlx::test(migrations = "../../migrations")]
async fn abandoned_running_run_is_requeued_after_lease(pool: PgPool) {
    let f = fixture(pool).await;
    let stale = f.queue("npm", &["install"], 100).await;
    let again = f.queue("npm", &["test"], 80).await;
    sqlx::query("UPDATE project_runs SET status = 'running', started_at = now() - make_interval(secs => $2) WHERE id = $1").bind(stale).bind(PAST_LEASE_SECS).execute(&f.pool).await.unwrap();
    sqlx::query("UPDATE project_runs SET status = 'running', output = $3, started_at = now() - make_interval(secs => $2) WHERE id = $1")
        .bind(again)
        .bind(PAST_LEASE_SECS)
        .bind(REQUEUED_NOTE)
        .execute(&f.pool)
        .await
        .unwrap();
    f.steps().execute_pending(f.project, &f.sandbox).await;
    assert_eq!(f.run(stale).await.0, "done", "requeued once, then run");
    assert_eq!(f.run(again).await.0, "timed_out", "abandoned a second time");
    assert_eq!(f.execs().len(), 1);
}

#[sqlx::test(migrations = "../../migrations")]
async fn command_failure_is_finished_as_failed(pool: PgPool) {
    let f = fixture(pool).await;
    let bad = f.queue("rm", &["-rf", "/"], 20).await;
    let chained = f.queue("npm", &["install", "x; rm -rf /"], 15).await;
    let slow = f.queue("npm", &["run", "slow"], 10).await;
    f.provider.on_exec("'slow'", ExecOutput { exit_code: None, stdout: "partial".into(), stderr: String::new() });
    f.steps().execute_pending(f.project, &f.sandbox).await;
    let (status, code, output) = f.run(bad).await;
    assert_eq!((status.as_str(), code), ("failed", None));
    assert!(output.unwrap().contains("only npm, npx, node"));
    assert_eq!(f.run(chained).await.0, "failed");
    let (status, code, output) = f.run(slow).await;
    assert_eq!((status.as_str(), code), ("failed", None));
    assert!(output.unwrap().starts_with("partial"));
    assert_eq!(f.execs().len(), 1, "only the allowed command reached the sandbox");
}

async fn statuses(f: &Fixture) -> Vec<String> {
    sqlx::query_scalar("SELECT status FROM project_runs WHERE project_id = $1 ORDER BY created_at").bind(f.project).fetch_all(&f.pool).await.unwrap()
}

#[sqlx::test(migrations = "../../migrations")]
async fn queued_runs_stay_pending_while_an_earlier_run_executes(pool: PgPool) {
    let f = fixture(pool).await;
    let first = f.queue("npm", &["install"], 30).await;
    f.queue("npm", &["run", "build"], 20).await;
    f.queue("npm", &["test"], 10).await;
    let mut conn = f.pool.acquire().await.unwrap();
    let claimed = claim_next(&mut conn, f.project).await.unwrap().expect("the oldest is claimed");
    assert_eq!(claimed.id, first);
    assert_eq!(statuses(&f).await, ["running", "pending", "pending"]);
    // A long wait for the first run doesn't make the queued ones look abandoned to another worker.
    sqlx::query("UPDATE project_runs SET started_at = now() - make_interval(secs => $2) WHERE id = $1").bind(first).bind(PAST_LEASE_SECS).execute(&f.pool).await.unwrap();
    f.steps().execute_pending(f.project, &f.sandbox).await;
    assert_eq!(statuses(&f).await[1..], ["done", "done"]);
}

#[sqlx::test(migrations = "../../migrations")]
async fn second_instance_does_not_start_a_run_while_one_is_running(pool: PgPool) {
    let f = fixture(pool).await;
    let busy = f.queue("npm", &["install"], 30).await;
    f.queue("npm", &["test"], 10).await;
    sqlx::query("UPDATE project_runs SET status = 'running', started_at = now() WHERE id = $1").bind(busy).execute(&f.pool).await.unwrap();
    f.steps().execute_pending(f.project, &f.sandbox).await;
    assert_eq!(statuses(&f).await, ["running", "pending"]);
    assert!(f.execs().is_empty());
}

/// How often the test's fake reconciler and the waiting command look at the row.
const TEST_POLL: Duration = Duration::from_millis(20);
/// Room for a woken sandbox to come up in a test.
const WAKE_ROOM: Duration = Duration::from_secs(10);

async fn owner_of(f: &Fixture) -> Uuid {
    sqlx::query_scalar("SELECT user_id FROM projects WHERE id = $1").bind(f.project).fetch_one(&f.pool).await.unwrap()
}

#[sqlx::test(migrations = "../../migrations")]
async fn sandbox_project_wakes_a_stopped_sandbox_and_runs_the_command(pool: PgPool) {
    let f = fixture(pool).await;
    sqlx::query("UPDATE project_sandboxes SET desired = 'stopped', state = 'stopped' WHERE project_id = $1").bind(f.project).execute(&f.pool).await.unwrap();
    f.provider.on_exec("'check'", out(0, "all good", ""));
    // The reconciler: once the project wants its sandbox running, it comes up, and the worker
    // runs what is queued. No browser page ever checks in.
    let (pool, project, steps, sandbox) = (f.pool.clone(), f.project, f.steps(), f.sandbox.clone());
    let worker = tokio::spawn(async move {
        loop {
            let desired: String = sqlx::query_scalar("SELECT desired FROM project_sandboxes WHERE project_id = $1").bind(project).fetch_one(&pool).await.unwrap();
            if desired == "running" {
                break;
            }
            tokio::time::sleep(TEST_POLL).await;
        }
        sqlx::query("UPDATE project_sandboxes SET state = 'running' WHERE project_id = $1").bind(project).execute(&pool).await.unwrap();
        loop {
            steps.execute_pending(project, &sandbox).await;
            // One snapshot: counted apart, a run queued between the two counts reads as "all done".
            let (any, waiting): (i64, i64) = sqlx::query_as("SELECT COUNT(*), COUNT(*) FILTER (WHERE status IN ('pending', 'running')) FROM project_runs")
                .fetch_one(&pool)
                .await
                .unwrap();
            if any > 0 && waiting == 0 {
                return;
            }
            tokio::time::sleep(TEST_POLL).await;
        }
    });
    let ops = ServerSandboxOps::new(f.pool.clone()).for_tests(TEST_POLL, WAKE_ROOM);
    let mut conn = f.pool.acquire().await.unwrap();
    let input = json!({ "project_id": f.project.to_string(), "command": "npm run check" });
    let answer = run_command_with(&mut conn, &ops, owner_of(&f).await, input).await.unwrap();
    worker.await.unwrap();
    assert_eq!(answer, "$ npm run check\nexit code 0\nall good");
    let desired: String = sqlx::query_scalar("SELECT desired FROM project_sandboxes WHERE project_id = $1").bind(f.project).fetch_one(&f.pool).await.unwrap();
    assert_eq!(desired, "running");
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_sandbox_that_stays_down_gives_the_starting_message(pool: PgPool) {
    let f = fixture(pool).await;
    sqlx::query("UPDATE project_sandboxes SET state = 'starting' WHERE project_id = $1").bind(f.project).execute(&f.pool).await.unwrap();
    let ops = ServerSandboxOps::new(f.pool.clone()).for_tests(TEST_POLL, Duration::from_millis(100));
    let mut conn = f.pool.acquire().await.unwrap();
    let input = json!({ "project_id": f.project.to_string(), "command": "npm run check" });
    let answer = run_command_with(&mut conn, &ops, owner_of(&f).await, input).await.unwrap();
    assert_eq!(answer, SANDBOX_STARTING);
    assert_eq!(statuses(&f).await.len(), 0);
}

#[sqlx::test(migrations = "../../migrations")]
async fn failed_first_check_creates_one_fix_request_per_version(pool: PgPool) {
    let f = fixture(pool).await;
    f.provider.on_exec("test -d", out(1, "", ""));
    f.provider.on_exec("npm install", out(1, "", "ERESOLVE could not resolve"));
    let steps = f.steps();
    let delegations = |pool: PgPool| async move { sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM agent_delegations").fetch_one(&pool).await.unwrap() };
    assert!(ProjectSteps::start_preview(&steps, f.project, &f.sandbox).await.is_err());
    assert_eq!(delegations(f.pool.clone()).await, 1);
    let (checked, ok, output): (Option<i64>, Option<bool>, Option<String>) =
        sqlx::query_as("SELECT checked_version, check_ok, check_output FROM projects WHERE id = $1").bind(f.project).fetch_one(&f.pool).await.unwrap();
    assert_eq!((checked, ok), (Some(0), Some(false)));
    assert!(output.unwrap().contains("ERESOLVE"));
    // The reconciler retries the start: same version, no second task.
    let _ = ProjectSteps::start_preview(&steps, f.project, &f.sandbox).await;
    assert_eq!(delegations(f.pool.clone()).await, 1);
    // New files, a new version, and so a new check and a new task once Koda is free again.
    sqlx::query("UPDATE projects SET files_version = files_version + 1 WHERE id = $1").bind(f.project).execute(&f.pool).await.unwrap();
    sqlx::query("UPDATE agent_delegations SET status = 'completed'").execute(&f.pool).await.unwrap();
    let _ = ProjectSteps::start_preview(&steps, f.project, &f.sandbox).await;
    assert_eq!(delegations(f.pool.clone()).await, 2);
}

#[sqlx::test(migrations = "../../migrations")]
async fn queued_commands_are_checked_again_before_they_reach_the_shell(pool: PgPool) {
    let f = fixture(pool).await;
    // Rows that never came from Koda's tool: a program off the allow-list, shell operators, a
    // server start, and an argument trying to break out of its quotes.
    let bad = [
        f.queue("bash", &["-c", "id"], 50).await,
        f.queue("npm", &["install", "&&", "curl", "evil.test"], 40).await,
        f.queue("npm", &["run", "dev"], 30).await,
        f.queue("node", &["-e", "x'\"; id; echo \"'"], 20).await,
        f.queue("node; id", &[], 10).await,
    ];
    let ok = f.queue("node", &["a'b.js"], 5).await;
    f.steps().execute_pending(f.project, &f.sandbox).await;
    let execs = f.execs();
    assert_eq!(execs.len(), 1, "only the valid command reaches the sandbox: {execs:?}");
    assert!(execs[0].contains(r#"node 'a'\''b.js'"#), "every argument is shell-quoted: {}", execs[0]);
    for id in bad {
        let (status, exit_code, _) = f.run(id).await;
        assert_eq!((status.as_str(), exit_code), ("failed", None), "{id}");
    }
    assert_eq!(f.run(ok).await.0, "done");
}

#[sqlx::test(migrations = "../../migrations")]
async fn runs_wait_until_the_sandbox_has_the_current_files(pool: PgPool) {
    let f = fixture(pool).await;
    let id = f.queue("npm", &["run", "check"], 1).await;
    // The project moved past what the sandbox holds: the run waits for the resync.
    sqlx::query("UPDATE projects SET files_version = 1 WHERE id = $1").bind(f.project).execute(&f.pool).await.unwrap();
    f.steps().execute_pending(f.project, &f.sandbox).await;
    assert!(f.execs().is_empty());
    assert_eq!(f.run(id).await.0, "pending");
    // Synced, but a walk (an install) holds the row: still waiting.
    sqlx::query("UPDATE project_sandboxes SET synced_version = 1, leased_until = now() + interval '1 hour' WHERE project_id = $1").bind(f.project).execute(&f.pool).await.unwrap();
    f.steps().execute_pending(f.project, &f.sandbox).await;
    assert_eq!(f.run(id).await.0, "pending");
    sqlx::query("UPDATE project_sandboxes SET leased_until = NULL WHERE project_id = $1").bind(f.project).execute(&f.pool).await.unwrap();
    f.steps().execute_pending(f.project, &f.sandbox).await;
    assert_eq!(f.run(id).await.0, "done");
}

#[sqlx::test(migrations = "../../migrations")]
async fn sandbox_does_not_claim_runs_while_a_browser_page_is_online(pool: PgPool) {
    let f = fixture(pool).await;
    let first = f.queue("npm", &["run", "check"], 2).await;
    // The person runs the project in their browser ("run in my browser", the fallback): its
    // check-in marks the runner online, and that page claims Koda's commands.
    sqlx::query("UPDATE projects SET runner_seen_at = now() WHERE id = $1").bind(f.project).execute(&f.pool).await.unwrap();
    f.steps().execute_pending(f.project, &f.sandbox).await;
    assert!(f.execs().is_empty(), "the sandbox ran nothing beside the browser");
    assert_eq!(f.run(first).await.0, "pending", "left for the browser page");
    let mut conn = f.pool.acquire().await.unwrap();
    let claimed = nomi_agent_coding::runtime::claim_runs(&mut conn, f.project).await.unwrap();
    assert_eq!(claimed.iter().map(|r| r.id).collect::<Vec<_>>(), vec![first], "the browser claims it as before");
    sqlx::query("UPDATE project_runs SET status = 'done', finished_at = now() WHERE id = $1").bind(first).execute(&f.pool).await.unwrap();
    drop(conn);

    // The page has gone quiet: the sandbox takes the queue again, and marks what it ran.
    sqlx::query("UPDATE projects SET runner_seen_at = now() - interval '1 hour' WHERE id = $1").bind(f.project).execute(&f.pool).await.unwrap();
    let second = f.queue("npm", &["run", "check"], 1).await;
    f.steps().execute_pending(f.project, &f.sandbox).await;
    assert_eq!(f.run(second).await.0, "done");
    let ran: Vec<(Uuid, bool)> = sqlx::query_as("SELECT id, ran_in_sandbox FROM project_runs WHERE project_id = $1 ORDER BY created_at").bind(f.project).fetch_all(&f.pool).await.unwrap();
    assert_eq!(ran, vec![(first, false), (second, true)]);
}
