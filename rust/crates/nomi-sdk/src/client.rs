use std::future::Future;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

use tokio::runtime::{Handle, Runtime};
use tokio::sync::watch;
use tokio::task::JoinHandle;

use crate::config::{SdkConfig, ValidatedConfig};
use crate::error::NomiError;
use crate::event::{EventBus, EventSubscription, SdkEvent, SdkState};
use crate::logging;
use crate::network::auth::{
    CredentialStore, Profile, Refresh, SignInResult, SignedInUser, VerificationInfo,
};
use crate::network::realtime::{normalize_topic, ConnectionState, RealtimeSettings, TopicWatch};
use crate::network::Network;
use crate::services::account::{ConnectionsOverview, Memory, Project, UsageBrief};
use crate::services::home::{CrewMember, HomeSummary, Preferences, PreferencesUpdate};
use crate::services::money::{MoneySummary, NewTransaction};
use crate::services::reminders::{NewReminder, Reminder, ReminderAction, RemindersOverview};
use crate::services::Services;
use crate::storage::{AccountStore, Accounts};
use crate::sync::{ChatBinding, Shared, SyncEngine, SyncSettings};

/// Worker threads for the SDK's runtime: background work is I/O-bound, and phones are small.
const WORKER_THREADS: usize = 2;
const THREAD_NAME: &str = "nomi-sdk";
/// How long [`NomiSdk::shutdown`] waits for background tasks before abandoning them.
const SHUTDOWN_GRACE: Duration = Duration::from_secs(2);

/// The engine's tunables; tests shorten them.
#[derive(Debug, Clone, Default)]
pub(crate) struct EngineSettings {
    pub realtime: RealtimeSettings,
    pub sync: SyncSettings,
}

impl From<RealtimeSettings> for EngineSettings {
    fn from(realtime: RealtimeSettings) -> Self {
        Self {
            realtime,
            sync: SyncSettings::default(),
        }
    }
}

/// A snapshot of the SDK's health, for support screens and bug reports.
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct Diagnostics {
    pub sdk_version: String,
    pub state: SdkState,
    pub base_url: String,
    pub data_dir: String,
    /// The open account, if any.
    pub account_id: Option<String>,
    /// Wall-clock milliseconds since the epoch when the SDK last started; `None` until then.
    pub started_at_ms: Option<u64>,
}

/// The SDK's entry point: one per signed-in app process, owned by the app, not by a screen.
///
/// `new` initializes (validates the config, prepares the data directory, builds the runtime),
/// `start` launches background work, `shutdown` stops it for good. All three are safe to call
/// from any thread; `start` and `shutdown` are idempotent.
#[derive(uniffi::Object)]
pub struct NomiSdk {
    config: ValidatedConfig,
    bus: Arc<EventBus>,
    accounts: Arc<Accounts>,
    network: Network,
    services: Services,
    sync_settings: SyncSettings,
    /// The open account's sync engine; runs exactly while that account is open.
    sync: Mutex<Option<SyncEngine>>,
    /// The chat on screen, if any: replies there don't mark it unread.
    foreground: Arc<Mutex<Option<String>>>,
    inner: Mutex<Inner>,
}

struct Inner {
    state: SdkState,
    runtime: Option<Runtime>,
    stop: Option<watch::Sender<bool>>,
    tasks: Vec<JoinHandle<()>>,
    started_at_ms: Option<u64>,
}

impl Drop for Inner {
    fn drop(&mut self) {
        // A runtime dropped normally blocks, which panics inside another async context.
        if let Some(runtime) = self.runtime.take() {
            runtime.shutdown_background();
        }
    }
}

#[uniffi::export]
impl NomiSdk {
    /// `credentials` is the platform's secure store for the session's tokens.
    #[uniffi::constructor]
    pub fn new(
        config: SdkConfig,
        credentials: Box<dyn CredentialStore>,
    ) -> Result<Arc<Self>, NomiError> {
        Self::with_settings(config, credentials, EngineSettings::default())
    }

