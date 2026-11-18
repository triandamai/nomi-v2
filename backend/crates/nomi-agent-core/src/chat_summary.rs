//! "Summarise and continue": the person asks for a long chat's older messages to be folded into
//! one summary message and carries on from it. The summarised messages stay in the chat but are
//! no longer sent to the model (`messages.summarized_by`); the summary takes their place in the
//! history, just before the newest messages, which are kept as they were.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use nomi_llm::{ContentBlock, LlmMessage, LlmProvider, LlmRequest, LlmRole};

use crate::content_block::ContentBlock as RichBlock;
use crate::working_memory::SAID_IN_CHAT;

/// The newest messages that stay as they were; only what's older is summarised.
pub const KEEP_RECENT_MESSAGES: usize = 6;
/// Each message's share of the summariser's input.
const MESSAGE_CHARS: usize = 3000;
const SUMMARY_MAX_TOKENS: u32 = 1536;
/// The most the summariser is given to read, in estimated tokens (see `nomi_embedding::estimate_tokens`),
/// well inside any model's context. When a chat has more, its oldest messages are left out first.
pub const SUMMARY_INPUT_MAX_TOKENS: u32 = 24_000;
/// The kept messages come just after the summary: it is placed this many microseconds before
/// the first of them.
const SUMMARY_LEAD_MICROS: i64 = 1;

/// A message a summary may replace, oldest first.
#[derive(Debug, Clone, PartialEq)]
pub struct Candidate {
    pub id: Uuid,
    pub from_person: bool,
    pub agent: Option<String>,
    pub content: String,
    pub created_at: DateTime<Utc>,
    /// An earlier summary: it is folded into the new one.
    pub is_summary: bool,
}

/// `messages` (oldest first) split into what gets summarised and the newest `keep` that stay.
pub fn split_for_summary<T>(mut messages: Vec<T>, keep: usize) -> (Vec<T>, Vec<T>) {
    let kept = messages.split_off(messages.len().saturating_sub(keep));
    (messages, kept)
}

/// Whether summarising `older` would fold in any real message (not only an earlier summary).
pub fn worth_summarising(older: &[Candidate]) -> bool {
    older.iter().any(|m| !m.is_summary)
}

/// The text the summariser is given: the earlier summary, if any, then the messages.
pub fn summary_input(older: &[Candidate]) -> String {
    let clip = |text: &str| text.chars().take(MESSAGE_CHARS).collect::<String>();
    let earlier: Vec<String> = older.iter().filter(|m| m.is_summary).map(|m| clip(&m.content)).collect();
    let head = format!("Summary so far:\n{}\n\nMessages to fold in:\n", if earlier.is_empty() { "(none yet)".to_string() } else { earlier.join("\n") });
    let lines: Vec<String> = older
        .iter()
        .filter(|m| !m.is_summary)
        .map(|m| {
            let who = match (m.from_person, &m.agent) {
                (true, _) => "Person".to_string(),
                (false, Some(agent)) => format!("Assistant ({agent})"),
                (false, None) => "Assistant".to_string(),
            };
            format!("{who}: {}\n", clip(&m.content))
        })
        .collect();
    // Newest first until the budget is used, so a long chat loses its oldest messages.
    let mut budget = SUMMARY_INPUT_MAX_TOKENS.saturating_sub(nomi_embedding::estimate_tokens(&head) + nomi_embedding::estimate_tokens(crate::prompts::SUMMARY_INPUT_TRIMMED_NOTE));
    let mut kept_from = lines.len();
    for (at, line) in lines.iter().enumerate().rev() {
        let cost = nomi_embedding::estimate_tokens(line);
        if cost > budget {
            break;
        }
        budget -= cost;
        kept_from = at;
    }
    let mut input = head;
    if kept_from > 0 {
        input.push_str(crate::prompts::SUMMARY_INPUT_TRIMMED_NOTE);
        input.push('\n');
    }
    for line in &lines[kept_from..] {
        input.push_str(line);
    }
    input
}

