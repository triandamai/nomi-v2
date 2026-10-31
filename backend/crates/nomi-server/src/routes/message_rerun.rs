//! Running a turn again: the person edits their message, or asks for the crew's reply again.
//!
//! Only the latest turn runs again (a later message would have been answered with the old one in
//! mind), and only when nothing is running in the chat. Before it does, the current take is kept
//! as a version of the person's message (what they wrote, the replies it got), and what the turn
//! made is taken back (the replies, transactions logged, reminders set, plans drafted, tasks
//! scheduled; see message_delete), so running it again doesn't do it twice.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::message_delete::{drop_blobs, related_to, remove};
use crate::app::AppState;
use nomi_auth::extractor::AuthClaims;

type ApiError = (StatusCode, &'static str);

fn internal(context: &'static str) -> impl Fn(sqlx::Error) -> ApiError {
    move |e| {
        tracing::error!(error = %e, "{context}");
        (StatusCode::INTERNAL_SERVER_ERROR, context)
    }
}

#[derive(Serialize)]
pub struct RerunResponse {
    /// The person's message whose turn runs again.
    pub message_id: Uuid,
    /// The take now running: the newest, numbered from 1.
    pub version: i64,
    pub total: i64,
    pub edited_at: Option<DateTime<Utc>>,
}

/// The person's message that started the turn `message_id` belongs to: itself, or for a reply,
/// the person's last message before it.
async fn turn_start(pool: &sqlx::PgPool, session_id: Uuid, message_id: Uuid) -> Result<Uuid, ApiError> {
    let start: Option<Uuid> = sqlx::query_scalar(
        "SELECT p.id FROM messages m JOIN messages p ON p.session_id = m.session_id AND p.sender_channel_identity_id IS NOT NULL \
           AND (p.id = m.id OR p.created_at < m.created_at) \
         WHERE m.id = $1 AND m.session_id = $2 ORDER BY (p.id = m.id) DESC, p.created_at DESC LIMIT 1",
    )
    .bind(message_id)
    .bind(session_id)
    .fetch_optional(pool)
    .await
    .map_err(internal("failed to look up message"))?;
    start.ok_or((StatusCode::NOT_FOUND, "message not found"))
}

/// How to answer again, from the regenerate menu.
#[derive(Deserialize, Clone, Copy, Default)]
#[serde(rename_all = "snake_case")]
pub enum Style {
    #[default]
    Again,
    Shorter,
    Longer,
    List,
}

impl Style {
    /// What's added after the person's message for the crew to read (never to the message itself).
    fn hint(self) -> Option<&'static str> {
        match self {
            Style::Again => None,
            Style::Shorter => Some(nomi_agent_core::prompts::REGENERATE_SHORTER),
            Style::Longer => Some(nomi_agent_core::prompts::REGENERATE_LONGER),
            Style::List => Some(nomi_agent_core::prompts::REGENERATE_LIST),
        }
    }
}

async fn rerun(state: &AppState, user_id: Uuid, session_id: Uuid, start: Uuid, new_text: Option<&str>, style: Style) -> Result<RerunResponse, ApiError> {
    let (old_text, sender, org_id, created_at): (String, Uuid, Uuid, DateTime<Utc>) = sqlx::query_as(
        "SELECT m.content, m.sender_channel_identity_id, s.org_id, m.created_at FROM messages m JOIN sessions s ON s.id = m.session_id WHERE m.id = $1",
    )
    .bind(start)
    .fetch_one(&state.pool)
    .await
    .map_err(internal("failed to look up message"))?;
    let later: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM messages WHERE session_id = $1 AND sender_channel_identity_id IS NOT NULL AND created_at > $2)",
    )
    .bind(session_id)
    .bind(created_at)
    .fetch_one(&state.pool)
    .await
    .map_err(internal("failed to look up message"))?;
    if later {
        return Err((StatusCode::CONFLICT, "only the latest message can run again"));
    }
    let busy: bool = sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM turn_jobs WHERE session_id = $1 AND status IN ('pending', 'processing'))")
        .bind(session_id)
        .fetch_one(&state.pool)
        .await
        .map_err(internal("failed to check the chat"))?;
    if busy {
        return Err((StatusCode::CONFLICT, "the crew is already working in this chat"));
    }

    // Who wrote it is checked here too: only its author takes a turn back.
    let related = related_to(&state.pool, user_id, session_id, start).await?;
    let replies: serde_json::Value = sqlx::query_scalar(
        "SELECT COALESCE(jsonb_agg(jsonb_build_object('content', content, 'agent_display_name', agent_display_name, \
           'content_blocks', content_blocks, 'created_at', created_at) ORDER BY created_at), '[]') \
         FROM messages WHERE id = ANY($1) AND id <> $2",
    )
    .bind(&related.message_ids)
    .bind(start)
    .fetch_one(&state.pool)
    .await
    .map_err(internal("failed to keep the earlier reply"))?;

    let mut tx = state.pool.begin().await.map_err(internal("failed to run it again"))?;
    let saved: i64 = sqlx::query_scalar(
        "INSERT INTO message_versions (message_id, version, text, replies) \
         SELECT $1, COALESCE(max(version), 0) + 1, $2, $3 FROM message_versions WHERE message_id = $1 RETURNING version::bigint",
    )
    .bind(start)
    .bind(&old_text)
    .bind(&replies)
    .fetch_one(&mut *tx)
    .await
    .map_err(internal("failed to keep the earlier reply"))?;
    let blobs = remove(&mut tx, &related, Some(start)).await?;
    let edited_at: Option<DateTime<Utc>> = match new_text {
        Some(text) => sqlx::query_scalar("UPDATE messages SET content = $2, edited_at = now() WHERE id = $1 RETURNING edited_at")
            .bind(start)
            .bind(text)
            .fetch_one(&mut *tx)
            .await
            .map_err(internal("failed to save the message"))?,
        None => sqlx::query_scalar("SELECT edited_at FROM messages WHERE id = $1").bind(start).fetch_one(&mut *tx).await.map_err(internal("failed to run it again"))?,
    };
    let text = new_text.unwrap_or(&old_text);
    let asked = match style.hint() {
        Some(hint) => format!("{text}\n\n{hint}"),
        None => text.to_string(),
    };
    nomi_turn::queue::enqueue(&mut tx, session_id, sender, &asked, Some(org_id)).await.map_err(|e| {
        tracing::error!(error = %e, "failed to queue the turn again");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to run it again")
    })?;
    tx.commit().await.map_err(internal("failed to run it again"))?;
    drop_blobs(blobs).await;

    let total = saved + 1;
    Ok(RerunResponse { message_id: start, version: total, total, edited_at })
}

