//! Rena's design discovery: while Sora is on (Admin → Agents), an app or site
//! request is explored step by step instead of with one card of questions. She is told what the
//! chat already asked, so she never asks it twice.

use std::collections::HashSet;

use nomi_agent_core::prompts::{PLANNING_APP_PLAYBOOK, PLANNING_DESIGN_DISCOVERY};
use nomi_agent_core::design_brief::{self, DesignBrief};
use nomi_agent_core::ContentBlock;
use serde_json::Value;
use sqlx::PgConnection;
use uuid::Uuid;

/// The heading of the list of questions the chat already asked.
pub use nomi_agent_core::prompts::PLANNING_ALREADY_ASKED_HEADING as ALREADY_ASKED_HEADING;

/// The most questions listed as already asked: the newest ones.
pub const MAX_ASKED_LISTED: usize = 40;

/// The questions on the cards in `messages` (each a message's `content_blocks`), oldest first.
/// Anything that isn't a list of blocks is passed over.
pub fn asked_questions(messages: &[Value]) -> Vec<String> {
    messages
        .iter()
        .filter_map(|blocks| serde_json::from_value::<Vec<ContentBlock>>(blocks.clone()).ok())
        .flatten()
        .filter_map(|block| match block {
            ContentBlock::Questions { questions, .. } => Some(questions),
            _ => None,
        })
        .flatten()
        .map(|q| q.question)
        .collect()
}

/// A question as it is compared: its letters and digits, lowercased.
fn normalized(question: &str) -> String {
    question.chars().filter(|c| c.is_alphanumeric()).flat_map(char::to_lowercase).collect()
}

/// The list of what was asked, each question once (however it was typed), the newest
/// `MAX_ASKED_LISTED` of them. `None` when nothing was asked.
pub fn already_asked_section(asked: &[String]) -> Option<String> {
    let mut seen = HashSet::new();
    let unique: Vec<&str> = asked
        .iter()
        .map(|q| q.trim())
        .filter(|q| {
            let key = normalized(q);
            !key.is_empty() && seen.insert(key)
        })
        .collect();
    if unique.is_empty() {
        return None;
    }
    let newest = &unique[unique.len().saturating_sub(MAX_ASKED_LISTED)..];
    let list: Vec<String> = newest.iter().map(|q| format!("- {q}")).collect();
    Some(format!("{ALREADY_ASKED_HEADING}\n{}", list.join("\n")))
}

pub const SAVE_DESIGN_BRIEF: &str = "save_design_brief";

/// `save_design_brief`: Rena's tool for the structured brief, while Sora is on.
pub fn save_design_brief_definition() -> nomi_llm::ToolDefinition {
    nomi_llm::ToolDefinition {
        name: SAVE_DESIGN_BRIEF.to_string(),
        description: "Save the design brief: what you found out about this site or app, section by section, each entry marked \
                      with where it came from. Only company, audience and objectives are required; leave the rest out \
                      when it isn't known. Every call saves the whole brief as a new version, so send it complete \
                      whenever something changes."
            .to_string(),
        input_schema: design_brief::brief_input_schema(),
    }
}

/// What Rena is told after a brief was saved.
pub fn saved_message(version: i32, brief: &DesignBrief) -> String {
    let mut said = format!("Design brief saved as version {version}.");
    let unspecified = brief.unspecified();
    if !unspecified.is_empty() {
        said.push_str(&format!(" Not specified: {}.", unspecified.join(", ")));
    }
    match brief.open.len() {
        0 => {}
        1 => said.push_str(" 1 open question."),
        n => said.push_str(&format!(" {n} open questions.")),
    }
    said
}

/// Checks the brief in `input` and saves it as the chat's next version. An invalid brief saves
/// nothing and says what to fix.
pub async fn save_design_brief(conn: &mut PgConnection, session_id: Uuid, agent_session_id: Uuid, user_id: Uuid, input: &Value) -> Result<String, String> {
    let brief = DesignBrief::parse(input)?;
    let version = design_brief::save(conn, session_id, agent_session_id, user_id, &brief).await.map_err(|e| {
        tracing::error!(error = %e, "design brief: save failed");
        "the brief couldn't be saved right now".to_string()
    })?;
    Ok(saved_message(version, &brief))
}

/// What Rena is told for an app or site request: her app-building playbook as it always was, and
/// while Sora is on (`design_on`), design discovery and what the
/// chat already asked.
pub fn app_context(design_on: bool, asked: &[String]) -> String {
    if !design_on {
        return PLANNING_APP_PLAYBOOK.to_string();
    }
    let mut context = format!("{PLANNING_APP_PLAYBOOK}\n\n{PLANNING_DESIGN_DISCOVERY}");
    if let Some(section) = already_asked_section(asked) {
        context.push_str("\n\n");
        context.push_str(&section);
    }
    context
}

