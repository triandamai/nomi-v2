//! `/realtime` on a project's host: the WebSocket `nomi.realtime` keeps open. Changes come from
//! one `LISTEN nomi_realtime` per shared database (the triggers `realtime::save` adds announce
//! project, table, operation and key). Each insert or update is read back as the subscriber, under
//! the table's row-level security, and sent only to those who may see it; a delete sends its key.
//! Delivery is at most once: a reconnect or a burst over the limit becomes `resync`, and the app
//! refetches.
//!
//! The app sends `{ type: auth, token }`, `{ type: subscribe, table }` and
//! `{ type: unsubscribe, table }`; it gets `subscribed`, `change`, `resync` and `error`.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{FromRequestParts, Request};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use relay_control::{realtime, Relay};
use relay_core::names::schema_for;
use relay_core::{Env, Tier};
use relay_gateway::auth::{database_role, request_claims, resolve, Caller};
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::postgres::PgListener;
use sqlx::PgPool;
use tokio::sync::broadcast;
use uuid::Uuid;

use crate::gateway::{refuse, Gateway};

pub const REALTIME_PATH: &str = "/realtime";
/// Changes one socket gets in a second; the rest of that second becomes one `resync`.
pub const MAX_CHANGES_PER_SECOND: usize = 50;
/// Open sockets per project environment.
pub const MAX_SOCKETS: usize = 200;
const RATE_WINDOW: Duration = Duration::from_secs(1);
/// Changes waiting for slow sockets before they're told to resync instead.
const FEED_CAPACITY: usize = 4096;
const RECONNECT_DELAY: Duration = Duration::from_secs(1);
/// Reading a changed row as the subscriber stops after this, like the app's own requests.
const READ_TIMEOUT: &str = "8s";

/// One change, as the trigger announced it.
#[derive(Debug, Deserialize)]
struct Change {
    #[serde(rename = "p")]
    project: Uuid,
    #[serde(rename = "s")]
    schema: String,
    #[serde(rename = "t")]
    table: String,
    op: String,
    #[serde(rename = "k")]
    key: Value,
}

#[derive(Debug, Clone)]
enum Event {
    Change(Arc<Change>),
    /// The listener lost its connection: changes may have been missed.
    Resync,
}

/// The feeds, one per environment's shared database, and how many sockets each project has open.
#[derive(Default)]
pub struct Hub {
    feeds: Mutex<HashMap<Env, broadcast::Sender<Event>>>,
    sockets: Arc<Mutex<HashMap<(Uuid, Env), usize>>>,
}

/// Holds one of a project's sockets; letting go frees it.
struct Seat {
    sockets: Arc<Mutex<HashMap<(Uuid, Env), usize>>>,
    key: (Uuid, Env),
}

impl Drop for Seat {
    fn drop(&mut self) {
        if let Some(count) = self.sockets.lock().expect("sockets lock").get_mut(&self.key) {
            *count = count.saturating_sub(1);
        }
    }
}

impl Hub {
    fn seat(&self, project: Uuid, env: Env) -> Option<Seat> {
        let mut sockets = self.sockets.lock().expect("sockets lock");
        let count = sockets.entry((project, env)).or_default();
        if *count >= MAX_SOCKETS {
            return None;
        }
        *count += 1;
        Some(Seat { sockets: self.sockets.clone(), key: (project, env) })
    }

    /// The environment's feed, starting its listener the first time.
    fn feed(&self, relay: &Arc<Relay>, env: Env) -> broadcast::Receiver<Event> {
        let mut feeds = self.feeds.lock().expect("feeds lock");
        if let Some(sender) = feeds.get(&env) {
            return sender.subscribe();
        }
        let (sender, receiver) = broadcast::channel(FEED_CAPACITY);
        feeds.insert(env, sender.clone());
        tokio::spawn(listen(relay.clone(), env, sender));
        receiver
    }
}

