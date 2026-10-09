//! Files people attach in chat: upload (streamed to a temp file, checked, read, stored), the
//! file and its preview back to its owner, and removing one that was never sent. Messages point
//! at uploads with reference tags (nomi_attachments::reference); `check_message_references`
//! makes sure a message only points at the sender's own files, within the limits.

use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::{header, HeaderMap, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use futures_util::StreamExt;
use serde::Serialize;
use serde_json::json;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

use nomi_attachments::{Kind, Limits, Reference};
use nomi_auth::extractor::AuthClaims;

use crate::app::AppState;

type ApiError = (StatusCode, &'static str);

/// Bytes read before deciding what a file is.
const SNIFF_BYTES: usize = 8 * 1024;
/// The browser's live transcript of a voice note, kept as a fallback.
const MAX_TRANSCRIPT_CHARS: usize = 20_000;

#[derive(Serialize)]
pub struct AttachmentResponse {
    pub id: Uuid,
    pub name: String,
    pub kind: Kind,
    pub mime: String,
    pub size: u64,
    /// pending → ready, or failed when it couldn't be read.
    pub status: String,
    /// The tag a message carries to point at this file.
    pub reference: String,
}

fn per_person_quota_bytes() -> i64 {
    let mb = std::env::var("ATTACHMENT_QUOTA_MB").ok().and_then(|v| v.trim().parse::<i64>().ok()).filter(|v| *v > 0).unwrap_or(2048);
    mb * 1024 * 1024
}

/// `%`-decoding for the file name and transcript headers (headers carry ASCII only).
fn percent_decode(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(byte) = u8::from_str_radix(&value[i + 1..i + 3], 16) {
                out.push(byte);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn header_text(headers: &HeaderMap, name: &str) -> Option<String> {
    headers.get(name).and_then(|v| v.to_str().ok()).map(percent_decode)
}

fn clean_name(name: &str) -> String {
    let name: String = name.chars().filter(|c| !c.is_control()).collect();
    let name = name.rsplit(['/', '\\']).next().unwrap_or("").trim().to_string();
    let name: String = name.chars().take(200).collect();
    if name.is_empty() {
        "file".to_string()
    } else {
        name
    }
}

pub(crate) fn storage_key(user_id: Uuid, id: Uuid) -> String {
    format!("attachments/{user_id}/{id}")
}

fn preview_key(user_id: Uuid, id: Uuid) -> String {
    format!("attachments/{user_id}/{id}.preview.jpg")
}

/// Removes the temp file however the upload ends.
struct TempFile(std::path::PathBuf);

impl Drop for TempFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

/// `POST /api/attachments`: the raw file as the body, its name in `X-File-Name` (%-encoded),
/// `X-Attachment-Kind: voice` for a composer voice note, with the browser's own transcript (if it
/// made one) in `X-Transcript`.
pub async fn upload(State(state): State<AppState>, AuthClaims(claims): AuthClaims, headers: HeaderMap, body: Body) -> Result<(StatusCode, Json<AttachmentResponse>), ApiError> {
    let limits = Limits::from_env();
    let max = limits.max_upload_bytes();
    let declared: Option<u64> = headers.get(header::CONTENT_LENGTH).and_then(|v| v.to_str().ok()).and_then(|v| v.parse().ok());
    if declared.is_some_and(|len| len > max) {
        return Err((StatusCode::PAYLOAD_TOO_LARGE, "file is too big"));
    }
    let used: i64 = sqlx::query_scalar("SELECT COALESCE(SUM(size_bytes), 0)::bigint FROM attachments WHERE user_id = $1")
        .bind(claims.sub)
        .fetch_one(&state.pool)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to check storage"))?;
    if used + declared.unwrap_or(0) as i64 > per_person_quota_bytes() {
        return Err((StatusCode::INSUFFICIENT_STORAGE, "storage is full"));
    }

    let name = clean_name(&header_text(&headers, "x-file-name").unwrap_or_default());
    let claimed = headers.get(header::CONTENT_TYPE).and_then(|v| v.to_str().ok()).unwrap_or("").to_string();
    let voice = headers.get("x-attachment-kind").and_then(|v| v.to_str().ok()) == Some("voice");
    let transcript = header_text(&headers, "x-transcript")
        .map(|t| t.chars().take(MAX_TRANSCRIPT_CHARS).collect::<String>())
        .filter(|t| !t.trim().is_empty());

    // Stream to a temp file, counting as it goes, so a big upload never sits in memory.
    let dir = std::env::temp_dir().join("nomi-uploads");
    tokio::fs::create_dir_all(&dir).await.map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to store file"))?;
    let temp = TempFile(dir.join(Uuid::new_v4().to_string()));
    let mut file = tokio::fs::File::create(&temp.0).await.map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to store file"))?;
    let mut head = Vec::with_capacity(SNIFF_BYTES);
    let mut size: u64 = 0;
    let mut stream = body.into_data_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|_| (StatusCode::BAD_REQUEST, "upload was interrupted"))?;
        size += chunk.len() as u64;
        if size > max {
            return Err((StatusCode::PAYLOAD_TOO_LARGE, "file is too big"));
        }
        if head.len() < SNIFF_BYTES {
            let take = (SNIFF_BYTES - head.len()).min(chunk.len());
            head.extend_from_slice(&chunk[..take]);
        }
        file.write_all(&chunk).await.map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to store file"))?;
    }
    file.flush().await.map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to store file"))?;
    drop(file);

    let sniffed = nomi_attachments::sniff(&head, &name, &claimed, voice).map_err(|refused| match refused {
        nomi_attachments::kind::Refused::Executable => (StatusCode::UNSUPPORTED_MEDIA_TYPE, "programs can't be attached"),
        nomi_attachments::kind::Refused::Empty => (StatusCode::BAD_REQUEST, "file is empty"),
    })?;
    if size > limits.max_bytes_for(sniffed.kind) {
        return Err((StatusCode::PAYLOAD_TOO_LARGE, "file is too big"));
    }

    // Documents, spreadsheets, slides and PDFs are read now; media is left to the files model.
    let extracted = if matches!(sniffed.kind, Kind::Audio | Kind::Voice | Kind::Video) {
        nomi_attachments::Extracted { needs_model: true, details: json!({}), ..Default::default() }
    } else {
        let bytes = tokio::fs::read(&temp.0).await.map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to store file"))?;
        let sniffed = sniffed.clone();
        tokio::task::spawn_blocking(move || nomi_attachments::extract(&bytes, &sniffed))
            .await
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to read file"))?
    };

    let id = Uuid::new_v4();
    let store = nomi_storage::blob::attachment_store();
    let key = storage_key(claims.sub, id);
    store.put_file(&key, &temp.0, &sniffed.mime).await.map_err(|e| {
        tracing::error!(error = %e, "failed to store attachment");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to store file")
    })?;
    let preview = match extracted.preview_jpeg {
        Some(jpeg) => {
            let key = preview_key(claims.sub, id);
            match store.put_bytes(&key, jpeg, "image/jpeg").await {
                Ok(()) => Some(key),
                Err(e) => {
                    tracing::warn!(error = %e, "failed to store attachment preview");
                    None
                }
            }
        }
        None => None,
    };

    let mut details = extracted.details;
    if let Some(transcript) = &transcript {
        details["browser_transcript"] = json!(transcript);
    }
    let status = if extracted.needs_model { "pending" } else { "ready" };
    // A voice note starts with the browser's transcript until the files model's is in.
    let text = extracted.text.or(transcript);
    sqlx::query(
        "INSERT INTO attachments (id, user_id, name, mime, kind, size_bytes, storage_key, preview_key, status, extracted_text, details) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)",
    )
    .bind(id)
    .bind(claims.sub)
    .bind(&name)
    .bind(&sniffed.mime)
    .bind(sniffed.kind.as_str())
    .bind(size as i64)
    .bind(&key)
    .bind(&preview)
    .bind(status)
    .bind(&text)
    .bind(&details)
    .execute(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "failed to save attachment");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to store file")
    })?;

    let reference = Reference { id, name: name.clone(), kind: sniffed.kind, mime: sniffed.mime.clone(), size };
    Ok((
        StatusCode::CREATED,
        Json(AttachmentResponse {
            id,
            name,
            kind: sniffed.kind,
            mime: sniffed.mime,
            size,
            status: status.to_string(),
            reference: nomi_attachments::write_reference(&reference),
        }),
    ))
}

