//! A project's schema read back: as a description for Koda and as TypeScript for the app.
use std::collections::HashMap;

use relay_control::backend::ensure_backend;
use relay_control::migrations::apply;
use relay_control::schema::{describe, types};
use relay_control::Relay;
use relay_core::{Env, Tier};
use relay_migrate::MigrationFile;
use sqlx::PgPool;
use uuid::Uuid;

const INIT: &str = "create table task (\n  id bigint generated always as identity primary key,\n  owner uuid not null default nomi.uid() references nomi.users(id) on delete cascade,\n  title text not null,\n  done boolean not null default false,\n  notes text,\n  tags text[] not null default '{}',\n  meta jsonb,\n  created_at timestamptz not null default now()\n);\nalter table task enable row level security;\ncreate policy \"own tasks\" on task using (owner = nomi.uid()) with check (owner = nomi.uid());\n";

async fn migrated(pool: &PgPool) -> (Relay, Uuid) {
    let shared = HashMap::from([(Env::Dev, pool.clone()), (Env::Production, pool.clone())]);
    let relay = Relay::new(pool.clone(), shared, [9u8; 32], "api.relay.test".to_string());
    let project = Uuid::new_v4();
    ensure_backend(&relay, project, "Todo", Tier::Free, None).await.unwrap();
    apply(&relay, project, Env::Dev, &[MigrationFile { path: "nomi/migrations/0001_init.sql".into(), content: INIT.into() }]).await.unwrap();
    (relay, project)
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_schema_says_tables_columns_and_policies(pool: PgPool) {
    let (relay, project) = migrated(&pool).await;

    let schema = describe(&relay, project, Env::Dev).await.unwrap();

    assert_eq!(schema.tables.len(), 1);
    let task = &schema.tables[0];
    assert_eq!(task.name, "task");
    assert!(task.rls_enabled);
    let column = |name: &str| task.columns.iter().find(|c| c.name == name).unwrap();
    assert_eq!(column("title").r#type, "text");
    assert!(!column("title").nullable);
    assert!(column("notes").nullable);
    assert!(column("done").default.as_deref().is_some_and(|d| d.contains("false")));
    assert_eq!(task.policies.len(), 1);
    assert_eq!(task.policies[0].name, "own tasks");
    assert!(task.policies[0].using.as_deref().is_some_and(|u| u.contains("nomi.uid()")), "{:?}", task.policies[0].using);
}

#[sqlx::test(migrations = "../../migrations")]
async fn types_match_what_the_client_expects(pool: PgPool) {
    let (relay, project) = migrated(&pool).await;

    let ts = types(&relay, project, Env::Dev).await.unwrap();

    assert!(ts.contains("export interface Database {"), "{ts}");
    assert!(ts.contains("task: {"), "{ts}");
    assert!(ts.contains("Row: { id: number; owner: string; title: string; done: boolean; notes: string | null; tags: string[]; meta: Json | null; created_at: string };"), "{ts}");
    assert!(ts.contains("Insert: { id?: number; owner?: string; title: string; done?: boolean; notes?: string | null; tags?: string[]; meta?: Json | null; created_at?: string };"), "{ts}");
    assert!(ts.contains("Update: { id?: number; owner?: string; title?: string; done?: boolean; notes?: string | null; tags?: string[]; meta?: Json | null; created_at?: string };"), "{ts}");
    assert!(ts.contains("functions: Record<string, never>;"), "{ts}");
}
