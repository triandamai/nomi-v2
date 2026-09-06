use async_trait::async_trait;
use serde_json::{json, Value};
use sqlx::pool::PoolConnection;
use sqlx::Postgres;
use uuid::Uuid;

use nomi_agent_core::prompts::CODING_SYSTEM_PROMPT;
use nomi_agent_core::SubAgent;
use nomi_llm::ToolDefinition;
use nomi_storage::LocalFsStore;

pub const CODING_AGENT_TYPE: &str = "coding";

/// The local-disk key every file for a project lives at — shared with nomi-server's HTTP routes
/// so both sides agree on where content is, without either duplicating the format string.
pub fn project_file_key(project_id: Uuid, path: &str) -> String {
    format!("{project_id}/{path}")
}

/// Checks whether `task` starts with the "Project <uuid>: ..." prefix CODING_SYSTEM_PROMPT
/// requires — used by validate_delegation_task to reject a delegation before it's created,
/// rather than let this agent hallucinate a project_id when the planning agent skipped
/// create_project/write_plan and delegated without ever making a project (a real failure mode:
/// the model doesn't always follow the create-then-delegate sequence its own prompt asks for).
fn has_project_prefix(task: &str) -> bool {
    task.strip_prefix("Project ")
        .and_then(|rest| rest.split_once(':'))
        .map(|(id, _)| id.trim().parse::<Uuid>().is_ok())
        .unwrap_or(false)
}

/// Rejects paths that look like a filesystem traversal attempt or an absolute path — this one
/// actually matters now that keys resolve to real filesystem paths under LocalFsStore's root,
/// unlike the old S3 keys, which were opaque flat strings a `..` segment couldn't escape.
pub fn validate_path(path: &str) -> Result<(), String> {
    if path.starts_with('/') || path.split('/').any(|segment| segment == "..") {
        return Err("invalid path".to_string());
    }
    Ok(())
}

/// Coarse content-type guess from a file extension — good enough for both the S3 object's
/// Content-Type and the static preview route; not a full MIME database.
pub fn guess_content_type(path: &str) -> &'static str {
    match path.rsplit('.').next().unwrap_or("") {
        "html" | "htm" => "text/html",
        "css" => "text/css",
        "js" | "mjs" => "application/javascript",
        "json" => "application/json",
        "svg" => "image/svg+xml",
        "md" => "text/markdown",
        _ => "text/plain",
    }
}

pub struct CodingAgent {
    storage: LocalFsStore,
}

impl CodingAgent {
    pub fn new(storage: LocalFsStore) -> Self {
        Self { storage }
    }
}

#[async_trait]
impl SubAgent for CodingAgent {
    fn agent_type(&self) -> &'static str {
        CODING_AGENT_TYPE
    }

    fn system_prompt(&self) -> &'static str {
        CODING_SYSTEM_PROMPT
    }

    fn tools(&self) -> Vec<ToolDefinition> {
        vec![
            ToolDefinition {
                name: "write_file".to_string(),
                description: "Create or overwrite a file in the project.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "project_id": {"type": "string"},
                        "path": {"type": "string", "description": "Relative path, e.g. 'src/index.html'"},
                        "content": {"type": "string"}
                    },
                    "required": ["project_id", "path", "content"]
                }),
            },
            ToolDefinition {
                name: "read_file".to_string(),
                description: "Read the current content of a file in the project.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "project_id": {"type": "string"},
                        "path": {"type": "string"}
                    },
                    "required": ["project_id", "path"]
                }),
            },
            ToolDefinition {
                name: "list_files".to_string(),
                description: "List every file path currently in the project.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {"project_id": {"type": "string"}},
                    "required": ["project_id"]
                }),
            },
            ToolDefinition {
                name: "delete_file".to_string(),
                description: "Delete a file from the project.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "project_id": {"type": "string"},
                        "path": {"type": "string"}
                    },
                    "required": ["project_id", "path"]
                }),
            },
        ]
    }

    async fn execute_tool(
        &self,
        conn: &mut PoolConnection<Postgres>,
        _session_id: Uuid,
        _agent_session_id: Uuid,
        user_id: Uuid,
        name: &str,
        input: Value,
    ) -> Result<String, String> {
        match name {
            "write_file" => self.write_file(conn, user_id, input).await,
            "read_file" => self.read_file(conn, user_id, input).await,
            "list_files" => list_files(conn, user_id, input).await,
            "delete_file" => self.delete_file(conn, user_id, input).await,
            other => Err(format!("unknown tool: {other}")),
        }
    }

    fn intent_label(&self) -> &'static str {
        CODING_AGENT_TYPE
    }

    fn intent_description(&self) -> &'static str {
        "writing or editing code for a project — not reachable directly, only via delegation from planning"
    }

    fn surfaces_activity(&self) -> bool {
        true
    }

    fn validate_delegation_task(&self, task: &str) -> Result<(), String> {
        if has_project_prefix(task) {
            Ok(())
        } else {
            Err(
                "This task has no 'Project <uuid>: ...' prefix, so there's no project to write \
                 files into. Call create_project (and write_plan) first, then delegate again \
                 with a task formatted exactly as 'Project <the real project id>: <summary>'."
                    .to_string(),
            )
        }
    }
}

fn parse_project_id(input: &Value) -> Result<Uuid, String> {
    input
        .get("project_id")
        .and_then(|v| v.as_str())
        .ok_or("project_id is required")?
        .parse()
        .map_err(|_| "project_id is not a valid UUID".to_string())
}