/// Listens on the shared database for good, reconnecting (and telling every socket to resync)
/// whenever the connection drops. Reads the current settings on each connect.
async fn listen(relay: Arc<Relay>, env: Env, feed: broadcast::Sender<Event>) {
    let mut first = true;
    loop {
        let pool = relay.live().ok().and_then(|live| live.shared(env).ok());
        let listener = match pool {
            Some(pool) => match PgListener::connect_with(&pool).await {
                Ok(mut listener) => listener.listen(realtime::CHANNEL).await.map(|_| listener),
                Err(e) => Err(e),
            },
            None => Err(sqlx::Error::PoolClosed),
        };
        match listener {
            Ok(mut listener) => {
                if !first {
                    let _ = feed.send(Event::Resync);
                }
                first = false;
                loop {
                    match listener.try_recv().await {
                        Ok(Some(note)) => match serde_json::from_str::<Change>(note.payload()) {
                            Ok(change) => {
                                let _ = feed.send(Event::Change(Arc::new(change)));
                            }
                            Err(e) => tracing::warn!(error = %e, "unreadable realtime notification"),
                        },
                        // The connection dropped and will be made again: some changes are lost.
                        Ok(None) => {
                            let _ = feed.send(Event::Resync);
                        }
                        Err(e) => {
                            tracing::warn!(error = %e, env = env.as_str(), "realtime listener stopped");
                            break;
                        }
                    }
                }
            }
            Err(e) => tracing::warn!(error = %e, env = env.as_str(), "realtime listener can't connect"),
        }
        tokio::time::sleep(RECONNECT_DELAY).await;
    }
}

fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or_default()
}

fn query(request: &Request, name: &str) -> Option<String> {
    request.uri().query()?.split('&').find_map(|pair| {
        let (key, value) = pair.split_once('=')?;
        (key == name).then(|| percent_encoding::percent_decode_str(value).decode_utf8().ok().map(|v| v.into_owned())).flatten()
    })
}

/// Checks the key, then opens the socket.
pub async fn upgrade(gateway: &Gateway, slug: &str, env: Env, request: Request) -> Response {
    let (project, row) = match gateway.locate(slug, env).await {
        Ok(found) => found,
        Err(response) => return response,
    };
    if row.tier == Tier::Dedicated && gateway.relay.standalone().is_none() {
        return refuse(StatusCode::NOT_IMPLEMENTED, "RELAY_DEDICATED", "Dedicated backends aren't served by the gateway yet.");
    }
    let apikey = query(&request, "apikey").unwrap_or_default();
    if resolve(&row.signing_secret, project, env, Some(&apikey), None, now()).is_err() {
        return refuse(StatusCode::UNAUTHORIZED, "RELAY_BAD_KEY", "That apikey isn't this project's, or it's for the other environment.");
    }
    let pool = match gateway.relay.placement(project, env).await {
        Ok(placement) => placement.pool,
        Err(e) => return gateway.failed(e.to_string()),
    };
    let Some(seat) = gateway.hub.seat(project, env) else {
        return refuse(StatusCode::TOO_MANY_REQUESTS, "RELAY_TOO_MANY", "This app has as many live connections open as its plan allows.");
    };
    let (mut parts, _) = request.into_parts();
    let upgrade = match WebSocketUpgrade::from_request_parts(&mut parts, &()).await {
        Ok(upgrade) => upgrade,
        Err(rejection) => return rejection.into_response(),
    };
    let feed = gateway.hub.feed(&gateway.relay, env);
    let session = Session {
        relay: gateway.relay.clone(),
        pool,
        project,
        env,
        schema: schema_for(project, row.tier),
        secret: row.signing_secret,
        apikey,
        bearer: None,
        tables: HashSet::new(),
        window: Instant::now(),
        in_window: 0,
        _seat: seat,
    };
    upgrade.on_upgrade(move |socket| session.run(socket, feed))
}

/// One open socket.
struct Session {
    relay: Arc<Relay>,
    pool: PgPool,
    project: Uuid,
    env: Env,
    schema: String,
    secret: String,
    apikey: String,
    bearer: Option<String>,
    tables: HashSet<String>,
    window: Instant,
    in_window: usize,
    _seat: Seat,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ClientMessage {
    Auth { token: String },
    Subscribe { table: String },
    Unsubscribe { table: String },
}

fn text(message: Value) -> Message {
    Message::Text(message.to_string())
}

impl Session {
    async fn run(mut self, mut socket: WebSocket, mut feed: broadcast::Receiver<Event>) {
        loop {
            let outgoing = tokio::select! {
                incoming = socket.recv() => match incoming {
                    Some(Ok(Message::Text(text))) => self.on_message(&text).await,
                    Some(Ok(Message::Close(_))) | None | Some(Err(_)) => return,
                    Some(Ok(_)) => Vec::new(),
                },
                event = feed.recv() => match event {
                    Ok(Event::Change(change)) => self.on_change(&change).await,
                    Ok(Event::Resync) | Err(broadcast::error::RecvError::Lagged(_)) => vec![json!({ "type": "resync" })],
                    Err(broadcast::error::RecvError::Closed) => return,
                },
            };
            for message in outgoing {
                if socket.send(text(message)).await.is_err() {
                    return;
                }
            }
        }
    }

