//! Sandboxes are deleted with their projects (through the deletion outbox) and leaked ones are
//! found by the reaper.

mod support;

use std::collections::BTreeMap;
use std::sync::Arc;

use async_trait::async_trait;
use nomi_sandbox::{CreateSpec, FakeProvider, PreviewUrl, SandboxError, SandboxId, SandboxProvider, SandboxState, Size};
use nomi_server::sandbox_store as store;
use nomi_server::configuration::{ReaperMode, Rollout, SandboxConfig};
use nomi_server::sandbox_manager::{ProjectSteps, SandboxManager};
use nomi_server::sandbox_worker::SandboxWorker;
use sqlx::PgPool;
use support::App;
use uuid::Uuid;

/// The provider-side auto-stop used in these tests, in seconds.
const AUTO_STOP_SECS: u64 = 6 * 60 * 60;

const PROVIDER: &str = "boat";
/// Start limits that never get in the way.
const GENEROUS_LIMIT: u32 = 1000;
/// Idle stop used in the test config, in seconds.
const IDLE_STOP_SECS: u64 = 600;

struct OkSteps;

#[async_trait]
impl ProjectSteps for OkSteps {
    async fn sync(&self, _: Uuid, _: &SandboxId) -> Result<i64, String> {
        Ok(1)
    }
    async fn start_preview(&self, _: Uuid, _: &SandboxId) -> Result<PreviewUrl, nomi_server::sandbox_manager::StepError> {
        Ok(PreviewUrl { url: "https://preview.example.test".into() })
    }
}

fn config(mode: ReaperMode) -> SandboxConfig {
    SandboxConfig {
        api_base: String::new(),
        api_key: String::new(),
        size: Size::Small,
        idle_stop_secs: IDLE_STOP_SECS,
        auto_stop_secs: AUTO_STOP_SECS,
        rollout: Rollout::All,
        starts_per_minute: GENEROUS_LIMIT,
        starts_per_hour: GENEROUS_LIMIT,
        monthly_minutes_per_user: GENEROUS_LIMIT,
        reaper_mode: mode,
    }
}

fn worker(pool: &PgPool, mode: ReaperMode) -> (SandboxWorker, Arc<FakeProvider>) {
    let provider = Arc::new(FakeProvider::new());
    let manager = Arc::new(SandboxManager::new(provider.clone(), Arc::new(OkSteps), pool.clone(), config(mode)));
    (SandboxWorker::new(pool.clone(), manager, IDLE_STOP_SECS).with_reaper_mode(mode), provider)
}

/// A project (with its owner) that has a sandbox at the provider and a row pointing at it.
async fn project_with_sandbox(pool: &PgPool, provider: &FakeProvider, owner: &str) -> (Uuid, Uuid, SandboxId) {
    let app = App::new(pool.clone());
    let person = app.person(&format!("{owner}@cleanup.test")).await;
    let project = app.project(&person, owner).await.0;
    let user: Uuid = sqlx::query_scalar("SELECT user_id FROM projects WHERE id = $1").bind(project).fetch_one(pool).await.unwrap();
    let id = labelled(provider, &project.to_string()).await;
    sqlx::query("INSERT INTO project_sandboxes (project_id, provider, sandbox_id, desired, state) VALUES ($1, $2, $3, 'stopped', 'stopped')")
        .bind(project)
        .bind(PROVIDER)
        .bind(&id.0)
        .execute(pool)
        .await
        .unwrap();
    (project, user, id)
}

async fn labelled(provider: &FakeProvider, project: &str) -> SandboxId {
    provider
        .create(CreateSpec {
            size: Size::Small,
            ttl: nomi_sandbox::Ttl::ProviderDefault,
            from_snapshot: None,
            idempotency_key: format!("key-{project}-{}", Uuid::new_v4()),
            labels: BTreeMap::from([("nomi-project".to_string(), project.to_string())]),
        })
        .await
        .unwrap()
        .id
}

async fn outbox(pool: &PgPool) -> i64 {
    sqlx::query_scalar("SELECT count(*) FROM sandbox_deletions").fetch_one(pool).await.unwrap()
}

async fn exists(provider: &FakeProvider, id: &SandboxId) -> bool {
    provider.list().await.unwrap().iter().any(|s| &s.id == id)
}

#[sqlx::test(migrations = "../../migrations")]
async fn deleting_a_project_enqueues_and_the_worker_deletes_its_sandbox(pool: PgPool) {
    let (worker, provider) = worker(&pool, ReaperMode::DryRun);
    let (project, _, id) = project_with_sandbox(&pool, &provider, "gone").await;
    sqlx::query("DELETE FROM projects WHERE id = $1").bind(project).execute(&pool).await.unwrap();
    assert_eq!(outbox(&pool).await, 1);
    assert!(exists(&provider, &id).await, "deleting the project must not call the provider");
    assert_eq!(worker.drain_deletions().await, 1);
    assert!(!exists(&provider, &id).await);
    assert_eq!(outbox(&pool).await, 0);
}

