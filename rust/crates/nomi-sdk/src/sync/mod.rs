//! Sync: keeps the open account's local data in step with the backend.
//!
//! - **Reads** come from REST and land in storage: the chat list, a chat's newest page and older
//!   pages, the inbox. Paging cursors live in `sync_state`, so they survive restarts.
//! - **Realtime** events (`SdkEvent::Realtime`) are applied to storage in order, one lane per topic,
//!   so a slow fetch in one chat never holds up another. A resync refetches what may be stale.
//! - **Writes** (sending a message, marking notifications read) go through a durable outbox: shown
//!   locally at once, retried with backoff until the backend has them, replayed after a restart.
//!
//! One engine runs per open account, and only does network work while that account's user is
//! signed in.

pub(crate) mod api;
mod outbox;
mod realtime;
#[cfg(test)]
mod tests;

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use tokio::runtime::Handle;
use tokio::sync::{mpsc, Notify};
use tokio::task::JoinHandle;

use crate::error::NomiError;
use crate::event::EventBus;
use crate::logging;
use crate::network::auth::Auth;
use crate::network::events::chat_of;
use crate::network::realtime::{normalize_topic, Realtime, TopicWatch};
use crate::storage::models::{Message, OutboxOperation, STATUS_SENDING};
use crate::storage::repositories::{
    messages, notifications, outbox as queue, sessions, sync_state,
};
use crate::storage::AccountStore;
use api::Api;
use realtime::Work;

/// Messages fetched per page; the web client's `MESSAGE_PAGE_SIZE` (the server allows 1..=100).
pub(crate) const MESSAGE_PAGE_SIZE: u32 = 50;
/// The longest message the backend takes, in characters (`MAX_MESSAGE_CHARS`).
const MAX_MESSAGE_CHARS: usize = 10_000;
const FIRST_NOTIFICATION_PAGE: u32 = 1;
const USER_TOPIC: &str = "user";
const NOTIFICATIONS_LANE: &str = "notifications";
const LOCAL_ID_PREFIX: &str = "local-";
const ROLE_USER: &str = "user";
const OLDER_KEY_PREFIX: &str = "older:";
const NOTIFICATION_PAGE_KEY: &str = "notifications.page";
const NOTIFICATION_PAGES_KEY: &str = "notifications.pages";

/// How sync retries and how long it remembers. Tests shorten these.
#[derive(Debug, Clone)]
pub(crate) struct SyncSettings {
    /// The first retry's ceiling; each retry doubles it, with full jitter.
    pub retry_base: Duration,
    pub retry_max: Duration,
    /// Attempts before an entry is marked failed.
    pub max_attempts: u32,
    /// How far before its entry was queued a message found on the server may have been created
    /// (clock skew) and still count as that entry's.
    pub echo_window: Duration,
    /// How long finished outbox entries are kept as idempotency records.
    pub done_retention: Duration,
}

impl Default for SyncSettings {
    fn default() -> Self {
        Self {
            retry_base: Duration::from_secs(1),
            retry_max: Duration::from_secs(5 * 60),
            max_attempts: 8,
            echo_window: Duration::from_secs(5 * 60),
            done_retention: Duration::from_secs(24 * 60 * 60),
        }
    }
}

/// Keeps one chat in sync while held: its topic is watched and its events applied. The first
/// binding of a chat resyncs it; later ones refresh. Released explicitly, or when dropped.
#[derive(uniffi::Object)]
pub struct ChatBinding {
    session_id: String,
    shared: Arc<Shared>,
    watch: TopicWatch,
    released: AtomicBool,
}

#[uniffi::export]
impl ChatBinding {
    pub fn session_id(&self) -> String {
        self.session_id.clone()
    }

    /// Stops keeping the chat in sync. Idempotent.
    pub fn release(&self) {
        if !self.released.swap(true, Ordering::SeqCst) {
            self.watch.release();
            self.shared.unbind(&self.session_id);
        }
    }
}

impl Drop for ChatBinding {
    fn drop(&mut self) {
        self.release();
    }
}

