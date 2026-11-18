//! Sharing one reply or a whole chat by link: read-only, optionally open to anyone (no sign-in),
//! with amounts, email addresses and phone numbers masked by default, and an expiry. The chat's
//! owner makes, changes and revokes the link; the link itself is a random token.

use std::sync::LazyLock;

use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::Json;
use chrono::{DateTime, Duration, Utc};
use rand::distributions::{Alphanumeric, DistString};
use regex::Regex;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::app::AppState;
use nomi_auth::extractor::AuthClaims;

/// Characters in a link's token: about 143 bits, so links can't be guessed.
const TOKEN_LENGTH: usize = 24;
/// How long a link lasts, in days, when the owner picks one (none: it never expires).
const EXPIRY_DAYS: [i64; 3] = [1, 7, 30];
/// How much of the shared reply the owner's preview shows.
const PREVIEW_CHARS: usize = 280;
/// What a masked value reads as.
const MASK: &str = "•••";
/// The error a request is refused with when it asks for masking and open files together.
pub const MASK_AND_FILES_OPEN: &str = "mask_and_files_open";

static AMOUNT: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)\b(Rp\.?|IDR|USD|SGD|EUR|GBP)\s?\d+(?:[.,]\d+)*|[$€£¥]\s?\d+(?:[.,]\d+)*").unwrap());
static CURRENCY: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)^(Rp\.?|IDR|USD|SGD|EUR|GBP|[$€£¥])").unwrap());
static EMAIL: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[\w.+-]+@[\w-]+(\.[\w-]+)+").unwrap());
/// A phone number starts with its country code (+62) or a trunk 0, so dates and counts stay.
static PHONE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(\+\d{1,3}|\b0)[\d \-()]{7,}\d").unwrap());

/// `text` with its amounts ("Rp •••"), email addresses and phone numbers hidden.
pub fn mask(text: &str) -> String {
    let text = AMOUNT.replace_all(text, |caps: &regex::Captures| {
        let currency = CURRENCY.find(&caps[0]).map(|m| m.as_str()).unwrap_or("");
        format!("{currency} {MASK}").trim_start().to_string()
    });
    let text = EMAIL.replace_all(&text, MASK);
    PHONE.replace_all(&text, MASK).into_owned()
}

#[derive(Deserialize)]
pub struct ShareRequest {
    /// "reply" or "chat".
    pub scope: String,
    /// The reply, when sharing one.
    #[serde(default)]
    pub message_id: Option<Uuid>,
    #[serde(default = "yes")]
    pub mask: bool,
    #[serde(default = "yes")]
    pub anyone: bool,
    /// 1, 7 or 30; none means the link never expires.
    #[serde(default)]
    pub expires_in_days: Option<i64>,
    /// Whether the link also opens the chat's files. Never with `mask`.
    #[serde(default)]
    pub files_open: bool,
}

fn yes() -> bool {
    true
}

#[derive(Serialize)]
pub struct ShareResponse {
    pub id: Uuid,
    pub token: String,
    pub scope: String,
    pub message_id: Option<Uuid>,
    pub mask: bool,
    pub anyone: bool,
    pub expires_at: Option<DateTime<Utc>>,
    pub files_open: bool,
    /// What the link opens with (the reply, or the chat's first reply), masked when `mask` is on.
    pub preview: String,
    /// Who wrote it: the crew member's name, or none when it's the person's own message.
    pub preview_by: Option<String>,
    /// How many messages the whole chat would share.
    pub chat_messages: i64,
}

