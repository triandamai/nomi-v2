//! Admin → Integrations → Relay: how Nomi reaches Relay (kept encrypted here) and Relay's own
//! settings (kept by Relay, passed through), behind the `integrations` access levels.
use axum::{body::Body, http::{Request, StatusCode}};
use http_body_util::BodyExt;
use nomi_server::app::{build_router, AppState};
use serde_json::{json, Value};
use sqlx::PgPool;
use tower::ServiceExt;
use wiremock::matchers::{body_json, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

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

async fn request(router: axum::Router, method: &str, uri: &str, token: &str, body: Option<Value>) -> (StatusCode, Value) {
    let mut builder = Request::builder().method(method).uri(uri).header("authorization", format!("Bearer {token}"));
    let body = match body {
        Some(b) => {
            builder = builder.header("content-type", "application/json");
            Body::from(b.to_string())
        }
        None => Body::empty(),
    };
    let response = router.oneshot(builder.body(body).unwrap()).await.unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    (status, if bytes.is_empty() { Value::Null } else { serde_json::from_slice(&bytes).unwrap_or(Value::Null) })
}

async fn login(router: axum::Router, pool: &PgPool, email: &str, admin: bool) -> String {
    let register = Request::builder()
        .method("POST")
        .uri("/api/auth/register")
        .header("content-type", "application/json")
        .body(Body::from(json!({ "email": email, "password": "correct-password", "org": { "mode": "create", "name": "Acme" } }).to_string()))
        .unwrap();
    router.clone().oneshot(register).await.unwrap();
    if admin {
        sqlx::query("UPDATE users SET is_platform_admin = true WHERE id = (SELECT user_id FROM web_credentials WHERE email = $1)")
            .bind(email)
            .execute(pool)
            .await
            .unwrap();
    }
    let login = Request::builder()
        .method("POST")
        .uri("/api/auth/login")
        .header("content-type", "application/json")
        .body(Body::from(json!({ "email": email, "password": "correct-password" }).to_string()))
        .unwrap();
    let response = router.oneshot(login).await.unwrap();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body: Value = serde_json::from_slice(&bytes).unwrap();
    body["access_token"].as_str().unwrap().to_string()
}

fn envelope(data: Value) -> ResponseTemplate {
    ResponseTemplate::new(200).set_body_json(json!({ "data": data, "error": null }))
}

fn relay_settings() -> Value {
    json!({ "configured": true, "public_domain": "api.relay.test", "shared_dev_url": "postgres://relay@db/dev", "shared_production_url": "postgres://relay@db/prod",
            "postgrest_dev_url": "http://pgrst-dev", "postgrest_production_url": "http://pgrst-prod", "postgrest_role": "authenticator",
            "preview_origins": [], "checks": [{ "name": "dev database", "ok": true }] })
}

#[sqlx::test(migrations = "../../migrations")]
async fn only_integration_admins_reach_relay_settings(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = login(router.clone(), &pool, "member@relay.test", false).await;
    assert_eq!(request(router.clone(), "GET", "/api/admin/relay", &token, None).await.0, StatusCode::FORBIDDEN);
    let body = json!({ "control_url": "https://relay.example", "control_key": "k" });
    assert_eq!(request(router.clone(), "PUT", "/api/admin/relay/connection", &token, Some(body)).await.0, StatusCode::FORBIDDEN);
    assert_eq!(request(router, "PUT", "/api/admin/relay/settings", &token, Some(json!({}))).await.0, StatusCode::FORBIDDEN);
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_saved_connection_shows_relays_settings_and_never_its_key(pool: PgPool) {
    let relay = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/control/settings"))
        .and(header("authorization", "Bearer relay-control-key"))
        .respond_with(envelope(relay_settings()))
        .mount(&relay)
        .await;
    let router = build_router(test_state(pool.clone()));
    let token = login(router.clone(), &pool, "admin@relay.test", true).await;

    let (status, before) = request(router.clone(), "GET", "/api/admin/relay", &token, None).await;
    assert_eq!(status, StatusCode::OK, "{before}");
    assert!(before["relay"].is_null());
    assert!(before["default_preview_origins"].as_array().is_some_and(|o| !o.is_empty()));

    let body = json!({ "control_url": relay.uri(), "control_key": "relay-control-key" });
    let (status, saved) = request(router.clone(), "PUT", "/api/admin/relay/connection", &token, Some(body)).await;
    assert_eq!(status, StatusCode::OK, "{saved}");

    let (_, after) = request(router, "GET", "/api/admin/relay", &token, None).await;
    assert_eq!(after["connection"]["source"], "admin");
    assert_eq!(after["connection"]["key_set"], true);
    assert_eq!(after["relay"]["public_domain"], "api.relay.test");
    assert!(!after.to_string().contains("relay-control-key"));
}

#[sqlx::test(migrations = "../../migrations")]
async fn relays_settings_are_passed_on_and_its_refusal_names_the_checks(pool: PgPool) {
    let relay = MockServer::start().await;
    let good = json!({ "public_domain": "api.relay.test", "postgrest_dev_url": "http://pgrst-dev", "postgrest_production_url": "http://pgrst-prod" });
    Mock::given(method("PUT")).and(path("/control/settings")).and(body_json(good.clone())).respond_with(envelope(relay_settings())).mount(&relay).await;
    Mock::given(method("PUT"))
        .and(path("/control/settings"))
        .respond_with(ResponseTemplate::new(422).set_body_json(json!({ "data": null, "error": {
            "code": "SETTINGS_INVALID", "message": "These didn't work: dev database",
            "details": { "checks": [{ "name": "dev database", "ok": false, "message": "password authentication failed" }] } } })))
        .mount(&relay)
        .await;
    let router = build_router(test_state(pool.clone()));
    let token = login(router.clone(), &pool, "admin@relay.test", true).await;
    request(router.clone(), "PUT", "/api/admin/relay/connection", &token, Some(json!({ "control_url": relay.uri(), "control_key": "k" }))).await;

    let (status, saved) = request(router.clone(), "PUT", "/api/admin/relay/settings", &token, Some(good)).await;
    assert_eq!(status, StatusCode::OK, "{saved}");
    assert_eq!(saved["configured"], true);

    let bad = json!({ "public_domain": "x", "postgrest_dev_url": "", "postgrest_production_url": "" });
    let (status, refused) = request(router, "PUT", "/api/admin/relay/settings", &token, Some(bad)).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(refused["checks"][0]["message"], "password authentication failed");
    assert!(refused["message"].as_str().unwrap().contains("dev database"));
}
