//! Small numbers sync keeps between runs: paging cursors and the like, by key.

use rusqlite::{params, Connection, OptionalExtension};

use crate::error::NomiError;
use crate::storage::database::Tx;

pub(crate) fn get(connection: &Connection, key: &str) -> Result<Option<i64>, NomiError> {
    Ok(connection
        .prepare_cached("SELECT value FROM sync_state WHERE key = ?1")?
        .query_row([key], |row| row.get(0))
        .optional()?)
}

/// No observer watches this table, so no table is marked changed.
pub(crate) fn set(tx: &mut Tx<'_>, key: &str, value: i64) -> Result<(), NomiError> {
    tx.raw().execute(
        "INSERT INTO sync_state (key, value) VALUES (?1, ?2)
         ON CONFLICT (key) DO UPDATE SET value = excluded.value",
        params![key, value],
    )?;
    Ok(())
}
