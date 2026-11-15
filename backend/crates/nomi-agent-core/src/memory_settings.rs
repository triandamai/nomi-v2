//! What a person lets Nomi do with memory (`memory_settings`), and the profile Nomi keeps of them
//! (`memory_profiles`).
//!
//! - `learn_from_chats` off: replies queue nothing to learn from, and queued exchanges are dropped.
//! - `keep_profile` off: no agent reads the profile and the memory worker stops rewriting it.
//! - `search_past_chats` off: agents don't get the "recently" note from the person's other chats,
//!   and the memory worker stops writing it.
//! - `memory_budget`: tokens of live memories kept. Saving past it retires the weakest memories;
//!   recall never hands an agent more than it.
//! - `profile_budget`: tokens of profile kept and read.

use serde::Serialize;
use sqlx::PgExecutor;
use uuid::Uuid;

use nomi_llm::{ContentBlock, LlmMessage, LlmProvider, LlmRequest, LlmRole};

/// Default tokens of live memories kept.
pub const DEFAULT_MEMORY_BUDGET: i32 = 8_000;
/// Default tokens of profile kept.
pub const DEFAULT_PROFILE_BUDGET: i32 = 1_000;
pub const MIN_MEMORY_BUDGET: i32 = 500;
pub const MAX_MEMORY_BUDGET: i32 = 100_000;
pub const MIN_PROFILE_BUDGET: i32 = 100;
pub const MAX_PROFILE_BUDGET: i32 = 4_000;
/// The rough size of a token, as `nomi_embedding::estimate_tokens` counts it.
pub const CHARS_PER_TOKEN: i32 = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct MemorySettings {
    pub learn_from_chats: bool,
    pub keep_profile: bool,
    pub search_past_chats: bool,
    pub memory_budget: i32,
    pub profile_budget: i32,
}

impl Default for MemorySettings {
    fn default() -> Self {
        Self {
            learn_from_chats: true,
            keep_profile: true,
            search_past_chats: true,
            memory_budget: DEFAULT_MEMORY_BUDGET,
            profile_budget: DEFAULT_PROFILE_BUDGET,
        }
    }
}

/// About how many tokens `text` is (~4 characters each; 0 for empty text).
pub fn tokens(text: &str) -> i32 {
    let chars = text.chars().count() as i32;
    (chars + CHARS_PER_TOKEN - 1) / CHARS_PER_TOKEN
}

/// `text` cut to fit `budget` tokens, at a line or word break where there is one.
pub fn fit_to_budget(text: &str, budget: i32) -> String {
    let text = text.trim();
    let max_chars = (budget.max(0) * CHARS_PER_TOKEN) as usize;
    if text.chars().count() <= max_chars {
        return text.to_string();
    }
    let cut: String = text.chars().take(max_chars).collect();
    let at = cut
        .rfind('\n')
        .or_else(|| cut.rfind(' '))
        .filter(|i| *i > 0)
        .unwrap_or(cut.len());
    cut[..at].trim_end().to_string()
}

/// The person's settings (the defaults when they never changed any).
pub async fn load<'c, E: PgExecutor<'c>>(executor: E, user_id: Uuid) -> MemorySettings {
    let row: Option<(bool, bool, bool, i32, i32)> = sqlx::query_as(
        "SELECT learn_from_chats, keep_profile, search_past_chats, memory_budget, profile_budget FROM memory_settings WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_optional(executor)
    .await
    .ok()
    .flatten();
    match row {
        Some((
            learn_from_chats,
            keep_profile,
            search_past_chats,
            memory_budget,
            profile_budget,
        )) => MemorySettings {
            learn_from_chats,
            keep_profile,
            search_past_chats,
            memory_budget,
            profile_budget,
        },
        None => MemorySettings::default(),
    }
}

/// Saves the person's settings whole.
pub async fn save<'c, E: PgExecutor<'c>>(
    executor: E,
    user_id: Uuid,
    s: &MemorySettings,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO memory_settings (user_id, learn_from_chats, keep_profile, search_past_chats, memory_budget, profile_budget, updated_at) \
         VALUES ($1, $2, $3, $4, $5, $6, now()) \
         ON CONFLICT (user_id) DO UPDATE SET learn_from_chats = $2, keep_profile = $3, search_past_chats = $4, \
             memory_budget = $5, profile_budget = $6, updated_at = now()",
    )
    .bind(user_id)
    .bind(s.learn_from_chats)
    .bind(s.keep_profile)
    .bind(s.search_past_chats)
    .bind(s.memory_budget)
    .bind(s.profile_budget)
    .execute(executor)
    .await?;
    Ok(())
}

