//! What agents can do with a project's Nomi Relay backend (see relay/README.md): make it, apply
//! the project's migrations, read what's in it, deploy its functions and get its TypeScript
//! types. The server provides it (it holds Relay's control-plane credential and checks the person
//! may act on the project); agents only see this trait, so an agent crate never talks to Relay
//! itself. A Nomi without Relay uses `NoRelayOps` and keeps building projects on Drizzle.

use std::sync::Arc;

use async_trait::async_trait;
use nomi_llm::ToolDefinition;
use serde_json::{json, Value};
use uuid::Uuid;

pub const APPLY_MIGRATIONS_TOOL_NAME: &str = "apply_migrations";
pub const DESCRIBE_BACKEND_TOOL_NAME: &str = "describe_backend";

/// The tools Koda gets for a project whose backend is Relay. They act on the project's own dev
/// backend, so they don't ask first.
pub const RELAY_TOOL_NAMES: &[&str] = &[APPLY_MIGRATIONS_TOOL_NAME, DESCRIBE_BACKEND_TOOL_NAME];

/// Which of a project's Relay environments: `dev`, which the preview uses, or `production`,
/// which the published app uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelayEnv {
    Dev,
    Production,
}

impl RelayEnv {
    pub fn parse(text: &str) -> Option<Self> {
        match text.trim().to_lowercase().as_str() {
            "dev" | "development" => Some(RelayEnv::Dev),
            "production" | "prod" => Some(RelayEnv::Production),
            _ => None,
        }
    }

    /// The name Relay's control plane uses in its paths.
    pub fn as_str(self) -> &'static str {
        match self {
            RelayEnv::Dev => "dev",
            RelayEnv::Production => "production",
        }
    }
}

/// One file of `nomi/migrations/` or `nomi/functions/`: its path in the project and its text.
#[derive(Debug, Clone, PartialEq)]
pub struct ProjectFile {
    pub path: String,
    pub content: String,
}

#[async_trait]
pub trait RelayOps: Send + Sync {
    /// Whether this Nomi has Relay at all. Without it, projects are built on Drizzle.
    fn is_available(&self) -> bool;

    /// Makes the project's Relay backend if it doesn't have one yet, as `user_id` (who must be
    /// able to open the project), and answers Relay's id for it.
    async fn ensure_backend(&self, user_id: Uuid, project_id: Uuid) -> Result<Uuid, String>;

    /// Lints and applies the migrations `env` doesn't have yet, in order. Answers what ran, or
    /// the exact error with its file and line.
    async fn apply_migrations(&self, user_id: Uuid, project_id: Uuid, env: RelayEnv, files: &[ProjectFile]) -> Result<String, String>;

    /// The backend's tables, columns, policies, buckets, auth providers and functions, as text.
    async fn describe_backend(&self, user_id: Uuid, project_id: Uuid, env: RelayEnv) -> Result<String, String>;

    /// Builds and deploys the project's functions to `env`.
    async fn deploy_functions(&self, user_id: Uuid, project_id: Uuid, env: RelayEnv, files: &[ProjectFile]) -> Result<String, String>;

    /// `src/lib/nomi.types.ts` for `env`'s schema.
    async fn types(&self, user_id: Uuid, project_id: Uuid, env: RelayEnv) -> Result<String, String>;
}

/// For tests and setups without Relay: every call says Relay isn't available.
pub struct NoRelayOps;

#[async_trait]
impl RelayOps for NoRelayOps {
    fn is_available(&self) -> bool {
        false
    }
    async fn ensure_backend(&self, _user_id: Uuid, _project_id: Uuid) -> Result<Uuid, String> {
        Err(NOT_SET_UP.to_string())
    }
    async fn apply_migrations(&self, _user_id: Uuid, _project_id: Uuid, _env: RelayEnv, _files: &[ProjectFile]) -> Result<String, String> {
        Err(NOT_SET_UP.to_string())
    }
    async fn describe_backend(&self, _user_id: Uuid, _project_id: Uuid, _env: RelayEnv) -> Result<String, String> {
        Err(NOT_SET_UP.to_string())
    }
    async fn deploy_functions(&self, _user_id: Uuid, _project_id: Uuid, _env: RelayEnv, _files: &[ProjectFile]) -> Result<String, String> {
        Err(NOT_SET_UP.to_string())
    }
    async fn types(&self, _user_id: Uuid, _project_id: Uuid, _env: RelayEnv) -> Result<String, String> {
        Err(NOT_SET_UP.to_string())
    }
}

