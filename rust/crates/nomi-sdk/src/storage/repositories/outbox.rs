use rusqlite::{params, Connection, OptionalExtension, Row};

use crate::error::NomiError;
use crate::storage::database::{Table, Tx};
use crate::storage::models::{OutboxEntry, OutboxOperation, OutboxState};

const STATE_PENDING: &str = "pending";
const STATE_FAILED: &str = "failed";
/// Finished: kept as an idempotency record until pruned.
const STATE_DONE: &str = "done";

const COLUMNS: &str =
    "seq, key, lane, operation, state, attempts, next_attempt_at, in_flight, last_error, created_at";

/// One outbox row as the sync engine works with it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Item {
    pub seq: i64,
    pub key: String,
    pub lane: String,
    pub operation: OutboxOperation,
    pub failed: bool,
    pub attempts: u32,
    pub next_attempt_at: i64,
    /// An attempt went out and its outcome is unknown: the backend may already have it.
    pub in_flight: bool,
    pub last_error: Option<String>,
    pub created_at: i64,
}

impl Item {
    pub fn entry(self) -> OutboxEntry {
        OutboxEntry {
            key: self.key,
            operation: self.operation,
            state: if self.failed {
                OutboxState::Failed
            } else {
                OutboxState::Pending
            },
            attempts: self.attempts,
            last_error: self.last_error,
            created_at: self.created_at,
            next_attempt_at: self.next_attempt_at,
        }
    }
}

fn from_row(row: &Row<'_>) -> rusqlite::Result<Item> {
    let operation: String = row.get(3)?;
    let operation = serde_json::from_str(&operation).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(3, rusqlite::types::Type::Text, Box::new(error))
    })?;
    let state: String = row.get(4)?;
    Ok(Item {
        seq: row.get(0)?,
        key: row.get(1)?,
        lane: row.get(2)?,
        operation,
        failed: state == STATE_FAILED,
        attempts: row.get(5)?,
        next_attempt_at: row.get(6)?,
        in_flight: row.get(7)?,
        last_error: row.get(8)?,
        created_at: row.get(9)?,
    })
}

fn encode(operation: &OutboxOperation) -> Result<String, NomiError> {
    serde_json::to_string(operation).map_err(|error| NomiError::Internal {
        reason: format!("encoding an outbox operation: {error}"),
    })
}

/// Queues `operation` under `key`. Returns false, changing nothing, when `key` is already known
/// (queued, failed, or done and not yet pruned).
pub(crate) fn insert(
    tx: &mut Tx<'_>,
    key: &str,
    lane: &str,
    operation: &OutboxOperation,
    now: i64,
) -> Result<bool, NomiError> {
    let inserted = tx.writes(Table::Outbox).execute(
        "INSERT INTO outbox (key, lane, operation, state, attempts, next_attempt_at, in_flight,
                             created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, 0, ?5, 0, ?5, ?5)
         ON CONFLICT (key) DO NOTHING",
        params![key, lane, encode(operation)?, STATE_PENDING, now],
    )?;
    Ok(inserted > 0)
}

pub(crate) fn get(connection: &Connection, key: &str) -> Result<Option<Item>, NomiError> {
    Ok(connection
        .prepare_cached(&format!(
            "SELECT {COLUMNS} FROM outbox WHERE key = ?1 AND state != '{STATE_DONE}'"
        ))?
        .query_row([key], from_row)
        .optional()?)
}

/// The oldest pending entry that is due and first in its lane. Entries in one lane go out in the
/// order they were queued; a lane waiting on a retry holds back only itself.
pub(crate) fn next_due(connection: &Connection, now: i64) -> Result<Option<Item>, NomiError> {
    Ok(connection
        .prepare_cached(&format!(
            "SELECT {COLUMNS} FROM outbox o
             WHERE o.state = '{STATE_PENDING}' AND o.next_attempt_at <= ?1
               AND NOT EXISTS (SELECT 1 FROM outbox p
                               WHERE p.state = '{STATE_PENDING}' AND p.lane = o.lane AND p.seq < o.seq)
             ORDER BY o.seq LIMIT 1"
        ))?
        .query_row([now], from_row)
        .optional()?)
}

/// When the earliest pending entry is due, if any is pending.
pub(crate) fn next_wake(connection: &Connection) -> Result<Option<i64>, NomiError> {
    Ok(connection
        .prepare_cached(&format!(
            "SELECT MIN(next_attempt_at) FROM outbox WHERE state = '{STATE_PENDING}'"
        ))?
        .query_row([], |row| row.get(0))?)
}

/// Pending and failed entries, oldest first.
pub(crate) fn list(connection: &Connection) -> Result<Vec<OutboxEntry>, NomiError> {
    let mut statement = connection.prepare_cached(&format!(
        "SELECT {COLUMNS} FROM outbox WHERE state != '{STATE_DONE}' ORDER BY seq"
    ))?;
    let rows = statement.query_map([], from_row)?;
    Ok(rows
        .map(|row| row.map(Item::entry))
        .collect::<Result<_, _>>()?)
}

/// Records that an attempt is going out: until it settles, the backend may get it.
pub(crate) fn begin_attempt(tx: &mut Tx<'_>, seq: i64, now: i64) -> Result<(), NomiError> {
    tx.writes(Table::Outbox).execute(
        "UPDATE outbox SET attempts = attempts + 1, in_flight = 1, updated_at = ?2 WHERE seq = ?1",
        params![seq, now],
    )?;
    Ok(())
}

