//! What agents can do to a project's app on Shipyard: publish it, and look inside it once it's
//! running (its status, its logs, its database, read-only). The server provides it (it holds the
//! Shipyard client and checks the person may act on the project); agents only see this trait, so
//! an agent crate never talks to Shipyard itself.

use std::sync::Arc;

use async_trait::async_trait;
use nomi_llm::ToolDefinition;
use serde_json::{json, Value};
use uuid::Uuid;

pub const PUBLISH_TOOL_NAME: &str = "publish_project";

/// Which of a project's databases: the development one Shipyard keeps for the preview and the
/// admin's SQL tool, or the published production app's.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataSource {
    Development,
    Production,
}

impl DataSource {
    pub fn parse(text: &str) -> Option<Self> {
        match text.trim().to_lowercase().as_str() {
            "development" | "dev" => Some(DataSource::Development),
            "production" | "prod" => Some(DataSource::Production),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            DataSource::Development => "development",
            DataSource::Production => "production",
        }
    }
}

/// A table in a project's database.
#[derive(Debug, Clone, PartialEq)]
pub struct TableInfo {
    pub schema: String,
    pub name: String,
    /// Postgres' estimate; -1 before the table was first analyzed.
    pub rows_estimate: i64,
}

/// What a read-only query returned.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct QueryRows {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<Value>>,
    /// More rows matched than came back.
    pub truncated: bool,
}
/// Where a publish goes unless the person names another environment.
pub const DEFAULT_PUBLISH_ENVIRONMENT: &str = "production";

#[async_trait]
pub trait ProjectOps: Send + Sync {
    /// Starts publishing the project's current files to `environment`, as `user_id` (who must be
    /// one of its admins); `agent` is the crew member doing it for them, if any. Says what
    /// happens next and where the app will be.
    async fn publish(&self, user_id: Uuid, project_id: Uuid, environment: &str, agent: Option<&str>) -> Result<String, String>;

    /// What runs in `environment` now and how the latest publishes went, as text.
    async fn status(&self, user_id: Uuid, project_id: Uuid, environment: &str) -> Result<String, String>;

    /// The newest `tail` lines the running app printed in `environment`.
    async fn logs(&self, user_id: Uuid, project_id: Uuid, environment: &str, tail: u32) -> Result<Vec<String>, String>;

    /// The tables in the project's `source` database (admins only).
    async fn tables(&self, user_id: Uuid, project_id: Uuid, source: DataSource) -> Result<Vec<TableInfo>, String>;

    /// Runs one read-only statement on the project's `source` database (admins only): writes
    /// are refused by the database itself.
    async fn query(&self, user_id: Uuid, project_id: Uuid, source: DataSource, sql: &str) -> Result<QueryRows, String>;
}

/// For tests and setups without Shipyard: every call says publishing isn't available.
pub struct NoProjectOps;

#[async_trait]
impl ProjectOps for NoProjectOps {
    async fn publish(&self, _user_id: Uuid, _project_id: Uuid, _environment: &str, _agent: Option<&str>) -> Result<String, String> {
        Err(NOT_SET_UP.to_string())
    }
    async fn status(&self, _user_id: Uuid, _project_id: Uuid, _environment: &str) -> Result<String, String> {
        Err(NOT_SET_UP.to_string())
    }
    async fn logs(&self, _user_id: Uuid, _project_id: Uuid, _environment: &str, _tail: u32) -> Result<Vec<String>, String> {
        Err(NOT_SET_UP.to_string())
    }
    async fn tables(&self, _user_id: Uuid, _project_id: Uuid, _source: DataSource) -> Result<Vec<TableInfo>, String> {
        Err(NOT_SET_UP.to_string())
    }
    async fn query(&self, _user_id: Uuid, _project_id: Uuid, _source: DataSource, _sql: &str) -> Result<QueryRows, String> {
        Err(NOT_SET_UP.to_string())
    }
}

const NOT_SET_UP: &str = "Publishing isn't set up on this Nomi.";

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

/// Runs `publish_project` with `input`, for `agent` (the crew member calling it).
pub async fn publish_from_tool(ops: &dyn ProjectOps, user_id: Uuid, agent: &str, input: &Value) -> Result<String, String> {
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
    ops.publish(user_id, project_id, environment, Some(agent)).await
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Recorder;

    #[async_trait]
    impl ProjectOps for Recorder {
        async fn publish(&self, _user_id: Uuid, project_id: Uuid, environment: &str, agent: Option<&str>) -> Result<String, String> {
            Ok(format!("{project_id} {environment} {}", agent.unwrap_or("-")))
        }
        async fn status(&self, _: Uuid, _: Uuid, _: &str) -> Result<String, String> {
            unimplemented!()
        }
        async fn logs(&self, _: Uuid, _: Uuid, _: &str, _: u32) -> Result<Vec<String>, String> {
            unimplemented!()
        }
        async fn tables(&self, _: Uuid, _: Uuid, _: DataSource) -> Result<Vec<TableInfo>, String> {
            unimplemented!()
        }
        async fn query(&self, _: Uuid, _: Uuid, _: DataSource, _: &str) -> Result<QueryRows, String> {
            unimplemented!()
        }
    }

    #[tokio::test]
    async fn publishes_to_production_unless_told_otherwise() {
        let project_id = Uuid::new_v4();
        let said = publish_from_tool(&Recorder, Uuid::nil(), "Koda", &json!({"project_id": project_id.to_string()})).await.unwrap();
        assert_eq!(said, format!("{project_id} production Koda"));
        let said = publish_from_tool(&Recorder, Uuid::nil(), "Koda", &json!({"project_id": project_id.to_string(), "environment": "staging"})).await.unwrap();
        assert_eq!(said, format!("{project_id} staging Koda"));
        assert!(publish_from_tool(&Recorder, Uuid::nil(), "Koda", &json!({"project_id": "nope"})).await.is_err());
    }
}
