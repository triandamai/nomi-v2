//! Resumable attachment uploads: `POST /api/attachments/uploads` opens one, `PATCH` sends a piece
//! with `Content-Range: bytes <from>-<to>/<total>`, `HEAD` says how far it got after a drop. The
//! pieces are staged on disk; the last one runs the same ingest as `POST /api/attachments`
//! (`attachments::ingest_file`) and answers the same `AttachmentResponse`.

use std::path::PathBuf;

use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::{header, HeaderMap, HeaderName, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use futures_util::StreamExt;
use serde_json::json;
use tokio::io::{AsyncSeekExt, AsyncWriteExt};
use uuid::Uuid;

use nomi_auth::extractor::AuthClaims;

use super::attachments::{
    check_quota, check_replaces, clean_name, coded, current_limits, header_text, ingest_file, replaces_header, server_error, too_large, AttachmentResponse,
    IngestInput, TempFile, UploadError, MAX_TRANSCRIPT_CHARS, SNIFF_BYTES,
};
use crate::app::AppState;

/// How long an upload may sit untouched before it's thrown away; every piece starts it again.
pub const UPLOAD_TTL_HOURS: i64 = 24;
/// The header that says where an upload stands (on every answer here), and how big it will be.
const OFFSET_HEADER: HeaderName = HeaderName::from_static("upload-offset");
const LENGTH_HEADER: HeaderName = HeaderName::from_static("upload-length");

/// Where the pieces of upload `id` are staged.
pub fn staged_path(id: Uuid) -> PathBuf {
    std::env::temp_dir().join("nomi-uploads").join("resumable").join(id.to_string())
}

/// `Content-Range: bytes <from>-<to>/<total>`, both ends inclusive.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContentRange {
    pub from: u64,
    pub to: u64,
    pub total: u64,
}

impl ContentRange {
    /// How many bytes the piece should hold.
    pub fn byte_len(&self) -> u64 {
        self.to - self.from + 1
    }
}

pub fn parse_content_range(value: &str) -> Option<ContentRange> {
    let spec = value.trim().strip_prefix("bytes ")?;
    let (span, total) = spec.split_once('/')?;
    let (from, to) = span.split_once('-')?;
    let (from, to, total): (u64, u64, u64) = (from.trim().parse().ok()?, to.trim().parse().ok()?, total.trim().parse().ok()?);
    (from <= to && to < total).then_some(ContentRange { from, to, total })
}

/// Why a piece was turned away.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChunkRefusal {
    /// The piece doesn't start where the upload stands; `current` is where it does.
    OffsetMismatch { current: u64 },
    /// The total differs from the one this upload began with.
    TotalChanged,
    /// The file is bigger than may be uploaded.
    TooLarge { max: u64 },
}

/// What accepting a piece means for the upload.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChunkPlan {
    pub total: u64,
    pub next_offset: u64,
    pub is_last: bool,
}

pub fn plan_chunk(offset: u64, known_total: Option<u64>, range: ContentRange, max: u64) -> Result<ChunkPlan, ChunkRefusal> {
    if range.total > max {
        return Err(ChunkRefusal::TooLarge { max });
    }
    if known_total.is_some_and(|total| total != range.total) {
        return Err(ChunkRefusal::TotalChanged);
    }
    if range.from != offset {
        return Err(ChunkRefusal::OffsetMismatch { current: offset });
    }
    let next_offset = range.to + 1;
    Ok(ChunkPlan { total: range.total, next_offset, is_last: next_offset == range.total })
}


#[derive(sqlx::FromRow)]
struct UploadRow {
    name: String,
    claimed_mime: String,
    voice: bool,
    transcript: Option<String>,
    replaces_attachment_id: Option<Uuid>,
    total_bytes: Option<i64>,
    received_bytes: i64,
}

const UPLOAD_COLUMNS: &str = "name, claimed_mime, voice, transcript, replaces_attachment_id, total_bytes, received_bytes";

fn not_found() -> UploadError {
    coded(StatusCode::NOT_FOUND, "upload_not_found", "that upload was not found, or it expired")
}

fn db_error(e: sqlx::Error) -> UploadError {
    tracing::error!(error = %e, "resumable upload: database error");
    server_error("failed to store file")
}

fn with_offset(mut response: Response, offset: u64, total: Option<u64>) -> Response {
    response.headers_mut().insert(OFFSET_HEADER, HeaderValue::from(offset));
    if let Some(total) = total {
        response.headers_mut().insert(LENGTH_HEADER, HeaderValue::from(total));
    }
    response.headers_mut().insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    response
}

