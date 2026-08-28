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

async fn user_id_for_email(pool: &PgPool, email: &str) -> uuid::Uuid {
    sqlx::query_scalar("SELECT user_id FROM web_credentials WHERE email = $1")
        .bind(email)
        .fetch_one(pool)
        .await
        .unwrap()
}

#[sqlx::test(migrations = "../../migrations")]
async fn history_is_empty_for_a_user_with_no_personality_set(pool: PgPool) {
    let router = build_router(test_state(pool));
    let token = register_and_login(router.clone(), "user1@example.com").await;

    let (status, body) = json_request(router, "GET", "/api/personality/history", Value::Null, Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["versions"].as_array().unwrap().len(), 0);
}

#[sqlx::test(migrations = "../../migrations")]
async fn rollback_creates_a_new_current_version_and_history_reflects_it(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register_and_login(router.clone(), "user2@example.com").await;

    let user_id = user_id_for_email(&pool, "user2@example.com").await;
    let mut conn = pool.acquire().await.unwrap();
    nomi_agent_core::personality::set_personality(&mut conn, None, None, user_id, "Be sarcastic.").await.unwrap();
    nomi_agent_core::personality::set_personality(&mut conn, None, None, user_id, "Be warm.").await.unwrap();
    drop(conn);

    let (status, _) =
        json_request(router.clone(), "POST", "/api/personality/rollback", json!({"version": 1}), Some(&token)).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (status, body) = json_request(router, "GET", "/api/personality/history", Value::Null, Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    let versions = body["versions"].as_array().unwrap();
    assert_eq!(versions.len(), 3);
    assert_eq!(versions[0]["version"], 3);
    assert_eq!(versions[0]["description"], "Be sarcastic.");
    assert_eq!(versions[0]["is_current"], true);
}

#[sqlx::test(migrations = "../../migrations")]
async fn rolling_back_to_an_unknown_version_returns_404(pool: PgPool) {
    let router = build_router(test_state(pool));
    let token = register_and_login(router.clone(), "user3@example.com").await;

    let (status, _) = json_request(router, "POST", "/api/personality/rollback", json!({"version": 99}), Some(&token)).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_second_users_history_never_leaks_into_the_first_users_response(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token2 = {
        register_and_login(router.clone(), "user4@example.com").await;
        register_and_login(router.clone(), "user5@example.com").await
    };

    let user1_id = user_id_for_email(&pool, "user4@example.com").await;
    let mut conn = pool.acquire().await.unwrap();
    nomi_agent_core::personality::set_personality(&mut conn, None, None, user1_id, "User 1's personality.").await.unwrap();
    drop(conn);

    let (_, body) = json_request(router, "GET", "/api/personality/history", Value::Null, Some(&token2)).await;
    assert_eq!(body["versions"].as_array().unwrap().len(), 0);
}
