//! Required reading: each of Sora's steps follows rules in her library, so a step's tool waits
//! until she has read those skills in this chat (with `read_design_skill`), and tells her which.
//! What she read is taken from the engine's log of her tool calls (`agent_events`), so a read
//! counts for the whole chat even after its text is trimmed from her context.

use std::collections::HashSet;

use nomi_agent_core::design_artifact::Kind;
use nomi_agent_core::prompts::DESIGN_READ_FIRST;
use sqlx::PgConnection;
use uuid::Uuid;

/// A step that has required reading.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    /// Proposing directions.
    Directions,
    /// Building a preview of this kind.
    Preview(Kind),
    /// Writing the handoff for Koda.
    Handoff,
}

/// The skills `step` follows.
pub fn required(step: Step) -> &'static [&'static str] {
    match step {
        Step::Directions => &["taste-core", "styles-index"],
        Step::Preview(Kind::Design | Kind::Asset) => &["pen-format", "components-kit"],
        Step::Preview(Kind::Deck) => &["decks-html-ppt", "decks-runtime-notes"],
        Step::Preview(Kind::Video) => &["video-hyperframes", "video-hyperframes-composition"],
        Step::Handoff => &["components-handoff", "systems-schema"],
    }
}

/// Why `step` must wait, given the skills already `read`; `None` once they're all read.
pub fn missing(step: Step, read: &HashSet<String>) -> Option<String> {
    let unread: Vec<&str> = required(step).iter().copied().filter(|name| !read.contains(*name)).collect();
    (!unread.is_empty()).then(|| DESIGN_READ_FIRST.replace("{skills}", &unread.join(", ")))
}

/// The skills Sora read in this chat (successful `read_design_skill` calls), by lowercase name.
pub async fn read_in(conn: &mut PgConnection, session_id: Uuid) -> Result<HashSet<String>, sqlx::Error> {
    let topics: Vec<String> = sqlx::query_scalar(
        "SELECT DISTINCT lower(trim(payload->'input'->>'topic')) FROM agent_events \
         WHERE session_id = $1 AND event_type = 'ToolCalled' AND payload->>'tool_name' = 'read_design_skill' \
           AND COALESCE((payload->>'is_error')::boolean, false) = false AND payload->'input'->>'topic' IS NOT NULL",
    )
    .bind(session_id)
    .fetch_all(conn)
    .await?;
    Ok(topics.into_iter().collect())
}

/// Refuses `step` until its skills are read; a failure to look is no reason to block her.
pub async fn check(conn: &mut PgConnection, session_id: Uuid, step: Step) -> Result<(), String> {
    match read_in(conn, session_id).await {
        Ok(read) => missing(step, &read).map_or(Ok(()), Err),
        Err(e) => {
            tracing::warn!(error = %e, "design: couldn't read which skills Sora read; letting the step go ahead");
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nomi_agent_core::design_artifact::Kind;
    use std::collections::HashSet;

    fn read(topics: &[&str]) -> HashSet<String> {
        topics.iter().map(|t| t.to_string()).collect()
    }

    #[test]
    fn each_step_names_the_skills_it_needs() {
        assert_eq!(required(Step::Directions), ["taste-core", "styles-index"]);
        assert_eq!(required(Step::Preview(Kind::Design)), ["pen-format", "components-kit"]);
        assert_eq!(required(Step::Preview(Kind::Asset)), ["pen-format", "components-kit"]);
        assert_eq!(required(Step::Preview(Kind::Deck)), ["decks-html-ppt", "decks-runtime-notes"]);
        assert_eq!(required(Step::Preview(Kind::Video)), ["video-hyperframes", "video-hyperframes-composition"]);
        assert_eq!(required(Step::Handoff), ["components-handoff", "systems-schema"]);
    }

    #[test]
    fn every_required_skill_is_in_the_library() {
        let steps = [Step::Directions, Step::Preview(Kind::Design), Step::Preview(Kind::Deck), Step::Preview(Kind::Video), Step::Handoff];
        for step in steps {
            for name in required(step) {
                assert!(crate::library::LIBRARY.iter().any(|s| s.name == *name), "{name} is a skill Sora can read");
            }
        }
    }

    #[test]
    fn a_step_waits_until_its_skills_are_read_and_says_which() {
        let none = missing(Step::Directions, &read(&[])).unwrap();
        assert!(none.contains("taste-core") && none.contains("styles-index") && none.contains("read_design_skill"), "{none}");
        let half = missing(Step::Directions, &read(&["taste-core"])).unwrap();
        assert!(half.contains("styles-index") && !half.contains("taste-core,"), "only what is still unread: {half}");
        assert_eq!(missing(Step::Directions, &read(&["taste-core", "styles-index", "pen-format"])), None);
        assert_eq!(missing(Step::Preview(Kind::Deck), &read(&["decks-html-ppt", "decks-runtime-notes"])), None);
    }
}
