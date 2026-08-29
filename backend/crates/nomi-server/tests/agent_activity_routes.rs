use axum::{body::Body, http::{Request, StatusCode}};
use http_body_util::BodyExt;
use nomi_server::app::{build_router, AppState};
use serde_json::{json, Value};
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;

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

async fn org_id_for(pool: &PgPool, email: &str) -> Uuid {
    sqlx::query_scalar(
        "SELECT m.org_id FROM memberships m \
         JOIN web_credentials w ON w.user_id = m.user_id \
         WHERE w.email = $1 AND m.status = 'active'",
    )
    .bind(email)
    .fetch_one(pool)
    .await
    .unwrap()
}

async fn user_id_for(pool: &PgPool, email: &str) -> Uuid {
    sqlx::query_scalar("SELECT user_id FROM web_credentials WHERE email = $1")
        .bind(email)
        .fetch_one(pool)
        .await
        .unwrap()
}

#[sqlx::test(migrations = "../../migrations")]
async fn returns_empty_when_no_delegations_exist_for_the_session(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register_and_login(router.clone(), "activity-empty@example.com").await;
    let org_id = org_id_for(&pool, "activity-empty@example.com").await;

    let session_id: Uuid = sqlx::query_scalar(
        "INSERT INTO sessions (org_id, channel, chat_id) VALUES ($1, 'telegram', 'c1') RETURNING id",
    )
    .bind(org_id)
    .fetch_one(&pool)
    .await
    .unwrap();

    let (status, body) =
        json_request(router, "GET", &format!("/api/sessions/{session_id}/agent-activity"), Value::Null, Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body.as_array().unwrap().len(), 0);
}

#[sqlx::test(migrations = "../../migrations")]
async fn lists_delegations_for_a_session_the_caller_owns(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register_and_login(router.clone(), "activity-owner@example.com").await;
    let org_id = org_id_for(&pool, "activity-owner@example.com").await;
    let user_id = user_id_for(&pool, "activity-owner@example.com").await;

    let session_id: Uuid = sqlx::query_scalar(
        "INSERT INTO sessions (org_id, channel, chat_id) VALUES ($1, 'telegram', 'c2') RETURNING id",
    )
    .bind(org_id)
    .fetch_one(&pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO agent_delegations (session_id, user_id, requesting_agent_type, target_agent_type, task, status, result) \
         VALUES ($1, $2, 'chitchat', 'money', 'check spending', 'completed', 'spent $42 on groceries')",
    )
    .bind(session_id)
    .bind(user_id)
    .execute(&pool)
    .await
    .unwrap();

    let (status, body) =
        json_request(router, "GET", &format!("/api/sessions/{session_id}/agent-activity"), Value::Null, Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    let items = body.as_array().unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["target_agent_type"], "money");
    assert_eq!(items[0]["status"], "completed");
    assert_eq!(items[0]["result"], "spent $42 on groceries");
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_session_belonging_to_a_different_org_is_not_accessible(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register_and_login(router.clone(), "activity-outsider@example.com").await;

    let other_org_id: Uuid =
        sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Someone Else') RETURNING id").fetch_one(&pool).await.unwrap();
    let session_id: Uuid = sqlx::query_scalar(
        "INSERT INTO sessions (org_id, channel, chat_id) VALUES ($1, 'telegram', 'c3') RETURNING id",
    )
    .bind(other_org_id)
    .fetch_one(&pool)
    .await
    .unwrap();

    let (status, _) =
        json_request(router, "GET", &format!("/api/sessions/{session_id}/agent-activity"), Value::Null, Some(&token)).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}
