//! The schema, as an ordered list of migrations tracked by SQLite's `user_version`.
//!
//! Migration `n` (1-based) moves a database from version `n - 1` to `n`. Never edit or reorder a
//! shipped migration: append a new one. Each runs in the same transaction as its version bump.

use rusqlite::{Connection, TransactionBehavior};

use crate::error::NomiError;

const MIGRATIONS: &[&str] = &[
    // 1: the tables the Kotlin apps kept in Room, timestamps in epoch milliseconds.
    "
    CREATE TABLE sessions (
        id                   TEXT PRIMARY KEY NOT NULL,
        title                TEXT NOT NULL,
        last_message_preview TEXT NOT NULL,
        updated_at           INTEGER NOT NULL,
        unread               INTEGER NOT NULL
    );
    CREATE INDEX sessions_by_updated_at ON sessions (updated_at DESC, id);

    CREATE TABLE messages (
        id                 TEXT PRIMARY KEY NOT NULL,
        session_id         TEXT NOT NULL,
        role               TEXT NOT NULL,
        agent_type         TEXT,
        agent_display_name TEXT,
        content            TEXT NOT NULL,
        content_json       TEXT,
        created_at         INTEGER NOT NULL,
        status             TEXT NOT NULL
    );
    CREATE INDEX messages_by_session ON messages (session_id, created_at, id);

    CREATE TABLE streaming_drafts (
        session_id  TEXT PRIMARY KEY NOT NULL,
        turn_job_id TEXT NOT NULL,
        text        TEXT NOT NULL,
        phase       TEXT,
        updated_at  INTEGER NOT NULL
    );

    CREATE TABLE notifications (
        id         TEXT PRIMARY KEY NOT NULL,
        kind       TEXT NOT NULL,
        title      TEXT NOT NULL,
        body       TEXT NOT NULL,
        link       TEXT,
        created_at INTEGER NOT NULL,
        read_at    INTEGER
    );
    CREATE INDEX notifications_by_created_at ON notifications (created_at DESC, id);

    CREATE TABLE agent_activity (
        agent_session_id TEXT PRIMARY KEY NOT NULL,
        session_id       TEXT NOT NULL,
        agent_type       TEXT NOT NULL,
        phase            TEXT NOT NULL,
        detail           TEXT,
        updated_at       INTEGER NOT NULL
    );
    CREATE INDEX agent_activity_by_session ON agent_activity (session_id);
    ",
    // 2: sync. The outbox holds writes until the backend has them; finished entries stay a while
    // as idempotency records. sync_state keeps paging cursors and similar small numbers.
    "
    CREATE TABLE outbox (
        seq             INTEGER PRIMARY KEY AUTOINCREMENT,
        key             TEXT NOT NULL UNIQUE,
        lane            TEXT NOT NULL,
        operation       TEXT NOT NULL,
        state           TEXT NOT NULL,
        attempts        INTEGER NOT NULL,
        next_attempt_at INTEGER NOT NULL,
        in_flight       INTEGER NOT NULL,
        last_error      TEXT,
        result_id       TEXT,
        created_at      INTEGER NOT NULL,
        updated_at      INTEGER NOT NULL
    );
    CREATE INDEX outbox_by_state ON outbox (state, lane, seq);

    CREATE TABLE sync_state (
        key   TEXT PRIMARY KEY NOT NULL,
        value INTEGER NOT NULL
    );
    ",
];

/// The version a fully migrated database reports.
#[cfg(test)]
pub(crate) fn latest_version() -> u32 {
    MIGRATIONS.len() as u32
}

/// Brings `connection` up to [`latest_version`]. A database from a newer SDK is refused rather
/// than guessed at.
pub(crate) fn migrate(connection: &mut Connection) -> Result<(), NomiError> {
    apply(connection, MIGRATIONS)
}

fn apply(connection: &mut Connection, migrations: &[&str]) -> Result<(), NomiError> {
    let current = version(connection)?;
    let latest = migrations.len() as u32;
    if current > latest {
        return Err(NomiError::Storage {
            reason: format!("the database is at schema {current}, newer than this SDK's {latest}"),
        });
    }
    for (index, sql) in migrations.iter().enumerate().skip(current as usize) {
        let target = index as u32 + 1;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        transaction
            .execute_batch(sql)
            .map_err(|error| NomiError::Storage {
                reason: format!("migration {target}: {error}"),
            })?;
        transaction.pragma_update(None, "user_version", target)?;
        transaction.commit()?;
        tracing::info!(version = target, "migrated the database");
    }
    Ok(())
}

pub(crate) fn version(connection: &Connection) -> Result<u32, NomiError> {
    Ok(connection.pragma_query_value(None, "user_version", |row| row.get(0))?)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tables(connection: &Connection) -> Vec<String> {
        let mut statement = connection
            .prepare("SELECT name FROM sqlite_master WHERE type = 'table' ORDER BY name")
            .unwrap();
        statement
            .query_map([], |row| row.get(0))
            .unwrap()
            .map(Result::unwrap)
            .collect()
    }

    #[test]
    fn migrates_an_empty_database_to_the_latest_version() {
        let mut connection = Connection::open_in_memory().unwrap();
        migrate(&mut connection).unwrap();
        assert_eq!(version(&connection).unwrap(), latest_version());
        assert_eq!(
            tables(&connection),
            [
                "agent_activity",
                "messages",
                "notifications",
                "outbox",
                "sessions",
                "sqlite_sequence",
                "streaming_drafts",
                "sync_state"
            ]
        );
    }

    #[test]
    fn migrating_twice_changes_nothing() {
        let mut connection = Connection::open_in_memory().unwrap();
        migrate(&mut connection).unwrap();
        connection
            .execute("INSERT INTO sessions VALUES ('s1', 't', 'p', 1, 0)", [])
            .unwrap();
        migrate(&mut connection).unwrap();
        let count: i64 = connection
            .query_row("SELECT COUNT(*) FROM sessions", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn applies_only_the_missing_migrations_and_keeps_data() {
        let first = "CREATE TABLE things (id TEXT PRIMARY KEY);";
        let second = "ALTER TABLE things ADD COLUMN name TEXT NOT NULL DEFAULT '';";
        let mut connection = Connection::open_in_memory().unwrap();
        apply(&mut connection, &[first]).unwrap();
        connection
            .execute("INSERT INTO things (id) VALUES ('a')", [])
            .unwrap();

        apply(&mut connection, &[first, second]).unwrap();
        assert_eq!(version(&connection).unwrap(), 2);
        let name: String = connection
            .query_row("SELECT name FROM things WHERE id = 'a'", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(name, "");
    }

    #[test]
    fn a_failing_migration_leaves_the_previous_version_intact() {
        let mut connection = Connection::open_in_memory().unwrap();
        let good = "CREATE TABLE things (id TEXT PRIMARY KEY);";
        let bad = "CREATE TABLE more (id TEXT); THIS IS NOT SQL;";
        assert!(apply(&mut connection, &[good, bad]).is_err());
        assert_eq!(version(&connection).unwrap(), 1);
        assert!(!tables(&connection).contains(&"more".to_owned()));
    }

    #[test]
    fn refuses_a_database_from_a_newer_sdk() {
        let mut connection = Connection::open_in_memory().unwrap();
        connection
            .pragma_update(None, "user_version", latest_version() + 1)
            .unwrap();
        assert!(matches!(
            migrate(&mut connection),
            Err(NomiError::Storage { .. })
        ));
    }
}