/// A message's text as the model reads it in the history: a summary is introduced as one.
pub fn history_text(content: &str, is_summary: bool) -> String {
    if is_summary {
        crate::prompts::SUMMARY_IN_HISTORY.replace("{summary}", content)
    } else {
        content.to_string()
    }
}

/// The chat's messages that could be summarised, oldest first: what was said and isn't already
/// replaced by a summary (an earlier summary counts, and is folded into the new one).
async fn candidates(pool: &PgPool, session_id: Uuid) -> Result<Vec<Candidate>, sqlx::Error> {
    let rows: Vec<(Uuid, Option<Uuid>, Option<String>, String, DateTime<Utc>, bool)> = sqlx::query_as(&format!(
        "SELECT id, sender_channel_identity_id, agent_display_name, content, created_at, COALESCE(content_blocks->0->>'kind', '') = 'summary' \
         FROM messages WHERE session_id = $1 AND {SAID_IN_CHAT} AND summarized_by IS NULL ORDER BY created_at ASC, id ASC"
    ))
    .bind(session_id)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|(id, sender, agent, content, created_at, is_summary)| Candidate { id, from_person: sender.is_some(), agent, content, created_at, is_summary })
        .collect())
}

/// Whether the chat has older messages a summary could replace.
pub async fn has_older_messages(pool: &PgPool, session_id: Uuid) -> Result<bool, sqlx::Error> {
    let (older, _) = split_for_summary(candidates(pool, session_id).await?, KEEP_RECENT_MESSAGES);
    Ok(worth_summarising(&older))
}

/// Writes the summary of the chat's older messages as a message of its own, placed just before
/// the messages kept, and marks the ones it replaces. The model that answers the chat's turns
/// writes it (the caller's `provider`, which meters the call like any other).
pub async fn summarize_chat(pool: &PgPool, provider: &dyn LlmProvider, session_id: Uuid) -> Result<Summarized, SummarizeError> {
    let (older, kept) = split_for_summary(candidates(pool, session_id).await?, KEEP_RECENT_MESSAGES);
    let Some(first_kept) = kept.first().map(|m| m.created_at) else { return Ok(Summarized::NothingToSummarise) };
    if !worth_summarising(&older) {
        return Ok(Summarized::NothingToSummarise);
    }

    let request = LlmRequest {
        system: Some(crate::prompts::SUMMARIZE_CHAT_SYSTEM_PROMPT.to_string()),
        messages: vec![LlmMessage { role: LlmRole::User, content: vec![ContentBlock::Text { text: summary_input(&older) }] }],
        tools: vec![],
        max_tokens: SUMMARY_MAX_TOKENS,
        enable_reasoning: false,
        reasoning_effort: Default::default(),
    };
    let response = nomi_llm::complete(provider, request).await.map_err(SummarizeError::Model)?;
    let text = response
        .content
        .into_iter()
        .find_map(|block| match block {
            ContentBlock::Text { text } => Some(text.trim().to_string()),
            _ => None,
        })
        .filter(|t| !t.is_empty())
        .ok_or_else(|| SummarizeError::Model(nomi_llm::LlmError::ParseError("the model's answer was empty".to_string())))?;

    let replaced = older.iter().filter(|m| !m.is_summary).count() as u32;
    let blocks = serde_json::json!([RichBlock::Summary { replaced }]);
    let mut tx = pool.begin().await?;
    let message_id: Uuid = sqlx::query_scalar(
        "INSERT INTO messages (session_id, sender_channel_identity_id, content, content_blocks, created_at) \
         VALUES ($1, NULL, $2, $3, $4 - make_interval(secs => $5 / 1000000.0)) RETURNING id",
    )
    .bind(session_id)
    .bind(&text)
    .bind(&blocks)
    .bind(first_kept)
    .bind(SUMMARY_LEAD_MICROS as f64)
    .fetch_one(&mut *tx)
    .await?;
    let ids: Vec<Uuid> = older.iter().map(|m| m.id).collect();
    sqlx::query("UPDATE messages SET summarized_by = $2 WHERE id = ANY($1)").bind(&ids).bind(message_id).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(Summarized::Done { message_id, replaced })
}

