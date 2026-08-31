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
    }
}

async fn json_request(router: axum::Router, method: &str, uri: &str, body: Value, bearer: Option<&str>) -> (StatusCode, Value) {
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
    json_request(router.clone(), "POST", "/api/auth/register", json!({"email": email, "password": "correct-password", "org": {"mode": "create", "name": "Acme"}}), None).await;
    let (_, body) = json_request(router, "POST", "/api/auth/login", json!({"email": email, "password": "correct-password"}), None).await;
    body["access_token"].as_str().unwrap().to_string()
}

// sessions has no user_id column (migration 0004_sessions.sql: org_id, channel, chat_id) — seed
// a fresh org per call so this works regardless of whether user_id came from registration (which
// creates its own org) or a raw `INSERT INTO users DEFAULT VALUES`.
async fn seed_project(pool: &PgPool, user_id: Uuid) -> Uuid {
    let org_id: Uuid = sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Acme') RETURNING id").fetch_one(pool).await.unwrap();
    let session_id: Uuid = sqlx::query_scalar("INSERT INTO sessions (org_id, channel, chat_id) VALUES ($1, 'telegram', $2) RETURNING id")
        .bind(org_id)
        .bind(Uuid::new_v4().to_string())
        .fetch_one(pool)
        .await
        .unwrap();
    sqlx::query_scalar("INSERT INTO projects (user_id, session_id, name, description, plan) VALUES ($1, $2, 'Todo app', 'a simple list', '1. index.html') RETURNING id")
        .bind(user_id)
        .bind(session_id)
        .fetch_one(pool)
        .await
        .unwrap()
}

async fn user_id_by_email(pool: &PgPool, email: &str) -> Uuid {
    sqlx::query_scalar("SELECT user_id FROM web_credentials WHERE email = $1").bind(email).fetch_one(pool).await.unwrap()
}

#[sqlx::test(migrations = "../../migrations")]
async fn listing_projects_only_returns_the_caller_owns(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register_and_login(router.clone(), "owner@example.com").await;
    let user_id = user_id_by_email(&pool, "owner@example.com").await;
    seed_project(&pool, user_id).await;

    let other_id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(&pool).await.unwrap();
    let other_org: Uuid = sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Other Org') RETURNING id").fetch_one(&pool).await.unwrap();
    let other_session: Uuid = sqlx::query_scalar("INSERT INTO sessions (org_id, channel, chat_id) VALUES ($1, 'telegram', $2) RETURNING id")
        .bind(other_org)
        .bind(Uuid::new_v4().to_string())
        .fetch_one(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO projects (user_id, session_id, name) VALUES ($1, $2, 'Someone elses app')").bind(other_id).bind(other_session).execute(&pool).await.unwrap();

    let (status, body) = json_request(router, "GET", "/api/projects", Value::Null, Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    let projects = body.as_array().unwrap();
    assert_eq!(projects.len(), 1);
    assert_eq!(projects[0]["name"], "Todo app");
}

#[sqlx::test(migrations = "../../migrations")]
async fn get_project_includes_plan_and_files(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register_and_login(router.clone(), "owner@example.com").await;
    let user_id = user_id_by_email(&pool, "owner@example.com").await;
    let project_id = seed_project(&pool, user_id).await;
    sqlx::query("INSERT INTO project_files (project_id, path, content_type, size_bytes) VALUES ($1, 'index.html', 'text/html', 20)")
        .bind(project_id)
        .execute(&pool)
        .await
        .unwrap();

    let (status, body) = json_request(router, "GET", &format!("/api/projects/{project_id}"), Value::Null, Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["plan"], "1. index.html");
    assert_eq!(body["files"].as_array().unwrap().len(), 1);
    assert_eq!(body["files"][0]["path"], "index.html");
}

#[sqlx::test(migrations = "../../migrations")]
async fn get_project_for_another_users_project_is_not_found(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register_and_login(router.clone(), "attacker@example.com").await;

    let owner_id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(&pool).await.unwrap();
    let project_id = seed_project(&pool, owner_id).await;

    let (status, _) = json_request(router, "GET", &format!("/api/projects/{project_id}"), Value::Null, Some(&token)).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[sqlx::test(migrations = "../../migrations")]
async fn getting_a_file_without_s3_configured_is_service_unavailable(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register_and_login(router.clone(), "owner@example.com").await;
    let user_id = user_id_by_email(&pool, "owner@example.com").await;
    let project_id = seed_project(&pool, user_id).await;

    let (status, _) = json_request(router, "GET", &format!("/api/projects/{project_id}/files/index.html"), Value::Null, Some(&token)).await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
}

#[sqlx::test(migrations = "../../migrations")]
async fn unauthenticated_requests_are_rejected(pool: PgPool) {
    let router = build_router(test_state(pool));
    let (status, _) = json_request(router, "GET", "/api/projects", Value::Null, None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}
