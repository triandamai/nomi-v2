//! Files in the conversation, as agents see them. A message points at each uploaded file with a
//! reference tag (nomi_attachments::reference); before a turn, [`expand_messages`] swaps each
//! tag for what Nomi read out of the file (a document's text, an image's description, a voice
//! note's transcript), and hands the model the newest message's images, audio and scanned PDFs
//! to look at itself. [`read_attachment`] lets an agent read further into a long file.
//!
//! Messages from before uploads carried a text file inline instead, as
//! `<attachment name="…" kind="file|voice">…</attachment>`; those are left as they are.

use std::time::Duration;

use base64::Engine;
use nomi_attachments::reference::{find_references, replace_references};
use nomi_attachments::{Kind, Reference};
use nomi_llm::{ContentBlock, LlmMessage, LlmRole, ToolDefinition};
use serde_json::{json, Value};
use sqlx::pool::PoolConnection;
use sqlx::Postgres;
use uuid::Uuid;

/// The agent every message with attachments is routed to (nomi-agent-files).
pub const FILES_AGENT_TYPE: &str = "files";
pub const READ_ATTACHMENT_TOOL_NAME: &str = "read_attachment";

/// A file's text shown with the newest message, and with older ones (agents read on with
/// `read_attachment`).
const LATEST_TEXT_CHARS: usize = 30_000;
const OLDER_TEXT_CHARS: usize = 2_000;
/// One `read_attachment` call's worth of text.
const READ_CHUNK_CHARS: usize = 20_000;
/// Files the model is handed to look at, all together. Bigger ones are described in text only.
const MAX_INLINE_MEDIA_BYTES: usize = 20 * 1024 * 1024;
/// How long a turn waits for a video to be watched in the background before going on without.
const VIDEO_WAIT: Duration = Duration::from_secs(120);

/// Whether a message carries files: uploaded ones, or an old message's inline text files.
pub fn has_attachments(text: &str) -> bool {
    !nomi_attachments::parse_references(text).is_empty() || (text.contains("<attachment name=\"") && text.contains("</attachment>"))
}

/// Whether the conversation (after [`expand_messages`]) holds uploaded files.
pub fn mentions_files(messages: &[LlmMessage]) -> bool {
    messages.iter().any(|m| m.content.iter().any(|b| matches!(b, ContentBlock::Text { text } if text.contains("<attachment id=\""))))
}

#[derive(Debug, Clone, sqlx::FromRow)]
struct StoredFile {
    id: Uuid,
    name: String,
    kind: String,
    mime: String,
    size_bytes: i64,
    status: String,
    extracted_text: Option<String>,
    details: Value,
    storage_key: String,
    preview_key: Option<String>,
}

impl StoredFile {
    fn kind(&self) -> Kind {
        Kind::parse(&self.kind).unwrap_or(Kind::Other)
    }

    fn scanned_pdf(&self) -> bool {
        self.kind() == Kind::Pdf && self.details.get("scanned").and_then(Value::as_bool).unwrap_or(false)
    }
}

async fn load_files(conn: &mut PoolConnection<Postgres>, user_id: Uuid, ids: &[Uuid]) -> Vec<StoredFile> {
    if ids.is_empty() {
        return Vec::new();
    }
    sqlx::query_as::<_, StoredFile>(
        "SELECT id, name, kind, mime, size_bytes, status, extracted_text, details, storage_key, preview_key \
         FROM attachments WHERE user_id = $1 AND id = ANY($2)",
    )
    .bind(user_id)
    .bind(ids)
    .fetch_all(&mut **conn)
    .await
    .unwrap_or_else(|e| {
        tracing::warn!(error = %e, "failed to load attachments for a turn");
        Vec::new()
    })
}

pub fn format_size(bytes: i64) -> String {
    let bytes = bytes.max(0) as f64;
    if bytes < 1024.0 {
        format!("{bytes} B")
    } else if bytes < 1024.0 * 1024.0 {
        format!("{:.0} KB", bytes / 1024.0)
    } else {
        format!("{:.1} MB", bytes / 1024.0 / 1024.0)
    }
}