    /// Launches background work. A no-op while already running.
    pub async fn start(&self) -> Result<(), NomiError> {
        {
            let mut inner = self.lock();
            match inner.state {
                SdkState::Running => return Ok(()),
                SdkState::ShutDown => return Err(NomiError::ShutDown),
                SdkState::Initialized => {}
            }
            let runtime = inner.runtime.as_ref().ok_or(NomiError::ShutDown)?;
            let (stop, stopped) = watch::channel(false);
            let forwarder = runtime.spawn(logging::forward(
                logging::subscribe(),
                self.config.log_level,
                self.bus.clone(),
                stopped,
            ));
            inner.tasks.push(forwarder);
            inner.stop = Some(stop);
            inner.state = SdkState::Running;
            inner.started_at_ms = Some(logging::now_ms());
        }
        self.bus.emit(SdkEvent::StateChanged {
            state: SdkState::Running,
        });
        tracing::info!("started");
        self.network.realtime.enable();
        Ok(())
    }

    // Session

    /// Signs in with email and password. The backend may answer with an emailed-code challenge
    /// instead; finish it with `verify_email_code`.
    pub async fn sign_in(
        &self,
        email: String,
        password: String,
    ) -> Result<SignInResult, NomiError> {
        let auth = self.network.auth.clone();
        self.on_runtime(async move { auth.sign_in(&email, &password).await })
            .await
    }

    pub async fn verify_email_code(
        &self,
        challenge_id: String,
        code: String,
    ) -> Result<SignedInUser, NomiError> {
        let auth = self.network.auth.clone();
        self.on_runtime(async move { auth.verify_email_code(&challenge_id, &code).await })
            .await
    }

    /// Emails a fresh code; the previous one stops working.
    pub async fn resend_email_code(
        &self,
        challenge_id: String,
    ) -> Result<VerificationInfo, NomiError> {
        let auth = self.network.auth.clone();
        self.on_runtime(async move { auth.resend_email_code(&challenge_id).await })
            .await
    }

    /// Exchanges the refresh token for a new access token. `false` when there is no session or
    /// the backend refused it (the session has then ended); an error says nothing about the session.
    pub async fn refresh_session(&self) -> Result<bool, NomiError> {
        let auth = self.network.auth.clone();
        let outcome = self
            .on_runtime(async move { auth.refresh(None).await })
            .await?;
        Ok(outcome == Refresh::Refreshed)
    }

    /// Signs out: forgets the session, closes the realtime socket and the open account (its data
    /// stays until `remove_account`), and revokes the session on the backend, best effort.
    pub async fn sign_out(&self) -> Result<(), NomiError> {
        let auth = self.network.auth.clone();
        self.on_runtime(async move {
            auth.sign_out().await;
            Ok(())
        })
        .await?;
        self.close_account().await
    }

    /// Who is signed in, if anyone. Known at construction from the stored session.
    pub fn signed_in_user(&self) -> Option<SignedInUser> {
        self.network.auth.user()
    }

    pub async fn fetch_profile(&self) -> Result<Profile, NomiError> {
        let auth = self.network.auth.clone();
        self.on_runtime(async move { auth.profile().await }).await
    }

    // Realtime

    pub fn connection_state(&self) -> ConnectionState {
        self.network.realtime.state()
    }

    /// Skips any backoff wait and reconnects now (e.g. when the device comes back online).
    pub fn reconnect(&self) {
        self.network.realtime.reconnect();
    }

    /// Watches a realtime topic (`chat:<id>`, `project:<id>`, or `user`): its events arrive as
    /// `SdkEvent::Realtime` while any watch for it is held.
    pub fn watch_topic(&self, topic: String) -> Result<Arc<TopicWatch>, NomiError> {
        let topic = normalize_topic(&topic)?;
        Ok(Arc::new(self.network.realtime.watch(topic)))
    }

    // Sync (the open account)

    /// Replaces the chat list with the server's.
    pub async fn refresh_sessions(&self) -> Result<(), NomiError> {
        let sync = self.sync()?;
        self.on_runtime(async move { sync.refresh_sessions().await })
            .await
    }

