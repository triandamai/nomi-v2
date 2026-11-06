//! The background workers' passes that run without a loop: reading attached files, learning
//! from exchanges, and the memory sweep. Which model reads a file depends on the provider this
//! test runs with, so outcomes are checked for being settled rather than for one model's words.

use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

const KEY: [u8; 32] = nomi_test_support::TEST_SETTINGS_KEY;

async fn person_and_chat(pool: &PgPool) -> (Uuid, Uuid) {
    let user: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(pool).await.unwrap();
    let org: Uuid = sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Acme') RETURNING id").fetch_one(pool).await.unwrap();
    let session: Uuid = sqlx::query_scalar("INSERT INTO sessions (org_id, channel, chat_id, user_id) VALUES ($1, 'web', $2, $3) RETURNING id")
        .bind(org)
        .bind(user.to_string())
        .bind(user)
        .fetch_one(pool)
        .await
        .unwrap();
    (user, session)
}

async fn attachment(pool: &PgPool, user: Uuid, kind: &str, mime: &str, key: &str, extra: &str) -> Uuid {
    sqlx::query_scalar(&format!(
        "INSERT INTO attachments (user_id, name, mime, kind, size_bytes, storage_key{}) VALUES ($1, 'file', $2, $3, 3, $4{}) RETURNING id",
        if extra.is_empty() { "" } else { ", attempts, status, created_at" },
        if extra.is_empty() { String::new() } else { format!(", {extra}") }
    ))
    .bind(user)
    .bind(mime)
    .bind(kind)
    .bind(key)
    .fetch_one(pool)
    .await
    .unwrap()
}

async fn status_of(pool: &PgPool, id: Uuid) -> Option<String> {
    sqlx::query_scalar("SELECT status FROM attachments WHERE id = $1").bind(id).fetch_optional(pool).await.unwrap()
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_pass_reads_waiting_files_gives_up_on_stuck_ones_and_clears_abandoned_ones(pool: PgPool) {
    let dir = std::env::temp_dir().join(format!("nomi-attachments-{}", Uuid::new_v4()));
    std::env::set_var("ATTACHMENTS_DIR", &dir);
    nomi_storage::blob::update_attachment_store(None);
    let store = nomi_storage::blob::attachment_store();
    let (user, _) = person_and_chat(&pool).await;
    store.put_bytes("u/voice.webm", b"abc".to_vec(), "audio/webm").await.unwrap();
    store.put_bytes("u/old.txt", b"old".to_vec(), "text/plain").await.unwrap();

    let voice = attachment(&pool, user, "voice", "audio/webm", "u/voice.webm", "").await;
    let missing = attachment(&pool, user, "image", "image/png", "u/missing.png", "").await;
    let tired = attachment(&pool, user, "pdf", "application/pdf", "u/x.pdf", "5, 'pending', now()").await;
    let abandoned = attachment(&pool, user, "text", "text/plain", "u/old.txt", "0, 'ready', now() - interval '7 hours'").await;

    nomi_server::attachment_worker::pass(&pool, &KEY, &reqwest::Client::new()).await.unwrap();

    assert_eq!(status_of(&pool, tired).await.as_deref(), Some("failed"), "too many tries");
    assert_eq!(status_of(&pool, abandoned).await, None, "an unsent file is removed after six hours");
    assert_eq!(store.get_bytes("u/old.txt").await.unwrap(), None, "with its stored bytes");
    for id in [voice, missing] {
        let status = status_of(&pool, id).await.unwrap();
        assert_ne!(status, "processing", "every claimed file is settled or handed back");
    }
    let details: serde_json::Value = sqlx::query_scalar("SELECT details FROM attachments WHERE id = $1").bind(tired).fetch_one(&pool).await.unwrap();
    assert!(details["error"].as_str().unwrap().contains("several tries"));
    // Nothing waiting: a quiet pass.
    sqlx::query("UPDATE attachments SET status = 'ready'").execute(&pool).await.unwrap();
    nomi_server::attachment_worker::pass(&pool, &KEY, &reqwest::Client::new()).await.unwrap();
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn each_kind_of_file_has_its_own_instructions() {
    use nomi_attachments::Kind;
    let all: std::collections::HashSet<&str> = [Kind::Image, Kind::Voice, Kind::Audio, Kind::Video, Kind::Pdf].into_iter().map(nomi_server::attachment_worker::instructions).collect();
    assert_eq!(all.len(), 5);
    assert!(nomi_server::attachment_worker::instructions(Kind::Voice).contains("Transcribe"));
}

#[sqlx::test(migrations = "../../migrations")]
async fn learning_and_sweeping_settle_what_was_queued(pool: PgPool) {
    let (user, session) = person_and_chat(&pool).await;
    let identity: Uuid = nomi_turn::bootstrap::ensure_web_channel_identity(&pool, user).await.unwrap();
    for i in 0..30 {
        let sender = if i % 2 == 0 { Some(identity) } else { None };
        sqlx::query("INSERT INTO messages (session_id, sender_channel_identity_id, content, created_at) VALUES ($1, $2, $3, now() - make_interval(mins => $4))")
            .bind(session)
            .bind(sender)
            .bind(format!("message {i}: I take the MRT to work"))
            .bind(60 - i)
            .execute(&pool)
            .await
            .unwrap();
    }
    let mut conn = pool.acquire().await.unwrap();
    nomi_agent_core::memory::queue_learning(&mut conn, user, session, "I take the MRT to work", "Noted!").await;
    drop(conn);
    let queued: i64 = sqlx::query_scalar("SELECT count(*) FROM memory_jobs").fetch_one(&pool).await.unwrap();
    assert_eq!(queued, 1);

    let http = reqwest::Client::new();
    nomi_server::memory_worker::learn(&pool, &KEY, &http).await;
    let attempts: Option<i32> = sqlx::query_scalar("SELECT attempts FROM memory_jobs").fetch_optional(&pool).await.unwrap();
    assert!(attempts.is_none() || attempts == Some(1), "learned and removed, or tried once and kept: {attempts:?}");

    nomi_server::memory_worker::sweep(&pool, &KEY, &http).await;
    // Nothing queued any more: a quiet pass.
    nomi_server::memory_worker::learn(&pool, &KEY, &http).await;
    let _ = json!({});
}
