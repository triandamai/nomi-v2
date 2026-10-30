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
    /// What the link opens with (the reply, or the chat's first reply), masked when `mask` is on.
    pub preview: String,
    /// Who wrote it: the crew member's name, or none when it's the person's own message.
    pub preview_by: Option<String>,
    /// How many messages the whole chat would share.
    pub chat_messages: i64,
}

fn failed<E: std::fmt::Display>(error: E) -> (StatusCode, &'static str) {
    tracing::error!(error = %error, "chat share failed");
    (StatusCode::INTERNAL_SERVER_ERROR, "failed to share chat")
}

/// Shared messages: what the person and the crew said, without thinking steps or notices.
const SHARED_MESSAGES: &str = "SELECT m.id, m.sender_channel_identity_id IS NOT NULL, m.agent_display_name, m.content, m.created_at \
     FROM messages m WHERE m.session_id = $1 AND m.content NOT LIKE '🧠%' AND NOT m.is_notice AND m.content <> ''";

type SharedRow = (Uuid, bool, Option<String>, String, DateTime<Utc>);
/// A share as its owner sees it: id, token, chat, scope, reply, mask, anyone, expiry.
type ShareRow = (Uuid, String, Uuid, String, Option<Uuid>, bool, bool, Option<DateTime<Utc>>);
/// A share as its link reads it: chat, scope, reply, mask, anyone, expiry, the chat's title.
type LinkRow = (Uuid, String, Option<Uuid>, bool, bool, Option<DateTime<Utc>>, Option<String>);

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

fn checked(req: &ShareRequest) -> Result<Option<Duration>, (StatusCode, &'static str)> {
    if req.scope != "reply" && req.scope != "chat" {
        return Err((StatusCode::BAD_REQUEST, "scope must be reply or chat"));
    }
    if req.scope == "reply" && req.message_id.is_none() {
        return Err((StatusCode::BAD_REQUEST, "a reply's share needs its message_id"));
    }
    match req.expires_in_days {
        None => Ok(None),
        Some(days) if EXPIRY_DAYS.contains(&days) => Ok(Some(Duration::days(days))),
        Some(_) => Err((StatusCode::BAD_REQUEST, "expires_in_days must be 1, 7 or 30")),
    }
}

async fn respond(pool: &sqlx::PgPool, share_id: Uuid) -> Result<ShareResponse, (StatusCode, &'static str)> {
    let (id, token, session_id, scope, message_id, mask_on, anyone, expires_at): ShareRow =
        sqlx::query_as("SELECT id, token, session_id, scope, message_id, mask, anyone, expires_at FROM chat_shares WHERE id = $1")
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
) -> Result<(StatusCode, Json<ShareResponse>), (StatusCode, &'static str)> {
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
            return Err((StatusCode::NOT_FOUND, "message not found"));
        }
    }
    let token = Alphanumeric.sample_string(&mut rand::thread_rng(), TOKEN_LENGTH);
    let id: Uuid = sqlx::query_scalar(
        "INSERT INTO chat_shares (token, session_id, created_by, scope, message_id, mask, anyone, expires_at) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING id",
    )
    .bind(&token)
    .bind(session_id)
    .bind(claims.sub)
    .bind(&req.scope)
    .bind(req.message_id.filter(|_| req.scope == "reply"))
    .bind(req.mask)
    .bind(req.anyone)
    .bind(lasts.map(|duration| Utc::now() + duration))
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
) -> Result<Json<ShareResponse>, (StatusCode, &'static str)> {
    owned_session(&state.pool, claims.sub, session_id).await?;
    let lasts = checked(&req)?;
    let changed = sqlx::query(
        "UPDATE chat_shares SET scope = $3, message_id = COALESCE($4, message_id), mask = $5, anyone = $6, expires_at = $7 \
         WHERE id = $1 AND session_id = $2 AND revoked_at IS NULL",
    )
    .bind(share_id)
    .bind(session_id)
    .bind(&req.scope)
    .bind(req.message_id)
    .bind(req.mask)
    .bind(req.anyone)
    .bind(lasts.map(|duration| Utc::now() + duration))
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
        return Err((StatusCode::NOT_FOUND, "share not found"));
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
}

/// Whether the request carries a valid sign-in, for links only signed-in people may open.
fn signed_in(headers: &HeaderMap, secret: &str) -> bool {
    headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .is_some_and(|token| nomi_auth::claims::Claims::decode(token, secret).is_ok())
}

/// GET /api/shared/:token — what the link shows. Public when the owner said anyone may view; an
/// expired, revoked or unknown link is a 404, so links can't be probed.
pub async fn read(State(state): State<AppState>, headers: HeaderMap, Path(token): Path<String>) -> Result<Json<SharedChat>, (StatusCode, &'static str)> {
    let share: Option<LinkRow> = sqlx::query_as(
        "SELECT c.session_id, c.scope, c.message_id, c.mask, c.anyone, c.expires_at, s.title FROM chat_shares c JOIN sessions s ON s.id = c.session_id \
         WHERE c.token = $1 AND c.revoked_at IS NULL AND (c.expires_at IS NULL OR c.expires_at > now())",
    )
    .bind(&token)
    .fetch_optional(&state.pool)
    .await
    .map_err(failed)?;
    let (session_id, scope, message_id, mask_on, anyone, expires_at, title) = share.ok_or((StatusCode::NOT_FOUND, "link not found"))?;
    if !anyone && !signed_in(&headers, &state.jwt_secret) {
        return Err((StatusCode::UNAUTHORIZED, "sign in to open this link"));
    }
    let rows: Vec<SharedRow> = sqlx::query_as(&format!("{SHARED_MESSAGES} ORDER BY m.created_at")).bind(session_id).fetch_all(&state.pool).await.map_err(failed)?;
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
    Ok(Json(SharedChat { title: if mask_on { title.map(|t| mask(&t)) } else { title }, scope, masked: mask_on, messages, expires_at }))
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
    fn masking_leaves_plain_numbers_and_dates_alone() {
        assert_eq!(mask("3 visits on 2026-10-31, 12 steps"), "3 visits on 2026-10-31, 12 steps");
    }
}
