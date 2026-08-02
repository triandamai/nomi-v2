use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::app::AppState;
use crate::auth::extractor::AuthClaims;
use crate::turn::bootstrap::bootstrap_identity_and_session;
use crate::web_identity::ensure_web_channel_identity;

#[derive(Serialize)]
pub struct CreateSessionResponse {
    pub session_id: Uuid,
}

pub async fn create_session(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
) -> Result<(StatusCode, Json<CreateSessionResponse>), (StatusCode, &'static str)> {
    ensure_web_channel_identity(&state.pool, claims.sub)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to resolve web identity"))?;

    let chat_id = Uuid::new_v4().to_string();
    let bootstrap_result = bootstrap_identity_and_session(
        &state.pool,
        "web",
        "dm",
        &chat_id,
        &claims.sub.to_string(),
        Some(claims.active_org_id),
    )
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to create session"))?;

    Ok((StatusCode::CREATED, Json(CreateSessionResponse { session_id: bootstrap_result.session_id })))
}

#[derive(Serialize)]
pub struct LastMessagePreview {
    pub content: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct SessionListItem {
    pub id: Uuid,
    pub channel: String,
    pub chat_type: String,
    pub chat_id: String,
    pub last_message: Option<LastMessagePreview>,
    pub agent_active: bool,
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct ListSessionsResponse {
    pub sessions: Vec<SessionListItem>,
}

pub async fn list_sessions(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
) -> Result<Json<ListSessionsResponse>, (StatusCode, &'static str)> {
    let rows: Vec<(Uuid, String, String, String, Option<String>, Option<DateTime<Utc>>, bool, DateTime<Utc>)> = sqlx::query_as(
        "SELECT \
            s.id, s.channel, s.chat_type, s.chat_id, \
            lm.content, lm.created_at, \
            EXISTS (SELECT 1 FROM agent_sessions ag WHERE ag.session_id = s.id AND ag.status = 'active'), \
            COALESCE(lm.created_at, s.created_at) \
         FROM sessions s \
         LEFT JOIN LATERAL ( \
             SELECT content, created_at FROM messages m WHERE m.session_id = s.id ORDER BY m.created_at DESC LIMIT 1 \
         ) lm ON true \
         WHERE s.org_id = $1 \
         ORDER BY COALESCE(lm.created_at, s.created_at) DESC",
    )
    .bind(claims.active_org_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to list sessions"))?;

    let sessions = rows
        .into_iter()
        .map(|(id, channel, chat_type, chat_id, last_content, last_created_at, agent_active, updated_at)| {
            let last_message = match (last_content, last_created_at) {
                (Some(content), Some(created_at)) => Some(LastMessagePreview { content, created_at }),
                _ => None,
            };
            SessionListItem { id, channel, chat_type, chat_id, last_message, agent_active, updated_at }
        })
        .collect();

    Ok(Json(ListSessionsResponse { sessions }))
}

async fn authorize_session_access(
    pool: &sqlx::PgPool,
    user_id: Uuid,
    session_id: Uuid,
) -> Result<(), (StatusCode, &'static str)> {
    let org_id: Option<Uuid> = sqlx::query_scalar("SELECT org_id FROM sessions WHERE id = $1")
        .bind(session_id)
        .fetch_optional(pool)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to look up session"))?;

    let org_id = org_id.ok_or((StatusCode::NOT_FOUND, "session not found"))?;

    crate::auth::authorize::authorize_org_action(pool, user_id, org_id, &["owner", "admin", "member"])
        .await
        .map_err(|_| (StatusCode::NOT_FOUND, "session not found"))?;

    Ok(())
}

#[derive(Deserialize)]
pub struct MessagesQuery {
    pub before: Option<Uuid>,
    pub limit: Option<i64>,
}

#[derive(Serialize)]
pub struct MessageItem {
    pub id: Uuid,
    pub sender: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct ListMessagesResponse {
    pub messages: Vec<MessageItem>,
}

fn to_message_item((id, sender, content, created_at): (Uuid, Option<Uuid>, String, DateTime<Utc>)) -> MessageItem {
    MessageItem {
        id,
        sender: if sender.is_some() { "user".to_string() } else { "assistant".to_string() },
        content,
        created_at,
    }
}

pub async fn list_messages(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(session_id): Path<Uuid>,
    Query(query): Query<MessagesQuery>,
) -> Result<Json<ListMessagesResponse>, (StatusCode, &'static str)> {
    authorize_session_access(&state.pool, claims.sub, session_id).await?;

    let limit = query.limit.unwrap_or(50).clamp(1, 100);

    let rows: Vec<(Uuid, Option<Uuid>, String, DateTime<Utc>)> = match query.before {
        Some(before_id) => sqlx::query_as(
            "SELECT id, sender_channel_identity_id, content, created_at FROM messages \
             WHERE session_id = $1 AND created_at < (SELECT created_at FROM messages WHERE id = $2) \
             ORDER BY created_at DESC LIMIT $3",
        )
        .bind(session_id)
        .bind(before_id)
        .bind(limit)
        .fetch_all(&state.pool)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to fetch messages"))?,
        None => sqlx::query_as(
            "SELECT id, sender_channel_identity_id, content, created_at FROM messages \
             WHERE session_id = $1 \
             ORDER BY created_at DESC LIMIT $2",
        )
        .bind(session_id)
        .bind(limit)
        .fetch_all(&state.pool)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to fetch messages"))?,
    };

    let mut messages: Vec<MessageItem> = rows.into_iter().map(to_message_item).collect();
    messages.reverse();

    Ok(Json(ListMessagesResponse { messages }))
}

#[derive(Deserialize)]
pub struct SendMessageRequest {
    pub text: String,
}

#[derive(Serialize)]
pub struct SendMessageResponse {
    pub user_message: MessageItem,
    pub assistant_message: MessageItem,
}

pub async fn send_message(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(session_id): Path<Uuid>,
    Json(req): Json<SendMessageRequest>,
) -> Result<Json<SendMessageResponse>, (StatusCode, &'static str)> {
    authorize_session_access(&state.pool, claims.sub, session_id).await?;

    if req.text.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "text must not be empty"));
    }

    let (channel, chat_type, chat_id): (String, String, String) =
        sqlx::query_as("SELECT channel, chat_type, chat_id FROM sessions WHERE id = $1")
            .bind(session_id)
            .fetch_one(&state.pool)
            .await
            .map_err(|_| (StatusCode::NOT_FOUND, "session not found"))?;

    let provider = state.provider.read().await.clone();
    let embedding_provider = state.embedding_provider.read().await.clone();

    crate::turn::handle_inbound_message(
        &state.pool,
        provider.as_ref(),
        embedding_provider.as_ref(),
        &channel,
        &chat_type,
        &chat_id,
        &claims.sub.to_string(),
        &req.text,
        Some(claims.active_org_id),
    )
    .await
    .map_err(|_| (StatusCode::BAD_GATEWAY, "failed to process message"))?;

    let rows: Vec<(Uuid, Option<Uuid>, String, DateTime<Utc>)> = sqlx::query_as(
        "SELECT id, sender_channel_identity_id, content, created_at FROM messages \
         WHERE session_id = $1 ORDER BY created_at DESC LIMIT 2",
    )
    .bind(session_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to fetch persisted messages"))?;

    if rows.len() != 2 {
        return Err((StatusCode::INTERNAL_SERVER_ERROR, "expected exactly two new messages after a successful turn"));
    }

    let assistant_message = to_message_item(rows[0].clone());
    let user_message = to_message_item(rows[1].clone());

    Ok(Json(SendMessageResponse { user_message, assistant_message }))
}
