//! Making and deleting a project's backend, against a real Postgres (the test database stands in
//! for Relay's state and for both shared databases).
use std::collections::HashMap;

use relay_control::backend::{delete_backend, ensure_backend, Databases};
use relay_control::Relay;
use relay_core::{Env, Tier};
use sqlx::PgPool;
use uuid::Uuid;

fn relay(pool: &PgPool) -> Relay {
    let shared = HashMap::from([(Env::Dev, pool.clone()), (Env::Production, pool.clone())]);
    Relay::new(pool.clone(), shared, [9u8; 32], "api.relay.test".to_string())
}

async fn exists(pool: &PgPool, sql: &str, name: &str) -> bool {
    sqlx::query_scalar(sql).bind(name).fetch_one(pool).await.unwrap()
}
const SCHEMA_EXISTS: &str = "SELECT EXISTS (SELECT 1 FROM pg_namespace WHERE nspname = $1)";
const ROLE_EXISTS: &str = "SELECT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = $1)";

#[sqlx::test(migrations = "../../migrations")]
async fn a_free_project_gets_a_schema_roles_the_nomi_schema_and_keys(pool: PgPool) {
    let relay = relay(&pool);
    let project = Uuid::new_v4();

    let backend = ensure_backend(&relay, project, "Bakery", Tier::Free, None).await.unwrap();

    let schema = relay_core::names::schema_for(project, Tier::Free);
    assert!(exists(&pool, SCHEMA_EXISTS, &schema).await);
    assert!(exists(&pool, SCHEMA_EXISTS, "nomi").await);
    for kind in relay_core::names::RoleKind::ALL {
        assert!(exists(&pool, ROLE_EXISTS, &relay_core::names::role(project, kind)).await);
    }
    assert_eq!(backend.id, project);
    let dev = &backend.environments["dev"];
    assert!(dev.url.starts_with("https://bakery-") && dev.url.ends_with("-dev.api.relay.test"));
    assert_ne!(dev.anon_key, backend.environments["production"].anon_key);
    assert_ne!(dev.anon_key, dev.service_key);
}

#[sqlx::test(migrations = "../../migrations")]
async fn asking_again_changes_nothing_and_answers_the_same_keys(pool: PgPool) {
    let relay = relay(&pool);
    let project = Uuid::new_v4();
    let first = ensure_backend(&relay, project, "Bakery", Tier::Free, None).await.unwrap();
    let second = ensure_backend(&relay, project, "Bakery renamed", Tier::Free, None).await.unwrap();
    assert_eq!(first.environments["production"].anon_key, second.environments["production"].anon_key);
    assert_eq!(first.environments["dev"].url, second.environments["dev"].url);
}

/// Runs `sql` as `role` with `claims`, the way the gateway will, and says whether it worked and
/// what it returned (a count).
async fn as_role(pool: &PgPool, role: &str, sql: &str) -> Result<i64, sqlx::Error> {
    let mut tx = pool.begin().await?;
    sqlx::query(&format!("SET LOCAL ROLE \"{role}\"")).execute(&mut *tx).await?;
    let count = sqlx::query_scalar(sql).fetch_one(&mut *tx).await;
    tx.rollback().await?;
    count
}

