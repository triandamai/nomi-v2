//! Koda's file tools: the project's code lives in `ProjectStore` (S3 or disk), with one
//! `project_files` row per file. Every change bumps `projects.files_version`, which the open
//! project page watches to copy the change into the running preview.

use serde_json::{json, Value};
use sqlx::pool::PoolConnection;
use sqlx::{PgConnection, Postgres};
use uuid::Uuid;

use nomi_agent_core::{ContentBlock, FileWriteEntry, ToolOutcome};
use nomi_llm::ToolDefinition;
use nomi_storage::ProjectStore;

/// Largest text file Koda or the editor may save (source files).
pub const MAX_FILE_BYTES: usize = 512 * 1024;
/// Largest binary file (an image, font, audio or video the person added) a project may hold.
pub const MAX_BINARY_FILE_BYTES: usize = 10 * 1024 * 1024;
/// Most files one project may hold.
pub const MAX_FILES: i64 = 600;
/// Most files one write_files call may write.
const MAX_BATCH: usize = 40;
/// How much of a file read_file returns before cutting it off.
const READ_LIMIT: usize = 120 * 1024;
const SEARCH_MAX_MATCHES: usize = 80;
const SEARCH_MAX_FILES: i64 = 400;

/// The storage key every file for a project lives at — shared with nomi-server's HTTP routes.
pub fn project_file_key(project_id: Uuid, path: &str) -> String {
    format!("{project_id}/{path}")
}

/// Rejects absolute paths, `..` segments and anything else that isn't a plain relative path.
pub fn validate_path(path: &str) -> Result<(), String> {
    if path.is_empty()
        || path.starts_with('/')
        || path.contains('\\')
        || path.contains('\0')
        || path.split('/').any(|segment| segment == ".." || segment.is_empty())
    {
        return Err("invalid path".to_string());
    }
    if path.split('/').next() == Some("node_modules") {
        return Err("node_modules is installed by npm; don't write into it".to_string());
    }
    Ok(())
}

/// Coarse content type from a file extension, for the static preview and downloads.
pub fn guess_content_type(path: &str) -> &'static str {
    match path.rsplit('.').next().unwrap_or("") {
        "html" | "htm" => "text/html",
        "css" => "text/css",
        "js" | "mjs" | "cjs" => "application/javascript",
        "json" => "application/json",
        "svg" => "image/svg+xml",
        "md" => "text/markdown",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "avif" => "image/avif",
        "ico" => "image/x-icon",
        "bmp" => "image/bmp",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "ttf" => "font/ttf",
        "otf" => "font/otf",
        "mp3" => "audio/mpeg",
        "wav" => "audio/wav",
        "ogg" => "audio/ogg",
        "mp4" => "video/mp4",
        "webm" => "video/webm",
        "pdf" => "application/pdf",
        "zip" => "application/zip",
        "wasm" => "application/wasm",
        _ => "text/plain",
    }
}

/// A file's bytes as text, or None when it's binary (not UTF-8, or holding NUL bytes).
pub fn as_text(bytes: Vec<u8>) -> Option<String> {
    String::from_utf8(bytes).ok().filter(|text| !text.contains('\0'))
}

/// What a project file holds: text Koda can read and edit, or the bytes of a binary file.
pub enum FileBody {
    Text(String),
    Binary(Vec<u8>),
}

/// Reads one project file. `Ok(None)` when there's no such file.
pub async fn read_body(storage: &ProjectStore, project_id: Uuid, path: &str) -> Result<Option<FileBody>, String> {
    let Some(bytes) = storage.get_bytes(&project_file_key(project_id, path)).await.map_err(|e| e.to_string())? else {
        return Ok(None);
    };
    Ok(Some(match String::from_utf8(bytes) {
        Ok(text) if !text.contains('\0') => FileBody::Text(text),
        Ok(text) => FileBody::Binary(text.into_bytes()),
        Err(e) => FileBody::Binary(e.into_bytes()),
    }))
}

fn binary_note(path: &str, size: usize) -> String {
    format!(
        "{path} is a binary file ({} KB, {}), so it can't be read or edited as text. Use it by its path: \
         reference it from the code (an <img src>, a CSS url(), an import) where the stack serves it from.",
        size.div_ceil(1024),
        guess_content_type(path)
    )
}

pub(crate) fn parse_project_id(input: &Value) -> Result<Uuid, String> {
    input
        .get("project_id")
        .and_then(|v| v.as_str())
        .ok_or("project_id is required")?
        .trim()
        .parse()
        .map_err(|_| "project_id is not a valid UUID".to_string())
}

