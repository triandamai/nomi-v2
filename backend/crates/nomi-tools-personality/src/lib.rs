
use serde_json::{json, Value};
use sqlx::pool::PoolConnection;
use sqlx::Postgres;
use uuid::Uuid;

use nomi_llm::ToolDefinition;

pub mod store;
pub mod tools;

pub use tools::tools;

/// The area personality tools belong to.
pub const GROUP: &str = PERSONALITY_AGENT_TYPE;

pub const PERSONALITY_AGENT_TYPE: &str = "personality";

/// How many past personality versions `list_personality_versions` shows.
const PERSONALITY_HISTORY_LIMIT: i64 = 10;

pub fn set_personality_tool_definition() -> ToolDefinition {
    ToolDefinition {
        name: "set_personality".to_string(),
        description: "Set how nomi talks and behaves. By default only in this chat; other chats keep theirs. \
            Use scope \"all_chats\" only when the person asks for it everywhere (\"in all chats\", \"from now on\", \
            \"always\", \"as my default\"): it becomes their default and replaces every chat's own personality."
            .to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "description": {
                    "type": "string",
                    "description": "A concise instruction describing the personality, e.g. 'Be sarcastic and blunt.'"
                },
                "scope": {
                    "type": "string",
                    "enum": ["chat", "all_chats"],
                    "description": "\"chat\" (default): only this chat. \"all_chats\": the person's default, in every chat."
                }
            },
            "required": ["description"]
        }),
    }
}

pub fn reset_chat_personality_tool_definition() -> ToolDefinition {
    ToolDefinition {
        name: "reset_chat_personality".to_string(),
        description: "Take this chat back to the person's default personality (drops the one set for this chat only).".to_string(),
        input_schema: json!({"type": "object", "properties": {}}),
    }
}

pub fn list_personality_versions_tool_definition() -> ToolDefinition {
    ToolDefinition {
        name: "list_personality_versions".to_string(),
        description: "List your past personality descriptions, most recent first, so you can decide what to roll back to.".to_string(),
        input_schema: json!({"type": "object", "properties": {}}),
    }
}

pub fn rollback_personality_tool_definition() -> ToolDefinition {
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
    }
}

pub async fn set_personality(
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

    let everywhere = input.get("scope").and_then(|v| v.as_str()) == Some("all_chats");
    if everywhere {
        crate::store::set_personality(conn, Some(session_id), Some(agent_session_id), user_id, description)
            .await
            .map_err(|e| e.to_string())?;
        Ok(format!("Personality for all chats (their default) is now: {description}"))
    } else {
        crate::store::set_chat_personality(conn, session_id, Some(agent_session_id), user_id, description)
            .await
            .map_err(|e| e.to_string())?;
        Ok(format!("Personality for this chat only is now: {description}. Other chats are unchanged."))
    }
}

pub async fn reset_chat_personality(conn: &mut PoolConnection<Postgres>, session_id: Uuid) -> Result<String, String> {
    let had_one = crate::store::clear_chat_personality(conn, session_id).await.map_err(|e| e.to_string())?;
    Ok(if had_one {
        "This chat is back to the default personality.".to_string()
    } else {
        "This chat already uses the default personality.".to_string()
    })
}

pub async fn list_personality_versions(conn: &mut PoolConnection<Postgres>, session_id: Uuid, user_id: Uuid) -> Result<String, String> {
    let versions = crate::store::list_versions(conn, user_id, PERSONALITY_HISTORY_LIMIT)
        .await
        .map_err(|e| e.to_string())?;
    let this_chat = crate::store::chat_personality(conn, session_id).await;
    let chat_line = match &this_chat {
        Some(p) => format!("This chat has its own personality: {p}\n"),
        None => "This chat uses the default personality.\n".to_string(),
    };

    if versions.is_empty() {
        return Ok(format!("{chat_line}No default personality history yet."));
    }

    let lines: Vec<String> = versions
        .iter()
        .map(|v| {
            let marker = if v.is_current { " (current)" } else { "" };
            format!("v{}{}: {}", v.version, marker, v.description)
        })
        .collect();

    Ok(format!("{chat_line}Default personality versions (all chats):\n{}", lines.join("\n")))
}

pub async fn rollback_personality(
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

    crate::store::rollback_to_version(conn, Some(session_id), Some(agent_session_id), user_id, version)
        .await
        .map_err(|e| e.to_string())?;

    Ok(format!("Rolled back to version {version}."))
}
