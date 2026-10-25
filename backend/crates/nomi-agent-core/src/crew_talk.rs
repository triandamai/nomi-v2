//! How the crew members working in one chat make sure with each other: `ask_agent` puts a
//! question to another crew member mid-task and brings the answer back (both shown in the
//! chat), and `post_crew_note` pins a fact or decision to the chat's notes board, which every
//! agent working there reads in its prompt.

use sqlx::pool::PoolConnection;
use sqlx::Postgres;
use uuid::Uuid;

use nomi_llm::{ContentBlock, LlmMessage, LlmProvider, LlmRequest, LlmRole, ToolDefinition};
use nomi_realtime::MqttPublisher;

use crate::registry::AgentRegistry;
use crate::subagent::SubAgent;

pub const ASK_AGENT_TOOL_NAME: &str = "ask_agent";
pub const NOTE_TOOL_NAME: &str = "post_crew_note";

/// Questions one agent may put to the crew in a single run, so two can't talk in circles.
pub const MAX_QUESTIONS_PER_RUN: usize = 4;
const MAX_NOTE: usize = 500;
const NOTES_SHOWN: i64 = 15;
/// How much of the chat the asked agent reads to answer.
const CONTEXT_MESSAGES: usize = 16;
const CONTEXT_CHARS_PER_MESSAGE: usize = 1500;
const ANSWER_MAX_TOKENS: u32 = 1024;

pub fn ask_agent_tool_definition(targets: &[String]) -> ToolDefinition {
    ToolDefinition {
        name: ASK_AGENT_TOOL_NAME.to_string(),
        description: "Ask another crew member a question when you're working on the same thing and need to make \
                      sure, instead of guessing: e.g. ask planning what was agreed with the person, coding what \
                      it built, money what was logged. They answer from this chat and their own work, and the \
                      exchange shows in the chat. Ask the person (ask_user) for what only they can decide."
            .to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "agent": {"type": "string", "enum": targets, "description": "Who to ask"},
                "question": {"type": "string", "description": "One clear question, with what they need to answer it"}
            },
            "required": ["agent", "question"]
        }),
    }
}

pub fn note_tool_definition() -> ToolDefinition {
    ToolDefinition {
        name: NOTE_TOOL_NAME.to_string(),
        description: "Pin a short note to this chat's crew notes, which every crew member working here reads: a \
                      fact or decision the others should keep to (\"Budget is 5 million rupiah\", \"They want the \
                      site in Indonesian\", \"Logged the receipt, don't log it again\"). One fact per note; not \
                      for chatting with the person."
            .to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {"note": {"type": "string", "maxLength": MAX_NOTE}},
            "required": ["note"]
        }),
    }
}

pub async fn write_note(conn: &mut PoolConnection<Postgres>, session_id: Uuid, agent_type: &str, input: &serde_json::Value) -> Result<String, String> {
    let note = input.get("note").and_then(|v| v.as_str()).map(str::trim).unwrap_or_default();
    if note.is_empty() {
        return Err("note is required".to_string());
    }
    if note.chars().count() > MAX_NOTE {
        return Err(format!("keep a note under {MAX_NOTE} characters: one fact per note"));
    }
    sqlx::query("INSERT INTO crew_notes (session_id, agent_type, note) VALUES ($1, $2, $3)")
        .bind(session_id)
        .bind(agent_type)
        .bind(note)
        .execute(&mut **conn)
        .await
        .map_err(|e| e.to_string())?;
    Ok("Pinned to the crew notes.".to_string())
}

/// The chat's latest crew notes, oldest first, for an agent's prompt. `None` when there are none.
pub async fn notes_board(conn: &mut sqlx::PgConnection, session_id: Uuid) -> Option<String> {
    let mut rows: Vec<(String, String)> =
        sqlx::query_as("SELECT agent_type, note FROM crew_notes WHERE session_id = $1 ORDER BY created_at DESC LIMIT $2")
            .bind(session_id)
            .bind(NOTES_SHOWN)
            .fetch_all(conn)
            .await
            .ok()?;
    if rows.is_empty() {
        return None;
    }
    rows.reverse();
    let mut board = "Crew notes in this chat (facts and decisions the crew pinned; keep to them, and pin new ones with \
                     post_crew_note):\n"
        .to_string();
    for (agent, note) in rows {
        board.push_str(&format!("- {}: {}\n", crate::crew::display_name_for(&agent), note));
    }
    Some(board.trim_end().to_string())
}

/// Who an agent can ask: the rest of the crew that takes work (not itself, not the supervisor).
pub fn ask_targets(registry: &AgentRegistry, me: &str) -> Vec<String> {
    registry.reminder_target_agent_types().into_iter().filter(|t| t != me).collect()
}

fn clip(text: &str, max: usize) -> String {
    let text = text.trim();
    if text.chars().count() <= max { text.to_string() } else { format!("{}…", text.chars().take(max).collect::<String>()) }
}

/// The text of the chat so far, as the asked agent reads it.
fn transcript(conversation: &[LlmMessage]) -> String {
    let lines: Vec<String> = conversation
        .iter()
        .filter_map(|m| {
            let text: Vec<&str> = m.content.iter().filter_map(|b| match b {
                ContentBlock::Text { text } if !text.trim().is_empty() => Some(text.as_str()),
                _ => None,
            }).collect();
            if text.is_empty() {
                return None;
            }
            let who = if m.role == LlmRole::User { "Person" } else { "Crew" };
            Some(format!("{who}: {}", clip(&text.join("\n"), CONTEXT_CHARS_PER_MESSAGE)))
        })
        .collect();
    let start = lines.len().saturating_sub(CONTEXT_MESSAGES);
    lines[start..].join("\n\n")
}

