//! Local data: one SQLite database per account under `<data_dir>/accounts/<account_id>/`.
//!
//! Only one account is open at a time. Opening another closes the previous one, whose
//! [`AccountStore`] handles stop working, so no screen can read or write the wrong account.

mod database;
mod migrations;
pub(crate) mod models;
pub(crate) mod observation;
pub(crate) mod repositories;
mod store;
#[cfg(test)]
mod tests;

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use tokio::runtime::Handle;

pub(crate) use database::Tx;
pub use store::AccountStore;

use crate::error::NomiError;
use database::Database;

const ACCOUNTS_DIR: &str = "accounts";
const DATABASE_FILE: &str = "nomi.db";
const MAX_ACCOUNT_ID_LEN: usize = 128;

pub(crate) struct Accounts {
    root: PathBuf,
    current: Mutex<Option<Arc<AccountStore>>>,
}

impl Accounts {
    pub fn new(data_dir: &Path) -> Self {
        Self {
            root: data_dir.join(ACCOUNTS_DIR),
            current: Mutex::new(None),
        }
    }

    pub fn current(&self) -> Option<Arc<AccountStore>> {
        self.lock().clone()
    }

    /// The open store for `account_id`, opening (and migrating) its database if needed and
    /// closing any other account first. Blocking: call it off the async threads.
    pub fn open(&self, account_id: &str, runtime: Handle) -> Result<Arc<AccountStore>, NomiError> {
        let directory = self.directory(account_id)?;
        let mut current = self.lock();
        if let Some(store) = current
            .as_ref()
            .filter(|store| store.account_id() == account_id)
        {
            return Ok(store.clone());
        }
        if let Some(previous) = current.take() {
            previous.database().close();
        }
        let database = Database::open(&directory.join(DATABASE_FILE))?;
        let store = Arc::new(AccountStore::new(account_id.to_owned(), database, runtime));
        *current = Some(store.clone());
        Ok(store)
    }

    /// Closes the open account, keeping its data. Returns whether one was open.
    pub fn close_current(&self) -> bool {
        let previous = self.lock().take();
        if let Some(store) = &previous {
            store.database().close();
        }
        previous.is_some()
    }

    /// Deletes `account_id`'s data, closing it first if it is open. Returns whether it was open.
    pub fn remove(&self, account_id: &str) -> Result<bool, NomiError> {
        let directory = self.directory(account_id)?;
        let was_open = self
            .current()
            .is_some_and(|store| store.account_id() == account_id)
            && self.close_current();
        remove_dir_if_present(&directory)?;
        Ok(was_open)
    }

    /// Deletes every account's data. Returns whether an account was open.
    pub fn remove_all(&self) -> Result<bool, NomiError> {
        let was_open = self.close_current();
        remove_dir_if_present(&self.root)?;
        Ok(was_open)
    }

    fn directory(&self, account_id: &str) -> Result<PathBuf, NomiError> {
        let valid = !account_id.is_empty()
            && account_id.len() <= MAX_ACCOUNT_ID_LEN
            && !account_id.starts_with('.')
            && account_id
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'));
        if !valid {
            return Err(NomiError::InvalidArgument {
                reason: format!(
                    "account id must be 1..={MAX_ACCOUNT_ID_LEN} of A-Z a-z 0-9 - _ . and not start with '.', got `{account_id}`"
                ),
            });
        }
        Ok(self.root.join(account_id))
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Option<Arc<AccountStore>>> {
        self.current.lock().expect("accounts poisoned")
    }
}

fn remove_dir_if_present(directory: &Path) -> Result<(), NomiError> {
    match std::fs::remove_dir_all(directory) {
        Err(error) if error.kind() != std::io::ErrorKind::NotFound => Err(error.into()),
        _ => Ok(()),
    }
}
