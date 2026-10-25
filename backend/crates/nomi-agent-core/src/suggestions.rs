//! What the person might want next, offered as buttons when the crew finishes.
//!
//! Every agent may end its final reply with a few `<suggest>…</suggest>` lines
//! (`prompts::FOLLOW_UP_SUGGESTIONS`). They're taken out of the reply before it's saved and
//! posted after it as a row of buttons (`ContentBlock::Suggestions`); tapping one sends its text
//! as the person's next message. The row is a notice, so agents never read it back as something
//! they said.

use nomi_realtime::{MqttPublisher, StreamEnvelope};
use sqlx::PgConnection;
use uuid::Uuid;

use crate::content_block::ContentBlock;

const OPEN_TAG: &str = "<suggest>";
const CLOSE_TAG: &str = "</suggest>";
/// The most buttons one reply gets.
pub const MAX_SUGGESTIONS: usize = 3;
/// Longer than this isn't a button, it's a message.
pub const MAX_SUGGESTION_CHARS: usize = 60;

/// The reply without its suggestion lines, and the suggestions (at most `MAX_SUGGESTIONS`,
/// each short and distinct).
pub fn split_suggestions(reply: &str) -> (String, Vec<String>) {
    let mut kept = String::with_capacity(reply.len());
    let mut suggestions: Vec<String> = Vec::new();
    let mut rest = reply;
    while let Some(start) = rest.find(OPEN_TAG) {
        let after_open = &rest[start + OPEN_TAG.len()..];
        let Some(end) = after_open.find(CLOSE_TAG) else { break };
        kept.push_str(&rest[..start]);
        let label = after_open[..end].split_whitespace().collect::<Vec<_>>().join(" ");
        let fits = !label.is_empty() && label.chars().count() <= MAX_SUGGESTION_CHARS;
        if fits && suggestions.len() < MAX_SUGGESTIONS && !suggestions.iter().any(|s| s.eq_ignore_ascii_case(&label)) {
            suggestions.push(label);
        }
        rest = &after_open[end + CLOSE_TAG.len()..];
    }
    kept.push_str(rest);
    let cleaned = kept.lines().map(str::trim_end).collect::<Vec<_>>().join("\n");
    (cleaned.trim().to_string(), suggestions)
}

/// Posts the buttons after the reply, under the same agent. Best-effort, like other activity.
pub async fn post_suggestions(
    conn: &mut PgConnection,
    mqtt: Option<&MqttPublisher>,
    session_id: Uuid,
    agent_display_name: &str,
    suggestions: Vec<String>,
) -> Option<Uuid> {
    if suggestions.is_empty() {
        return None;
    }
    let text = suggestions.join(" · ");
    let blocks = serde_json::json!([ContentBlock::Suggestions { items: suggestions }]);
    let message_id: Option<Uuid> = sqlx::query_scalar(
        "INSERT INTO messages (session_id, sender_channel_identity_id, content, content_blocks, agent_display_name, is_notice) \
         VALUES ($1, NULL, $2, $3, $4, true) RETURNING id",
    )
    .bind(session_id)
    .bind(&text)
    .bind(&blocks)
    .bind(agent_display_name)
    .fetch_one(conn)
    .await
    .ok();
    if let (Some(id), Some(mqtt)) = (message_id, mqtt) {
        let _ = mqtt.publish(session_id, &StreamEnvelope::MessageCreated { message_id: id }).await;
    }
    message_id
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn takes_the_suggestions_out_of_the_reply() {
        let reply = "Done: the shop has a cart now.\n\n<suggest>Publish it</suggest>\n<suggest>Add a checkout page</suggest>\n";
        let (text, suggestions) = split_suggestions(reply);
        assert_eq!(text, "Done: the shop has a cart now.");
        assert_eq!(suggestions, vec!["Publish it", "Add a checkout page"]);
    }

    #[test]
    fn keeps_a_reply_without_suggestions_as_it_is() {
        assert_eq!(split_suggestions("Hello there"), ("Hello there".to_string(), vec![]));
        // An unclosed tag is just text.
        assert_eq!(split_suggestions("a <suggest>b").0, "a <suggest>b");
    }

    #[test]
    fn drops_empty_long_repeated_and_extra_suggestions() {
        let long = "x".repeat(MAX_SUGGESTION_CHARS + 1);
        let reply = format!(
            "Hi <suggest> </suggest><suggest>{long}</suggest><suggest>One</suggest><suggest>one</suggest>\
             <suggest>Two</suggest><suggest>Three</suggest><suggest>Four</suggest>"
        );
        let (text, suggestions) = split_suggestions(&reply);
        assert_eq!(text, "Hi");
        assert_eq!(suggestions, vec!["One", "Two", "Three"]);
    }
}