/// `POST /api/attachments/uploads`: opens an upload. Takes the headers `POST /api/attachments`
/// does (`X-File-Name`, `Content-Type`, `X-Attachment-Kind`, `X-Transcript`, `X-Replaces`), and an
/// optional `X-Upload-Length` so a file that is too big is refused before any piece is sent.
pub async fn create(State(state): State<AppState>, AuthClaims(claims): AuthClaims, headers: HeaderMap) -> Result<Response, UploadError> {
    let limits = current_limits().await;
    let max = limits.max_upload_bytes();
    let declared: Option<u64> = headers.get("x-upload-length").and_then(|v| v.to_str().ok()).and_then(|v| v.trim().parse().ok());
    if let Some(total) = declared {
        if total > max {
            return Err(too_large(max));
        }
        check_quota(&state, claims.sub, total).await?;
    }
    let replaces = replaces_header(&headers).map_err(|_| coded(StatusCode::BAD_REQUEST, "invalid_replaces", "X-Replaces must be the id of a file"))?;
    check_replaces(&state, claims.sub, replaces).await?;

    let name = clean_name(&header_text(&headers, "x-file-name").unwrap_or_default());
    let claimed = headers.get(header::CONTENT_TYPE).and_then(|v| v.to_str().ok()).unwrap_or("").to_string();
    let voice = headers.get("x-attachment-kind").and_then(|v| v.to_str().ok()) == Some("voice");
    let transcript = header_text(&headers, "x-transcript")
        .map(|t| t.chars().take(MAX_TRANSCRIPT_CHARS).collect::<String>())
        .filter(|t| !t.trim().is_empty());

    let (id, expires_at): (Uuid, chrono::DateTime<chrono::Utc>) = sqlx::query_as(
        "INSERT INTO attachment_uploads (user_id, name, claimed_mime, voice, transcript, replaces_attachment_id, total_bytes, expires_at) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, now() + make_interval(hours => $8::int)) RETURNING id, expires_at",
    )
    .bind(claims.sub)
    .bind(&name)
    .bind(&claimed)
    .bind(voice)
    .bind(&transcript)
    .bind(replaces)
    .bind(declared.map(|t| t as i64))
    .bind(UPLOAD_TTL_HOURS as i32)
    .fetch_one(&state.pool)
    .await
    .map_err(db_error)?;
    let response = (StatusCode::CREATED, Json(json!({ "upload_id": id, "offset": 0, "expires_at": expires_at }))).into_response();
    Ok(with_offset(response, 0, declared))
}

/// `HEAD /api/attachments/uploads/:upload_id`: where the upload stands, in `Upload-Offset`.
pub async fn offset(State(state): State<AppState>, AuthClaims(claims): AuthClaims, Path(upload_id): Path<Uuid>) -> Result<Response, UploadError> {
    let row: Option<(i64, Option<i64>)> =
        sqlx::query_as("SELECT received_bytes, total_bytes FROM attachment_uploads WHERE id = $1 AND user_id = $2 AND expires_at > now()")
            .bind(upload_id)
            .bind(claims.sub)
            .fetch_optional(&state.pool)
            .await
            .map_err(db_error)?;
    let (received, total) = row.ok_or_else(not_found)?;
    Ok(with_offset(StatusCode::OK.into_response(), received as u64, total.map(|t| t as u64)))
}

async fn drop_upload(pool: &sqlx::PgPool, id: Uuid) {
    let _ = sqlx::query("DELETE FROM attachment_uploads WHERE id = $1").bind(id).execute(pool).await;
    let _ = tokio::fs::remove_file(staged_path(id)).await;
}