fn str_field<'a>(input: &'a Value, name: &str) -> Result<&'a str, String> {
    input.get(name).and_then(|v| v.as_str()).ok_or(format!("{name} is required"))
}

pub(crate) async fn owns_project(conn: &mut PgConnection, project_id: Uuid, user_id: Uuid) -> Result<bool, String> {
    sqlx::query_scalar("SELECT can_open_project($1, $2)")
        .bind(project_id)
        .bind(user_id)
        .fetch_one(&mut *conn)
        .await
        .map_err(|e| e.to_string())
}

pub(crate) async fn owned_project_id(conn: &mut PgConnection, user_id: Uuid, input: &Value) -> Result<Uuid, String> {
    let project_id = parse_project_id(input)?;
    if !owns_project(conn, project_id, user_id).await? {
        return Err("project not found".to_string());
    }
    Ok(project_id)
}

/// Marks the project's files as changed (the open preview picks it up). The version people see
/// moves once per crew reply: a reply's later changes stay in the version its first one made.
/// A change outside a reply (the code editor, an import) is a version of its own.
pub async fn bump_files_version(conn: &mut PgConnection, project_id: Uuid) -> Result<(), sqlx::Error> {
    bump_files_version_in(conn, project_id, nomi_agent_core::reply::current_reply()).await
}

async fn bump_files_version_in(conn: &mut PgConnection, project_id: Uuid, reply: Option<Uuid>) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE projects SET files_version = files_version + 1, updated_at = now(), \
         version = CASE WHEN $2::uuid IS NOT NULL AND version_reply_id = $2 THEN version ELSE version + 1 END, \
         version_reply_id = $2, \
         status = CASE WHEN status = 'planning' THEN 'building' ELSE status END WHERE id = $1",
    )
    .bind(project_id)
    .bind(reply)
    .execute(&mut *conn)
    .await?;
    Ok(())
}

/// Saves one text file and its row, without bumping the version (callers do, once per change).
/// Returns what it replaced, if that was text.
pub async fn save_file(
    conn: &mut PgConnection,
    storage: &ProjectStore,
    project_id: Uuid,
    path: &str,
    content: &str,
) -> Result<Option<String>, String> {
    if content.len() > MAX_FILE_BYTES {
        return Err(format!("{path} is {} KB; files can be at most {} KB", content.len() / 1024, MAX_FILE_BYTES / 1024));
    }
    save_bytes(conn, storage, project_id, path, content.as_bytes()).await
}

/// Saves one file of any kind (text or binary) and its row, without bumping the version.
/// Returns what it replaced, if that was text.
pub async fn save_bytes(
    conn: &mut PgConnection,
    storage: &ProjectStore,
    project_id: Uuid,
    path: &str,
    content: &[u8],
) -> Result<Option<String>, String> {
    validate_path(path)?;
    if content.len() > MAX_BINARY_FILE_BYTES {
        return Err(format!(
            "{path} is {} MB; files can be at most {} MB",
            content.len().div_ceil(1024 * 1024),
            MAX_BINARY_FILE_BYTES / (1024 * 1024)
        ));
    }
    let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM project_files WHERE project_id = $1 AND path = $2)")
        .bind(project_id)
        .bind(path)
        .fetch_one(&mut *conn)
        .await
        .map_err(|e| e.to_string())?;
    if !exists {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM project_files WHERE project_id = $1")
            .bind(project_id)
            .fetch_one(&mut *conn)
            .await
            .map_err(|e| e.to_string())?;
        if count >= MAX_FILES {
            return Err(format!("the project already has {MAX_FILES} files, the most it can hold"));
        }
    }
    let key = project_file_key(project_id, path);
    let previous = if exists {
        storage
            .get_bytes(&key)
            .await
            .map_err(|e| {
                tracing::error!(%project_id, %path, %key, error = %e, "failed to read existing file from project storage");
                e.to_string()
            })?
            .and_then(as_text)
    } else {
        None
    };
    let content_type = guess_content_type(path);
    if let Err(e) = storage.put_bytes(&key, content.to_vec(), content_type).await {
        tracing::error!(%project_id, %path, %key, error = %e, "failed to save file to project storage");
        return Err(format!("failed to save {path} to storage: {e}"));
    }
    sqlx::query(
        "INSERT INTO project_files (project_id, path, content_type, size_bytes) VALUES ($1, $2, $3, $4) \
         ON CONFLICT (project_id, path) DO UPDATE SET content_type = EXCLUDED.content_type, size_bytes = EXCLUDED.size_bytes, updated_at = now()",
    )
    .bind(project_id)
    .bind(path)
    .bind(content_type)
    .bind(content.len() as i32)
    .execute(&mut *conn)
    .await
    .map_err(|e| e.to_string())?;
    Ok(previous)
}

