use rusqlite::{params, Connection, OptionalExtension, Row};

use super::clamp_limit;
use crate::error::NomiError;
use crate::storage::database::{Table, Tx};
use crate::storage::models::NotificationItem;

const COLUMNS: &str = "id, kind, title, body, link, created_at, read_at";

fn from_row(row: &Row<'_>) -> rusqlite::Result<NotificationItem> {
    Ok(NotificationItem {
        id: row.get(0)?,
        kind: row.get(1)?,
        title: row.get(2)?,
        body: row.get(3)?,
        link: row.get(4)?,
        created_at: row.get(5)?,
        read_at: row.get(6)?,
    })
}

/// Stores `notifications`. A notification already read here stays read even if the server's copy
/// says otherwise, so a refresh never resurrects it as unread.
pub(crate) fn upsert(tx: &mut Tx<'_>, notifications: &[NotificationItem]) -> Result<(), NomiError> {
    let connection = tx.writes(Table::Notifications);
    let mut statement = connection.prepare_cached(&format!(
        "INSERT INTO notifications ({COLUMNS}) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
         ON CONFLICT (id) DO UPDATE SET kind = excluded.kind, title = excluded.title,
             body = excluded.body, link = excluded.link, created_at = excluded.created_at,
             read_at = COALESCE(notifications.read_at, excluded.read_at)"
    ))?;
    for item in notifications {
        statement.execute(params![
            item.id,
            item.kind,
            item.title,
            item.body,
            item.link,
            item.created_at,
            item.read_at
        ])?;
    }
    Ok(())
}

pub(crate) fn list(
    connection: &Connection,
    limit: u32,
) -> Result<Vec<NotificationItem>, NomiError> {
    let mut statement = connection.prepare_cached(&format!(
        "SELECT {COLUMNS} FROM notifications ORDER BY created_at DESC, id LIMIT ?1"
    ))?;
    let rows = statement.query_map([clamp_limit(limit)], from_row)?;
    Ok(rows.collect::<Result<_, _>>()?)
}

pub(crate) fn unread_count(connection: &Connection) -> Result<u32, NomiError> {
    Ok(connection
        .prepare_cached("SELECT COUNT(*) FROM notifications WHERE read_at IS NULL")?
        .query_row([], |row| row.get(0))?)
}

pub(crate) fn mark_all_read(tx: &mut Tx<'_>, read_at: i64) -> Result<(), NomiError> {
    tx.writes(Table::Notifications).execute(
        "UPDATE notifications SET read_at = ?1 WHERE read_at IS NULL",
        [read_at],
    )?;
    Ok(())
}

pub(crate) fn get(
    connection: &Connection,
    id: &str,
) -> Result<Option<NotificationItem>, NomiError> {
    Ok(connection
        .prepare_cached(&format!(
            "SELECT {COLUMNS} FROM notifications WHERE id = ?1"
        ))?
        .query_row([id], from_row)
        .optional()?)
}

pub(crate) fn mark_read(tx: &mut Tx<'_>, id: &str, read_at: i64) -> Result<(), NomiError> {
    tx.writes(Table::Notifications).execute(
        "UPDATE notifications SET read_at = ?2 WHERE id = ?1 AND read_at IS NULL",
        params![id, read_at],
    )?;
    Ok(())
}
