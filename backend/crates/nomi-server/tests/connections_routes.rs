//! Connections: the list of connectors, and connecting Hevy with an API key.
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
    }
}

async fn request(router: axum::Router, method: &str, uri: &str, token: Option<&str>, body: Option<Value>) -> (StatusCode, Value) {
    let mut builder = Request::builder().method(method).uri(uri);
    if let Some(token) = token {
        builder = builder.header("authorization", format!("Bearer {token}"));
    }
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

async fn login(router: axum::Router, email: &str) -> String {
    let (_, body) = request(router, "POST", "/api/auth/login", None, Some(json!({ "email": email, "password": "correct-password" }))).await;
    body["access_token"].as_str().unwrap().to_string()
}


async fn register(router: axum::Router, email: &str) -> String {
    let (status, _) = request(
        router.clone(),
        "POST",
        "/api/auth/register",
        None,
        Some(json!({ "email": email, "password": "correct-password", "org": { "mode": "create", "name": "Acme" } })),
    )
    .await;
    assert!(status.is_success(), "{status}");
    login(router, email).await
}

#[sqlx::test(migrations = "../../migrations")]
async fn hevy_is_connected_with_a_checked_key_and_listed_with_the_others(pool: PgPool) {
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};
    let hevy = MockServer::start().await;
    // Only this test reads HEVY_BASE_URL in this binary.
    std::env::set_var("HEVY_BASE_URL", hevy.uri());
    Mock::given(method("GET"))
        .and(path("/v1/user/info"))
        .and(header("api-key", "good-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"data": {"id": "u1", "name": "ana_lifts", "url": ""}})))
        .mount(&hevy)
        .await;
    Mock::given(method("GET")).and(path("/v1/user/info")).respond_with(ResponseTemplate::new(401)).mount(&hevy).await;

    let router = build_router(test_state(pool.clone()));
    let token = register(router.clone(), "ana@acme.test").await;

    let (status, overview) = request(router.clone(), "GET", "/api/connections", Some(&token), None).await;
    assert_eq!(status, StatusCode::OK);
    assert!(overview["hevy"].is_null());
    assert!(overview["google"]["email"].is_null());

    let (status, body) = request(router.clone(), "PUT", "/api/connections/hevy", Some(&token), Some(json!({"api_key": "wrong"}))).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"], "key_rejected");
    let (status, body) = request(router.clone(), "PUT", "/api/connections/hevy", Some(&token), Some(json!({"api_key": "  "}))).await;
    assert_eq!((status, body["error"].as_str()), (StatusCode::BAD_REQUEST, Some("missing_key")));

    let (status, body) = request(router.clone(), "PUT", "/api/connections/hevy", Some(&token), Some(json!({"api_key": "good-key"}))).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["connection"]["account_name"], "ana_lifts");
    assert!(!body.to_string().contains("good-key"), "the key never comes back");

    let (_, overview) = request(router.clone(), "GET", "/api/connections", Some(&token), None).await;
    assert_eq!(overview["hevy"]["account_name"], "ana_lifts");

    // Someone else sees nothing of it.
    let budi = register(router.clone(), "budi@acme.test").await;
    let (_, theirs) = request(router.clone(), "GET", "/api/connections/hevy", Some(&budi), None).await;
    assert!(theirs["connection"].is_null());

    let (status, _) = request(router.clone(), "DELETE", "/api/connections/hevy", Some(&token), None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (_, mine) = request(router, "GET", "/api/connections/hevy", Some(&token), None).await;
    assert!(mine["connection"].is_null());
}