/// Deletes one file and its row, without bumping the version. False when there was no such file.
pub async fn remove_file(conn: &mut PgConnection, storage: &ProjectStore, project_id: Uuid, path: &str) -> Result<bool, String> {
    validate_path(path)?;
    storage.delete_object(&project_file_key(project_id, path)).await.map_err(|e| e.to_string())?;
    let removed = sqlx::query("DELETE FROM project_files WHERE project_id = $1 AND path = $2")
        .bind(project_id)
        .bind(path)
        .execute(&mut *conn)
        .await
        .map_err(|e| e.to_string())?
        .rows_affected();
    Ok(removed > 0)
}

pub fn write_file_tool_definition() -> ToolDefinition {
    ToolDefinition {
        name: "write_file".to_string(),
        description: "Create or overwrite one file in the project. To change part of an existing file, use edit_file instead."
            .to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "project_id": {"type": "string"},
                "path": {"type": "string", "description": "Relative path, e.g. 'src/routes/+page.svelte'"},
                "content": {"type": "string", "description": "The whole file"}
            },
            "required": ["project_id", "path", "content"]
        }),
    }
}

pub fn add_attachment_tool_definition() -> ToolDefinition {
    ToolDefinition {
        name: "add_attachment".to_string(),
        description: format!(
            "Copy a file the person attached in the chat into the project, byte for byte: any kind (a logo as PNG, JPG \
             or SVG, a photo, an icon, a font, a PDF, data as CSV or JSON), up to {} MB. Use its id from its \
             <attachment id=\"…\"> tag, and a path where the stack serves files from, keeping the file's extension, \
             such as 'static/logo.png' (SvelteKit) or 'public/logo.png' (Vite, React, Vue, Astro), then reference it from \
             the code by that served path.",
            MAX_BINARY_FILE_BYTES / (1024 * 1024)
        ),
        input_schema: json!({
            "type": "object",
            "properties": {
                "project_id": {"type": "string"},
                "attachment_id": {"type": "string", "description": "The attachment's id."},
                "path": {"type": "string", "description": "Where it goes in the project, e.g. 'static/logo.png'"}
            },
            "required": ["project_id", "attachment_id", "path"]
        }),
    }
}

pub fn write_files_tool_definition() -> ToolDefinition {
    ToolDefinition {
        name: "write_files".to_string(),
        description: format!(
            "Create or overwrite several files at once (up to {MAX_BATCH}). Use it to set a project up or add a feature that spans files."
        ),
        input_schema: json!({
            "type": "object",
            "properties": {
                "project_id": {"type": "string"},
                "files": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {"path": {"type": "string"}, "content": {"type": "string"}},
                        "required": ["path", "content"]
                    }
                }
            },
            "required": ["project_id", "files"]
        }),
    }
}

pub fn edit_file_tool_definition() -> ToolDefinition {
    ToolDefinition {
        name: "edit_file".to_string(),
        description: "Change part of a file: replace old_string (copied exactly from the file, with enough lines around it to be unique) with new_string. Set replace_all to change every occurrence."
            .to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "project_id": {"type": "string"},
                "path": {"type": "string"},
                "old_string": {"type": "string"},
                "new_string": {"type": "string"},
                "replace_all": {"type": "boolean"}
            },
            "required": ["project_id", "path", "old_string", "new_string"]
        }),
    }
}

pub fn read_file_tool_definition() -> ToolDefinition {
    ToolDefinition {
        name: "read_file".to_string(),
        description: "Read the current content of a file in the project.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {"project_id": {"type": "string"}, "path": {"type": "string"}},
            "required": ["project_id", "path"]
        }),
    }
}

pub fn list_files_tool_definition() -> ToolDefinition {
    ToolDefinition {
        name: "list_files".to_string(),
        description: "List the project's stack and every file in it, with sizes.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {"project_id": {"type": "string"}},
            "required": ["project_id"]
        }),
    }
}

pub fn delete_file_tool_definition() -> ToolDefinition {
    ToolDefinition {
        name: "delete_file".to_string(),
        description: "Delete a file from the project.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {"project_id": {"type": "string"}, "path": {"type": "string"}},
            "required": ["project_id", "path"]
        }),
    }
}