/// `PATCH /api/attachments/uploads/:upload_id`: one piece, `Content-Range: bytes <from>-<to>/<total>`.
/// Answers `{ offset }`, or for the last piece 201 and the usual `AttachmentResponse`. A piece that
/// doesn't start at the upload's offset is a 409 saying where it does.
pub async fn append(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(upload_id): Path<Uuid>,
    headers: HeaderMap,
    body: Body,
) -> Result<Response, UploadError> {
    let range = headers
        .get(header::CONTENT_RANGE)
        .and_then(|v| v.to_str().ok())
        .and_then(parse_content_range)
        .ok_or_else(|| coded(StatusCode::BAD_REQUEST, "invalid_range", "Content-Range must be bytes <from>-<to>/<total>"))?;

    // The row stays locked while the piece is written, so two pieces can't both land at one offset.
    let mut tx = state.pool.begin().await.map_err(db_error)?;
    let row: UploadRow = sqlx::query_as(&format!("SELECT {UPLOAD_COLUMNS} FROM attachment_uploads WHERE id = $1 AND user_id = $2 AND expires_at > now() FOR UPDATE"))
        .bind(upload_id)
        .bind(claims.sub)
        .fetch_optional(&mut *tx)
        .await
        .map_err(db_error)?
        .ok_or_else(not_found)?;
    let received = row.received_bytes as u64;

    let limits = current_limits().await;
    let plan = match plan_chunk(received, row.total_bytes.map(|t| t as u64), range, limits.max_upload_bytes()) {
        Ok(plan) => plan,
        Err(ChunkRefusal::OffsetMismatch { current }) => {
            let mut refusal = coded(StatusCode::CONFLICT, "offset_mismatch", "the piece doesn't start where the upload stands");
            refusal.1 .0["offset"] = json!(current);
            return Err(refusal);
        }
        Err(ChunkRefusal::TotalChanged) => return Err(coded(StatusCode::BAD_REQUEST, "total_changed", "the total size changed since the upload began")),
        Err(ChunkRefusal::TooLarge { max }) => {
            drop(tx);
            drop_upload(&state.pool, upload_id).await;
            return Err(too_large(max));
        }
    };
    if row.total_bytes.is_none() {
        if let Err(refusal) = check_quota(&state, claims.sub, plan.total).await {
            drop(tx);
            drop_upload(&state.pool, upload_id).await;
            return Err(refusal);
        }
    }

    // Append at the offset. Whatever an earlier dropped piece left past it is cut off first.
    let path = staged_path(upload_id);
    if let Some(dir) = path.parent() {
        tokio::fs::create_dir_all(dir).await.map_err(|_| server_error("failed to store file"))?;
    }
    let mut file = tokio::fs::OpenOptions::new().write(true).create(true).truncate(false).open(&path).await.map_err(|_| server_error("failed to store file"))?;
    let on_disk = file.metadata().await.map_err(|_| server_error("failed to store file"))?.len();
    if on_disk < received {
        drop(file);
        drop(tx);
        drop_upload(&state.pool, upload_id).await;
        return Err(coded(StatusCode::GONE, "interrupted", "the upload's earlier pieces were lost, start the upload again"));
    }
    file.set_len(received).await.map_err(|_| server_error("failed to store file"))?;
    file.seek(std::io::SeekFrom::Start(received)).await.map_err(|_| server_error("failed to store file"))?;

    let expected = range.byte_len();
    let mut written: u64 = 0;
    let mut stream = body.into_data_stream();
    let mut failure: Option<UploadError> = None;
    while let Some(piece) = stream.next().await {
        let piece = match piece {
            Ok(piece) => piece,
            Err(_) => {
                failure = Some(coded(StatusCode::BAD_REQUEST, "interrupted", "upload was interrupted"));
                break;
            }
        };
        written += piece.len() as u64;
        if written > expected {
            failure = Some(coded(StatusCode::BAD_REQUEST, "invalid_range", "the piece is longer than its Content-Range says"));
            break;
        }
        if file.write_all(&piece).await.is_err() {
            failure = Some(server_error("failed to store file"));
            break;
        }
    }
    if failure.is_none() && written != expected {
        failure = Some(coded(StatusCode::BAD_REQUEST, "interrupted", "upload was interrupted"));
    }
    if failure.is_none() && file.flush().await.is_err() {
        failure = Some(server_error("failed to store file"));
    }
    if let Some(failure) = failure {
        // Back to where the upload stood, so the same piece can be sent again.
        let _ = file.set_len(received).await;
        return Err(failure);
    }
    drop(file);

    sqlx::query("UPDATE attachment_uploads SET received_bytes = $2, total_bytes = $3, expires_at = now() + make_interval(hours => $4::int) WHERE id = $1")
        .bind(upload_id)
        .bind(plan.next_offset as i64)
        .bind(plan.total as i64)
        .bind(UPLOAD_TTL_HOURS as i32)
        .execute(&mut *tx)
        .await
        .map_err(db_error)?;
    tx.commit().await.map_err(db_error)?;

    if !plan.is_last {
        let response = Json(json!({ "offset": plan.next_offset })).into_response();
        return Ok(with_offset(response, plan.next_offset, Some(plan.total)));
    }

    // The last piece: the file is whole, so it goes where a single-request upload goes.
    let staged = TempFile(path);
    let result = finish(&state, claims.sub, &staged, plan.total, row, &limits).await;
    drop_upload(&state.pool, upload_id).await;
    let (status, Json(attachment)) = result?;
    Ok(with_offset((status, Json(attachment)).into_response(), plan.total, Some(plan.total)))
}

