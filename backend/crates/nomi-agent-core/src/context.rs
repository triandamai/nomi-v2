//! Keeping what each model call carries in check.
//!
//! A turn's tool loop resends everything said so far on every round: a long build (read a
//! file, write it, run the build, read the error, ...) grows each request by every earlier
//! result. [`compact_tool_results`] keeps the latest rounds whole and trims older tool results
//! once the conversation passes a budget. It trims everything old at once, and leaves trimmed
//! results as they are, so the request's prefix only changes now and then (a provider's prompt
//! cache keeps serving it in between).
//!
//! Earlier turns are bounded elsewhere: only the chat's latest messages are sent in full
//! (`working_memory::RECENT_MESSAGES`, older ones as a running summary), and
//! [`clip_older_message`] shortens long ones among those that aren't the newest few.

use nomi_llm::{ContentBlock, LlmMessage, LlmRole};

/// About 30k tokens of conversation before older tool results are trimmed.
pub const TURN_BUDGET_CHARS: usize = 120_000;
/// Rounds of tool calls (an assistant message with tool calls and its results) kept whole.
pub const KEEP_RECENT_ROUNDS: usize = 4;
/// A trimmed tool result keeps this much of its start.
const TRIMMED_KEEP_CHARS: usize = 600;
/// Tool results shorter than this are left alone.
const TRIM_OVER_CHARS: usize = 1_500;
/// The marker a trimmed result ends with (also how an already-trimmed one is recognized).
const TRIMMED_MARK: &str = "[Trimmed to save room:";

/// Roughly how much text a conversation carries (files sent as media aren't counted: they're
/// only ever in the newest message).
pub fn conversation_chars(messages: &[LlmMessage]) -> usize {
    messages
        .iter()
        .flat_map(|m| &m.content)
        .map(|block| match block {
            ContentBlock::Text { text } => text.len(),
            ContentBlock::ToolResult { content, .. } => content.len(),
            ContentBlock::ToolUse { input, .. } => input.to_string().len(),
            ContentBlock::Thinking { text, .. } => text.len(),
            ContentBlock::Media { .. } => 0,
        })
        .sum()
}

/// Trims tool results older than the last `keep_rounds` rounds once the conversation is over
/// `budget_chars`. Returns how many were trimmed.
pub fn compact_tool_results(messages: &mut [LlmMessage], budget_chars: usize, keep_rounds: usize) -> usize {
    if conversation_chars(messages) <= budget_chars {
        return 0;
    }
    // Where the kept rounds start: the `keep_rounds`-th newest assistant message that called tools.
    let rounds: Vec<usize> = messages
        .iter()
        .enumerate()
        .filter(|(_, m)| m.role == LlmRole::Assistant && m.content.iter().any(|b| matches!(b, ContentBlock::ToolUse { .. })))
        .map(|(i, _)| i)
        .collect();
    let Some(&kept_from) = rounds.len().checked_sub(keep_rounds).and_then(|i| rounds.get(i)) else {
        return 0;
    };
    let mut trimmed = 0;
    for message in &mut messages[..kept_from] {
        for block in &mut message.content {
            if let ContentBlock::ToolResult { content, .. } = block {
                if content.len() > TRIM_OVER_CHARS && !content.contains(TRIMMED_MARK) {
                    *content = trim(content);
                    trimmed += 1;
                }
            }
        }
    }
    trimmed
}

fn trim(content: &str) -> String {
    let kept: String = content.chars().take(TRIMMED_KEEP_CHARS).collect();
    let dropped = content.chars().count() - kept.chars().count();
    format!("{kept}\n…\n{TRIMMED_MARK} {dropped} more characters of an earlier tool result. Call the tool again if you need it in full.]")
}

/// A message among the chat's earlier ones, shortened to `max_chars` with a note saying so.
/// Messages carrying attachment tags are left whole (the tags are read later).
pub fn clip_older_message(text: String, max_chars: usize) -> String {
    if text.chars().count() <= max_chars || text.contains("<attachment") {
        return text;
    }
    let kept: String = text.chars().take(max_chars).collect();
    format!("{kept}\n…[This earlier message was longer; the rest is left out to save room.]")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn round(id: &str, result: &str) -> [LlmMessage; 2] {
        [
            LlmMessage { role: LlmRole::Assistant, content: vec![ContentBlock::ToolUse { id: id.into(), name: "read_file".into(), input: json!({}), thought_signature: None }] },
            LlmMessage { role: LlmRole::User, content: vec![ContentBlock::ToolResult { tool_use_id: id.into(), content: result.into(), is_error: false }] },
        ]
    }

    fn result_of(message: &LlmMessage) -> &str {
        match &message.content[0] {
            ContentBlock::ToolResult { content, .. } => content,
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn under_budget_nothing_changes() {
        let mut messages: Vec<LlmMessage> = (0..6).flat_map(|i| round(&i.to_string(), &"x".repeat(5_000))).collect();
        assert_eq!(compact_tool_results(&mut messages, 1_000_000, 4), 0);
        assert!(messages.iter().skip(1).step_by(2).all(|m| result_of(m).len() == 5_000));
    }

    #[test]
    fn over_budget_older_results_are_trimmed_and_the_latest_rounds_kept() {
        let mut messages = vec![LlmMessage { role: LlmRole::User, content: vec![ContentBlock::Text { text: "build it".into() }] }];
        messages.extend((0..6).flat_map(|i| round(&i.to_string(), &"x".repeat(5_000))));
        assert_eq!(compact_tool_results(&mut messages, 10_000, 4), 2);
        // Rounds 0 and 1 trimmed, rounds 2..5 whole.
        assert!(result_of(&messages[2]).contains(TRIMMED_MARK));
        assert!(result_of(&messages[4]).contains(TRIMMED_MARK));
        assert!(result_of(&messages[2]).len() < 800);
        for i in [6, 8, 10, 12] {
            assert_eq!(result_of(&messages[i]).len(), 5_000);
        }
        // Already trimmed results stay as they are, so the prefix is stable.
        let before = messages.clone();
        assert_eq!(compact_tool_results(&mut messages, 10_000, 4), 0);
        assert_eq!(before, messages);
    }

    #[test]
    fn small_results_and_short_conversations_are_left_alone() {
        let mut messages: Vec<LlmMessage> = (0..3).flat_map(|i| round(&i.to_string(), "ok")).collect();
        assert_eq!(compact_tool_results(&mut messages, 0, 4), 0);
    }

    #[test]
    fn long_earlier_messages_are_clipped_but_attachments_kept() {
        let clipped = clip_older_message("a".repeat(5_000), 1_000);
        assert!(clipped.starts_with(&"a".repeat(1_000)) && clipped.contains("left out"));
        let with_file = format!("{}<attachment id=\"x\"/>", "a".repeat(5_000));
        assert_eq!(clip_older_message(with_file.clone(), 1_000), with_file);
        assert_eq!(clip_older_message("short".into(), 1_000), "short");
    }
}
