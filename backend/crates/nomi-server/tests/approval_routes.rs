use axum::{body::Body, http::{Request, StatusCode}};
use http_body_util::BodyExt;
use nomi_server::app::{build_router, AppState};
use serde_json::{json, Value};
use sqlx::PgPool;
use tower::ServiceExt;

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
    }
}

async fn json_request(router: axum::Router, method: &str, uri: &str, body: Value, bearer: Option<&str>) -> (StatusCode, Value) {
    let mut builder = Request::builder().method(method).uri(uri).header("content-type", "application/json");
    if let Some(token) = bearer {
        builder = builder.header("authorization", format!("Bearer {token}"));
    }
    let response = router.oneshot(builder.body(Body::from(body.to_string())).unwrap()).await.unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let json_body = if bytes.is_empty() { Value::Null } else { serde_json::from_slice(&bytes).unwrap_or(Value::Null) };
    (status, json_body)
}

async fn register_and_login(router: axum::Router, email: &str) -> String {
    json_request(router.clone(), "POST", "/api/auth/register", json!({"email": email, "password": "correct-password", "org": {"mode": "create", "name": "Acme"}}), None).await;
    let (_, login_body) = json_request(router, "POST", "/api/auth/login", json!({"email": email, "password": "correct-password"}), None).await;
    login_body["access_token"].as_str().unwrap().to_string()
}

#[sqlx::test(migrations = "../../migrations")]
async fn resolving_an_approval_that_is_not_pending_returns_409(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register_and_login(router.clone(), "user@example.com").await;

    // Create a session and a plain message with no pending approval state on any agent_sessions row.
    let (_, create_body) = json_request(router.clone(), "POST", "/api/sessions", Value::Null, Some(&token)).await;
    let session_id = create_body["session_id"].as_str().unwrap();
    let message_id: uuid::Uuid = sqlx::query_scalar("INSERT INTO messages (session_id, content) VALUES ($1, 'hi') RETURNING id")
        .bind(uuid::Uuid::parse_str(session_id).unwrap())
        .fetch_one(&pool)
        .await
        .unwrap();

    let (status, _) = json_request(
        router,
        "PUT",
        &format!("/api/sessions/{session_id}/messages/{message_id}/approval"),
        json!({"decision": "approve"}),
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
}

#[sqlx::test(migrations = "../../migrations")]
async fn an_invalid_decision_value_is_rejected(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register_and_login(router.clone(), "user2@example.com").await;

    let (_, create_body) = json_request(router.clone(), "POST", "/api/sessions", Value::Null, Some(&token)).await;
    let session_id = create_body["session_id"].as_str().unwrap();
    let message_id: uuid::Uuid = sqlx::query_scalar("INSERT INTO messages (session_id, content) VALUES ($1, 'hi') RETURNING id")
        .bind(uuid::Uuid::parse_str(session_id).unwrap())
        .fetch_one(&pool)
        .await
        .unwrap();

    let (status, _) = json_request(
        router,
        "PUT",
        &format!("/api/sessions/{session_id}/messages/{message_id}/approval"),
        json!({"decision": "maybe"}),
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}