async fn owns_project(conn: &mut PoolConnection<Postgres>, project_id: Uuid, user_id: Uuid) -> Result<bool, String> {
    let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM projects WHERE id = $1 AND user_id = $2)")
        .bind(project_id)
        .bind(user_id)
        .fetch_one(&mut **conn)
        .await
        .map_err(|e| e.to_string())?;
    Ok(exists)
}

impl CodingAgent {
    async fn write_file(&self, conn: &mut PoolConnection<Postgres>, user_id: Uuid, input: Value) -> Result<String, String> {
        let project_id = parse_project_id(&input)?;
        if !owns_project(conn, project_id, user_id).await? {
            return Err("project not found".to_string());
        }
        let path = input.get("path").and_then(|v| v.as_str()).ok_or("path is required")?;
        validate_path(path)?;
        let content = input.get("content").and_then(|v| v.as_str()).ok_or("content is required")?;
        let content_type = guess_content_type(path);

        self.storage.put_object(&project_file_key(project_id, path), content, content_type).await.map_err(|e| e.to_string())?;

        sqlx::query(
            "INSERT INTO project_files (project_id, path, content_type, size_bytes) VALUES ($1, $2, $3, $4) \
             ON CONFLICT (project_id, path) DO UPDATE SET content_type = EXCLUDED.content_type, size_bytes = EXCLUDED.size_bytes, updated_at = now()",
        )
        .bind(project_id)
        .bind(path)
        .bind(content_type)
        .bind(content.len() as i32)
        .execute(&mut **conn)
        .await
        .map_err(|e| e.to_string())?;

        sqlx::query("UPDATE projects SET status = 'building' WHERE id = $1 AND status = 'planning'")
            .bind(project_id)
            .execute(&mut **conn)
            .await
            .map_err(|e| e.to_string())?;

        Ok(format!("wrote {path}"))
    }

    async fn read_file(&self, conn: &mut PoolConnection<Postgres>, user_id: Uuid, input: Value) -> Result<String, String> {
        let project_id = parse_project_id(&input)?;
        if !owns_project(conn, project_id, user_id).await? {
            return Err("project not found".to_string());
        }
        let path = input.get("path").and_then(|v| v.as_str()).ok_or("path is required")?;
        validate_path(path)?;

        match self.storage.get_object(&project_file_key(project_id, path)).await.map_err(|e| e.to_string())? {
            Some(content) => Ok(content),
            None => Ok("file not found".to_string()),
        }
    }

    async fn delete_file(&self, conn: &mut PoolConnection<Postgres>, user_id: Uuid, input: Value) -> Result<String, String> {
        let project_id = parse_project_id(&input)?;
        if !owns_project(conn, project_id, user_id).await? {
            return Err("project not found".to_string());
        }
        let path = input.get("path").and_then(|v| v.as_str()).ok_or("path is required")?;
        validate_path(path)?;

        self.storage.delete_object(&project_file_key(project_id, path)).await.map_err(|e| e.to_string())?;
        sqlx::query("DELETE FROM project_files WHERE project_id = $1 AND path = $2")
            .bind(project_id)
            .bind(path)
            .execute(&mut **conn)
            .await
            .map_err(|e| e.to_string())?;

        Ok(format!("deleted {path}"))
    }
}

async fn list_files(conn: &mut PoolConnection<Postgres>, user_id: Uuid, input: Value) -> Result<String, String> {
    let project_id = parse_project_id(&input)?;
    if !owns_project(conn, project_id, user_id).await? {
        return Err("project not found".to_string());
    }

    let paths: Vec<String> = sqlx::query_scalar("SELECT path FROM project_files WHERE project_id = $1 ORDER BY path")
        .bind(project_id)
        .fetch_all(&mut **conn)
        .await
        .map_err(|e| e.to_string())?;

    if paths.is_empty() {
        return Ok("no files yet".to_string());
    }
    Ok(paths.join("\n"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_path_rejects_leading_slash() {
        assert!(validate_path("/etc/passwd").is_err());
    }

    #[test]
    fn validate_path_rejects_dot_dot_segment() {
        assert!(validate_path("../secret").is_err());
        assert!(validate_path("a/../b").is_err());
        assert!(validate_path("..").is_err());
    }

    #[test]
    fn validate_path_accepts_normal_relative_paths() {
        assert!(validate_path("src/index.html").is_ok());
        assert!(validate_path("index.html").is_ok());
        assert!(validate_path("a/b/c.js").is_ok());
    }

    #[test]
    fn rejects_a_delegation_task_with_no_project_prefix() {
        let agent = CodingAgent::new(nomi_storage::LocalFsStore::at(std::env::temp_dir()));
        let result = agent.validate_delegation_task("Build an advanced calculator app with trig functions.");
        assert!(result.is_err());
    }

    #[test]
    fn rejects_a_delegation_task_with_a_malformed_project_id() {
        let agent = CodingAgent::new(nomi_storage::LocalFsStore::at(std::env::temp_dir()));
        assert!(agent.validate_delegation_task("Project not-a-real-uuid: build it").is_err());
    }

    #[test]
    fn accepts_a_delegation_task_with_a_real_project_prefix() {
        let agent = CodingAgent::new(nomi_storage::LocalFsStore::at(std::env::temp_dir()));
        let task = format!("Project {}: build a calculator", Uuid::new_v4());
        assert!(agent.validate_delegation_task(&task).is_ok());
    }
}