    /// Creates an empty chat and returns its id. The server lists it once a message is sent.
    pub async fn create_session(&self) -> Result<String, NomiError> {
        let sync = self.sync()?;
        self.on_runtime(async move { sync.create_session().await })
            .await
    }

    /// Keeps a chat in sync while the binding is held (its screen is open): watches its topic,
    /// applies its events, and loads its newest page.
    pub fn bind_chat(&self, session_id: String) -> Result<Arc<ChatBinding>, NomiError> {
        self.sync()?.bind(&session_id)
    }

    /// Loads a chat's newest page.
    pub async fn refresh_messages(&self, session_id: String) -> Result<(), NomiError> {
        let sync = self.sync()?;
        let session_id = session_id.to_lowercase();
        self.on_runtime(async move { sync.refresh_messages(&session_id).await })
            .await
    }

    /// Loads the page before the oldest stored message; returns whether older ones still exist.
    pub async fn load_older_messages(&self, session_id: String) -> Result<bool, NomiError> {
        let sync = self.sync()?;
        let session_id = session_id.to_lowercase();
        self.on_runtime(async move { sync.load_older_messages(&session_id).await })
            .await
    }

    /// Whether older messages may exist on the server; true until a load says otherwise.
    pub async fn has_older_messages(&self, session_id: String) -> Result<bool, NomiError> {
        let sync = self.sync()?;
        let session_id = session_id.to_lowercase();
        self.on_runtime(async move { sync.has_older_messages(&session_id).await })
            .await
    }

    /// Queues the person's message, shown at once with status `sending`; returns its local id.
    /// It is delivered in order with the chat's other queued messages, retried until the backend
    /// has it, and replaced by the server's copy then.
    pub async fn send_message(
        &self,
        session_id: String,
        text: String,
    ) -> Result<String, NomiError> {
        let sync = self.sync()?;
        let session_id = session_id.to_lowercase();
        self.on_runtime(async move { sync.send_message(&session_id, text).await })
            .await
    }

    /// Approves or denies a pending approval card. `Http { status: 409 }`: no longer pending.
    pub async fn resolve_approval(
        &self,
        session_id: String,
        message_id: String,
        approve: bool,
        remember: bool,
    ) -> Result<(), NomiError> {
        let sync = self.sync()?;
        self.on_runtime(async move {
            sync.resolve_approval(&session_id, &message_id, approve, remember)
                .await
        })
        .await
    }

    /// Refetches the inbox's first page.
    pub async fn refresh_notifications(&self) -> Result<(), NomiError> {
        let sync = self.sync()?;
        self.on_runtime(async move { sync.refresh_notifications().await })
            .await
    }

    /// Loads the next inbox page; returns whether more pages exist.
    pub async fn load_more_notifications(&self) -> Result<bool, NomiError> {
        let sync = self.sync()?;
        self.on_runtime(async move { sync.load_more_notifications().await })
            .await
    }

    /// Marks a notification read now, and on the backend through the outbox.
    pub async fn mark_notification_read(&self, notification_id: String) -> Result<(), NomiError> {
        let sync = self.sync()?;
        self.on_runtime(async move { sync.mark_notification_read(&notification_id).await })
            .await
    }

    /// Marks every notification read now, and on the backend through the outbox.
    pub async fn mark_all_notifications_read(&self) -> Result<(), NomiError> {
        let sync = self.sync()?;
        self.on_runtime(async move { sync.mark_all_notifications_read().await })
            .await
    }

    /// Puts a failed outbox entry back in line; false when `key` names no failed entry.
    pub async fn retry_outbox(&self, key: String) -> Result<bool, NomiError> {
        let sync = self.sync()?;
        self.on_runtime(async move { sync.retry_outbox(key).await })
            .await
    }

    /// Drops a pending or failed outbox entry and the message it showed; false when `key` names
    /// neither.
    pub async fn discard_outbox(&self, key: String) -> Result<bool, NomiError> {
        let sync = self.sync()?;
        self.on_runtime(async move { sync.discard_outbox(key).await })
            .await
    }

