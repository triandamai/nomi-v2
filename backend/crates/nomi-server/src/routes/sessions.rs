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
use nomi_turn::bootstrap::ensure_web_channel_identity;

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

/// The person's chats, latest first. A chat nobody has written in yet isn't listed: a new chat
/// only counts once its first message is sent (a project's chat is listed from the start).
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
             SELECT content, created_at FROM messages m WHERE m.session_id = s.id AND m.content NOT LIKE '🧠%' \
             ORDER BY m.created_at DESC LIMIT 1 \
         ) lm ON true \
         LEFT JOIN LATERAL ( \
             SELECT id FROM projects pr WHERE pr.session_id = s.id ORDER BY pr.created_at DESC LIMIT 1 \
         ) p ON true \
         WHERE s.org_id = $1 AND s.user_id = $2 \
           AND (lm.created_at IS NOT NULL OR p.id IS NOT NULL) \
         ORDER BY COALESCE(lm.created_at, s.created_at) DESC",
    )
    .bind(claims.active_org_id)
    .bind(claims.sub)
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

/// A chat is private to the person who started it: sharing an organization with them doesn't let
/// anyone else read it, post into it or see its memories. Someone else's chat answers exactly like
/// one that doesn't exist.
async fn authorize_session_access(
    pool: &sqlx::PgPool,
    user_id: Uuid,
    session_id: Uuid,
) -> Result<(), (StatusCode, &'static str)> {
    let org_id: Option<Uuid> = sqlx::query_scalar("SELECT org_id FROM sessions WHERE id = $1 AND user_id = $2")
        .bind(session_id)
        .bind(user_id)
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
    pub content_blocks: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub my_feedback: Option<String>,
    pub agent_display_name: Option<String>,
    /// How many memories this reply drew on.
    pub memory_count: i64,
}

#[derive(Serialize)]
pub struct ListMessagesResponse {
    pub messages: Vec<MessageItem>,
}

type MessageRow = (Uuid, Option<Uuid>, String, DateTime<Utc>, Option<serde_json::Value>, Option<String>, Option<String>, i64);

