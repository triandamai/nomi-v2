//! Live changes over `/realtime`: what `nomi.realtime` connects to. A real server and real
//! WebSocket clients, the test database standing in for the shared one.
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use futures::{SinkExt, StreamExt};
use relay_control::backend::{ensure_backend, BackendKeys};
use relay_control::migrations::apply;
use relay_control::{realtime, state, Relay};
use relay_core::keys::user_token;
use relay_core::names::schema_for;
use relay_core::{Env, Tier};
use relay_migrate::MigrationFile;
use serde_json::{json, Value};
use sqlx::PgPool;
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};
use uuid::Uuid;

const DOMAIN: &str = "api.relay.test";
const ENV: Env = Env::Dev;
const WAIT: Duration = Duration::from_secs(5);
/// Long enough that a change that's coming would have come.
const QUIET: Duration = Duration::from_millis(700);

const INIT: &str = "create table note (\n  id bigint generated always as identity primary key,\n  owner uuid not null default nomi.uid(),\n  body text not null\n);\nalter table note enable row level security;\ncreate policy \"own notes\" on note using (owner = nomi.uid()) with check (owner = nomi.uid());\ncreate table room (\n  id int primary key,\n  name text not null\n);\nalter table room enable row level security;\ncreate policy \"everyone\" on room using (true);\n";

type Socket = WebSocketStream<MaybeTlsStream<TcpStream>>;

struct World {
    address: std::net::SocketAddr,
    relay: Arc<Relay>,
    keys: BackendKeys,
    pool: PgPool,
}

async fn world(pool: &PgPool) -> World {
    let shared = HashMap::from([(Env::Dev, pool.clone()), (Env::Production, pool.clone())]);
    let relay = Arc::new(Relay::new(pool.clone(), shared, [9u8; 32], DOMAIN.into()));
    let keys = ensure_backend(&relay, Uuid::new_v4(), "Notes", Tier::Free, None).await.unwrap();
    apply(&relay, keys.id, ENV, &[MigrationFile { path: "nomi/migrations/0001_init.sql".into(), content: INIT.into() }]).await.unwrap();
    realtime::save(&relay, keys.id, ENV, &["note".to_string(), "room".to_string()]).await.unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let app = relay_server::app::router(relay.clone(), "control-key".into());
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    World { address, relay, keys, pool: pool.clone() }
}

fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
}

impl World {
    fn anon(&self) -> String {
        self.keys.environments[ENV.as_str()].anon_key.clone()
    }

    async fn token(&self, user: Uuid) -> String {
        let row = state::environment(&self.relay, self.keys.id, ENV).await.unwrap().unwrap();
        user_token(&row.signing_secret, self.keys.id, ENV, user, Uuid::new_v4(), None, now(), 900)
    }

    async fn open(&self, apikey: &str) -> Result<Socket, tokio_tungstenite::tungstenite::Error> {
        let mut request = format!("ws://{}/realtime?apikey={apikey}", self.address).into_client_request().unwrap();
        request.headers_mut().insert("host", format!("{}-dev.{DOMAIN}", self.keys.slug).parse().unwrap());
        tokio_tungstenite::connect_async(request).await.map(|(socket, _)| socket)
    }

    /// Signed in as `user` (or anonymous) and subscribed to `tables`.
    async fn subscriber(&self, user: Option<Uuid>, tables: &[&str]) -> Socket {
        let mut socket = self.open(&self.anon()).await.unwrap();
        let token = match user {
            Some(user) => self.token(user).await,
            None => self.anon(),
        };
        send(&mut socket, json!({ "type": "auth", "token": token })).await;
        for table in tables {
            send(&mut socket, json!({ "type": "subscribe", "table": table })).await;
        }
        // Subscribing answers once it's in place.
        for _ in tables {
            let ready = next(&mut socket).await.expect("subscribed");
            assert_eq!(ready["type"], "subscribed", "{ready}");
        }
        socket
    }

    async fn sql(&self, statement: &str) {
        let schema = schema_for(self.keys.id, Tier::Free);
        sqlx::raw_sql(&format!("SET search_path = \"{schema}\", public; {statement}")).execute(&self.pool).await.unwrap();
    }
}

