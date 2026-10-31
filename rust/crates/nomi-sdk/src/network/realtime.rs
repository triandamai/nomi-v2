//! The app's single realtime socket (`/api/realtime/ws`), ported from the Kotlin `RealtimeHub`.
//!
//! An actor task owns every piece of state. Watched topics are ref-counted into server
//! subscriptions that survive reconnects; after each `ready` every live topic gets
//! `SdkEvent::TopicResynced` so its listeners refetch what they may have missed. A dead socket is
//! retried with jittered exponential backoff. A 401 upgrade or a 4001 close refreshes the token
//! once; a backend that still refuses ends the session.

use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use futures_util::{SinkExt as _, StreamExt as _};
use rustls::ClientConfig;
use serde::{Deserialize, Serialize};
use tokio::runtime::Handle;
use tokio::sync::{mpsc, watch};
use tokio::task::JoinHandle;
use tokio_tungstenite::tungstenite::client::IntoClientRequest as _;
use tokio_tungstenite::tungstenite::http::{header, HeaderValue, StatusCode};
use tokio_tungstenite::tungstenite::{self, Message};
use tokio_tungstenite::Connector;
use url::Url;

use super::auth::{Auth, Refresh, SignOutReason};
use crate::error::NomiError;
use crate::event::{EventBus, SdkEvent};

pub(crate) const REALTIME_PATH: &str = "/api/realtime/ws";
const USER_TOPIC: &str = "user";
const CHAT_PREFIX: &str = "chat:";
const PROJECT_PREFIX: &str = "project:";
/// The backend's private close code for "the access token this socket was opened with expired".
const CLOSE_TOKEN_EXPIRED: u16 = 4001;
/// `base · 2^6` is already past any sensible cap; clamping also keeps the shift from overflowing.
const MAX_BACKOFF_EXPONENT: u32 = 6;

/// Timings, overridable in tests.
#[derive(Debug, Clone)]
pub(crate) struct RealtimeSettings {
    /// Client `ping` cadence; the gateway's `pong` also feeds the stale watchdog.
    pub ping_every: Duration,
    /// A socket with no frame for this long is considered dead and recycled.
    pub stale_after: Duration,
    /// A connection `ready` this long resets the backoff attempt counter.
    pub stable_after: Duration,
    /// How long the WebSocket handshake may take.
    pub connect_timeout: Duration,
    /// How long closing a finished socket may take.
    pub close_timeout: Duration,
    pub backoff_base: Duration,
    pub backoff_max: Duration,
}

impl Default for RealtimeSettings {
    fn default() -> Self {
        Self {
            ping_every: Duration::from_secs(25),
            stale_after: Duration::from_secs(60),
            stable_after: Duration::from_secs(10),
            connect_timeout: Duration::from_secs(15),
            close_timeout: Duration::from_secs(5),
            backoff_base: Duration::from_millis(500),
            backoff_max: Duration::from_secs(30),
        }
    }
}

/// The realtime socket's state.
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Enum)]
pub enum ConnectionState {
    /// Signed out, or the SDK isn't running.
    Idle,
    Connecting,
    Connected {
        user_id: String,
    },
    /// Waiting `retry_in_ms` before reconnect attempt number `attempt`.
    Waiting {
        retry_in_ms: u64,
        attempt: u32,
    },
}

/// Frames the app sends (`ClientFrame` in the backend's `nomi-realtime`, tag `op`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "op", rename_all = "snake_case")]
enum ClientFrame {
    Subscribe { topic: String },
    Unsubscribe { topic: String },
    Ping,
}

/// Frames the gateway sends (`ServerFrame`, tag `type`).
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ServerFrame {
    Ready {
        user_id: String,
    },
    Subscribed {
        topic: String,
    },
    Unsubscribed {
        topic: String,
    },
    Denied {
        topic: String,
    },
    /// Access couldn't be checked right now (not a refusal): ask again later.
    Unavailable {
        topic: String,
    },
    Event {
        topic: String,
        payload: serde_json::Value,
    },
    Error {
        message: String,
    },
    Pong,
}