/// SQL for a memory's size in tokens, matching [`tokens`].
const MEMORY_TOKENS_SQL: &str = "ceil(char_length(content) / 4.0)::int";

/// Tokens the person's live memories take.
pub async fn memory_tokens<'c, E: PgExecutor<'c>>(
    executor: E,
    user_id: Uuid,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(&format!("SELECT COALESCE(sum({MEMORY_TOKENS_SQL}), 0)::bigint FROM memory_items WHERE user_id = $1 AND archived_at IS NULL"))
        .bind(user_id)
        .fetch_one(executor)
        .await
}

/// Keeps the person's live memories within their budget: past it, the weakest (then oldest)
/// are archived (kept, but out of recall). Returns how many were archived.
pub async fn enforce_memory_budget<'c, E: PgExecutor<'c>>(
    executor: E,
    user_id: Uuid,
    budget: i32,
) -> Result<u64, sqlx::Error> {
    Ok(sqlx::query(&format!(
        "UPDATE memory_items SET archived_at = now(), updated_at = now() WHERE id IN ( \
             SELECT id FROM ( \
                 SELECT id, sum({MEMORY_TOKENS_SQL}) OVER (ORDER BY weight DESC, updated_at DESC, id) AS running \
                 FROM memory_items WHERE user_id = $1 AND archived_at IS NULL \
             ) ranked WHERE running > $2)"
    ))
    .bind(user_id)
    .bind(budget)
    .execute(executor)
    .await?
    .rows_affected())
}

/// The profile Nomi keeps of the person.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Profile {
    pub text: String,
    pub tokens: i32,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

pub async fn profile<'c, E: PgExecutor<'c>>(
    executor: E,
    user_id: Uuid,
) -> Result<Profile, sqlx::Error> {
    let row: Option<(String, chrono::DateTime<chrono::Utc>)> =
        sqlx::query_as("SELECT text, updated_at FROM memory_profiles WHERE user_id = $1")
            .bind(user_id)
            .fetch_optional(executor)
            .await?;
    Ok(match row {
        Some((text, updated_at)) => Profile {
            tokens: tokens(&text),
            text,
            updated_at: Some(updated_at),
        },
        None => Profile {
            text: String::new(),
            tokens: 0,
            updated_at: None,
        },
    })
}

/// Sets the profile's text (the person's own words: it's marked edited). Empty text clears it.
pub async fn set_profile<'c, E: PgExecutor<'c>>(
    executor: E,
    user_id: Uuid,
    text: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO memory_profiles (user_id, text, updated_at, edited_at) VALUES ($1, $2, now(), now()) \
         ON CONFLICT (user_id) DO UPDATE SET text = $2, updated_at = now(), edited_at = now()",
    )
    .bind(user_id)
    .bind(text.trim())
    .execute(executor)
    .await?;
    Ok(())
}

/// What an agent reads of the profile: nothing when the person turned it off or it's empty;
/// otherwise the text, within its budget.
pub async fn profile_for_agents(
    conn: &mut sqlx::PgConnection,
    user_id: Uuid,
    settings: &MemorySettings,
) -> Option<String> {
    if !settings.keep_profile {
        return None;
    }
    let profile = profile(&mut *conn, user_id).await.ok()?;
    let text = fit_to_budget(&profile.text, settings.profile_budget);
    (!text.is_empty()).then_some(text)
}

/// How often a profile is rewritten at most.
pub const PROFILE_REFRESH_EVERY_HOURS: i32 = 20;
/// The strongest memories a profile is written from.
const PROFILE_MEMORIES: i64 = 40;
/// Room for the rewrite (reasoning models think first).
const PROFILE_MAX_TOKENS: u32 = 2048;

