//! GET /api/tools/catalog: the tools a crew member can be given, searched, without the ones an
//! admin switched off.
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use nomi_server::app::{build_router, AppState};
use serde_json::{json, Value};
use sqlx::PgPool;
use tower::ServiceExt;

const SECRET: &str = "test-secret-do-not-use-in-prod";
/// A limit smaller than the catalog.
const SMALL_LIMIT: usize = 2;

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
        tool_catalog: nomi_server::build_tool_catalog(nomi_test_support::test_project_storage()),
        email_codes: nomi_server::sign_in_codes::EmailCodes::Off,
        push: std::sync::Arc::new(nomi_server::push::NoopPush),
        shipyard: None,
    }
}

async fn get(router: axum::Router, uri: &str, token: Option<&str>) -> (StatusCode, Value) {
    let mut builder = Request::builder().method("GET").uri(uri);
    if let Some(token) = token {
        builder = builder.header("authorization", format!("Bearer {token}"));
    }
    let response = router.oneshot(builder.body(Body::empty()).unwrap()).await.unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    (status, serde_json::from_slice(&bytes).unwrap_or(Value::Null))
}

async fn register(router: axum::Router, email: &str) -> String {
    let request = Request::builder()
        .method("POST")
        .uri("/api/auth/register")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({ "email": email, "password": "correct-password", "org": { "mode": "create", "name": "Acme" } }).to_string(),
        ))
        .unwrap();
    let response = router.clone().oneshot(request).await.unwrap();
    assert!(response.status().is_success());
    let request = Request::builder()
        .method("POST")
        .uri("/api/auth/login")
        .header("content-type", "application/json")
        .body(Body::from(json!({ "email": email, "password": "correct-password" }).to_string()))
        .unwrap();
    let response = router.oneshot(request).await.unwrap();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body: Value = serde_json::from_slice(&bytes).unwrap();
    body["access_token"].as_str().unwrap().to_string()
}

fn names(body: &Value) -> Vec<String> {
    body["tools"].as_array().unwrap().iter().map(|t| t["name"].as_str().unwrap().to_string()).collect()
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_catalog_lists_searches_and_skips_switched_off_tools(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let (status, _) = get(router.clone(), "/api/tools/catalog", None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    let token = register(router.clone(), "tool-catalog@example.com").await;
    let catalog_size = nomi_server::build_tool_catalog(nomi_test_support::test_project_storage()).entries().len();

    let (status, all) = get(router.clone(), "/api/tools/catalog", Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(names(&all).len(), catalog_size);
    let first = &all["tools"][0];
    assert!(first["description"].is_string() && first["group"].is_string() && first["keywords"].is_array(), "{first}");

    let (_, budget) = get(router.clone(), "/api/tools/catalog?q=budget", Some(&token)).await;
    let found = names(&budget);
    assert!(!found.is_empty() && found[0].contains("budget"), "{found:?}");
    let (_, limited) = get(router.clone(), &format!("/api/tools/catalog?q=&limit={SMALL_LIMIT}"), Some(&token)).await;
    assert_eq!(names(&limited).len(), SMALL_LIMIT);

    let switched_off = &found[0];
    sqlx::query("INSERT INTO tool_settings (name, enabled) VALUES ($1, false)").bind(switched_off).execute(&pool).await.unwrap();
    let (_, budget) = get(router.clone(), "/api/tools/catalog?q=budget", Some(&token)).await;
    assert!(!names(&budget).contains(switched_off));
    let (_, all) = get(router, "/api/tools/catalog", Some(&token)).await;
    assert_eq!(names(&all).len(), catalog_size - 1);
    sqlx::query("DELETE FROM tool_settings WHERE name = $1").bind(switched_off).execute(&pool).await.unwrap();
}
