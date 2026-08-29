
use axum::{body::Body, http::{Request, StatusCode}};
use http_body_util::BodyExt;
use nomi_server::app::{build_router, AppState};
use serde_json::{json, Value};
use sqlx::PgPool;
use tower::ServiceExt;

use nomi_test_support::TEST_SETTINGS_KEY;

const SECRET: &str = "test-secret-do-not-use-in-prod";

fn test_state(pool: PgPool) -> AppState {
    AppState {
        pool,
        jwt_secret: SECRET.to_string(),
        http_client: reqwest::Client::new(),
        settings_key: TEST_SETTINGS_KEY,
        mqtt_broker_host: nomi_test_support::TEST_MQTT_BROKER_HOST.to_string(),
        mqtt_broker_port: nomi_test_support::TEST_MQTT_BROKER_PORT,
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

#[sqlx::test(migrations = "../../migrations")]
async fn non_admin_is_forbidden_from_reading_embedding_settings(pool: PgPool) {
    let router = build_router(test_state(pool));
    register_via_api(router.clone(), "regular-embed@example.com").await;
    let token = login_via_api(router.clone(), "regular-embed@example.com").await;

    let (status, _) =
        json_request(router, "GET", "/api/admin/settings/embedding", Value::Null, Some(&token)).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}

#[sqlx::test(migrations = "../../migrations")]
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

#[sqlx::test(migrations = "../../migrations")]
async fn admin_can_configure_a_non_openai_embedding_provider(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register_admin_and_login(router.clone(), &pool, "admin-embed-gemini@example.com").await;

    let (status, put_body) = json_request(
        router,
        "PUT",
        "/api/admin/settings/embedding",
        json!({ "provider": "gemini", "model_id": "gemini-embedding-001", "api_key": "test-gemini-key1", "base_url": null }),
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(put_body["provider"], "gemini");
}

#[sqlx::test(migrations = "../../migrations")]
async fn unknown_provider_is_rejected(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register_admin_and_login(router.clone(), &pool, "admin-embed-bad@example.com").await;

    let (status, _) = json_request(
        router,
        "PUT",
        "/api/admin/settings/embedding",
        json!({ "provider": "not-a-real-provider", "model_id": "x", "api_key": "test-key12345678", "base_url": null }),
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}