/// Why a share request was refused: plain text like every other refusal here, or (for the files
/// switch, which the new app reads) a coded `{ "error": … }` body.
#[derive(Debug, PartialEq)]
pub enum ShareError {
    Plain(StatusCode, &'static str),
    Coded(StatusCode, &'static str),
}

impl From<(StatusCode, &'static str)> for ShareError {
    fn from((status, message): (StatusCode, &'static str)) -> Self {
        ShareError::Plain(status, message)
    }
}

impl axum::response::IntoResponse for ShareError {
    fn into_response(self) -> axum::response::Response {
        match self {
            ShareError::Plain(status, message) => (status, message).into_response(),
            ShareError::Coded(status, code) => (status, Json(serde_json::json!({ "error": code }))).into_response(),
        }
    }
}

fn failed<E: std::fmt::Display>(error: E) -> (StatusCode, &'static str) {
    tracing::error!(error = %error, "chat share failed");
    (StatusCode::INTERNAL_SERVER_ERROR, "failed to share chat")
}

/// Shared messages: what the person and the crew said, without thinking steps or notices.
const SHARED_MESSAGES: &str = "SELECT m.id, m.sender_channel_identity_id IS NOT NULL, m.agent_display_name, m.content, m.created_at \
     FROM messages m WHERE m.session_id = $1 AND m.content NOT LIKE '🧠%' AND NOT m.is_notice AND m.content <> ''";

type SharedRow = (Uuid, bool, Option<String>, String, DateTime<Utc>);
/// A share as its owner sees it: id, token, chat, scope, reply, mask, anyone, expiry, files open.
type ShareRow = (Uuid, String, Uuid, String, Option<Uuid>, bool, bool, Option<DateTime<Utc>>, bool);

/// Only the chat's owner shares it: a 404 for anyone else, as for a chat they can't see.
async fn owned_session(pool: &sqlx::PgPool, user_id: Uuid, session_id: Uuid) -> Result<(), (StatusCode, &'static str)> {
    let owned: bool = sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM sessions WHERE id = $1 AND user_id = $2)")
        .bind(session_id)
        .bind(user_id)
        .fetch_one(pool)
        .await
        .map_err(failed)?;
    if owned { Ok(()) } else { Err((StatusCode::NOT_FOUND, "session not found")) }
}

fn checked(req: &ShareRequest) -> Result<Option<Duration>, ShareError> {
    if req.mask && req.files_open {
        return Err(ShareError::Coded(StatusCode::BAD_REQUEST, MASK_AND_FILES_OPEN));
    }
    if req.scope != "reply" && req.scope != "chat" {
        return Err((StatusCode::BAD_REQUEST, "scope must be reply or chat").into());
    }
    if req.scope == "reply" && req.message_id.is_none() {
        return Err((StatusCode::BAD_REQUEST, "a reply's share needs its message_id").into());
    }
    match req.expires_in_days {
        None => Ok(None),
        Some(days) if EXPIRY_DAYS.contains(&days) => Ok(Some(Duration::days(days))),
        Some(_) => Err((StatusCode::BAD_REQUEST, "expires_in_days must be 1, 7 or 30").into()),
    }
}

async fn respond(pool: &sqlx::PgPool, share_id: Uuid) -> Result<ShareResponse, (StatusCode, &'static str)> {
    let (id, token, session_id, scope, message_id, mask_on, anyone, expires_at, files_open): ShareRow =
        sqlx::query_as("SELECT id, token, session_id, scope, message_id, mask, anyone, expires_at, files_open FROM chat_shares WHERE id = $1")
            .bind(share_id)
            .fetch_one(pool)
            .await
            .map_err(failed)?;
    let rows: Vec<SharedRow> = sqlx::query_as(&format!("{SHARED_MESSAGES} ORDER BY m.created_at")).bind(session_id).fetch_all(pool).await.map_err(failed)?;
    let shown = match message_id.filter(|_| scope == "reply") {
        Some(reply) => rows.iter().find(|row| row.0 == reply),
        None => rows.iter().find(|row| !row.1).or(rows.first()),
    };
    let excerpt: String = shown.map(|row| row.3.chars().take(PREVIEW_CHARS).collect()).unwrap_or_default();
    Ok(ShareResponse {
        id,
        token,
        scope,
        message_id,
        mask: mask_on,
        anyone,
        expires_at,
        files_open,
        preview: if mask_on { mask(&excerpt) } else { excerpt },
        preview_by: shown.filter(|row| !row.1).map(|row| row.2.clone().unwrap_or_else(|| "Nomi".to_string())),
        chat_messages: rows.len() as i64,
    })
}

/// POST /api/sessions/:id/shares — a new link.
pub async fn create(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(session_id): Path<Uuid>,
    Json(req): Json<ShareRequest>,
) -> Result<(StatusCode, Json<ShareResponse>), ShareError> {
    owned_session(&state.pool, claims.sub, session_id).await?;
    let lasts = checked(&req)?;
    if let Some(message_id) = req.message_id {
        let in_chat: bool = sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM messages WHERE id = $1 AND session_id = $2)")
            .bind(message_id)
            .bind(session_id)
            .fetch_one(&state.pool)
            .await
            .map_err(failed)?;
        if !in_chat {
            return Err((StatusCode::NOT_FOUND, "message not found").into());
        }
    }
    let token = Alphanumeric.sample_string(&mut rand::thread_rng(), TOKEN_LENGTH);
    let id: Uuid = sqlx::query_scalar(
        "INSERT INTO chat_shares (token, session_id, created_by, scope, message_id, mask, anyone, expires_at, files_open) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) RETURNING id",
    )
    .bind(&token)
    .bind(session_id)
    .bind(claims.sub)
    .bind(&req.scope)
    .bind(req.message_id.filter(|_| req.scope == "reply"))
    .bind(req.mask)
    .bind(req.anyone)
    .bind(lasts.map(|duration| Utc::now() + duration))
    .bind(req.files_open)
    .fetch_one(&state.pool)
    .await
    .map_err(failed)?;
    Ok((StatusCode::CREATED, Json(respond(&state.pool, id).await?)))
}

/// PUT /api/sessions/:id/shares/:share_id — the same link, with other settings.
pub async fn update(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path((session_id, share_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<ShareRequest>,
) -> Result<Json<ShareResponse>, ShareError> {
    owned_session(&state.pool, claims.sub, session_id).await?;
    let lasts = checked(&req)?;
    let changed = sqlx::query(
        "UPDATE chat_shares SET scope = $3, message_id = COALESCE($4, message_id), mask = $5, anyone = $6, expires_at = $7, files_open = $8 \
         WHERE id = $1 AND session_id = $2 AND revoked_at IS NULL",
    )
    .bind(share_id)
    .bind(session_id)
    .bind(&req.scope)
    .bind(req.message_id)
    .bind(req.mask)
    .bind(req.anyone)
    .bind(lasts.map(|duration| Utc::now() + duration))
    .bind(req.files_open)
    .execute(&state.pool)
    .await
    .map_err(|error| {
        // A reply's share switched to scope "reply" without a message to show.
        if error.as_database_error().is_some_and(|e| e.is_check_violation()) {
            (StatusCode::BAD_REQUEST, "a reply's share needs its message_id")
        } else {
            failed(error)
        }
    })?;
    if changed.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "share not found").into());
    }
    Ok(Json(respond(&state.pool, share_id).await?))
}