/// Puts `asker`'s question to the agent it names and returns what that agent said, for the
/// asker's tool result. Both the question and the answer are posted in the chat.
#[allow(clippy::too_many_arguments)]
pub async fn ask(
    conn: &mut PoolConnection<Postgres>,
    mqtt: Option<&MqttPublisher>,
    s3: Option<&nomi_storage::S3Config>,
    provider: Option<&dyn LlmProvider>,
    registry: &AgentRegistry,
    asker: &dyn SubAgent,
    session_id: Uuid,
    user_id: Uuid,
    input: &serde_json::Value,
    conversation: &[LlmMessage],
) -> Result<String, String> {
    let target_type = input.get("agent").and_then(|v| v.as_str()).unwrap_or_default();
    let question = input.get("question").and_then(|v| v.as_str()).map(str::trim).unwrap_or_default();
    if question.is_empty() {
        return Err("question is required".to_string());
    }
    if target_type == asker.agent_type().as_ref() {
        return Err("that's you: ask someone else, or work it out from the chat".to_string());
    }
    if !ask_targets(registry, asker.agent_type().as_ref()).iter().any(|t| t == target_type) {
        return Err(format!("{target_type} isn't a crew member you can ask"));
    }
    let asked_so_far = conversation
        .iter()
        .flat_map(|m| &m.content)
        .filter(|b| matches!(b, ContentBlock::ToolUse { name, .. } if name.as_str() == ASK_AGENT_TOOL_NAME))
        .count();
    if asked_so_far > MAX_QUESTIONS_PER_RUN {
        return Err("you've asked the crew enough for now: go on with what you know, or ask the person".to_string());
    }
    let provider = provider.ok_or("the crew can't be asked right now: go on with what you know")?;
    let target = registry.find(target_type).ok_or_else(|| format!("{target_type} isn't a crew member you can ask"))?;
    let asker_name = asker.display_name();
    let target_name = target.display_name();

    let mut system = target.system_prompt().to_string();
    if target.supports_plans() || target.reads_plans() {
        system.push_str(&format!("\n\nThe latest plan in this chat:\n{}", crate::engine::read_latest_plan(conn, s3, session_id).await));
    }
    if let Some(board) = notes_board(&mut **conn, session_id).await {
        system.push_str(&format!("\n\n{board}"));
    }
    let locale = crate::locale::user_locale(conn, user_id).await;
    system.push_str(&format!(
        "\n\nRight now {asker_name}, a crew member working in this chat, is asking you something to make sure before \
         going on. Answer in a few sentences, from this chat, the plan and your own work. Don't use tools and don't \
         start new work. If you don't know, say so plainly and say what to ask the person. {}",
        locale.reply_instruction()
    ));
    let prompt = format!(
        "The chat so far (latest last):\n\n{}\n\n{asker_name} asks you: {question}",
        match transcript(conversation) {
            t if t.is_empty() => "(nothing yet)".to_string(),
            t => t,
        }
    );

    crate::engine::post_activity_message(conn, mqtt, session_id, &asker_name, &format!("{target_name}, {question}"), None).await;
    let request = LlmRequest {
        system: Some(system),
        messages: vec![LlmMessage { role: LlmRole::User, content: vec![ContentBlock::Text { text: prompt }] }],
        tools: vec![],
        max_tokens: ANSWER_MAX_TOKENS + nomi_llm::UNREQUESTED_THINKING_ALLOWANCE,
        enable_reasoning: false,
        reasoning_effort: Default::default(),
    };
    let response = nomi_llm::complete(provider, request).await.map_err(|e| {
        tracing::warn!(error = %e, "crew: asking another agent failed");
        format!("{target_name} couldn't be reached: go on with what you know, or ask the person")
    })?;
    let answer: String = response
        .content
        .iter()
        .filter_map(|b| match b {
            ContentBlock::Text { text } => Some(text.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string();
    if answer.is_empty() {
        return Err(format!("{target_name} had no answer: go on with what you know, or ask the person"));
    }
    crate::engine::post_activity_message(conn, mqtt, session_id, &target_name, &answer, None).await;
    Ok(format!("{target_name} says: {answer}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn said(role: LlmRole, text: &str) -> LlmMessage {
        LlmMessage { role, content: vec![ContentBlock::Text { text: text.to_string() }] }
    }

    #[test]
    fn the_transcript_keeps_the_latest_text_and_drops_tool_noise() {
        let mut conversation: Vec<LlmMessage> = (0..20).map(|i| said(LlmRole::User, &format!("message {i}"))).collect();
        conversation.push(LlmMessage {
            role: LlmRole::Assistant,
            content: vec![ContentBlock::ToolUse { id: "t".into(), name: "write_file".into(), input: serde_json::json!({}), thought_signature: None }],
        });
        conversation.push(said(LlmRole::Assistant, "Building it now"));
        let text = transcript(&conversation);
        assert!(!text.contains("message 0\n") && !text.contains("message 4\n"));
        assert!(text.contains("Person: message 19") && text.ends_with("Crew: Building it now"));
        assert!(!text.contains("write_file"));
    }
}
