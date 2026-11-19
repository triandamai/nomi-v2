//! Files the crew made in a chat (`make_file`, see nomi_agent_core::made_files): the file itself,
//! a preview of its first rows, deleting it, and having it made again. They are attachment rows,
//! so the owner checks are the attachments' own: only the person the file belongs to, through the
//! chat it was made in.

use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::Response;
use axum::Json;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

use nomi_attachments::made::{rows_preview, DEFAULT_PREVIEW_ROWS};
use nomi_auth::extractor::AuthClaims;

use crate::app::AppState;

type ApiError = (StatusCode, &'static str);

/// Lines of a text file a preview shows, when it has no rows to show.
const PREVIEW_TEXT_LINES: usize = 40;

#[derive(sqlx::FromRow)]
struct MadeRow {
    name: String,
    mime: String,
    storage_key: String,
    made_summary: Option<String>,
    made_from: Vec<Uuid>,
    deleted_at: Option<DateTime<Utc>>,
}

/// The made file `file_id` in chat `session_id`, if it is the caller's.
async fn find_made(state: &AppState, user_id: Uuid, session_id: Uuid, file_id: Uuid) -> Result<MadeRow, ApiError> {
    crate::routes::sessions::authorize_session_access(&state.pool, user_id, session_id).await?;
    sqlx::query_as::<_, MadeRow>(
        "SELECT name, mime, storage_key, made_summary, made_from, deleted_at FROM attachments \
         WHERE id = $1 AND user_id = $2 AND session_id = $3 AND made_by IS NOT NULL",
    )
    .bind(file_id)
    .bind(user_id)
    .bind(session_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to load file"))?
    .ok_or((StatusCode::NOT_FOUND, "file not found"))
}

/// A file that is still there: a deleted one is gone (410), like its bytes.
fn present(file: MadeRow) -> Result<MadeRow, ApiError> {
    if file.deleted_at.is_some() {
        return Err((StatusCode::GONE, "file was deleted"));
    }
    Ok(file)
}

/// `GET /api/sessions/:id/files/:file_id/content`: the file itself, to its owner only.
pub async fn content(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path((session_id, file_id)): Path<(Uuid, Uuid)>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let file = present(find_made(&state, claims.sub, session_id, file_id).await?)?;
    crate::routes::attachments::stored_file_response(&file.name, &file.mime, &file.storage_key, &headers).await
}

#[derive(Deserialize)]
pub struct PreviewQuery {
    #[serde(default)]
    pub rows: Option<usize>,
}

/// `GET /api/sessions/:id/files/:file_id/preview?rows=20`: a CSV's or spreadsheet's first rows;
/// any other file's first lines as `text`.
pub async fn preview(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path((session_id, file_id)): Path<(Uuid, Uuid)>,
    Query(query): Query<PreviewQuery>,
) -> Result<Json<Value>, ApiError> {
    let file = present(find_made(&state, claims.sub, session_id, file_id).await?)?;
    let bytes = nomi_storage::blob::attachment_store()
        .get_bytes(&file.storage_key)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to load file"))?
        .ok_or((StatusCode::NOT_FOUND, "file not found"))?;
    let limit = query.rows.unwrap_or(DEFAULT_PREVIEW_ROWS);
    let body = match rows_preview(&bytes, &file.name, limit) {
        Some(rows) => json!({ "file_id": file_id, "name": file.name, "mime": file.mime, "summary": file.made_summary, "rows": rows.rows, "total_rows": rows.total_rows, "truncated": rows.truncated }),
        None => {
            let text = String::from_utf8_lossy(&bytes);
            let lines: Vec<&str> = text.lines().take(PREVIEW_TEXT_LINES).collect();
            json!({ "file_id": file_id, "name": file.name, "mime": file.mime, "summary": file.made_summary, "rows": null, "text": lines.join("\n"), "truncated": text.lines().count() > lines.len() })
        }
    };
    Ok(Json(body))
}

#[derive(Serialize, Debug, PartialEq)]
pub struct DeleteReport {
    pub file_id: Uuid,
    /// When it was deleted; kept (the first time wins) so a repeated delete changes nothing.
    pub deleted_at: DateTime<Utc>,
    /// Whether the files it was made from are still there; null when it was made from none.
    pub sources_exist: Option<bool>,
}

/// What a delete tells the app: the files it was made from (`declared`) against those still
/// there (`alive`).
pub fn delete_report(file_id: Uuid, deleted_at: DateTime<Utc>, declared: &[Uuid], alive: &[Uuid]) -> DeleteReport {
    let sources_exist = if declared.is_empty() { None } else { Some(declared.iter().all(|id| alive.contains(id))) };
    DeleteReport { file_id, deleted_at, sources_exist }
}

/// `DELETE /api/sessions/:id/files/:file_id`: removes the file's bytes and keeps its row with
/// `deleted_at`. Deleting again changes nothing.
pub async fn delete(State(state): State<AppState>, AuthClaims(claims): AuthClaims, Path((session_id, file_id)): Path<(Uuid, Uuid)>) -> Result<Json<DeleteReport>, ApiError> {
    let file = find_made(&state, claims.sub, session_id, file_id).await?;
    let deleted_at: DateTime<Utc> = sqlx::query_scalar("UPDATE attachments SET deleted_at = COALESCE(deleted_at, now()), updated_at = now() WHERE id = $1 RETURNING deleted_at")
        .bind(file_id)
        .fetch_one(&state.pool)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to delete file"))?;
    if file.deleted_at.is_none() {
        let _ = nomi_storage::blob::attachment_store().delete(&file.storage_key).await;
    }
    let alive: Vec<Uuid> = sqlx::query_scalar("SELECT id FROM attachments WHERE user_id = $1 AND id = ANY($2) AND deleted_at IS NULL")
        .bind(claims.sub)
        .bind(&file.made_from)
        .fetch_all(&state.pool)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to look up its sources"))?;
    Ok(Json(delete_report(file_id, deleted_at, &file.made_from, &alive)))
}

#[derive(Serialize, Debug, PartialEq)]
pub struct RemakeResponse {
    /// What was asked of the crew, now in the chat as the person's message.
    pub user_message_id: Uuid,
    pub turn_job_id: Uuid,
}

/// `POST /api/sessions/:id/files/:file_id/remake` -> 202: asks the crew, in the chat and through
/// the same ingest and turn queue as the person's own message, to make the file again.
pub async fn remake(State(state): State<AppState>, AuthClaims(claims): AuthClaims, Path((session_id, file_id)): Path<(Uuid, Uuid)>) -> Result<(StatusCode, Json<RemakeResponse>), ApiError> {
    crate::routes::sessions::refuse_project_viewer(&state.pool, claims.sub, session_id).await?;
    let file = find_made(&state, claims.sub, session_id, file_id).await?;
    let (channel, chat_type, chat_id, chat_title): (String, String, String, Option<String>) = sqlx::query_as("SELECT channel, chat_type, chat_id, title FROM sessions WHERE id = $1")
        .bind(session_id)
        .fetch_one(&state.pool)
        .await
        .map_err(|_| (StatusCode::NOT_FOUND, "session not found"))?;
    nomi_turn::bootstrap::ensure_web_channel_identity(&state.pool, claims.sub).await.map_err(|e| {
        tracing::error!(error = %e, "failed to resolve web identity");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to make the file again")
    })?;
    let request = nomi_agent_core::made_files::remake_request(&file.name, file.made_summary.as_deref().unwrap_or(&file.name), file_id);
    let ingested = nomi_turn::ingest::ingest_inbound_message(&state.pool, &channel, &chat_type, &chat_id, &claims.sub.to_string(), &request, Some(claims.active_org_id))
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "failed to make the file again");
            (StatusCode::INTERNAL_SERVER_ERROR, "failed to make the file again")
        })?;
    crate::routes::sessions::announce_sent(&state.pool, claims.sub, session_id, chat_title, [ingested.user_message_id]).await;
    Ok((StatusCode::ACCEPTED, Json(RemakeResponse { user_message_id: ingested.user_message_id, turn_job_id: ingested.turn_job_id })))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_delete_keeps_its_time_and_says_whether_the_sources_are_still_there() {
        let (file, a, b) = (Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4());
        let when = Utc::now();
        let all_there = delete_report(file, when, &[a, b], &[b, a]);
        assert_eq!(all_there, DeleteReport { file_id: file, deleted_at: when, sources_exist: Some(true) });
        assert_eq!(delete_report(file, when, &[a, b], &[a]).sources_exist, Some(false));
        assert_eq!(delete_report(file, when, &[], &[a]).sources_exist, None);
        let body = serde_json::to_value(&all_there).unwrap();
        assert!(body["deleted_at"].is_string() && body["sources_exist"] == true);
    }
}
