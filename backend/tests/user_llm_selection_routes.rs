mod support;

use axum::{body::Body, http::{Request, StatusCode}};
use http_body_util::BodyExt;
use nomi_orchestrator::app::{build_router, AppState};
use serde_json::{json, Value};
use sqlx::PgPool;
use tower::ServiceExt;

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

async fn make_platform_admin(pool: &PgPool, email: &str) {
    sqlx::query("UPDATE users SET is_platform_admin = true WHERE id = (SELECT user_id FROM web_credentials WHERE email = $1)")
        .bind(email)
        .execute(pool)
        .await
        .unwrap();
}

/// Registers, promotes to platform admin, then logs in — login must happen *after* promotion,
/// since permissions are baked into the JWT at login time (same helper as
/// `tests/admin_llm_models_routes.rs` and `tests/settings_routes.rs`).
async fn register_admin_and_login(router: axum::Router, pool: &PgPool, email: &str) -> String {
    json_request(
        router.clone(),
        "POST",
        "/api/auth/register",
        json!({ "email": email, "password": "correct-password", "org": { "mode": "create", "name": "Acme" } }),
        None,
    )
    .await;
    make_platform_admin(pool, email).await;
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
async fn get_models_returns_the_admin_list_and_no_selection_initially(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register_and_login(router.clone(), "user1@example.com").await;

    let (status, body) = json_request(router, "GET", "/api/llm/models", Value::Null, Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["admin_models"].as_array().unwrap().len(), 0);
    assert!(body["selection"].is_null());
}

#[sqlx::test]
async fn a_user_can_select_an_admin_model(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let admin_token = register_admin_and_login(router.clone(), &pool, "admin@example.com").await;

    let (_, create_body) = json_request(
        router.clone(),
        "POST",
        "/api/admin/settings/llm/models",
        json!({ "label": "Claude Sonnet", "provider": "fake", "model_id": "", "api_key": "", "base_url": null }),
        Some(&admin_token),
    )
    .await;
    let model_id = create_body["id"].as_str().unwrap();

    let user_token = register_and_login(router.clone(), "user2@example.com").await;
    let (status, _) = json_request(
        router.clone(),
        "PUT",
        "/api/llm/selection",
        json!({ "kind": "admin", "admin_model_id": model_id }),
        Some(&user_token),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (_, models_body) = json_request(router, "GET", "/api/llm/models", Value::Null, Some(&user_token)).await;
    assert_eq!(models_body["selection"]["kind"], "admin");
    assert_eq!(models_body["selection"]["admin_model_id"], model_id);
}

#[sqlx::test]
async fn selecting_an_unknown_admin_model_returns_404(pool: PgPool) {
    let router = build_router(test_state(pool));
    let token = register_and_login(router.clone(), "user3@example.com").await;

    let (status, _) = json_request(
        router,
        "PUT",
        "/api/llm/selection",
        json!({ "kind": "admin", "admin_model_id": "00000000-0000-0000-0000-000000000000" }),
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[sqlx::test]
async fn a_user_can_save_a_valid_custom_fake_selection(pool: PgPool) {
    let router = build_router(test_state(pool));
    let token = register_and_login(router.clone(), "user4@example.com").await;

    let (status, _) = json_request(
        router.clone(),
        "PUT",
        "/api/llm/selection",
        json!({ "kind": "custom", "label": "My key", "provider": "fake", "model_id": "", "api_key": "", "base_url": null }),
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (_, models_body) = json_request(router, "GET", "/api/llm/models", Value::Null, Some(&token)).await;
    assert_eq!(models_body["selection"]["kind"], "custom");
    assert_eq!(models_body["selection"]["label"], "My key");
}

#[sqlx::test]
async fn custom_selection_rejects_an_unknown_provider(pool: PgPool) {
    let router = build_router(test_state(pool));
    let token = register_and_login(router.clone(), "user5@example.com").await;

    let (status, _) = json_request(
        router,
        "PUT",
        "/api/llm/selection",
        json!({ "kind": "custom", "label": "X", "provider": "not-real", "model_id": "x", "api_key": "key", "base_url": null }),
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}
