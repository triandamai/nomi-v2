
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

async fn register_admin_and_login(router: axum::Router, pool: &PgPool, email: &str) -> String {
    register_via_api(router.clone(), email).await;
    make_platform_admin(pool, email).await;
    login_via_api(router, email).await
}

#[sqlx::test(migrations = "../../migrations")]
async fn non_admin_is_forbidden_from_listing_models(pool: PgPool) {
    let router = build_router(test_state(pool));
    register_via_api(router.clone(), "regular@example.com").await;
    let token = login_via_api(router.clone(), "regular@example.com").await;

    let (status, _) = json_request(router, "GET", "/api/admin/settings/llm/models", Value::Null, Some(&token)).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}

#[sqlx::test(migrations = "../../migrations")]
async fn admin_can_create_then_list_a_model(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register_admin_and_login(router.clone(), &pool, "admin1@example.com").await;

    let (status, create_body) = json_request(
        router.clone(),
        "POST",
        "/api/admin/settings/llm/models",
        json!({ "label": "Claude Sonnet", "provider": "anthropic", "model_id": "claude-sonnet-5", "api_key": "sk-abcdefgh1234", "base_url": null }),
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(create_body["label"], "Claude Sonnet");
    assert_eq!(create_body["is_default"], true);
    assert_eq!(create_body["api_key_masked"], "...1234");

    let (status, list_body) = json_request(router, "GET", "/api/admin/settings/llm/models", Value::Null, Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(list_body.as_array().unwrap().len(), 1);
}

#[sqlx::test(migrations = "../../migrations")]
async fn create_rejects_an_unknown_provider(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register_admin_and_login(router.clone(), &pool, "admin2@example.com").await;

    let (status, _) = json_request(
        router,
        "POST",
        "/api/admin/settings/llm/models",
        json!({ "label": "X", "provider": "not-real", "model_id": "x", "api_key": "key", "base_url": null }),
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[sqlx::test(migrations = "../../migrations")]
async fn update_keeps_the_existing_key_when_none_given(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register_admin_and_login(router.clone(), &pool, "admin3@example.com").await;

    let (_, create_body) = json_request(
        router.clone(),
        "POST",
        "/api/admin/settings/llm/models",
        json!({ "label": "First", "provider": "anthropic", "model_id": "claude-haiku-4-5", "api_key": "sk-original-key9", "base_url": null }),
        Some(&token),
    )
    .await;
    let id = create_body["id"].as_str().unwrap();

    let (status, update_body) = json_request(
        router,
        "PUT",
        &format!("/api/admin/settings/llm/models/{id}"),
        json!({ "label": "Renamed", "provider": "anthropic", "model_id": "claude-sonnet-5", "api_key": null, "base_url": null }),
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(update_body["label"], "Renamed");
    assert_eq!(update_body["api_key_masked"], "...key9");
}

#[sqlx::test(migrations = "../../migrations")]
async fn delete_rejects_the_last_remaining_model(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register_admin_and_login(router.clone(), &pool, "admin4@example.com").await;

    let (_, create_body) = json_request(
        router.clone(),
        "POST",
        "/api/admin/settings/llm/models",
        json!({ "label": "Only", "provider": "anthropic", "model_id": "claude-haiku-4-5", "api_key": "sk-key", "base_url": null }),
        Some(&token),
    )
    .await;
    let id = create_body["id"].as_str().unwrap();

    let (status, _) = json_request(router, "DELETE", &format!("/api/admin/settings/llm/models/{id}"), Value::Null, Some(&token)).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[sqlx::test(migrations = "../../migrations")]
async fn set_default_then_delete_the_old_default_succeeds(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register_admin_and_login(router.clone(), &pool, "admin5@example.com").await;

    let (_, first) = json_request(
        router.clone(),
        "POST",
        "/api/admin/settings/llm/models",
        json!({ "label": "First", "provider": "anthropic", "model_id": "claude-haiku-4-5", "api_key": "sk-key1", "base_url": null }),
        Some(&token),
    )
    .await;
    let first_id = first["id"].as_str().unwrap();

    let (_, second) = json_request(
        router.clone(),
        "POST",
        "/api/admin/settings/llm/models",
        json!({ "label": "Second", "provider": "openai", "model_id": "gpt-4o", "api_key": "sk-key2", "base_url": null }),
        Some(&token),
    )
    .await;
    let second_id = second["id"].as_str().unwrap();

    let (status, _) = json_request(
        router.clone(),
        "PUT",
        &format!("/api/admin/settings/llm/models/{second_id}/default"),
        Value::Null,
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (status, _) = json_request(router, "DELETE", &format!("/api/admin/settings/llm/models/{first_id}"), Value::Null, Some(&token)).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
}

#[sqlx::test(migrations = "../../migrations")]
async fn non_admin_is_forbidden_from_fetching_provider_models(pool: PgPool) {
    let router = build_router(test_state(pool));
    register_via_api(router.clone(), "regular-fetch@example.com").await;
    let token = login_via_api(router.clone(), "regular-fetch@example.com").await;

    let (status, _) = json_request(
        router,
        "POST",
        "/api/admin/settings/llm/models/fetch-models",
        json!({ "provider": "fake", "api_key": "any", "base_url": null, "existing_model_id": null }),
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}

#[sqlx::test(migrations = "../../migrations")]
async fn admin_can_fetch_models_for_the_fake_provider(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register_admin_and_login(router.clone(), &pool, "admin-fetch@example.com").await;

    let (status, body) = json_request(
        router,
        "POST",
        "/api/admin/settings/llm/models/fetch-models",
        json!({ "provider": "fake", "api_key": "any", "base_url": null, "existing_model_id": null }),
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let models = body["models"].as_array().unwrap();
    assert_eq!(models.len(), 1);
    assert_eq!(models[0]["id"], "fake-model");
}

#[sqlx::test(migrations = "../../migrations")]
async fn fetching_models_with_a_blank_key_reuses_the_stored_key_for_an_existing_model(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register_admin_and_login(router.clone(), &pool, "admin-fetch-reuse@example.com").await;

    let (_, create_body) = json_request(
        router.clone(),
        "POST",
        "/api/admin/settings/llm/models",
        json!({ "label": "Fake", "provider": "fake", "model_id": "fake-model", "api_key": "irrelevant-for-fake", "base_url": null }),
        Some(&token),
    )
    .await;
    let id = create_body["id"].as_str().unwrap();

    let (status, body) = json_request(
        router,
        "POST",
        "/api/admin/settings/llm/models/fetch-models",
        json!({ "provider": "fake", "api_key": null, "base_url": null, "existing_model_id": id }),
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["models"].as_array().unwrap().len(), 1);
}

#[sqlx::test(migrations = "../../migrations")]
async fn fetching_models_without_a_key_or_existing_model_id_is_rejected(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register_admin_and_login(router.clone(), &pool, "admin-fetch-noargs@example.com").await;

    let (status, _) = json_request(
        router,
        "POST",
        "/api/admin/settings/llm/models/fetch-models",
        json!({ "provider": "fake", "api_key": null, "base_url": null, "existing_model_id": null }),
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}
