//! A chat's name: set by the person, or by any agent when asked ("call this chat Bali trip").

use sqlx::{PgConnection, Postgres};
use sqlx::pool::PoolConnection;
use uuid::Uuid;

/// Longest chat name kept.
pub const MAX_TITLE_CHARS: usize = 80;

/// The name as stored: one line, trimmed, at most [`MAX_TITLE_CHARS`]. `None` when it's empty.
pub fn clean_title(title: &str) -> Option<String> {
    let one_line = title.split_whitespace().collect::<Vec<_>>().join(" ");
    let trimmed = one_line.trim_matches(|c: char| c == '"' || c == '\'' || c.is_whitespace());
    if trimmed.is_empty() {
        return None;
    }
    Some(trimmed.chars().take(MAX_TITLE_CHARS).collect::<String>().trim_end().to_string())
}

/// Renames the chat. Returns the name as stored, or `None` when the name is empty.
pub async fn rename(conn: &mut PgConnection, session_id: Uuid, title: &str) -> Result<Option<String>, sqlx::Error> {
    let Some(title) = clean_title(title) else { return Ok(None) };
    sqlx::query("UPDATE sessions SET title = $1 WHERE id = $2").bind(&title).bind(session_id).execute(conn).await?;
    Ok(Some(title))
}

/// [`rename`] for the engine's `rename_chat` tool: the result is what the agent is told.
pub async fn rename_from_tool(conn: &mut PoolConnection<Postgres>, session_id: Uuid, input: &serde_json::Value) -> Result<String, String> {
    let title = input.get("title").and_then(|v| v.as_str()).unwrap_or_default();
    match rename(&mut **conn, session_id, title).await {
        Ok(Some(title)) => Ok(title),
        Ok(None) => Err("title must not be empty".to_string()),
        Err(e) => Err(e.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::clean_title;

    #[test]
    fn a_title_is_one_trimmed_line() {
        assert_eq!(clean_title("  \"Bali   trip\nplanning\"  ").as_deref(), Some("Bali trip planning"));
        assert_eq!(clean_title("   "), None);
        assert_eq!(clean_title(&"x".repeat(200)).map(|t| t.chars().count()), Some(super::MAX_TITLE_CHARS));
    }
}