/// People whose profile is due a rewrite: they keep one, have memories that changed since it was
/// last written, and it wasn't written in the past [`PROFILE_REFRESH_EVERY_HOURS`].
pub async fn profiles_to_refresh(
    pool: &sqlx::PgPool,
    limit: i64,
) -> Result<Vec<Uuid>, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT m.user_id FROM memory_items m \
         LEFT JOIN memory_settings s ON s.user_id = m.user_id \
         LEFT JOIN memory_profiles p ON p.user_id = m.user_id \
         WHERE m.archived_at IS NULL AND COALESCE(s.keep_profile, TRUE) \
           AND (p.updated_at IS NULL OR p.updated_at < now() - make_interval(hours => $1)) \
         GROUP BY m.user_id, p.updated_at \
         HAVING p.updated_at IS NULL OR max(m.updated_at) > p.updated_at \
         LIMIT $2",
    )
    .bind(PROFILE_REFRESH_EVERY_HOURS)
    .bind(limit)
    .fetch_all(pool)
    .await
}

/// Rewrites the person's profile from the profile so far (which keeps their own edits) and their
/// strongest memories, within their budget. Returns whether it changed; a failed model call
/// leaves it as it was.
pub async fn refresh_profile(
    pool: &sqlx::PgPool,
    provider: &dyn LlmProvider,
    user_id: Uuid,
) -> Result<bool, sqlx::Error> {
    let settings = load(pool, user_id).await;
    if !settings.keep_profile {
        return Ok(false);
    }
    let current = profile(pool, user_id).await?;
    let memories: Vec<(String, String)> = sqlx::query_as(
        "SELECT kind, content FROM memory_items WHERE user_id = $1 AND archived_at IS NULL ORDER BY weight DESC, updated_at DESC LIMIT $2",
    )
    .bind(user_id)
    .bind(PROFILE_MEMORIES)
    .fetch_all(pool)
    .await?;
    if memories.is_empty() {
        return Ok(false);
    }
    let mut input = format!(
        "Profile so far:\n{}\n\nAt most {} words.\n\nMemories, strongest first:\n",
        if current.text.is_empty() {
            "(none yet)"
        } else {
            current.text.as_str()
        },
        // ~0.75 words per token.
        settings.profile_budget * 3 / 4,
    );
    for (kind, content) in &memories {
        input.push_str(&format!("- ({kind}) {content}\n"));
    }
    let request = LlmRequest {
        system: Some(crate::prompts::MEMORY_PROFILE_SYSTEM_PROMPT.to_string()),
        messages: vec![LlmMessage {
            role: LlmRole::User,
            content: vec![ContentBlock::Text { text: input }],
        }],
        tools: vec![],
        max_tokens: PROFILE_MAX_TOKENS,
        enable_reasoning: false,
        reasoning_effort: Default::default(),
    };
    let response = match nomi_llm::complete(provider, request).await {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!(error = %e, %user_id, "memory profile: rewriting failed");
            return Ok(false);
        }
    };
    let text = response.content.into_iter().find_map(|b| match b {
        ContentBlock::Text { text } => Some(text),
        _ => None,
    });
    let Some(text) = text
        .map(|t| fit_to_budget(&t, settings.profile_budget))
        .filter(|t| !t.is_empty())
    else {
        return Ok(false);
    };
    sqlx::query(
        "INSERT INTO memory_profiles (user_id, text, updated_at) VALUES ($1, $2, now()) \
         ON CONFLICT (user_id) DO UPDATE SET text = $2, updated_at = now()",
    )
    .bind(user_id)
    .bind(&text)
    .execute(pool)
    .await?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokens_are_about_four_characters() {
        assert_eq!(tokens(""), 0);
        assert_eq!(tokens("abcd"), 1);
        assert_eq!(tokens("abcde"), 2);
    }

    #[test]
    fn text_is_cut_to_its_budget_at_a_break() {
        assert_eq!(fit_to_budget("  short  ", 10), "short");
        assert_eq!(fit_to_budget("one two three four", 3), "one two");
        assert_eq!(fit_to_budget("line one\nline two", 3), "line one");
        assert_eq!(fit_to_budget("abcdefghij", 2), "abcdefgh");
    }
}
