
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
        tool_catalog: std::sync::Arc::new(nomi_agent_core::ToolCatalog::empty()),
        email_codes: nomi_server::sign_in_codes::EmailCodes::Off,
        push: std::sync::Arc::new(nomi_server::push::NoopPush),
        shipyard: None,
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
    assert_eq!(body["accent_color"], "canopy");
}

#[sqlx::test(migrations = "../../migrations")]
async fn setting_accent_color_does_not_clobber_theme_and_vice_versa(pool: PgPool) {
    let router = build_router(test_state(pool));
    let token = register_and_login(router.clone(), "mira@example.com").await;

    let (status, put_body) = json_request(router.clone(), "PUT", "/api/preferences", json!({ "theme": "dark" }), Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(put_body["theme"], "dark");
    assert_eq!(put_body["accent_color"], "canopy");

    let (status, put_body) =
        json_request(router.clone(), "PUT", "/api/preferences", json!({ "accent_color": "phantom" }), Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(put_body["theme"], "dark");
    assert_eq!(put_body["accent_color"], "phantom");

    let (_, get_body) = json_request(router, "GET", "/api/preferences", Value::Null, Some(&token)).await;
    assert_eq!(get_body["theme"], "dark");
    assert_eq!(get_body["accent_color"], "phantom");
}

#[sqlx::test(migrations = "../../migrations")]
async fn every_new_theme_can_be_saved(pool: PgPool) {
    let router = build_router(test_state(pool));
    let token = register_and_login(router.clone(), "themes@example.com").await;
    for theme in ["lautan", "matcha", "saffron", "lagoon", "orchid", "ruby", "graphite"] {
        let (status, body) = json_request(router.clone(), "PUT", "/api/preferences", json!({ "accent_color": theme }), Some(&token)).await;
        assert_eq!(status, StatusCode::OK, "{theme}");
        assert_eq!(body["accent_color"], theme);
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn setting_an_invalid_accent_color_is_rejected(pool: PgPool) {
    let router = build_router(test_state(pool));
    let token = register_and_login(router.clone(), "nell@example.com").await;

    let (status, _) =
        json_request(router, "PUT", "/api/preferences", json!({ "accent_color": "chartreuse" }), Some(&token)).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
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

#[sqlx::test(migrations = "../../migrations")]
async fn get_preferences_defaults_timezone_to_utc(pool: PgPool) {
    let router = build_router(test_state(pool));
    let token = register_and_login(router.clone(), "omar@example.com").await;

    let (status, body) = json_request(router, "GET", "/api/preferences", Value::Null, Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["timezone"], "UTC");
}

#[sqlx::test(migrations = "../../migrations")]
async fn put_preferences_rejects_an_invalid_timezone_string(pool: PgPool) {
    let router = build_router(test_state(pool));
    let token = register_and_login(router.clone(), "pia@example.com").await;

    let (status, _) =
        json_request(router, "PUT", "/api/preferences", json!({ "timezone": "Not/A_Real_Zone" }), Some(&token)).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[sqlx::test(migrations = "../../migrations")]
async fn put_preferences_saves_a_valid_timezone(pool: PgPool) {
    let router = build_router(test_state(pool));
    let token = register_and_login(router.clone(), "quinn@example.com").await;

    let (status, put_body) =
        json_request(router.clone(), "PUT", "/api/preferences", json!({ "timezone": "America/New_York" }), Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(put_body["timezone"], "America/New_York");

    let (_, get_body) = json_request(router, "GET", "/api/preferences", Value::Null, Some(&token)).await;
    assert_eq!(get_body["timezone"], "America/New_York");
}

#[sqlx::test(migrations = "../../migrations")]
async fn language_defaults_to_english_and_switches_to_indonesian(pool: PgPool) {
    let router = build_router(test_state(pool));
    let token = register_and_login(router.clone(), "sari@example.com").await;

    let (_, body) = json_request(router.clone(), "GET", "/api/preferences", Value::Null, Some(&token)).await;
    assert_eq!(body["language"], "en");

    let (status, body) = json_request(router.clone(), "PUT", "/api/preferences", json!({ "language": "id" }), Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["language"], "id");
    assert_eq!(body["theme"], "system");

    let (status, _) = json_request(router.clone(), "PUT", "/api/preferences", json!({ "language": "fr" }), Some(&token)).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    let (_, body) = json_request(router, "GET", "/api/preferences", Value::Null, Some(&token)).await;
    assert_eq!(body["language"], "id");
}

async fn post_avatar(router: axum::Router, token: &str, content_type: &str, body: Vec<u8>) -> StatusCode {
    let request = axum::http::Request::builder()
        .method("POST")
        .uri("/api/profile/avatar")
        .header("content-type", content_type)
        .header("authorization", format!("Bearer {token}"))
        .body(axum::body::Body::from(body))
        .unwrap();
    tower::ServiceExt::oneshot(router, request).await.unwrap().status()
}

#[sqlx::test(migrations = "../../migrations")]
async fn an_uploaded_avatar_must_be_a_real_small_image(pool: PgPool) {
    let router = build_router(test_state(pool));
    let token = register_and_login(router.clone(), "ivy@example.com").await;
    let png = b"\x89PNG\r\n\x1a\n rest of the picture".to_vec();

    assert_eq!(post_avatar(router.clone(), &token, "text/plain", png.clone()).await, StatusCode::BAD_REQUEST);
    assert_eq!(post_avatar(router.clone(), &token, "image/jpeg", png.clone()).await, StatusCode::BAD_REQUEST);
    assert_eq!(post_avatar(router.clone(), &token, "image/png", Vec::new()).await, StatusCode::BAD_REQUEST);
    let mut huge = png.clone();
    huge.resize(5 * 1024 * 1024 + 10, 0);
    assert_eq!(post_avatar(router.clone(), &token, "image/png", huge).await, StatusCode::PAYLOAD_TOO_LARGE);
}

#[sqlx::test(migrations = "../../migrations")]
async fn an_avatar_is_stored_without_a_bucket_and_served_by_nomi(pool: PgPool) {
    std::env::set_var("ATTACHMENTS_DIR", std::env::temp_dir().join(format!("nomi-test-avatars-{}", std::process::id())));
    let router = build_router(test_state(pool));
    let token = register_and_login(router.clone(), "avatar-round@example.com").await;
    let png = b"\x89PNG\r\n\x1a\n rest of the picture".to_vec();

    let request = axum::http::Request::builder()
        .method("POST")
        .uri("/api/profile/avatar")
        .header("content-type", "image/png")
        .header("authorization", format!("Bearer {token}"))
        .body(axum::body::Body::from(png.clone()))
        .unwrap();
    let response = tower::ServiceExt::oneshot(router.clone(), request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = serde_json::from_slice(&http_body_util::BodyExt::collect(response.into_body()).await.unwrap().to_bytes()).unwrap();
    let url = body["avatar_url"].as_str().unwrap().to_string();
    assert!(url.starts_with("/avatars/") && url.ends_with(".png"), "a path Nomi serves, not a bucket URL: {url}");

    let (_, profile) = json_request(router.clone(), "GET", "/api/profile", Value::Null, Some(&token)).await;
    assert_eq!(profile["avatar_url"], url);

    let get = |bearer: Option<String>, uri: String| {
        let mut request = axum::http::Request::builder().uri(uri);
        if let Some(bearer) = bearer {
            request = request.header("authorization", format!("Bearer {bearer}"));
        }
        tower::ServiceExt::oneshot(router.clone(), request.body(axum::body::Body::empty()).unwrap())
    };
    let response = get(Some(token.clone()), format!("/api{url}")).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()["content-type"], "image/png");
    let bytes = http_body_util::BodyExt::collect(response.into_body()).await.unwrap().to_bytes();
    assert_eq!(bytes.to_vec(), png);

    assert_eq!(get(None, format!("/api{url}")).await.unwrap().status(), StatusCode::UNAUTHORIZED);
    let user = url.split('/').nth(2).unwrap();
    assert_eq!(get(Some(token.clone()), format!("/api/avatars/{user}/..%2F..%2Fsecret.png")).await.unwrap().status(), StatusCode::NOT_FOUND);
    assert_eq!(get(Some(token), format!("/api/avatars/{user}/{}.png", uuid::Uuid::new_v4())).await.unwrap().status(), StatusCode::NOT_FOUND);
}

#[sqlx::test(migrations = "../../migrations")]
async fn drawer_pins_are_unset_until_saved_then_keep_their_order(pool: PgPool) {
    let router = build_router(test_state(pool));
    let token = register_and_login(router.clone(), "pia@example.com").await;

    let (_, body) = json_request(router.clone(), "GET", "/api/preferences", Value::Null, Some(&token)).await;
    assert!(body["drawer_pins"].is_null());

    let pins = json!({ "drawer_pins": ["reminders", "money", "crew"] });
    let (status, body) = json_request(router.clone(), "PUT", "/api/preferences", pins, Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["drawer_pins"], json!(["reminders", "money", "crew"]));

    // Other preference saves leave the pins alone.
    let (_, body) = json_request(router.clone(), "PUT", "/api/preferences", json!({ "theme": "dark" }), Some(&token)).await;
    assert_eq!(body["drawer_pins"], json!(["reminders", "money", "crew"]));

    // Nothing pinned is a valid choice.
    let (_, body) = json_request(router, "PUT", "/api/preferences", json!({ "drawer_pins": [] }), Some(&token)).await;
    assert_eq!(body["drawer_pins"], json!([]));
}

#[sqlx::test(migrations = "../../migrations")]
async fn drawer_pins_reject_unknown_and_repeated_features(pool: PgPool) {
    let router = build_router(test_state(pool));
    let token = register_and_login(router.clone(), "rio@example.com").await;

    for pins in [json!(["money", "admin"]), json!(["home"]), json!(["money", "money"])] {
        let (status, _) = json_request(router.clone(), "PUT", "/api/preferences", json!({ "drawer_pins": pins }), Some(&token)).await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{pins}");
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn home_layout_is_unset_until_picked_and_only_takes_a_slug(pool: PgPool) {
    let router = build_router(test_state(pool));
    let token = register_and_login(router.clone(), "hana@example.com").await;

    let (_, body) = json_request(router.clone(), "GET", "/api/preferences", Value::Null, Some(&token)).await;
    assert!(body["home_layout"].is_null());

    let (status, body) = json_request(router.clone(), "PUT", "/api/preferences", json!({ "home_layout": "stage" }), Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["home_layout"], "stage");

    // Other preference saves leave the layout alone.
    let (_, body) = json_request(router.clone(), "PUT", "/api/preferences", json!({ "theme": "dark" }), Some(&token)).await;
    assert_eq!(body["home_layout"], "stage");

    for layout in ["", "Stage", "../home", "9lives", &"a".repeat(33)] {
        let (status, _) = json_request(router.clone(), "PUT", "/api/preferences", json!({ "home_layout": layout }), Some(&token)).await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{layout}");
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn animated_backdrop_is_on_until_turned_off(pool: PgPool) {
    let router = build_router(test_state(pool));
    let token = register_and_login(router.clone(), "bea@example.com").await;

    let (_, body) = json_request(router.clone(), "GET", "/api/preferences", Value::Null, Some(&token)).await;
    assert_eq!(body["animated_backdrop"], true);

    let (_, body) = json_request(router.clone(), "PUT", "/api/preferences", json!({ "animated_backdrop": false }), Some(&token)).await;
    assert_eq!(body["animated_backdrop"], false);

    // Other preference saves leave it alone.
    let (_, body) = json_request(router.clone(), "PUT", "/api/preferences", json!({ "theme": "dark" }), Some(&token)).await;
    assert_eq!(body["animated_backdrop"], false);
    let (_, body) = json_request(router, "GET", "/api/preferences", Value::Null, Some(&token)).await;
    assert_eq!(body["animated_backdrop"], false);
}
