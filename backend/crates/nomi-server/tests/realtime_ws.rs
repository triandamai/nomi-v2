

use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use sqlx::PgPool;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use uuid::Uuid;

use nomi_server::app::{build_router, AppState};

const SECRET: &str = "test-secret-do-not-use-in-prod";

fn test_state(pool: PgPool) -> AppState {
    AppState {
        pool,
        jwt_secret: SECRET.to_string(),
        http_client: reqwest::Client::new(),
        settings_key: nomi_test_support::TEST_SETTINGS_KEY,
        mqtt_broker_host: nomi_test_support::TEST_MQTT_BROKER_HOST.to_string(),
        mqtt_broker_port: nomi_test_support::TEST_MQTT_BROKER_PORT,
        s3: None,
        project_storage: nomi_test_support::test_project_storage(),
        tool_catalog: std::sync::Arc::new(nomi_agent_core::ToolCatalog::empty()),
        email_codes: nomi_server::sign_in_codes::EmailCodes::Off,
        push: std::sync::Arc::new(nomi_server::push::NoopPush),
        shipyard: None,
    }
}

/// Binds a real TCP listener and serves the app on it — a WebSocket upgrade is a persistent
/// duplex connection, which `tower::oneshot` (used by the other route tests) cannot drive.
/// Returns (http_base_url, ws_base_url).
async fn spawn_app(pool: PgPool) -> (String, String) {
    let app = build_router(test_state(pool));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    (format!("http://{addr}"), format!("ws://{addr}"))
}

