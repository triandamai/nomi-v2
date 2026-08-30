
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

async fn try_connect_ws(
    ws_base: &str,
    session_id: Uuid,
    token: Option<&str>,
) -> Result<
    (
        tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
        tokio_tungstenite::tungstenite::http::Response<Option<Vec<u8>>>,
    ),
    tokio_tungstenite::tungstenite::Error,
> {
    let url = format!("{ws_base}/api/sessions/{session_id}/ws");
    let mut request = url.into_client_request().unwrap();
    if let Some(token) = token {
        request
            .headers_mut()
            .insert("authorization", format!("Bearer {token}").parse().unwrap());
    }
    tokio_tungstenite::connect_async(request).await
}

fn assert_rejected_with(err: tokio_tungstenite::tungstenite::Error, expected_status: u16) {
    match err {
        tokio_tungstenite::tungstenite::Error::Http(response) => {
            assert_eq!(response.status().as_u16(), expected_status);
        }
        other => panic!("expected an HTTP rejection with status {expected_status}, got {other:?}"),
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn ws_upgrade_without_a_token_is_rejected(pool: PgPool) {
    let (_, ws_base) = spawn_app(pool).await;
    let err = try_connect_ws(&ws_base, Uuid::new_v4(), None).await.unwrap_err();
    assert_rejected_with(err, 401);
}

#[sqlx::test(migrations = "../../migrations")]
async fn ws_upgrade_for_another_orgs_session_is_rejected(pool: PgPool) {
    let (http_base, ws_base) = spawn_app(pool).await;
    let client = reqwest::Client::new();
    let token_a = register_and_login(&client, &http_base, "a@example.com").await;
    let token_b = register_and_login(&client, &http_base, "b@example.com").await;
    let session_id = create_session(&client, &http_base, &token_a).await;

    let err = try_connect_ws(&ws_base, session_id, Some(&token_b)).await.unwrap_err();
    assert_rejected_with(err, 404);
}

#[sqlx::test(migrations = "../../migrations")]
async fn ws_upgrade_with_a_valid_token_for_the_callers_own_session_succeeds(pool: PgPool) {
    let (http_base, ws_base) = spawn_app(pool).await;
    let client = reqwest::Client::new();
    let token = register_and_login(&client, &http_base, "c@example.com").await;
    let session_id = create_session(&client, &http_base, &token).await;

    let (ws, response) = try_connect_ws(&ws_base, session_id, Some(&token)).await.unwrap();
    assert_eq!(response.status().as_u16(), 101);
    drop(ws);
}

use std::time::Duration as StdDuration;

use nomi_agent_chitchat::ChitchatAgent;
use nomi_agent_core::AgentRegistry;
use nomi_agent_money::MoneyAgent;
use nomi_llm::{ContentBlock, LlmResponse, PartialBlock, StopReason, StreamEvent};
use nomi_realtime::{MqttPublisher, StreamEnvelope};
use nomi_turn::{process_turn, queue};

use nomi_test_support::{dummy_embedding, FakeEmbeddingProvider, FakeLlmProvider};

fn canned_response(text: &str) -> LlmResponse {
    LlmResponse {
        content: vec![ContentBlock::Text { text: text.to_string() }],
        stop_reason: StopReason::EndTurn,
        input_tokens: 10,
        output_tokens: 5,
    }
}

/// Claims the next pending turn job, resolves its user_id the same way
/// backend/src/bin/worker.rs does, runs it through process_turn with a fake provider, and
/// publishes TurnCompleted afterwards — mirroring the worker's own success path
/// (backend/src/bin/worker.rs:91-104), since process_turn itself only publishes on failure.
async fn run_one_claimed_turn(pool: &PgPool, reply_text: &str) -> Uuid {
    let claimed = queue::claim_next(pool).await.unwrap().unwrap();
    let user_id: Uuid = sqlx::query_scalar("SELECT user_id FROM channel_identities WHERE id = $1")
        .bind(claimed.sender_channel_identity_id)
        .fetch_one(pool)
        .await
        .unwrap();

    let provider = FakeLlmProvider::success(canned_response(reply_text));
    let embedder = FakeEmbeddingProvider::success(dummy_embedding());
    let registry = AgentRegistry::new(vec![Box::new(MoneyAgent), Box::new(ChitchatAgent)]);
    let mqtt = MqttPublisher::connect(
        nomi_test_support::TEST_MQTT_BROKER_HOST,
        nomi_test_support::TEST_MQTT_BROKER_PORT,
        &format!("test-ws-relay-{}", Uuid::new_v4()),
    );

    process_turn(
        pool,
        &mqtt,
        &provider,
        &embedder,
        &registry,
        claimed.id,
        claimed.session_id,
        claimed.sender_channel_identity_id,
        user_id,
        &claimed.text,
    )
    .await
    .unwrap();

    mqtt.publish(
        claimed.session_id,
        &StreamEnvelope::TurnCompleted { turn_job_id: claimed.id, message_id: Uuid::nil() },
    )
    .await
    .unwrap();

    claimed.id
}

fn expected_frames_for(turn_job_id: Uuid, reply_text: &str) -> Vec<StreamEnvelope> {
    vec![
        StreamEnvelope::Delta {
            turn_job_id,
            event: StreamEvent::ContentBlockStart { index: 0, block: PartialBlock::Text },
        },
        StreamEnvelope::Delta {
            turn_job_id,
            event: StreamEvent::TextDelta { index: 0, text: reply_text.to_string() },
        },
        StreamEnvelope::Delta { turn_job_id, event: StreamEvent::ContentBlockDone { index: 0 } },
        StreamEnvelope::Delta {
            turn_job_id,
            event: StreamEvent::Done { stop_reason: StopReason::EndTurn, input_tokens: 10, output_tokens: 5 },
        },
        StreamEnvelope::TurnCompleted { turn_job_id, message_id: Uuid::nil() },
    ]
}

async fn recv_n_frames(
    ws: &mut tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
    n: usize,
) -> Vec<StreamEnvelope> {
    let mut received = Vec::new();
    for _ in 0..n {
        let msg = tokio::time::timeout(StdDuration::from_secs(5), ws.next())
            .await
            .expect("timed out waiting for a frame")
            .expect("stream ended")
            .unwrap();
        if let tokio_tungstenite::tungstenite::Message::Text(text) = msg {
            received.push(serde_json::from_str::<StreamEnvelope>(&text).unwrap());
        }
    }
    received
}

#[sqlx::test(migrations = "../../migrations")]
async fn relays_delta_and_turn_completed_events_for_a_single_turn(pool: PgPool) {
    let (http_base, ws_base) = spawn_app(pool.clone()).await;
    let client = reqwest::Client::new();
    let token = register_and_login(&client, &http_base, "d@example.com").await;
    let session_id = create_session(&client, &http_base, &token).await;

    let (mut ws, _) = try_connect_ws(&ws_base, session_id, Some(&token)).await.unwrap();

    client
        .post(format!("{http_base}/api/sessions/{session_id}/messages"))
        .bearer_auth(&token)
        .json(&json!({ "text": "hello" }))
        .send()
        .await
        .unwrap();

    let turn_job_id = run_one_claimed_turn(&pool, "hi there").await;

    let received = recv_n_frames(&mut ws, 5).await;
    assert_eq!(received, expected_frames_for(turn_job_id, "hi there"));
}

#[sqlx::test(migrations = "../../migrations")]
async fn relays_events_across_two_turns_without_reconnecting(pool: PgPool) {
    let (http_base, ws_base) = spawn_app(pool.clone()).await;
    let client = reqwest::Client::new();
    let token = register_and_login(&client, &http_base, "e@example.com").await;
    let session_id = create_session(&client, &http_base, &token).await;

    let (mut ws, _) = try_connect_ws(&ws_base, session_id, Some(&token)).await.unwrap();

    client
        .post(format!("{http_base}/api/sessions/{session_id}/messages"))
        .bearer_auth(&token)
        .json(&json!({ "text": "first" }))
        .send()
        .await
        .unwrap();
    let first_turn_job_id = run_one_claimed_turn(&pool, "first reply").await;
    let first_received = recv_n_frames(&mut ws, 5).await;
    assert_eq!(first_received, expected_frames_for(first_turn_job_id, "first reply"));

    client
        .post(format!("{http_base}/api/sessions/{session_id}/messages"))
        .bearer_auth(&token)
        .json(&json!({ "text": "second" }))
        .send()
        .await
        .unwrap();
    let second_turn_job_id = run_one_claimed_turn(&pool, "second reply").await;
    let second_received = recv_n_frames(&mut ws, 5).await;
    assert_eq!(second_received, expected_frames_for(second_turn_job_id, "second reply"));

    assert_ne!(first_turn_job_id, second_turn_job_id);
}