/// `app_context` for this chat and person: the design setting as saved now, and the question
/// cards posted in the chat so far.
pub async fn app_context_for(conn: &mut PgConnection, session_id: Uuid, _user_id: Uuid) -> String {
    if !nomi_settings::config::agent_on(nomi_agent_core::crew::DESIGN_AGENT_TYPE).await {
        return app_context(false, &[]);
    }
    let cards: Vec<Value> = sqlx::query_scalar(
        "SELECT content_blocks FROM messages WHERE session_id = $1 AND content_blocks IS NOT NULL ORDER BY created_at, id",
    )
    .bind(session_id)
    .fetch_all(conn)
    .await
    .unwrap_or_default();
    app_context(true, &asked_questions(&cards))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn card(questions: &[&str]) -> serde_json::Value {
        json!([{"kind": "questions", "intro": "A few things first", "questions": questions.iter().map(|q| json!({"question": q, "options": [{"label": "Yes"}]})).collect::<Vec<_>>()}])
    }

    #[test]
    fn the_questions_already_on_cards_are_read_in_order() {
        let messages = [
            card(&["Who is it for?", "What should a visitor do first?"]),
            json!([{"kind": "plan", "plan_id": "00000000-0000-0000-0000-000000000000", "agent_session_id": "00000000-0000-0000-0000-000000000000", "title": "Bakery", "version": 1}]),
            json!(null),
            json!("not blocks"),
            card(&["Do you have a logo?"]),
        ];
        assert_eq!(asked_questions(&messages), ["Who is it for?", "What should a visitor do first?", "Do you have a logo?"]);
    }

    #[test]
    fn the_same_question_is_listed_once_however_it_was_typed() {
        let asked = ["Who is it for?".to_string(), "  who is it FOR ".to_string(), "Do you have a logo?".to_string(), " ".to_string()];
        let section = already_asked_section(&asked).expect("something was asked");
        assert!(section.starts_with(ALREADY_ASKED_HEADING));
        assert_eq!(section.matches("- ").count(), 2, "{section}");
        assert!(section.contains("- Who is it for?\n- Do you have a logo?"));
    }

    #[test]
    fn nothing_asked_yet_adds_no_section() {
        assert_eq!(already_asked_section(&[]), None);
        assert_eq!(already_asked_section(&["  ".to_string()]), None);
    }

    #[test]
    fn only_the_newest_questions_are_listed_when_there_are_too_many() {
        let asked: Vec<String> = (0..MAX_ASKED_LISTED + 2).map(|n| format!("Question {n}?")).collect();
        let section = already_asked_section(&asked).unwrap();
        assert_eq!(section.matches("- Question").count(), MAX_ASKED_LISTED);
        assert!(!section.contains("- Question 0?") && section.contains(&format!("- Question {}?", MAX_ASKED_LISTED + 1)));
    }

    #[test]
    fn without_design_the_app_context_is_the_playbook_as_it_was() {
        assert_eq!(app_context(false, &["Who is it for?".to_string()]), PLANNING_APP_PLAYBOOK);
    }

    #[test]
    fn with_design_the_playbook_is_followed_by_discovery_and_what_was_asked() {
        let context = app_context(true, &["Who is it for?".to_string()]);
        assert!(context.starts_with(PLANNING_APP_PLAYBOOK), "the app playbook is still all there");
        assert!(context.contains(PLANNING_DESIGN_DISCOVERY));
        assert!(context.ends_with("- Who is it for?"));
        assert!(!app_context(true, &[]).contains(ALREADY_ASKED_HEADING));
    }

    #[test]
    fn the_brief_tool_takes_the_briefs_sections() {
        let tool = save_design_brief_definition();
        assert_eq!(tool.name, SAVE_DESIGN_BRIEF);
        assert_eq!(tool.input_schema, nomi_agent_core::design_brief::brief_input_schema());
        assert!(tool.description.contains("new version"));
    }

    #[test]
    fn saving_reports_the_version_and_what_was_left_unspecified() {
        let brief = nomi_agent_core::design_brief::DesignBrief::parse(&json!({
            "company": [{"value": "Roti Pagi", "source": "confirmed"}],
            "audience": [{"value": "Neighbours", "source": "assumed"}],
            "objectives": [{"value": "Pre-orders", "source": "confirmed"}],
            "open": ["Is there a logo?"]
        }))
        .unwrap();
        let said = saved_message(2, &brief);
        assert!(said.starts_with("Design brief saved as version 2."), "{said}");
        assert!(said.contains("Not specified: positioning, brand_personality, visual_preferences, features, assets, references"));
        assert!(said.contains("1 open question"));
    }

    #[test]
    fn discovery_tells_rena_to_research_ask_little_and_keep_facts_apart() {
        for said in ["target_agent 'design'", "save_design_brief", "read_web_page", "## Confirmed", "## From research", "## Recommended", "## Assumed", "## Open", "Already asked", "replaces step (1)"] {
            assert!(PLANNING_DESIGN_DISCOVERY.contains(said), "discovery still says {said:?}");
        }
        for topic in ["audience", "brand", "competitors", "objectives", "features", "assets"] {
            assert!(PLANNING_DESIGN_DISCOVERY.contains(topic), "discovery covers {topic}");
        }
    }
}
