mod support;

use axum::{body::Body, http::{Request, StatusCode}};
use http_body_util::BodyExt;
use nomi_orchestrator::app::{build_router, AppState};
use serde_json::{json, Value};
use sqlx::PgPool;
use tower::ServiceExt;

use support::TEST_SETTINGS_KEY;

const SECRET: &str = "test-secret-do-not-use-in-prod";

fn test_state(pool: PgPool) -> AppState {
    AppState {
        pool,
        jwt_secret: SECRET.to_string(),
        http_client: reqwest::Client::new(),
        settings_key: TEST_SETTINGS_KEY,
        mqtt_broker_host: support::TEST_MQTT_BROKER_HOST.to_string(),
        mqtt_broker_port: support::TEST_MQTT_BROKER_PORT,
    }
}

async fn json_request(
    router: axum::Router,
    method: &str,
    uri: &str,
    body: Value,
    bearer: Option<&str>,
) -> (StatusCode, Value) {
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

async fn register_via_api(router: axum::Router, email: &str) {
    json_request(
        router,
        "POST",
        "/api/auth/register",
        json!({ "email": email, "password": "correct-password", "org": { "mode": "create", "name": "Acme" } }),
        None,
    )
    .await;
}

async fn login_via_api(router: axum::Router, email: &str) -> String {
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

async fn make_platform_admin(pool: &PgPool, email: &str) {
    sqlx::query("UPDATE users SET is_platform_admin = true WHERE id = (SELECT user_id FROM web_credentials WHERE email = $1)")
        .bind(email)
        .execute(pool)
        .await
        .unwrap();
}

/// Registers, promotes to platform admin, then logs in — login must happen
/// *after* promotion, since permissions are baked into the JWT at login time.
async fn register_admin_and_login(router: axum::Router, pool: &PgPool, email: &str) -> String {
    register_via_api(router.clone(), email).await;
    make_platform_admin(pool, email).await;
    login_via_api(router, email).await
}

#[sqlx::test]
async fn get_llm_settings_returns_404_when_unconfigured(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register_admin_and_login(router.clone(), &pool, "admin1@example.com").await;

    let (status, _) = json_request(router, "GET", "/api/admin/settings/llm", Value::Null, Some(&token)).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[sqlx::test]
async fn non_admin_is_forbidden_from_reading_settings(pool: PgPool) {
    let router = build_router(test_state(pool));
    register_via_api(router.clone(), "regular@example.com").await;
    let token = login_via_api(router.clone(), "regular@example.com").await;

    let (status, _) = json_request(router, "GET", "/api/admin/settings/llm", Value::Null, Some(&token)).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}

#[sqlx::test]
async fn admin_can_save_and_then_read_back_masked_llm_settings(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register_admin_and_login(router.clone(), &pool, "admin2@example.com").await;

    let (status, put_body) = json_request(
        router.clone(),
        "PUT",
        "/api/admin/settings/llm",
        json!({ "provider": "anthropic", "model_id": "claude-haiku-4-5", "api_key": "sk-abcdefgh1234", "base_url": null }),
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(put_body["api_key_masked"], "...1234");

    let (status, get_body) = json_request(router, "GET", "/api/admin/settings/llm", Value::Null, Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(get_body["provider"], "anthropic");
    assert_eq!(get_body["model_id"], "claude-haiku-4-5");
    assert_eq!(get_body["api_key_masked"], "...1234");
}

#[sqlx::test]
async fn put_rejects_an_unknown_provider(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register_admin_and_login(router.clone(), &pool, "admin3@example.com").await;

    let (status, _) = json_request(
        router,
        "PUT",
        "/api/admin/settings/llm",
        json!({ "provider": "not-a-real-provider", "model_id": "x", "api_key": "key", "base_url": null }),
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[sqlx::test]
async fn put_without_api_key_keeps_the_existing_key(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register_admin_and_login(router.clone(), &pool, "admin4@example.com").await;

    json_request(
        router.clone(),
        "PUT",
        "/api/admin/settings/llm",
        json!({ "provider": "anthropic", "model_id": "claude-haiku-4-5", "api_key": "sk-original-key9", "base_url": null }),
        Some(&token),
    )
    .await;

    let (status, put_body) = json_request(
        router.clone(),
        "PUT",
        "/api/admin/settings/llm",
        json!({ "provider": "anthropic", "model_id": "claude-sonnet-5", "api_key": null, "base_url": null }),
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(put_body["model_id"], "claude-sonnet-5");
    assert_eq!(put_body["api_key_masked"], "...key9");
}

#[sqlx::test]
async fn non_admin_is_forbidden_from_reading_embedding_settings(pool: PgPool) {
    let router = build_router(test_state(pool));
    register_via_api(router.clone(), "regular-embed@example.com").await;
    let token = login_via_api(router.clone(), "regular-embed@example.com").await;

    let (status, _) =
        json_request(router, "GET", "/api/admin/settings/embedding", Value::Null, Some(&token)).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}

#[sqlx::test]
async fn admin_can_save_and_then_read_back_masked_embedding_settings(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register_admin_and_login(router.clone(), &pool, "admin-embed@example.com").await;

    let (status, put_body) = json_request(
        router.clone(),
        "PUT",
        "/api/admin/settings/embedding",
        json!({ "provider": "openai", "model_id": "text-embedding-3-small", "api_key": "sk-embed-test1234", "base_url": null }),
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(put_body["api_key_masked"], "...1234");

    let (status, get_body) =
        json_request(router, "GET", "/api/admin/settings/embedding", Value::Null, Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(get_body["provider"], "openai");
    assert_eq!(get_body["model_id"], "text-embedding-3-small");
    assert_eq!(get_body["api_key_masked"], "...1234");
}