/// The sync engine for one open account. Stopped when the account closes.
pub(crate) struct SyncEngine {
    shared: Arc<Shared>,
    tasks: Vec<JoinHandle<()>>,
    _user: TopicWatch,
}

pub(crate) struct Shared {
    store: Arc<AccountStore>,
    api: Api,
    realtime: Realtime,
    bus: Arc<EventBus>,
    runtime: Handle,
    settings: SyncSettings,
    foreground: Arc<Mutex<Option<String>>>,
    /// Wakes the outbox worker: something was queued, retried, or the connection came back.
    wake: Notify,
    /// Bindings held per chat.
    bound: Mutex<HashMap<String, usize>>,
    lanes: Mutex<HashMap<String, mpsc::UnboundedSender<Work>>>,
    lane_tasks: Mutex<Vec<JoinHandle<()>>>,
    stopped: AtomicBool,
}

impl SyncEngine {
    pub fn start(
        store: Arc<AccountStore>,
        auth: Arc<Auth>,
        realtime: Realtime,
        bus: Arc<EventBus>,
        runtime: Handle,
        settings: SyncSettings,
        foreground: Arc<Mutex<Option<String>>>,
    ) -> Self {
        // Subscribed before anything is spawned, so no event in between is missed.
        let events = bus.subscribe();
        let user = realtime.watch(USER_TOPIC.to_owned());
        let shared = Arc::new(Shared {
            store,
            api: Api::new(auth),
            realtime,
            bus,
            runtime: runtime.clone(),
            settings,
            foreground,
            wake: Notify::new(),
            bound: Mutex::new(HashMap::new()),
            lanes: Mutex::new(HashMap::new()),
            lane_tasks: Mutex::new(Vec::new()),
            stopped: AtomicBool::new(false),
        });
        let tasks = vec![
            runtime.spawn(realtime::dispatch(shared.clone(), events)),
            runtime.spawn(outbox::run(shared.clone())),
        ];
        tracing::info!(account_id = %shared.store.account_id(), "sync started");
        Self {
            shared,
            tasks,
            _user: user,
        }
    }

    pub fn account_id(&self) -> String {
        self.shared.store.account_id()
    }

    pub fn shared(&self) -> Arc<Shared> {
        self.shared.clone()
    }
}

impl Drop for SyncEngine {
    fn drop(&mut self) {
        self.shared.stopped.store(true, Ordering::SeqCst);
        for task in &self.tasks {
            task.abort();
        }
        for task in self
            .shared
            .lane_tasks
            .lock()
            .expect("lanes poisoned")
            .drain(..)
        {
            task.abort();
        }
        self.shared.lanes.lock().expect("lanes poisoned").clear();
        tracing::info!(account_id = %self.shared.store.account_id(), "sync stopped");
    }
}

impl Shared {
    fn now() -> i64 {
        logging::now_ms() as i64
    }

    fn account_id(&self) -> String {
        self.store.account_id()
    }

    /// Network work is for the account's own user only.
    fn ensure_owner(&self) -> Result<(), NomiError> {
        match self.api.auth().user() {
            None => Err(NomiError::NotSignedIn),
            Some(user) if user.user_id == self.account_id() => Ok(()),
            Some(user) => Err(NomiError::InvalidArgument {
                reason: format!(
                    "the open account `{}` isn't the signed-in user `{}`",
                    self.account_id(),
                    user.user_id
                ),
            }),
        }
    }

    fn is_owner(&self) -> bool {
        self.ensure_owner().is_ok()
    }

    fn is_foreground(&self, session_id: &str) -> bool {
        self.foreground
            .lock()
            .expect("foreground poisoned")
            .as_deref()
            == Some(session_id)
    }

    // Chats and messages