/// "12 pages", "3 sheets", "1920×1080" — what kind of thing it is at a glance.
fn shape(file: &StoredFile) -> Option<String> {
    let d = &file.details;
    if let Some(pages) = d.get("pages").and_then(Value::as_u64).filter(|p| *p > 0) {
        return Some(format!("{pages} page{}", if pages == 1 { "" } else { "s" }));
    }
    if let Some(sheets) = d.get("sheets").and_then(Value::as_array) {
        let names: Vec<&str> = sheets.iter().filter_map(|s| s.get("name").and_then(Value::as_str)).collect();
        return Some(format!("sheets: {}", names.join(", ")));
    }
    if let Some(slides) = d.get("slides").and_then(Value::as_u64) {
        return Some(format!("{slides} slides"));
    }
    match (d.get("width").and_then(Value::as_u64), d.get("height").and_then(Value::as_u64)) {
        (Some(w), Some(h)) => Some(format!("{w}×{h}")),
        _ => None,
    }
}

fn excerpt(text: &str, max: usize) -> (String, Option<usize>) {
    match text.char_indices().nth(max) {
        Some((cut, _)) => (text[..cut].to_string(), Some(text[cut..].chars().count())),
        None => (text.to_string(), None),
    }
}

/// What an agent reads in place of a file's reference tag. `shown` says the model is also
/// handed the file itself in this message.
fn describe(file: &StoredFile, max_chars: usize, shown: bool) -> String {
    let mut out = format!(
        "<attachment id=\"{}\" name=\"{}\" kind=\"{}\" type=\"{}\" size=\"{}\">\n",
        file.id,
        file.name,
        file.kind,
        file.mime,
        format_size(file.size_bytes)
    );
    if let Some(shape) = shape(file) {
        out.push_str(&format!("({shape})\n"));
    }
    if shown {
        out.push_str("(The file itself is attached to this message: look at it.)\n");
    }
    let label = match file.kind() {
        Kind::Image => "What Nomi saw in it",
        Kind::Audio | Kind::Voice => "Transcript",
        Kind::Video => "What Nomi saw and heard in it",
        _ => "Content",
    };
    match (&file.extracted_text, file.status.as_str()) {
        (Some(text), _) if !text.trim().is_empty() => {
            let (shown_text, rest) = excerpt(text, max_chars);
            if file.status != "ready" && matches!(file.kind(), Kind::Voice) {
                out.push_str("Transcript (made by the browser while recording; may have mistakes):\n");
            } else {
                out.push_str(&format!("{label}:\n"));
            }
            out.push_str(&shown_text);
            if let Some(rest) = rest {
                let offset = shown_text.chars().count();
                out.push_str(&format!(
                    "\n… ({rest} more characters: call {READ_ATTACHMENT_TOOL_NAME} with id {} and offset {offset} to read on)",
                    file.id
                ));
            }
            out.push('\n');
        }
        (_, "pending" | "processing") if !shown => out.push_str("(Nomi is still reading this file.)\n"),
        (_, "failed") if !shown => {
            let why = file.details.get("error").and_then(Value::as_str).unwrap_or("it couldn't be read");
            out.push_str(&format!("(Nomi couldn't read this file: {why})\n"));
        }
        _ => {
            if let Some(why) = file.details.get("unreadable").and_then(Value::as_str) {
                out.push_str(&format!("({why})\n"));
            }
        }
    }
    out.push_str("</attachment>");
    out
}

/// What the model should look at itself from the newest message: images (as their preview),
/// audio not yet transcribed, scanned PDFs, a short video not yet watched.
fn wants_media(file: &StoredFile) -> bool {
    let has_text = file.extracted_text.as_deref().is_some_and(|t| !t.trim().is_empty());
    match file.kind() {
        Kind::Image => true,
        Kind::Audio | Kind::Voice => file.status != "ready" || !has_text,
        Kind::Pdf => file.scanned_pdf(),
        Kind::Video => file.status != "ready" || !has_text,
        _ => false,
    }
}

