use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use chrono::{DateTime, Utc};
use rumqttc::{AsyncClient, Event, MqttOptions, Packet, QoS};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use uuid::Uuid;

use crate::app::AppState;
use crate::bootstrap::build_llm_provider_for_user;
use nomi_auth::extractor::AuthClaims;
use nomi_turn::bootstrap::bootstrap_identity_and_session;
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
    pub title: Option<String>,
    pub last_message: Option<LastMessagePreview>,
    pub agent_active: bool,
    pub updated_at: DateTime<Utc>,
    /// Set when this session has a project attached — the history list uses this to badge it and
    /// route into the project workspace instead of the plain chat page.
    pub project_id: Option<Uuid>,
}

#[derive(Serialize)]
pub struct ListSessionsResponse {
    pub sessions: Vec<SessionListItem>,
}

pub async fn list_sessions(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
) -> Result<Json<ListSessionsResponse>, (StatusCode, &'static str)> {
    #[allow(clippy::type_complexity)]
    let rows: Vec<(Uuid, String, String, String, Option<String>, Option<String>, Option<DateTime<Utc>>, bool, DateTime<Utc>, Option<Uuid>)> = sqlx::query_as(
        "SELECT \
            s.id, s.channel, s.chat_type, s.chat_id, s.title, \
            lm.content, lm.created_at, \
            EXISTS (SELECT 1 FROM agent_sessions ag WHERE ag.session_id = s.id AND ag.status = 'active'), \
            COALESCE(lm.created_at, s.created_at), \
            p.id \
         FROM sessions s \
         LEFT JOIN LATERAL ( \
             SELECT content, created_at FROM messages m WHERE m.session_id = s.id ORDER BY m.created_at DESC LIMIT 1 \
         ) lm ON true \
         LEFT JOIN LATERAL ( \
             SELECT id FROM projects pr WHERE pr.session_id = s.id ORDER BY pr.created_at DESC LIMIT 1 \
         ) p ON true \
         WHERE s.org_id = $1 \
         ORDER BY COALESCE(lm.created_at, s.created_at) DESC",
    )
    .bind(claims.active_org_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to list sessions"))?;

    let sessions = rows
        .into_iter()
        .map(|(id, channel, chat_type, chat_id, title, last_content, last_created_at, agent_active, updated_at, project_id)| {
            let last_message = match (last_content, last_created_at) {
                (Some(content), Some(created_at)) => Some(LastMessagePreview { content, created_at }),
                _ => None,
            };
            SessionListItem { id, channel, chat_type, chat_id, title, last_message, agent_active, updated_at, project_id }
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

    nomi_auth::authorize::authorize_org_action(pool, user_id, org_id, &["owner", "admin", "member"])
        .await
        .map_err(|_| (StatusCode::NOT_FOUND, "session not found"))?;

    Ok(())
}

async fn exec_delete_step(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    sql: &str,
    session_id: Uuid,
) -> Result<(), (StatusCode, &'static str)> {
    sqlx::query(sql).bind(session_id).execute(&mut **tx).await.map_err(|e| {
        tracing::error!(error = %e, sql = %sql, "failed to delete chat data");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to delete chat")
    })?;
    Ok(())
}

/// Deletes a chat and everything hanging off it: any attached project's files on disk, then (in
/// dependency order — none of these tables cascade from `sessions` except `projects`, which does)
/// every DB row that references the session, then the session itself. `projects`/`project_files`
/// clean up automatically via their existing `ON DELETE CASCADE`.
#[tracing::instrument(skip(state, claims))]
pub async fn delete_session(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(session_id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, &'static str)> {
    authorize_session_access(&state.pool, claims.sub, session_id).await?;

    let project_ids: Vec<Uuid> = sqlx::query_scalar("SELECT id FROM projects WHERE session_id = $1")
        .bind(session_id)
        .fetch_all(&state.pool)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "failed to look up projects for session");
            (StatusCode::INTERNAL_SERVER_ERROR, "failed to delete chat")
        })?;

    for project_id in &project_ids {
        if let Err(e) = state.project_storage.delete_prefix(&project_id.to_string()).await {
            tracing::warn!(error = %e, project_id = %project_id, "failed to delete project files from disk");
        }
    }

    let mut tx = state.pool.begin().await.map_err(|e| {
        tracing::error!(error = %e, "failed to start transaction");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to delete chat")
    })?;

    for sql in [
        "DELETE FROM message_memory_usage WHERE message_id IN (SELECT id FROM messages WHERE session_id = $1)",
        "DELETE FROM agent_events WHERE session_id = $1",
        "DELETE FROM agent_sessions WHERE session_id = $1",
        "DELETE FROM agent_delegations WHERE session_id = $1",
        "DELETE FROM turn_jobs WHERE session_id = $1",
        "DELETE FROM messages WHERE session_id = $1",
        "DELETE FROM session_participants WHERE session_id = $1",
        "DELETE FROM sessions WHERE id = $1",
    ] {
        exec_delete_step(&mut tx, sql, session_id).await?;
    }

    tx.commit().await.map_err(|e| {
        tracing::error!(error = %e, "failed to commit chat deletion");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to delete chat")
    })?;

    Ok(StatusCode::NO_CONTENT)
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
    pub my_feedback: Option<String>,
}

#[derive(Serialize)]
pub struct ListMessagesResponse {
    pub messages: Vec<MessageItem>,
}

fn to_message_item(
    (id, sender, content, created_at, my_feedback): (Uuid, Option<Uuid>, String, DateTime<Utc>, Option<String>),
) -> MessageItem {
    MessageItem {
        id,
        sender: if sender.is_some() { "user".to_string() } else { "assistant".to_string() },
        content,
        created_at,
        my_feedback,
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

    let rows: Vec<(Uuid, Option<Uuid>, String, DateTime<Utc>, Option<String>)> = match query.before {
        Some(before_id) => sqlx::query_as(
            "SELECT m.id, m.sender_channel_identity_id, m.content, m.created_at, mf.rating \
             FROM messages m \
             LEFT JOIN message_feedback mf ON mf.message_id = m.id AND mf.user_id = $4 \
             WHERE m.session_id = $1 AND m.created_at < (SELECT created_at FROM messages WHERE id = $2) \
             ORDER BY m.created_at DESC LIMIT $3",
        )
        .bind(session_id)
        .bind(before_id)
        .bind(limit)
        .bind(claims.sub)
        .fetch_all(&state.pool)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to fetch messages"))?,
        None => sqlx::query_as(
            "SELECT m.id, m.sender_channel_identity_id, m.content, m.created_at, mf.rating \
             FROM messages m \
             LEFT JOIN message_feedback mf ON mf.message_id = m.id AND mf.user_id = $3 \
             WHERE m.session_id = $1 \
             ORDER BY m.created_at DESC LIMIT $2",
        )
        .bind(session_id)
        .bind(limit)
        .bind(claims.sub)
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
pub struct IngestMessageResponse {
    pub user_message: MessageItem,
}

#[tracing::instrument(skip(state, claims, req), fields(session_id = %session_id, user_id = %claims.sub, text_len = req.text.len()))]
pub async fn send_message(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(session_id): Path<Uuid>,
    Json(req): Json<SendMessageRequest>,
) -> Result<(StatusCode, Json<IngestMessageResponse>), (StatusCode, &'static str)> {
    authorize_session_access(&state.pool, claims.sub, session_id).await?;

    if req.text.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "text must not be empty"));
    }

    let (channel, chat_type, chat_id, title): (String, String, String, Option<String>) =
        sqlx::query_as("SELECT channel, chat_type, chat_id, title FROM sessions WHERE id = $1")
            .bind(session_id)
            .fetch_one(&state.pool)
            .await
            .map_err(|e| {
                tracing::warn!(error = %e, "session lookup failed");
                (StatusCode::NOT_FOUND, "session not found")
            })?;

    tracing::debug!(channel = %channel, chat_type = %chat_type, "ingesting inbound message");
    let ingested = nomi_turn::ingest::ingest_inbound_message(
        &state.pool,
        &channel,
        &chat_type,
        &chat_id,
        &claims.sub.to_string(),
        &req.text,
        Some(claims.active_org_id),
    )
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "ingest failed");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to ingest message")
    })?;
    tracing::info!(turn_job_id = %ingested.turn_job_id, "message ingested and queued");

    // First message on this session — kick off title generation in the background so it never
    // adds latency to sending a message. Race-safe: the UPDATE only applies WHERE title IS NULL.
    if title.is_none() {
        let pool = state.pool.clone();
        let settings_key = state.settings_key;
        let http_client = state.http_client.clone();
        let user_id = claims.sub;
        let first_message = req.text.clone();
        tokio::spawn(async move {
            generate_session_title(pool, session_id, user_id, settings_key, http_client, first_message).await;
        });
    }

    let (created_at,): (DateTime<Utc>,) = sqlx::query_as("SELECT created_at FROM messages WHERE id = $1")
        .bind(ingested.user_message_id)
        .fetch_one(&state.pool)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "failed to fetch persisted user message");
            (StatusCode::INTERNAL_SERVER_ERROR, "failed to fetch persisted user message")
        })?;

    let user_message = MessageItem {
        id: ingested.user_message_id,
        sender: "user".to_string(),
        content: req.text,
        created_at,
        my_feedback: None,
    };

    Ok((StatusCode::ACCEPTED, Json(IngestMessageResponse { user_message })))
}

