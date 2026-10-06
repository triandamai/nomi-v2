//! Working memory, kept tidy in the background by the memory worker:
//!
//! - **Chat summaries.** Only a chat's latest [`RECENT_MESSAGES`] are sent to the model in full.
//!   Once [`SWEEP_EVERY`] older messages have piled up behind them, they're folded into the
//!   chat's running summary (`sessions.summary`), which every agent sees in place of them.
//! - **Consolidation.** About once a day per person: near-identical memories are merged into the
//!   stronger one, memories nobody has used in a month slowly fade, and faint, long-unused ones
//!   are archived (kept, but out of recall).

use sqlx::PgPool;
use uuid::Uuid;

use nomi_llm::{ContentBlock, LlmMessage, LlmProvider, LlmRequest, LlmRole};

/// How many of a chat's latest messages agents see in full.
pub const RECENT_MESSAGES: i64 = 20;
/// How many older messages have to pile up before they're folded into the summary.
pub const SWEEP_EVERY: i64 = 20;
/// The most messages folded in one go (a long chat catches up over a few sweeps).
const FOLD_LIMIT: i64 = 60;
/// Each message's share of the summarizer's input.
const MESSAGE_CHARS: usize = 1500;
const SUMMARY_MAX_TOKENS: u32 = 1024;

/// What was actually said: shown thinking (🧠), progress notes (💭) and reasoning steps are a
/// window into the work, not part of the conversation.
pub const SAID_IN_CHAT: &str = "NOT (sender_channel_identity_id IS NULL AND (content LIKE '🧠%' OR content LIKE '💭%' \
     OR COALESCE(content_blocks->0->>'kind', '') = 'reasoning'))";

const SUMMARY_SYSTEM_PROMPT: &str = "You keep a running summary of a chat between a person and their \
assistant crew, so the chat can go on without its full history. Update the summary with the new \
messages. Keep what the person wants, decisions made, facts and names that matter, and open \
questions or unfinished tasks. Drop greetings, small talk, and anything settled that no longer \
matters. At most 200 words, as short plain lines, in the language of the chat. Reply with the \
summary only.";

/// Chats with enough unsummarized older messages to fold, and whose they are. Only chats active
/// in the past week are looked at.
pub async fn sessions_to_summarize(pool: &PgPool, limit: i64) -> Result<Vec<(Uuid, Uuid)>, sqlx::Error> {
    sqlx::query_as(&format!(
        "SELECT s.id, s.user_id FROM sessions s \
         WHERE s.user_id IS NOT NULL \
           AND EXISTS (SELECT 1 FROM messages r WHERE r.session_id = s.id AND r.created_at > now() - interval '7 days') \
           AND (SELECT count(*) FROM ( \
                    SELECT created_at FROM messages WHERE session_id = s.id AND {SAID_IN_CHAT} \
                    ORDER BY created_at DESC, id DESC OFFSET $1 \
                ) older WHERE older.created_at > COALESCE(s.summary_through, '-infinity'::timestamptz)) >= $2 \
         LIMIT $3"
    ))
    .bind(RECENT_MESSAGES)
    .bind(SWEEP_EVERY)
    .bind(limit)
    .fetch_all(pool)
    .await
}

/// Folds a chat's older, not yet summarized messages into its running summary. Returns whether
/// anything was folded. A failed model call leaves the summary as it was.
pub async fn summarize_session(pool: &PgPool, provider: &dyn LlmProvider, session_id: Uuid) -> Result<bool, sqlx::Error> {
    let (summary, through): (Option<String>, Option<chrono::DateTime<chrono::Utc>>) =
        sqlx::query_as("SELECT summary, summary_through FROM sessions WHERE id = $1").bind(session_id).fetch_one(pool).await?;
    let older: Vec<(Option<Uuid>, Option<String>, String, chrono::DateTime<chrono::Utc>)> = sqlx::query_as(&format!(
        "SELECT sender_channel_identity_id, agent_display_name, content, created_at FROM ( \
             SELECT sender_channel_identity_id, agent_display_name, content, created_at, id FROM messages \
             WHERE session_id = $1 AND {SAID_IN_CHAT} \
             ORDER BY created_at DESC, id DESC OFFSET $2 \
         ) older WHERE created_at > COALESCE($3, '-infinity'::timestamptz) \
         ORDER BY created_at ASC, id ASC LIMIT $4"
    ))
    .bind(session_id)
    .bind(RECENT_MESSAGES)
    .bind(through)
    .bind(FOLD_LIMIT)
    .fetch_all(pool)
    .await?;
    let Some(last) = older.last().map(|m| m.3) else {
        return Ok(false);
    };

    let mut input = format!("Summary so far:\n{}\n\nNew messages:\n", summary.as_deref().unwrap_or("(none yet)"));
    for (sender, agent, content, _) in &older {
        let who = match (sender, agent) {
            (Some(_), _) => "Person".to_string(),
            (None, Some(agent)) => format!("Assistant ({agent})"),
            (None, None) => "Assistant".to_string(),
        };
        let content: String = content.chars().take(MESSAGE_CHARS).collect();
        input.push_str(&format!("{who}: {content}\n"));
    }

    let request = LlmRequest {
        system: Some(SUMMARY_SYSTEM_PROMPT.to_string()),
        messages: vec![LlmMessage { role: LlmRole::User, content: vec![ContentBlock::Text { text: input }] }],
        tools: vec![],
        max_tokens: SUMMARY_MAX_TOKENS,
        enable_reasoning: false,
        reasoning_effort: Default::default(),
    };
    let response = match nomi_llm::complete(provider, request).await {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!(error = %e, %session_id, "working memory: summarizing the chat failed");
            return Ok(false);
        }
    };
    let text = response.content.into_iter().find_map(|block| match block {
        ContentBlock::Text { text } => Some(text),
        _ => None,
    });
    let Some(new_summary) = text.map(|t| t.trim().to_string()).filter(|t| !t.is_empty()) else {
        return Ok(false);
    };
    sqlx::query("UPDATE sessions SET summary = $2, summary_through = $3 WHERE id = $1")
        .bind(session_id)
        .bind(new_summary)
        .bind(last)
        .execute(pool)
        .await?;
    Ok(true)
}

