//! Every sandbox a project has had: a record per provider sandbox, kept by the database from the
//! project's one live sandbox row, so the worker needs no change and can't forget it.

mod support;

use sqlx::PgPool;
use support::App;
use uuid::Uuid;

async fn project(pool: &PgPool, name: &str) -> Uuid {
    let app = App::new(pool.clone());
    let owner = app.person(&format!("{name}@instances.test")).await;
    app.project(&owner, name).await.0
}

/// (sandbox id, state, ended) for each record, oldest first.
async fn records(pool: &PgPool, project_id: Uuid) -> Vec<(String, String, bool)> {
    sqlx::query_as("SELECT sandbox_id, state, ended_at IS NOT NULL FROM sandbox_instances WHERE project_id = $1 ORDER BY created_at, id")
        .bind(project_id)
        .fetch_all(pool)
        .await
        .unwrap()
}

async fn set(pool: &PgPool, project_id: Uuid, sandbox_id: Option<&str>, state: &str) {
    sqlx::query("UPDATE project_sandboxes SET sandbox_id = $2, state = $3 WHERE project_id = $1").bind(project_id).bind(sandbox_id).bind(state).execute(pool).await.unwrap();
}

#[sqlx::test(migrations = "../../migrations")]
async fn each_provider_sandbox_becomes_a_record_that_follows_its_state(pool: PgPool) {
    let id = project(&pool, "bakery").await;
    sqlx::query("INSERT INTO project_sandboxes (project_id, provider) VALUES ($1, 'boat')").bind(id).execute(&pool).await.unwrap();
    assert_eq!(records(&pool, id).await, vec![], "no provider sandbox yet");

    set(&pool, id, Some("box-1"), "creating").await;
    set(&pool, id, Some("box-1"), "running").await;
    assert_eq!(records(&pool, id).await, vec![("box-1".into(), "running".into(), false)]);

    // Boat deleted it while it was stopped: the record ends, and the next one is a new record.
    set(&pool, id, Some("box-1"), "stopped").await;
    set(&pool, id, None, "gone").await;
    set(&pool, id, Some("box-2"), "starting").await;
    assert_eq!(records(&pool, id).await, vec![("box-1".into(), "gone".into(), true), ("box-2".into(), "starting".into(), false)]);
    let open: i64 = sqlx::query_scalar("SELECT count(*) FROM sandbox_instances WHERE project_id = $1 AND ended_at IS NULL").bind(id).fetch_one(&pool).await.unwrap();
    assert_eq!(open, 1, "one live sandbox per project");
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_sandbox_replaced_without_going_away_first_is_ended_too(pool: PgPool) {
    let id = project(&pool, "kopi").await;
    sqlx::query("INSERT INTO project_sandboxes (project_id, provider, sandbox_id, state) VALUES ($1, 'boat', 'box-a', 'running')").bind(id).execute(&pool).await.unwrap();
    set(&pool, id, Some("box-b"), "starting").await;
    assert_eq!(records(&pool, id).await, vec![("box-a".into(), "running".into(), true), ("box-b".into(), "starting".into(), false)]);
}
