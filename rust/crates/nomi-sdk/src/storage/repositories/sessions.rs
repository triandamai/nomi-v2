use rusqlite::{params, Connection, OptionalExtension, Row};

use super::clamp_limit;
use crate::error::NomiError;
use crate::storage::database::{Table, Tx};
use crate::storage::models::{Session, SessionFilter};

const COLUMNS: &str = "id, title, last_message_preview, updated_at, unread";
const LIKE_ESCAPE: char = '\\';

fn from_row(row: &Row<'_>) -> rusqlite::Result<Session> {
    Ok(Session {
        id: row.get(0)?,
        title: row.get(1)?,
        last_message_preview: row.get(2)?,
        updated_at: row.get(3)?,
        unread: row.get(4)?,
    })
}

pub(crate) fn upsert(tx: &mut Tx<'_>, sessions: &[Session]) -> Result<(), NomiError> {
    let connection = tx.writes(Table::Sessions);
    let mut statement = connection.prepare_cached(&format!(
        "INSERT INTO sessions ({COLUMNS}) VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT (id) DO UPDATE SET title = excluded.title,
             last_message_preview = excluded.last_message_preview,
             updated_at = excluded.updated_at, unread = excluded.unread"
    ))?;
    for session in sessions {
        statement.execute(params![
            session.id,
            session.title,
            session.last_message_preview,
            session.updated_at,
            session.unread
        ])?;
    }
    Ok(())
}

pub(crate) fn get(connection: &Connection, id: &str) -> Result<Option<Session>, NomiError> {
    Ok(connection
        .prepare_cached(&format!("SELECT {COLUMNS} FROM sessions WHERE id = ?1"))?
        .query_row([id], from_row)
        .optional()?)
}

pub(crate) fn list(
    connection: &Connection,
    filter: &SessionFilter,
) -> Result<Vec<Session>, NomiError> {
    let pattern = filter
        .search
        .as_deref()
        .map(str::trim)
        .filter(|search| !search.is_empty())
        .map(|search| format!("%{}%", escape_like(search)));
    let mut statement = connection.prepare_cached(&format!(
        "SELECT {COLUMNS} FROM sessions
         WHERE (?1 IS NULL OR title LIKE ?1 ESCAPE '{LIKE_ESCAPE}' OR last_message_preview LIKE ?1 ESCAPE '{LIKE_ESCAPE}')
           AND (?2 = 0 OR unread = 1)
         ORDER BY updated_at DESC, id
         LIMIT ?3"
    ))?;
    let rows = statement.query_map(
        params![pattern, filter.unread_only, clamp_limit(filter.limit)],
        from_row,
    )?;
    Ok(rows.collect::<Result<_, _>>()?)
}

pub(crate) fn mark_read(tx: &mut Tx<'_>, id: &str) -> Result<(), NomiError> {
    tx.writes(Table::Sessions).execute(
        "UPDATE sessions SET unread = 0 WHERE id = ?1 AND unread = 1",
        [id],
    )?;
    Ok(())
}

/// Removes the chat and everything stored for it.
pub(crate) fn delete(tx: &mut Tx<'_>, id: &str) -> Result<(), NomiError> {
    tx.writes(Table::Messages)
        .execute("DELETE FROM messages WHERE session_id = ?1", [id])?;
    tx.writes(Table::Drafts)
        .execute("DELETE FROM streaming_drafts WHERE session_id = ?1", [id])?;
    tx.writes(Table::Activity)
        .execute("DELETE FROM agent_activity WHERE session_id = ?1", [id])?;
    tx.writes(Table::Sessions)
        .execute("DELETE FROM sessions WHERE id = ?1", [id])?;
    Ok(())
}

fn escape_like(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());
    for character in text.chars() {
        if matches!(character, '%' | '_' | LIKE_ESCAPE) {
            escaped.push(LIKE_ESCAPE);
        }
        escaped.push(character);
    }
    escaped
}