#[derive(Deserialize)]
pub struct EditRequest {
    pub text: String,
}

/// `PATCH /api/sessions/:id/messages/:message_id`: the person's message, as they now want it;
/// the crew answers it again.
pub async fn edit(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path((session_id, message_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<EditRequest>,
) -> Result<Json<RerunResponse>, ApiError> {
    super::sessions::authorize_session_access(&state.pool, claims.sub, session_id).await?;
    let text = req.text.trim();
    if text.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "text must not be empty"));
    }
    if super::sessions::too_long(text) {
        return Err((StatusCode::PAYLOAD_TOO_LARGE, "message is too long"));
    }
    let start = turn_start(&state.pool, session_id, message_id).await?;
    if start != message_id {
        return Err((StatusCode::BAD_REQUEST, "only the person's own message can be edited"));
    }
    Ok(Json(rerun(&state, claims.sub, session_id, start, Some(text), Style::Again).await?))
}

#[derive(Deserialize, Default)]
pub struct RegenerateRequest {
    #[serde(default)]
    pub style: Style,
}

/// `POST /api/sessions/:id/messages/:message_id/regenerate`, optionally `{ style }`: the crew's
/// reply, written again (or shorter, longer, as a list). For another model, the app sets the
/// chat's model first (`PUT /api/sessions/:id/model`).
pub async fn regenerate(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path((session_id, message_id)): Path<(Uuid, Uuid)>,
    body: Option<Json<RegenerateRequest>>,
) -> Result<(StatusCode, Json<RerunResponse>), ApiError> {
    super::sessions::authorize_session_access(&state.pool, claims.sub, session_id).await?;
    let start = turn_start(&state.pool, session_id, message_id).await?;
    let style = body.map(|Json(req)| req.style).unwrap_or_default();
    Ok((StatusCode::ACCEPTED, Json(rerun(&state, claims.sub, session_id, start, None, style).await?)))
}

#[derive(Serialize, sqlx::FromRow)]
pub struct Version {
    pub version: i32,
    /// What the person wrote then.
    pub text: String,
    /// The replies it got: `content`, `agent_display_name`, `content_blocks`, `created_at`.
    pub replies: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct VersionsResponse {
    pub message_id: Uuid,
    /// The earlier takes, oldest first; the live one in the chat is number `total`.
    pub versions: Vec<Version>,
    pub total: i64,
}

/// `GET /api/sessions/:id/messages/:message_id/versions`: the earlier takes of the turn this
/// message belongs to.
pub async fn versions(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path((session_id, message_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<VersionsResponse>, ApiError> {
    super::sessions::authorize_session_access(&state.pool, claims.sub, session_id).await?;
    let start = turn_start(&state.pool, session_id, message_id).await?;
    let versions: Vec<Version> = sqlx::query_as("SELECT version, text, replies, created_at FROM message_versions WHERE message_id = $1 ORDER BY version")
        .bind(start)
        .fetch_all(&state.pool)
        .await
        .map_err(internal("failed to list versions"))?;
    let total = versions.len() as i64 + 1;
    Ok(Json(VersionsResponse { message_id: start, versions, total }))
}