async fn register_and_login(client: &reqwest::Client, base: &str, email: &str) -> String {
    client
        .post(format!("{base}/api/auth/register"))
        .json(&json!({ "email": email, "password": "correct-password", "org": { "mode": "create", "name": "Acme" } }))
        .send()
        .await
        .unwrap();
    let login: Value = client
        .post(format!("{base}/api/auth/login"))
        .json(&json!({ "email": email, "password": "correct-password" }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    login["access_token"].as_str().unwrap().to_string()
}

async fn create_session(client: &reqwest::Client, base: &str, token: &str) -> Uuid {
    let body: Value = client
        .post(format!("{base}/api/sessions"))
        .bearer_auth(token)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    Uuid::parse_str(body["session_id"].as_str().unwrap()).unwrap()
}

use std::time::Duration;
use tokio_tungstenite::tungstenite::Message as WsMessage;
use nomi_realtime::MqttPublisher;

const FRAME_TIMEOUT: Duration = Duration::from_secs(5);
const PUBLISH_RETRY_INTERVAL: Duration = Duration::from_millis(100);
const SILENCE_WINDOW: Duration = Duration::from_millis(800);

type Ws = tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

async fn connect(ws_base: &str, token: Option<&str>) -> Result<Ws, tokio_tungstenite::tungstenite::Error> {
    let mut req = format!("{ws_base}/api/realtime/ws").into_client_request().unwrap();
    if let Some(t) = token {
        req.headers_mut().insert("Authorization", format!("Bearer {t}").parse().unwrap());
    }
    tokio_tungstenite::connect_async(req).await.map(|(ws, _)| ws)
}

async fn next_json(ws: &mut Ws) -> Value {
    loop {
        let msg = tokio::time::timeout(FRAME_TIMEOUT, ws.next()).await.expect("timed out").unwrap().unwrap();
        if let WsMessage::Text(t) = msg { return serde_json::from_str(&t).unwrap(); }
    }
}

async fn send(ws: &mut Ws, v: Value) {
    ws.send(WsMessage::Text(v.to_string().into())).await.unwrap();
}

/// The gateway's MQTT SUBSCRIBE is asynchronous, so publish repeatedly until the event shows up.
async fn publish_until_event(ws: &mut Ws, mqtt_topic: &str, payload: Value) -> Value {
    let publisher = MqttPublisher::connect(nomi_test_support::TEST_MQTT_BROKER_HOST, nomi_test_support::TEST_MQTT_BROKER_PORT, &format!("test-pub-{}", Uuid::new_v4()));
    let deadline = tokio::time::Instant::now() + FRAME_TIMEOUT;
    loop {
        publisher.try_publish_to(mqtt_topic, &payload).unwrap();
        if let Ok(Some(Ok(WsMessage::Text(t)))) = tokio::time::timeout(PUBLISH_RETRY_INTERVAL, ws.next()).await {
            let v: Value = serde_json::from_str(&t).unwrap();
            if v["type"] == "event" { return v; }
        }
        assert!(tokio::time::Instant::now() < deadline, "event never arrived on {mqtt_topic}");
    }
}

async fn assert_silent(ws: &mut Ws) {
    if let Ok(Some(Ok(WsMessage::Text(t)))) = tokio::time::timeout(SILENCE_WINDOW, ws.next()).await {
        panic!("expected silence, got {t}");
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn upgrade_without_token_is_rejected(pool: PgPool) {
    let (_, ws_base) = spawn_app(pool).await;
    assert!(connect(&ws_base, None).await.is_err());
}

#[sqlx::test(migrations = "../../migrations")]
async fn ready_then_user_topic_events_arrive(pool: PgPool) {
    let (base, ws_base) = spawn_app(pool).await;
    let client = reqwest::Client::new();
    let token = register_and_login(&client, &base, "rt-user@nomi.test").await;
    let mut ws = connect(&ws_base, Some(&token)).await.unwrap();
    let ready = next_json(&mut ws).await;
    assert_eq!(ready["type"], "ready");
    let user_id = Uuid::parse_str(ready["user_id"].as_str().unwrap()).unwrap();

    let ev = publish_until_event(&mut ws, &format!("user/{user_id}/stream"), json!({"kind":"Ping"})).await;
    assert_eq!(ev["topic"], "user");
    assert_eq!(ev["payload"]["kind"], "Ping");
}

#[sqlx::test(migrations = "../../migrations")]
async fn chat_activity_on_the_user_activity_topic_arrives_as_user(pool: PgPool) {
    let (base, ws_base) = spawn_app(pool).await;
    let client = reqwest::Client::new();
    let token = register_and_login(&client, &base, "rt-activity@nomi.test").await;
    let mut ws = connect(&ws_base, Some(&token)).await.unwrap();
    let ready = next_json(&mut ws).await;
    let user_id = Uuid::parse_str(ready["user_id"].as_str().unwrap()).unwrap();

    let activity = nomi_realtime::UserEnvelope::ChatActivity {
        session_id: Uuid::new_v4(),
        title: Some("Trip".into()),
        project_id: None,
        outcome: "waiting".into(),
        waiting: Some("question".into()),
        agent: Some("Nomi".into()),
    };
    let ev = publish_until_event(&mut ws, &nomi_realtime::user_topic(user_id), serde_json::to_value(&activity).unwrap()).await;
    assert_eq!(ev["topic"], "user");
    assert_eq!(ev["payload"]["kind"], "ChatActivity");
    assert_eq!(ev["payload"]["waiting"], "question");
}

#[sqlx::test(migrations = "../../migrations")]
async fn subscribe_to_own_chat_relays_its_events(pool: PgPool) {
    let (base, ws_base) = spawn_app(pool).await;
    let client = reqwest::Client::new();
    let token = register_and_login(&client, &base, "rt-chat@nomi.test").await;
    let session = create_session(&client, &base, &token).await;
    let mut ws = connect(&ws_base, Some(&token)).await.unwrap();
    next_json(&mut ws).await; // ready

    send(&mut ws, json!({"op":"subscribe","topic":format!("chat:{session}")})).await;
    let ack = next_json(&mut ws).await;
    assert_eq!(ack, json!({"type":"subscribed","topic":format!("chat:{session}")}));

    let ev = publish_until_event(&mut ws, &format!("chat/{session}/stream"), json!({"kind":"PersonalityChanged"})).await;
    assert_eq!(ev["topic"], format!("chat:{session}"));
}

#[sqlx::test(migrations = "../../migrations")]
async fn subscribe_with_non_hyphenated_uuid_echoes_canonical_topic(pool: PgPool) {
    let (base, ws_base) = spawn_app(pool).await;
    let client = reqwest::Client::new();
    let token = register_and_login(&client, &base, "rt-canon@nomi.test").await;
    let session = create_session(&client, &base, &token).await;
    let mut ws = connect(&ws_base, Some(&token)).await.unwrap();
    next_json(&mut ws).await; // ready

    send(&mut ws, json!({"op":"subscribe","topic":format!("chat:{}", session.simple())})).await;
    let ack = next_json(&mut ws).await;
    assert_eq!(ack, json!({"type":"subscribed","topic":format!("chat:{session}")}));
}

#[sqlx::test(migrations = "../../migrations")]
async fn subscribe_to_foreign_chat_is_denied_and_silent(pool: PgPool) {
    let (base, ws_base) = spawn_app(pool).await;
    let client = reqwest::Client::new();
    let owner = register_and_login(&client, &base, "rt-owner@nomi.test").await;
    let foreign_session = create_session(&client, &base, &owner).await;
    let intruder = register_and_login(&client, &base, "rt-intruder@nomi.test").await;

    let mut ws = connect(&ws_base, Some(&intruder)).await.unwrap();
    next_json(&mut ws).await; // ready
    send(&mut ws, json!({"op":"subscribe","topic":format!("chat:{foreign_session}")})).await;
    assert_eq!(next_json(&mut ws).await["type"], "denied");

    let publisher = MqttPublisher::connect(nomi_test_support::TEST_MQTT_BROKER_HOST, nomi_test_support::TEST_MQTT_BROKER_PORT, "test-pub-foreign");
    publisher.try_publish_to(&format!("chat/{foreign_session}/stream"), &json!({"kind":"Leak"})).unwrap();
    assert_silent(&mut ws).await;
}

#[sqlx::test(migrations = "../../migrations")]
async fn unsubscribe_stops_events(pool: PgPool) {
    let (base, ws_base) = spawn_app(pool).await;
    let client = reqwest::Client::new();
    let token = register_and_login(&client, &base, "rt-unsub@nomi.test").await;
    let session = create_session(&client, &base, &token).await;
    let mut ws = connect(&ws_base, Some(&token)).await.unwrap();
    next_json(&mut ws).await;
    send(&mut ws, json!({"op":"subscribe","topic":format!("chat:{session}")})).await;
    next_json(&mut ws).await;
    publish_until_event(&mut ws, &format!("chat/{session}/stream"), json!({"kind":"A"})).await;

    send(&mut ws, json!({"op":"unsubscribe","topic":format!("chat:{session}")})).await;
    // drain until the unsubscribed ack (late events from the retry loop may precede it)
    loop { if next_json(&mut ws).await["type"] == "unsubscribed" { break; } }
    let publisher = MqttPublisher::connect(nomi_test_support::TEST_MQTT_BROKER_HOST, nomi_test_support::TEST_MQTT_BROKER_PORT, "test-pub-unsub");
    publisher.try_publish_to(&format!("chat/{session}/stream"), &json!({"kind":"B"})).unwrap();
    assert_silent(&mut ws).await;
}

#[sqlx::test(migrations = "../../migrations")]
async fn malformed_frame_gets_error_and_socket_survives(pool: PgPool) {
    let (base, ws_base) = spawn_app(pool).await;
    let client = reqwest::Client::new();
    let token = register_and_login(&client, &base, "rt-bad@nomi.test").await;
    let mut ws = connect(&ws_base, Some(&token)).await.unwrap();
    next_json(&mut ws).await;
    ws.send(WsMessage::Text("{not json".into())).await.unwrap();
    assert_eq!(next_json(&mut ws).await["type"], "error");
    send(&mut ws, json!({"op":"ping"})).await;
    assert_eq!(next_json(&mut ws).await, json!({"type":"pong"}));
}

#[sqlx::test(migrations = "../../migrations")]
async fn client_topic_cap_excludes_the_user_topic(pool: PgPool) {
    const CAP: usize = 64;
    let (base, ws_base) = spawn_app(pool).await;
    let client = reqwest::Client::new();
    let token = register_and_login(&client, &base, "rt-cap@nomi.test").await;
    let mut ws = connect(&ws_base, Some(&token)).await.unwrap();
    next_json(&mut ws).await; // ready
    for _ in 0..CAP {
        let session = create_session(&client, &base, &token).await;
        send(&mut ws, json!({"op":"subscribe","topic":format!("chat:{session}")})).await;
        assert_eq!(next_json(&mut ws).await["type"], "subscribed");
    }
    let one_more = create_session(&client, &base, &token).await;
    send(&mut ws, json!({"op":"subscribe","topic":format!("chat:{one_more}")})).await;
    assert_eq!(next_json(&mut ws).await["type"], "denied");
}

#[sqlx::test(migrations = "../../migrations")]
async fn inbox_notification_reaches_the_user_topic(pool: PgPool) {
    use nomi_server::notifications::{Kind, Notice};
    let (base, ws_base) = spawn_app(pool.clone()).await;
    let client = reqwest::Client::new();
    let token = register_and_login(&client, &base, "rt-inbox@nomi.test").await;
    let mut ws = connect(&ws_base, Some(&token)).await.unwrap();
    let ready = next_json(&mut ws).await;
    let user_id = Uuid::parse_str(ready["user_id"].as_str().unwrap()).unwrap();
    nomi_server::user_events::init(nomi_test_support::TEST_MQTT_BROKER_HOST, nomi_test_support::TEST_MQTT_BROKER_PORT);

    // The gateway's MQTT SUBSCRIBE is asynchronous, so send until the event shows up.
    let deadline = tokio::time::Instant::now() + FRAME_TIMEOUT;
    let ev = loop {
        nomi_server::notifications::send_to_inbox(&pool, user_id, Notice { kind: Kind::Account, title: "Hello".into(), body: "B".into(), link: None }).await.unwrap();
        if let Ok(Some(Ok(WsMessage::Text(t)))) = tokio::time::timeout(PUBLISH_RETRY_INTERVAL, ws.next()).await {
            let v: Value = serde_json::from_str(&t).unwrap();
            if v["type"] == "event" { break v; }
        }
        assert!(tokio::time::Instant::now() < deadline, "notification never arrived");
    };
    assert_eq!(ev["topic"], "user");
    assert_eq!(ev["payload"]["kind"], "NotificationCreated");
    assert_eq!(ev["payload"]["title"], "Hello");
}

/// Long enough to read `ready`, short enough that the test does not drag.
const SHORT_TOKEN_TTL_SECS: i64 = 2;
const CLOSE_TOKEN_EXPIRED: u16 = 4001;

#[sqlx::test(migrations = "../../migrations")]
async fn socket_closes_with_4001_when_its_token_expires(pool: PgPool) {
    let (_, ws_base) = spawn_app(pool).await;
    let claims = nomi_auth::claims::Claims::new(Uuid::new_v4(), Uuid::new_v4(), vec![], SHORT_TOKEN_TTL_SECS);
    let token = claims.encode(SECRET).unwrap();
    let mut ws = connect(&ws_base, Some(&token)).await.unwrap();
    assert_eq!(next_json(&mut ws).await["type"], "ready");

    let wait = FRAME_TIMEOUT + Duration::from_secs(SHORT_TOKEN_TTL_SECS as u64);
    let close = tokio::time::timeout(wait, async {
        loop {
            match ws.next().await {
                Some(Ok(WsMessage::Close(frame))) => break frame,
                Some(Ok(_)) => continue,
                other => panic!("socket ended without a close frame: {other:?}"),
            }
        }
    })
    .await
    .expect("socket outlived its token");
    let frame = close.expect("close frame carries a code");
    assert_eq!(u16::from(frame.code), CLOSE_TOKEN_EXPIRED);
    assert_eq!(frame.reason, "token expired");
}

/// Past `exp` but inside jsonwebtoken's default 60s leeway, so the extractor still accepts it.
const RECENTLY_EXPIRED_SECS: i64 = -5;

#[sqlx::test(migrations = "../../migrations")]
async fn upgrade_with_an_already_expired_token_is_rejected(pool: PgPool) {
    let (_, ws_base) = spawn_app(pool).await;
    // Accepting it would open a socket only to close it with 4001 at once, over and over until the leeway passes;
    // a 401 sends the client through its token refresh instead.
    let claims = nomi_auth::claims::Claims::new(Uuid::new_v4(), Uuid::new_v4(), vec![], RECENTLY_EXPIRED_SECS);
    let token = claims.encode(SECRET).unwrap();
    assert!(connect(&ws_base, Some(&token)).await.is_err());
}
