//! The project a chat belongs to. A project's chat lives inside the project, so the crew never
//! needs its id from the person: tools that take a `project_id` get this chat's project when
//! they leave it out, and every agent's prompt names it.

use serde_json::Value;
use uuid::Uuid;

use nomi_llm::ToolDefinition;

/// This chat's project (the newest made in it) when `user_id` can open it: its id and name.
pub async fn of_chat(conn: &mut sqlx::PgConnection, session_id: Uuid, user_id: Uuid) -> Option<(Uuid, String)> {
    sqlx::query_as(
        "SELECT id, name FROM projects WHERE session_id = $1 AND can_open_project(id, $2) ORDER BY created_at DESC LIMIT 1",
    )
    .bind(session_id)
    .bind(user_id)
    .fetch_optional(conn)
    .await
    .ok()
    .flatten()
}

/// Whether `tool` takes a `project_id`.
pub fn takes_project(tool: &ToolDefinition) -> bool {
    tool.input_schema.get("properties").and_then(|p| p.get("project_id")).is_some()
}

/// Makes `project_id` optional in `tool`, defaulting to this chat's project.
pub fn default_to_chat_project(tool: &mut ToolDefinition, name: &str) {
    if !takes_project(tool) {
        return;
    }
    if let Some(required) = tool.input_schema.get_mut("required").and_then(Value::as_array_mut) {
        required.retain(|r| r != "project_id");
    }
    tool.input_schema["properties"]["project_id"] = serde_json::json!({
        "type": "string",
        "description": format!("Leave out for this chat's project, \"{name}\"; only for another project the person named.")
    });
}

/// `input` with `project_id` set to `project_id` when it has none, or one that isn't an id.
/// `None` when it already names a project.
pub fn fill(input: &Value, project_id: Uuid) -> Option<Value> {
    let object = input.as_object()?;
    let named = object.get("project_id").and_then(Value::as_str).is_some_and(|id| id.trim().parse::<Uuid>().is_ok());
    if named {
        return None;
    }
    let mut filled = object.clone();
    filled.insert("project_id".to_string(), Value::String(project_id.to_string()));
    Some(Value::Object(filled))
}

/// The line every agent's prompt gets in a project's chat.
pub fn prompt_line(project_id: Uuid, name: &str) -> String {
    format!(
        "This chat belongs to the project \"{name}\" (id {project_id}): you're working inside it. Tools that take a \
         project_id use it when you leave it out. Never ask the person for a project id."
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn tool() -> ToolDefinition {
        ToolDefinition {
            name: "publish_project".into(),
            description: String::new(),
            input_schema: json!({
                "type": "object",
                "properties": {"project_id": {"type": "string"}, "environment": {"type": "string"}},
                "required": ["project_id"]
            }),
        }
    }

    #[test]
    fn the_chats_project_fills_a_missing_or_made_up_id_but_not_a_named_one() {
        let chat = Uuid::new_v4();
        let other = Uuid::new_v4();
        assert_eq!(fill(&json!({"environment": "staging"}), chat), Some(json!({"environment": "staging", "project_id": chat.to_string()})));
        assert_eq!(fill(&json!({"project_id": "my-app"}), chat), Some(json!({"project_id": chat.to_string()})));
        assert_eq!(fill(&json!({"project_id": other.to_string()}), chat), None);
    }

    #[test]
    fn a_project_tool_stops_requiring_the_id() {
        let mut publish = tool();
        default_to_chat_project(&mut publish, "Shop");
        assert_eq!(publish.input_schema["required"], json!([]));
        assert!(publish.input_schema["properties"]["project_id"]["description"].as_str().unwrap().contains("\"Shop\""));
        let mut other = ToolDefinition { name: "web_search".into(), description: String::new(), input_schema: json!({"type": "object", "properties": {"q": {}}, "required": ["q"]}) };
        default_to_chat_project(&mut other, "Shop");
        assert_eq!(other.input_schema["required"], json!(["q"]));
    }
}
