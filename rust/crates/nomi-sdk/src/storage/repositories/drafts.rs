use rusqlite::{params, Connection, OptionalExtension, Row};

use crate::error::NomiError;
use crate::storage::database::{Table, Tx};
use crate::storage::models::Draft;

const COLUMNS: &str = "session_id, turn_job_id, text, phase, updated_at";

fn from_row(row: &Row<'_>) -> rusqlite::Result<Draft> {
    Ok(Draft {
        session_id: row.get(0)?,
        turn_job_id: row.get(1)?,
        text: row.get(2)?,
        phase: row.get(3)?,
        updated_at: row.get(4)?,
    })
}

pub(crate) fn get(connection: &Connection, session_id: &str) -> Result<Option<Draft>, NomiError> {
    Ok(connection
        .prepare_cached(&format!(
            "SELECT {COLUMNS} FROM streaming_drafts WHERE session_id = ?1"
        ))?
        .query_row([session_id], from_row)
        .optional()?)
}

/// Appends `text` to the chat's draft. A draft from another turn is replaced, phase included, so
/// the previous turn never leaks into the new one.
pub(crate) fn append(
    tx: &mut Tx<'_>,
    session_id: &str,
    turn_job_id: &str,
    text: &str,
    updated_at: i64,
) -> Result<(), NomiError> {
    tx.writes(Table::Drafts).execute(
        &format!(
            "INSERT INTO streaming_drafts ({COLUMNS}) VALUES (?1, ?2, ?3, NULL, ?4)
             ON CONFLICT (session_id) DO UPDATE SET
                 text = CASE WHEN turn_job_id = excluded.turn_job_id THEN text || excluded.text ELSE excluded.text END,
                 phase = CASE WHEN turn_job_id = excluded.turn_job_id THEN phase ELSE NULL END,
                 turn_job_id = excluded.turn_job_id,
                 updated_at = excluded.updated_at"
        ),
        params![session_id, turn_job_id, text, updated_at],
    )?;
    Ok(())
}

/// Sets the phase of the chat's draft for `turn_job_id`, starting an empty draft if there is none.
pub(crate) fn set_phase(
    tx: &mut Tx<'_>,
    session_id: &str,
    turn_job_id: &str,
    phase: Option<&str>,
    updated_at: i64,
) -> Result<(), NomiError> {
    tx.writes(Table::Drafts).execute(
        &format!(
            "INSERT INTO streaming_drafts ({COLUMNS}) VALUES (?1, ?2, '', ?3, ?4)
             ON CONFLICT (session_id) DO UPDATE SET
                 text = CASE WHEN turn_job_id = excluded.turn_job_id THEN text ELSE '' END,
                 turn_job_id = excluded.turn_job_id,
                 phase = excluded.phase,
                 updated_at = excluded.updated_at"
        ),
        params![session_id, turn_job_id, phase, updated_at],
    )?;
    Ok(())
}

/// Removes the chat's draft; with `turn_job_id`, only if it belongs to that turn.
pub(crate) fn clear(
    tx: &mut Tx<'_>,
    session_id: &str,
    turn_job_id: Option<&str>,
) -> Result<(), NomiError> {
    tx.writes(Table::Drafts).execute(
        "DELETE FROM streaming_drafts WHERE session_id = ?1 AND (?2 IS NULL OR turn_job_id = ?2)",
        params![session_id, turn_job_id],
    )?;
    Ok(())
}
