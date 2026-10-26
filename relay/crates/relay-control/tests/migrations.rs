//! Applying a project's migrations to one of its environments, against a real Postgres.
use std::collections::HashMap;

use relay_control::backend::ensure_backend;
use relay_control::migrations::{apply, STATEMENT_TIMEOUT};
use relay_control::{Relay, RelayError};
use relay_core::{Env, Tier};
use relay_migrate::MigrationFile;
use sqlx::PgPool;
use uuid::Uuid;

async fn project(pool: &PgPool) -> (Relay, Uuid) {
    let shared = HashMap::from([(Env::Dev, pool.clone()), (Env::Production, pool.clone())]);
    let relay = Relay::new(pool.clone(), shared, [9u8; 32], "api.relay.test".to_string());
    let project = Uuid::new_v4();
    ensure_backend(&relay, project, "Todo", Tier::Free, None).await.unwrap();
    (relay, project)
}

fn file(name: &str, content: &str) -> MigrationFile {
    MigrationFile { path: format!("nomi/migrations/{name}"), content: content.to_string() }
}

const INIT: &str = "create table task (\n  id bigint generated always as identity primary key,\n  owner uuid not null default nomi.uid() references nomi.users(id) on delete cascade,\n  title text not null\n);\nalter table task enable row level security;\ncreate policy \"own tasks\" on task using (owner = nomi.uid()) with check (owner = nomi.uid());\n";
const TAGS: &str = "create table tag (id int primary key, name text);\nalter table tag enable row level security;\n";

fn rejected(result: Result<Vec<String>, RelayError>) -> (&'static str, String, serde_json::Value) {
    match result {
        Err(RelayError::Rejected { code, message, details }) => (code, message, details),
        other => panic!("expected a refusal, got {other:?}"),
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn migrations_apply_in_order_once_each(pool: PgPool) {
    let (relay, project) = project(&pool).await;
    let files = vec![file("0002_tags.sql", TAGS), file("0001_init.sql", INIT)];

    let applied = apply(&relay, project, Env::Dev, &files).await.unwrap();
    assert_eq!(applied, vec!["0001_init.sql", "0002_tags.sql"]);
    let again = apply(&relay, project, Env::Dev, &files).await.unwrap();
    assert!(again.is_empty());

    let schema = relay_core::names::schema_for(project, Tier::Free);
    let tables: i64 = sqlx::query_scalar("SELECT count(*) FROM pg_tables WHERE schemaname = $1").bind(&schema).fetch_one(&pool).await.unwrap();
    assert_eq!(tables, 2);
    let owner: String = sqlx::query_scalar("SELECT tableowner FROM pg_tables WHERE schemaname = $1 AND tablename = 'task'").bind(&schema).fetch_one(&pool).await.unwrap();
    assert_eq!(owner, relay_core::names::role(project, relay_core::names::RoleKind::Owner));
    let recorded: Vec<String> = sqlx::query_scalar("SELECT name FROM nomi.migrations WHERE project_id = $1 ORDER BY name").bind(project).fetch_all(&pool).await.unwrap();
    assert_eq!(recorded, vec!["0001_init.sql", "0002_tags.sql"]);
}

#[sqlx::test(migrations = "../../migrations")]
async fn an_applied_migration_that_was_edited_is_refused(pool: PgPool) {
    let (relay, project) = project(&pool).await;
    apply(&relay, project, Env::Dev, &[file("0001_init.sql", INIT)]).await.unwrap();

    let edited = format!("{INIT}-- one more line\n");
    let (code, _, details) = rejected(apply(&relay, project, Env::Dev, &[file("0001_init.sql", &edited), file("0002_tags.sql", TAGS)]).await);

    assert_eq!(code, "MIGRATION_EDITED");
    assert_eq!(details["file"], "0001_init.sql");
    let recorded: i64 = sqlx::query_scalar("SELECT count(*) FROM nomi.migrations WHERE project_id = $1").bind(project).fetch_one(&pool).await.unwrap();
    assert_eq!(recorded, 1, "0002 must not run after the refusal");
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_failing_statement_rolls_back_its_file_and_says_where(pool: PgPool) {
    let (relay, project) = project(&pool).await;
    let broken = "create table note (id int);\nalter table note enable row level security;\ninsert into nope values (1);\n";

    let (code, message, details) = rejected(apply(&relay, project, Env::Dev, &[file("0001_init.sql", INIT), file("0002_note.sql", broken)]).await);

    assert_eq!(code, "MIGRATION_FAILED");
    assert!(message.contains("nope"), "{message}");
    assert_eq!(details["file"], "0002_note.sql");
    assert_eq!(details["line"], 3);
    let schema = relay_core::names::schema_for(project, Tier::Free);
    let tables: Vec<String> = sqlx::query_scalar("SELECT tablename::text FROM pg_tables WHERE schemaname = $1 ORDER BY 1").bind(&schema).fetch_all(&pool).await.unwrap();
    assert_eq!(tables, vec!["task"], "0001 stays applied, 0002 rolled back entirely");
}

#[sqlx::test(migrations = "../../migrations")]
async fn lint_is_checked_before_anything_runs(pool: PgPool) {
    let (relay, project) = project(&pool).await;

    let (code, _, details) = rejected(apply(&relay, project, Env::Dev, &[file("0001_init.sql", INIT), file("0002_bad.sql", "set role postgres;\n")]).await);

    assert_eq!(code, "LINT_FAILED");
    assert_eq!(details["file"], "0002_bad.sql");
    let recorded: i64 = sqlx::query_scalar("SELECT count(*) FROM nomi.migrations WHERE project_id = $1").bind(project).fetch_one(&pool).await.unwrap();
    assert_eq!(recorded, 0);
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_badly_named_file_is_refused(pool: PgPool) {
    let (relay, project) = project(&pool).await;
    let (code, _, details) = rejected(apply(&relay, project, Env::Dev, &[file("init.sql", INIT)]).await);
    assert_eq!(code, "LINT_FAILED");
    assert_eq!(details["file"], "init.sql");
}

#[sqlx::test(migrations = "../../migrations")]
async fn migrations_run_with_a_statement_timeout_as_the_projects_owner(pool: PgPool) {
    let (relay, project) = project(&pool).await;
    let seen = "create table seen as select current_setting('statement_timeout') as timeout, current_user::text as who;\nalter table seen enable row level security;\n";

    apply(&relay, project, Env::Dev, &[file("0001_seen.sql", seen)]).await.unwrap();

    let schema = relay_core::names::schema_for(project, Tier::Free);
    let (timeout, who): (String, String) = sqlx::query_as(&format!("SELECT timeout, who FROM \"{schema}\".seen")).fetch_one(&pool).await.unwrap();
    assert_eq!(timeout, STATEMENT_TIMEOUT);
    assert_eq!(who, relay_core::names::role(project, relay_core::names::RoleKind::Owner));
}