/// `user`, `chat:<id>` or `project:<id>`, ids lowercased as the gateway spells them.
pub(crate) fn normalize_topic(raw: &str) -> Result<String, NomiError> {
    let raw = raw.trim();
    let normalized = if raw == USER_TOPIC {
        Some(USER_TOPIC.to_owned())
    } else {
        [CHAT_PREFIX, PROJECT_PREFIX].iter().find_map(|prefix| {
            raw.strip_prefix(prefix)
                .filter(|id| !id.is_empty())
                .map(|id| format!("{prefix}{}", id.to_lowercase()))
        })
    };
    normalized.ok_or_else(|| NomiError::InvalidArgument {
        reason: format!("a topic is `user`, `chat:<id>` or `project:<id>`, got `{raw}`"),
    })
}

/// Full-jitter exponential backoff: uniform in `0..=min(base · 2^attempt, max)`.
pub(crate) fn backoff_delay(attempt: u32, base: Duration, max: Duration) -> Duration {
    let ceiling = base
        .saturating_mul(1 << attempt.min(MAX_BACKOFF_EXPONENT))
        .min(max);
    Duration::from_millis(fastrand::u64(0..=ceiling.as_millis() as u64))
}

/// A handle on the hub actor. Cheap to clone.
#[derive(Clone)]
pub(crate) struct Realtime {
    commands: mpsc::UnboundedSender<Msg>,
    state: watch::Receiver<ConnectionState>,
}

impl Realtime {
    pub fn spawn(
        runtime: &Handle,
        url: Url,
        tls: Arc<ClientConfig>,
        auth: Arc<Auth>,
        bus: Arc<EventBus>,
        settings: RealtimeSettings,
    ) -> Self {
        let (commands, inbox) = mpsc::unbounded_channel();
        let (state_tx, state) = watch::channel(ConnectionState::Idle);
        let hub = Hub {
            tx: commands.clone(),
            runtime: runtime.clone(),
            url,
            tls,
            session: auth.watch_session(),
            auth,
            bus,
            settings,
            state: state_tx,
            enabled: false,
            running: false,
            generation: 0,
            ready: false,
            failures: 0,
            refreshed_since_ready: false,
            outgoing: None,
            token_in_use: None,
            timer: None,
            registrations: HashMap::new(),
            denied: HashSet::new(),
            unavailable: HashMap::new(),
            subscribe_retries: HashMap::new(),
        };
        runtime.spawn(hub.run(inbox));
        Self { commands, state }
    }

    /// Lets the socket run whenever someone is signed in (SDK `start`).
    pub fn enable(&self) {
        self.post(Msg::Enable);
    }

    /// Closes the socket and keeps it closed (SDK `shutdown`).
    pub fn disable(&self) {
        self.post(Msg::Disable);
    }

    /// Skips any backoff wait and connects now.
    pub fn reconnect(&self) {
        self.post(Msg::Reconnect);
    }

    pub fn state(&self) -> ConnectionState {
        self.state.borrow().clone()
    }

    /// Resolves once the socket is closed (`Idle`).
    pub async fn wait_idle(&self) {
        let mut state = self.state.clone();
        let _ = state
            .wait_for(|state| *state == ConnectionState::Idle)
            .await;
    }

    pub fn watch(&self, topic: String) -> TopicWatch {
        self.post(Msg::Watch(topic.clone()));
        TopicWatch {
            topic,
            realtime: self.clone(),
            released: AtomicBool::new(false),
        }
    }

    fn post(&self, msg: Msg) {
        // The actor only goes away with the runtime, when nothing is listening any more.
        let _ = self.commands.send(msg);
    }
}

/// Interest in one realtime topic: its events arrive on the SDK event stream while any watch for
/// it is held. Released explicitly, or when dropped.
#[derive(uniffi::Object)]
pub struct TopicWatch {
    topic: String,
    realtime: Realtime,
    released: AtomicBool,
}

#[uniffi::export]
impl TopicWatch {
    pub fn topic(&self) -> String {
        self.topic.clone()
    }

    /// Stops watching. Idempotent.
    pub fn release(&self) {
        if !self.released.swap(true, Ordering::SeqCst) {
            self.realtime.post(Msg::Release(self.topic.clone()));
        }
    }
}