async fn media_block(file: &StoredFile, budget: &mut usize) -> Option<ContentBlock> {
    let store = nomi_storage::blob::attachment_store();
    let (key, media_type) = match (&file.preview_key, file.kind()) {
        (Some(preview), Kind::Image) => (preview.as_str(), "image/jpeg".to_string()),
        _ => (file.storage_key.as_str(), file.mime.clone()),
    };
    let bytes = store.get_bytes(key).await.ok().flatten()?;
    if bytes.len() > *budget {
        return None;
    }
    *budget -= bytes.len();
    Some(ContentBlock::Media { media_type, data: base64::engine::general_purpose::STANDARD.encode(bytes), name: file.name.clone() })
}

/// Waits (a while) for the background reader to finish the videos in `ids`.
async fn wait_for_videos(conn: &mut PoolConnection<Postgres>, user_id: Uuid, ids: &[Uuid]) {
    let deadline = tokio::time::Instant::now() + VIDEO_WAIT;
    while tokio::time::Instant::now() < deadline {
        let waiting: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM attachments WHERE user_id = $1 AND id = ANY($2) AND kind = 'video' AND status IN ('pending', 'processing')",
        )
        .bind(user_id)
        .bind(ids)
        .fetch_one(&mut **conn)
        .await
        .unwrap_or(0);
        if waiting == 0 {
            return;
        }
        tokio::time::sleep(Duration::from_secs(2)).await;
    }
}

/// Swaps every file reference in the user's messages for what was read out of the file, and
/// hands the model the newest message's images, audio and scanned PDFs. Only `user_id`'s own
/// files are ever opened.
pub async fn expand_messages(conn: &mut PoolConnection<Postgres>, user_id: Uuid, messages: &mut [LlmMessage]) {
    let referenced = |m: &LlmMessage| -> Vec<Reference> {
        m.content
            .iter()
            .filter_map(|b| match b {
                ContentBlock::Text { text } => Some(nomi_attachments::parse_references(text)),
                _ => None,
            })
            .flatten()
            .collect()
    };
    let Some(latest) = messages.iter().rposition(|m| m.role == LlmRole::User && !referenced(m).is_empty()) else {
        return;
    };

    let latest_ids: Vec<Uuid> = referenced(&messages[latest]).iter().map(|r| r.id).collect();
    let latest_videos: Vec<Uuid> = referenced(&messages[latest]).iter().filter(|r| r.kind == Kind::Video).map(|r| r.id).collect();
    if !latest_videos.is_empty() {
        wait_for_videos(conn, user_id, &latest_videos).await;
    }

    let all_ids: Vec<Uuid> = messages.iter().filter(|m| m.role == LlmRole::User).flat_map(referenced).map(|r| r.id).collect();
    let files = load_files(conn, user_id, &all_ids).await;

    let mut budget = MAX_INLINE_MEDIA_BYTES;
    for (index, message) in messages.iter_mut().enumerate() {
        if message.role != LlmRole::User {
            continue;
        }
        let is_latest = index == latest;
        let mut media = Vec::new();
        if is_latest {
            for id in &latest_ids {
                let Some(file) = files.iter().find(|f| f.id == *id) else { continue };
                if wants_media(file) {
                    if let Some(block) = media_block(file, &mut budget).await {
                        media.push((file.id, block));
                    }
                }
            }
        }
        for block in &mut message.content {
            let ContentBlock::Text { text } = block else { continue };
            if find_references(text).is_empty() {
                continue;
            }
            *text = replace_references(text, |r| match files.iter().find(|f| f.id == r.id) {
                Some(file) => {
                    let shown = media.iter().any(|(id, _)| *id == file.id);
                    describe(file, if is_latest { LATEST_TEXT_CHARS } else { OLDER_TEXT_CHARS }, shown)
                }
                None => format!("<attachment name=\"{}\">(This file is no longer available.)</attachment>", r.name),
            });
        }
        message.content.extend(media.into_iter().map(|(_, block)| block));
    }
}