#[derive(sqlx::FromRow)]
struct StoredFile {
    name: String,
    mime: String,
    kind: String,
    size_bytes: i64,
    storage_key: String,
    preview_key: Option<String>,
    status: String,
    message_id: Option<Uuid>,
    details: serde_json::Value,
}

async fn find_own(state: &AppState, user_id: Uuid, id: Uuid) -> Result<StoredFile, ApiError> {
    sqlx::query_as::<_, StoredFile>(
        "SELECT name, mime, kind, size_bytes, storage_key, preview_key, status, message_id, details FROM attachments WHERE id = $1 AND user_id = $2",
    )
    .bind(id)
    .bind(user_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to load file"))?
    .ok_or((StatusCode::NOT_FOUND, "file not found"))
}

/// `GET /api/attachments/:id`: what Nomi knows about the file, for the chat's file chips.
pub async fn get_attachment(State(state): State<AppState>, AuthClaims(claims): AuthClaims, Path(id): Path<Uuid>) -> Result<Json<serde_json::Value>, ApiError> {
    let file = find_own(&state, claims.sub, id).await?;
    Ok(Json(json!({
        "id": id,
        "name": file.name,
        "kind": file.kind,
        "mime": file.mime,
        "size": file.size_bytes,
        "status": file.status,
        "has_preview": file.preview_key.is_some(),
        "details": file.details,
    })))
}

/// Types a browser may show in the page. Anything else (HTML, SVG, scripts) is downloaded, so
/// an attached file can never run in Nomi's origin.
fn shown_inline(mime: &str) -> bool {
    (mime.starts_with("image/") && mime != "image/svg+xml") || mime.starts_with("audio/") || mime.starts_with("video/") || mime == "application/pdf"
}

/// `bytes=start-end` → the range within `len`, for video seeking (Safari needs it to play).
fn byte_range(headers: &HeaderMap, len: usize) -> Option<(usize, usize)> {
    let spec = headers.get(header::RANGE)?.to_str().ok()?.strip_prefix("bytes=")?;
    let (start, end) = spec.split_once('-')?;
    let (start, end) = match (start.trim(), end.trim()) {
        ("", suffix) => {
            let n: usize = suffix.parse().ok()?;
            (len.saturating_sub(n), len.checked_sub(1)?)
        }
        (start, "") => (start.parse().ok()?, len.checked_sub(1)?),
        (start, end) => (start.parse().ok()?, end.parse::<usize>().ok()?.min(len.checked_sub(1)?)),
    };
    (start <= end && end < len).then_some((start, end))
}

fn content_disposition(name: &str, inline: bool) -> HeaderValue {
    let ascii: String = name.chars().map(|c| if c.is_ascii_graphic() && c != '"' && c != '\\' || c == ' ' { c } else { '_' }).collect();
    let encoded: String = name.bytes().map(|b| if b.is_ascii_alphanumeric() || b"-._~".contains(&b) { (b as char).to_string() } else { format!("%{b:02X}") }).collect();
    let disposition = if inline { "inline" } else { "attachment" };
    HeaderValue::from_str(&format!("{disposition}; filename=\"{ascii}\"; filename*=UTF-8''{encoded}")).unwrap_or(HeaderValue::from_static("attachment"))
}

/// `GET /api/attachments/:id/content`: the file itself, to its owner only.
pub async fn get_content(State(state): State<AppState>, AuthClaims(claims): AuthClaims, Path(id): Path<Uuid>, headers: HeaderMap) -> Result<Response, ApiError> {
    let file = find_own(&state, claims.sub, id).await?;
    let bytes = nomi_storage::blob::attachment_store()
        .get_bytes(&file.storage_key)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to load file"))?
        .ok_or((StatusCode::NOT_FOUND, "file not found"))?;

    let inline = shown_inline(&file.mime);
    let content_type = if inline { file.mime.as_str() } else { "application/octet-stream" };
    let mut response_headers = HeaderMap::new();
    response_headers.insert(header::CONTENT_TYPE, HeaderValue::from_str(content_type).unwrap_or(HeaderValue::from_static("application/octet-stream")));
    response_headers.insert(header::CONTENT_DISPOSITION, content_disposition(&file.name, inline));
    response_headers.insert(header::X_CONTENT_TYPE_OPTIONS, HeaderValue::from_static("nosniff"));
    response_headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("private, max-age=3600"));
    response_headers.insert(header::ACCEPT_RANGES, HeaderValue::from_static("bytes"));

    if let Some((start, end)) = byte_range(&headers, bytes.len()) {
        let range = HeaderValue::from_str(&format!("bytes {start}-{end}/{}", bytes.len())).expect("ascii");
        response_headers.insert(header::CONTENT_RANGE, range);
        let slice = bytes[start..=end].to_vec();
        return Ok((StatusCode::PARTIAL_CONTENT, response_headers, Body::from(slice)).into_response());
    }
    Ok((StatusCode::OK, response_headers, Body::from(bytes)).into_response())
}