impl Drop for TopicWatch {
    fn drop(&mut self) {
        self.release();
    }
}

#[derive(Debug)]
enum Msg {
    Enable,
    Disable,
    Reconnect,
    Watch(String),
    Release(String),
    Frame(u64, ServerFrame),
    Closed(u64),
    Failed(u64, String),
    Unauthorized(u64),
    Refreshed(u64, Result<Refresh, NomiError>),
    Retry(u64),
    Stable(u64),
    RetrySubscribe(u64, String),
}

struct Hub {
    tx: mpsc::UnboundedSender<Msg>,
    runtime: Handle,
    url: Url,
    tls: Arc<ClientConfig>,
    auth: Arc<Auth>,
    session: watch::Receiver<Option<super::auth::SignedInUser>>,
    bus: Arc<EventBus>,
    settings: RealtimeSettings,
    state: watch::Sender<ConnectionState>,
    /// The SDK wants the socket (it is running).
    enabled: bool,
    /// A connection lifecycle is under way: connecting, connected or waiting to retry.
    running: bool,
    /// Bumped whenever the current attempt ends; messages from older attempts are ignored.
    generation: u64,
    ready: bool,
    failures: u32,
    refreshed_since_ready: bool,
    /// Frames for the current socket. Dropping it closes that socket.
    outgoing: Option<mpsc::UnboundedSender<ClientFrame>>,
    token_in_use: Option<String>,
    timer: Option<JoinHandle<()>>,
    registrations: HashMap<String, usize>,
    denied: HashSet<String>,
    /// Topics the server couldn't authorize yet, with the retry attempts so far.
    unavailable: HashMap<String, u32>,
    subscribe_retries: HashMap<String, JoinHandle<()>>,
}

impl Hub {
    async fn run(mut self, mut inbox: mpsc::UnboundedReceiver<Msg>) {
        loop {
            tokio::select! {
                msg = inbox.recv() => match msg {
                    Some(msg) => self.handle(msg),
                    None => break,
                },
                changed = self.session.changed() => {
                    if changed.is_err() {
                        break;
                    }
                    self.reconcile();
                }
            }
        }
        self.stop();
    }

    fn handle(&mut self, msg: Msg) {
        let current = self.generation;
        match msg {
            Msg::Enable => {
                self.enabled = true;
                self.reconcile();
            }
            Msg::Disable => {
                self.enabled = false;
                self.reconcile();
            }
            Msg::Reconnect => {
                if self.running {
                    self.failures = 0;
                    self.connect();
                }
            }
            Msg::Watch(topic) => self.on_watch(topic),
            Msg::Release(topic) => self.on_release(&topic),
            Msg::Frame(generation, frame) if generation == current => self.on_frame(frame),
            Msg::Closed(generation) if generation == current => self.on_disconnected(),
            Msg::Failed(generation, reason) if generation == current => {
                tracing::info!(%reason, "realtime: connection failed");
                self.on_disconnected();
            }
            Msg::Unauthorized(generation) if generation == current => self.on_unauthorized(),
            Msg::Refreshed(generation, outcome) if generation == current => {
                self.on_refreshed(outcome)
            }
            Msg::Retry(generation) if generation == current && self.running => self.connect(),
            Msg::Stable(generation) if generation == current && self.ready => self.failures = 0,
            Msg::RetrySubscribe(generation, topic) if generation == current => {
                self.subscribe_retries.remove(&topic);
                if self.registrations.contains_key(&topic) && !self.denied.contains(&topic) {
                    self.send_subscription(&topic, true);
                }
            }
            stale => tracing::trace!(
                ?stale,
                "realtime: ignoring a message from a finished attempt"
            ),
        }
    }

    /// Runs the socket exactly while the SDK is running and someone is signed in.
    fn reconcile(&mut self) {
        let signed_in = self.session.borrow_and_update().is_some();
        let wanted = self.enabled && signed_in;
        if wanted && !self.running {
            self.running = true;
            self.connect();
        } else if !wanted && self.running {
            self.stop();
        }
    }