async fn send(socket: &mut Socket, message: Value) {
    socket.send(Message::Text(message.to_string())).await.unwrap();
}

/// The next message, or `None` if nothing comes in `WAIT`.
async fn next(socket: &mut Socket) -> Option<Value> {
    within(socket, WAIT).await
}

async fn within(socket: &mut Socket, wait: Duration) -> Option<Value> {
    loop {
        match tokio::time::timeout(wait, socket.next()).await {
            Ok(Some(Ok(Message::Text(text)))) => return Some(serde_json::from_str(&text).unwrap()),
            Ok(Some(Ok(_))) => continue,
            _ => return None,
        }
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn everyone_subscribed_sees_a_change_they_may_read(pool: PgPool) {
    let w = world(&pool).await;
    let mut first = w.subscriber(Some(Uuid::new_v4()), &["room"]).await;
    let mut second = w.subscriber(None, &["room"]).await;

    w.sql("INSERT INTO room (id, name) VALUES (7, 'lobby')").await;

    for socket in [&mut first, &mut second] {
        let change = next(socket).await.expect("a change");
        assert_eq!(change, json!({ "type": "change", "table": "room", "event": "insert", "row": { "id": 7, "name": "lobby" }, "key": { "id": 7 } }));
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn row_level_security_decides_who_hears_about_a_row(pool: PgPool) {
    let w = world(&pool).await;
    let (ana, ben) = (Uuid::new_v4(), Uuid::new_v4());
    let mut anas = w.subscriber(Some(ana), &["note"]).await;
    let mut bens = w.subscriber(Some(ben), &["note"]).await;

    w.sql(&format!("INSERT INTO note (owner, body) VALUES ('{ana}', 'mine')")).await;

    let change = next(&mut anas).await.expect("Ana sees her note");
    assert_eq!(change["row"]["body"], "mine");
    assert_eq!(within(&mut bens, QUIET).await, None, "Ben never hears of Ana's note");

    w.sql("UPDATE note SET body = 'edited'").await;
    assert_eq!(next(&mut anas).await.unwrap()["event"], "update");
    w.sql("DELETE FROM note").await;
    let deleted = next(&mut anas).await.unwrap();
    assert_eq!(deleted, json!({ "type": "change", "table": "note", "event": "delete", "row": null, "key": { "id": 1 } }));
}

#[sqlx::test(migrations = "../../migrations")]
async fn only_subscribed_realtime_tables_are_sent(pool: PgPool) {
    let w = world(&pool).await;
    let mut socket = w.subscriber(None, &["room"]).await;

    send(&mut socket, json!({ "type": "subscribe", "table": "secret_table" })).await;
    let refused = next(&mut socket).await.unwrap();
    assert_eq!(refused["type"], "error");
    assert!(refused["message"].as_str().unwrap().contains("secret_table"), "{refused}");

    send(&mut socket, json!({ "type": "unsubscribe", "table": "room" })).await;
    tokio::time::sleep(Duration::from_millis(100)).await;
    w.sql("INSERT INTO room (id, name) VALUES (1, 'gone')").await;
    assert_eq!(within(&mut socket, QUIET).await, None);
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_burst_over_the_limit_becomes_one_resync(pool: PgPool) {
    let w = world(&pool).await;
    let mut socket = w.subscriber(None, &["room"]).await;

    w.sql("INSERT INTO room (id, name) SELECT g, 'r' || g FROM generate_series(1, 80) g").await;

    let mut changes = 0;
    let mut resyncs = 0;
    while let Some(message) = within(&mut socket, QUIET).await {
        match message["type"].as_str() {
            Some("change") => changes += 1,
            Some("resync") => resyncs += 1,
            other => panic!("{other:?}"),
        }
    }
    assert_eq!(changes, relay_server::realtime::MAX_CHANGES_PER_SECOND);
    assert_eq!(resyncs, 1);
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_socket_needs_the_projects_key(pool: PgPool) {
    let w = world(&pool).await;
    assert!(w.open("not-a-key").await.is_err());
    let mut socket = w.open(&w.anon()).await.unwrap();
    send(&mut socket, json!({ "type": "auth", "token": "not-a-token" })).await;
    let refused = next(&mut socket).await.unwrap();
    assert_eq!(refused["type"], "error");
}
