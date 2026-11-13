//! A burst of wake-ups against the fake provider: the start limits hold and every project settles.

mod support;

use std::sync::Arc;

use async_trait::async_trait;
use nomi_sandbox::{FakeProvider, PreviewUrl, SandboxId};
use nomi_server::configuration::{ReaperMode, Rollout, SandboxConfig};
use nomi_server::sandbox_manager::{ProjectSteps, SandboxManager};
use nomi_server::sandbox_store::{self as store, State};
use nomi_server::sandbox_worker::{SandboxWorker, PROVIDER_NAME};
use sqlx::PgPool;
use support::App;
use uuid::Uuid;

/// The provider-side auto-stop used in these tests, in seconds.
const AUTO_STOP_SECS: u64 = 6 * 60 * 60;

/// Projects that ask for a sandbox at the same moment.
const BURST: usize = 200;
/// Starts allowed per minute window in this test.
const STARTS_PER_MINUTE: u32 = 50;
/// An hour limit that never gets in the way of the per-minute one.
const GENEROUS_HOURLY: u32 = 10_000;
/// Idle stop used in the test config, in seconds.
const IDLE_STOP_SECS: u64 = 600;
/// Monthly minutes that never get in the way.
const GENEROUS_MINUTES: u32 = 1_000_000;
/// Minute windows the burst needs at the limit: 200 starts at 50 per minute.
const WINDOWS_NEEDED: usize = BURST / STARTS_PER_MINUTE as usize;
/// A safety stop so a stuck worker fails the test instead of hanging it.
const MAX_TICKS_PER_WINDOW: usize = 100;
/// How far a deferred row's retry time is moved back to stand in for a minute passing.
const WINDOW_REWIND_SECS: f64 = 3600.0;

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

fn config() -> SandboxConfig {
    SandboxConfig {
        api_base: String::new(),
        api_key: String::new(),
        size: nomi_sandbox::Size::Small,
        idle_stop_secs: IDLE_STOP_SECS,
        auto_stop_secs: AUTO_STOP_SECS,
        rollout: Rollout::All,
        starts_per_minute: STARTS_PER_MINUTE,
        starts_per_hour: GENEROUS_HOURLY,
        monthly_minutes_per_user: GENEROUS_MINUTES,
        reaper_mode: ReaperMode::DryRun,
    }
}

/// A fresh worker over the same provider. The start limiter lives in the manager and runs on the
/// real clock, so a new manager is a new, empty minute window: that is how a minute passing is
/// stood in for without sleeping.
fn worker_for_next_window(pool: &PgPool, provider: &Arc<FakeProvider>) -> SandboxWorker {
    let manager = Arc::new(SandboxManager::new(provider.clone(), Arc::new(OkSteps), pool.clone(), config()));
    SandboxWorker::new(pool.clone(), manager, IDLE_STOP_SECS)
}

fn starts(provider: &FakeProvider) -> usize {
    provider.calls().iter().filter(|c| c.starts_with("create") || c.starts_with("resume")).count()
}

async fn running_count(pool: &PgPool) -> i64 {
    sqlx::query_scalar("SELECT count(*) FROM project_sandboxes WHERE state = 'running'").fetch_one(pool).await.unwrap()
}

#[sqlx::test(migrations = "../../migrations")]
async fn burst_of_200_wakes_respects_start_limits_and_all_settle(pool: PgPool) {
    let app = App::new(pool.clone());
    // One owner: signing people up is slow and the limits under test are not per user.
    let owner = app.person("burst@load.test").await;
    let mut projects = Vec::with_capacity(BURST);
    for i in 0..BURST {
        projects.push(app.project(&owner, &format!("burst{i}")).await.0);
    }
    {
        let mut conn = pool.acquire().await.unwrap();
        for p in &projects {
            store::want_running(&mut conn, *p, PROVIDER_NAME).await.unwrap();
        }
    }
    let provider = Arc::new(FakeProvider::new());
    let mut seen_starts = 0;
    for window in 1..=WINDOWS_NEEDED {
        let worker = worker_for_next_window(&pool, &provider);
        for _ in 0..MAX_TICKS_PER_WINDOW {
            if worker.tick().await == 0 {
                break;
            }
        }
        let this_window = starts(&provider) - seen_starts;
        seen_starts += this_window;
        assert!(this_window <= STARTS_PER_MINUTE as usize, "window {window} admitted {this_window} starts");
        assert_eq!(running_count(&pool).await as usize, STARTS_PER_MINUTE as usize * window, "window {window}");
        // The rest were deferred, not failed: let their retry time pass.
        sqlx::query("UPDATE project_sandboxes SET retry_at = now() - make_interval(secs => $1) WHERE retry_at IS NOT NULL")
            .bind(WINDOW_REWIND_SECS)
            .execute(&pool)
            .await
            .unwrap();
    }
    assert_eq!(running_count(&pool).await as usize, BURST);
    assert_eq!(seen_starts, BURST, "one start per project, none repeated");
    let mut conn = pool.acquire().await.unwrap();
    for p in &projects {
        let row = store::get(&mut conn, *p).await.unwrap().unwrap();
        assert_eq!(row.state, State::Running);
        assert_eq!(row.attempts, 0, "a deferral is not a failed attempt");
    }
}
