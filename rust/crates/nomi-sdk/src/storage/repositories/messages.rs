use rusqlite::{params, Connection, OptionalExtension, Row};

use super::clamp_limit;
use crate::error::NomiError;
use crate::storage::database::{Table, Tx};
use crate::storage::models::{Message, MessageCursor, MessagePage, STATUS_SENT};

const COLUMNS: &str =
    "id, session_id, role, agent_type, agent_display_name, content, content_json, created_at, status";

fn from_row(row: &Row<'_>) -> rusqlite::Result<Message> {
    Ok(Message {
        id: row.get(0)?,
        session_id: row.get(1)?,
        role: row.get(2)?,
        agent_type: row.get(3)?,
        agent_display_name: row.get(4)?,
        content: row.get(5)?,
        content_json: row.get(6)?,
        created_at: row.get(7)?,
        status: row.get(8)?,
    })
}

pub(crate) fn upsert(tx: &mut Tx<'_>, messages: &[Message]) -> Result<(), NomiError> {
    let connection = tx.writes(Table::Messages);
    let mut statement = connection.prepare_cached(&format!(
        "INSERT INTO messages ({COLUMNS}) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
         ON CONFLICT (id) DO UPDATE SET session_id = excluded.session_id, role = excluded.role,
             agent_type = excluded.agent_type, agent_display_name = excluded.agent_display_name,
             content = excluded.content, content_json = excluded.content_json,
             created_at = excluded.created_at, status = excluded.status"
    ))?;
    for message in messages {
        statement.execute(params![
            message.id,
            message.session_id,
            message.role,
            message.agent_type,
            message.agent_display_name,
            message.content,
            message.content_json,
            message.created_at,
            message.status
        ])?;
    }
    Ok(())
}

pub(crate) fn delete(tx: &mut Tx<'_>, id: &str) -> Result<(), NomiError> {
    tx.writes(Table::Messages)
        .execute("DELETE FROM messages WHERE id = ?1", [id])?;
    Ok(())
}

/// Up to `limit` messages older than `before` (or the newest, without it), oldest first.
pub(crate) fn page(
    connection: &Connection,
    session_id: &str,
    before: Option<&MessageCursor>,
    limit: u32,
) -> Result<MessagePage, NomiError> {
    let limit = clamp_limit(limit);
    // One extra row tells whether an older page exists.
    let mut statement = connection.prepare_cached(&format!(
        "SELECT {COLUMNS} FROM messages
         WHERE session_id = ?1 AND (?2 IS NULL OR created_at < ?2 OR (created_at = ?2 AND id < ?3))
         ORDER BY created_at DESC, id DESC
         LIMIT ?4"
    ))?;
    let rows = statement.query_map(
        params![
            session_id,
            before.map(|c| c.created_at),
            before.map(|c| c.id.as_str()),
            limit + 1
        ],
        from_row,
    )?;
    let mut newest_first: Vec<Message> = rows.collect::<Result<_, _>>()?;
    let has_older = newest_first.len() > limit as usize;
    newest_first.truncate(limit as usize);
    newest_first.reverse();
    let older = has_older
        .then(|| {
            newest_first.first().map(|m| MessageCursor {
                created_at: m.created_at,
                id: m.id.clone(),
            })
        })
        .flatten();
    Ok(MessagePage {
        messages: newest_first,
        older,
    })
}

#[cfg(test)]
pub(crate) fn get(connection: &Connection, id: &str) -> Result<Option<Message>, NomiError> {
    Ok(connection
        .prepare_cached(&format!("SELECT {COLUMNS} FROM messages WHERE id = ?1"))?
        .query_row([id], from_row)
        .optional()?)
}

pub(crate) fn set_status(tx: &mut Tx<'_>, id: &str, status: &str) -> Result<(), NomiError> {
    tx.writes(Table::Messages).execute(
        "UPDATE messages SET status = ?2 WHERE id = ?1 AND status != ?2",
        params![id, status],
    )?;
    Ok(())
}

/// The oldest message of the chat the server has stored, as a cursor.
pub(crate) fn oldest_sent(
    connection: &Connection,
    session_id: &str,
) -> Result<Option<MessageCursor>, NomiError> {
    Ok(connection
        .prepare_cached(
            "SELECT created_at, id FROM messages WHERE session_id = ?1 AND status = ?2
             ORDER BY created_at, id LIMIT 1",
        )?
        .query_row(params![session_id, STATUS_SENT], |row| {
            Ok(MessageCursor {
                created_at: row.get(0)?,
                id: row.get(1)?,
            })
        })
        .optional()?)
}

/// Deletes the chat's server messages created strictly between `after` and `until` that are not
/// in `keep`: the server's page for that span no longer has them, so they were deleted there.
/// The ends are exclusive, so a message that merely shares a timestamp with a page edge stays.
/// Local `sending`/`failed` rows are left alone.
pub(crate) fn remove_missing(
    tx: &mut Tx<'_>,
    session_id: &str,
    after: i64,
    until: i64,
    keep: &[String],
) -> Result<usize, NomiError> {
    let connection = tx.writes(Table::Messages);
    let mut statement = connection.prepare_cached(
        "SELECT id FROM messages
         WHERE session_id = ?1 AND status = ?2 AND created_at > ?3 AND created_at < ?4",
    )?;
    let present: Vec<String> = statement
        .query_map(params![session_id, STATUS_SENT, after, until], |row| {
            row.get(0)
        })?
        .collect::<Result<_, _>>()?;
    let mut removed = 0;
    for id in present.iter().filter(|id| !keep.contains(id)) {
        removed += connection.execute("DELETE FROM messages WHERE id = ?1", [id])?;
    }
    Ok(removed)
}
