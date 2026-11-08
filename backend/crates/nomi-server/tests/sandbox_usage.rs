//! Sandbox metering: minutes per person per month, the monthly allowance, the admin view and the
//! drift check against the provider.

mod support;

use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Duration;

use nomi_sandbox::{FakeProvider, SandboxId};
use nomi_server::configuration::{Rollout, SandboxConfig};
use nomi_server::sandbox_manager::{ProjectSteps, SandboxManager};
use nomi_server::sandbox_store::{self as store, Desired, State};
use nomi_server::sandbox_usage::{minutes_this_month, reconcile_usage_drift, within_allowance};
use nomi_server::sandbox_worker::{SandboxWorker, PROVIDER_NAME};
use serde_json::json;
use sqlx::PgPool;
use support::App;
use uuid::Uuid;

/// `sandbox_config()` reads through the process-wide settings cache and pool, so the tests that
/// set it take turns.
static GLOBALS: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

const PREVIEW: &str = "https://preview.example.test";
/// An allowance far above what any test uses.
const GENEROUS_MINUTES: u32 = 100_000;
/// A session length the tests use.
const NINETY_MINUTES: i64 = 90;

fn cfg(minutes: u32) -> SandboxConfig {
    SandboxConfig {
        api_base: String::new(),
        api_key: String::new(),
        size: nomi_sandbox::Size::Small,
        idle_stop_secs: 600,
        rollout: Rollout::All,
        starts_per_minute: 1000,
        starts_per_hour: 1000,
        monthly_minutes_per_user: minutes,
        reaper_mode: nomi_server::configuration::ReaperMode::DryRun,
    }
}

/// A session that began `start_min` minutes ago and ended `end_min` minutes ago (open when `None`).
async fn session(pool: &PgPool, project: Uuid, user: Uuid, start_min: i64, end_min: Option<i64>) {
    sqlx::query(
        "INSERT INTO sandbox_sessions (project_id, user_id, started_at, stopped_at) VALUES ($1, $2, now() - make_interval(mins => $3::int), \
         CASE WHEN $4::int IS NULL THEN NULL ELSE now() - make_interval(mins => $4::int) END)",
    )
    .bind(project)
    .bind(user)
    .bind(start_min as i32)
    .bind(end_min.map(|m| m as i32))
    .execute(pool)
    .await
    .unwrap();
}

async fn minutes(pool: &PgPool, user: Uuid) -> i64 {
    let mut conn = pool.acquire().await.unwrap();
    minutes_this_month(&mut conn, user).await.unwrap()
}

/// Minutes since the start of this month (UTC), as the database sees them.
async fn minutes_into_month(pool: &PgPool) -> i64 {
    sqlx::query_scalar("SELECT floor(extract(epoch FROM now() - (date_trunc('month', now() AT TIME ZONE 'UTC') AT TIME ZONE 'UTC')) / 60)::bigint")
        .fetch_one(pool)
        .await
        .unwrap()
}

#[sqlx::test(migrations = "../../migrations")]
async fn open_session_counts_up_to_now(pool: PgPool) {
    let app = App::new(pool.clone());
    let owner = app.person("owner@usage.test").await;
    let (p, _) = app.project(&owner, "A").await;
    assert_eq!(minutes(&pool, owner.id).await, 0);
    session(&pool, p, owner.id, NINETY_MINUTES, None).await;
    assert_eq!(minutes(&pool, owner.id).await, NINETY_MINUTES.min(minutes_into_month(&pool).await));
}

