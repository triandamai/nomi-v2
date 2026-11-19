//! Files the Files agent (Maya) makes in a chat: the `make_file` tool. The file is stored in the
//! same blob store and table as an attachment, announced on the chat's stream as it is made
//! (`file_generating`, then `file_ready` or `file_failed`), and from then on the person opens,
//! previews, saves or deletes it through `/api/sessions/:id/files`.

use chrono::Utc;
use nomi_attachments::made::{csv_bytes, storage_key, xlsx_bytes};
use nomi_llm::ToolDefinition;
use nomi_realtime::{FileFailCode, MqttPublisher, StreamEnvelope};
use nomi_settings::config;
use serde_json::{json, Value};
use sqlx::pool::PoolConnection;
use sqlx::Postgres;
use uuid::Uuid;

use crate::prompts;

pub const MAKE_FILE_TOOL_NAME: &str = "make_file";

/// What each person may keep stored, unless Admin → Configuration (or ATTACHMENT_QUOTA_MB) says.
pub const DEFAULT_QUOTA_MB: u64 = 2048;
const MB: u64 = 1024 * 1024;
/// The longest a made file's name may be, in characters.
const MAX_NAME_CHARS: usize = 120;
/// The longest the summary shown with a file may be, in characters.
const MAX_SUMMARY_CHARS: usize = 300;
/// The most rows one made table may have.
const MAX_ROWS: usize = 5_000;
/// The most characters one made text file may have.
const MAX_TEXT_CHARS: usize = 500_000;
/// The most source files one made file can name.
const MAX_SOURCES: usize = 20;
/// What the stream says the crew is doing while the file is written.
const STEP_WRITING: &str = "writing";

pub async fn per_person_quota_bytes() -> i64 {
    let mb = config::current(&config::ATTACHMENTS).await.number("quota_mb").unwrap_or(DEFAULT_QUOTA_MB);
    (mb * MB) as i64
}

pub fn make_file_tool_definition() -> ToolDefinition {
    ToolDefinition {
        name: MAKE_FILE_TOOL_NAME.to_string(),
        description: prompts::MAKE_FILE_TOOL_DESCRIPTION.to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "name": { "type": "string", "description": "The file's name with its extension, e.g. spending-may.csv." },
                "format": { "type": "string", "enum": ["csv", "xlsx", "txt", "md", "json"] },
                "rows": {
                    "type": "array",
                    "description": "For csv and xlsx: the table, header row first; each row is a list of cells.",
                    "items": { "type": "array", "items": {} }
                },
                "content": { "type": "string", "description": "For txt, md and json: the file's text." },
                "summary": { "type": "string", "description": "One line on what's in the file." },
                "sources": { "type": "array", "items": { "type": "string" }, "description": "Ids of the attachments it was made from." },
                "replaces": { "type": "string", "description": "The id of an earlier file this one is made again to replace." }
            },
            "required": ["name", "format", "summary"]
        }),
    }
}

/// A file the model asked for, checked and turned into bytes.
#[derive(Debug, PartialEq)]
pub struct MadeFile {
    pub name: String,
    pub mime: &'static str,
    /// The attachment kind it is stored as.
    pub kind: &'static str,
    pub bytes: Vec<u8>,
    /// The text kept for reading it back.
    pub text: String,
    /// Rows, for a table.
    pub rows: Option<u64>,
    pub summary: String,
    pub sources: Vec<Uuid>,
    pub replaces: Option<Uuid>,
}