#[sqlx::test(migrations = "../../migrations")]
async fn two_free_projects_cant_see_each_other(pool: PgPool) {
    let relay = relay(&pool);
    let (a, b) = (Uuid::new_v4(), Uuid::new_v4());
    ensure_backend(&relay, a, "A", Tier::Free, None).await.unwrap();
    ensure_backend(&relay, b, "B", Tier::Free, None).await.unwrap();
    let schema_b = relay_core::names::schema_for(b, Tier::Free);
    let owner_b = relay_core::names::role(b, relay_core::names::RoleKind::Owner);
    let mut tx = pool.begin().await.unwrap();
    sqlx::query(&format!("SET LOCAL ROLE \"{owner_b}\"")).execute(&mut *tx).await.unwrap();
    sqlx::query(&format!("CREATE TABLE \"{schema_b}\".secret (id int)")).execute(&mut *tx).await.unwrap();
    tx.commit().await.unwrap();
    for (project, email) in [(a, "a@a.test"), (b, "b@b.test")] {
        sqlx::query("INSERT INTO nomi.users (project_id, email) VALUES ($1, $2)").bind(project).bind(email).execute(&pool).await.unwrap();
    }
    let authenticated_a = relay_core::names::role(a, relay_core::names::RoleKind::Authenticated);
    let owner_a = relay_core::names::role(a, relay_core::names::RoleKind::Owner);

    assert!(as_role(&pool, &authenticated_a, &format!("SELECT count(*) FROM \"{schema_b}\".secret")).await.is_err());
    assert!(as_role(&pool, &owner_a, &format!("SELECT count(*) FROM \"{schema_b}\".secret")).await.is_err());
    assert_eq!(as_role(&pool, &authenticated_a, "SELECT count(*) FROM nomi.users").await.unwrap(), 1);
    assert_eq!(as_role(&pool, &authenticated_a, "SELECT count(*) FROM nomi.users WHERE email = 'b@b.test'").await.unwrap(), 0);
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_dedicated_project_uses_public_in_its_own_database(pool: PgPool) {
    let relay = relay(&pool);
    let project = Uuid::new_v4();
    let url = std::env::var("DATABASE_URL").unwrap();
    // The test database itself stands in for the project's own databases.
    let current: String = sqlx::query_scalar("SELECT current_database()").fetch_one(&pool).await.unwrap();
    let url = format!("{}/{current}", url.rsplit_once('/').unwrap().0);
    let databases = Databases { dev: url.clone(), production: url };

    let backend = ensure_backend(&relay, project, "Big", Tier::Dedicated, Some(databases)).await.unwrap();

    assert_eq!(backend.environments.len(), 2);
    let owner = relay_core::names::role(project, relay_core::names::RoleKind::Owner);
    let can_create: bool = sqlx::query_scalar("SELECT has_schema_privilege($1, 'public', 'CREATE')").bind(&owner).fetch_one(&pool).await.unwrap();
    assert!(can_create);
    let sealed: Option<Vec<u8>> = sqlx::query_scalar("SELECT database_url_sealed FROM relay.environments WHERE project_id = $1 AND env = 'dev'")
        .bind(project)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert!(sealed.is_some_and(|s| !String::from_utf8_lossy(&s).contains("postgres://")));
}

#[sqlx::test(migrations = "../../migrations")]
async fn deleting_removes_exactly_that_projects_backend(pool: PgPool) {
    let relay = relay(&pool);
    let (gone, kept) = (Uuid::new_v4(), Uuid::new_v4());
    ensure_backend(&relay, gone, "Gone", Tier::Free, None).await.unwrap();
    ensure_backend(&relay, kept, "Kept", Tier::Free, None).await.unwrap();
    for project in [gone, kept] {
        sqlx::query("INSERT INTO nomi.users (project_id, email) VALUES ($1, 'x@x.test')").bind(project).execute(&pool).await.unwrap();
    }

    delete_backend(&relay, gone).await.unwrap();

    assert!(!exists(&pool, SCHEMA_EXISTS, &relay_core::names::schema_for(gone, Tier::Free)).await);
    assert!(!exists(&pool, ROLE_EXISTS, &relay_core::names::role(gone, relay_core::names::RoleKind::Owner)).await);
    assert!(exists(&pool, SCHEMA_EXISTS, &relay_core::names::schema_for(kept, Tier::Free)).await);
    let users: Vec<Uuid> = sqlx::query_scalar("SELECT project_id FROM nomi.users").fetch_all(&pool).await.unwrap();
    assert_eq!(users, vec![kept]);
    let state: i64 = sqlx::query_scalar("SELECT count(*) FROM relay.projects WHERE id = $1").bind(gone).fetch_one(&pool).await.unwrap();
    assert_eq!(state, 0);
}