/// `GET /api/attachments/:id/preview`: an image's small JPEG.
pub async fn get_preview(State(state): State<AppState>, AuthClaims(claims): AuthClaims, Path(id): Path<Uuid>) -> Result<Response, ApiError> {
    let file = find_own(&state, claims.sub, id).await?;
    let key = file.preview_key.ok_or((StatusCode::NOT_FOUND, "no preview"))?;
    let bytes = nomi_storage::blob::attachment_store()
        .get_bytes(&key)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to load preview"))?
        .ok_or((StatusCode::NOT_FOUND, "no preview"))?;
    Ok((
        [(header::CONTENT_TYPE, "image/jpeg"), (header::CACHE_CONTROL, "private, max-age=86400"), (header::X_CONTENT_TYPE_OPTIONS, "nosniff")],
        Body::from(bytes),
    )
        .into_response())
}

/// `DELETE /api/attachments/:id`: removes a file taken out of the composer before sending.
/// Files already in a message stay, as part of the chat.
pub async fn delete_attachment(State(state): State<AppState>, AuthClaims(claims): AuthClaims, Path(id): Path<Uuid>) -> Result<StatusCode, ApiError> {
    let file = find_own(&state, claims.sub, id).await?;
    if file.message_id.is_some() {
        return Err((StatusCode::CONFLICT, "file was already sent"));
    }
    sqlx::query("DELETE FROM attachments WHERE id = $1 AND user_id = $2 AND message_id IS NULL")
        .bind(id)
        .bind(claims.sub)
        .execute(&state.pool)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to remove file"))?;
    let store = nomi_storage::blob::attachment_store();
    let _ = store.delete(&file.storage_key).await;
    if let Some(key) = file.preview_key {
        let _ = store.delete(&key).await;
    }
    Ok(StatusCode::NO_CONTENT)
}

