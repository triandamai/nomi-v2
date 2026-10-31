//! Snapshot-plus-change streams over the database.
//!
//! An observation answers its query once, then again after every commit that touched one of its
//! tables, skipping answers equal to the previous one. Bursts of writes coalesce: a slow consumer
//! gets the latest state, never a backlog. Nothing polls; a pending `next` waits on the change
//! counters.

use std::sync::Arc;

use rusqlite::Connection;
use tokio::runtime::Handle;
use tokio::sync::watch;

use super::database::{Database, Table, Versions};
use super::models::{AgentActivity, Draft, Message, NotificationItem, Session};
use crate::error::NomiError;

type Query<T> = Arc<dyn Fn(&Connection) -> Result<T, NomiError> + Send + Sync>;

pub(crate) struct Observer<T> {
    database: Arc<Database>,
    runtime: Handle,
    tables: Vec<Table>,
    query: Query<T>,
    state: tokio::sync::Mutex<State<T>>,
    cancelled: watch::Sender<bool>,
}

struct State<T> {
    versions: watch::Receiver<Versions>,
    closed: watch::Receiver<bool>,
    /// The counters the last answer was computed at; `None` before the first answer.
    seen: Option<Versions>,
    last: Option<T>,
}

impl<T: Clone + PartialEq + Send + 'static> Observer<T> {
    pub fn new(
        database: Arc<Database>,
        runtime: Handle,
        tables: &[Table],
        query: impl Fn(&Connection) -> Result<T, NomiError> + Send + Sync + 'static,
    ) -> Self {
        let state = State {
            versions: database.watch_versions(),
            closed: database.watch_closed(),
            seen: None,
            last: None,
        };
        Self {
            database,
            runtime,
            tables: tables.to_vec(),
            query: Arc::new(query),
            state: tokio::sync::Mutex::new(state),
            cancelled: watch::Sender::new(false),
        }
    }

    /// The current answer first, then the next different one; `None` once the account closes or
    /// the observation is cancelled.
    pub async fn next(&self) -> Result<Option<T>, NomiError> {
        let mut cancelled = self.cancelled.subscribe();
        let mut guard = self.state.lock().await;
        let state = &mut *guard;
        loop {
            if let Some(seen) = state.seen {
                loop {
                    if *cancelled.borrow() || *state.closed.borrow() {
                        return Ok(None);
                    }
                    let current = *state.versions.borrow_and_update();
                    if self
                        .tables
                        .iter()
                        .any(|&table| current.of(table) != seen.of(table))
                    {
                        break;
                    }
                    tokio::select! {
                        biased;
                        _ = cancelled.wait_for(|cancelled| *cancelled) => return Ok(None),
                        _ = state.closed.wait_for(|closed| *closed) => return Ok(None),
                        changed = state.versions.changed() => if changed.is_err() { return Ok(None) },
                    }
                }
            } else if *cancelled.borrow() || *state.closed.borrow() {
                return Ok(None);
            }
            // Mark the counters seen before querying, so a commit during the query isn't missed.
            state.seen = Some(*state.versions.borrow_and_update());
            let answer = match self.answer().await {
                Err(NomiError::AccountClosed) => return Ok(None),
                other => other?,
            };
            if state.last.as_ref() != Some(&answer) {
                state.last = Some(answer.clone());
                return Ok(Some(answer));
            }
        }
    }

    pub fn cancel(&self) {
        self.cancelled.send_replace(true);
    }

    async fn answer(&self) -> Result<T, NomiError> {
        let (database, query) = (self.database.clone(), self.query.clone());
        self.runtime
            .spawn_blocking(move || database.read(|connection| query(connection)))
            .await
            .map_err(|_| NomiError::ShutDown)?
    }
}

/// Declares an observation type Kotlin can pull from (`next`) and stop (`cancel`).
macro_rules! observation {
    ($(#[$doc:meta])* $name:ident => $item:ty) => {
        $(#[$doc])*
        #[derive(uniffi::Object)]
        pub struct $name(pub(crate) Observer<$item>);

        #[uniffi::export]
        impl $name {
            /// The current answer first, then the next different one after a change; `None`
            /// once the account store closes or the observation is cancelled.
            pub async fn next(&self) -> Result<Option<$item>, NomiError> {
                self.0.next().await
            }

            /// Ends the stream; a pending `next` returns `None`.
            pub fn cancel(&self) {
                self.0.cancel()
            }
        }
    };
}

/// A chat's streaming draft, or its absence. A record, so Kotlin can tell "no draft" from "ended".
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct DraftState {
    pub draft: Option<Draft>,
}

observation!(
    /// The chat list for a [`super::models::SessionFilter`].
    SessionsObservation => Vec<Session>
);
observation!(
    /// The newest messages of one chat, oldest first.
    MessagesObservation => Vec<Message>
);
observation!(
    /// One chat's streaming draft.
    DraftObservation => DraftState
);
observation!(
    /// The newest notifications.
    NotificationsObservation => Vec<NotificationItem>
);
observation!(
    /// How many notifications are unread.
    UnreadCountObservation => u32
);
observation!(
    /// Agents doing something, most recently updated first.
    AgentActivityObservation => Vec<AgentActivity>
);
