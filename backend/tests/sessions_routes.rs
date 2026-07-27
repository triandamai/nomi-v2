mod support;

use axum::{body::Body, http::{Request, StatusCode}};
use http_body_util::BodyExt;
use nomi_orchestrator::app::{build_router, AppState};
use nomi_orchestrator::llm::{LlmResponse, StopReason};
use serde_json::{json, Value};
use sqlx::PgPool;
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

use support::{dummy_embedding, FakeEmbeddingProvider, FakeLlmProvider};

const SECRET: &str = "test-secret-do-not-use-in-prod";

fn inert_provider() -> FakeLlmProvider {
    FakeLlmProvider::success(LlmResponse {
        content: vec![],
        stop_reason: StopReason::EndTurn,
        input_tokens: 0,
        output_tokens: 0,
    })
}

fn test_state(pool: PgPool, provider: FakeLlmProvider) -> AppState {
    AppState {
        pool,
        jwt_secret: SECRET.to_string(),
        provider: Arc::new(provider),
        embedding_provider: Arc::new(FakeEmbeddingProvider::success(dummy_embedding())),
    }
}

async fn json_request(
    router: axum::Router,
    method: &str,
    uri: &str,
    body: Value,
    bearer: Option<&str>,
) -> (StatusCode, Value) {
    let mut builder = Request::builder()
        .method(method)
        .uri(uri)
        .header("content-type", "application/json");
    if let Some(token) = bearer {
        builder = builder.header("authorization", format!("Bearer {token}"));
    }
    let response = router
        .oneshot(builder.body(Body::from(body.to_string())).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let json_body = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).unwrap_or(Value::Null)
    };
    (status, json_body)
}

async fn register_and_login(router: axum::Router, email: &str) -> String {
    json_request(
        router.clone(),
        "POST",
        "/api/auth/register",
        json!({ "email": email, "password": "correct-password", "org": { "mode": "create", "name": "Acme" } }),
        None,
    )
    .await;
    let (_, login_body) = json_request(
        router,
        "POST",
        "/api/auth/login",
        json!({ "email": email, "password": "correct-password" }),
        None,
    )
    .await;
    login_body["access_token"].as_str().unwrap().to_string()
}

#[sqlx::test]
async fn create_session_then_appears_in_list(pool: PgPool) {
    let router = build_router(test_state(pool, inert_provider()));
    let token = register_and_login(router.clone(), "alice@example.com").await;

    let (status, create_body) = json_request(router.clone(), "POST", "/api/sessions", Value::Null, Some(&token)).await;
    assert_eq!(status, StatusCode::CREATED);
    let session_id = create_body["session_id"].as_str().unwrap().to_string();

    let (status, list_body) = json_request(router, "GET", "/api/sessions", Value::Null, Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    let sessions = list_body["sessions"].as_array().unwrap();
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0]["id"].as_str().unwrap(), session_id);
    assert_eq!(sessions[0]["channel"], "web");
    assert!(sessions[0]["last_message"].is_null());
    assert_eq!(sessions[0]["agent_active"], false);
}

#[sqlx::test]
async fn creating_two_sessions_reuses_the_same_web_identity(pool: PgPool) {
    let router = build_router(test_state(pool.clone(), inert_provider()));
    let token = register_and_login(router.clone(), "bob@example.com").await;

    let (_, first) = json_request(router.clone(), "POST", "/api/sessions", Value::Null, Some(&token)).await;
    let (_, second) = json_request(router.clone(), "POST", "/api/sessions", Value::Null, Some(&token)).await;
    assert_ne!(first["session_id"], second["session_id"]);

    let identity_count: i64 = sqlx::query_scalar("SELECT count(*) FROM channel_identities WHERE channel = 'web'")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(identity_count, 1);

    let (_, list_body) = json_request(router, "GET", "/api/sessions", Value::Null, Some(&token)).await;
    assert_eq!(list_body["sessions"].as_array().unwrap().len(), 2);
}

#[sqlx::test]
async fn list_sessions_only_returns_the_callers_active_org(pool: PgPool) {
    let router = build_router(test_state(pool, inert_provider()));
    let token_a = register_and_login(router.clone(), "carol@example.com").await;
    let token_b = register_and_login(router.clone(), "dave@example.com").await;

    json_request(router.clone(), "POST", "/api/sessions", Value::Null, Some(&token_a)).await;
    json_request(router.clone(), "POST", "/api/sessions", Value::Null, Some(&token_b)).await;

    let (_, list_a) = json_request(router.clone(), "GET", "/api/sessions", Value::Null, Some(&token_a)).await;
    assert_eq!(list_a["sessions"].as_array().unwrap().len(), 1);

    let (_, list_b) = json_request(router, "GET", "/api/sessions", Value::Null, Some(&token_b)).await;
    assert_eq!(list_b["sessions"].as_array().unwrap().len(), 1);
}

#[sqlx::test]
async fn create_session_requires_authentication(pool: PgPool) {
    let router = build_router(test_state(pool, inert_provider()));
    let (status, _) = json_request(router, "POST", "/api/sessions", Value::Null, None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}