    fn on_watch(&mut self, topic: String) {
        let count = self.registrations.entry(topic.clone()).or_default();
        *count += 1;
        let first = *count == 1;
        let was_denied = self.denied.remove(&topic);
        if first || was_denied {
            self.send_subscription(&topic, true);
        }
    }

    fn on_release(&mut self, topic: &str) {
        let Some(count) = self.registrations.get_mut(topic) else {
            return;
        };
        *count -= 1;
        if *count > 0 {
            return;
        }
        self.registrations.remove(topic);
        self.forget_unavailable(topic);
        // A denied topic has no server-side subscription left to drop.
        if !self.denied.remove(topic) {
            self.send_subscription(topic, false);
        }
    }

    fn on_frame(&mut self, frame: ServerFrame) {
        match frame {
            ServerFrame::Ready { user_id } => self.on_ready(user_id),
            ServerFrame::Event { topic, payload } => {
                self.bus.emit(SdkEvent::Realtime {
                    event: super::events::decode(&topic, &payload),
                    payload_json: payload.to_string(),
                    topic,
                });
            }
            ServerFrame::Denied { topic } => {
                if self.registrations.contains_key(&topic) {
                    self.forget_unavailable(&topic);
                    self.denied.insert(topic.clone());
                    self.bus.emit(SdkEvent::TopicAccessLost { topic });
                }
            }
            ServerFrame::Unavailable { topic } => {
                if self.registrations.contains_key(&topic) {
                    self.schedule_subscribe_retry(topic);
                }
            }
            ServerFrame::Subscribed { topic } => self.forget_unavailable(&topic),
            ServerFrame::Error { message } => tracing::warn!(%message, "realtime: server error"),
            ServerFrame::Unsubscribed { .. } | ServerFrame::Pong => {}
        }
    }

    fn on_ready(&mut self, user_id: String) {
        if self.ready {
            return;
        }
        self.ready = true;
        self.refreshed_since_ready = false;
        self.set_state(ConnectionState::Connected { user_id });
        let (generation, stable_after) = (self.generation, self.settings.stable_after);
        self.set_timer(stable_after, Msg::Stable(generation));
        let live: Vec<String> = self
            .registrations
            .keys()
            .filter(|topic| !self.denied.contains(*topic))
            .cloned()
            .collect();
        for topic in &live {
            self.send_subscription(topic, true);
        }
        // Also on the first `ready`: anything watched before it may have missed events.
        for topic in live {
            self.bus.emit(SdkEvent::TopicResynced { topic });
        }
    }

    /// `user` is implicit on the gateway; before `ready` nothing is sent, `ready` subscribes all.
    fn send_subscription(&self, topic: &str, subscribe: bool) {
        if topic == USER_TOPIC || !self.ready {
            return;
        }
        let topic = topic.to_owned();
        let frame = if subscribe {
            ClientFrame::Subscribe { topic }
        } else {
            ClientFrame::Unsubscribe { topic }
        };
        if let Some(outgoing) = &self.outgoing {
            let _ = outgoing.send(frame);
        }
    }

    fn schedule_subscribe_retry(&mut self, topic: String) {
        let attempt = self.unavailable.entry(topic.clone()).or_default();
        let delay = backoff_delay(
            *attempt,
            self.settings.backoff_base,
            self.settings.backoff_max,
        );
        *attempt += 1;
        let (tx, generation, retry_topic) = (self.tx.clone(), self.generation, topic.clone());
        let retry = self.runtime.spawn(async move {
            tokio::time::sleep(delay).await;
            let _ = tx.send(Msg::RetrySubscribe(generation, retry_topic));
        });
        if let Some(previous) = self.subscribe_retries.insert(topic, retry) {
            previous.abort();
        }
    }

    fn forget_unavailable(&mut self, topic: &str) {
        self.unavailable.remove(topic);
        if let Some(retry) = self.subscribe_retries.remove(topic) {
            retry.abort();
        }
    }