pub const NOT_SET_UP: &str = "Relay isn't set up on this Nomi.";

pub type SharedRelayOps = Arc<dyn RelayOps>;

pub fn apply_migrations_tool_definition() -> ToolDefinition {
    ToolDefinition {
        name: APPLY_MIGRATIONS_TOOL_NAME.to_string(),
        description: "Apply the project's migrations (nomi/migrations/*.sql) to its Relay dev database, the one the preview uses. \
             Lints them first, applies only the ones dev doesn't have, in order, and answers what ran or the exact SQL error \
             with its file and line. Never edit a migration that was already applied: write the next one."
            .to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "project_id": {"type": "string", "description": "The project's id."}
            },
            "required": ["project_id"]
        }),
    }
}

pub fn describe_backend_tool_definition() -> ToolDefinition {
    ToolDefinition {
        name: DESCRIBE_BACKEND_TOOL_NAME.to_string(),
        description: "What the project's Relay backend really has: tables and columns, row-level security policies, buckets, \
             auth providers (and which are missing their secret) and functions. Check it instead of remembering the schema."
            .to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "project_id": {"type": "string", "description": "The project's id."},
                "environment": {"type": "string", "enum": ["dev", "production"], "description": "dev (default) or production."}
            },
            "required": ["project_id"]
        }),
    }
}

/// The `project_id` of a Relay tool call.
pub fn project_id_from(input: &Value) -> Result<Uuid, String> {
    input
        .get("project_id")
        .and_then(Value::as_str)
        .ok_or("project_id is required")?
        .trim()
        .parse::<Uuid>()
        .map_err(|_| "project_id is not a valid UUID".to_string())
}

/// The `environment` of a Relay tool call: dev unless it names production.
pub fn environment_from(input: &Value) -> Result<RelayEnv, String> {
    match input.get("environment").and_then(Value::as_str).map(str::trim).filter(|e| !e.is_empty()) {
        None => Ok(RelayEnv::Dev),
        Some(text) => RelayEnv::parse(text).ok_or_else(|| format!("environment must be dev or production, not {text}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn without_relay_every_call_says_it_isnt_set_up() {
        let ops = NoRelayOps;
        let (user, project) = (Uuid::nil(), Uuid::nil());
        assert!(!ops.is_available());
        assert_eq!(ops.ensure_backend(user, project).await.unwrap_err(), NOT_SET_UP);
        assert_eq!(ops.apply_migrations(user, project, RelayEnv::Dev, &[]).await.unwrap_err(), NOT_SET_UP);
        assert_eq!(ops.describe_backend(user, project, RelayEnv::Dev).await.unwrap_err(), NOT_SET_UP);
        assert_eq!(ops.deploy_functions(user, project, RelayEnv::Dev, &[]).await.unwrap_err(), NOT_SET_UP);
        assert_eq!(ops.types(user, project, RelayEnv::Production).await.unwrap_err(), NOT_SET_UP);
    }

    #[test]
    fn environments_read_the_way_people_write_them() {
        assert_eq!(RelayEnv::parse("dev"), Some(RelayEnv::Dev));
        assert_eq!(RelayEnv::parse(" Development "), Some(RelayEnv::Dev));
        assert_eq!(RelayEnv::parse("prod"), Some(RelayEnv::Production));
        assert_eq!(RelayEnv::parse("staging"), None);
        assert_eq!(RelayEnv::Dev.as_str(), "dev");
        assert_eq!(RelayEnv::Production.as_str(), "production");
    }

    #[test]
    fn koda_s_relay_tools_need_a_project() {
        for definition in [apply_migrations_tool_definition(), describe_backend_tool_definition()] {
            assert_eq!(definition.input_schema["required"], json!(["project_id"]));
        }
        assert_eq!(apply_migrations_tool_definition().name, APPLY_MIGRATIONS_TOOL_NAME);
        assert_eq!(describe_backend_tool_definition().name, DESCRIBE_BACKEND_TOOL_NAME);
        assert_eq!(RELAY_TOOL_NAMES, &[APPLY_MIGRATIONS_TOOL_NAME, DESCRIBE_BACKEND_TOOL_NAME]);
    }

    #[test]
    fn describe_backend_defaults_to_dev() {
        assert_eq!(environment_from(&json!({})).unwrap(), RelayEnv::Dev);
        assert_eq!(environment_from(&json!({"environment": "production"})).unwrap(), RelayEnv::Production);
        assert!(environment_from(&json!({"environment": "staging"})).is_err());
    }
}