/// Memories not used (nor confirmed) for this long start to fade...
const FADE_AFTER_DAYS: i32 = 30;
/// ...by this much per consolidation (about daily).
const FADE_FACTOR: f64 = 0.95;
/// Faint memories unused for this long are archived.
const ARCHIVE_AFTER_DAYS: i32 = 60;
const ARCHIVE_BELOW_WEIGHT: f64 = 0.15;
/// Memories this close (cosine distance) say the same thing and are merged.
const MERGE_DISTANCE: f64 = 0.12;
/// How often each person's memories are tidied.
const CONSOLIDATE_EVERY_HOURS: i32 = 20;

/// What one consolidation did.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Tidied {
    pub merged: u64,
    pub faded: u64,
    pub archived: u64,
}

/// People with live memories whose last tidy-up was long enough ago.
pub async fn people_to_consolidate(pool: &PgPool, limit: i64) -> Result<Vec<Uuid>, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT DISTINCT m.user_id FROM memory_items m LEFT JOIN memory_upkeep u ON u.user_id = m.user_id \
         WHERE m.archived_at IS NULL AND (u.consolidated_at IS NULL OR u.consolidated_at < now() - make_interval(hours => $1)) \
         LIMIT $2",
    )
    .bind(CONSOLIDATE_EVERY_HOURS)
    .bind(limit)
    .fetch_all(pool)
    .await
}

/// Tidies one person's memories: merges near-identical ones, fades unused ones, archives faint
/// ones. No model call, so it costs nothing.
pub async fn consolidate(pool: &PgPool, user_id: Uuid) -> Result<Tidied, sqlx::Error> {
    let mut tx = pool.begin().await?;
    let mut tidied = Tidied::default();

    // Merge: each pair, stronger (then older) memory first; the weaker one is retired into it.
    let pairs: Vec<(Uuid, Uuid)> = sqlx::query_as(
        "SELECT a.id, b.id FROM memory_items a JOIN memory_items b \
           ON b.user_id = a.user_id AND b.id <> a.id \
          AND b.embedding_provider = a.embedding_provider AND b.embedding_model = a.embedding_model \
         WHERE a.user_id = $1 AND a.archived_at IS NULL AND b.archived_at IS NULL \
           AND (a.embedding <=> b.embedding) < $2 \
           AND (a.weight, b.created_at, b.id) > (b.weight, a.created_at, a.id) \
         ORDER BY a.weight DESC, a.created_at ASC",
    )
    .bind(user_id)
    .bind(MERGE_DISTANCE)
    .fetch_all(&mut *tx)
    .await?;
    let mut retired = std::collections::HashSet::new();
    for (keep, drop) in pairs {
        if retired.contains(&keep) || retired.contains(&drop) {
            continue;
        }
        sqlx::query(
            "UPDATE memory_items k SET weight = LEAST(GREATEST(k.weight, d.weight) * 1.1, 5.0), \
                 last_used_at = GREATEST(k.last_used_at, d.last_used_at), updated_at = now() \
             FROM memory_items d WHERE k.id = $1 AND d.id = $2",
        )
        .bind(keep)
        .bind(drop)
        .execute(&mut *tx)
        .await?;
        sqlx::query("UPDATE memory_items SET archived_at = now(), superseded_by = $2, updated_at = now() WHERE id = $1")
            .bind(drop)
            .bind(keep)
            .execute(&mut *tx)
            .await?;
        retired.insert(drop);
        tidied.merged += 1;
    }

    tidied.faded = sqlx::query(
        "UPDATE memory_items SET weight = GREATEST(weight * $2, 0.1) \
         WHERE user_id = $1 AND archived_at IS NULL \
           AND COALESCE(last_used_at, confirmed_at, created_at) < now() - make_interval(days => $3)",
    )
    .bind(user_id)
    .bind(FADE_FACTOR)
    .bind(FADE_AFTER_DAYS)
    .execute(&mut *tx)
    .await?
    .rows_affected();

    tidied.archived = sqlx::query(
        "UPDATE memory_items SET archived_at = now() \
         WHERE user_id = $1 AND archived_at IS NULL AND weight <= $2 \
           AND COALESCE(last_used_at, confirmed_at, created_at) < now() - make_interval(days => $3)",
    )
    .bind(user_id)
    .bind(ARCHIVE_BELOW_WEIGHT)
    .bind(ARCHIVE_AFTER_DAYS)
    .execute(&mut *tx)
    .await?
    .rows_affected();

    sqlx::query(
        "INSERT INTO memory_upkeep (user_id, consolidated_at) VALUES ($1, now()) \
         ON CONFLICT (user_id) DO UPDATE SET consolidated_at = now()",
    )
    .bind(user_id)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(tidied)
}