/// Finishes the entry, remembering what the backend made of it (`result_id`).
pub(crate) fn complete(
    tx: &mut Tx<'_>,
    seq: i64,
    result_id: Option<&str>,
    now: i64,
) -> Result<(), NomiError> {
    tx.writes(Table::Outbox).execute(
        &format!(
            "UPDATE outbox SET state = '{STATE_DONE}', in_flight = 0, last_error = NULL,
                 result_id = ?2, updated_at = ?3
             WHERE seq = ?1"
        ),
        params![seq, result_id, now],
    )?;
    Ok(())
}

/// Tries the entry again at `next_attempt_at`.
pub(crate) fn reschedule(
    tx: &mut Tx<'_>,
    seq: i64,
    attempts: u32,
    next_attempt_at: i64,
    in_flight: bool,
    error: &str,
    now: i64,
) -> Result<(), NomiError> {
    tx.writes(Table::Outbox).execute(
        &format!(
            "UPDATE outbox SET attempts = ?2, next_attempt_at = ?3, in_flight = ?4, last_error = ?5,
                 updated_at = ?6
             WHERE seq = ?1 AND state = '{STATE_PENDING}'"
        ),
        params![seq, attempts, next_attempt_at, in_flight, error, now],
    )?;
    Ok(())
}

/// Gives up on the entry until someone retries it.
pub(crate) fn fail(tx: &mut Tx<'_>, seq: i64, error: &str, now: i64) -> Result<(), NomiError> {
    tx.writes(Table::Outbox).execute(
        &format!(
            "UPDATE outbox SET state = '{STATE_FAILED}', in_flight = 0, last_error = ?2,
                 updated_at = ?3
             WHERE seq = ?1 AND state = '{STATE_PENDING}'"
        ),
        params![seq, error, now],
    )?;
    Ok(())
}

/// Puts a failed entry back in line, due now. Returns the entry, or `None` when `key` names no
/// failed entry. It keeps its place in its lane and its in-flight mark.
pub(crate) fn retry(tx: &mut Tx<'_>, key: &str, now: i64) -> Result<Option<Item>, NomiError> {
    let updated = tx.writes(Table::Outbox).execute(
        &format!(
            "UPDATE outbox SET state = '{STATE_PENDING}', attempts = 0, next_attempt_at = ?2,
                 updated_at = ?2
             WHERE key = ?1 AND state = '{STATE_FAILED}'"
        ),
        params![key, now],
    )?;
    if updated == 0 {
        return Ok(None);
    }
    get(tx.writes(Table::Outbox), key)
}

/// Drops a pending or failed entry. Returns it, or `None` when `key` names neither.
pub(crate) fn discard(tx: &mut Tx<'_>, key: &str) -> Result<Option<Item>, NomiError> {
    let connection = tx.writes(Table::Outbox);
    let item = get(connection, key)?;
    if item.is_some() {
        connection.execute("DELETE FROM outbox WHERE key = ?1", [key])?;
    }
    Ok(item)
}

/// Pending entries in `lane` that no attempt has gone out for yet, oldest first.
pub(crate) fn idle_in_lane(connection: &Connection, lane: &str) -> Result<Vec<Item>, NomiError> {
    let mut statement = connection.prepare_cached(&format!(
        "SELECT {COLUMNS} FROM outbox
         WHERE lane = ?1 AND state = '{STATE_PENDING}' AND in_flight = 0 ORDER BY seq"
    ))?;
    let rows = statement.query_map([lane], from_row)?;
    Ok(rows.collect::<Result<_, _>>()?)
}

/// Deletes the entries `seqs`, whatever their state.
pub(crate) fn remove(tx: &mut Tx<'_>, seqs: &[i64]) -> Result<(), NomiError> {
    let connection = tx.writes(Table::Outbox);
    let mut statement = connection.prepare_cached("DELETE FROM outbox WHERE seq = ?1")?;
    for seq in seqs {
        statement.execute([seq])?;
    }
    Ok(())
}

/// What finished entries in `lane` produced (e.g. the server ids of messages already sent), so a
/// later entry never claims one of them as its own.
pub(crate) fn results_in_lane(
    connection: &Connection,
    lane: &str,
) -> Result<Vec<String>, NomiError> {
    let mut statement = connection.prepare_cached(&format!(
        "SELECT result_id FROM outbox
         WHERE lane = ?1 AND state = '{STATE_DONE}' AND result_id IS NOT NULL"
    ))?;
    let rows = statement.query_map([lane], |row| row.get(0))?;
    Ok(rows.collect::<Result<_, _>>()?)
}

/// Whether `lane` has entries not yet done (pending or failed).
pub(crate) fn lane_busy(connection: &Connection, lane: &str) -> Result<bool, NomiError> {
    Ok(connection
        .prepare_cached(&format!(
            "SELECT EXISTS (SELECT 1 FROM outbox WHERE lane = ?1 AND state != '{STATE_DONE}')"
        ))?
        .query_row([lane], |row| row.get(0))?)
}

/// Forgets idempotency records finished before `before`. Returns how many went.
pub(crate) fn prune(tx: &mut Tx<'_>, before: i64) -> Result<usize, NomiError> {
    Ok(tx.writes(Table::Outbox).execute(
        &format!("DELETE FROM outbox WHERE state = '{STATE_DONE}' AND updated_at < ?1"),
        [before],
    )?)
}

/// Makes every pending entry due now (the connection came back), keeping its attempt count.
pub(crate) fn due_now(tx: &mut Tx<'_>, now: i64) -> Result<(), NomiError> {
    tx.writes(Table::Outbox).execute(
        &format!(
            "UPDATE outbox SET next_attempt_at = ?1
             WHERE state = '{STATE_PENDING}' AND next_attempt_at > ?1"
        ),
        [now],
    )?;
    Ok(())
}