    /// Replaces the chat list with the server's. A chat the server no longer lists was deleted
    /// elsewhere and goes, unless the outbox still owes it a message.
    pub async fn refresh_sessions(&self) -> Result<(), NomiError> {
        self.ensure_owner()?;
        let listed = self
            .api
            .sessions()
            .await?
            .into_iter()
            .map(api::SessionDto::into_session)
            .collect::<Result<Vec<_>, _>>()?;
        self.store
            .write(move |tx| {
                sessions::merge_from_server(tx, &listed)?;
                for id in sessions::ids(tx.raw())? {
                    let gone = !listed.iter().any(|session| session.id == id);
                    if gone && !queue::lane_busy(tx.raw(), &chat_lane(&id))? {
                        sessions::delete(tx, &id)?;
                    }
                }
                Ok(())
            })
            .await
    }

    pub async fn create_session(&self) -> Result<String, NomiError> {
        self.ensure_owner()?;
        Ok(self.api.create_session().await?.to_lowercase())
    }

    /// Loads the chat's newest page. The first load also records whether older messages exist.
    /// The newest page is the whole tail of the chat: a stored message newer than its oldest that it
    /// lacks was deleted on the server (a reply a regenerate replaced, a delete from another device),
    /// up to when the page was asked for, so a message arriving meanwhile stays.
    pub async fn refresh_messages(&self, session_id: &str) -> Result<(), NomiError> {
        self.ensure_owner()?;
        let asked_at = Self::now();
        let page = self
            .api
            .messages(session_id, None, MESSAGE_PAGE_SIZE)
            .await?;
        let has_more = page.has_more;
        let empty = page.messages.is_empty();
        let session = session_id.to_owned();
        self.store_page(session_id, page, Some(asked_at), move |tx| {
            // The server has nothing left in this chat: what was sent before is gone.
            if empty && !has_more {
                messages::remove_missing(tx, &session, i64::MIN, asked_at, &[])?;
            }
            let key = older_key(&session);
            if sync_state::get(tx.raw(), &key)?.is_none() {
                sync_state::set(tx, &key, i64::from(has_more))?;
            }
            Ok(())
        })
        .await
    }

    /// Loads the page before the oldest stored message; returns whether even older ones exist.
    pub async fn load_older_messages(&self, session_id: &str) -> Result<bool, NomiError> {
        self.ensure_owner()?;
        let session = session_id.to_owned();
        let Some(cursor) = self
            .store
            .read(move |connection| messages::oldest_sent(connection, &session))
            .await?
        else {
            self.refresh_messages(session_id).await?;
            return self.has_older_messages(session_id).await;
        };
        let page = self
            .api
            .messages(session_id, Some(&cursor.id), MESSAGE_PAGE_SIZE)
            .await?;
        let has_more = page.has_more;
        let session = session_id.to_owned();
        self.store_page(session_id, page, Some(cursor.created_at), move |tx| {
            sync_state::set(tx, &older_key(&session), i64::from(has_more))
        })
        .await?;
        Ok(has_more)
    }

    /// Whether older messages may exist on the server; true until a load says otherwise.
    pub async fn has_older_messages(&self, session_id: &str) -> Result<bool, NomiError> {
        let key = older_key(session_id);
        let stored = self
            .store
            .read(move |connection| sync_state::get(connection, &key))
            .await?;
        Ok(stored.is_none_or(|value| value != 0))
    }

