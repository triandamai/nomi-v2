mod support;

use futures_util::StreamExt;
use serde_json::{json, Value};
use sqlx::PgPool;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use uuid::Uuid;

use nomi_orchestrator::app::{build_router, AppState};

const SECRET: &str = "test-secret-do-not-use-in-prod";

fn test_state(pool: PgPool) -> AppState {
    AppState {
        pool,
        jwt_secret: SECRET.to_string(),
        http_client: reqwest::Client::new(),
        settings_key: support::TEST_SETTINGS_KEY,
        mqtt_broker_host: support::TEST_MQTT_BROKER_HOST.to_string(),
        mqtt_broker_port: support::TEST_MQTT_BROKER_PORT,
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

#[sqlx::test]
async fn ws_upgrade_without_a_token_is_rejected(pool: PgPool) {
    let (_, ws_base) = spawn_app(pool).await;
    let err = try_connect_ws(&ws_base, Uuid::new_v4(), None).await.unwrap_err();
    assert_rejected_with(err, 401);
}

#[sqlx::test]
async fn ws_upgrade_for_another_orgs_session_is_rejected(pool: PgPool) {
    let (http_base, ws_base) = spawn_app(pool).await;
    let client = reqwest::Client::new();
    let token_a = register_and_login(&client, &http_base, "a@example.com").await;
    let token_b = register_and_login(&client, &http_base, "b@example.com").await;
    let session_id = create_session(&client, &http_base, &token_a).await;

    let err = try_connect_ws(&ws_base, session_id, Some(&token_b)).await.unwrap_err();
    assert_rejected_with(err, 404);
}

#[sqlx::test]
async fn ws_upgrade_with_a_valid_token_for_the_callers_own_session_succeeds(pool: PgPool) {
    let (http_base, ws_base) = spawn_app(pool).await;
    let client = reqwest::Client::new();
    let token = register_and_login(&client, &http_base, "c@example.com").await;
    let session_id = create_session(&client, &http_base, &token).await;

    let (ws, response) = try_connect_ws(&ws_base, session_id, Some(&token)).await.unwrap();
    assert_eq!(response.status().as_u16(), 101);
    drop(ws);
}
