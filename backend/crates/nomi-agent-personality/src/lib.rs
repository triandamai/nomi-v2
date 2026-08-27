use async_trait::async_trait;
use serde_json::{json, Value};
use sqlx::pool::PoolConnection;
use sqlx::Postgres;
use uuid::Uuid;

use nomi_agent_core::SubAgent;
use nomi_llm::ToolDefinition;

pub const PERSONALITY_AGENT_TYPE: &str = "personality";

const PERSONALITY_SYSTEM_PROMPT: &str =
    "You help the user customize nomi's personality — the tone, style, and manner nomi should \
     adopt in future replies. When the user describes how they want nomi to talk or behave, call \
     set_personality with a concise (one or two sentence) description of that personality, written \
     in the second person as an instruction (e.g. 'Be sarcastic and blunt, never overly polite.'). \
     Confirm the change back to the user in a friendly way, in the new personality if one was just \
     set. When you're done, call complete_task.";

pub struct PersonalityAgent;

#[async_trait]
impl SubAgent for PersonalityAgent {
    fn agent_type(&self) -> &'static str {
        PERSONALITY_AGENT_TYPE
    }

    fn system_prompt(&self) -> &'static str {
        PERSONALITY_SYSTEM_PROMPT
    }

    fn tools(&self) -> Vec<ToolDefinition> {
        vec![ToolDefinition {
            name: "set_personality".to_string(),
            description: "Set nomi's personality — how it should talk and behave in future replies — for this user."
                .to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "description": {
                        "type": "string",
                        "description": "A concise instruction describing the personality, e.g. 'Be sarcastic and blunt.'"
                    }
                },
                "required": ["description"]
            }),
        }]
    }

    async fn execute_tool(
        &self,
        conn: &mut PoolConnection<Postgres>,
        session_id: Uuid,
        agent_session_id: Uuid,
        user_id: Uuid,
        name: &str,
        input: Value,
    ) -> Result<String, String> {
        match name {
            "set_personality" => set_personality(conn, session_id, agent_session_id, user_id, input).await,
            other => Err(format!("unknown tool: {other}")),
        }
    }

    fn intent_label(&self) -> &'static str {
        PERSONALITY_AGENT_TYPE
    }

    fn intent_description(&self) -> &'static str {
        "The user wants to change how nomi talks or behaves — its tone, style, or personality"
    }

    fn uses_personality(&self) -> bool {
        true
    }
}

async fn set_personality(
    conn: &mut PoolConnection<Postgres>,
    session_id: Uuid,
    agent_session_id: Uuid,
    user_id: Uuid,
    input: Value,
) -> Result<String, String> {
    let description = input
        .get("description")
        .and_then(|v| v.as_str())
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| "description is required and must not be empty".to_string())?;

    nomi_agent_core::personality::set_personality(
        conn,
        Some(session_id),
        Some(agent_session_id),
        user_id,
        description,
    )
    .await
    .map_err(|e| e.to_string())?;

    Ok(format!("Personality updated to: {description}"))
}
