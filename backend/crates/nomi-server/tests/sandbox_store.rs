//! The per-project sandbox rows the reconciler works from.

mod support;

use chrono::{Duration, Utc};
use nomi_server::sandbox_store::{self as store, Desired, State};
use sqlx::PgPool;
use support::App;
use uuid::Uuid;

const PROVIDER: &str = "boat";
/// Rows claimed per call in these tests.
const CLAIM_LIMIT: i64 = 10;
/// A runner check-in older than this is stale for the idle tests.
const IDLE_SECS: u64 = 600;
/// How long ago a stale check-in happened.
const STALE_AGE_SECS: i64 = 3600;
/// How long ago a recent check-in happened.
const RECENT_AGE_SECS: i64 = 5;

async fn project(pool: &PgPool, name: &str) -> Uuid {
    let app = App::new(pool.clone());
    let owner = app.person(&format!("{name}@sandbox.test")).await;
    app.project(&owner, name).await.0
}

async fn count(pool: &PgPool) -> i64 {
    sqlx::query_scalar("SELECT count(*) FROM project_sandboxes").fetch_one(pool).await.unwrap()
}

#[sqlx::test(migrations = "../../migrations")]
async fn want_running_creates_row_once(pool: PgPool) {
    let p = project(&pool, "one").await;
    let mut conn = pool.acquire().await.unwrap();
    store::want_running(&mut conn, p, PROVIDER).await.unwrap();
    store::want_running(&mut conn, p, PROVIDER).await.unwrap();
    assert_eq!(count(&pool).await, 1);
    let rows = {
        let mut tx = pool.begin().await.unwrap();
        let rows = store::claim_due(&mut tx, CLAIM_LIMIT).await.unwrap();
        tx.rollback().await.unwrap();
        rows
    };
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].desired, Desired::Running);
    assert_eq!(rows[0].state, State::None);
    assert_eq!(rows[0].provider, PROVIDER);
}

#[sqlx::test(migrations = "../../migrations")]
async fn claim_due_skips_rows_locked_by_another_claimer(pool: PgPool) {
    let p = project(&pool, "two").await;
    let mut conn = pool.acquire().await.unwrap();
    store::want_running(&mut conn, p, PROVIDER).await.unwrap();
    let mut first = pool.begin().await.unwrap();
    assert_eq!(store::claim_due(&mut first, CLAIM_LIMIT).await.unwrap().len(), 1);
    let mut second = pool.begin().await.unwrap();
    assert!(store::claim_due(&mut second, CLAIM_LIMIT).await.unwrap().is_empty());
    first.rollback().await.unwrap();
    second.rollback().await.unwrap();
}

#[sqlx::test(migrations = "../../migrations")]
async fn claim_due_honours_retry_at(pool: PgPool) {
    let p = project(&pool, "three").await;
    let mut conn = pool.acquire().await.unwrap();
    store::want_running(&mut conn, p, PROVIDER).await.unwrap();
    store::record_attempt_failure(&mut conn, p, Utc::now() + Duration::hours(1), "boom").await.unwrap();
    let mut tx = pool.begin().await.unwrap();
    assert!(store::claim_due(&mut tx, CLAIM_LIMIT).await.unwrap().is_empty());
    tx.rollback().await.unwrap();

    store::record_attempt_failure(&mut conn, p, Utc::now() - Duration::seconds(1), "boom").await.unwrap();
    let mut tx = pool.begin().await.unwrap();
    let rows = store::claim_due(&mut tx, CLAIM_LIMIT).await.unwrap();
    tx.rollback().await.unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].attempts, 2);

    store::set_state(&mut conn, p, State::Running, None).await.unwrap();
    let (attempts, retry): (i32, Option<chrono::DateTime<Utc>>) =
        sqlx::query_as("SELECT attempts, retry_at FROM project_sandboxes WHERE project_id = $1").bind(p).fetch_one(&pool).await.unwrap();
    assert_eq!((attempts, retry), (0, None));
}

#[sqlx::test(migrations = "../../migrations")]
async fn idle_candidates_excludes_projects_with_pending_runs(pool: PgPool) {
    let p = project(&pool, "four").await;
    let mut conn = pool.acquire().await.unwrap();
    store::want_running(&mut conn, p, PROVIDER).await.unwrap();
    sqlx::query("UPDATE projects SET runner_seen_at = now() - make_interval(secs => $2) WHERE id = $1")
        .bind(p).bind(STALE_AGE_SECS as f64).execute(&pool).await.unwrap();
    assert_eq!(store::idle_candidates(&mut conn, IDLE_SECS).await.unwrap(), vec![p]);
    sqlx::query("INSERT INTO project_runs (project_id, command) VALUES ($1, 'npm')").bind(p).execute(&pool).await.unwrap();
    assert!(store::idle_candidates(&mut conn, IDLE_SECS).await.unwrap().is_empty());
}

