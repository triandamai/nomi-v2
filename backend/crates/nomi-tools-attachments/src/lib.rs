//! Reading the files a person attached: the `read_attachment` tool, and the lookups core's
//! message expansion shares with it.

use nomi_attachments::Kind;
use nomi_llm::{ContentBlock, LlmMessage, ToolDefinition};
use serde_json::{json, Value};
use sqlx::pool::PoolConnection;
use sqlx::Postgres;
use uuid::Uuid;

mod tools;
pub use tools::tools;

pub const READ_ATTACHMENT_TOOL_NAME: &str = "read_attachment";
/// One `read_attachment` call's worth of text.
const READ_CHUNK_CHARS: usize = 20_000;

/// Whether a message carries files: uploaded ones, or an old message's inline text files.
pub fn has_attachments(text: &str) -> bool {
    !nomi_attachments::parse_references(text).is_empty() || (text.contains("<attachment name=\"") && text.contains("</attachment>"))
}

/// Whether the conversation (after [`expand_messages`]) holds uploaded files.
pub fn mentions_files(messages: &[LlmMessage]) -> bool {
    messages.iter().any(|m| m.content.iter().any(|b| matches!(b, ContentBlock::Text { text } if text.contains("<attachment id=\""))))
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct StoredFile {
    pub id: Uuid,
    pub name: String,
    pub kind: String,
    pub mime: String,
    pub size_bytes: i64,
    pub status: String,
    pub extracted_text: Option<String>,
    pub details: Value,
    pub storage_key: String,
    pub preview_key: Option<String>,
}

impl StoredFile {
    pub fn kind(&self) -> Kind {
        Kind::parse(&self.kind).unwrap_or(Kind::Other)
    }

    pub fn scanned_pdf(&self) -> bool {
        self.kind() == Kind::Pdf && self.details.get("scanned").and_then(Value::as_bool).unwrap_or(false)
    }
}

pub async fn load_files(conn: &mut PoolConnection<Postgres>, user_id: Uuid, ids: &[Uuid]) -> Vec<StoredFile> {
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

/// One file the person attached, of any kind (an image, a font, an SVG, a CSV…), whole, for
/// copying into a project: its name and its bytes. Only `user_id`'s own files are opened.
pub async fn attachment_bytes(conn: &mut PoolConnection<Postgres>, user_id: Uuid, id: Uuid) -> Result<(String, Vec<u8>), String> {
    let file = load_files(conn, user_id, &[id]).await.into_iter().next().ok_or("No attached file has that id.")?;
    let bytes = nomi_storage::blob::attachment_store()
        .get_bytes(&file.storage_key)
        .await
        .map_err(|e| format!("couldn't open {}: {e}", file.name))?
        .ok_or_else(|| format!("{} is no longer available.", file.name))?;
    Ok((file.name, bytes))
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

