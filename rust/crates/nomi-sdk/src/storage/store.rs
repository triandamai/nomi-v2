use std::sync::Arc;

use rusqlite::Connection;
use tokio::runtime::Handle;

use super::database::{Database, Table, Tx};
use super::models::{
    AgentActivity, DatabaseStatus, Message, MessageCursor, MessagePage, NotificationItem,
    OutboxEntry, Session, SessionFilter,
};
use super::observation::{
    AgentActivityObservation, DraftObservation, DraftState, MessagesObservation,
    NotificationsObservation, Observer, OutboxObservation, SessionsObservation,
    UnreadCountObservation,
};
use super::repositories::{activity, drafts, messages, notifications, outbox, sessions};
use crate::error::NomiError;

/// SQLite's write-ahead log, next to the database file.
const WAL_SUFFIX: &str = "-wal";

/// One signed-in account's local data. Every read and write goes through here; Kotlin never sees
/// SQL. It stops working (`NomiError::AccountClosed`, observations end) once the SDK closes the
/// account, switches to another, or shuts down.
#[derive(uniffi::Object)]
pub struct AccountStore {
    account_id: String,
    database: Arc<Database>,
    runtime: Handle,
}

impl AccountStore {
    pub(crate) fn new(account_id: String, database: Database, runtime: Handle) -> Self {
        Self {
            account_id,
            database: Arc::new(database),
            runtime,
        }
    }

    pub(crate) fn database(&self) -> &Database {
        &self.database
    }

    async fn run<R: Send + 'static>(
        &self,
        body: impl FnOnce(&Database) -> Result<R, NomiError> + Send + 'static,
    ) -> Result<R, NomiError> {
        let database = self.database.clone();
        self.runtime
            .spawn_blocking(move || body(&database))
            .await
            .map_err(|_| NomiError::ShutDown)?
    }

    pub(crate) async fn read<R: Send + 'static>(
        &self,
        body: impl FnOnce(&Connection) -> Result<R, NomiError> + Send + 'static,
    ) -> Result<R, NomiError> {
        self.run(move |database| database.read(body)).await
    }

    pub(crate) async fn write<R: Send + 'static>(
        &self,
        body: impl FnOnce(&mut Tx<'_>) -> Result<R, NomiError> + Send + 'static,
    ) -> Result<R, NomiError> {
        self.run(move |database| database.write(body)).await
    }

    fn observer<T: Clone + PartialEq + Send + 'static>(
        &self,
        tables: &[Table],
        query: impl Fn(&Connection) -> Result<T, NomiError> + Send + Sync + 'static,
    ) -> Observer<T> {
        Observer::new(self.database.clone(), self.runtime.clone(), tables, query)
    }
}

#[uniffi::export]
impl AccountStore {
    pub fn account_id(&self) -> String {
        self.account_id.clone()
    }

    pub async fn database_status(&self) -> Result<DatabaseStatus, NomiError> {
        let account_id = self.account_id.clone();
        self.run(move |database| {
            let path = database.path().to_owned();
            let mut wal = path.clone().into_os_string();
            wal.push(WAL_SUFFIX);
            let size_bytes = [path.as_os_str(), wal.as_os_str()]
                .iter()
                .filter_map(|file| std::fs::metadata(file).ok())
                .map(|metadata| metadata.len())
                .sum();
            Ok(DatabaseStatus {
                account_id,
                path: path.display().to_string(),
                schema_version: database.schema_version()?,
                size_bytes,
            })
        })
        .await
    }

    /// Empties every table in one transaction, keeping the account open. Writes still in the
    /// outbox are dropped with it.
    pub async fn clear_all(&self) -> Result<(), NomiError> {
        self.write(|tx| {
            tx.writes(Table::Outbox).execute("DELETE FROM outbox", [])?;
            tx.raw().execute("DELETE FROM sync_state", [])?;
            tx.writes(Table::Activity)
                .execute("DELETE FROM agent_activity", [])?;
            tx.writes(Table::Drafts)
                .execute("DELETE FROM streaming_drafts", [])?;
            tx.writes(Table::Messages)
                .execute("DELETE FROM messages", [])?;
            tx.writes(Table::Notifications)
                .execute("DELETE FROM notifications", [])?;
            tx.writes(Table::Sessions)
                .execute("DELETE FROM sessions", [])?;
            Ok(())
        })
        .await
    }

    // Sessions

    pub async fn upsert_sessions(&self, sessions: Vec<Session>) -> Result<(), NomiError> {
        self.write(move |tx| sessions::upsert(tx, &sessions)).await
    }

    pub async fn get_session(&self, id: String) -> Result<Option<Session>, NomiError> {
        self.read(move |connection| sessions::get(connection, &id))
            .await
    }

    pub async fn mark_session_read(&self, id: String) -> Result<(), NomiError> {
        self.write(move |tx| sessions::mark_read(tx, &id)).await
    }

    /// Removes the chat with its messages, draft and agent activity.
    pub async fn delete_session(&self, id: String) -> Result<(), NomiError> {
        self.write(move |tx| sessions::delete(tx, &id)).await
    }