#[sqlx::test(migrations = "../../migrations")]
async fn closed_zero_length_and_other_months_are_handled(pool: PgPool) {
    let app = App::new(pool.clone());
    let owner = app.person("owner@usage.test").await;
    let other = app.person("other@usage.test").await;
    let (p, _) = app.project(&owner, "A").await;
    let (q, _) = app.project(&other, "B").await;
    // Zero-length generation rows (no user) and a zero-length one with a user count nothing.
    sqlx::query("INSERT INTO sandbox_sessions (project_id, started_at, stopped_at) VALUES ($1, now(), now())").bind(p).execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO sandbox_sessions (project_id, user_id, started_at, stopped_at) VALUES ($1, $2, now(), now())").bind(p).bind(owner.id).execute(&pool).await.unwrap();
    // A session that ended exactly as this month began adds nothing, even though it was long.
    sqlx::query(
        "INSERT INTO sandbox_sessions (project_id, user_id, started_at, stopped_at) VALUES ($1, $2, \
         (date_trunc('month', now() AT TIME ZONE 'UTC') AT TIME ZONE 'UTC') - interval '5 hours', date_trunc('month', now() AT TIME ZONE 'UTC') AT TIME ZONE 'UTC')",
    )
    .bind(p)
    .bind(owner.id)
    .execute(&pool)
    .await
    .unwrap();
    // One that began before the month and is still open counts only its time in this month.
    sqlx::query(
        "INSERT INTO sandbox_sessions (project_id, user_id, started_at) VALUES ($1, $2, (date_trunc('month', now() AT TIME ZONE 'UTC') AT TIME ZONE 'UTC') - interval '5 hours')",
    )
    .bind(q)
    .bind(other.id)
    .execute(&pool)
    .await
    .unwrap();
    assert_eq!(minutes(&pool, owner.id).await, 0);
    let into = minutes_into_month(&pool).await;
    let got = minutes(&pool, other.id).await;
    assert!((into..=into + 1).contains(&got), "{got} vs {into}");
}

#[sqlx::test(migrations = "../../migrations")]
async fn allowance_compares_minutes_with_the_limit(pool: PgPool) {
    let app = App::new(pool.clone());
    let owner = app.person("owner@usage.test").await;
    let (p, _) = app.project(&owner, "A").await;
    session(&pool, p, owner.id, 20, Some(10)).await; // 10 minutes
    let mut conn = pool.acquire().await.unwrap();
    let used = minutes_this_month(&mut conn, owner.id).await.unwrap();
    assert!(used >= 1, "the test needs this month to be older than a minute");
    assert!(within_allowance(&mut conn, owner.id, &cfg(used as u32 + 1)).await.unwrap());
    assert!(!within_allowance(&mut conn, owner.id, &cfg(used as u32)).await.unwrap());
}

/// Turns the admin switch on with the allowance given.
async fn switch(pool: &PgPool, admin: Uuid, allowance: u32) {
    std::env::set_var("SETTINGS_ENCRYPTION_KEY", hex::encode(nomi_test_support::TEST_SETTINGS_KEY));
    nomi_settings::config::install(pool.clone());
    let values: BTreeMap<String, String> = [("enabled", "on".to_string()), ("api_key", "boat-key-123".to_string()), ("rollout", "all".to_string()), ("monthly_minutes_per_user", allowance.to_string())]
        .into_iter()
        .map(|(k, v)| (k.to_string(), v))
        .collect();
    nomi_settings::config::save(pool, &nomi_test_support::TEST_SETTINGS_KEY, &nomi_settings::config::SANDBOX, &values, admin).await.unwrap();
    nomi_settings::config::forget();
}

struct NoSteps;

