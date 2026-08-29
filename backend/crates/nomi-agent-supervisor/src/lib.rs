use async_trait::async_trait;
use serde_json::{json, Value};
use sqlx::pool::PoolConnection;
use sqlx::Postgres;
use uuid::Uuid;

use nomi_agent_core::SubAgent;
use nomi_llm::ToolDefinition;

pub const SUPERVISOR_AGENT_TYPE: &str = "supervisor";

const SUPERVISOR_SYSTEM_PROMPT: &str =
    "You are the coordinator among a small team of specialist agents. When asked what the team \
     is doing, or for a status report, use list_recent_agent_activity and summarize it plainly — \
     what was asked, of whom, and the outcome if it finished. You never do the specialist work \
     yourself; you only report on it.";

pub struct SupervisorAgent;

#[async_trait]
impl SubAgent for SupervisorAgent {
    fn agent_type(&self) -> &'static str {
        SUPERVISOR_AGENT_TYPE
    }

    fn system_prompt(&self) -> &'static str {
        SUPERVISOR_SYSTEM_PROMPT
    }

    fn tools(&self) -> Vec<ToolDefinition> {
        vec![ToolDefinition {
            name: "list_recent_agent_activity".to_string(),
            description: "List recent and currently active background delegations for this chat session.".to_string(),
            input_schema: json!({"type": "object", "properties": {}}),
        }]
    }

    async fn execute_tool(
        &self,
        conn: &mut PoolConnection<Postgres>,
        session_id: Uuid,
        _agent_session_id: Uuid,
        _user_id: Uuid,
        name: &str,
        _input: Value,
    ) -> Result<String, String> {
        match name {
            "list_recent_agent_activity" => list_recent_agent_activity(conn, session_id).await,
            other => Err(format!("supervisor has no tool named {other}")),
        }
    }

    fn intent_label(&self) -> &'static str {
        SUPERVISOR_AGENT_TYPE
    }

    fn intent_description(&self) -> &'static str {
        "The user is asking what the other agents are doing, wants a status update, or explicitly asks for a report across multiple specialists"
    }

    fn uses_personality(&self) -> bool {
        true
    }

    fn is_delegation_target(&self) -> bool {
        false
    }
}

async fn list_recent_agent_activity(conn: &mut PoolConnection<Postgres>, session_id: Uuid) -> Result<String, String> {
    let rows: Vec<(String, String, String, Option<String>, Option<String>)> = sqlx::query_as(
        "SELECT target_agent_type, task, status, result, error FROM agent_delegations \
         WHERE session_id = $1 ORDER BY created_at DESC LIMIT 10",
    )
    .bind(session_id)
    .fetch_all(&mut **conn)
    .await
    .map_err(|e| e.to_string())?;

    if rows.is_empty() {
        return Ok("No background delegations for this session yet.".to_string());
    }

    let mut out = String::new();
    for (target, task, status, result, error) in rows {
        out.push_str(&format!("- {target} ({status}): asked to \"{task}\""));
        if let Some(r) = result {
            out.push_str(&format!(" — result: {r}"));
        }
        if let Some(e) = error {
            out.push_str(&format!(" — error: {e}"));
        }
        out.push('\n');
    }
    Ok(out)
}

/// One-shot, tool-free completion that phrases a completed delegation's raw result in the
/// supervisor's voice, folding in the user's personality the same way run_agent_turn does for
/// any agent with uses_personality() == true (duplicated here, not shared, because this path
/// intentionally doesn't go through run_agent_turn's full tool loop — this is a deliberate,
/// documented design decision, not an oversight; see the design spec).
pub async fn phrase_delegation_result(
    provider: &dyn nomi_llm::LlmProvider,
    conn: &mut PoolConnection<Postgres>,
    user_id: Uuid,
    target_agent_type: &str,
    task: &str,
    raw_result: &str,
) -> Result<String, String> {
    let mut system = format!(
        "{SUPERVISOR_SYSTEM_PROMPT}\n\nDeliver this result from the {target_agent_type} agent to the user, in your \
         own voice, as if reporting back after they asked you to look into something. Be natural and brief.\n\n\
         What was asked: {task}\nRaw result: {raw_result}"
    );
    if let Some(p) = nomi_agent_core::personality::get_current_personality(conn, user_id).await {
        system = format!("{system}\n\nAdopt this personality in your reply: {p}");
    }

    let request = nomi_llm::LlmRequest {
        system: Some(system),
        messages: vec![nomi_llm::LlmMessage {
            role: nomi_llm::LlmRole::User,
            content: vec![nomi_llm::ContentBlock::Text { text: "Report back.".to_string() }],
        }],
        tools: vec![],
        max_tokens: 256,
    };
    let response = nomi_llm::complete(provider, request).await.map_err(|e| e.to_string())?;
    Ok(response
        .content
        .into_iter()
        .find_map(|block| match block {
            nomi_llm::ContentBlock::Text { text } => Some(text),
            _ => None,
        })
        .unwrap_or(raw_result.to_string()))
}