fn to_message_item(
    (id, sender, content, created_at, content_blocks, my_feedback, agent_display_name, memory_count): MessageRow,
) -> MessageItem {
    MessageItem {
        id,
        sender: if sender.is_some() { "user".to_string() } else { "assistant".to_string() },
        content,
        content_blocks,
        created_at,
        my_feedback,
        agent_display_name,
        memory_count,
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

    let rows: Vec<MessageRow> = match query.before {
        Some(before_id) => sqlx::query_as(
            "SELECT m.id, m.sender_channel_identity_id, m.content, m.created_at, m.content_blocks, mf.rating, m.agent_display_name, \
                    (SELECT count(*) FROM message_memory_usage u WHERE u.message_id = m.id) AS memory_count \
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
            "SELECT m.id, m.sender_channel_identity_id, m.content, m.created_at, m.content_blocks, mf.rating, m.agent_display_name, \
                    (SELECT count(*) FROM message_memory_usage u WHERE u.message_id = m.id) AS memory_count \
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

pub async fn get_message(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path((session_id, message_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<MessageItem>, (StatusCode, &'static str)> {
    authorize_session_access(&state.pool, claims.sub, session_id).await?;

    let row: MessageRow = sqlx::query_as(
        "SELECT m.id, m.sender_channel_identity_id, m.content, m.created_at, m.content_blocks, mf.rating, m.agent_display_name, \
                    (SELECT count(*) FROM message_memory_usage u WHERE u.message_id = m.id) AS memory_count \
         FROM messages m \
         LEFT JOIN message_feedback mf ON mf.message_id = m.id AND mf.user_id = $3 \
         WHERE m.id = $1 AND m.session_id = $2",
    )
    .bind(message_id)
    .bind(session_id)
    .bind(claims.sub)
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to fetch message"))?
    .ok_or((StatusCode::NOT_FOUND, "message not found"))?;

    Ok(Json(to_message_item(row)))
}

#[derive(Serialize)]
pub struct AgentPlanItem {
    pub id: Uuid,
    pub title: String,
    pub version: i32,
    pub content: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct ListAgentPlansResponse {
    pub plans: Vec<AgentPlanItem>,
}

pub async fn list_agent_plans(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path((session_id, agent_session_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ListAgentPlansResponse>, (StatusCode, &'static str)> {
    authorize_session_access(&state.pool, claims.sub, session_id).await?;

    let rows: Vec<(Uuid, String, i32, Option<String>, Option<String>, DateTime<Utc>)> = sqlx::query_as(
        "SELECT id, title, version, content, content_s3_key, created_at FROM agent_plans \
         WHERE session_id = $1 AND agent_session_id = $2 ORDER BY version ASC",
    )
    .bind(session_id)
    .bind(agent_session_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to fetch plans"))?;

    let mut plans = Vec::with_capacity(rows.len());
    for (id, title, version, content, content_s3_key, created_at) in rows {
        let resolved_content = match (content, content_s3_key) {
            (Some(inline), _) => Some(inline),
            (None, Some(key)) => match &state.s3 {
                Some(s3) => s3.get_object(&key).await.ok().flatten(),
                None => None,
            },
            (None, None) => None,
        };
        plans.push(AgentPlanItem { id, title, version, content: resolved_content, created_at });
    }

    Ok(Json(ListAgentPlansResponse { plans }))
}

/// Ticks or unticks the `index`-th task-list item (`- [ ]` / `- [x]`) in a plan's markdown,
/// counting items the way the chat's checklist does. `None` when there's no such item.
pub fn set_plan_item_done(markdown: &str, index: usize, done: bool) -> Option<String> {
    let mut seen = 0;
    let mut changed = false;
    let lines: Vec<String> = markdown
        .split('\n')
        .map(|line| {
            let indent = line.len() - line.trim_start().len();
            let rest = &line[indent..];
            let is_task = rest.starts_with(['-', '*', '+'])
                && rest[1..].starts_with(char::is_whitespace)
                && {
                    let after = rest[1..].trim_start();
                    (after.starts_with("[ ]") || after.starts_with("[x]") || after.starts_with("[X]"))
                        && after[3..].starts_with(char::is_whitespace)
                        && !after[3..].trim().is_empty()
                };
            if !is_task {
                return line.to_string();
            }
            let this = seen;
            seen += 1;
            if this != index {
                return line.to_string();
            }
            changed = true;
            let box_at = indent + 1 + (rest[1..].len() - rest[1..].trim_start().len());
            let mark = if done { "[x]" } else { "[ ]" };
            format!("{}{}{}", &line[..box_at], mark, &line[box_at + 3..])
        })
        .collect();
    changed.then(|| lines.join("\n"))
}

#[derive(Deserialize)]
pub struct PlanItemRequest {
    pub done: bool,
}

/// The person ticks a step off (or back on) in a plan's checklist. Saved into that version's
/// markdown in place, so the agent sees the progress the next time it reads the plan.
pub async fn set_agent_plan_item(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path((session_id, agent_session_id, plan_id, index)): Path<(Uuid, Uuid, Uuid, usize)>,
    Json(req): Json<PlanItemRequest>,
) -> Result<StatusCode, (StatusCode, &'static str)> {
    authorize_session_access(&state.pool, claims.sub, session_id).await?;

    let row: Option<(Option<String>, Option<String>)> = sqlx::query_as(
        "SELECT content, content_s3_key FROM agent_plans WHERE id = $1 AND session_id = $2 AND agent_session_id = $3",
    )
    .bind(plan_id)
    .bind(session_id)
    .bind(agent_session_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to update plan"))?;
    let (content, s3_key) = row.ok_or((StatusCode::NOT_FOUND, "plan not found"))?;

    let current = match (&content, &s3_key, &state.s3) {
        (Some(inline), _, _) => inline.clone(),
        (None, Some(key), Some(s3)) => s3
            .get_object(key)
            .await
            .ok()
            .flatten()
            .ok_or((StatusCode::BAD_GATEWAY, "failed to load plan"))?,
        _ => return Err((StatusCode::BAD_GATEWAY, "failed to load plan")),
    };
    let updated = set_plan_item_done(&current, index, req.done).ok_or((StatusCode::NOT_FOUND, "no such checklist item"))?;

    match (content, s3_key, &state.s3) {
        (Some(_), _, _) => {
            sqlx::query("UPDATE agent_plans SET content = $1 WHERE id = $2")
                .bind(&updated)
                .bind(plan_id)
                .execute(&state.pool)
                .await
                .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to update plan"))?;
        }
        (None, Some(key), Some(s3)) => {
            s3.put_object(&key, &updated, "text/markdown")
                .await
                .map_err(|_| (StatusCode::BAD_GATEWAY, "failed to save plan"))?;
        }
        _ => unreachable!("loaded above"),
    }
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
pub struct RenameSessionRequest {
    pub title: String,
}

#[derive(Serialize)]
pub struct RenameSessionResponse {
    pub title: String,
}

/// The person renames a chat.
pub async fn rename_session(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(session_id): Path<Uuid>,
    Json(req): Json<RenameSessionRequest>,
) -> Result<Json<RenameSessionResponse>, (StatusCode, &'static str)> {
    authorize_session_access(&state.pool, claims.sub, session_id).await?;
    let mut conn = state.pool.acquire().await.map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to rename chat"))?;
    match nomi_agent_core::chat_title::rename(&mut conn, session_id, &req.title).await {
        Ok(Some(title)) => Ok(Json(RenameSessionResponse { title })),
        Ok(None) => Err((StatusCode::BAD_REQUEST, "title must not be empty")),
        Err(_) => Err((StatusCode::INTERNAL_SERVER_ERROR, "failed to rename chat")),
    }
}

#[derive(Deserialize)]
pub struct SendMessageRequest {
    pub text: String,
}

#[derive(Serialize)]
pub struct IngestMessageResponse {
    pub user_message: MessageItem,
    /// Set when the message was a stop command the supervisor handled on the spot (see
    /// nomi_agent_supervisor::stop): its reply, already in the chat. Nothing was queued.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supervisor_reply: Option<MessageItem>,
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

    if let Some(response) = try_stop_command(&state, &claims, &channel, &chat_type, &chat_id, &req.text).await? {
        return Ok((StatusCode::OK, Json(response)));
    }

    // Files the message points at must be the sender's own; their tags are rewritten from our
    // records so what agents read about a file is what was really uploaded.
    let (text, attachment_ids) = crate::routes::attachments::check_message_references(&state.pool, claims.sub, &req.text).await?;
    let req = SendMessageRequest { text };

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
    crate::routes::attachments::link_to_message(&state.pool, claims.sub, &attachment_ids, session_id, ingested.user_message_id).await;

    // First message on this session — kick off title generation in the background so it never
    // adds latency to sending a message. Race-safe: the UPDATE only applies WHERE title IS NULL.
    if title.is_none() {
        let pool = state.pool.clone();
        let settings_key = state.settings_key;
        let http_client = state.http_client.clone();
        let user_id = claims.sub;
        let first_message = nomi_attachments::reference::readable(&req.text);
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
        content_blocks: None,
        created_at,
        my_feedback: None,
        agent_display_name: None,
        memory_count: 0,
    };

    Ok((StatusCode::ACCEPTED, Json(IngestMessageResponse { user_message, supervisor_reply: None })))
}

async fn fetch_message_created_at(pool: &sqlx::PgPool, message_id: Uuid) -> Result<DateTime<Utc>, (StatusCode, &'static str)> {
    sqlx::query_scalar("SELECT created_at FROM messages WHERE id = $1").bind(message_id).fetch_one(pool).await.map_err(|e| {
        tracing::error!(error = %e, "failed to fetch persisted message");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to fetch persisted message")
    })
}

/// "stop" / "stop all agents" can't wait in the turn queue behind the agent it means to stop,
/// so the supervisor handles it here, before anything is queued.
async fn try_stop_command(
    state: &AppState,
    claims: &nomi_auth::claims::Claims,
    channel: &str,
    chat_type: &str,
    chat_id: &str,
    text: &str,
) -> Result<Option<IngestMessageResponse>, (StatusCode, &'static str)> {
    if !nomi_agent_supervisor::stop::looks_like_stop_command(text) {
        return Ok(None);
    }
    let bootstrap = nomi_turn::bootstrap::bootstrap_identity_and_session(
        &state.pool,
        channel,
        chat_type,
        chat_id,
        &claims.sub.to_string(),
        Some(claims.active_org_id),
    )
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "bootstrap failed");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to ingest message")
    })?;
    let registry = crate::build_agent_registry(state.project_storage.clone());
    let outcome = nomi_agent_supervisor::stop::handle_stop_message(
        &state.pool,
        &registry,
        bootstrap.session_id,
        bootstrap.sender_channel_identity_id,
        bootstrap.user_id,
        text,
    )
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "stop command failed");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to stop agents")
    })?;
    let Some(outcome) = outcome else {
        return Ok(None);
    };
    tracing::info!(stopped = ?outcome.report.stopped, chats = outcome.report.chats, "supervisor stopped agents");

    let user_message = MessageItem {
        id: outcome.user_message_id,
        sender: "user".to_string(),
        content: text.to_string(),
        content_blocks: None,
        created_at: fetch_message_created_at(&state.pool, outcome.user_message_id).await?,
        my_feedback: None,
        agent_display_name: None,
        memory_count: 0,
    };
    let supervisor_reply = MessageItem {
        id: outcome.reply_message_id,
        sender: "assistant".to_string(),
        content: outcome.reply,
        content_blocks: None,
        created_at: fetch_message_created_at(&state.pool, outcome.reply_message_id).await?,
        my_feedback: None,
        agent_display_name: Some("Supervisor".to_string()),
        memory_count: 0,
    };
    Ok(Some(IngestMessageResponse { user_message, supervisor_reply: Some(supervisor_reply) }))
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
    let locale = match pool.acquire().await {
        Ok(mut conn) => nomi_agent_core::user_locale(&mut conn, user_id).await,
        Err(_) => nomi_agent_core::Locale::En,
    };
    let request = nomi_llm::LlmRequest {
        system: Some(format!(
            "{}\n\nWrite the title in {}, unless the message is in another language.",
            nomi_agent_core::prompts::SESSION_TITLE_SYSTEM_PROMPT,
            locale.english_name()
        )),
        messages: vec![nomi_llm::LlmMessage {
            role: nomi_llm::LlmRole::User,
            content: vec![nomi_llm::ContentBlock::Text { text: first_message.clone() }],
        }],
        tools: vec![],
        max_tokens: TITLE_GENERATION_MAX_TOKENS,
        enable_reasoning: false,
        reasoning_effort: Default::default(),
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
    /// Why a reply missed, when the person says (thumbs-down only).
    #[serde(default)]
    pub reason: Option<String>,
}

const ALLOWED_REASONS: [&str; 4] = ["wrong_memory", "not_relevant", "too_long", "other"];

async fn previous_rating(pool: &sqlx::PgPool, message_id: Uuid, user_id: Uuid) -> Result<Option<String>, (StatusCode, &'static str)> {
    sqlx::query_scalar("SELECT rating FROM message_feedback WHERE message_id = $1 AND user_id = $2")
        .bind(message_id)
        .bind(user_id)
        .fetch_optional(pool)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to save feedback"))
}

/// The memories a reply drew on get stronger with a thumbs-up and weaker with a thumbs-down;
/// changing or removing the rating takes its effect back. Best-effort: the rating itself is saved.
async fn reinforce_memories(pool: &sqlx::PgPool, message_id: Uuid, previous: Option<&str>, current: Option<&str>) {
    use nomi_agent_core::memory::{reinforce_change, ReinforcementSignal};
    let previous = previous.and_then(ReinforcementSignal::from_rating);
    let current = current.and_then(ReinforcementSignal::from_rating);
    if let Err(e) = reinforce_change(pool, message_id, previous, current).await {
        tracing::warn!(error = %e, %message_id, "failed to reinforce memories from feedback");
    }
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
    let reason = req.reason.as_deref().filter(|_| req.rating == "down");
    if reason.is_some_and(|r| !ALLOWED_REASONS.contains(&r)) {
        return Err((StatusCode::BAD_REQUEST, "reason must be wrong_memory, not_relevant, too_long or other"));
    }
    let previous = previous_rating(&state.pool, message_id, claims.sub).await?;

    sqlx::query(
        "INSERT INTO message_feedback (message_id, user_id, rating, reason) VALUES ($1, $2, $3, $4) \
         ON CONFLICT (message_id, user_id) DO UPDATE SET rating = EXCLUDED.rating, reason = EXCLUDED.reason",
    )
    .bind(message_id)
    .bind(claims.sub)
    .bind(&req.rating)
    .bind(reason)
    .execute(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "failed to save message feedback");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to save feedback")
    })?;
    reinforce_memories(&state.pool, message_id, previous.as_deref(), Some(&req.rating)).await;

    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
pub struct ApprovalRequest {
    pub decision: String,
    #[serde(default)]
    pub remember: bool,
}

pub async fn resolve_approval(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path((session_id, message_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<ApprovalRequest>,
) -> Result<StatusCode, (StatusCode, &'static str)> {
    authorize_session_access(&state.pool, claims.sub, session_id).await?;
    authorize_message_in_session(&state.pool, session_id, message_id).await?;

    if req.decision != "approve" && req.decision != "deny" {
        return Err((StatusCode::BAD_REQUEST, "decision must be 'approve' or 'deny'"));
    }

    let pending: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM agent_sessions WHERE state->>'pending_approval_message_id' = $1 AND (state->>'paused_for_approval')::boolean = true)",
    )
    .bind(message_id.to_string())
    .fetch_one(&state.pool)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to check approval status"))?;

    if !pending {
        return Err((StatusCode::CONFLICT, "this action is no longer pending"));
    }

    let mut tx = state.pool.begin().await.map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to start transaction"))?;
    nomi_turn::approval::enqueue(&mut tx, message_id, &req.decision, req.remember)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to queue approval decision"))?;
    tx.commit().await.map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to queue approval decision"))?;

    Ok(StatusCode::ACCEPTED)
}

pub async fn delete_message_feedback(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path((session_id, message_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, (StatusCode, &'static str)> {
    authorize_session_access(&state.pool, claims.sub, session_id).await?;
    authorize_message_in_session(&state.pool, session_id, message_id).await?;

    let previous = previous_rating(&state.pool, message_id, claims.sub).await?;
    sqlx::query("DELETE FROM message_feedback WHERE message_id = $1 AND user_id = $2")
        .bind(message_id)
        .bind(claims.sub)
        .execute(&state.pool)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "failed to delete message feedback");
            (StatusCode::INTERNAL_SERVER_ERROR, "failed to delete feedback")
        })?;
    reinforce_memories(&state.pool, message_id, previous.as_deref(), None).await;

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

#[derive(Serialize)]
pub struct AgentStatusResponse {
    pub agent_session_id: Uuid,
    pub agent_type: String,
    pub current_phase: String,
    pub current_phase_detail: Option<String>,
}

pub async fn get_agent_status(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(session_id): Path<Uuid>,
) -> Result<Json<Option<AgentStatusResponse>>, (StatusCode, &'static str)> {
    authorize_session_access(&state.pool, claims.sub, session_id).await?;

    let row: Option<(Uuid, String, String, Option<String>)> = sqlx::query_as(
        "SELECT id, agent_type, current_phase, current_phase_detail FROM agent_sessions \
         WHERE session_id = $1 AND status = 'active' ORDER BY started_at DESC LIMIT 1",
    )
    .bind(session_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "failed to load agent status");
        (StatusCode::INTERNAL_SERVER_ERROR, "failed to load agent status")
    })?;

    Ok(Json(row.map(|(agent_session_id, agent_type, current_phase, current_phase_detail)| AgentStatusResponse {
        agent_session_id,
        agent_type,
        current_phase,
        current_phase_detail,
    })))
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

const THINKING_LEVELS: [&str; 4] = ["off", "low", "medium", "high"];

#[derive(Serialize, Deserialize)]
pub struct ThinkingLevel {
    pub level: String,
}

/// The chat's thinking level (see migration 0034).
pub async fn get_thinking_level(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(session_id): Path<Uuid>,
) -> Result<Json<ThinkingLevel>, (StatusCode, &'static str)> {
    authorize_session_access(&state.pool, claims.sub, session_id).await?;
    let level: String = sqlx::query_scalar("SELECT thinking_level FROM sessions WHERE id = $1")
        .bind(session_id)
        .fetch_one(&state.pool)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to load thinking level"))?;
    Ok(Json(ThinkingLevel { level }))
}

pub async fn set_thinking_level(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(session_id): Path<Uuid>,
    Json(req): Json<ThinkingLevel>,
) -> Result<Json<ThinkingLevel>, (StatusCode, &'static str)> {
    authorize_session_access(&state.pool, claims.sub, session_id).await?;
    if !THINKING_LEVELS.contains(&req.level.as_str()) {
        return Err((StatusCode::BAD_REQUEST, "level must be off, low, medium or high"));
    }
    sqlx::query("UPDATE sessions SET thinking_level = $1 WHERE id = $2")
        .bind(&req.level)
        .bind(session_id)
        .execute(&state.pool)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to save thinking level"))?;
    Ok(Json(req))
}

#[derive(Serialize)]
pub struct UsedMemory {
    pub id: Uuid,
    pub content: String,
    pub kind: String,
    /// Replaced, merged or faded since the reply: no longer recalled.
    pub archived: bool,
}

#[derive(Serialize)]
pub struct UsedMemoriesResponse {
    pub memories: Vec<UsedMemory>,
}

/// The memories a reply drew on, so the person can see why Nomi said it, and forget or fix one.
pub async fn list_message_memories(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path((session_id, message_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<UsedMemoriesResponse>, (StatusCode, &'static str)> {
    authorize_session_access(&state.pool, claims.sub, session_id).await?;
    authorize_message_in_session(&state.pool, session_id, message_id).await?;
    let rows: Vec<(Uuid, String, String, bool)> = sqlx::query_as(
        "SELECT mi.id, mi.content, mi.kind, mi.archived_at IS NOT NULL FROM message_memory_usage u \
         JOIN memory_items mi ON mi.id = u.memory_id \
         WHERE u.message_id = $1 AND mi.user_id = $2 ORDER BY mi.weight DESC",
    )
    .bind(message_id)
    .bind(claims.sub)
    .fetch_all(&state.pool)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "failed to load memories"))?;
    Ok(Json(UsedMemoriesResponse {
        memories: rows.into_iter().map(|(id, content, kind, archived)| UsedMemory { id, content, kind, archived }).collect(),
    }))
}

#[cfg(test)]
mod plan_item_tests {
    use super::set_plan_item_done;

    const PLAN: &str = "# Bali\n\n- [x] Pick dates\n- [ ] Book flights\n  * [ ] Compare airlines\n- not a task\n- [ ]\n+ [X] Budget";

    #[test]
    fn ticks_the_nth_item_and_leaves_the_rest() {
        let updated = set_plan_item_done(PLAN, 1, true).unwrap();
        assert_eq!(updated, PLAN.replace("- [ ] Book flights", "- [x] Book flights"));
    }

    #[test]
    fn unticks_and_counts_nested_and_other_bullets() {
        assert_eq!(set_plan_item_done(PLAN, 2, true).unwrap(), PLAN.replace("* [ ] Compare", "* [x] Compare"));
        assert_eq!(set_plan_item_done(PLAN, 3, false).unwrap(), PLAN.replace("+ [X] Budget", "+ [ ] Budget"));
        assert_eq!(set_plan_item_done(PLAN, 0, false).unwrap(), PLAN.replace("- [x] Pick", "- [ ] Pick"));
    }

    #[test]
    fn no_such_item_is_none() {
        assert_eq!(set_plan_item_done(PLAN, 4, true), None);
        assert_eq!(set_plan_item_done("no tasks", 0, true), None);
    }
}