    /// Refetches the chat list and the inbox, and sends what the outbox holds now (e.g. when the
    /// app comes to the foreground).
    pub async fn sync_now(&self) -> Result<(), NomiError> {
        let sync = self.sync()?;
        self.on_runtime(async move { sync.sync_now().await }).await
    }

    /// The chat on screen (`None` when none is): a reply there doesn't mark it unread.
    pub fn set_foreground_chat(&self, session_id: Option<String>) {
        *self.foreground.lock().expect("foreground poisoned") =
            session_id.map(|id| id.to_lowercase());
    }

    // Account pages (online calls; see services/)

    /// The person's crew and what each member is doing.
    pub async fn crew(&self) -> Result<Vec<CrewMember>, NomiError> {
        let services = self.services.clone();
        self.on_runtime(async move { services.crew().await }).await
    }

    /// Appearance and layout preferences.
    pub async fn preferences(&self) -> Result<Preferences, NomiError> {
        let services = self.services.clone();
        self.on_runtime(async move { services.preferences().await })
            .await
    }

    /// Changes the preferences set in `update`; returns them all.
    pub async fn update_preferences(
        &self,
        update: PreferencesUpdate,
    ) -> Result<Preferences, NomiError> {
        let services = self.services.clone();
        self.on_runtime(async move { services.update_preferences(update).await })
            .await
    }

    /// The home page's status: what happened while away, today, plans.
    pub async fn home_summary(&self) -> Result<HomeSummary, NomiError> {
        let services = self.services.clone();
        self.on_runtime(async move { services.home_summary().await })
            .await
    }

    /// The money month starting in `month` (`YYYY-MM`), or the current one.
    pub async fn money_summary(&self, month: Option<String>) -> Result<MoneySummary, NomiError> {
        let services = self.services.clone();
        self.on_runtime(async move { services.money_summary(month).await })
            .await
    }

    /// Adds a transaction from the Money page.
    pub async fn add_transaction(&self, transaction: NewTransaction) -> Result<(), NomiError> {
        let services = self.services.clone();
        self.on_runtime(async move { services.add_transaction(transaction).await })
            .await
    }

    /// Sets a category's monthly budget.
    pub async fn set_budget(&self, category: String, limit_cents: i64) -> Result<(), NomiError> {
        let services = self.services.clone();
        self.on_runtime(async move { services.set_budget(category, limit_cents).await })
            .await
    }

    /// Upcoming and past reminders, and agent work scheduled for later.
    pub async fn reminders(&self) -> Result<RemindersOverview, NomiError> {
        let services = self.services.clone();
        self.on_runtime(async move { services.reminders().await })
            .await
    }

    /// Creates a reminder.
    pub async fn create_reminder(&self, reminder: NewReminder) -> Result<Reminder, NomiError> {
        let services = self.services.clone();
        self.on_runtime(async move { services.create_reminder(reminder).await })
            .await
    }

    /// Marks a reminder done, cancels it, or snoozes it.
    pub async fn act_on_reminder(
        &self,
        id: String,
        action: ReminderAction,
    ) -> Result<(), NomiError> {
        let services = self.services.clone();
        self.on_runtime(async move { services.act_on_reminder(id, action).await })
            .await
    }

    /// Cancels agent work scheduled for later.
    pub async fn cancel_scheduled_task(&self, id: String) -> Result<(), NomiError> {
        let services = self.services.clone();
        self.on_runtime(async move { services.cancel_scheduled_task(id).await })
            .await
    }

    /// What Nomi remembers about the person.
    pub async fn memories(&self) -> Result<Vec<Memory>, NomiError> {
        let services = self.services.clone();
        self.on_runtime(async move { services.memories().await })
            .await
    }

    /// Rewrites a memory.
    pub async fn edit_memory(
        &self,
        id: String,
        content: String,
        kind: Option<String>,
    ) -> Result<(), NomiError> {
        let services = self.services.clone();
        self.on_runtime(async move { services.edit_memory(id, content, kind).await })
            .await
    }

