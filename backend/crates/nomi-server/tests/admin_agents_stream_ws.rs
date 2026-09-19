use std::sync::Arc;
use std::time::Duration;

use futures_util::StreamExt;
use serde_json::{json, Value};
use sqlx::PgPool;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use uuid::Uuid;

use nomi_agent_chitchat::ChitchatAgent;
use nomi_agent_core::{AgentRegistry, ToolCatalog};
use nomi_agent_money::MoneyAgent;
use nomi_llm::{ContentBlock, LlmResponse, StopReason};
use nomi_realtime::MqttPublisher;
use nomi_server::app::{build_router, AppState};
use nomi_test_support::{dummy_embedding, FakeEmbeddingProvider, FakeLlmProvider};
use nomi_turn::process_turn;

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
        tool_catalog: Arc::new(ToolCatalog::empty()),
    }
}

/// Binds a real TCP listener and serves the app on it — a WebSocket upgrade is a persistent
/// duplex connection, which `tower::oneshot` (used by this crate's other route tests) cannot
/// drive. Returns (http_base_url, ws_base_url).
async fn spawn_app(pool: PgPool) -> (String, String) {
    let app = build_router(test_state(pool));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    (format!("http://{addr}"), format!("ws://{addr}"))
}

async fn register(client: &reqwest::Client, base: &str, email: &str) {
    client
        .post(format!("{base}/api/auth/register"))
        .json(&json!({ "email": email, "password": "correct-password", "org": { "mode": "create", "name": "Acme" } }))
        .send()
        .await
        .unwrap();
}