/// DELETE /api/sessions/:id/shares/:share_id — the link stops working.
pub async fn revoke(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path((session_id, share_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, (StatusCode, &'static str)> {
    owned_session(&state.pool, claims.sub, session_id).await?;
    let changed = sqlx::query("UPDATE chat_shares SET revoked_at = now() WHERE id = $1 AND session_id = $2 AND revoked_at IS NULL")
        .bind(share_id)
        .bind(session_id)
        .execute(&state.pool)
        .await
        .map_err(failed)?;
    if changed.rows_affected() == 0 { Err((StatusCode::NOT_FOUND, "share not found")) } else { Ok(StatusCode::NO_CONTENT) }
}

#[derive(Serialize, sqlx::FromRow)]
pub struct ShareListItem {
    pub id: Uuid,
    pub session_id: Uuid,
    /// The chat's title, if it has one yet.
    pub title: Option<String>,
    pub scope: String,
    pub message_id: Option<Uuid>,
    pub mask: bool,
    pub anyone: bool,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub files_open: bool,
}

#[derive(Serialize)]
pub struct ShareList {
    pub shares: Vec<ShareListItem>,
}

/// GET /api/shares — the person's links that still work (not revoked, not expired), newest
/// first: Settings › Data lists them, each with a way to revoke it.
pub async fn list(State(state): State<AppState>, AuthClaims(claims): AuthClaims) -> Result<Json<ShareList>, (StatusCode, &'static str)> {
    let shares: Vec<ShareListItem> = sqlx::query_as(
        "SELECT c.id, c.session_id, s.title, c.scope, c.message_id, c.mask, c.anyone, c.expires_at, c.created_at, c.files_open \
         FROM chat_shares c JOIN sessions s ON s.id = c.session_id \
         WHERE c.created_by = $1 AND c.revoked_at IS NULL AND (c.expires_at IS NULL OR c.expires_at > now()) \
         ORDER BY c.created_at DESC",
    )
    .bind(claims.sub)
    .fetch_all(&state.pool)
    .await
    .map_err(failed)?;
    Ok(Json(ShareList { shares }))
}

#[derive(Serialize)]
pub struct SharedMessage {
    /// "user" or "assistant".
    pub sender: String,
    pub agent_display_name: Option<String>,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct SharedChat {
    pub title: Option<String>,
    pub scope: String,
    pub masked: bool,
    pub messages: Vec<SharedMessage>,
    pub expires_at: Option<DateTime<Utc>>,
    /// The link opens the chat's files too.
    pub files_open: bool,
    /// The files in the shared messages; each has a `url` only when `files_open`.
    pub files: Vec<SharedFile>,
    /// Who shared it: their name, or its initials ("M. P.") when the link is masked.
    pub shared_by: String,
    /// When the link was made.
    pub shared_at: DateTime<Utc>,
    /// The signed-in viewer made this link: the page shows its owner's banner.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub own: bool,
    /// For the owner only: the link's id and its chat (to stop sharing: `DELETE
    /// /api/sessions/:session_id/shares/:share_id`), and how many others opened it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub share_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub views: Option<i64>,
}

/// One file in a shared message.
#[derive(Serialize, Debug, PartialEq)]
pub struct SharedFile {
    /// Where its message is in `messages`.
    pub message_index: usize,
    pub name: String,
    pub kind: String,
    pub size: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pages: Option<i64>,
    /// The crew made it (it came with a reply), rather than the person sending it.
    pub made_by_crew: bool,
    /// Where it opens, only when the link has its files open.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

/// A file attached to a message of the chat, as stored.
pub struct FileRow {
    pub id: Uuid,
    pub message_id: Uuid,
    pub name: String,
    pub kind: String,
    pub size_bytes: i64,
    pub details: serde_json::Value,
    /// Its message was written by the crew.
    pub from_crew: bool,
}

/// The messages a link shows, as `(id)` in the order they are shown: all of a chat's, or a
/// reply's one.
fn shown_message_ids(rows: &[SharedRow], scope: &str, message_id: Option<Uuid>) -> Vec<Uuid> {
    rows.iter().filter(|row| scope == "chat" || Some(row.0) == message_id).map(|row| row.0).collect()
}

/// The link's file list: only files of the messages it shows (`shown`, in order), each with an
/// `url` only when the link has its files open. A masked link's file names are masked too.
fn shared_files(rows: Vec<FileRow>, shown: &[Uuid], files_open: bool, token: &str, mask_on: bool) -> Vec<SharedFile> {
    rows.into_iter()
        .filter_map(|file| {
            let message_index = shown.iter().position(|id| *id == file.message_id)?;
            Some(SharedFile {
                message_index,
                name: if mask_on { mask(&file.name) } else { file.name },
                kind: file.kind,
                size: file.size_bytes,
                pages: file.details.get("pages").and_then(|pages| pages.as_i64()),
                made_by_crew: file.from_crew,
                url: files_open.then(|| format!("/api/shared/{token}/files/{}", file.id)),
            })
        })
        .collect()
}

/// Who the request is signed in as, if anyone.
fn signed_in_as(headers: &HeaderMap, secret: &str) -> Option<Uuid> {
    headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .and_then(|token| nomi_auth::claims::Claims::decode(token, secret).ok())
        .map(|claims| claims.sub)
}

/// A name as its initials: "Maya Putri" reads "M. P.".
pub fn initials(name: &str) -> String {
    name.split_whitespace().filter_map(|word| word.chars().next()).map(|first| format!("{}.", first.to_uppercase())).collect::<Vec<_>>().join(" ")
}

/// A share as its link reads it: id, owner, chat, scope, reply, mask, anyone, expiry, made, views,
/// the chat's title and the owner's name.
type LinkRow = (Uuid, Uuid, Uuid, String, Option<Uuid>, bool, bool, Option<DateTime<Utc>>, DateTime<Utc>, i64, Option<String>, String, bool);

/// The live link behind `token`: an expired, revoked or unknown one is a 404.
async fn live_link(pool: &sqlx::PgPool, token: &str) -> Result<LinkRow, (StatusCode, &'static str)> {
    let share: Option<LinkRow> = sqlx::query_as(
        "SELECT c.id, c.created_by, c.session_id, c.scope, c.message_id, c.mask, c.anyone, c.expires_at, c.created_at, c.views, s.title, \
                COALESCE(NULLIF(up.display_name, ''), split_part(COALESCE(w.email, g.email, ''), '@', 1)), c.files_open \
         FROM chat_shares c JOIN sessions s ON s.id = c.session_id \
         LEFT JOIN user_profiles up ON up.user_id = c.created_by \
         LEFT JOIN web_credentials w ON w.user_id = c.created_by \
         LEFT JOIN google_identities g ON g.user_id = c.created_by \
         WHERE c.token = $1 AND c.revoked_at IS NULL AND (c.expires_at IS NULL OR c.expires_at > now())",
    )
    .bind(token)
    .fetch_optional(pool)
    .await
    .map_err(failed)?;
    share.ok_or((StatusCode::NOT_FOUND, "link not found"))
}

/// GET /api/shared/:token — what the link shows. Public when the owner said anyone may view; an
/// expired, revoked or unknown link is a 404, so links can't be probed. Each open by someone other
/// than the owner counts as a view; the owner sees the count, and the link's id to stop sharing.
pub async fn read(State(state): State<AppState>, headers: HeaderMap, Path(token): Path<String>) -> Result<Json<SharedChat>, (StatusCode, &'static str)> {
    let (share_id, owner, session_id, scope, message_id, mask_on, anyone, expires_at, shared_at, views, title, owner_name, files_open) = live_link(&state.pool, &token).await?;
    let viewer = signed_in_as(&headers, &state.jwt_secret);
    if !anyone && viewer.is_none() {
        return Err((StatusCode::UNAUTHORIZED, "sign in to open this link"));
    }
    let own = viewer == Some(owner);
    if !own {
        sqlx::query("UPDATE chat_shares SET views = views + 1 WHERE id = $1").bind(share_id).execute(&state.pool).await.map_err(failed)?;
    }
    let rows: Vec<SharedRow> = sqlx::query_as(&format!("{SHARED_MESSAGES} ORDER BY m.created_at")).bind(session_id).fetch_all(&state.pool).await.map_err(failed)?;
    let shown = shown_message_ids(&rows, &scope, message_id);
    let files = shared_files(chat_files(&state.pool, session_id, &rows, &shown).await?, &shown, files_open, &token, mask_on);
    let messages = rows
        .into_iter()
        .filter(|row| scope == "chat" || Some(row.0) == message_id)
        .map(|(_, from_person, agent_display_name, content, created_at)| SharedMessage {
            sender: if from_person { "user" } else { "assistant" }.to_string(),
            agent_display_name,
            content: if mask_on { mask(&content) } else { content },
            created_at,
        })
        .collect();
    Ok(Json(SharedChat {
        title: if mask_on { title.map(|t| mask(&t)) } else { title },
        scope,
        masked: mask_on,
        messages,
        expires_at,
        files_open,
        files,
        shared_by: if mask_on { initials(&owner_name) } else { owner_name },
        shared_at,
        own,
        share_id: own.then_some(share_id),
        session_id: own.then_some(session_id),
        views: own.then_some(views),
    }))
}

/// A file the crew made in the chat: it belongs to the chat, not to one message.
type MadeRow = (Uuid, String, String, i64, serde_json::Value, DateTime<Utc>);

/// Sets each made file at the message it came with: the last one written at or before it (the
/// chat's first when none was). Only files that land on a shown message are kept, so a reply's
/// link lists just the files that reply brought.
fn place_made_files(made: Vec<MadeRow>, rows: &[SharedRow], shown: &[Uuid]) -> Vec<FileRow> {
    made.into_iter()
        .filter_map(|(id, name, kind, size_bytes, details, created_at)| {
            let nearest = rows.iter().rev().find(|row| row.4 <= created_at).or(rows.first())?;
            shown.contains(&nearest.0).then_some(FileRow { id, message_id: nearest.0, name, kind, size_bytes, details, from_crew: true })
        })
        .collect()
}

/// The files of the chat a link shows: those attached to `shown` messages, and those the crew made
/// in the chat (by session, as they carry no message), oldest first.
async fn chat_files(pool: &sqlx::PgPool, session_id: Uuid, rows: &[SharedRow], shown: &[Uuid]) -> Result<Vec<FileRow>, (StatusCode, &'static str)> {
    let attached: Vec<(Uuid, Uuid, String, String, i64, serde_json::Value, bool)> = sqlx::query_as(
        "SELECT a.id, a.message_id, a.name, a.kind, a.size_bytes, a.details, a.made_by IS NOT NULL \
         FROM attachments a JOIN messages m ON m.id = a.message_id \
         WHERE a.message_id = ANY($1) ORDER BY m.created_at, a.created_at, a.id",
    )
    .bind(shown)
    .fetch_all(pool)
    .await
    .map_err(failed)?;
    let made: Vec<MadeRow> = sqlx::query_as(
        "SELECT id, name, kind, size_bytes, details, created_at FROM attachments \
         WHERE session_id = $1 AND made_by IS NOT NULL AND message_id IS NULL AND deleted_at IS NULL ORDER BY created_at, id",
    )
    .bind(session_id)
    .fetch_all(pool)
    .await
    .map_err(failed)?;
    let mut files: Vec<FileRow> = attached.into_iter().map(|(id, message_id, name, kind, size_bytes, details, from_crew)| FileRow { id, message_id, name, kind, size_bytes, details, from_crew }).collect();
    files.extend(place_made_files(made, rows, shown));
    Ok(files)
}

/// GET /api/shared/:token/files/:file_id — one file of a shared message, for a link that has its
/// files open. The link's own token is what lets it through (the same rules as reading the link:
/// a live link, signed in unless `anyone`); anything else — a link with its files closed, a file
/// that isn't in a message the link shows — is a 404, so files can't be probed.
pub async fn open_file(State(state): State<AppState>, headers: HeaderMap, Path((token, file_id)): Path<(String, Uuid)>) -> Result<axum::response::Response, (StatusCode, &'static str)> {
    let (_, _, session_id, scope, message_id, _, anyone, _, _, _, _, _, files_open) = live_link(&state.pool, &token).await?;
    if !anyone && signed_in_as(&headers, &state.jwt_secret).is_none() {
        return Err((StatusCode::UNAUTHORIZED, "sign in to open this link"));
    }
    if !files_open {
        return Err((StatusCode::NOT_FOUND, "file not found"));
    }
    let rows: Vec<SharedRow> = sqlx::query_as(&format!("{SHARED_MESSAGES} ORDER BY m.created_at")).bind(session_id).fetch_all(&state.pool).await.map_err(failed)?;
    let shown = shown_message_ids(&rows, &scope, message_id);
    if !chat_files(&state.pool, session_id, &rows, &shown).await?.iter().any(|f| f.id == file_id) {
        return Err((StatusCode::NOT_FOUND, "file not found"));
    }
    let file: Option<(String, String, String)> = sqlx::query_as("SELECT name, mime, storage_key FROM attachments WHERE id = $1")
        .bind(file_id)
        .fetch_optional(&state.pool)
        .await
        .map_err(failed)?;
    let (name, mime, storage_key) = file.ok_or((StatusCode::NOT_FOUND, "file not found"))?;
    crate::routes::attachments::stored_file_response(&name, &mime, &storage_key, &headers).await
}

/// Why a viewer flags a shared page.
pub const REPORT_REASONS: [&str; 5] = ["spam", "abuse", "personal_info", "illegal", "other"];
/// The longest note a report keeps, in characters.
const REPORT_NOTE_CHARS: usize = 1000;
/// How many reports one viewer (person or address) may send in [`REPORT_WINDOW_MINUTES`].
const REPORTS_PER_WINDOW: i64 = 5;
/// The window reports are counted over.
const REPORT_WINDOW_MINUTES: i32 = 60;
/// The most reports the admin list returns.
const ADMIN_REPORTS_LIMIT: i64 = 200;

#[derive(Deserialize)]
pub struct ReportRequest {
    /// One of [`REPORT_REASONS`].
    pub reason: String,
    #[serde(default)]
    pub note: Option<String>,
}

/// Who a report came from, for rate-limiting: the signed-in person, or a hash of the address the
/// request came from (the first hop the proxy saw), so raw addresses aren't stored.
fn reporter(headers: &HeaderMap, viewer: Option<Uuid>) -> String {
    use sha2::{Digest, Sha256};
    if let Some(user_id) = viewer {
        return format!("user:{user_id}");
    }
    let address = ["x-forwarded-for", "x-real-ip"]
        .iter()
        .find_map(|name| headers.get(*name).and_then(|value| value.to_str().ok()))
        .and_then(|value| value.split(',').next())
        .map(str::trim)
        .unwrap_or("unknown");
    format!("addr:{}", hex::encode(Sha256::digest(address.as_bytes())))
}

/// POST /api/shared/:token/report — flag a shared page for an admin to look at. Open without a
/// sign-in (a link only for signed-in people still needs one); each viewer may send a few an hour.
pub async fn report(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(token): Path<String>,
    Json(req): Json<ReportRequest>,
) -> Result<StatusCode, (StatusCode, &'static str)> {
    if !REPORT_REASONS.contains(&req.reason.as_str()) {
        return Err((StatusCode::BAD_REQUEST, "reason must be spam, abuse, personal_info, illegal or other"));
    }
    let (share_id, _, _, _, _, _, anyone, ..) = live_link(&state.pool, &token).await?;
    let viewer = signed_in_as(&headers, &state.jwt_secret);
    if !anyone && viewer.is_none() {
        return Err((StatusCode::UNAUTHORIZED, "sign in to open this link"));
    }
    let who = reporter(&headers, viewer);
    let recent: i64 = sqlx::query_scalar("SELECT count(*) FROM shared_page_reports WHERE reporter = $1 AND created_at > now() - make_interval(mins => $2)")
        .bind(&who)
        .bind(REPORT_WINDOW_MINUTES)
        .fetch_one(&state.pool)
        .await
        .map_err(failed)?;
    if recent >= REPORTS_PER_WINDOW {
        return Err((StatusCode::TOO_MANY_REQUESTS, "too many reports; try again later"));
    }
    let note = req.note.map(|note| note.trim().chars().take(REPORT_NOTE_CHARS).collect::<String>()).filter(|note| !note.is_empty());
    sqlx::query("INSERT INTO shared_page_reports (share_id, reason, note, reporter) VALUES ($1, $2, $3, $4)")
        .bind(share_id)
        .bind(&req.reason)
        .bind(note)
        .bind(&who)
        .execute(&state.pool)
        .await
        .map_err(failed)?;
    Ok(StatusCode::CREATED)
}

#[derive(Serialize, sqlx::FromRow)]
pub struct SharedPageReport {
    pub id: Uuid,
    pub share_id: Uuid,
    pub token: String,
    pub session_id: Uuid,
    pub title: Option<String>,
    /// The link's owner.
    pub shared_by: Uuid,
    pub reason: String,
    pub note: Option<String>,
    /// The link was revoked since.
    pub revoked: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct SharedPageReports {
    pub reports: Vec<SharedPageReport>,
}

/// GET /api/admin/shared-reports — what viewers flagged, newest first (support, view).
pub async fn admin_list(State(state): State<AppState>, AuthClaims(claims): AuthClaims) -> Result<Json<SharedPageReports>, (StatusCode, &'static str)> {
    crate::routes::admin_access::require_admin(&claims, "support", "view")?;
    let reports: Vec<SharedPageReport> = sqlx::query_as(
        "SELECT r.id, r.share_id, c.token, c.session_id, s.title, c.created_by AS shared_by, r.reason, r.note, \
                c.revoked_at IS NOT NULL AS revoked, r.created_at \
         FROM shared_page_reports r JOIN chat_shares c ON c.id = r.share_id JOIN sessions s ON s.id = c.session_id \
         ORDER BY r.created_at DESC LIMIT $1",
    )
    .bind(ADMIN_REPORTS_LIMIT)
    .fetch_all(&state.pool)
    .await
    .map_err(failed)?;
    Ok(Json(SharedPageReports { reports }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn masking_hides_amounts_but_keeps_their_currency() {
        assert_eq!(mask("Food was Rp 1.250.000 and transport Rp400.000."), "Food was Rp ••• and transport Rp •••.");
        assert_eq!(mask("That's $12.50, or €9"), "That's $ •••, or € •••");
        assert_eq!(mask("Spent IDR 50,000 today"), "Spent IDR ••• today");
    }

    #[test]
    fn masking_hides_email_addresses_and_phone_numbers() {
        assert_eq!(mask("Mail sri@example.co.id or call +62 812-3456-7890"), "Mail ••• or call •••");
    }

    #[test]
    fn initials_keep_each_word_s_first_letter() {
        assert_eq!(initials("maya putri"), "M. P.");
        assert_eq!(initials("sri"), "S.");
        assert_eq!(initials("  "), "");
    }

    fn file(message_id: Uuid, name: &str, from_crew: bool) -> FileRow {
        FileRow { id: Uuid::new_v4(), message_id, name: name.to_string(), kind: "pdf".to_string(), size_bytes: 2048, details: serde_json::json!({ "pages": 12 }), from_crew }
    }

    fn request(mask: bool, files_open: bool) -> ShareRequest {
        ShareRequest { scope: "chat".to_string(), message_id: None, mask, anyone: true, expires_in_days: None, files_open }
    }

    #[test]
    fn files_open_defaults_to_false_and_is_refused_with_a_mask() {
        let old: ShareRequest = serde_json::from_value(serde_json::json!({ "scope": "chat" })).unwrap();
        assert!(!old.files_open);
        assert_eq!(checked(&request(true, true)).unwrap_err(), ShareError::Coded(StatusCode::BAD_REQUEST, "mask_and_files_open"));
        assert!(checked(&request(false, true)).is_ok());
        assert!(checked(&request(true, false)).is_ok());
    }

    #[test]
    fn a_coded_refusal_answers_with_its_error_code() {
        use axum::response::IntoResponse;
        let response = ShareError::Coded(StatusCode::BAD_REQUEST, MASK_AND_FILES_OPEN).into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn a_file_has_a_url_only_when_the_link_has_its_files_open() {
        let (first, second) = (Uuid::new_v4(), Uuid::new_v4());
        let rows = || vec![file(first, "report.pdf", false), file(second, "chart.png", true)];
        let closed = shared_files(rows(), &[first, second], false, "tok", false);
        assert!(closed.iter().all(|f| f.url.is_none()));
        assert_eq!(closed.iter().map(|f| (f.message_index, f.made_by_crew, f.pages)).collect::<Vec<_>>(), vec![(0, false, Some(12)), (1, true, Some(12))]);

        let open = shared_files(rows(), &[first, second], true, "tok", false);
        assert!(open.iter().all(|f| f.url.as_deref().is_some_and(|u| u.starts_with("/api/shared/tok/files/"))));
    }

    #[test]
    fn only_files_of_the_messages_the_link_shows_are_listed() {
        let (shown, hidden) = (Uuid::new_v4(), Uuid::new_v4());
        let files = shared_files(vec![file(shown, "a.pdf", false), file(hidden, "b.pdf", false)], &[shown], true, "tok", false);
        assert_eq!(files.iter().map(|f| f.name.as_str()).collect::<Vec<_>>(), vec!["a.pdf"]);
    }

    #[test]
    fn a_made_file_lands_on_the_message_before_it_and_a_reply_link_keeps_only_its_own() {
        let (ask, reply, later) = (Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4());
        let t0 = Utc::now();
        let at = |s: i64| t0 + chrono::Duration::seconds(s);
        let rows: Vec<SharedRow> = vec![(ask, true, None, "q".into(), at(0)), (reply, false, None, "a".into(), at(10)), (later, false, None, "b".into(), at(100))];
        let made = || vec![(Uuid::new_v4(), "rows.csv".to_string(), "csv".to_string(), 10, serde_json::json!({}), at(11))];
        let whole = place_made_files(made(), &rows, &[ask, reply, later]);
        assert_eq!(whole.len(), 1);
        assert_eq!(whole[0].message_id, reply);
        assert!(whole[0].from_crew);
        assert_eq!(place_made_files(made(), &rows, &[reply]).len(), 1);
        assert!(place_made_files(made(), &rows, &[later]).is_empty());
        assert!(place_made_files(made(), &[], &[]).is_empty());
    }

    #[test]
    fn a_replys_link_shows_only_that_reply() {
        let (a, b) = (Uuid::new_v4(), Uuid::new_v4());
        let now = Utc::now();
        let rows: Vec<SharedRow> = vec![(a, true, None, "q".into(), now), (b, false, None, "a".into(), now)];
        assert_eq!(shown_message_ids(&rows, "chat", None), vec![a, b]);
        assert_eq!(shown_message_ids(&rows, "reply", Some(b)), vec![b]);
    }

    #[test]
    fn a_masked_link_masks_file_names_and_a_file_without_pages_has_none() {
        let id = Uuid::new_v4();
        let mut row = file(id, "invoice for sri@example.com.pdf", false);
        row.details = serde_json::json!({});
        let files = shared_files(vec![row], &[id], false, "tok", true);
        assert_eq!(files[0].name, "invoice for •••");
        assert_eq!(files[0].pages, None);
    }

    #[test]
    fn masking_leaves_plain_numbers_and_dates_alone() {
        assert_eq!(mask("3 visits on 2026-10-31, 12 steps"), "3 visits on 2026-10-31, 12 steps");
    }
}
