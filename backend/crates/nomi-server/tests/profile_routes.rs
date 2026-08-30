
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
    let (_, login_body) =
        json_request(router, "POST", "/api/auth/login", json!({ "email": email, "password": "correct-password" }), None).await;
    login_body["access_token"].as_str().unwrap().to_string()
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_fresh_users_profile_has_no_display_name_or_username(pool: PgPool) {
    let router = build_router(test_state(pool));
    let token = register_and_login(router.clone(), "amara@example.com").await;

    let (status, body) = json_request(router, "GET", "/api/profile", Value::Null, Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["email"], "amara@example.com");
    assert!(body["display_name"].is_null());
    assert!(body["username"].is_null());
    assert!(body["avatar_url"].is_null());
}

#[sqlx::test(migrations = "../../migrations")]
async fn getting_profile_without_a_token_is_unauthorized(pool: PgPool) {
    let router = build_router(test_state(pool));
    let (status, _) = json_request(router, "GET", "/api/profile", Value::Null, None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[sqlx::test(migrations = "../../migrations")]
async fn updating_profile_then_reading_it_back_reflects_the_change(pool: PgPool) {
    let router = build_router(test_state(pool));
    let token = register_and_login(router.clone(), "bo@example.com").await;

    let (status, put_body) = json_request(
        router.clone(),
        "PUT",
        "/api/profile",
        json!({ "display_name": "Bo", "username": "bo_the_builder", "avatar_url": null }),
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(put_body["display_name"], "Bo");
    assert_eq!(put_body["username"], "bo_the_builder");

    let (_, get_body) = json_request(router, "GET", "/api/profile", Value::Null, Some(&token)).await;
    assert_eq!(get_body["display_name"], "Bo");
    assert_eq!(get_body["username"], "bo_the_builder");
}

#[sqlx::test(migrations = "../../migrations")]
async fn updating_only_the_display_name_leaves_a_previously_set_username_untouched(pool: PgPool) {
    let router = build_router(test_state(pool));
    let token = register_and_login(router.clone(), "cara@example.com").await;

    json_request(router.clone(), "PUT", "/api/profile", json!({ "username": "cara" }), Some(&token)).await;
    let (_, put_body) = json_request(router, "PUT", "/api/profile", json!({ "display_name": "Cara" }), Some(&token)).await;

    assert_eq!(put_body["display_name"], "Cara");
    assert_eq!(put_body["username"], "cara");
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_username_already_taken_by_someone_else_is_rejected(pool: PgPool) {
    let router = build_router(test_state(pool));
    let token_a = register_and_login(router.clone(), "dan@example.com").await;
    let token_b = register_and_login(router.clone(), "eve@example.com").await;

    json_request(router.clone(), "PUT", "/api/profile", json!({ "username": "taken" }), Some(&token_a)).await;
    let (status, _) = json_request(router, "PUT", "/api/profile", json!({ "username": "taken" }), Some(&token_b)).await;
    assert_eq!(status, StatusCode::CONFLICT);
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_username_with_invalid_characters_is_rejected(pool: PgPool) {
    let router = build_router(test_state(pool));
    let token = register_and_login(router.clone(), "frank@example.com").await;

    let (status, _) = json_request(router, "PUT", "/api/profile", json!({ "username": "not valid!" }), Some(&token)).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[sqlx::test(migrations = "../../migrations")]
async fn requesting_an_avatar_upload_url_without_s3_configured_is_service_unavailable(pool: PgPool) {
    let router = build_router(test_state(pool));
    let token = register_and_login(router.clone(), "gina@example.com").await;

    let (status, _) =
        json_request(router, "POST", "/api/profile/avatar/upload-url", json!({ "content_type": "image/png" }), Some(&token)).await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
}

#[sqlx::test(migrations = "../../migrations")]
async fn preferences_default_to_system_theme(pool: PgPool) {
    let router = build_router(test_state(pool));
    let token = register_and_login(router.clone(), "hana@example.com").await;

    let (status, body) = json_request(router, "GET", "/api/preferences", Value::Null, Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["theme"], "system");
}

#[sqlx::test(migrations = "../../migrations")]
async fn setting_theme_then_reading_preferences_reflects_it(pool: PgPool) {
    let router = build_router(test_state(pool));
    let token = register_and_login(router.clone(), "ivo@example.com").await;

    let (status, put_body) = json_request(router.clone(), "PUT", "/api/preferences", json!({ "theme": "dark" }), Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(put_body["theme"], "dark");

    let (_, get_body) = json_request(router, "GET", "/api/preferences", Value::Null, Some(&token)).await;
    assert_eq!(get_body["theme"], "dark");
}

#[sqlx::test(migrations = "../../migrations")]
async fn setting_an_invalid_theme_is_rejected(pool: PgPool) {
    let router = build_router(test_state(pool));
    let token = register_and_login(router.clone(), "jill@example.com").await;

    let (status, _) = json_request(router, "PUT", "/api/preferences", json!({ "theme": "rainbow" }), Some(&token)).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[sqlx::test(migrations = "../../migrations")]
async fn preferences_are_isolated_per_user(pool: PgPool) {
    let router = build_router(test_state(pool));
    let token_a = register_and_login(router.clone(), "ken@example.com").await;
    let token_b = register_and_login(router.clone(), "liv@example.com").await;

    json_request(router.clone(), "PUT", "/api/preferences", json!({ "theme": "dark" }), Some(&token_a)).await;
    let (_, body_b) = json_request(router, "GET", "/api/preferences", Value::Null, Some(&token_b)).await;
    assert_eq!(body_b["theme"], "system");
}