const TITLE_FALLBACK_MAX_CHARS: usize = 60;
const TITLE_GENERATION_MAX_TOKENS: u32 = 32;

/// Fallback title source when the LLM's own title attempt is missing or degenerate — the raw
/// first message, trimmed to a sane length on a word boundary.
fn truncate_for_title(message: &str) -> String {
    let trimmed = message.trim();
    if trimmed.chars().count() <= TITLE_FALLBACK_MAX_CHARS {
        return trimmed.to_string();
    }
    let mut truncated: String = trimmed.chars().take(TITLE_FALLBACK_MAX_CHARS).collect();
    if let Some(last_space) = truncated.rfind(' ') {
        truncated.truncate(last_space);
    }
    format!("{}…", truncated.trim_end())
}

/// One-shot, tool-free completion that titles a new chat from its first message — fired in the
/// background from send_message so it never delays the reply the user is actually waiting for.
async fn generate_session_title(
    pool: sqlx::PgPool,
    session_id: Uuid,
    user_id: Uuid,
    settings_key: [u8; 32],
    http_client: reqwest::Client,
    first_message: String,
) {
    let provider = build_llm_provider_for_user(&pool, user_id, &settings_key, http_client).await;
    let request = nomi_llm::LlmRequest {
        system: Some(nomi_agent_core::prompts::SESSION_TITLE_SYSTEM_PROMPT.to_string()),
        messages: vec![nomi_llm::LlmMessage {
            role: nomi_llm::LlmRole::User,
            content: vec![nomi_llm::ContentBlock::Text { text: first_message.clone() }],
        }],
        tools: vec![],
        max_tokens: TITLE_GENERATION_MAX_TOKENS,
        enable_reasoning: false,
    };

    let generated = match nomi_llm::complete(provider.as_ref(), request).await {
        Ok(response) => response.content.into_iter().find_map(|block| match block {
            nomi_llm::ContentBlock::Text { text } => Some(text.trim().trim_matches('"').to_string()),
            _ => None,
        }),
        Err(e) => {
            tracing::warn!(error = %e, session_id = %session_id, "failed to generate session title");
            None
        }
    };

    // A degenerate reply (empty, or a single word the model produced despite the prompt) isn't
    // worth persisting as-is — fall back to the raw first message, trimmed to a sane length,
    // rather than titling the chat "Spicy".
    let title = match generated {
        Some(t) if t.split_whitespace().count() >= 2 => t,
        _ => truncate_for_title(&first_message),
    };
    if title.is_empty() {
        return;
    }

    let _ = sqlx::query("UPDATE sessions SET title = $1 WHERE id = $2 AND title IS NULL")
        .bind(&title)
        .bind(session_id)
        .execute(&pool)
        .await;
}

