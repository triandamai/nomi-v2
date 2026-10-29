//! `backend.json`'s buckets and realtime tables, saved per environment: buckets kept for the
//! gateway and mirrored into `nomi.buckets`, realtime tables given the trigger that announces
//! their changes.
use std::collections::HashMap;

use relay_control::backend::ensure_backend;
use relay_control::migrations::apply;
use relay_control::{realtime, storage, Relay, RelayError};
use relay_core::backend::StorageConfig;
use relay_core::names::schema_for;
use relay_core::{Env, Tier};
use relay_migrate::MigrationFile;
use serde_json::json;
use sqlx::postgres::PgListener;
use sqlx::PgPool;
use uuid::Uuid;

const INIT: &str = "create table message (\n  id bigint generated always as identity primary key,\n  body text not null\n);\nalter table message enable row level security;\ncreate policy \"anyone\" on message using (true) with check (true);\n-- nomi: public\ncreate table tag (name text);\n";

async fn migrated(pool: &PgPool) -> (Relay, Uuid) {
    let shared = HashMap::from([(Env::Dev, pool.clone()), (Env::Production, pool.clone())]);
    let relay = Relay::new(pool.clone(), shared, [9u8; 32], "api.relay.test".to_string());
    let project = Uuid::new_v4();
    ensure_backend(&relay, project, "Chat", Tier::Free, None).await.unwrap();
    apply(&relay, project, Env::Dev, &[MigrationFile { path: "nomi/migrations/0001_init.sql".into(), content: INIT.into() }]).await.unwrap();
    (relay, project)
}

fn config(value: serde_json::Value) -> StorageConfig {
    serde_json::from_value(value).unwrap()
}

#[sqlx::test(migrations = "../../migrations")]
async fn buckets_are_kept_per_environment_and_mirrored_into_nomi(pool: PgPool) {
    let (relay, project) = migrated(&pool).await;

    storage::save(&relay, project, Env::Dev, &config(json!({ "buckets": [{ "name": "avatars", "public": true }, { "name": "docs" }] }))).await.unwrap();
    storage::save(&relay, project, Env::Dev, &config(json!({ "buckets": [{ "name": "avatars", "public": false, "write": "owner" }] }))).await.unwrap();

    let saved = storage::load(&relay, project, Env::Dev).await.unwrap();
    assert_eq!(saved.buckets.len(), 1);
    assert_eq!(saved.buckets[0].write, relay_core::backend::Rule::Owner);
    assert!(storage::load(&relay, project, Env::Production).await.unwrap().buckets.is_empty());
    let mirrored: Vec<(String, bool)> = sqlx::query_as("SELECT name, public FROM nomi.buckets WHERE project_id = $1").bind(project).fetch_all(&pool).await.unwrap();
    assert_eq!(mirrored, vec![("avatars".to_string(), false)]);
}

#[sqlx::test(migrations = "../../migrations")]
async fn bad_buckets_are_refused_with_what_is_wrong(pool: PgPool) {
    let (relay, project) = migrated(&pool).await;

    let refused = storage::save(&relay, project, Env::Dev, &config(json!({ "buckets": [{ "name": "Avatars" }] }))).await.unwrap_err();

    match refused {
        RelayError::Rejected { code, message, .. } => {
            assert_eq!(code, "BACKEND_INVALID");
            assert!(message.contains("Avatars"), "{message}");
        }
        other => panic!("{other:?}"),
    }
    assert!(storage::save(&relay, Uuid::new_v4(), Env::Dev, &StorageConfig::default()).await.is_err(), "no backend");
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_realtime_table_announces_each_change_with_its_key(pool: PgPool) {
    let (relay, project) = migrated(&pool).await;
    let mut listener = PgListener::connect_with(&pool).await.unwrap();
    listener.listen(realtime::CHANNEL).await.unwrap();

    realtime::save(&relay, project, Env::Dev, &["message".to_string()]).await.unwrap();
    let schema = schema_for(project, Tier::Free);
    sqlx::query(&format!("INSERT INTO \"{schema}\".message (body) VALUES ('hi')")).execute(&pool).await.unwrap();
    sqlx::query(&format!("DELETE FROM \"{schema}\".message")).execute(&pool).await.unwrap();

    for op in ["insert", "delete"] {
        let note = tokio::time::timeout(std::time::Duration::from_secs(5), listener.recv()).await.unwrap().unwrap();
        let payload: serde_json::Value = serde_json::from_str(note.payload()).unwrap();
        assert_eq!(payload, json!({ "p": project.to_string(), "s": schema, "t": "message", "op": op, "k": { "id": 1 } }));
    }
    assert_eq!(realtime::load(&relay, project, Env::Dev).await.unwrap(), vec!["message".to_string()]);
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_table_taken_off_the_list_stops_announcing(pool: PgPool) {
    let (relay, project) = migrated(&pool).await;
    realtime::save(&relay, project, Env::Dev, &["message".to_string()]).await.unwrap();

    realtime::save(&relay, project, Env::Dev, &[]).await.unwrap();

    let triggers: i64 = sqlx::query_scalar("SELECT count(*) FROM pg_trigger WHERE tgname = 'nomi_realtime'").fetch_one(&pool).await.unwrap();
    assert_eq!(triggers, 0);
    assert!(realtime::load(&relay, project, Env::Dev).await.unwrap().is_empty());
}

#[sqlx::test(migrations = "../../migrations")]
async fn missing_tables_and_tables_without_a_key_are_refused(pool: PgPool) {
    let (relay, project) = migrated(&pool).await;

    let refused = realtime::save(&relay, project, Env::Dev, &["message".to_string(), "order".to_string(), "tag".to_string()]).await.unwrap_err();

    let RelayError::Rejected { code, message, .. } = refused else { panic!("not a refusal") };
    assert_eq!(code, "BACKEND_INVALID");
    assert!(message.contains("no table order"), "{message}");
    assert!(message.contains("tag has no primary key"), "{message}");
    let triggers: i64 = sqlx::query_scalar("SELECT count(*) FROM pg_trigger WHERE tgname = 'nomi_realtime'").fetch_one(&pool).await.unwrap();
    assert_eq!(triggers, 0, "nothing changes when any table is refused");
}

#[sqlx::test(migrations = "../../migrations")]
async fn deleting_a_backend_deletes_its_files_and_nobody_elses(pool: PgPool) {
    use object_store::{memory::InMemory, ObjectStore, PutPayload};
    let store = std::sync::Arc::new(InMemory::new());
    let shared = HashMap::from([(Env::Dev, pool.clone()), (Env::Production, pool.clone())]);
    let relay = Relay::new(pool.clone(), shared, [9u8; 32], "api.relay.test".to_string()).with_store(store.clone());
    let (doomed, kept) = (Uuid::new_v4(), Uuid::new_v4());
    for project in [doomed, kept] {
        ensure_backend(&relay, project, "Files", Tier::Free, None).await.unwrap();
        for env in [Env::Dev, Env::Production] {
            store.put(&relay_control::files::object_path(project, env, "docs", "a/b.txt"), PutPayload::from_static(b"x")).await.unwrap();
        }
    }

    relay_control::backend::delete_backend(&relay, doomed).await.unwrap();

    let left: Vec<String> = futures::TryStreamExt::try_collect::<Vec<_>>(store.list(None)).await.unwrap().into_iter().map(|o| o.location.to_string()).collect();
    assert_eq!(left.len(), 2, "{left:?}");
    assert!(left.iter().all(|p| p.starts_with(&kept.to_string())), "{left:?}");
}