    fn connect(&mut self) {
        self.reset_connection();
        let Some(credentials) = self.auth.credentials() else {
            self.stop();
            return;
        };
        self.set_state(ConnectionState::Connecting);
        let (outgoing, frames) = mpsc::unbounded_channel();
        self.outgoing = Some(outgoing);
        self.token_in_use = Some(credentials.access_token.clone());
        let connection = Connection {
            generation: self.generation,
            url: self.url.clone(),
            token: credentials.access_token,
            tls: self.tls.clone(),
            tx: self.tx.clone(),
            settings: self.settings.clone(),
        };
        self.runtime.spawn(connection.run(frames));
    }

    fn on_disconnected(&mut self) {
        self.reset_connection();
        if !self.running {
            return;
        }
        let delay = backoff_delay(
            self.failures,
            self.settings.backoff_base,
            self.settings.backoff_max,
        );
        self.failures += 1;
        self.set_state(ConnectionState::Waiting {
            retry_in_ms: delay.as_millis() as u64,
            attempt: self.failures,
        });
        let generation = self.generation;
        self.set_timer(delay, Msg::Retry(generation));
    }

    fn on_unauthorized(&mut self) {
        if self.refreshed_since_ready {
            tracing::warn!(
                "realtime: still unauthorized right after a token refresh; ending the session"
            );
            self.stop();
            self.auth.end_session(SignOutReason::AccessRevoked);
            return;
        }
        self.refreshed_since_ready = true;
        let stale = self.token_in_use.clone();
        self.reset_connection();
        self.set_state(ConnectionState::Connecting);
        let (auth, tx, generation) = (self.auth.clone(), self.tx.clone(), self.generation);
        self.timer = Some(self.runtime.spawn(async move {
            let outcome = auth.refresh(stale.as_deref()).await;
            let _ = tx.send(Msg::Refreshed(generation, outcome));
        }));
    }

    fn on_refreshed(&mut self, outcome: Result<Refresh, NomiError>) {
        match outcome {
            Ok(Refresh::Refreshed) => self.connect(),
            // The session ended; the session watch stops the hub too, this just doesn't wait for it.
            Ok(Refresh::Rejected | Refresh::NotSignedIn) => self.stop(),
            Err(error) => {
                tracing::info!(%error, "realtime: token refresh failed");
                self.refreshed_since_ready = false;
                self.on_disconnected();
            }
        }
    }

    fn stop(&mut self) {
        self.running = false;
        self.reset_connection();
        self.failures = 0;
        self.refreshed_since_ready = false;
        self.denied.clear();
        self.set_state(ConnectionState::Idle);
    }

    /// Ends the current attempt (dropping `outgoing` closes its socket) and invalidates its
    /// pending messages.
    fn reset_connection(&mut self) {
        self.generation += 1;
        self.outgoing = None;
        self.token_in_use = None;
        if let Some(timer) = self.timer.take() {
            timer.abort();
        }
        self.ready = false;
        for (_, retry) in self.subscribe_retries.drain() {
            retry.abort();
        }
        // The next `ready` subscribes every live topic again anyway.
        self.unavailable.clear();
    }

    fn set_timer(&mut self, delay: Duration, msg: Msg) {
        let tx = self.tx.clone();
        if let Some(previous) = self.timer.replace(self.runtime.spawn(async move {
            tokio::time::sleep(delay).await;
            let _ = tx.send(msg);
        })) {
            previous.abort();
        }
    }

    fn set_state(&self, state: ConnectionState) {
        let changed = self.state.send_if_modified(|current| {
            let changed = *current != state;
            *current = state.clone();
            changed
        });
        if changed {
            self.bus.emit(SdkEvent::ConnectionChanged { state });
        }
    }
}

/// One socket. It never touches hub state; it only reports to the hub, tagged with its generation.
struct Connection {
    generation: u64,
    url: Url,
    token: String,
    tls: Arc<ClientConfig>,
    tx: mpsc::UnboundedSender<Msg>,
    settings: RealtimeSettings,
}

enum Ending {
    /// The hub let go of this socket; nothing to report.
    Dropped,
    Closed,
    Unauthorized,
    Failed(String),
}