/// The message text with every file reference rewritten from the sender's own records, and the
/// files it points at. Refuses files that aren't theirs and messages over the limits.
pub async fn check_message_references(pool: &sqlx::PgPool, user_id: Uuid, text: &str) -> Result<(String, Vec<Uuid>), ApiError> {
    let references = nomi_attachments::parse_references(text);
    if references.is_empty() {
        return Ok((text.to_string(), Vec::new()));
    }
    let limits = Limits::from_env();
    let mut ids: Vec<Uuid> = references.iter().map(|r| r.id).collect();
    ids.sort();
    ids.dedup();
    if ids.len() > limits.max_files {
        return Err((StatusCode::BAD_REQUEST, "too many files in one message"));
    }
    let rows: Vec<(Uuid, String, String, String, i64)> =
        sqlx::query_as("SELECT id, name, kind, mime, size_bytes FROM attachments WHERE user_id = $1 AND id = ANY($2)")
            .bind(user_id)
            .bind(&ids)
            .fetch_all(pool)
            .await
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to check files"))?;
    if rows.len() != ids.len() {
        return Err((StatusCode::BAD_REQUEST, "unknown file"));
    }
    let total: i64 = rows.iter().map(|r| r.4).sum();
    if total as u64 > limits.max_message_bytes {
        return Err((StatusCode::PAYLOAD_TOO_LARGE, "files are too big together"));
    }
    let trusted = nomi_attachments::reference::replace_references(text, |r| {
        let row = rows.iter().find(|row| row.0 == r.id).expect("every id was found");
        nomi_attachments::write_reference(&Reference {
            id: row.0,
            name: row.1.clone(),
            kind: Kind::parse(&row.2).unwrap_or(Kind::Other),
            mime: row.3.clone(),
            size: row.4 as u64,
        })
    });
    Ok((trusted, ids))
}

/// Ties uploaded files to the message that carries them.
pub async fn link_to_message(pool: &sqlx::PgPool, user_id: Uuid, ids: &[Uuid], session_id: Uuid, message_id: Uuid) {
    if ids.is_empty() {
        return;
    }
    if let Err(e) = sqlx::query("UPDATE attachments SET message_id = $1, session_id = $2, updated_at = now() WHERE user_id = $3 AND id = ANY($4)")
        .bind(message_id)
        .bind(session_id)
        .bind(user_id)
        .bind(ids)
        .execute(pool)
        .await
    {
        tracing::warn!(error = %e, "failed to link attachments to their message");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_percent_encoded_names() {
        assert_eq!(percent_decode("Struk%20Mei%20%F0%9F%A7%BE.jpg"), "Struk Mei 🧾.jpg");
        assert_eq!(percent_decode("100%"), "100%");
        assert_eq!(clean_name("../../etc/passwd"), "passwd");
        assert_eq!(clean_name(""), "file");
    }

    #[test]
    fn reads_byte_ranges() {
        let mut headers = HeaderMap::new();
        headers.insert(header::RANGE, HeaderValue::from_static("bytes=0-1"));
        assert_eq!(byte_range(&headers, 10), Some((0, 1)));
        headers.insert(header::RANGE, HeaderValue::from_static("bytes=4-"));
        assert_eq!(byte_range(&headers, 10), Some((4, 9)));
        headers.insert(header::RANGE, HeaderValue::from_static("bytes=-3"));
        assert_eq!(byte_range(&headers, 10), Some((7, 9)));
        headers.insert(header::RANGE, HeaderValue::from_static("bytes=12-"));
        assert_eq!(byte_range(&headers, 10), None);
    }

    #[test]
    fn only_safe_types_open_in_the_page() {
        assert!(shown_inline("image/png"));
        assert!(shown_inline("video/mp4"));
        assert!(!shown_inline("image/svg+xml"));
        assert!(!shown_inline("text/html"));
    }
}
