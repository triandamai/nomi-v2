use std::time::Duration;

use futures_util::StreamExt;
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

type Socket = tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

/// The bridge subscribes to the broker after the upgrade; give it a moment before publishing.
const SUBSCRIBE_SETTLE: Duration = Duration::from_millis(500);
const FRAME_TIMEOUT: Duration = Duration::from_secs(5);

/// Another of the person's devices, listening on `path`.
async fn other_device(ws_base: &str, path: &str, token: &str) -> Socket {
    let mut request = format!("{ws_base}{path}").into_client_request().unwrap();
    request.headers_mut().insert("authorization", format!("Bearer {token}").parse().unwrap());
    let (ws, _) = tokio_tungstenite::connect_async(request).await.unwrap();
    tokio::time::sleep(SUBSCRIBE_SETTLE).await;
    ws
}

/// The first frame of `kind` the device hears, skipping others (the reply's stream, say).
async fn first_of_kind(ws: &mut Socket, kind: &str) -> Value {
    loop {
        let msg = tokio::time::timeout(FRAME_TIMEOUT, ws.next())
            .await
            .unwrap_or_else(|_| panic!("timed out waiting for {kind}"))
            .expect("stream ended")
            .unwrap();
        if let tokio_tungstenite::tungstenite::Message::Text(text) = msg {
            let frame: Value = serde_json::from_str(&text).unwrap();
            if frame["kind"] == kind {
                return frame;
            }
        }
    }
}

async fn send_from_this_device(client: &reqwest::Client, http_base: &str, token: &str, session_id: Uuid, text: &str) -> Value {
    client
        .post(format!("{http_base}/api/sessions/{session_id}/messages"))
        .bearer_auth(token)
        .json(&json!({ "text": text }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap()
}

// One test, not two: the server's shared publisher runs on the runtime that first connected
// it, and each sqlx::test has its own runtime, so a second test would publish into a dead one.
#[sqlx::test(migrations = "../../migrations")]
async fn a_message_sent_on_one_device_reaches_the_persons_other_devices(pool: PgPool) {
    nomi_server::project_events::init(nomi_test_support::TEST_MQTT_BROKER_HOST, nomi_test_support::TEST_MQTT_BROKER_PORT);
    nomi_server::user_events::init(nomi_test_support::TEST_MQTT_BROKER_HOST, nomi_test_support::TEST_MQTT_BROKER_PORT);
    let (http_base, ws_base) = spawn_app(pool.clone()).await;
    let client = reqwest::Client::new();
    let token = register_and_login(&client, &http_base, "two-devices@example.com").await;
    let session_id = create_session(&client, &http_base, &token).await;
    // A tablet with this chat open, and a laptop somewhere else in the app.
    let mut tablet = other_device(&ws_base, &format!("/api/sessions/{session_id}/ws"), &token).await;
    let mut laptop = other_device(&ws_base, "/api/me/activity/ws", &token).await;

    let sent = send_from_this_device(&client, &http_base, &token, session_id, "hello from my phone").await;

    let in_chat = first_of_kind(&mut tablet, "MessageCreated").await;
    assert_eq!(in_chat["message_id"], sent["user_message"]["id"]);
    let in_list = first_of_kind(&mut laptop, "ChatUpdated").await;
    assert_eq!(in_list["session_id"], json!(session_id));

    // A new inbox item (a publish that failed in the background) reaches the web app straight away.
    let user_id: Uuid = sqlx::query_scalar("SELECT user_id FROM web_credentials WHERE email = $1")
        .bind("two-devices@example.com")
        .fetch_one(&pool)
        .await
        .unwrap();
    let notice = nomi_server::notifications::Notice {
        kind: nomi_server::notifications::Kind::Deploy,
        title: "Shop couldn't be published".to_string(),
        body: "Shipyard refused it: plan limit reached".to_string(),
        link: Some("/projects/p1/app?sheet=deploy".to_string()),
    };
    nomi_server::notifications::send_to_inbox(&pool, user_id, notice).await.unwrap();
    let arrived = first_of_kind(&mut laptop, "NotificationCreated").await;
    assert_eq!(arrived["body"], "Shipyard refused it: plan limit reached");
}