#[derive(Deserialize)]
pub struct FeedbackRequest {
    pub rating: String,
}

const ALLOWED_RATINGS: [&str; 2] = ["up", "down"];

async fn authorize_message_in_session(
    pool: &sqlx::PgPool,
    session_id: Uuid,
    message_id: Uuid,
) -> Result<(), (StatusCode, &'static str)> {
    let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM messages WHERE id = $1 AND session_id = $2)")
        .bind(message_id)
        .bind(session_id)
        .fetch_one(pool)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to look up message"))?;
    if exists {
        Ok(())
    } else {
        Err((StatusCode::NOT_FOUND, "message not found"))
    }
}

pub async fn put_message_feedback(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path((session_id, message_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<FeedbackRequest>,
) -> Result<StatusCode, (StatusCode, &'static str)> {
    authorize_session_access(&state.pool, claims.sub, session_id).await?;
    authorize_message_in_session(&state.pool, session_id, message_id).await?;

    if !ALLOWED_RATINGS.contains(&req.rating.as_str()) {
        return Err((StatusCode::BAD_REQUEST, "rating must be 'up' or 'down'"));
    }

    sqlx::query(
        "INSERT INTO message_feedback (message_id, user_id, rating) VALUES ($1, $2, $3) \
         ON CONFLICT (message_id, user_id) DO UPDATE SET rating = EXCLUDED.rating",
    )
    .bind(message_id)
    .bind(claims.sub)
    .bind(&req.rating)
    .execute(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "failed to save message feedback");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to save feedback")
    })?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn delete_message_feedback(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path((session_id, message_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, (StatusCode, &'static str)> {
    authorize_session_access(&state.pool, claims.sub, session_id).await?;
    authorize_message_in_session(&state.pool, session_id, message_id).await?;

    sqlx::query("DELETE FROM message_feedback WHERE message_id = $1 AND user_id = $2")
        .bind(message_id)
        .bind(claims.sub)
        .execute(&state.pool)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "failed to delete message feedback");
            (StatusCode::INTERNAL_SERVER_ERROR, "failed to delete feedback")
        })?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn session_stream(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(session_id): Path<Uuid>,
    ws: WebSocketUpgrade,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    authorize_session_access(&state.pool, claims.sub, session_id).await?;
    let broker_host = state.mqtt_broker_host.clone();
    let broker_port = state.mqtt_broker_port;
    Ok(ws.on_upgrade(move |socket| relay_session_stream(socket, session_id, broker_host, broker_port)))
}

#[derive(Serialize)]
pub struct AgentActivityItem {
    pub id: Uuid,
    pub target_agent_type: String,
    pub task: String,
    pub status: String,
    pub result: Option<String>,
    pub error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

pub async fn list_agent_activity(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(session_id): Path<Uuid>,
) -> Result<Json<Vec<AgentActivityItem>>, (StatusCode, &'static str)> {
    authorize_session_access(&state.pool, claims.sub, session_id).await?;

    let rows: Vec<(Uuid, String, String, String, Option<String>, Option<String>, DateTime<Utc>, Option<DateTime<Utc>>)> =
        sqlx::query_as(
            "SELECT id, target_agent_type, task, status, result, error, created_at, completed_at \
             FROM agent_delegations WHERE session_id = $1 ORDER BY created_at DESC LIMIT 20",
        )
        .bind(session_id)
        .fetch_all(&state.pool)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "failed to list agent activity");
            (StatusCode::INTERNAL_SERVER_ERROR, "failed to list agent activity")
        })?;

    let items = rows
        .into_iter()
        .map(|(id, target_agent_type, task, status, result, error, created_at, completed_at)| AgentActivityItem {
            id, target_agent_type, task, status, result, error, created_at, completed_at,
        })
        .collect();

    Ok(Json(items))
}

async fn relay_session_stream(mut socket: WebSocket, session_id: Uuid, broker_host: String, broker_port: u16) {
    let mut options = MqttOptions::new(format!("ws-bridge-{}", Uuid::new_v4()), broker_host, broker_port);
    options.set_keep_alive(Duration::from_secs(30));
    let (client, mut eventloop) = AsyncClient::new(options, 16);
    let topic = format!("chat/{session_id}/stream");
    if client.subscribe(&topic, QoS::AtMostOnce).await.is_err() {
        return; // socket closes on drop
    }

    loop {
        tokio::select! {
            event = eventloop.poll() => {
                match event {
                    Ok(Event::Incoming(Packet::Publish(publish))) => {
                        if socket.send(Message::Text(String::from_utf8_lossy(&publish.payload).into_owned())).await.is_err() {
                            break; // client disconnected
                        }
                    }
                    Ok(_) => continue,
                    Err(_) => break, // broker connection lost; let the client reconnect
                }
            }
            incoming = socket.recv() => {
                if incoming.is_none() { break; } // client closed the socket
            }
        }
    }
}