    /// Confirms a memory is still true.
    pub async fn confirm_memory(&self, id: String) -> Result<(), NomiError> {
        let services = self.services.clone();
        self.on_runtime(async move { services.confirm_memory(id).await })
            .await
    }

    /// Forgets a memory.
    pub async fn delete_memory(&self, id: String) -> Result<(), NomiError> {
        let services = self.services.clone();
        self.on_runtime(async move { services.delete_memory(id).await })
            .await
    }

    /// The person's connected services.
    pub async fn connections(&self) -> Result<ConnectionsOverview, NomiError> {
        let services = self.services.clone();
        self.on_runtime(async move { services.connections().await })
            .await
    }

    /// This month's allowance and how much is used.
    pub async fn usage_brief(&self) -> Result<UsageBrief, NomiError> {
        let services = self.services.clone();
        self.on_runtime(async move { services.usage_brief().await })
            .await
    }

    /// A small preview of an attached image, as the encoded image.
    pub async fn attachment_preview(&self, id: String) -> Result<Vec<u8>, NomiError> {
        let services = self.services.clone();
        self.on_runtime(async move { services.attachment_preview(&id).await })
            .await
    }

    /// An attached file's content as uploaded.
    pub async fn attachment_content(&self, id: String) -> Result<Vec<u8>, NomiError> {
        let services = self.services.clone();
        self.on_runtime(async move { services.attachment_content(&id).await })
            .await
    }

    /// Projects in `org_id`, or in the organization picked for projects.
    pub async fn projects(&self, org_id: Option<String>) -> Result<Vec<Project>, NomiError> {
        let services = self.services.clone();
        self.on_runtime(async move { services.projects(org_id).await })
            .await
    }

    /// The local store for `account_id`, opening and migrating its database if needed. Opening a
    /// different account closes the current one first; its store handles stop working.
    ///
    /// `account_id` names a directory, so it is limited to `A-Z a-z 0-9 - _ .` (user ids are UUIDs).
    pub async fn open_account(&self, account_id: String) -> Result<Arc<AccountStore>, NomiError> {
        let runtime = self.runtime()?;
        let (accounts, handle, id) = (self.accounts.clone(), runtime.clone(), account_id.clone());
        let (store, switched) = runtime
            .spawn_blocking(move || {
                let switched = accounts
                    .current()
                    .is_none_or(|store| store.account_id() != id);
                Ok::<_, NomiError>((accounts.open(&id, handle)?, switched))
            })
            .await
            .map_err(|_| NomiError::ShutDown)??;
        if self.state() == SdkState::ShutDown {
            self.accounts.close_current();
            return Err(NomiError::ShutDown);
        }
        self.ensure_sync(&store, runtime);
        if switched {
            tracing::info!(account_id, "opened account");
            self.bus.emit(SdkEvent::AccountChanged {
                account_id: Some(account_id),
            });
        }
        Ok(store)
    }

    /// The open account's store, if one is open.
    pub fn current_account(&self) -> Option<Arc<AccountStore>> {
        self.accounts.current()
    }

    /// Closes the open account, keeping its data on disk. A no-op when none is open.
    pub async fn close_account(&self) -> Result<(), NomiError> {
        let accounts = self.accounts.clone();
        let closed = self
            .runtime()?
            .spawn_blocking(move || accounts.close_current())
            .await
            .map_err(|_| NomiError::ShutDown)?;
        self.announce_closed(closed);
        Ok(())
    }

    /// Deletes everything stored for `account_id` (sign-out), closing it first if it is open.
    pub async fn remove_account(&self, account_id: String) -> Result<(), NomiError> {
        let accounts = self.accounts.clone();
        let closed = self
            .runtime()?
            .spawn_blocking(move || accounts.remove(&account_id))
            .await
            .map_err(|_| NomiError::ShutDown)??;
        self.announce_closed(closed);
        Ok(())
    }

    /// Deletes every account's local data.
    pub async fn reset_storage(&self) -> Result<(), NomiError> {
        let accounts = self.accounts.clone();
        let closed = self
            .runtime()?
            .spawn_blocking(move || accounts.remove_all())
            .await
            .map_err(|_| NomiError::ShutDown)??;
        self.announce_closed(closed);
        Ok(())
    }

