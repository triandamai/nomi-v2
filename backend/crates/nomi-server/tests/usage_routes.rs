//! Billing & usage: each person reads their own token usage, spend and plan.
use axum::{body::Body, http::{Request, StatusCode}};
use http_body_util::BodyExt;
use nomi_server::app::{build_router, AppState};
use serde_json::{json, Value};
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;

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

async fn user_id(pool: &PgPool, email: &str) -> Uuid {
    sqlx::query_scalar("SELECT user_id FROM web_credentials WHERE email = $1").bind(email).fetch_one(pool).await.unwrap()
}


#[sqlx::test(migrations = "../../migrations")]
async fn people_see_their_own_usage_and_plan(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let (status, _) = request(router.clone(), "GET", "/api/usage", None, None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    let (status, _) = request(
        router.clone(),
        "POST",
        "/api/auth/register",
        None,
        Some(json!({ "email": "ana@acme.test", "password": "correct-password", "org": { "mode": "create", "name": "Acme" } })),
    )
    .await;
    assert!(status.is_success(), "{status}");
    let ana = user_id(&pool, "ana@acme.test").await;
    sqlx::query(
        "INSERT INTO llm_usage (user_id, source, model_label, provider, model_id, input_tokens, output_tokens, cost_usd) \
         VALUES ($1, 'nomi', 'Nomi Standard', 'anthropic', 'claude-x', 1200, 300, 0.0081)",
    )
    .bind(ana)
    .execute(&pool)
    .await
    .unwrap();
    let token = login(router.clone(), "ana@acme.test").await;

    let (status, usage) = request(router.clone(), "GET", "/api/usage", Some(&token), None).await;
    assert_eq!(status, StatusCode::OK, "{usage}");
    assert_eq!(usage["tokens_used"], 1500);
    assert_eq!(usage["plan"]["id"], "free");
    assert!(usage["plan"]["monthly_tokens"].as_i64().unwrap() > 0);
    assert_eq!(usage["by_model"][0]["label"], "Nomi Standard");
    assert!((usage["spend_usd"].as_f64().unwrap() - 0.0081).abs() < 1e-9);

    let (status, brief) = request(router, "GET", "/api/usage/brief", Some(&token), None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(brief["tokens_used"], 1500);
    assert_eq!(brief["month"], usage["current_month"]);
}
