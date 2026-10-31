//! One account's SQLite database: the connection, transactions, and change notifications.

use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::Duration;

use rusqlite::{Connection, OpenFlags, TransactionBehavior};
use tokio::sync::watch;

use super::migrations;
use crate::error::NomiError;

/// How long a statement waits on a lock held by another connection (e.g. a backup tool).
const BUSY_TIMEOUT: Duration = Duration::from_secs(5);

/// The tables observers can watch. A write marks every table it touched; observers re-query when
/// one of theirs changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Table {
    Sessions,
    Messages,
    Drafts,
    Notifications,
    Activity,
}

impl Table {
    pub const ALL: [Table; 5] = [
        Table::Sessions,
        Table::Messages,
        Table::Drafts,
        Table::Notifications,
        Table::Activity,
    ];

    fn index(self) -> usize {
        self as usize
    }
}

/// Per-table write counters, bumped after each commit that touched the table.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct Versions([u64; Table::ALL.len()]);

impl Versions {
    pub fn of(&self, table: Table) -> u64 {
        self.0[table.index()]
    }
}

/// An open transaction that remembers which tables it wrote.
pub(crate) struct Tx<'c> {
    transaction: rusqlite::Transaction<'c>,
    touched: [bool; Table::ALL.len()],
}

impl Tx<'_> {
    /// The connection, for statements that write `table`.
    pub fn writes(&mut self, table: Table) -> &rusqlite::Transaction<'_> {
        self.touched[table.index()] = true;
        &self.transaction
    }
}

pub(crate) struct Database {
    path: PathBuf,
    connection: Mutex<Option<Connection>>,
    versions: watch::Sender<Versions>,
    closed: watch::Sender<bool>,
}

