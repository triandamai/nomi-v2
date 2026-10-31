use rusqlite::{params, Connection, Row};

use crate::error::NomiError;
use crate::storage::database::{Table, Tx};
use crate::storage::models::AgentActivity;

const COLUMNS: &str = "agent_session_id, session_id, agent_type, phase, detail, updated_at";
/// The phase the backend reports for an agent that is alive but not doing anything.
pub(crate) const IDLE_PHASE: &str = "waiting";

fn from_row(row: &Row<'_>) -> rusqlite::Result<AgentActivity> {
    Ok(AgentActivity {
        agent_session_id: row.get(0)?,
        session_id: row.get(1)?,
        agent_type: row.get(2)?,
        phase: row.get(3)?,
        detail: row.get(4)?,
        updated_at: row.get(5)?,
    })
}

pub(crate) fn upsert(tx: &mut Tx<'_>, activity: &AgentActivity) -> Result<(), NomiError> {
    tx.writes(Table::Activity).execute(
        &format!(
            "INSERT INTO agent_activity ({COLUMNS}) VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT (agent_session_id) DO UPDATE SET session_id = excluded.session_id,
                 agent_type = excluded.agent_type, phase = excluded.phase,
                 detail = excluded.detail, updated_at = excluded.updated_at"
        ),
        params![
            activity.agent_session_id,
            activity.session_id,
            activity.agent_type,
            activity.phase,
            activity.detail,
            activity.updated_at
        ],
    )?;
    Ok(())
}

pub(crate) fn delete(tx: &mut Tx<'_>, agent_session_id: &str) -> Result<(), NomiError> {
    tx.writes(Table::Activity).execute(
        "DELETE FROM agent_activity WHERE agent_session_id = ?1",
        [agent_session_id],
    )?;
    Ok(())
}

pub(crate) fn clear_for_session(tx: &mut Tx<'_>, session_id: &str) -> Result<(), NomiError> {
    tx.writes(Table::Activity).execute(
        "DELETE FROM agent_activity WHERE session_id = ?1",
        [session_id],
    )?;
    Ok(())
}

/// Agents doing something, most recently updated first; `session_id` narrows it to one chat.
pub(crate) fn active(
    connection: &Connection,
    session_id: Option<&str>,
) -> Result<Vec<AgentActivity>, NomiError> {
    let mut statement = connection.prepare_cached(&format!(
        "SELECT {COLUMNS} FROM agent_activity
         WHERE phase != '{IDLE_PHASE}' AND (?1 IS NULL OR session_id = ?1)
         ORDER BY updated_at DESC, agent_session_id"
    ))?;
    let rows = statement.query_map([session_id], from_row)?;
    Ok(rows.collect::<Result<_, _>>()?)
}