    /// Stops background work and ends every event stream. A no-op once shut down.
    pub async fn shutdown(&self) {
        let (runtime, stop, tasks) = {
            let mut inner = self.lock();
            if inner.state == SdkState::ShutDown {
                return;
            }
            inner.state = SdkState::ShutDown;
            (
                inner.runtime.take(),
                inner.stop.take(),
                std::mem::take(&mut inner.tasks),
            )
        };
        tracing::info!("shutting down");
        self.network.realtime.disable();
        if let Some(stop) = stop {
            stop.send_replace(true);
        }
        if let Some(runtime) = runtime {
            let realtime = self.network.realtime.clone();
            let drained = runtime.spawn(async move {
                let joined = async {
                    realtime.wait_idle().await;
                    for task in tasks {
                        let _ = task.await;
                    }
                };
                if tokio::time::timeout(SHUTDOWN_GRACE, joined).await.is_err() {
                    tracing::warn!("background tasks outlived the shutdown grace period");
                }
            });
            let _ = drained.await;
            runtime.shutdown_background();
        }
        self.announce_closed(self.accounts.close_current());
        self.bus.emit(SdkEvent::StateChanged {
            state: SdkState::ShutDown,
        });
        self.bus.close();
    }

    pub fn state(&self) -> SdkState {
        self.lock().state
    }

    /// A new stream of the events sent from now on.
    pub fn subscribe(&self) -> Arc<EventSubscription> {
        self.bus.subscribe()
    }

    pub fn diagnostics(&self) -> Diagnostics {
        let inner = self.lock();
        Diagnostics {
            sdk_version: crate::sdk_version(),
            state: inner.state,
            base_url: self.config.base_url.to_string(),
            data_dir: self.config.data_dir.display().to_string(),
            account_id: self.accounts.current().map(|store| store.account_id()),
            started_at_ms: inner.started_at_ms,
        }
    }
}

impl NomiSdk {
    pub(crate) fn with_settings(
        config: SdkConfig,
        credentials: Box<dyn CredentialStore>,
        settings: impl Into<EngineSettings>,
    ) -> Result<Arc<Self>, NomiError> {
        let settings = settings.into();
        let config = config.validate()?;
        std::fs::create_dir_all(&config.data_dir).map_err(|error| NomiError::Storage {
            reason: format!(
                "can't create data_dir `{}`: {error}",
                config.data_dir.display()
            ),
        })?;
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(WORKER_THREADS)
            .thread_name(THREAD_NAME)
            .enable_all()
            .build()
            .map_err(|error| NomiError::Internal {
                reason: format!("can't build the async runtime: {error}"),
            })?;
        logging::init();

        let bus = Arc::new(EventBus::new(config.event_buffer_capacity));
        let network = Network::new(
            runtime.handle(),
            config.base_url.clone(),
            credentials,
            bus.clone(),
            settings.realtime,
        )?;
        let sdk = Arc::new(Self {
            accounts: Arc::new(Accounts::new(&config.data_dir)),
            bus,
            services: Services::new(network.auth.clone()),
            network,
            config,
            sync_settings: settings.sync,
            sync: Mutex::new(None),
            foreground: Arc::new(Mutex::new(None)),
            inner: Mutex::new(Inner {
                state: SdkState::Initialized,
                runtime: Some(runtime),
                stop: None,
                tasks: Vec::new(),
                started_at_ms: None,
            }),
        });
        tracing::info!(base_url = %sdk.config.base_url, "initialized");
        Ok(sdk)
    }

    /// Puts `event` on the SDK's event stream, as the engine itself would.
    #[cfg(test)]
    pub(crate) fn emit(&self, event: SdkEvent) {
        self.bus.emit(event);
    }

    /// The SDK's runtime, for blocking work such as SQLite; `ShutDown` once it is gone.
    fn runtime(&self) -> Result<Handle, NomiError> {
        let inner = self.lock();
        match (&inner.state, &inner.runtime) {
            (SdkState::ShutDown, _) | (_, None) => Err(NomiError::ShutDown),
            (_, Some(runtime)) => Ok(runtime.handle().clone()),
        }
    }

