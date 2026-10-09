use nomi_agent_core::attachments::{expand_messages, read_attachment};
use nomi_llm::{ContentBlock, LlmMessage, LlmRole};
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

async fn user(pool: &PgPool) -> Uuid {
    sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(pool).await.unwrap()
}

/// Stores a file for `user_id` the way the upload route does.
async fn stored(pool: &PgPool, user_id: Uuid, kind: &str, mime: &str, status: &str, text: Option<&str>, bytes: &[u8]) -> Uuid {
    std::env::set_var("ATTACHMENTS_DIR", std::env::temp_dir().join(format!("nomi-core-attachments-{}", std::process::id())));
    let id = Uuid::new_v4();
    let key = format!("attachments/{user_id}/{id}");
    nomi_storage::blob::attachment_store().put_bytes(&key, bytes.to_vec(), mime).await.unwrap();
    sqlx::query(
        "INSERT INTO attachments (id, user_id, name, mime, kind, size_bytes, storage_key, status, extracted_text) VALUES ($1, $2, 'f', $3, $4, $5, $6, $7, $8)",
    )
    .bind(id)
    .bind(user_id)
    .bind(mime)
    .bind(kind)
    .bind(bytes.len() as i64)
    .bind(&key)
    .bind(status)
    .bind(text)
    .execute(pool)
    .await
    .unwrap();
    id
}

fn said(text: String) -> LlmMessage {
    LlmMessage { role: LlmRole::User, content: vec![ContentBlock::Text { text }] }
}

fn tag(id: Uuid, kind: &str, mime: &str) -> String {
    format!("<attachment id=\"{id}\" name=\"f\" kind=\"{kind}\" mime=\"{mime}\" size=\"4\"/>")
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_newest_messages_image_is_shown_to_the_model_and_older_files_become_text(pool: PgPool) {
    let me = user(&pool).await;
    let doc = stored(&pool, me, "text", "text/plain", "ready", Some("buy flights on friday"), b"buy flights on friday").await;
    let photo = stored(&pool, me, "image", "image/png", "pending", None, b"\x89PNG").await;

    let mut messages = vec![
        said(format!("plan this\n\n{}", tag(doc, "text", "text/plain"))),
        LlmMessage { role: LlmRole::Assistant, content: vec![ContentBlock::Text { text: "Sure".into() }] },
        said(format!("and log this\n\n{}", tag(photo, "image", "image/png"))),
    ];
    let mut conn = pool.acquire().await.unwrap();
    expand_messages(&mut conn, me, &mut messages).await;

    let ContentBlock::Text { text } = &messages[0].content[0] else { panic!() };
    assert!(text.contains("buy flights on friday"), "{text}");
    assert_eq!(messages[0].content.len(), 1, "older files are text only");

    let latest = &messages[2].content;
    assert!(matches!(&latest[0], ContentBlock::Text { text } if text.contains("attached to this message")));
    assert!(matches!(&latest[1], ContentBlock::Media { media_type, .. } if media_type == "image/png"));
}

#[sqlx::test(migrations = "../../migrations")]
async fn someone_elses_file_is_never_opened(pool: PgPool) {
    let owner = user(&pool).await;
    let other = user(&pool).await;
    let secret = stored(&pool, owner, "text", "text/plain", "ready", Some("the secret"), b"the secret").await;

    let mut messages = vec![said(tag(secret, "text", "text/plain"))];
    let mut conn = pool.acquire().await.unwrap();
    expand_messages(&mut conn, other, &mut messages).await;
    let ContentBlock::Text { text } = &messages[0].content[0] else { panic!() };
    assert!(!text.contains("the secret"));
    assert!(text.contains("no longer available"));
    assert!(read_attachment(&mut conn, other, &json!({ "id": secret.to_string() })).await.is_err());
}

#[sqlx::test(migrations = "../../migrations")]
async fn read_attachment_reads_on_from_an_offset(pool: PgPool) {
    let me = user(&pool).await;
    let long = "abcdefghij".repeat(3000);
    let id = stored(&pool, me, "text", "text/plain", "ready", Some(&long), long.as_bytes()).await;
    let mut conn = pool.acquire().await.unwrap();

    let first = read_attachment(&mut conn, me, &json!({ "id": id.to_string() })).await.unwrap();
    assert!(first.contains("characters 0 to 20000 of 30000"));
    assert!(first.contains("read on with offset 20000"));
    let rest = read_attachment(&mut conn, me, &json!({ "id": id.to_string(), "offset": 20000 })).await.unwrap();
    assert!(rest.contains("characters 20000 to 30000 of 30000"));
}