impl Database {
    /// Opens (creating if needed) and migrates the database at `path`.
    pub fn open(path: &Path) -> Result<Self, NomiError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let flags = OpenFlags::SQLITE_OPEN_READ_WRITE
            | OpenFlags::SQLITE_OPEN_CREATE
            | OpenFlags::SQLITE_OPEN_NO_MUTEX;
        let mut connection = Connection::open_with_flags(path, flags)?;
        connection.busy_timeout(BUSY_TIMEOUT)?;
        // WAL: readers don't block the writer; NORMAL is durable across app crashes in WAL mode.
        connection.pragma_update(None, "journal_mode", "WAL")?;
        connection.pragma_update(None, "synchronous", "NORMAL")?;
        connection.pragma_update(None, "foreign_keys", true)?;
        migrations::migrate(&mut connection)?;
        Ok(Self {
            path: path.to_owned(),
            connection: Mutex::new(Some(connection)),
            versions: watch::Sender::new(Versions::default()),
            closed: watch::Sender::new(false),
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Runs `body` against the connection outside any write transaction.
    pub fn read<R>(
        &self,
        body: impl FnOnce(&Connection) -> Result<R, NomiError>,
    ) -> Result<R, NomiError> {
        let guard = self.connection.lock().expect("database poisoned");
        let connection = guard.as_ref().ok_or(NomiError::AccountClosed)?;
        body(connection)
    }

    /// Runs `body` in one immediate transaction. It commits only if `body` succeeds; observers of
    /// the tables it wrote are told after the commit.
    pub fn write<R>(
        &self,
        body: impl FnOnce(&mut Tx<'_>) -> Result<R, NomiError>,
    ) -> Result<R, NomiError> {
        let (result, touched) = {
            let mut guard = self.connection.lock().expect("database poisoned");
            let connection = guard.as_mut().ok_or(NomiError::AccountClosed)?;
            let transaction =
                connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
            let mut tx = Tx {
                transaction,
                touched: [false; Table::ALL.len()],
            };
            let result = body(&mut tx)?;
            let touched = tx.touched;
            tx.transaction.commit()?;
            (result, touched)
        };
        if touched.contains(&true) {
            self.versions.send_modify(|versions| {
                for (version, touched) in versions.0.iter_mut().zip(touched) {
                    if touched {
                        *version += 1;
                    }
                }
            });
        }
        Ok(result)
    }

    pub fn schema_version(&self) -> Result<u32, NomiError> {
        self.read(migrations::version)
    }

    pub fn watch_versions(&self) -> watch::Receiver<Versions> {
        self.versions.subscribe()
    }

    pub fn watch_closed(&self) -> watch::Receiver<bool> {
        self.closed.subscribe()
    }

    /// Closes the connection; later calls fail with `AccountClosed` and observers end.
    pub fn close(&self) {
        if let Some(connection) = self.connection.lock().expect("database poisoned").take() {
            if let Err((_, error)) = connection.close() {
                tracing::warn!(%error, "closing the database failed");
            }
        }
        self.closed.send_replace(true);
    }
}

impl Drop for Database {
    fn drop(&mut self) {
        self.close();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn open() -> (tempfile::TempDir, Database) {
        let dir = tempfile::tempdir().unwrap();
        let database = Database::open(&dir.path().join("nested").join("nomi.db")).unwrap();
        (dir, database)
    }

    fn insert_session(tx: &mut Tx<'_>, id: &str) -> Result<(), NomiError> {
        tx.writes(Table::Sessions)
            .execute("INSERT INTO sessions VALUES (?1, '', '', 0, 0)", [id])?;
        Ok(())
    }

    fn session_count(database: &Database) -> i64 {
        database
            .read(|c| Ok(c.query_row("SELECT COUNT(*) FROM sessions", [], |row| row.get(0))?))
            .unwrap()
    }

    #[test]
    fn opens_migrated_in_wal_mode() {
        let (_dir, database) = open();
        assert_eq!(
            database.schema_version().unwrap(),
            migrations::latest_version()
        );
        let mode: String = database
            .read(|c| Ok(c.query_row("PRAGMA journal_mode", [], |row| row.get(0))?))
            .unwrap();
        assert_eq!(mode, "wal");
    }

    #[test]
    fn a_committed_write_bumps_only_the_tables_it_touched() {
        let (_dir, database) = open();
        let versions = database.watch_versions();
        database.write(|tx| insert_session(tx, "s1")).unwrap();
        let after = *versions.borrow();
        assert_eq!(after.of(Table::Sessions), 1);
        assert_eq!(after.of(Table::Messages), 0);
    }

    #[test]
    fn a_failed_write_rolls_back_and_notifies_nobody() {
        let (_dir, database) = open();
        let versions = database.watch_versions();
        let result: Result<(), NomiError> = database.write(|tx| {
            insert_session(tx, "s1")?;
            Err(NomiError::Internal {
                reason: "boom".into(),
            })
        });
        assert!(result.is_err());
        assert_eq!(session_count(&database), 0);
        assert_eq!(versions.borrow().of(Table::Sessions), 0);
    }

    #[test]
    fn a_read_only_transaction_notifies_nobody() {
        let (_dir, database) = open();
        let versions = database.watch_versions();
        database.write(|_| Ok(())).unwrap();
        assert!(!versions.has_changed().unwrap());
    }

    #[test]
    fn data_survives_reopening() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nomi.db");
        Database::open(&path)
            .unwrap()
            .write(|tx| insert_session(tx, "s1"))
            .unwrap();
        assert_eq!(session_count(&Database::open(&path).unwrap()), 1);
    }

    #[test]
    fn calls_after_close_fail_and_watchers_hear_it() {
        let (_dir, database) = open();
        let closed = database.watch_closed();
        database.close();
        assert!(*closed.borrow());
        assert_eq!(
            database.write(|tx| insert_session(tx, "s1")),
            Err(NomiError::AccountClosed)
        );
        assert_eq!(database.schema_version(), Err(NomiError::AccountClosed));
    }
}
