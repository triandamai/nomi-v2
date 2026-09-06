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
    }
}

async fn json_request(router: axum::Router, method: &str, uri: &str, bearer: Option<&str>) -> (StatusCode, Value) {
    let mut builder = Request::builder().method(method).uri(uri);
    if let Some(token) = bearer {
        builder = builder.header("authorization", format!("Bearer {token}"));
    }
    let response = router.oneshot(builder.body(Body::empty()).unwrap()).await.unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let json_body = if bytes.is_empty() { Value::Null } else { serde_json::from_slice(&bytes).unwrap_or(Value::Null) };
    (status, json_body)
}

async fn register_and_login(router: axum::Router, pool: &PgPool, email: &str) -> (String, uuid::Uuid) {
    json_request_post(
        router.clone(),
        "/api/auth/register",
        json!({ "email": email, "password": "correct-password", "org": { "mode": "create", "name": "Acme" } }),
    )
    .await;
    let (_, login_body) = json_request_post(router, "/api/auth/login", json!({ "email": email, "password": "correct-password" })).await;
    let token = login_body["access_token"].as_str().unwrap().to_string();
    let user_id: uuid::Uuid = sqlx::query_scalar("SELECT user_id FROM web_credentials WHERE email = $1")
        .bind(email)
        .fetch_one(pool)
        .await
        .unwrap();
    (token, user_id)
}

async fn json_request_post(router: axum::Router, uri: &str, body: Value) -> (StatusCode, Value) {
    let response = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(uri)
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let json_body = if bytes.is_empty() { Value::Null } else { serde_json::from_slice(&bytes).unwrap_or(Value::Null) };
    (status, json_body)
}

async fn seed_memory(pool: &PgPool, user_id: uuid::Uuid, content: &str) {
    let embedding = format!("[{}]", vec!["0.01"; 1536].join(","));
    sqlx::query("INSERT INTO memory_items (user_id, content, embedding) VALUES ($1, $2, $3::vector)")
        .bind(user_id)
        .bind(content)
        .bind(&embedding)
        .execute(pool)
        .await
        .unwrap();
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_user_with_no_memories_gets_an_empty_list(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let (token, _) = register_and_login(router.clone(), &pool, "nomemory@example.com").await;

    let (status, body) = json_request(router, "GET", "/api/memory", Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["memories"].as_array().unwrap().len(), 0);
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_user_sees_their_own_memories_with_the_full_embedding(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let (token, user_id) = register_and_login(router.clone(), &pool, "hasmemory@example.com").await;
    seed_memory(&pool, user_id, "likes window seats").await;

    let (status, body) = json_request(router, "GET", "/api/memory", Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    let memories = body["memories"].as_array().unwrap();
    assert_eq!(memories.len(), 1);
    assert_eq!(memories[0]["content"], "likes window seats");
    assert_eq!(memories[0]["weight"], 1.0);
    assert_eq!(memories[0]["embedding"].as_array().unwrap().len(), 1536);
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_user_never_sees_another_users_memories(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let (_, owner_id) = register_and_login(router.clone(), &pool, "owner@example.com").await;
    seed_memory(&pool, owner_id, "owner's secret fact").await;

    let (other_token, _) = register_and_login(router.clone(), &pool, "other@example.com").await;
    let (status, body) = json_request(router, "GET", "/api/memory", Some(&other_token)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["memories"].as_array().unwrap().len(), 0);
}

#[sqlx::test(migrations = "../../migrations")]
async fn fetching_memories_without_auth_is_rejected(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let (status, _) = json_request(router, "GET", "/api/memory", None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}
