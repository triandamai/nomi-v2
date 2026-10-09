//! Reads the files only a model can read, soon after they're uploaded: describes images (and
//! copies out any text in them), transcribes audio and voice notes, watches videos, and reads
//! scanned PDFs. Each runs on the uploader's model, or the files model an admin picked when theirs
//! can't open that kind of file, and counts toward the uploader's usage. What it reads becomes
//! the attachment's `extracted_text`, which agents see in later turns (see
//! nomi_agent_core::attachments).

use std::time::Duration;

use base64::Engine;
use serde_json::{json, Value};
use sqlx::PgPool;
use uuid::Uuid;

use nomi_attachments::Kind;
use nomi_llm::{ContentBlock, LlmMessage, LlmRequest, LlmRole};

use crate::bootstrap::{build_llm_provider_for_user, can_open_for_user};

const INTERVAL: Duration = Duration::from_secs(3);
/// Files read per pass.
const PER_PASS: i64 = 4;
/// Tries before a file is marked unreadable.
const MAX_ATTEMPTS: i32 = 3;
/// One file's model call: room for a long video's upload and transcript.
const TIME_LIMIT: Duration = Duration::from_secs(300);
const MAX_TOKENS: u32 = 8192;

pub async fn run(pool: PgPool, settings_key: [u8; 32], http_client: reqwest::Client) {
    tracing::info!("attachment worker: started");
    loop {
        if let Err(e) = pass(&pool, &settings_key, &http_client).await {
            tracing::error!(error = %e, "attachment worker: pass failed");
        }
        tokio::time::sleep(INTERVAL).await;
    }
}

#[derive(sqlx::FromRow)]
struct Claimed {
    id: Uuid,
    user_id: Uuid,
    name: String,
    kind: String,
    mime: String,
    storage_key: String,
    preview_key: Option<String>,
    extracted_text: Option<String>,
    details: Value,
}

/// Claims and reads the waiting files. A file being read for over ten minutes was dropped by a
/// worker that stopped, so it's picked up again.
pub async fn pass(pool: &PgPool, settings_key: &[u8; 32], http_client: &reqwest::Client) -> Result<(), sqlx::Error> {
    remove_abandoned(pool).await?;

    sqlx::query(
        "UPDATE attachments SET status = 'failed', details = details || '{\"error\": \"it couldn''t be read after several tries\"}'::jsonb, updated_at = now() \
         WHERE status IN ('pending', 'processing') AND attempts >= $1 AND (status = 'pending' OR updated_at < now() - interval '10 minutes')",
    )
    .bind(MAX_ATTEMPTS)
    .execute(pool)
    .await?;

    let claimed: Vec<Claimed> = sqlx::query_as(
        "UPDATE attachments SET status = 'processing', attempts = attempts + 1, updated_at = now() \
         WHERE id IN ( \
             SELECT id FROM attachments \
             WHERE attempts < $1 AND (status = 'pending' OR (status = 'processing' AND updated_at < now() - interval '10 minutes')) \
             ORDER BY created_at LIMIT $2 FOR UPDATE SKIP LOCKED \
         ) \
         RETURNING id, user_id, name, kind, mime, storage_key, preview_key, extracted_text, details",
    )
    .bind(MAX_ATTEMPTS)
    .bind(PER_PASS)
    .fetch_all(pool)
    .await?;

    for file in claimed {
        let id = file.id;
        match tokio::time::timeout(TIME_LIMIT, read(pool, settings_key, http_client, &file)).await {
            Ok(Ok(Some(text))) => finish(pool, id, "ready", Some(&text), json!({})).await,
            // Nothing a model here can open: say so, keeping whatever was read already.
            Ok(Ok(None)) => {
                let kind = Kind::parse(&file.kind).unwrap_or(Kind::Other);
                if file.extracted_text.as_deref().is_some_and(|t| !t.trim().is_empty()) {
                    finish(pool, id, "ready", None, json!({})).await;
                } else {
                    let error = format!(
                        "no model here can open {} files; an admin can pick a files model that can in Admin → Models",
                        kind.as_str()
                    );
                    finish(pool, id, "failed", None, json!({ "error": error })).await;
                }
            }
            // Retried on the next pass, up to MAX_ATTEMPTS.
            Ok(Err(e)) => {
                tracing::warn!(error = %e, attachment_id = %id, "attachment worker: reading a file failed");
                back_to_pending(pool, id).await;
            }
            Err(_) => {
                tracing::warn!(attachment_id = %id, "attachment worker: reading a file timed out");
                back_to_pending(pool, id).await;
            }
        }
    }
    Ok(())
}

