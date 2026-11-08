//! What each person did lately, across all their chats: one short note (`users.recent_activity`)
//! that every agent reads, so a new chat knows the person logged lunch with Dana an hour ago or
//! has a build waiting with Koda.
//!
//! The memory worker rewrites it once a chat has gone quiet for [`QUIET_FOR`], from the note so
//! far and that chat's messages since it was last noted (`sessions.activity_noted_through`):
//! one model call for a whole burst of messages, never one per reply. Each line says when, and
//! which agent worked on it. It isn't a memory: it's replaced each time and never recalled by
//! similarity, so passing things don't crowd out what stays true about the person.

use sqlx::PgPool;
use uuid::Uuid;

use nomi_llm::{ContentBlock, LlmMessage, LlmProvider, LlmRequest, LlmRole};

use crate::working_memory::SAID_IN_CHAT;

/// How long a chat has to be quiet before what happened in it goes into the note.
pub const QUIET_FOR: &str = "5 minutes";
/// Chats quiet for longer than this are left alone (so turning this on doesn't note old chats).
pub const LOOK_BACK: &str = "1 day";
/// The most new messages read from one chat per update.
pub const MESSAGES_PER_UPDATE: i64 = 40;
/// Each message's share of the input.
const MESSAGE_CHARS: usize = 800;
const NOTE_MAX_TOKENS: u32 = 600;

/// A message as the note reads it: who sent it (a person's identity, else the agent's name), what
/// they said, and when.
type SaidMessage = (Option<Uuid>, Option<String>, String, chrono::DateTime<chrono::Utc>);

/// What one update did.
#[derive(Debug, PartialEq, Eq)]
pub enum Noted {
    /// Nothing new was said in the chat since it was last noted.
    Nothing,
    /// The note was rewritten from `messages` new messages.
    Updated { messages: usize, chars: usize },
    /// The model failed or answered nothing; the chat is tried again on the next sweep.
    ModelFailed(String),
}

/// Chats that went quiet with something said since they were last noted, and whose they are.
/// Oldest first, so a backlog catches up in order.
pub async fn chats_gone_quiet(pool: &PgPool, limit: i64) -> Result<Vec<(Uuid, Uuid)>, sqlx::Error> {
    sqlx::query_as(&format!(
        "SELECT s.id, s.user_id FROM sessions s \
         CROSS JOIN LATERAL (SELECT max(created_at) AS at FROM messages WHERE session_id = s.id AND {SAID_IN_CHAT}) last \
         WHERE s.user_id IS NOT NULL \
           AND last.at > COALESCE(s.activity_noted_through, '-infinity'::timestamptz) \
           AND last.at < now() - $2::interval \
           AND last.at > now() - $3::interval \
         ORDER BY last.at ASC LIMIT $1"
    ))
    .bind(limit)
    .bind(QUIET_FOR)
    .bind(LOOK_BACK)
    .fetch_all(pool)
    .await
}

/// Rewrites the person's note from what was said in the chat since it was last noted.
pub async fn note_chat(pool: &PgPool, provider: &dyn LlmProvider, session_id: Uuid, user_id: Uuid) -> Result<Noted, sqlx::Error> {
    let (title, through): (Option<String>, Option<chrono::DateTime<chrono::Utc>>) =
        sqlx::query_as("SELECT title, activity_noted_through FROM sessions WHERE id = $1").bind(session_id).fetch_one(pool).await?;
    let new: Vec<SaidMessage> = sqlx::query_as(&format!(
        "SELECT sender_channel_identity_id, agent_display_name, content, created_at FROM ( \
             SELECT sender_channel_identity_id, agent_display_name, content, created_at, id FROM messages \
             WHERE session_id = $1 AND {SAID_IN_CHAT} AND created_at > COALESCE($2, '-infinity'::timestamptz) \
             ORDER BY created_at DESC, id DESC LIMIT $3 \
         ) newest ORDER BY created_at ASC, id ASC"
    ))
    .bind(session_id)
    .bind(through)
    .bind(MESSAGES_PER_UPDATE)
    .fetch_all(pool)
    .await?;
    let Some(last) = new.last().map(|m| m.3) else {
        return Ok(Noted::Nothing);
    };
    tracing::info!(%user_id, %session_id, messages = new.len(), "recent activity: noting a chat that went quiet");

    let note: Option<String> = sqlx::query_scalar("SELECT recent_activity FROM users WHERE id = $1").bind(user_id).fetch_one(pool).await?;
    let today = chrono::Utc::now().format("%Y-%m-%d");
    let mut input = format!(
        "Note so far:\n{}\n\nToday is {today}.\n\nNew messages in the chat \"{}\":\n",
        note.as_deref().filter(|n| !n.trim().is_empty()).unwrap_or("(none yet)"),
        title.as_deref().unwrap_or("untitled"),
    );
    for (sender, agent, content, at) in &new {
        let who = match (sender, agent) {
            (Some(_), _) => "Person".to_string(),
            (None, Some(agent)) => agent.clone(),
            (None, None) => "Nomi".to_string(),
        };
        let content: String = content.chars().take(MESSAGE_CHARS).collect();
        input.push_str(&format!("[{}] {who}: {content}\n", at.format("%Y-%m-%d %H:%M")));
    }

    let request = LlmRequest {
        system: Some(crate::prompts::RECENT_ACTIVITY_SYSTEM_PROMPT.to_string()),
        messages: vec![LlmMessage { role: LlmRole::User, content: vec![ContentBlock::Text { text: input }] }],
        tools: vec![],
        max_tokens: NOTE_MAX_TOKENS,
        enable_reasoning: false,
        reasoning_effort: Default::default(),
    };
    let response = match nomi_llm::complete(provider, request).await {
        Ok(response) => response,
        Err(e) => return Ok(Noted::ModelFailed(e.to_string())),
    };
    let text = response.content.into_iter().find_map(|block| match block {
        ContentBlock::Text { text } => Some(text),
        _ => None,
    });
    let Some(note) = text.map(|t| t.trim().to_string()).filter(|t| !t.is_empty()) else {
        return Ok(Noted::ModelFailed("the model answered nothing".to_string()));
    };

    let mut tx = pool.begin().await?;
    sqlx::query("UPDATE users SET recent_activity = $2, recent_activity_at = now() WHERE id = $1").bind(user_id).bind(&note).execute(&mut *tx).await?;
    sqlx::query("UPDATE sessions SET activity_noted_through = $2 WHERE id = $1").bind(session_id).bind(last).execute(&mut *tx).await?;
    tx.commit().await?;
    tracing::info!(%user_id, %session_id, note = %note, "recent activity: note updated");
    Ok(Noted::Updated { messages: new.len(), chars: note.chars().count() })
}

/// The person's note, if there is one, for an agent's system prompt.
pub async fn for_person<'c, E: sqlx::PgExecutor<'c>>(executor: E, user_id: Uuid) -> Result<Option<String>, sqlx::Error> {
    let note: Option<Option<String>> = sqlx::query_scalar("SELECT recent_activity FROM users WHERE id = $1").bind(user_id).fetch_optional(executor).await?;
    Ok(note.flatten().map(|n| n.trim().to_string()).filter(|n| !n.is_empty()))
}