#[sqlx::test(migrations = "../../migrations")]
async fn provider_failure_on_delete_keeps_the_outbox_row_for_retry(pool: PgPool) {
    let (worker, provider) = worker(&pool, ReaperMode::DryRun);
    let (project, _, id) = project_with_sandbox(&pool, &provider, "flaky").await;
    sqlx::query("DELETE FROM projects WHERE id = $1").bind(project).execute(&pool).await.unwrap();
    provider.fail_next(SandboxError::Unavailable("boom".into()));
    assert_eq!(worker.drain_deletions().await, 0);
    let (attempts, retry_in_future, error): (i32, bool, Option<String>) = sqlx::query_as("SELECT attempts, retry_at > now(), last_error FROM sandbox_deletions").fetch_one(&pool).await.unwrap();
    assert_eq!(attempts, 1);
    assert!(retry_in_future);
    assert!(error.unwrap().contains("boom"));
    assert!(exists(&provider, &id).await);
    // Not due yet: a second pass leaves it alone.
    assert_eq!(worker.drain_deletions().await, 0);
    sqlx::query("UPDATE sandbox_deletions SET retry_at = now() - interval '1 second'").execute(&pool).await.unwrap();
    assert_eq!(worker.drain_deletions().await, 1);
    assert!(!exists(&provider, &id).await);
    assert_eq!(outbox(&pool).await, 0);
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_sandbox_already_gone_counts_as_deleted(pool: PgPool) {
    let (worker, provider) = worker(&pool, ReaperMode::DryRun);
    let (project, _, id) = project_with_sandbox(&pool, &provider, "vanished").await;
    provider.delete(&id).await.unwrap();
    sqlx::query("DELETE FROM projects WHERE id = $1").bind(project).execute(&pool).await.unwrap();
    assert_eq!(worker.drain_deletions().await, 1);
    assert_eq!(outbox(&pool).await, 0);
}

#[sqlx::test(migrations = "../../migrations")]
async fn account_deletion_enqueues_all_project_sandboxes(pool: PgPool) {
    let (worker, provider) = worker(&pool, ReaperMode::DryRun);
    let (first, user, a) = project_with_sandbox(&pool, &provider, "acct").await;
    // A second project of the same person, with its own sandbox.
    let second: Uuid = sqlx::query_scalar("INSERT INTO projects (user_id, session_id, name) SELECT user_id, session_id, 'second' FROM projects WHERE id = $1 RETURNING id")
        .bind(first)
        .fetch_one(&pool)
        .await
        .unwrap();
    let b = labelled(&provider, &second.to_string()).await;
    sqlx::query("INSERT INTO project_sandboxes (project_id, provider, sandbox_id) VALUES ($1, $2, $3)").bind(second).bind(PROVIDER).bind(&b.0).execute(&pool).await.unwrap();
    // The app has no account purge yet (users are referenced by many tables without a cascade), so
    // this deletes all of the person's projects at once, which is what a purge would do.
    sqlx::query("DELETE FROM projects WHERE user_id = $1").bind(user).execute(&pool).await.unwrap();
    assert_eq!(outbox(&pool).await, 2);
    assert_eq!(worker.drain_deletions().await, 2);
    assert!(!exists(&provider, &a).await);
    assert!(!exists(&provider, &b).await);
}

#[sqlx::test(migrations = "../../migrations")]
async fn reaper_deletes_only_unreferenced_nomi_labelled_sandboxes(pool: PgPool) {
    let (worker, provider) = worker(&pool, ReaperMode::Delete);
    let (_, _, kept) = project_with_sandbox(&pool, &provider, "kept").await;
    let leaked = labelled(&provider, &Uuid::new_v4().to_string()).await;
    let queued = labelled(&provider, &Uuid::new_v4().to_string()).await;
    sqlx::query("INSERT INTO sandbox_deletions (provider, sandbox_id) VALUES ($1, $2)").bind(PROVIDER).bind(&queued.0).execute(&pool).await.unwrap();
    // Someone else's sandbox: no nomi-project label.
    let foreign = provider
        .create(CreateSpec { size: Size::Small, ttl: nomi_sandbox::Ttl::ProviderDefault, from_snapshot: None, idempotency_key: "foreign".into(), labels: BTreeMap::from([("team".to_string(), "other".to_string())]) })
        .await
        .unwrap()
        .id;
    assert_eq!(worker.reap_orphans().await, 1);
    assert!(!exists(&provider, &leaked).await);
    assert!(exists(&provider, &kept).await);
    assert!(exists(&provider, &queued).await, "the outbox owns this one");
    assert!(exists(&provider, &foreign).await);
}

#[sqlx::test(migrations = "../../migrations")]
async fn reaper_dry_run_deletes_nothing(pool: PgPool) {
    let (worker, provider) = worker(&pool, ReaperMode::DryRun);
    let leaked = labelled(&provider, &Uuid::new_v4().to_string()).await;
    assert_eq!(worker.reap_orphans().await, 1, "it still reports what it found");
    assert!(exists(&provider, &leaked).await);
    assert!(!provider.calls().iter().any(|c| c.starts_with("delete")));
}

#[sqlx::test(migrations = "../../migrations")]
async fn reaper_leaves_a_sandbox_whose_project_is_still_creating_it(pool: PgPool) {
    let (worker, provider) = worker(&pool, ReaperMode::Delete);
    let app = App::new(pool.clone());
    let person = app.person("creating@cleanup.test").await;
    let project = app.project(&person, "creating").await.0;
    sqlx::query("INSERT INTO project_sandboxes (project_id, provider, desired, state) VALUES ($1, $2, 'running', 'creating')").bind(project).bind(PROVIDER).execute(&pool).await.unwrap();
    let fresh = labelled(&provider, &project.to_string()).await;
    assert_eq!(worker.reap_orphans().await, 0);
    assert!(exists(&provider, &fresh).await);
}

/// A manager over a fresh fake provider, and a project that wants to run.
async fn manager_with_running_project(pool: &PgPool, owner: &str) -> (SandboxManager, Arc<FakeProvider>, Uuid) {
    let provider = Arc::new(FakeProvider::new());
    let manager = SandboxManager::new(provider.clone(), Arc::new(OkSteps), pool.clone(), config(ReaperMode::DryRun));
    let app = App::new(pool.clone());
    let person = app.person(&format!("{owner}@cleanup.test")).await;
    let project = app.project(&person, owner).await.0;
    let mut conn = pool.acquire().await.unwrap();
    store::want_running(&mut conn, project, PROVIDER).await.unwrap();
    (manager, provider, project)
}

async fn reconcile(manager: &SandboxManager, pool: &PgPool, project: Uuid) {
    let mut conn = pool.acquire().await.unwrap();
    let row = store::get(&mut conn, project).await.unwrap().unwrap();
    manager.reconcile(row).await;
}

#[sqlx::test(migrations = "../../migrations")]
async fn provider_error_sandbox_is_enqueued_for_deletion_when_forgotten(pool: PgPool) {
    let (manager, provider, project) = manager_with_running_project(&pool, "errored").await;
    reconcile(&manager, &pool, project).await;
    let id = SandboxId(sqlx::query_scalar("SELECT sandbox_id FROM project_sandboxes WHERE project_id = $1").bind(project).fetch_one(&pool).await.unwrap());
    provider.set_state(&id, SandboxState::Error);
    reconcile(&manager, &pool, project).await;
    let queued: Vec<String> = sqlx::query_scalar("SELECT sandbox_id FROM sandbox_deletions").fetch_all(&pool).await.unwrap();
    assert_eq!(queued, vec![id.0.clone()]);
    // The row let go of it.
    assert_ne!(sqlx::query_scalar::<_, Option<String>>("SELECT sandbox_id FROM project_sandboxes WHERE project_id = $1").bind(project).fetch_one(&pool).await.unwrap(), Some(id.0));
}

#[sqlx::test(migrations = "../../migrations")]
async fn failed_best_effort_delete_is_enqueued(pool: PgPool) {
    let (manager, provider, project) = manager_with_running_project(&pool, "expired").await;
    reconcile(&manager, &pool, project).await;
    let old: String = sqlx::query_scalar("SELECT sandbox_id FROM project_sandboxes WHERE project_id = $1").bind(project).fetch_one(&pool).await.unwrap();
    let mut conn = pool.acquire().await.unwrap();
    store::want_stopped(&mut conn, project).await.unwrap();
    reconcile(&manager, &pool, project).await;
    store::want_running(&mut conn, project, PROVIDER).await.unwrap();
    // Resume says the snapshot expired, then deleting the old sandbox fails.
    provider.fail_next(SandboxError::SnapshotExpired);
    provider.fail_next(SandboxError::Unavailable("down".into()));
    reconcile(&manager, &pool, project).await;
    let queued: Vec<String> = sqlx::query_scalar("SELECT sandbox_id FROM sandbox_deletions").fetch_all(&pool).await.unwrap();
    assert_eq!(queued, vec![old]);
}