pub fn search_files_tool_definition() -> ToolDefinition {
    ToolDefinition {
        name: "search_files".to_string(),
        description: "Find text in the project's files (case-insensitive). Returns path:line: text for each match.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "project_id": {"type": "string"},
                "query": {"type": "string"},
                "path_prefix": {"type": "string", "description": "Only search under this folder, e.g. 'src/lib'"}
            },
            "required": ["project_id", "query"]
        }),
    }
}

pub async fn write_file(
    conn: &mut PoolConnection<Postgres>,
    storage: &ProjectStore,
    user_id: Uuid,
    input: Value,
) -> Result<ToolOutcome, String> {
    let project_id = owned_project_id(conn, user_id, &input).await?;
    let path = str_field(&input, "path")?;
    let content = str_field(&input, "content")?;
    let previous_content = match save_file(conn, storage, project_id, path, content).await {
        Ok(prev) => prev,
        Err(e) => {
            tracing::error!(%project_id, %path, error = %e, "write_file failed");
            return Err(e);
        }
    };
    bump_files_version(conn, project_id).await.map_err(|e| e.to_string())?;
    Ok(ToolOutcome {
        display_text: format!("📝 Wrote `{path}`"),
        block: Some(ContentBlock::FileWrite { project_id, path: path.to_string(), content: content.to_string(), previous_content }),
    })
}

/// `add_attachment`: an attached text file, written into the project like `write_file` would.
pub async fn add_attachment(
    conn: &mut PoolConnection<Postgres>,
    storage: &ProjectStore,
    user_id: Uuid,
    input: Value,
) -> Result<ToolOutcome, String> {
    let project_id = owned_project_id(conn, user_id, &input).await?;
    let path = str_field(&input, "path")?;
    let attachment_id: Uuid =
        str_field(&input, "attachment_id")?.trim().parse().map_err(|_| "attachment_id is not an attachment id".to_string())?;
    let (name, bytes) = nomi_agent_core::attachments::attachment_bytes(conn, user_id, attachment_id).await?;
    let previous_content = save_bytes(conn, storage, project_id, path, &bytes).await?;
    bump_files_version(conn, project_id).await.map_err(|e| e.to_string())?;
    // A text file shows as a file card with its content; a binary one only as the line saying it was added.
    let block = as_text(bytes).map(|content| ContentBlock::FileWrite { project_id, path: path.to_string(), content, previous_content });
    Ok(ToolOutcome { display_text: format!("📎 Added {name} as `{path}`"), block })
}

pub async fn write_files(
    conn: &mut PoolConnection<Postgres>,
    storage: &ProjectStore,
    user_id: Uuid,
    input: Value,
) -> Result<ToolOutcome, String> {
    let project_id = owned_project_id(conn, user_id, &input).await?;
    let files = input.get("files").and_then(|v| v.as_array()).ok_or("files is required")?;
    if files.is_empty() {
        return Err("files is empty".to_string());
    }
    if files.len() > MAX_BATCH {
        return Err(format!("write at most {MAX_BATCH} files at a time"));
    }
    // Check every path first, so a bad one doesn't leave the batch half-written.
    let mut batch = Vec::with_capacity(files.len());
    for file in files {
        let path = str_field(file, "path")?;
        let content = str_field(file, "content")?;
        validate_path(path).map_err(|e| format!("{path}: {e}"))?;
        batch.push((path, content));
    }
    let mut written: Vec<FileWriteEntry> = Vec::with_capacity(batch.len());
    for (path, content) in batch {
        match save_file(conn, storage, project_id, path, content).await {
            Ok(previous_content) => written.push(FileWriteEntry { path: path.to_string(), content: content.to_string(), previous_content }),
            Err(e) => {
                tracing::error!(%project_id, %path, error = %e, "write_files: file save failed");
                if !written.is_empty() {
                    bump_files_version(conn, project_id).await.map_err(|e| e.to_string())?;
                }
                return Err(format!("wrote {} file(s), then {path} failed: {e}", written.len()));
            }
        }
    }
    bump_files_version(conn, project_id).await.map_err(|e| e.to_string())?;
    let list = written.iter().map(|f| format!("`{}`", f.path)).collect::<Vec<_>>().join(", ");
    Ok(ToolOutcome {
        display_text: format!("📝 Wrote {} files: {list}", written.len()),
        // Every file of the batch reaches the chat, not only a summary line.
        block: Some(ContentBlock::FileWrites { project_id, files: written }),
    })
}