#[derive(Debug, thiserror::Error)]
pub enum SummarizeError {
    #[error(transparent)]
    Db(#[from] sqlx::Error),
    #[error("the model couldn't write the summary: {0}")]
    Model(nomi_llm::LlmError),
}

impl SummarizeError {
    /// The failure as a run failure, so it shows in the chat like any other.
    pub fn into_turn_error(self) -> crate::TurnError {
        match self {
            SummarizeError::Db(e) => crate::TurnError::Db(e),
            SummarizeError::Model(e) => crate::TurnError::LlmCallFailed(e),
        }
    }
}

/// What a summary did.
#[derive(Debug, Clone, PartialEq)]
pub enum Summarized {
    Done { message_id: Uuid, replaced: u32 },
    NothingToSummarise,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(minute: i64) -> DateTime<Utc> {
        DateTime::from_timestamp(1_700_000_000 + minute * 60, 0).unwrap()
    }

    fn candidate(n: i64, is_summary: bool) -> Candidate {
        Candidate { id: Uuid::new_v4(), from_person: n % 2 == 0, agent: None, content: format!("message {n}"), created_at: at(n), is_summary }
    }

    #[test]
    fn the_newest_messages_are_kept_and_the_older_ones_summarised() {
        let (older, kept) = split_for_summary((0..10).collect::<Vec<_>>(), 4);
        assert_eq!(older, vec![0, 1, 2, 3, 4, 5]);
        assert_eq!(kept, vec![6, 7, 8, 9]);

        let (older, kept) = split_for_summary(vec![1, 2], 4);
        assert!(older.is_empty());
        assert_eq!(kept, vec![1, 2]);
    }

    #[test]
    fn only_an_earlier_summary_behind_the_kept_messages_is_not_worth_summarising() {
        assert!(!worth_summarising(&[]));
        assert!(!worth_summarising(&[candidate(0, true)]));
        assert!(worth_summarising(&[candidate(0, true), candidate(1, false)]));
    }

    #[test]
    fn the_summariser_reads_the_earlier_summary_then_who_said_what() {
        let mut agent = candidate(3, false);
        agent.agent = Some("Dana".into());
        let input = summary_input(&[candidate(0, true), candidate(2, false), agent]);
        let earlier = input.find("message 0").unwrap();
        let person = input.find("Person: message 2").unwrap();
        let dana = input.find("Assistant (Dana): message 3").unwrap();
        assert!(earlier < person && person < dana);
        assert!(!input.contains("Person: message 0"));
    }

    #[test]
    fn a_message_is_clipped_in_the_summariser_input() {
        let mut long = candidate(1, false);
        long.content = "x".repeat(MESSAGE_CHARS + 500);
        let input = summary_input(&[long]);
        assert!(input.chars().filter(|c| *c == 'x').count() == MESSAGE_CHARS);
    }

    #[test]
    fn a_long_chat_is_cut_to_the_input_budget_oldest_first() {
        let many: Vec<Candidate> = (0..400)
            .map(|n| {
                let mut c = candidate(n, false);
                c.content = format!("m{n} {}", "x".repeat(MESSAGE_CHARS));
                c
            })
            .collect();
        let input = summary_input(&many);
        assert!(nomi_embedding::estimate_tokens(&input) <= SUMMARY_INPUT_MAX_TOKENS);
        assert!(input.contains("m399 "));
        assert!(!input.contains("m0 "));
        assert!(input.contains(crate::prompts::SUMMARY_INPUT_TRIMMED_NOTE));
    }

    #[test]
    fn a_chat_within_the_budget_is_not_cut() {
        let input = summary_input(&[candidate(0, false), candidate(1, false)]);
        assert!(input.contains("message 0") && input.contains("message 1"));
        assert!(!input.contains(crate::prompts::SUMMARY_INPUT_TRIMMED_NOTE));
    }

    #[test]
    fn a_summary_is_introduced_in_the_history_and_other_messages_are_not() {
        assert_eq!(history_text("hello", false), "hello");
        let text = history_text("Trip to Bali in May.", true);
        assert!(text.contains("Trip to Bali in May."));
        assert!(text.contains("Summary of the earlier part"));
    }
}