#[sqlx::test(migrations = "../../migrations")]
async fn idle_candidates_excludes_recent_runner_seen_at(pool: PgPool) {
    let p = project(&pool, "five").await;
    let mut conn = pool.acquire().await.unwrap();
    store::want_running(&mut conn, p, PROVIDER).await.unwrap();
    sqlx::query("UPDATE projects SET runner_seen_at = now() - make_interval(secs => $2) WHERE id = $1")
        .bind(p).bind(RECENT_AGE_SECS as f64).execute(&pool).await.unwrap();
    assert!(store::idle_candidates(&mut conn, IDLE_SECS).await.unwrap().is_empty());
}

#[sqlx::test(migrations = "../../migrations")]
async fn deleting_a_project_cascades_the_row(pool: PgPool) {
    let p = project(&pool, "six").await;
    let mut conn = pool.acquire().await.unwrap();
    store::want_running(&mut conn, p, PROVIDER).await.unwrap();
    sqlx::query("DELETE FROM projects WHERE id = $1").bind(p).execute(&pool).await.unwrap();
    assert_eq!(count(&pool).await, 0);
}

#[sqlx::test(migrations = "../../migrations")]
async fn overlapping_claims_do_not_both_walk_a_row(pool: PgPool) {
    let p = project(&pool, "seven").await;
    let mut conn = pool.acquire().await.unwrap();
    store::want_running(&mut conn, p, PROVIDER).await.unwrap();
    let mut first = pool.begin().await.unwrap();
    assert_eq!(store::claim_due(&mut first, CLAIM_LIMIT).await.unwrap().len(), 1);
    first.commit().await.unwrap();
    // The first claimer's transaction is over but its walk is still going: the lease holds.
    let mut second = pool.begin().await.unwrap();
    assert!(store::claim_due(&mut second, CLAIM_LIMIT).await.unwrap().is_empty());
    second.rollback().await.unwrap();

    store::defer(&mut conn, p, Utc::now() - Duration::seconds(1)).await.unwrap();
    let mut third = pool.begin().await.unwrap();
    assert_eq!(store::claim_due(&mut third, CLAIM_LIMIT).await.unwrap().len(), 1);
    third.rollback().await.unwrap();
}

#[sqlx::test(migrations = "../../migrations")]
async fn want_running_clears_backoff_on_a_settled_row_but_not_an_in_flight_lease(pool: PgPool) {
    let p = project(&pool, "backoff").await;
    let mut conn = pool.acquire().await.unwrap();
    store::want_running(&mut conn, p, PROVIDER).await.unwrap();
    let retry_at = || async { sqlx::query_scalar::<_, Option<chrono::DateTime<Utc>>>("SELECT retry_at FROM project_sandboxes WHERE project_id = $1").bind(p).fetch_one(&pool).await.unwrap() };
    let set = |state: &'static str, desired: &'static str| {
        let pool = pool.clone();
        async move {
            sqlx::query("UPDATE project_sandboxes SET state = $2, desired = $3, retry_at = now() + interval '1 hour' WHERE project_id = $1").bind(p).bind(state).bind(desired).execute(&pool).await.unwrap();
        }
    };

    // Settled, desired flips: the user's open/close drops the leftover backoff.
    set("stopped", "stopped").await;
    store::want_running(&mut conn, p, PROVIDER).await.unwrap();
    assert!(retry_at().await.is_none());
    set("running", "running").await;
    store::want_stopped(&mut conn, p).await.unwrap();
    assert!(retry_at().await.is_none());

    // In flight: the retry_at is the walk's lease and stays, even when desired flips.
    set("starting", "stopped").await;
    store::want_running(&mut conn, p, PROVIDER).await.unwrap();
    assert!(retry_at().await.is_some());
    set("stopping", "running").await;
    store::want_stopped(&mut conn, p).await.unwrap();
    assert!(retry_at().await.is_some());
}

#[sqlx::test(migrations = "../../migrations")]
async fn check_in_does_not_clear_backoff_on_an_error_row(pool: PgPool) {
    let p = project(&pool, "errored").await;
    let mut conn = pool.acquire().await.unwrap();
    store::want_running(&mut conn, p, PROVIDER).await.unwrap();
    sqlx::query("UPDATE project_sandboxes SET state = 'error', retry_at = now() + interval '1 hour' WHERE project_id = $1").bind(p).execute(&pool).await.unwrap();
    store::want_running(&mut conn, p, PROVIDER).await.unwrap();
    let mut tx = pool.begin().await.unwrap();
    assert!(store::claim_due(&mut tx, CLAIM_LIMIT).await.unwrap().is_empty());
    tx.rollback().await.unwrap();
}

#[sqlx::test(migrations = "../../migrations")]
async fn repeated_want_running_keeps_the_walk_lease(pool: PgPool) {
    let p = project(&pool, "lease").await;
    let mut conn = pool.acquire().await.unwrap();
    store::want_running(&mut conn, p, PROVIDER).await.unwrap();
    let mut tx = pool.begin().await.unwrap();
    assert_eq!(store::claim_due(&mut tx, CLAIM_LIMIT).await.unwrap().len(), 1);
    tx.commit().await.unwrap();
    store::want_running(&mut conn, p, PROVIDER).await.unwrap();
    let mut tx = pool.begin().await.unwrap();
    assert!(store::claim_due(&mut tx, CLAIM_LIMIT).await.unwrap().is_empty());
    tx.rollback().await.unwrap();
}
