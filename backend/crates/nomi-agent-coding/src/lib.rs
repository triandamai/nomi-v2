use async_trait::async_trait;
use serde_json::{json, Value};
use sqlx::pool::PoolConnection;
use sqlx::Postgres;
use uuid::Uuid;

use nomi_agent_core::SubAgent;
use nomi_llm::ToolDefinition;
use nomi_storage::S3Config;

pub const CODING_AGENT_TYPE: &str = "coding";

const CODING_SYSTEM_PROMPT: &str =
    "You write real files for a project the user asked to have built, following the plan you \
     were given. The task you were delegated includes a line like 'Project <uuid>: ...' — use \
     that UUID as project_id in every tool call. Use write_file to create or overwrite files, \
     read_file to check existing content before editing it, list_files to see what's there \
     already, and delete_file to remove something you no longer need. Use relative paths with no \
     leading slash (e.g. 'index.html', 'src/app.js'). When you've finished building everything \
     the plan calls for, call complete_task with a short summary of what you built.";

/// The S3 key every file for a project lives at — shared with nomi-server's HTTP routes so both
/// sides agree on where content is, without either duplicating the format string.
pub fn s3_key(project_id: Uuid, path: &str) -> String {
    format!("projects/{project_id}/{path}")
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
    s3: Option<S3Config>,
}

impl CodingAgent {
    pub fn new(s3: Option<S3Config>) -> Self {
        Self { s3 }
    }

    fn s3(&self) -> Result<&S3Config, String> {
        self.s3.as_ref().ok_or_else(|| "code storage isn't configured".to_string())
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
        let s3 = self.s3()?;
        let path = input.get("path").and_then(|v| v.as_str()).ok_or("path is required")?;
        let content = input.get("content").and_then(|v| v.as_str()).ok_or("content is required")?;
        let content_type = guess_content_type(path);

        s3.put_object(&s3_key(project_id, path), content, content_type).await.map_err(|e| e.to_string())?;

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
        let s3 = self.s3()?;
        let path = input.get("path").and_then(|v| v.as_str()).ok_or("path is required")?;

        match s3.get_object(&s3_key(project_id, path)).await.map_err(|e| e.to_string())? {
            Some(content) => Ok(content),
            None => Ok("file not found".to_string()),
        }
    }

    async fn delete_file(&self, conn: &mut PoolConnection<Postgres>, user_id: Uuid, input: Value) -> Result<String, String> {
        let project_id = parse_project_id(&input)?;
        if !owns_project(conn, project_id, user_id).await? {
            return Err("project not found".to_string());
        }
        let s3 = self.s3()?;
        let path = input.get("path").and_then(|v| v.as_str()).ok_or("path is required")?;

        s3.delete_object(&s3_key(project_id, path)).await.map_err(|e| e.to_string())?;
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
