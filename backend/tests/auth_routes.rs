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
    let mut builder = Request::builder()
        .method(method)
        .uri(uri)
        .header("content-type", "application/json");
    if let Some(token) = bearer {
        builder = builder.header("authorization", format!("Bearer {token}"));
    }
    let response = router
        .oneshot(builder.body(Body::from(body.to_string())).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let json_body = if bytes.is_empty() {
        Value::Null
    } else {
        // Error responses in this handler set are `(StatusCode, &'static str)`
        // tuples, which axum's IntoResponse renders as plain-text bodies, not
        // JSON. No test in this file asserts on error-body content (only on
        // status codes), so fall back to Null rather than panicking on those.
        serde_json::from_slice(&bytes).unwrap_or(Value::Null)
    };
    (status, json_body)
}

#[sqlx::test]
async fn register_returns_a_usable_token_pair(pool: PgPool) {
    let router = build_router(test_state(pool));

    let (status, register_body) = json_request(
        router.clone(),
        "POST",
        "/api/auth/register",
        json!({ "email": "grace@example.com", "password": "correct-password", "org": { "mode": "create", "name": "Acme" } }),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let access_token = register_body["access_token"].as_str().unwrap().to_string();
    assert!(register_body["refresh_token"].as_str().is_some());

    let (status, whoami_body) = json_request(router, "GET", "/api/whoami", Value::Null, Some(&access_token)).await;
    assert_eq!(status, StatusCode::OK);
    assert!(whoami_body["sub"].is_string());
}

#[sqlx::test]
async fn login_after_registration_also_succeeds(pool: PgPool) {
    let router = build_router(test_state(pool));

    json_request(
        router.clone(),
        "POST",
        "/api/auth/register",
        json!({ "email": "grace2@example.com", "password": "correct-password", "org": { "mode": "create", "name": "Acme" } }),
        None,
    )
    .await;

    let (status, login_body) = json_request(
        router.clone(),
        "POST",
        "/api/auth/login",
        json!({ "email": "grace2@example.com", "password": "correct-password" }),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let access_token = login_body["access_token"].as_str().unwrap().to_string();

    let (status, whoami_body) = json_request(router, "GET", "/api/whoami", Value::Null, Some(&access_token)).await;
    assert_eq!(status, StatusCode::OK);
    assert!(whoami_body["sub"].is_string());
}

#[sqlx::test]
async fn remove_member_rejects_a_caller_scoped_to_a_different_org(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));

    let (_, _) = json_request(
        router.clone(),
        "POST",
        "/api/auth/register",
        json!({ "email": "owner-a@example.com", "password": "correct-password", "org": { "mode": "create", "name": "Org A" } }),
        None,
    )
    .await;
    let (_, login_body) = json_request(
        router.clone(),
        "POST",
        "/api/auth/login",
        json!({ "email": "owner-a@example.com", "password": "correct-password" }),
        None,
    )
    .await;
    let owner_a_token = login_body["access_token"].as_str().unwrap().to_string();

    let org_b_id: uuid::Uuid =
        sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Org B') RETURNING id")
            .fetch_one(&pool)
            .await
            .unwrap();
    let target_user_id: uuid::Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id")
        .fetch_one(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO memberships (org_id, user_id, role) VALUES ($1, $2, 'member')")
        .bind(org_b_id)
        .bind(target_user_id)
        .execute(&pool)
        .await
        .unwrap();

    let (status, _) = json_request(
        router,
        "DELETE",
        &format!("/api/orgs/{org_b_id}/members/{target_user_id}"),
        Value::Null,
        Some(&owner_a_token),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}

#[sqlx::test]
async fn remove_member_succeeds_for_the_owning_org_owner(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));

    let (_, _) = json_request(
        router.clone(),
        "POST",
        "/api/auth/register",
        json!({ "email": "owner@example.com", "password": "correct-password", "org": { "mode": "create", "name": "Acme" } }),
        None,
    )
    .await;
    let (_, login_body) = json_request(
        router.clone(),
        "POST",
        "/api/auth/login",
        json!({ "email": "owner@example.com", "password": "correct-password" }),
        None,
    )
    .await;
    let owner_token = login_body["access_token"].as_str().unwrap().to_string();
    let owner_claims = nomi_orchestrator::auth::claims::Claims::decode(&owner_token, SECRET).unwrap();
    let org_id = owner_claims.active_org_id;

    let member_user_id: uuid::Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id")
        .fetch_one(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO memberships (org_id, user_id, role) VALUES ($1, $2, 'member')")
        .bind(org_id)
        .bind(member_user_id)
        .execute(&pool)
        .await
        .unwrap();

    let (status, _) = json_request(
        router,
        "DELETE",
        &format!("/api/orgs/{org_id}/members/{member_user_id}"),
        Value::Null,
        Some(&owner_token),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let member_status: String = sqlx::query_scalar("SELECT status FROM memberships WHERE org_id = $1 AND user_id = $2")
        .bind(org_id)
        .bind(member_user_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(member_status, "removed");
}

#[sqlx::test]
async fn refresh_and_logout_flow(pool: PgPool) {
    let router = build_router(test_state(pool));

    json_request(
        router.clone(),
        "POST",
        "/api/auth/register",
        json!({ "email": "hank@example.com", "password": "correct-password", "org": { "mode": "create", "name": "Acme" } }),
        None,
    )
    .await;
    let (_, login_body) = json_request(
        router.clone(),
        "POST",
        "/api/auth/login",
        json!({ "email": "hank@example.com", "password": "correct-password" }),
        None,
    )
    .await;
    let refresh_token = login_body["refresh_token"].as_str().unwrap().to_string();

    let (status, refresh_body) = json_request(
        router.clone(),
        "POST",
        "/api/auth/refresh",
        json!({ "refresh_token": refresh_token }),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(refresh_body["access_token"].is_string());

    let (status, _) = json_request(
        router.clone(),
        "POST",
        "/api/auth/logout",
        json!({ "refresh_token": refresh_token }),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (status, _) = json_request(
        router,
        "POST",
        "/api/auth/refresh",
        json!({ "refresh_token": refresh_token }),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}
