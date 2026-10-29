//! SQL from a project's console (its admins) and from Iris (reads only), run as the project's own
//! database login: it sees its own tables and nothing of any other project's.
use std::collections::HashMap;

use relay_control::backend::ensure_backend;
use relay_control::migrations::apply;
use relay_control::query::{run, Access};
use relay_control::{Relay, RelayError};
use relay_core::{Env, Tier};
use relay_migrate::MigrationFile;
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

const INIT: &str = "create table task (\n  id bigint generated always as identity primary key,\n  title text not null,\n  done boolean not null default false\n);\nalter table task enable row level security;\n";

async fn project(relay: &Relay, name: &str) -> Uuid {
    let project = Uuid::new_v4();
    ensure_backend(relay, project, name, Tier::Free, None).await.unwrap();
    apply(relay, project, Env::Dev, &[MigrationFile { path: "nomi/migrations/0001_init.sql".into(), content: INIT.into() }]).await.unwrap();
    project
}

fn relay(pool: &PgPool) -> Relay {
    let shared = HashMap::from([(Env::Dev, pool.clone()), (Env::Production, pool.clone())]);
    Relay::new(pool.clone(), shared, [9u8; 32], "api.relay.test".to_string())
}

fn refused(result: Result<relay_control::query::QueryResult, RelayError>) -> String {
    match result {
        Err(RelayError::Rejected { code: "QUERY_FAILED", message, .. }) => message,
        other => panic!("not refused: {other:?}"),
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_console_writes_and_reads_its_own_tables_past_row_level_security(pool: PgPool) {
    let relay = relay(&pool);
    let project = project(&relay, "Todo").await;

    let wrote = run(&relay, project, Env::Dev, "insert into task (title) values ('milk'), ('eggs')", Access::ReadWrite).await.unwrap();
    assert_eq!(wrote.rows_affected, Some(2));

    let read = run(&relay, project, Env::Dev, "select title, done from task order by id;", Access::ReadOnly).await.unwrap();
    assert_eq!(read.columns, vec!["title", "done"]);
    assert_eq!(read.rows, vec![vec![json!("milk"), json!(false)], vec![json!("eggs"), json!(false)]]);
    assert!(!read.truncated);
}

#[sqlx::test(migrations = "../../migrations")]
async fn reads_only_never_change_anything(pool: PgPool) {
    let relay = relay(&pool);
    let project = project(&relay, "Todo").await;

    refused(run(&relay, project, Env::Dev, "delete from task", Access::ReadOnly).await);
    // A read that writes anyway is stopped by the read-only transaction.
    run(&relay, project, Env::Dev, "create function nuke() returns int language sql as 'delete from task returning 1'", Access::ReadWrite).await.unwrap();
    refused(run(&relay, project, Env::Dev, "select nuke()", Access::ReadOnly).await);
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_project_cannot_reach_another_projects_tables_or_switch_roles(pool: PgPool) {
    let relay = relay(&pool);
    let mine = project(&relay, "Mine").await;
    let theirs = project(&relay, "Theirs").await;
    run(&relay, theirs, Env::Dev, "insert into task (title) values ('secret')", Access::ReadWrite).await.unwrap();
    let their_schema = relay.placement(theirs, Env::Dev).await.unwrap().schema;

    refused(run(&relay, mine, Env::Dev, &format!("select * from \"{their_schema}\".task"), Access::ReadOnly).await);
    // The login is the project's own, so there is no other role to go back to.
    let relay_user: String = sqlx::query_scalar("select current_user::text").fetch_one(&pool).await.unwrap();
    refused(run(&relay, mine, Env::Dev, &format!("select set_config('role', '{relay_user}', false)"), Access::ReadOnly).await);
    refused(run(&relay, mine, Env::Dev, "reset role", Access::ReadWrite).await);
    let who = run(&relay, mine, Env::Dev, "select session_user::text as who, (select rolsuper from pg_roles where rolname = session_user) as superuser", Access::ReadOnly).await.unwrap();
    assert_eq!(who.rows[0][1], json!(false));
    assert!(who.rows[0][0].as_str().unwrap().ends_with("_owner"), "{:?}", who.rows);
}

#[sqlx::test(migrations = "../../migrations")]
async fn one_statement_at_a_time_and_big_reads_are_cut(pool: PgPool) {
    let relay = relay(&pool);
    let project = project(&relay, "Todo").await;

    refused(run(&relay, project, Env::Dev, "insert into task (title) values ('a'); drop table task", Access::ReadWrite).await);
    refused(run(&relay, project, Env::Dev, "  ", Access::ReadWrite).await);
    let many = run(&relay, project, Env::Dev, "select g from generate_series(1, 2000) g", Access::ReadOnly).await.unwrap();
    assert!(many.truncated);
    assert_eq!(many.rows.len(), relay_control::query::MAX_ROWS);
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_project_sees_only_its_own_signed_in_users(pool: PgPool) {
    let relay = relay(&pool);
    let mine = project(&relay, "Mine").await;
    let theirs = project(&relay, "Theirs").await;
    for (project, email) in [(mine, "ana@mine.test"), (theirs, "bo@theirs.test")] {
        sqlx::query("INSERT INTO nomi.users (project_id, email) VALUES ($1, $2)").bind(project).bind(email).execute(&pool).await.unwrap();
    }

    let users = run(&relay, mine, Env::Dev, "select email from nomi.users", Access::ReadOnly).await.unwrap();

    assert_eq!(users.rows, vec![vec![json!("ana@mine.test")]]);
}