#[async_trait::async_trait]
impl ProjectSteps for NoSteps {
    async fn sync(&self, _: Uuid, _: &SandboxId) -> Result<i64, String> {
        Ok(1)
    }
    async fn start_preview(&self, _: Uuid, _: &SandboxId) -> Result<nomi_sandbox::PreviewUrl, String> {
        Ok(nomi_sandbox::PreviewUrl { url: PREVIEW.into() })
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn over_allowance_makes_sandbox_unavailable_but_running_sandbox_is_not_killed(pool: PgPool) {
    let _g = GLOBALS.lock().await;
    let app = App::new(pool.clone());
    let owner = app.person("owner@usage.test").await;
    let (p, _) = app.project(&owner, "A").await;
    sqlx::query("UPDATE projects SET runtime = 'sandbox' WHERE id = $1").bind(p).execute(&pool).await.unwrap();
    switch(&pool, owner.id, GENEROUS_MINUTES).await;
    let uri = format!("/api/projects/{p}/sandbox");
    let (_, body) = app.get(&uri, &owner).await;
    assert_eq!(body["available"], json!(true));

    // The sandbox is up and its session has been open for 90 minutes; the allowance is now 1.
    {
        let mut conn = pool.acquire().await.unwrap();
        store::want_running(&mut conn, p, PROVIDER_NAME).await.unwrap();
        store::set_state(&mut conn, p, State::Running, None).await.unwrap();
    }
    session(&pool, p, owner.id, NINETY_MINUTES, None).await;
    sqlx::query("UPDATE sandbox_sessions SET started_at = (date_trunc('month', now() AT TIME ZONE 'UTC') AT TIME ZONE 'UTC') - interval '1 minute' WHERE project_id = $1")
        .bind(p)
        .execute(&pool)
        .await
        .unwrap();
    switch(&pool, owner.id, 1).await;
    let (_, body) = app.get(&uri, &owner).await;
    assert_eq!(body["available"], json!(false));
    assert_eq!(body["state"], "running");
    let (status, _) = app.send("POST", &format!("{uri}/start"), &owner, json!({})).await;
    assert_eq!(status, axum::http::StatusCode::CONFLICT);

    // The worker leaves it running (the page is still open), and its session stays open.
    sqlx::query("UPDATE projects SET runner_seen_at = now() WHERE id = $1").bind(p).execute(&pool).await.unwrap();
    let provider = Arc::new(FakeProvider::new());
    let manager = Arc::new(SandboxManager::new(provider, Arc::new(NoSteps), pool.clone(), cfg(1)));
    let worker = SandboxWorker::new(pool.clone(), manager, 600);
    worker.tick().await;
    worker.sweep_idle().await;
    let mut conn = pool.acquire().await.unwrap();
    let row = store::get(&mut conn, p).await.unwrap().unwrap();
    assert_eq!((row.state, row.desired), (State::Running, Desired::Running));
    let open: i64 = sqlx::query_scalar("SELECT count(*) FROM sandbox_sessions WHERE project_id = $1 AND stopped_at IS NULL").bind(p).fetch_one(&pool).await.unwrap();
    assert_eq!(open, 1);
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_session_opened_by_the_manager_belongs_to_the_project_owner(pool: PgPool) {
    let app = App::new(pool.clone());
    let owner = app.person("owner@usage.test").await;
    let (p, _) = app.project(&owner, "A").await;
    {
        let mut conn = pool.acquire().await.unwrap();
        store::want_running(&mut conn, p, PROVIDER_NAME).await.unwrap();
    }
    let provider = Arc::new(FakeProvider::new());
    let manager = Arc::new(SandboxManager::new(provider, Arc::new(NoSteps), pool.clone(), cfg(GENEROUS_MINUTES)));
    SandboxWorker::new(pool.clone(), manager, 600).tick().await;
    let user: Option<Uuid> = sqlx::query_scalar("SELECT user_id FROM sandbox_sessions WHERE project_id = $1 AND stopped_at IS NULL").bind(p).fetch_one(&pool).await.unwrap();
    assert_eq!(user, Some(owner.id));
}

#[sqlx::test(migrations = "../../migrations")]
async fn admin_usage_lists_minutes_per_user(pool: PgPool) {
    let app = App::new(pool.clone());
    let admin = app.admin("boss@usage.test").await;
    let owner = app.person("owner@usage.test").await;
    let (p, _) = app.project(&owner, "A").await;
    session(&pool, p, owner.id, 30, Some(10)).await;
    let (status, body) = app.get("/api/admin/usage", &admin).await;
    assert!(status.is_success(), "{status}");
    let rows = body["sandbox_minutes"].as_array().unwrap();
    assert_eq!(rows.len(), 1, "{rows:?}");
    assert_eq!(rows[0]["user_id"], json!(owner.id));
    assert_eq!(rows[0]["email"], "owner@usage.test");
    assert_eq!(rows[0]["minutes"], 20);
}

#[sqlx::test(migrations = "../../migrations")]
async fn drift_is_reported_for_sandboxes_that_disagree_with_the_provider(pool: PgPool) {
    let app = App::new(pool.clone());
    let owner = app.person("owner@usage.test").await;
    let (agree, _) = app.project(&owner, "Agree").await;
    let (differ, _) = app.project(&owner, "Differ").await;
    let provider = Arc::new(FakeProvider::new());
    for (project, provider_minutes) in [(agree, 10u64), (differ, 50)] {
        let id = nomi_sandbox::SandboxProvider::create(
            provider.as_ref(),
            nomi_sandbox::CreateSpec { size: nomi_sandbox::Size::Small, ttl_seconds: None, from_snapshot: None, idempotency_key: project.to_string(), labels: Default::default() },
        )
        .await
        .unwrap()
        .id;
        provider.set_usage(&id, Duration::from_secs(provider_minutes * 60));
        sqlx::query("INSERT INTO project_sandboxes (project_id, provider, sandbox_id, state) VALUES ($1, 'boat', $2, 'running')").bind(project).bind(&id.0).execute(&pool).await.unwrap();
        session(&pool, project, owner.id, 20, Some(10)).await; // ten minutes
    }
    let drifted = reconcile_usage_drift(&pool, provider.as_ref()).await;
    assert_eq!(drifted, vec![differ]);
}