    /// Stores a page, drops what the server no longer has inside the span the page covers (up to
    /// `until`: the message an older page was fetched before, or when the newest page was asked
    /// for; never short of the page's newest), then runs `also` in the same transaction.
    async fn store_page(
        &self,
        session_id: &str,
        page: api::MessagePageDto,
        until: Option<i64>,
        also: impl FnOnce(&mut crate::storage::Tx<'_>) -> Result<(), NomiError> + Send + 'static,
    ) -> Result<(), NomiError> {
        let stored = page
            .messages
            .into_iter()
            .map(|dto| dto.into_message(session_id))
            .collect::<Result<Vec<_>, _>>()?;
        let session = session_id.to_owned();
        self.store
            .write(move |tx| {
                messages::upsert(tx, &stored)?;
                if let (Some(oldest), Some(newest)) = (stored.first(), stored.last()) {
                    let keep: Vec<String> = stored.iter().map(|m| m.id.clone()).collect();
                    let until =
                        until.map_or(newest.created_at, |until| until.max(newest.created_at));
                    messages::remove_missing(tx, &session, oldest.created_at, until, &keep)?;
                }
                also(tx)
            })
            .await
    }

    /// Fetches one message into storage. A message the server no longer has is removed locally
    /// (`Ok(None)`).
    async fn fetch_message(
        &self,
        session_id: &str,
        message_id: &str,
    ) -> Result<Option<Message>, NomiError> {
        match self.api.message(session_id, message_id).await {
            Ok(dto) => {
                let message = dto.into_message(session_id)?;
                let stored = message.clone();
                self.store
                    .write(move |tx| messages::upsert(tx, std::slice::from_ref(&stored)))
                    .await?;
                Ok(Some(message))
            }
            Err(NomiError::Http { status, .. }) if status == HTTP_NOT_FOUND => {
                let id = message_id.to_lowercase();
                self.store
                    .write(move |tx| messages::delete(tx, &id))
                    .await?;
                Ok(None)
            }
            Err(error) => Err(error),
        }
    }

    /// Queues the person's message and shows it at once as `sending`. Returns its local id.
    pub async fn send_message(&self, session_id: &str, text: String) -> Result<String, NomiError> {
        if text.trim().is_empty() {
            return Err(NomiError::InvalidArgument {
                reason: "a message needs some text".into(),
            });
        }
        let length = text.chars().count();
        if length > MAX_MESSAGE_CHARS {
            return Err(NomiError::InvalidArgument {
                reason: format!(
                    "a message holds at most {MAX_MESSAGE_CHARS} characters, this one has {length}"
                ),
            });
        }
        let key = new_key();
        let local_message_id = format!("{LOCAL_ID_PREFIX}{key}");
        let operation = OutboxOperation::SendMessage {
            session_id: session_id.to_owned(),
            text: text.clone(),
            local_message_id: local_message_id.clone(),
        };
        let now = Self::now();
        let local = Message {
            id: local_message_id.clone(),
            session_id: session_id.to_owned(),
            role: ROLE_USER.to_owned(),
            agent_type: None,
            agent_display_name: None,
            content: text,
            content_json: None,
            my_feedback: None,
            steps_json: None,
            created_at: now,
            status: STATUS_SENDING.to_owned(),
        };
        let lane = chat_lane(session_id);
        let session = session_id.to_owned();
        self.store
            .write(move |tx| {
                queue::insert(tx, &key, &lane, &operation, now)?;
                messages::upsert(tx, std::slice::from_ref(&local))?;
                sessions::touch(tx, &session, Some(&local.content), now, None)
            })
            .await?;
        self.wake.notify_one();
        Ok(local_message_id)
    }

    /// Decides a pending approval card. Not queued: it only means something while the turn waits.
    pub async fn resolve_approval(
        &self,
        session_id: &str,
        message_id: &str,
        approve: bool,
        remember: bool,
    ) -> Result<(), NomiError> {
        self.ensure_owner()?;
        self.api
            .resolve_approval(session_id, message_id, approve, remember)
            .await
    }

    /// Rates a reply on the backend, then here. `None` takes the rating back.
    pub async fn rate_message(
        &self,
        session_id: &str,
        message_id: &str,
        rating: Option<String>,
        reasons: Vec<String>,
        note: Option<String>,
        include_chat: bool,
    ) -> Result<(), NomiError> {
        self.ensure_owner()?;
        self.api
            .rate_message(
                session_id,
                message_id,
                rating.as_deref(),
                &reasons,
                note.as_deref(),
                include_chat,
            )
            .await?;
        let id = message_id.to_lowercase();
        self.store
            .write(move |tx| messages::set_feedback(tx, &id, rating.as_deref()))
            .await
    }

    /// Deletes a message on the backend, then the messages that went with it here.
    pub async fn delete_message(
        &self,
        session_id: &str,
        message_id: &str,
    ) -> Result<(), NomiError> {
        self.ensure_owner()?;
        let deleted = match self.api.delete_message(session_id, message_id).await {
            Ok(deleted) => deleted.message_ids,
            // Already gone there (deleted elsewhere, or replaced by a regenerate): drop it here and
            // resync, which takes whatever went with it.
            Err(NomiError::Http { status, .. }) if status == HTTP_NOT_FOUND => {
                vec![message_id.to_owned()]
            }
            Err(error) => return Err(error),
        };
        self.store
            .write(move |tx| {
                for id in &deleted {
                    messages::delete(tx, &id.to_lowercase())?;
                }
                Ok(())
            })
            .await?;
        self.refresh_messages(session_id).await
    }

    /// Asks for a reply again; the old reply goes and the new one streams in.
    pub async fn regenerate_message(
        &self,
        session_id: &str,
        message_id: &str,
        style: &str,
    ) -> Result<(), NomiError> {
        self.ensure_owner()?;
        self.api
            .regenerate_message(session_id, message_id, style)
            .await?;
        self.refresh_messages(session_id).await
    }

    // Notifications

    /// Refetches the inbox's first page.
    pub async fn refresh_notifications(&self) -> Result<(), NomiError> {
        self.load_notification_page(FIRST_NOTIFICATION_PAGE).await?;
        Ok(())
    }

    /// Loads the inbox page after the last one loaded; returns whether more pages exist.
    pub async fn load_more_notifications(&self) -> Result<bool, NomiError> {
        let (loaded, pages) = self
            .store
            .read(|connection| {
                Ok((
                    sync_state::get(connection, NOTIFICATION_PAGE_KEY)?,
                    sync_state::get(connection, NOTIFICATION_PAGES_KEY)?,
                ))
            })
            .await?;
        let next = match (loaded, pages) {
            (Some(loaded), Some(pages)) if loaded >= pages => return Ok(false),
            (Some(loaded), _) => loaded as u32 + 1,
            (None, _) => FIRST_NOTIFICATION_PAGE,
        };
        self.load_notification_page(next).await
    }

    async fn load_notification_page(&self, page: u32) -> Result<bool, NomiError> {
        self.ensure_owner()?;
        let fetched = self.api.notifications(page).await?;
        let now = Self::now();
        let (page, pages) = (fetched.page, fetched.pages);
        let items = fetched
            .items
            .into_iter()
            .map(|dto| dto.into_item(now))
            .collect::<Result<Vec<_>, _>>()?;
        self.store
            .write(move |tx| {
                notifications::upsert(tx, &items)?;
                sync_state::set(tx, NOTIFICATION_PAGE_KEY, i64::from(page))?;
                sync_state::set(tx, NOTIFICATION_PAGES_KEY, i64::from(pages))
            })
            .await?;
        Ok(page < pages)
    }

    /// Marks the notification read here at once, and on the backend through the outbox.
    pub async fn mark_notification_read(&self, notification_id: &str) -> Result<(), NomiError> {
        let id = notification_id.to_lowercase();
        let now = Self::now();
        self.store
            .write(move |tx| {
                notifications::mark_read(tx, &id, now)?;
                let operation = OutboxOperation::MarkNotificationRead {
                    notification_id: id.clone(),
                };
                queue::insert(
                    tx,
                    &format!("read:{id}"),
                    NOTIFICATIONS_LANE,
                    &operation,
                    now,
                )?;
                Ok(())
            })
            .await?;
        self.wake.notify_one();
        Ok(())
    }

    /// Marks everything read here at once, and on the backend through the outbox. Single reads
    /// still waiting to go out are folded into it.
    pub async fn mark_all_notifications_read(&self) -> Result<(), NomiError> {
        let now = Self::now();
        self.store
            .write(move |tx| {
                notifications::mark_all_read(tx, now)?;
                let waiting = queue::idle_in_lane(tx.raw(), NOTIFICATIONS_LANE)?;
                if waiting
                    .iter()
                    .any(|item| item.operation == OutboxOperation::MarkAllNotificationsRead)
                {
                    return Ok(());
                }
                let folded: Vec<i64> = waiting.iter().map(|item| item.seq).collect();
                queue::remove(tx, &folded)?;
                queue::insert(
                    tx,
                    &new_key(),
                    NOTIFICATIONS_LANE,
                    &OutboxOperation::MarkAllNotificationsRead,
                    now,
                )?;
                Ok(())
            })
            .await?;
        self.wake.notify_one();
        Ok(())
    }

    // Outbox

    /// Puts a failed entry back in line. Returns false when `key` names no failed entry.
    pub async fn retry_outbox(&self, key: String) -> Result<bool, NomiError> {
        let now = Self::now();
        let retried = self
            .store
            .write(move |tx| {
                let item = queue::retry(tx, &key, now)?;
                if let Some(OutboxOperation::SendMessage {
                    local_message_id, ..
                }) = item.as_ref().map(|item| &item.operation)
                {
                    messages::set_status(tx, local_message_id, STATUS_SENDING)?;
                }
                Ok(item.is_some())
            })
            .await?;
        if retried {
            self.wake.notify_one();
        }
        Ok(retried)
    }

    /// Drops a pending or failed entry, and the local message it showed. Returns false when `key`
    /// names neither. An attempt already on its way may still reach the backend.
    pub async fn discard_outbox(&self, key: String) -> Result<bool, NomiError> {
        self.store
            .write(move |tx| {
                let item = queue::discard(tx, &key)?;
                if let Some(OutboxOperation::SendMessage {
                    local_message_id, ..
                }) = item.as_ref().map(|item| &item.operation)
                {
                    messages::delete(tx, local_message_id)?;
                }
                Ok(item.is_some())
            })
            .await
    }

    /// Refetches the chat list and the inbox, and sends whatever the outbox holds now.
    pub async fn sync_now(&self) -> Result<(), NomiError> {
        self.retry_now().await?;
        let sessions = self.refresh_sessions().await;
        let inbox = self.refresh_notifications().await;
        sessions.and(inbox)
    }

    /// Makes every pending outbox entry due now and wakes the worker.
    async fn retry_now(&self) -> Result<(), NomiError> {
        let now = Self::now();
        self.store.write(move |tx| queue::due_now(tx, now)).await?;
        self.wake.notify_one();
        Ok(())
    }

    // Chat bindings

    pub fn bind(self: &Arc<Self>, session_id: &str) -> Result<Arc<ChatBinding>, NomiError> {
        let topic = normalize_topic(&format!("{CHAT_TOPIC_PREFIX}{session_id}"))?;
        let session_id = chat_of(&topic)
            .ok_or_else(|| NomiError::InvalidArgument {
                reason: "a chat binding needs a session id".into(),
            })?
            .to_owned();
        let first = {
            let mut bound = self.bound.lock().expect("bindings poisoned");
            let count = bound.entry(session_id.clone()).or_default();
            *count += 1;
            *count == 1
        };
        let watch = self.realtime.watch(topic.clone());
        self.route(&topic, if first { Work::Resync } else { Work::Refresh });
        Ok(Arc::new(ChatBinding {
            session_id,
            shared: self.clone(),
            watch,
            released: AtomicBool::new(false),
        }))
    }

    fn unbind(&self, session_id: &str) {
        let mut bound = self.bound.lock().expect("bindings poisoned");
        if let Some(count) = bound.get_mut(session_id) {
            *count -= 1;
            if *count == 0 {
                bound.remove(session_id);
            }
        }
    }

    fn bound_chats(&self) -> Vec<String> {
        self.bound
            .lock()
            .expect("bindings poisoned")
            .keys()
            .cloned()
            .collect()
    }
}

const CHAT_TOPIC_PREFIX: &str = "chat:";
const HTTP_NOT_FOUND: u16 = 404;

/// Outbox entries for one chat go out in order.
fn chat_lane(session_id: &str) -> String {
    format!("{CHAT_TOPIC_PREFIX}{session_id}")
}

fn older_key(session_id: &str) -> String {
    format!("{OLDER_KEY_PREFIX}{session_id}")
}

/// A fresh idempotency key: 128 random bits in hex.
fn new_key() -> String {
    format!("{:016x}{:016x}", fastrand::u64(..), fastrand::u64(..))
}