fn cell_text(cell: &Value) -> String {
    match cell {
        Value::Null => String::new(),
        Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}

fn ids(input: &Value, field: &str) -> Vec<Uuid> {
    input.get(field).and_then(Value::as_array).map(|a| a.iter().filter_map(|v| v.as_str()?.trim().parse().ok()).collect()).unwrap_or_default()
}

/// Checks the model's input and builds the file's bytes.
pub fn build_file(input: &Value) -> Result<MadeFile, String> {
    let format = input.get("format").and_then(Value::as_str).unwrap_or("").trim().to_lowercase();
    let (mime, kind, extension) = match format.as_str() {
        "csv" => ("text/csv", "text", "csv"),
        "xlsx" => ("application/vnd.openxmlformats-officedocument.spreadsheetml.sheet", "spreadsheet", "xlsx"),
        "txt" => ("text/plain", "text", "txt"),
        "md" => ("text/markdown", "text", "md"),
        "json" => ("application/json", "text", "json"),
        _ => return Err("format must be csv, xlsx, txt, md or json".to_string()),
    };
    let raw_name = input.get("name").and_then(Value::as_str).unwrap_or("").trim();
    let stem: String = raw_name.rsplit(['/', '\\']).next().unwrap_or("").chars().filter(|c| !c.is_control()).collect();
    let stem = stem.trim().trim_end_matches('.').to_string();
    if stem.is_empty() {
        return Err("name is required".to_string());
    }
    let has_extension = stem.rsplit_once('.').is_some_and(|(_, ext)| ext.eq_ignore_ascii_case(extension));
    let name: String = if has_extension { stem } else { format!("{stem}.{extension}") };
    let name: String = name.chars().take(MAX_NAME_CHARS).collect();
    let summary: String = input.get("summary").and_then(Value::as_str).unwrap_or("").trim().chars().take(MAX_SUMMARY_CHARS).collect();
    if summary.is_empty() {
        return Err("summary is required: one line on what's in the file".to_string());
    }
    let mut sources = ids(input, "sources");
    sources.sort();
    sources.dedup();
    sources.truncate(MAX_SOURCES);
    let replaces = input.get("replaces").and_then(Value::as_str).and_then(|s| s.trim().parse().ok());

    let (bytes, text, rows) = if matches!(extension, "csv" | "xlsx") {
        let raw = input.get("rows").and_then(Value::as_array).ok_or("rows is required for a csv or xlsx")?;
        if raw.is_empty() {
            return Err("rows is empty".to_string());
        }
        if raw.len() > MAX_ROWS {
            return Err(format!("at most {MAX_ROWS} rows"));
        }
        let table: Vec<Vec<String>> = raw
            .iter()
            .map(|row| row.as_array().map(|cells| cells.iter().map(cell_text).collect()).ok_or("every row must be a list of cells"))
            .collect::<Result<_, _>>()?;
        let csv = csv_bytes(&table);
        let text = String::from_utf8_lossy(&csv).into_owned();
        let bytes = if extension == "xlsx" { xlsx_bytes(&table)? } else { csv };
        (bytes, text, Some(table.len() as u64))
    } else {
        let content = input.get("content").and_then(Value::as_str).ok_or("content is required for a text, Markdown or JSON file")?;
        if content.trim().is_empty() {
            return Err("content is empty".to_string());
        }
        if content.chars().count() > MAX_TEXT_CHARS {
            return Err(format!("at most {MAX_TEXT_CHARS} characters"));
        }
        if extension == "json" && serde_json::from_str::<Value>(content).is_err() {
            return Err("content isn't valid JSON".to_string());
        }
        (content.as_bytes().to_vec(), content.to_string(), None)
    };
    Ok(MadeFile { name, mime, kind, bytes, text, rows, summary, sources, replaces })
}

/// What the person's message says to have a made file made again.
pub fn remake_request(name: &str, summary: &str, file_id: Uuid) -> String {
    prompts::REMAKE_FILE_REQUEST.replace("{name}", name).replace("{summary}", summary).replace("{id}", &file_id.to_string())
}

async fn announce(mqtt: Option<&MqttPublisher>, session_id: Uuid, envelope: StreamEnvelope) {
    if let Some(publisher) = mqtt {
        if let Err(e) = publisher.publish(session_id, &envelope).await {
            tracing::debug!(error = %e, %session_id, "couldn't announce a made file");
        }
    }
}

/// `make_file`: builds, stores and records the file, announcing each stage on the chat's stream.
pub async fn make_file(
    conn: &mut PoolConnection<Postgres>,
    mqtt: Option<&MqttPublisher>,
    session_id: Uuid,
    user_id: Uuid,
    agent_type: &str,
    input: &Value,
) -> Result<String, String> {
    let file = build_file(input)?;
    let file_id = Uuid::new_v4();
    announce(
        mqtt,
        session_id,
        StreamEnvelope::FileGenerating { file_id, name: file.name.clone(), step: STEP_WRITING.to_string(), started_at: Utc::now().to_rfc3339() },
    )
    .await;
    match store_file(conn, user_id, session_id, agent_type, file_id, &file).await {
        Ok(()) => {
            announce(
                mqtt,
                session_id,
                StreamEnvelope::FileReady {
                    file_id,
                    name: file.name.clone(),
                    mime: file.mime.to_string(),
                    size: file.bytes.len() as u64,
                    rows: file.rows,
                    summary: file.summary.clone(),
                },
            )
            .await;
            Ok(format!("Made {} (attachment {file_id}) and showed it in the chat.", file.name))
        }
        Err(reason) => {
            announce(mqtt, session_id, StreamEnvelope::FileFailed { file_id, code: FileFailCode::WriteFailed }).await;
            Err(reason)
        }
    }
}

async fn store_file(conn: &mut PoolConnection<Postgres>, user_id: Uuid, session_id: Uuid, agent_type: &str, file_id: Uuid, file: &MadeFile) -> Result<(), String> {
    let used: i64 = sqlx::query_scalar("SELECT COALESCE(SUM(size_bytes), 0)::bigint FROM attachments WHERE user_id = $1 AND deleted_at IS NULL")
        .bind(user_id)
        .fetch_one(&mut **conn)
        .await
        .map_err(|e| format!("couldn't check storage: {e}"))?;
    if used + file.bytes.len() as i64 > per_person_quota_bytes().await {
        return Err("The person's file storage is full, so the file couldn't be saved.".to_string());
    }
    // Only their own files count as sources, or as the file this one replaces.
    let sources: Vec<Uuid> = sqlx::query_scalar("SELECT id FROM attachments WHERE user_id = $1 AND id = ANY($2)")
        .bind(user_id)
        .bind(&file.sources)
        .fetch_all(&mut **conn)
        .await
        .map_err(|e| format!("couldn't check the sources: {e}"))?;
    let replaces: Option<Uuid> = match file.replaces {
        Some(old) => sqlx::query_scalar("SELECT id FROM attachments WHERE user_id = $1 AND id = $2").bind(user_id).bind(old).fetch_optional(&mut **conn).await.map_err(|e| e.to_string())?,
        None => None,
    };
    let key = storage_key(user_id, file_id);
    nomi_storage::blob::attachment_store().put_bytes(&key, file.bytes.clone(), file.mime).await.map_err(|e| format!("couldn't store the file: {e}"))?;
    let details = json!({ "made": true, "rows": file.rows });
    sqlx::query(
        "INSERT INTO attachments (id, user_id, session_id, name, mime, kind, size_bytes, storage_key, status, extracted_text, details, replaces_attachment_id, made_by, made_summary, made_from) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 'ready', $9, $10, $11, $12, $13, $14)",
    )
    .bind(file_id)
    .bind(user_id)
    .bind(session_id)
    .bind(&file.name)
    .bind(file.mime)
    .bind(file.kind)
    .bind(file.bytes.len() as i64)
    .bind(&key)
    .bind(&file.text)
    .bind(&details)
    .bind(replaces)
    .bind(agent_type)
    .bind(&file.summary)
    .bind(&sources)
    .execute(&mut **conn)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "failed to record a made file");
        "couldn't record the file".to_string()
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_table_becomes_a_csv_named_with_its_extension() {
        let file = build_file(&json!({
            "name": "spending-may", "format": "csv", "summary": "May spending",
            "rows": [["item", "amount"], ["rice", 12500], ["tea", null]]
        }))
        .unwrap();
        assert_eq!(file.name, "spending-may.csv");
        assert_eq!(file.mime, "text/csv");
        assert_eq!(file.rows, Some(3));
        assert_eq!(String::from_utf8(file.bytes).unwrap(), "item,amount\r\nrice,12500\r\ntea,\r\n");
    }

    #[test]
    fn a_spreadsheet_keeps_its_name_and_reads_back() {
        let file = build_file(&json!({ "name": "a/b/Budget.XLSX", "format": "xlsx", "summary": "x", "rows": [["a"], ["b"]] })).unwrap();
        assert_eq!(file.name, "Budget.XLSX");
        assert_eq!(file.kind, "spreadsheet");
        let preview = nomi_attachments::made::rows_preview(&file.bytes, &file.name, 5).unwrap();
        assert_eq!(preview.total_rows, 2);
    }

    #[test]
    fn bad_requests_are_refused_with_a_reason() {
        assert!(build_file(&json!({ "name": "a", "format": "exe", "summary": "x" })).unwrap_err().contains("format"));
        assert!(build_file(&json!({ "name": "", "format": "txt", "summary": "x", "content": "hi" })).unwrap_err().contains("name"));
        assert!(build_file(&json!({ "name": "a", "format": "txt", "summary": "", "content": "hi" })).unwrap_err().contains("summary"));
        assert!(build_file(&json!({ "name": "a", "format": "csv", "summary": "x" })).unwrap_err().contains("rows"));
        assert!(build_file(&json!({ "name": "a", "format": "json", "summary": "x", "content": "{nope" })).unwrap_err().contains("JSON"));
    }

    #[test]
    fn sources_and_the_replaced_file_are_read_as_ids() {
        let a = Uuid::new_v4();
        let file = build_file(&json!({ "name": "n", "format": "md", "summary": "x", "content": "# hi", "sources": [a.to_string(), "junk", a.to_string()], "replaces": a.to_string() })).unwrap();
        assert_eq!(file.sources, vec![a]);
        assert_eq!(file.replaces, Some(a));
    }

    #[test]
    fn remaking_names_the_file_and_the_one_it_replaces() {
        let id = Uuid::nil();
        let text = remake_request("spend.csv", "May spending", id);
        assert!(text.contains("spend.csv") && text.contains("May spending") && text.contains(&id.to_string()));
        assert!(!text.contains('{'));
    }
}