    pub fn observe_sessions(&self, filter: SessionFilter) -> Arc<SessionsObservation> {
        Arc::new(SessionsObservation(
            self.observer(&[Table::Sessions], move |connection| {
                sessions::list(connection, &filter)
            }),
        ))
    }

    // Messages

    pub async fn upsert_messages(&self, messages: Vec<Message>) -> Result<(), NomiError> {
        self.write(move |tx| messages::upsert(tx, &messages)).await
    }

    pub async fn delete_message(&self, id: String) -> Result<(), NomiError> {
        self.write(move |tx| messages::delete(tx, &id)).await
    }

    /// Up to `limit` messages before `before` (the newest without it), oldest first.
    pub async fn messages_page(
        &self,
        session_id: String,
        before: Option<MessageCursor>,
        limit: u32,
    ) -> Result<MessagePage, NomiError> {
        self.read(move |connection| messages::page(connection, &session_id, before.as_ref(), limit))
            .await
    }

    /// The newest `limit` messages of the chat, oldest first. Raise `limit` to load older ones.
    pub fn observe_messages(&self, session_id: String, limit: u32) -> Arc<MessagesObservation> {
        Arc::new(MessagesObservation(
            self.observer(&[Table::Messages], move |connection| {
                Ok(messages::page(connection, &session_id, None, limit)?.messages)
            }),
        ))
    }

    // Streaming drafts

    /// Appends a streamed delta to the chat's draft; a draft from another turn is replaced.
    pub async fn append_draft(
        &self,
        session_id: String,
        turn_job_id: String,
        text: String,
        updated_at: i64,
    ) -> Result<(), NomiError> {
        self.write(move |tx| drafts::append(tx, &session_id, &turn_job_id, &text, updated_at))
            .await
    }

    pub async fn set_draft_phase(
        &self,
        session_id: String,
        turn_job_id: String,
        phase: Option<String>,
        updated_at: i64,
    ) -> Result<(), NomiError> {
        self.write(move |tx| {
            drafts::set_phase(tx, &session_id, &turn_job_id, phase.as_deref(), updated_at)
        })
        .await
    }

    /// Removes the chat's draft; with `turn_job_id`, only if it belongs to that turn.
    pub async fn clear_draft(
        &self,
        session_id: String,
        turn_job_id: Option<String>,
    ) -> Result<(), NomiError> {
        self.write(move |tx| drafts::clear(tx, &session_id, turn_job_id.as_deref()))
            .await
    }

    pub fn observe_draft(&self, session_id: String) -> Arc<DraftObservation> {
        Arc::new(DraftObservation(self.observer(
            &[Table::Drafts],
            move |connection| {
                Ok(DraftState {
                    draft: drafts::get(connection, &session_id)?,
                })
            },
        )))
    }

    // Notifications

    /// Stores notifications; one already read here stays read.
    pub async fn upsert_notifications(
        &self,
        notifications: Vec<NotificationItem>,
    ) -> Result<(), NomiError> {
        self.write(move |tx| notifications::upsert(tx, &notifications))
            .await
    }

    pub async fn mark_all_notifications_read(&self, read_at: i64) -> Result<(), NomiError> {
        self.write(move |tx| notifications::mark_all_read(tx, read_at))
            .await
    }

    pub fn observe_notifications(&self, limit: u32) -> Arc<NotificationsObservation> {
        Arc::new(NotificationsObservation(
            self.observer(&[Table::Notifications], move |connection| {
                notifications::list(connection, limit)
            }),
        ))
    }

    pub fn observe_unread_notification_count(&self) -> Arc<UnreadCountObservation> {
        Arc::new(UnreadCountObservation(
            self.observer(&[Table::Notifications], notifications::unread_count),
        ))
    }

    // Agent activity

    pub async fn upsert_agent_activity(&self, activity: AgentActivity) -> Result<(), NomiError> {
        self.write(move |tx| activity::upsert(tx, &activity)).await
    }

    pub async fn delete_agent_activity(&self, agent_session_id: String) -> Result<(), NomiError> {
        self.write(move |tx| activity::delete(tx, &agent_session_id))
            .await
    }

    pub async fn clear_agent_activity(&self, session_id: String) -> Result<(), NomiError> {
        self.write(move |tx| activity::clear_for_session(tx, &session_id))
            .await
    }

    /// Agents doing something (not idle), in every chat or only in `session_id`.
    pub fn observe_active_agents(
        &self,
        session_id: Option<String>,
    ) -> Arc<AgentActivityObservation> {
        Arc::new(AgentActivityObservation(
            self.observer(&[Table::Activity], move |connection| {
                activity::active(connection, session_id.as_deref())
            }),
        ))
    }

    // Outbox

    /// Writes waiting for the backend, or that it refused, oldest first.
    pub async fn outbox_entries(&self) -> Result<Vec<OutboxEntry>, NomiError> {
        self.read(outbox::list).await
    }

    pub fn observe_outbox(&self) -> Arc<OutboxObservation> {
        Arc::new(OutboxObservation(
            self.observer(&[Table::Outbox], outbox::list),
        ))
    }
}
