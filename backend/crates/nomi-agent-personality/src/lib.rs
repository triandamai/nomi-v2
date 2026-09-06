use async_trait::async_trait;
use serde_json::{json, Value};
use sqlx::pool::PoolConnection;
use sqlx::Postgres;
use uuid::Uuid;

use nomi_agent_core::prompts::PERSONALITY_SYSTEM_PROMPT;
use nomi_agent_core::SubAgent;
use nomi_llm::ToolDefinition;

pub const PERSONALITY_AGENT_TYPE: &str = "personality";

/// How many past personality versions `list_personality_versions` shows.
const PERSONALITY_HISTORY_LIMIT: i64 = 10;

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
        vec![
            ToolDefinition {
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
            },
            ToolDefinition {
                name: "list_personality_versions".to_string(),
                description: "List your past personality descriptions, most recent first, so you can decide what to roll back to.".to_string(),
                input_schema: json!({"type": "object", "properties": {}}),
            },
            ToolDefinition {
                name: "rollback_personality".to_string(),
                description: "Roll back to a previous personality version. This creates a new version with that version's description rather than deleting anything.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "version": {
                            "type": "integer",
                            "description": "The version number to restore, from list_personality_versions."
                        }
                    },
                    "required": ["version"]
                }),
            },
        ]
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
            "list_personality_versions" => list_personality_versions(conn, user_id).await,
            "rollback_personality" => rollback_personality(conn, session_id, agent_session_id, user_id, input).await,
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

    nomi_agent_core::personality::set_personality(conn, Some(session_id), Some(agent_session_id), user_id, description)
        .await
        .map_err(|e| e.to_string())?;

    Ok(format!("Personality updated to: {description}"))
}

async fn list_personality_versions(conn: &mut PoolConnection<Postgres>, user_id: Uuid) -> Result<String, String> {
    let versions = nomi_agent_core::personality::list_versions(conn, user_id, PERSONALITY_HISTORY_LIMIT)
        .await
        .map_err(|e| e.to_string())?;

    if versions.is_empty() {
        return Ok("No personality history yet.".to_string());
    }

    let lines: Vec<String> = versions
        .iter()
        .map(|v| {
            let marker = if v.is_current { " (current)" } else { "" };
            format!("v{}{}: {}", v.version, marker, v.description)
        })
        .collect();

    Ok(lines.join("\n"))
}

async fn rollback_personality(
    conn: &mut PoolConnection<Postgres>,
    session_id: Uuid,
    agent_session_id: Uuid,
    user_id: Uuid,
    input: Value,
) -> Result<String, String> {
    let version = input
        .get("version")
        .and_then(|v| v.as_i64())
        .ok_or_else(|| "version is required".to_string())? as i32;

    nomi_agent_core::personality::rollback_to_version(conn, Some(session_id), Some(agent_session_id), user_id, version)
        .await
        .map_err(|e| e.to_string())?;

    Ok(format!("Rolled back to version {version}."))
}