pub async fn edit_file(
    conn: &mut PoolConnection<Postgres>,
    storage: &ProjectStore,
    user_id: Uuid,
    input: Value,
) -> Result<ToolOutcome, String> {
    let project_id = owned_project_id(conn, user_id, &input).await?;
    let path = str_field(&input, "path")?;
    validate_path(path)?;
    let old = str_field(&input, "old_string")?;
    let new = str_field(&input, "new_string")?;
    let replace_all = input.get("replace_all").and_then(|v| v.as_bool()).unwrap_or(false);
    if old.is_empty() {
        return Err("old_string is empty; use write_file to write a whole file".to_string());
    }
    let current = match read_body(storage, project_id, path).await? {
        Some(FileBody::Text(text)) => text,
        Some(FileBody::Binary(bytes)) => return Err(binary_note(path, bytes.len())),
        None => return Err(format!("{path} doesn't exist; use write_file to create it")),
    };
    let count = current.matches(old).count();
    let updated = match (count, replace_all) {
        (0, _) => return Err(format!("old_string isn't in {path}. Read the file and copy the text exactly, including spaces and tabs.")),
        (1, _) | (_, true) => current.replace(old, new),
        (n, false) => return Err(format!("old_string is in {path} {n} times. Add surrounding lines to make it unique, or set replace_all.")),
    };
    save_file(conn, storage, project_id, path, &updated).await?;
    bump_files_version(conn, project_id).await.map_err(|e| e.to_string())?;
    Ok(ToolOutcome {
        display_text: format!("✏️ Edited `{path}`"),
        block: Some(ContentBlock::FileWrite { project_id, path: path.to_string(), content: updated, previous_content: Some(current) }),
    })
}

pub async fn read_file(conn: &mut PoolConnection<Postgres>, storage: &ProjectStore, user_id: Uuid, input: Value) -> Result<String, String> {
    let project_id = owned_project_id(conn, user_id, &input).await?;
    let path = str_field(&input, "path")?;
    validate_path(path)?;
    match read_body(storage, project_id, path).await? {
        Some(FileBody::Binary(bytes)) => Ok(binary_note(path, bytes.len())),
        Some(FileBody::Text(content)) if content.len() > READ_LIMIT => {
            let mut end = READ_LIMIT;
            while !content.is_char_boundary(end) {
                end -= 1;
            }
            Ok(format!("{}\n… (cut off: the file is {} KB)", &content[..end], content.len() / 1024))
        }
        Some(FileBody::Text(content)) => Ok(content),
        None => Ok("file not found".to_string()),
    }
}

pub async fn delete_file(
    conn: &mut PoolConnection<Postgres>,
    storage: &ProjectStore,
    user_id: Uuid,
    input: Value,
) -> Result<ToolOutcome, String> {
    let project_id = owned_project_id(conn, user_id, &input).await?;
    let path = str_field(&input, "path")?;
    // What was removed, so the chat card can show it (best effort: the delete doesn't depend on it).
    let previous_content = storage.get_bytes(&project_file_key(project_id, path)).await.ok().flatten().and_then(as_text);
    remove_file(conn, storage, project_id, path).await?;
    bump_files_version(conn, project_id).await.map_err(|e| e.to_string())?;
    Ok(ToolOutcome {
        display_text: format!("🗑️ Deleted `{path}`"),
        block: Some(ContentBlock::FileDelete { project_id, path: path.to_string(), previous_content }),
    })
}

pub async fn list_files(conn: &mut PoolConnection<Postgres>, user_id: Uuid, input: Value) -> Result<String, String> {
    let project_id = owned_project_id(conn, user_id, &input).await?;
    let stack: String = sqlx::query_scalar("SELECT stack FROM projects WHERE id = $1")
        .bind(project_id)
        .fetch_one(&mut **conn)
        .await
        .map_err(|e| e.to_string())?;
    let files: Vec<(String, i32)> = sqlx::query_as("SELECT path, size_bytes FROM project_files WHERE project_id = $1 ORDER BY path")
        .bind(project_id)
        .fetch_all(&mut **conn)
        .await
        .map_err(|e| e.to_string())?;
    let mut out = format!("Stack: {stack}\n");
    if files.is_empty() {
        out.push_str("no files yet");
    } else {
        for (path, size) in files {
            out.push_str(&format!("{path} ({size} B)\n"));
        }
    }
    Ok(out.trim_end().to_string())
}