/// Files in no message are removed once they're a day old, so they don't fill anyone's storage:
/// uploads never sent (the tab closed mid-draft), and the files of deleted chats (deleting a
/// message clears its files' `message_id`).
async fn remove_abandoned(pool: &PgPool) -> Result<(), sqlx::Error> {
    let abandoned: Vec<(Uuid, String, Option<String>)> = sqlx::query_as(
        "DELETE FROM attachments WHERE id IN ( \
             SELECT id FROM attachments WHERE message_id IS NULL AND created_at < now() - interval '1 day' LIMIT 50 \
         ) RETURNING id, storage_key, preview_key",
    )
    .fetch_all(pool)
    .await?;
    let store = nomi_storage::blob::attachment_store();
    for (id, key, preview) in abandoned {
        for key in std::iter::once(key).chain(preview) {
            if let Err(e) = store.delete(&key).await {
                tracing::warn!(error = %e, attachment_id = %id, "attachment worker: failed to remove an unsent file");
            }
        }
    }
    Ok(())
}

async fn finish(pool: &PgPool, id: Uuid, status: &str, text: Option<&str>, details: Value) {
    let result = sqlx::query(
        "UPDATE attachments SET status = $2, extracted_text = COALESCE($3, extracted_text), details = details || $4, updated_at = now() WHERE id = $1",
    )
    .bind(id)
    .bind(status)
    .bind(text)
    .bind(details)
    .execute(pool)
    .await;
    if let Err(e) = result {
        tracing::error!(error = %e, attachment_id = %id, "attachment worker: failed to save what was read");
    }
}

async fn back_to_pending(pool: &PgPool, id: Uuid) {
    let _ = sqlx::query("UPDATE attachments SET status = 'pending', updated_at = now() WHERE id = $1 AND status = 'processing'")
        .bind(id)
        .execute(pool)
        .await;
}

/// What to ask the model for each kind of file.
pub fn instructions(kind: Kind) -> &'static str {
    match kind {
        Kind::Image => {
            "Describe this image for an assistant that can't see it and will act on it later. Say what it is \
             (a receipt, a screenshot, a photo of a document, a chart, a photo) and what it shows. Copy every \
             piece of text in it word for word: on a receipt the shop, date, each item with its price, taxes \
             and the total; on a screenshot the messages, buttons or error text; on a document its text. \
             Plain text, no preamble."
        }
        Kind::Voice => {
            "This is a voice note someone recorded for their assistant. Transcribe it word for word, in the \
             language it's spoken in. Output only the transcript."
        }
        Kind::Audio => {
            "Transcribe this recording word for word, in the language it's spoken in. If more than one \
             person speaks, start each turn with a label (Speaker 1:, Speaker 2:). Output only the transcript."
        }
        Kind::Video => {
            "Describe this video for an assistant that can't watch it. First a one-paragraph summary. Then \
             what happens, scene by scene, each with a rough timestamp (0:00). Copy any text shown on screen. \
             Then, under \"Transcript:\", everything that's said, word for word. Plain text, no preamble."
        }
        _ => {
            "Copy out all the text in this document, page by page, each page starting with \"## Page N\". \
             Keep tables as rows with cells separated by \" | \". Describe charts and pictures in a sentence. \
             Output only the document's content."
        }
    }
}

/// `Ok(None)` when no model here can open this kind of file.
async fn read(pool: &PgPool, settings_key: &[u8; 32], http_client: &reqwest::Client, file: &Claimed) -> Result<Option<String>, String> {
    let kind = Kind::parse(&file.kind).unwrap_or(Kind::Other);
    let (key, media_type) = match (&file.preview_key, kind) {
        (Some(preview), Kind::Image) => (preview.clone(), "image/jpeg".to_string()),
        _ => (file.storage_key.clone(), file.mime.clone()),
    };
    if !can_open_for_user(pool, file.user_id, settings_key, &media_type).await {
        return Ok(None);
    }
    let bytes = nomi_storage::blob::attachment_store()
        .get_bytes(&key)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "the file is missing from storage".to_string())?;

    let mut prompt = instructions(kind).to_string();
    if let Some(hint) = file.details.get("browser_transcript").and_then(Value::as_str) {
        prompt.push_str(&format!("\n\nThe browser's own rough transcript, for names and words that are hard to hear: {hint}"));
    }
    let request = LlmRequest {
        system: None,
        messages: vec![LlmMessage {
            role: LlmRole::User,
            content: vec![
                ContentBlock::Media { media_type, data: base64::engine::general_purpose::STANDARD.encode(bytes), name: file.name.clone() },
                ContentBlock::Text { text: prompt },
            ],
        }],
        tools: vec![],
        max_tokens: MAX_TOKENS,
        enable_reasoning: false,
        reasoning_effort: Default::default(),
    };
    let provider = build_llm_provider_for_user(pool, file.user_id, settings_key, http_client.clone()).await;
    let response = nomi_llm::complete(provider.as_ref(), request).await.map_err(|e| e.to_string())?;
    let text: String = response
        .content
        .into_iter()
        .filter_map(|b| match b {
            ContentBlock::Text { text } => Some(text),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("\n");
    let text = text.trim();
    if text.is_empty() {
        return Err("the model returned nothing".to_string());
    }
    Ok(Some(text.chars().take(nomi_attachments::extract::MAX_TEXT_CHARS).collect()))
}