    /// Who's listening now: the user while their token is good, otherwise the key's role.
    fn caller(&self) -> Option<Caller> {
        resolve(&self.secret, self.project, self.env, Some(&self.apikey), self.bearer.as_deref(), now())
            .or_else(|_| resolve(&self.secret, self.project, self.env, Some(&self.apikey), None, now()))
            .ok()
    }

    async fn on_message(&mut self, text: &str) -> Vec<Value> {
        let Ok(message) = serde_json::from_str::<ClientMessage>(text) else {
            return vec![json!({ "type": "error", "message": "Send auth, subscribe or unsubscribe." })];
        };
        match message {
            ClientMessage::Auth { token } => {
                if token.is_empty() || token == self.apikey {
                    self.bearer = None;
                    return Vec::new();
                }
                match resolve(&self.secret, self.project, self.env, Some(&self.apikey), Some(&token), now()) {
                    Ok(_) => {
                        self.bearer = Some(token);
                        Vec::new()
                    }
                    Err(refusal) => {
                        self.bearer = None;
                        vec![json!({ "type": "error", "message": refusal.message() })]
                    }
                }
            }
            ClientMessage::Subscribe { table } => match realtime::load(&self.relay, self.project, self.env).await {
                Ok(tables) if tables.contains(&table) => {
                    self.tables.insert(table.clone());
                    vec![json!({ "type": "subscribed", "table": table })]
                }
                Ok(_) => vec![json!({ "type": "error", "message": format!("{table} isn't a realtime table: list it in nomi/backend.json's realtime.") })],
                Err(e) => {
                    tracing::error!(error = %e, "realtime tables");
                    vec![json!({ "type": "error", "message": "Relay couldn't subscribe right now." })]
                }
            },
            ClientMessage::Unsubscribe { table } => {
                self.tables.remove(&table);
                Vec::new()
            }
        }
    }

    async fn on_change(&mut self, change: &Change) -> Vec<Value> {
        if change.project != self.project || change.schema != self.schema || !self.tables.contains(&change.table) {
            return Vec::new();
        }
        if self.window.elapsed() >= RATE_WINDOW {
            self.window = Instant::now();
            self.in_window = 0;
        }
        self.in_window += 1;
        if self.in_window > MAX_CHANGES_PER_SECOND {
            return if self.in_window == MAX_CHANGES_PER_SECOND + 1 { vec![json!({ "type": "resync" })] } else { Vec::new() };
        }
        let row = if change.op == "delete" {
            Value::Null
        } else {
            let Some(caller) = self.caller() else { return Vec::new() };
            match read_row(&self.pool, self.project, &caller, &self.schema, &change.table, &change.key).await {
                Ok(Some(row)) => row,
                Ok(None) => return Vec::new(),
                Err(e) => {
                    tracing::warn!(error = %e, table = %change.table, "realtime read failed");
                    return Vec::new();
                }
            }
        };
        vec![json!({ "type": "change", "table": change.table, "event": change.op, "row": row, "key": change.key })]
    }
}

fn quoted(identifier: &str) -> String {
    format!("\"{}\"", identifier.replace('"', "\"\""))
}

/// The changed row as `caller` sees it, or `None` when row-level security hides it (or it's gone).
async fn read_row(pool: &PgPool, project: Uuid, caller: &Caller, schema: &str, table: &str, key: &Value) -> Result<Option<Value>, sqlx::Error> {
    let Some(columns) = key.as_object().filter(|k| !k.is_empty()) else { return Ok(None) };
    let relation = format!("{}.{}", quoted(schema), quoted(table));
    let matches: Vec<String> = columns.keys().map(|c| format!("t.{0} = (jsonb_populate_record(null::{relation}, $1)).{0}", quoted(c))).collect();
    let mut tx = pool.begin().await?;
    sqlx::query(&format!("SET LOCAL ROLE {}", quoted(&database_role(project, caller)))).execute(&mut *tx).await?;
    sqlx::query(&format!("SET LOCAL statement_timeout = '{READ_TIMEOUT}'")).execute(&mut *tx).await?;
    sqlx::query("SELECT set_config('request.jwt.claims', $1, true)").bind(request_claims(project, caller).to_string()).execute(&mut *tx).await?;
    let row: Option<Value> = sqlx::query_scalar(&format!("SELECT to_jsonb(t) FROM {relation} t WHERE {} LIMIT 1", matches.join(" AND ")))
        .bind(key)
        .fetch_optional(&mut *tx)
        .await?;
    tx.rollback().await?;
    Ok(row)
}