    /// Runs `work` on the SDK's runtime (network futures need it). Dropping the returned future,
    /// as a cancelled Kotlin coroutine does, aborts the work.
    async fn on_runtime<T: Send + 'static>(
        &self,
        work: impl Future<Output = Result<T, NomiError>> + Send + 'static,
    ) -> Result<T, NomiError> {
        let task = AbortOnDrop(self.runtime()?.spawn(work));
        task.join().await
    }

    /// Starts sync for `store` unless it already runs for it; any other account's engine stops.
    fn ensure_sync(&self, store: &Arc<AccountStore>, runtime: Handle) {
        let mut sync = self.sync.lock().expect("sync poisoned");
        if sync
            .as_ref()
            .is_some_and(|engine| engine.account_id() == store.account_id())
        {
            return;
        }
        sync.take();
        *sync = Some(SyncEngine::start(
            store.clone(),
            self.network.auth.clone(),
            self.network.realtime.clone(),
            self.bus.clone(),
            runtime,
            self.sync_settings.clone(),
            self.foreground.clone(),
        ));
    }

    /// The open account's sync engine, or `AccountClosed`.
    fn sync(&self) -> Result<Arc<Shared>, NomiError> {
        self.sync
            .lock()
            .expect("sync poisoned")
            .as_ref()
            .map(SyncEngine::shared)
            .ok_or(NomiError::AccountClosed)
    }

    fn announce_closed(&self, closed: bool) {
        // Sync runs exactly while its account is open.
        let open = self.accounts.current().map(|store| store.account_id());
        let mut sync = self.sync.lock().expect("sync poisoned");
        if sync.as_ref().map(SyncEngine::account_id) != open {
            sync.take();
        }
        drop(sync);
        if closed {
            tracing::info!("closed account");
            self.bus.emit(SdkEvent::AccountChanged { account_id: None });
        }
    }

    fn lock(&self) -> MutexGuard<'_, Inner> {
        self.inner.lock().expect("SDK state poisoned")
    }
}

/// A spawned task that is aborted if whoever awaits it goes away.
struct AbortOnDrop<T>(JoinHandle<Result<T, NomiError>>);

impl<T> AbortOnDrop<T> {
    async fn join(mut self) -> Result<T, NomiError> {
        (&mut self.0).await.map_err(|_| NomiError::ShutDown)?
    }
}