impl Connection {
    async fn run(self, frames: mpsc::UnboundedReceiver<ClientFrame>) {
        let ending = self.connect_and_pump(frames).await;
        let generation = self.generation;
        let msg = match ending {
            Ending::Dropped => return,
            Ending::Closed => Msg::Closed(generation),
            Ending::Unauthorized => Msg::Unauthorized(generation),
            Ending::Failed(reason) => Msg::Failed(generation, reason),
        };
        let _ = self.tx.send(msg);
    }

    async fn connect_and_pump(&self, mut frames: mpsc::UnboundedReceiver<ClientFrame>) -> Ending {
        let mut request = match self.url.as_str().into_client_request() {
            Ok(request) => request,
            Err(error) => return Ending::Failed(error.to_string()),
        };
        match HeaderValue::from_str(&format!("Bearer {}", self.token)) {
            Ok(value) => request.headers_mut().insert(header::AUTHORIZATION, value),
            Err(_) => return Ending::Failed("the access token isn't a valid header value".into()),
        };
        let connector = Connector::Rustls(self.tls.clone());
        let handshake =
            tokio_tungstenite::connect_async_tls_with_config(request, None, false, Some(connector));
        // Nothing is sent before `ready`, so the only thing `frames` can say while the socket opens
        // is that the hub moved on.
        let socket = tokio::select! {
            None = frames.recv() => return Ending::Dropped,
            opened = tokio::time::timeout(self.settings.connect_timeout, handshake) => match opened {
                Err(_) => return Ending::Failed("the handshake timed out".into()),
                Ok(Err(tungstenite::Error::Http(response))) if response.status() == StatusCode::UNAUTHORIZED => {
                    return Ending::Unauthorized;
                }
                Ok(Err(error)) => return Ending::Failed(error.to_string()),
                Ok(Ok((socket, _))) => socket,
            },
        };
        let (mut sink, mut stream) = socket.split();
        let start = tokio::time::Instant::now();
        let mut ping =
            tokio::time::interval_at(start + self.settings.ping_every, self.settings.ping_every);
        let stale = tokio::time::sleep(self.settings.stale_after);
        tokio::pin!(stale);
        loop {
            tokio::select! {
                frame = frames.recv() => match frame {
                    Some(frame) => {
                        if let Err(error) = send(&mut sink, &frame).await {
                            return Ending::Failed(error.to_string());
                        }
                    }
                    None => {
                        let _ = tokio::time::timeout(self.settings.close_timeout, sink.close()).await;
                        return Ending::Dropped;
                    }
                },
                _ = ping.tick() => {
                    if let Err(error) = send(&mut sink, &ClientFrame::Ping).await {
                        return Ending::Failed(error.to_string());
                    }
                }
                () = &mut stale => {
                    tracing::info!("realtime: no frame for {:?}; recycling the socket", self.settings.stale_after);
                    let _ = tokio::time::timeout(self.settings.close_timeout, sink.close()).await;
                    return Ending::Closed;
                }
                message = stream.next() => {
                    stale.as_mut().reset(tokio::time::Instant::now() + self.settings.stale_after);
                    match message {
                        Some(Ok(Message::Text(text))) => match serde_json::from_str::<ServerFrame>(text.as_str()) {
                            Ok(frame) => {
                                let _ = self.tx.send(Msg::Frame(self.generation, frame));
                            }
                            Err(error) => tracing::debug!(%error, "realtime: dropping an undecodable frame"),
                        },
                        Some(Ok(Message::Close(close))) => {
                            let expired = close.is_some_and(|close| u16::from(close.code) == CLOSE_TOKEN_EXPIRED);
                            return if expired { Ending::Unauthorized } else { Ending::Closed };
                        }
                        Some(Ok(_)) => {}
                        Some(Err(error)) => return Ending::Failed(error.to_string()),
                        None => return Ending::Closed,
                    }
                }
            }
        }
    }
}

async fn send<S>(sink: &mut S, frame: &ClientFrame) -> Result<(), tungstenite::Error>
where
    S: futures_util::Sink<Message, Error = tungstenite::Error> + Unpin,
{
    let text = serde_json::to_string(frame).expect("client frames always serialize");
    sink.send(Message::Text(text.into())).await
}