async fn finish(
    state: &AppState,
    user_id: Uuid,
    staged: &TempFile,
    size: u64,
    row: UploadRow,
    limits: &nomi_attachments::Limits,
) -> Result<(StatusCode, Json<AttachmentResponse>), UploadError> {
    use tokio::io::AsyncReadExt;
    let mut head = Vec::with_capacity(SNIFF_BYTES);
    let mut file = tokio::fs::File::open(&staged.0).await.map_err(|_| server_error("failed to read file"))?;
    (&mut file).take(SNIFF_BYTES as u64).read_to_end(&mut head).await.map_err(|_| server_error("failed to read file"))?;
    drop(file);
    let input = IngestInput {
        user_id,
        temp: &staged.0,
        size,
        head: &head,
        name: row.name,
        claimed: row.claimed_mime,
        voice: row.voice,
        transcript: row.transcript,
        replaces: row.replaces_attachment_id,
    };
    ingest_file(state, input, limits).await
}

/// Throws away uploads nobody came back to, and the pieces staged for them. Run by the attachment
/// worker's pass.
pub async fn remove_expired(pool: &sqlx::PgPool) -> Result<u64, sqlx::Error> {
    let expired: Vec<Uuid> = sqlx::query_scalar("DELETE FROM attachment_uploads WHERE expires_at < now() RETURNING id").fetch_all(pool).await?;
    for id in &expired {
        let _ = tokio::fs::remove_file(staged_path(*id)).await;
    }
    Ok(expired.len() as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_a_content_range() {
        assert_eq!(parse_content_range("bytes 0-99/1000"), Some(ContentRange { from: 0, to: 99, total: 1000 }));
        assert_eq!(parse_content_range(" bytes 100-199/1000 "), Some(ContentRange { from: 100, to: 199, total: 1000 }));
        assert_eq!(parse_content_range("bytes 0-0/1").map(|r| r.byte_len()), Some(1));
        for bad in ["", "bytes 0-99", "bytes 0-99/*", "bytes */1000", "items 0-9/10", "bytes 9-0/10", "bytes 0-10/10", "bytes a-b/c", "bytes 0-9/0"] {
            assert_eq!(parse_content_range(bad), None, "{bad}");
        }
    }

    fn range(from: u64, to: u64, total: u64) -> ContentRange {
        ContentRange { from, to, total }
    }

    #[test]
    fn a_piece_must_start_where_the_upload_stands() {
        assert_eq!(plan_chunk(0, None, range(0, 99, 300), 1000), Ok(ChunkPlan { total: 300, next_offset: 100, is_last: false }));
        assert_eq!(plan_chunk(100, Some(300), range(100, 199, 300), 1000), Ok(ChunkPlan { total: 300, next_offset: 200, is_last: false }));
        // Skipping ahead, or sending a piece again, says where the upload really is.
        assert_eq!(plan_chunk(100, Some(300), range(200, 299, 300), 1000), Err(ChunkRefusal::OffsetMismatch { current: 100 }));
        assert_eq!(plan_chunk(200, Some(300), range(100, 199, 300), 1000), Err(ChunkRefusal::OffsetMismatch { current: 200 }));
    }

    #[test]
    fn the_piece_reaching_the_total_is_the_last() {
        assert_eq!(plan_chunk(200, Some(300), range(200, 299, 300), 1000), Ok(ChunkPlan { total: 300, next_offset: 300, is_last: true }));
        assert_eq!(plan_chunk(0, None, range(0, 0, 1), 1000), Ok(ChunkPlan { total: 1, next_offset: 1, is_last: true }));
    }

    #[test]
    fn the_total_cannot_change_midway() {
        assert_eq!(plan_chunk(100, Some(300), range(100, 199, 400), 1000), Err(ChunkRefusal::TotalChanged));
    }

    #[test]
    fn the_size_limit_holds_across_pieces() {
        // A file over the limit is refused at its first piece, however small that piece is.
        assert_eq!(plan_chunk(0, None, range(0, 9, 30), 25), Err(ChunkRefusal::TooLarge { max: 25 }));
        // And a later piece can't smuggle it in: the total it names must still fit.
        assert_eq!(plan_chunk(10, Some(20), range(10, 19, 20), 25), Ok(ChunkPlan { total: 20, next_offset: 20, is_last: true }));
        assert_eq!(plan_chunk(10, Some(20), range(10, 29, 30), 25), Err(ChunkRefusal::TooLarge { max: 25 }));
        assert_eq!(plan_chunk(0, None, range(0, 24, 25), 25).map(|p| p.is_last), Ok(true));
    }
}