async fn login(client: &reqwest::Client, base: &str, email: &str) -> String {
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

async fn register_and_login(client: &reqwest::Client, base: &str, email: &str) -> String {
    register(client, base, email).await;
    login(client, base, email).await
}

async fn make_platform_admin(pool: &PgPool, email: &str) {
    sqlx::query("UPDATE users SET is_platform_admin = true WHERE id = (SELECT user_id FROM web_credentials WHERE email = $1)")
        .bind(email)
        .execute(pool)
        .await
        .unwrap();
}

/// Registers, promotes to platform admin, THEN logs in — JWT claims bake in permissions at
/// issuance, so promoting after an already-issued token would leave that token non-admin.
async fn register_admin_and_login(client: &reqwest::Client, pool: &PgPool, base: &str, email: &str) -> String {
    register(client, base, email).await;
    make_platform_admin(pool, email).await;
    login(client, base, email).await
}

async fn try_connect_admin_ws(
    ws_base: &str,
    token: Option<&str>,
) -> Result<
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
    tokio_tungstenite::tungstenite::Error,
> {
    let url = format!("{ws_base}/api/admin/agents/ws");
    let mut request = url.into_client_request().unwrap();
    if let Some(token) = token {
        request.headers_mut().insert("authorization", format!("Bearer {token}").parse().unwrap());
    }
    tokio_tungstenite::connect_async(request).await.map(|(ws, _)| ws)
}

fn canned_response(text: &str) -> LlmResponse {
    LlmResponse {
        content: vec![ContentBlock::Text { text: text.to_string() }],
        stop_reason: StopReason::EndTurn,
        input_tokens: 1,
        output_tokens: 1,
    }
}

/// Reads WS text frames for up to `timeout`, parsing each as JSON and returning every frame
/// received — used both to assert specific frames arrived (non-empty) and that none did (empty).
async fn collect_frames_within(
    ws: &mut tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
    timeout: Duration,
) -> Vec<Value> {
    let mut frames = Vec::new();
    let deadline = tokio::time::Instant::now() + timeout;
    loop {
        let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
        if remaining.is_zero() {
            break;
        }
        match tokio::time::timeout(remaining, ws.next()).await {
            Ok(Some(Ok(tokio_tungstenite::tungstenite::Message::Text(text)))) => {
                if let Ok(value) = serde_json::from_str::<Value>(&text) {
                    frames.push(value);
                }
            }
            Ok(Some(Ok(_))) => continue,
            _ => break,
        }
    }
    frames
}

#[sqlx::test(migrations = "../../migrations")]
async fn ws_upgrade_without_a_token_is_rejected(pool: PgPool) {
    let (_, ws_base) = spawn_app(pool).await;
    let err = try_connect_admin_ws(&ws_base, None).await.unwrap_err();
    match err {
        tokio_tungstenite::tungstenite::Error::Http(response) => assert_eq!(response.status().as_u16(), 401),
        other => panic!("expected a 401 HTTP rejection, got {other:?}"),
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn ws_upgrade_from_a_non_admin_is_rejected(pool: PgPool) {
    let (http_base, ws_base) = spawn_app(pool).await;
    let client = reqwest::Client::new();
    let token = register_and_login(&client, &http_base, "regular@example.com").await;

    let err = try_connect_admin_ws(&ws_base, Some(&token)).await.unwrap_err();
    match err {
        tokio_tungstenite::tungstenite::Error::Http(response) => assert_eq!(response.status().as_u16(), 403),
        other => panic!("expected a 403 HTTP rejection, got {other:?}"),
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn agent_session_started_and_ended_are_forwarded_live(pool: PgPool) {
    let (http_base, ws_base) = spawn_app(pool.clone()).await;
    let client = reqwest::Client::new();
    let admin_token = register_admin_and_login(&client, &pool, &http_base, "admin@example.com").await;
    let mut admin_ws = try_connect_admin_ws(&ws_base, Some(&admin_token)).await.unwrap();

    // A money-intent turn that immediately completes via complete_task — this spawns and then
    // closes an agent_sessions row in one turn, exercising both new envelope kinds.
    let ingested = nomi_turn::ingest::ingest_inbound_message(&pool, "telegram", "dm", "chat-1", "tg-1", "how much did I spend?", None)
        .await
        .unwrap();
    let user_id: Uuid = sqlx::query_scalar("SELECT user_id FROM channel_identities WHERE id = $1")
        .bind(ingested.sender_channel_identity_id)
        .fetch_one(&pool)
        .await
        .unwrap();

    let provider = FakeLlmProvider::sequence(vec![
        canned_response("money"),
        LlmResponse {
            content: vec![ContentBlock::ToolUse {
                id: "t1".to_string(),
                name: "complete_task".to_string(),
                input: json!({"status": "completed", "summary": "Done"}),
                thought_signature: None,
            }],
            stop_reason: StopReason::ToolUse,
            input_tokens: 1,
            output_tokens: 1,
        },
    ]);
    let embedder = FakeEmbeddingProvider::success(dummy_embedding());
    let registry = AgentRegistry::new(vec![Box::new(MoneyAgent), Box::new(ChitchatAgent)]);
    let catalog: Arc<ToolCatalog> = Arc::new(ToolCatalog::empty());
    let mqtt = MqttPublisher::connect(
        nomi_test_support::TEST_MQTT_BROKER_HOST,
        nomi_test_support::TEST_MQTT_BROKER_PORT,
        &format!("test-command-center-{}", Uuid::new_v4()),
    );

    process_turn(
        &pool, &mqtt, None, &provider, &embedder, &registry, &catalog, ingested.turn_job_id, ingested.session_id,
        ingested.sender_channel_identity_id, user_id, "how much did I spend?",
    )
    .await
    .unwrap();

    let frames = collect_frames_within(&mut admin_ws, Duration::from_secs(5)).await;
    let started = frames.iter().find(|f| f["kind"] == "AgentSessionStarted").expect("expected an AgentSessionStarted frame");
    assert_eq!(started["agent_type"], "money");
    assert_eq!(started["agent_display_name"], "Money");
    assert_eq!(started["session_id"], ingested.session_id.to_string());

    let ended = frames.iter().find(|f| f["kind"] == "AgentSessionEnded").expect("expected an AgentSessionEnded frame");
    assert_eq!(ended["reason"], "completed");
}

#[sqlx::test(migrations = "../../migrations")]
async fn message_content_never_reaches_the_admin_socket(pool: PgPool) {
    let (http_base, ws_base) = spawn_app(pool.clone()).await;
    let client = reqwest::Client::new();
    let admin_token = register_admin_and_login(&client, &pool, &http_base, "admin2@example.com").await;
    let mut admin_ws = try_connect_admin_ws(&ws_base, Some(&admin_token)).await.unwrap();

    // A plain chitchat turn: publishes Delta chunks and MessageCreated on the per-session topic
    // (and never AgentPhaseChanged, since chitchat has no real agent_sessions row — see
    // engine.rs's update_agent_phase, a no-op UPDATE when agent_session_id == session_id). None
    // of that should ever cross onto the admin-wide relay.
    let ingested = nomi_turn::ingest::ingest_inbound_message(&pool, "telegram", "dm", "chat-2", "tg-2", "hello", None).await.unwrap();
    let user_id: Uuid = sqlx::query_scalar("SELECT user_id FROM channel_identities WHERE id = $1")
        .bind(ingested.sender_channel_identity_id)
        .fetch_one(&pool)
        .await
        .unwrap();

    let provider = FakeLlmProvider::success(canned_response("Hi there!"));
    let embedder = FakeEmbeddingProvider::success(dummy_embedding());
    let registry = AgentRegistry::new(vec![Box::new(MoneyAgent), Box::new(ChitchatAgent)]);
    let catalog: Arc<ToolCatalog> = Arc::new(ToolCatalog::empty());
    let mqtt = MqttPublisher::connect(
        nomi_test_support::TEST_MQTT_BROKER_HOST,
        nomi_test_support::TEST_MQTT_BROKER_PORT,
        &format!("test-command-center-{}", Uuid::new_v4()),
    );

    process_turn(
        &pool, &mqtt, None, &provider, &embedder, &registry, &catalog, ingested.turn_job_id, ingested.session_id,
        ingested.sender_channel_identity_id, user_id, "hello",
    )
    .await
    .unwrap();

    // The admin socket is a wildcard relay across every session on the (shared, real) test MQTT
    // broker, so tests running concurrently in other threads legitimately publish frames for
    // their own sessions here too — that's the relay working as designed. Scope the assertion
    // to this test's own session_id: a chitchat turn (no real agent_sessions row) must produce
    // zero frames for *that* session specifically.
    let frames = collect_frames_within(&mut admin_ws, Duration::from_secs(3)).await;
    let own_session_frames: Vec<&Value> = frames.iter().filter(|f| f["session_id"] == ingested.session_id.to_string()).collect();
    assert!(own_session_frames.is_empty(), "expected no frames for the chitchat session on the admin socket, got: {own_session_frames:?}");
}