pub async fn search_files(conn: &mut PoolConnection<Postgres>, storage: &ProjectStore, user_id: Uuid, input: Value) -> Result<String, String> {
    let project_id = owned_project_id(conn, user_id, &input).await?;
    let query = str_field(&input, "query")?.to_lowercase();
    if query.trim().is_empty() {
        return Err("query is empty".to_string());
    }
    let prefix = input.get("path_prefix").and_then(|v| v.as_str()).unwrap_or("").trim_end_matches('/');
    let paths: Vec<String> = sqlx::query_scalar(
        "SELECT path FROM project_files WHERE project_id = $1 AND ($2 = '' OR path = $2 OR path LIKE $2 || '/%') \
         AND path NOT LIKE '%.lock' AND path <> 'package-lock.json' ORDER BY path LIMIT $3",
    )
    .bind(project_id)
    .bind(prefix)
    .bind(SEARCH_MAX_FILES)
    .fetch_all(&mut **conn)
    .await
    .map_err(|e| e.to_string())?;
    let mut matches = Vec::new();
    'files: for path in paths {
        // Binary files (images, fonts) have no lines to match.
        let Some(FileBody::Text(content)) = read_body(storage, project_id, &path).await? else {
            continue;
        };
        for (n, line) in content.lines().enumerate() {
            if line.to_lowercase().contains(&query) {
                let line = line.trim();
                let shown: String = line.chars().take(200).collect();
                matches.push(format!("{path}:{}: {shown}", n + 1));
                if matches.len() >= SEARCH_MAX_MATCHES {
                    break 'files;
                }
            }
        }
    }
    if matches.is_empty() {
        return Ok("no matches".to_string());
    }
    let more = if matches.len() >= SEARCH_MAX_MATCHES { "\n… (more matches; narrow the search)" } else { "" };
    Ok(format!("{}{more}", matches.join("\n")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paths_must_be_plain_and_relative() {
        assert!(validate_path("/etc/passwd").is_err());
        assert!(validate_path("../secret").is_err());
        assert!(validate_path("a/../b").is_err());
        assert!(validate_path("a//b").is_err());
        assert!(validate_path("a\\b").is_err());
        assert!(validate_path("").is_err());
        assert!(validate_path("node_modules/x/index.js").is_err());
        assert!(validate_path("src/routes/+page.svelte").is_ok());
        assert!(validate_path(".gitignore").is_ok());
    }

    #[test]
    fn content_types_follow_the_extension() {
        for (path, expected) in [
            ("index.html", "text/html"),
            ("a.htm", "text/html"),
            ("app.css", "text/css"),
            ("main.mjs", "application/javascript"),
            ("data.json", "application/json"),
            ("logo.svg", "image/svg+xml"),
            ("README.md", "text/markdown"),
            ("logo.png", "image/png"),
            ("photo.jpeg", "image/jpeg"),
            ("photo.jpg", "image/jpeg"),
            ("a.gif", "image/gif"),
            ("a.webp", "image/webp"),
            ("a.avif", "image/avif"),
            ("favicon.ico", "image/x-icon"),
            ("a.bmp", "image/bmp"),
            ("f.woff", "font/woff"),
            ("f.woff2", "font/woff2"),
            ("f.ttf", "font/ttf"),
            ("f.otf", "font/otf"),
            ("s.mp3", "audio/mpeg"),
            ("s.wav", "audio/wav"),
            ("s.ogg", "audio/ogg"),
            ("v.mp4", "video/mp4"),
            ("v.webm", "video/webm"),
            ("doc.pdf", "application/pdf"),
            ("a.zip", "application/zip"),
            ("m.wasm", "application/wasm"),
            ("Makefile", "text/plain"),
        ] {
            assert_eq!(guess_content_type(path), expected, "{path}");
        }
    }

    #[test]
    fn only_utf8_without_nul_bytes_is_text() {
        assert_eq!(as_text(b"<svg/>".to_vec()).as_deref(), Some("<svg/>"));
        assert_eq!(as_text(vec![0x89, 0x50, 0x4e, 0x47]), None);
        assert_eq!(as_text(b"a\0b".to_vec()), None);
    }

    #[test]
    fn the_binary_note_says_what_the_file_is_and_how_to_use_it() {
        let note = binary_note("static/logo.png", 1500);
        assert!(note.starts_with("static/logo.png is a binary file (2 KB, image/png)"), "{note}");
        assert!(note.contains("Use it by its path"), "{note}");
    }
}