impl<T> Drop for AbortOnDrop<T> {
    fn drop(&mut self) {
        self.0.abort();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::LogLevel;
    use crate::default_sdk_config;
    use crate::test_support::MemoryCredentials;

    const BASE_URL: &str = "https://api-nomi.trian.space";
    const EVENT_TIMEOUT: Duration = Duration::from_secs(5);

    fn sdk(dir: &tempfile::TempDir) -> Arc<NomiSdk> {
        NomiSdk::new(
            default_sdk_config(BASE_URL.to_owned(), dir.path().display().to_string()),
            MemoryCredentials::default().boxed(),
        )
        .unwrap()
    }

    /// The next event that isn't a log line (other tests share the process-wide log channel).
    async fn next_lifecycle_event(subscription: &EventSubscription) -> Option<SdkEvent> {
        tokio::time::timeout(EVENT_TIMEOUT, async {
            loop {
                match subscription.next_event().await {
                    Some(SdkEvent::Log { .. }) => continue,
                    other => return other,
                }
            }
        })
        .await
        .expect("timed out waiting for an event")
    }

    #[tokio::test]
    async fn walks_through_the_lifecycle() {
        let dir = tempfile::tempdir().unwrap();
        let sdk = sdk(&dir);
        assert_eq!(sdk.state(), SdkState::Initialized);
        assert_eq!(sdk.diagnostics().started_at_ms, None);

        sdk.start().await.unwrap();
        assert_eq!(sdk.state(), SdkState::Running);
        assert!(sdk.diagnostics().started_at_ms.is_some());

        sdk.shutdown().await;
        assert_eq!(sdk.state(), SdkState::ShutDown);
    }

    #[tokio::test]
    async fn start_and_shutdown_are_idempotent() {
        let dir = tempfile::tempdir().unwrap();
        let sdk = sdk(&dir);
        sdk.start().await.unwrap();
        sdk.start().await.unwrap();
        sdk.shutdown().await;
        sdk.shutdown().await;
        assert_eq!(sdk.state(), SdkState::ShutDown);
    }

    #[tokio::test]
    async fn start_after_shutdown_fails() {
        let dir = tempfile::tempdir().unwrap();
        let sdk = sdk(&dir);
        sdk.shutdown().await;
        assert_eq!(sdk.start().await, Err(NomiError::ShutDown));
    }

    #[tokio::test]
    async fn subscribers_see_state_changes_then_the_stream_ends() {
        let dir = tempfile::tempdir().unwrap();
        let sdk = sdk(&dir);
        let subscription = sdk.subscribe();
        sdk.start().await.unwrap();
        sdk.shutdown().await;

        assert_eq!(
            next_lifecycle_event(&subscription).await,
            Some(SdkEvent::StateChanged {
                state: SdkState::Running
            })
        );
        assert_eq!(
            next_lifecycle_event(&subscription).await,
            Some(SdkEvent::StateChanged {
                state: SdkState::ShutDown
            })
        );
        assert_eq!(next_lifecycle_event(&subscription).await, None);
        assert_eq!(sdk.subscribe().next_event().await, None);
    }

    #[tokio::test]
    async fn forwards_sdk_logs_at_the_configured_level() {
        let dir = tempfile::tempdir().unwrap();
        let mut config = default_sdk_config(BASE_URL.to_owned(), dir.path().display().to_string());
        config.log_level = LogLevel::Info;
        let sdk = NomiSdk::new(config, MemoryCredentials::default().boxed()).unwrap();
        let subscription = sdk.subscribe();
        sdk.start().await.unwrap();

        let marker = format!("marker-{}", logging::now_ms());
        tracing::debug!(target: "nomi_sdk::test", "{marker} hidden");
        tracing::info!(target: "nomi_sdk::test", "{marker} shown");

        let record = tokio::time::timeout(EVENT_TIMEOUT, async {
            loop {
                if let Some(SdkEvent::Log { record }) = subscription.next_event().await {
                    if record.message.starts_with(&marker) {
                        return record;
                    }
                }
            }
        })
        .await
        .expect("the info line never arrived");
        assert_eq!(record.level, LogLevel::Info);
        assert_eq!(record.message, format!("{marker} shown"));
        sdk.shutdown().await;
    }

    #[tokio::test]
    async fn creates_the_data_dir() {
        let dir = tempfile::tempdir().unwrap();
        let nested = dir.path().join("accounts").join("default");
        NomiSdk::new(
            default_sdk_config(BASE_URL.to_owned(), nested.display().to_string()),
            MemoryCredentials::default().boxed(),
        )
        .unwrap();
        assert!(nested.is_dir());
    }

    #[test]
    fn reports_a_data_dir_it_cannot_create() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("occupied");
        std::fs::write(&file, b"").unwrap();
        let config = default_sdk_config(
            BASE_URL.to_owned(),
            file.join("nested").display().to_string(),
        );
        assert!(matches!(
            NomiSdk::new(config, MemoryCredentials::default().boxed()),
            Err(NomiError::Storage { .. })
        ));
    }

    #[tokio::test]
    async fn dropping_a_running_sdk_inside_a_runtime_does_not_panic() {
        let dir = tempfile::tempdir().unwrap();
        let sdk = sdk(&dir);
        sdk.start().await.unwrap();
        drop(sdk);
    }

    #[test]
    fn can_be_driven_without_a_host_runtime() {
        let dir = tempfile::tempdir().unwrap();
        let sdk = sdk(&dir);
        let host = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        host.block_on(sdk.start()).unwrap();
        host.block_on(sdk.shutdown());
        assert_eq!(sdk.state(), SdkState::ShutDown);
    }
}