pub fn read_attachment_tool_definition() -> ToolDefinition {
    ToolDefinition {
        name: READ_ATTACHMENT_TOOL_NAME.to_string(),
        description: "Read the text of a file the user attached (a document, spreadsheet, slides, PDF, or the \
                      transcript or description Nomi made of an image, audio or video). Use it when the file's \
                      text in the conversation was cut short, or to read a file from earlier in the chat. Read on \
                      from where the last part ended with `offset`."
            .to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "id": { "type": "string", "description": "The attachment id from its <attachment id=\"…\"> tag." },
                "offset": { "type": "integer", "minimum": 0, "description": "Character to start from (0 for the beginning)." }
            },
            "required": ["id"]
        }),
    }
}

/// `read_attachment`: one chunk of a file's text, from `offset`.
pub async fn read_attachment(conn: &mut PoolConnection<Postgres>, user_id: Uuid, input: &Value) -> Result<String, String> {
    let id: Uuid = input.get("id").and_then(Value::as_str).and_then(|s| s.trim().parse().ok()).ok_or("id must be an attachment id")?;
    let offset = input.get("offset").and_then(Value::as_u64).unwrap_or(0) as usize;
    let file = load_files(conn, user_id, &[id]).await.into_iter().next().ok_or("No attached file has that id.")?;
    let Some(text) = file.extracted_text.as_deref().filter(|t| !t.trim().is_empty()) else {
        return Ok(match file.status.as_str() {
            "pending" | "processing" => format!("{} is still being read. Try again in a moment.", file.name),
            _ => format!("There's no text to read in {} ({}).", file.name, file.mime),
        });
    };
    let total = text.chars().count();
    if offset >= total {
        return Ok(format!("{} has {total} characters; there's nothing after offset {offset}.", file.name));
    }
    let chunk: String = text.chars().skip(offset).take(READ_CHUNK_CHARS).collect();
    let end = offset + chunk.chars().count();
    let more = if end < total { format!("\n… ({} more characters; read on with offset {end})", total - end) } else { String::new() };
    Ok(format!("{} — characters {offset} to {end} of {total}:\n{chunk}{more}", file.name))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn file(kind: &str, status: &str, text: Option<&str>) -> StoredFile {
        StoredFile {
            id: Uuid::nil(),
            name: "plan.pdf".into(),
            kind: kind.into(),
            mime: "application/pdf".into(),
            size_bytes: 2_500_000,
            status: status.into(),
            extracted_text: text.map(str::to_string),
            details: json!({ "pages": 3 }),
            storage_key: "k".into(),
            preview_key: None,
        }
    }

    #[test]
    fn spots_uploaded_and_old_inline_files() {
        assert!(has_attachments("check this\n\n<attachment name=\"a.csv\">\nx\n</attachment>"));
        assert!(has_attachments("<attachment id=\"00000000-0000-0000-0000-000000000000\" name=\"a.png\" kind=\"image\" mime=\"image/png\" size=\"1\"/>"));
        assert!(!has_attachments("I mentioned <attachment name= in passing"));
    }

    #[test]
    fn describes_a_long_document_and_says_how_to_read_on() {
        let long = "word ".repeat(1000);
        let text = describe(&file("pdf", "ready", Some(&long)), 100, false);
        assert!(text.starts_with("<attachment id=\"00000000-0000-0000-0000-000000000000\" name=\"plan.pdf\" kind=\"pdf\" type=\"application/pdf\" size=\"2.4 MB\">"));
        assert!(text.contains("(3 pages)"));
        assert!(text.contains("call read_attachment with id 00000000-0000-0000-0000-000000000000 and offset 100"));
    }

    #[test]
    fn says_when_a_file_is_still_being_read_or_failed() {
        assert!(describe(&file("video", "pending", None), 100, false).contains("still reading"));
        let mut failed = file("video", "failed", None);
        failed.details = json!({ "error": "the files model can't watch video" });
        assert!(describe(&failed, 100, false).contains("can't watch video"));
    }

    #[test]
    fn hands_the_model_images_and_untranscribed_audio() {
        assert!(wants_media(&file("image", "ready", Some("a cat"))));
        assert!(wants_media(&file("voice", "pending", Some("browser transcript"))));
        assert!(!wants_media(&file("voice", "ready", Some("transcript"))));
        assert!(!wants_media(&file("pdf", "ready", Some("text"))));
    }
}
