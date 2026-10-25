//! What agents can do to a project's app on Shipyard: publish it. The server provides it (it
//! holds the Shipyard client and checks the person may act on the project); agents only see this
//! trait, so an agent crate never talks to Shipyard itself.

use std::sync::Arc;

use async_trait::async_trait;
use nomi_llm::ToolDefinition;
use serde_json::{json, Value};
use uuid::Uuid;

pub const PUBLISH_TOOL_NAME: &str = "publish_project";
/// Where a publish goes unless the person names another environment.
pub const DEFAULT_PUBLISH_ENVIRONMENT: &str = "production";

#[async_trait]
pub trait ProjectOps: Send + Sync {
    /// Starts publishing the project's current files to `environment`, as `user_id` (who must be
    /// one of its admins). Says what happens next and where the app will be.
    async fn publish(&self, user_id: Uuid, project_id: Uuid, environment: &str) -> Result<String, String>;
}

/// For tests and setups without Shipyard: every call says publishing isn't available.
pub struct NoProjectOps;

#[async_trait]
impl ProjectOps for NoProjectOps {
    async fn publish(&self, _user_id: Uuid, _project_id: Uuid, _environment: &str) -> Result<String, String> {
        Err("Publishing isn't set up on this Nomi.".to_string())
    }
}

pub type SharedProjectOps = Arc<dyn ProjectOps>;

pub fn publish_tool_definition() -> ToolDefinition {
    ToolDefinition {
        name: PUBLISH_TOOL_NAME.to_string(),
        description: "Publish the project so it's live on the web (or update the live app with the latest changes). \
             Only when the person asked to publish, deploy, go live or update the live app, or tapped a button saying so: \
             the person approves it first. Publishing takes a few minutes; tell them it started and where it will be."
            .to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "project_id": {"type": "string", "description": "The project's id."},
                "environment": {"type": "string", "description": "Where to publish: production unless the person named another (a-z, 0-9, -)."}
            },
            "required": ["project_id"]
        }),
    }
}

/// Runs `publish_project` with `input`.
pub async fn publish_from_tool(ops: &dyn ProjectOps, user_id: Uuid, input: &Value) -> Result<String, String> {
    let project_id = input
        .get("project_id")
        .and_then(Value::as_str)
        .ok_or("project_id is required")?
        .trim()
        .parse::<Uuid>()
        .map_err(|_| "project_id is not a valid UUID".to_string())?;
    let environment = input
        .get("environment")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|e| !e.is_empty())
        .unwrap_or(DEFAULT_PUBLISH_ENVIRONMENT);
    ops.publish(user_id, project_id, environment).await
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Recorder;

    #[async_trait]
    impl ProjectOps for Recorder {
        async fn publish(&self, _user_id: Uuid, project_id: Uuid, environment: &str) -> Result<String, String> {
            Ok(format!("{project_id} {environment}"))
        }
    }

    #[tokio::test]
    async fn publishes_to_production_unless_told_otherwise() {
        let project_id = Uuid::new_v4();
        let said = publish_from_tool(&Recorder, Uuid::nil(), &json!({"project_id": project_id.to_string()})).await.unwrap();
        assert_eq!(said, format!("{project_id} production"));
        let said = publish_from_tool(&Recorder, Uuid::nil(), &json!({"project_id": project_id.to_string(), "environment": "staging"})).await.unwrap();
        assert_eq!(said, format!("{project_id} staging"));
        assert!(publish_from_tool(&Recorder, Uuid::nil(), &json!({"project_id": "nope"})).await.is_err());
    }
}
