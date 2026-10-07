//! Feedback that trains memory, and people fixing or confirming what Nomi remembers.
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

async fn send(router: axum::Router, method: &str, uri: &str, token: &str, body: Option<Value>) -> (StatusCode, Value) {
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

async fn weight(pool: &PgPool, memory_id: uuid::Uuid) -> f64 {
    sqlx::query_scalar("SELECT weight FROM memory_items WHERE id = $1").bind(memory_id).fetch_one(pool).await.unwrap()
}

/// Ana's chat with one reply that drew on one of her memories (weight 1.0).
async fn reply_with_memory(router: axum::Router, pool: &PgPool) -> (String, uuid::Uuid, uuid::Uuid, uuid::Uuid) {
    let (token, ana) = register_and_login(router.clone(), pool, "ana@acme.test").await;
    let (_, created) = send(router, "POST", "/api/sessions", &token, Some(json!({}))).await;
    let session_id: uuid::Uuid = created["session_id"].as_str().unwrap().parse().unwrap();
    let memory_id: uuid::Uuid = sqlx::query_scalar(
        "INSERT INTO memory_items (user_id, content, kind, embedding, embedding_provider, embedding_model) \
         VALUES ($1, 'Vegetarian', 'preference', $2::vector, 'fake', 'fake-model') RETURNING id",
    )
    .bind(ana)
    .bind(format!("[{}]", vec!["0.1"; 1536].join(",")))
    .fetch_one(pool)
    .await
    .unwrap();
    let message_id: uuid::Uuid = sqlx::query_scalar("INSERT INTO messages (session_id, content, agent_display_name) VALUES ($1, 'Try the tofu', 'Nomi') RETURNING id")
        .bind(session_id)
        .fetch_one(pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO message_memory_usage (message_id, memory_id) VALUES ($1, $2)").bind(message_id).bind(memory_id).execute(pool).await.unwrap();
    (token, session_id, message_id, memory_id)
}

#[sqlx::test(migrations = "../../migrations")]
async fn thumbs_train_the_memories_a_reply_used_and_changing_your_mind_undoes_it(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let (token, session_id, message_id, memory_id) = reply_with_memory(router.clone(), &pool).await;
    let feedback = format!("/api/sessions/{session_id}/messages/{message_id}/feedback");

    let (status, _) = send(router.clone(), "PUT", &feedback, &token, Some(json!({ "rating": "up" }))).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    assert!((weight(&pool, memory_id).await - 1.2).abs() < 1e-9);

    // Flipping to a thumbs-down takes the thumbs-up back first: 1.0 × 0.8, not 1.2 × 0.8.
    send(router.clone(), "PUT", &feedback, &token, Some(json!({ "rating": "down", "reason": "wrong_memory" }))).await;
    assert!((weight(&pool, memory_id).await - 0.8).abs() < 1e-9);
    let reason: Option<String> = sqlx::query_scalar("SELECT reason FROM message_feedback WHERE message_id = $1").bind(message_id).fetch_one(&pool).await.unwrap();
    assert_eq!(reason.as_deref(), Some("wrong_memory"));

    // Saying the same thing again changes nothing; removing the rating restores the memory.
    send(router.clone(), "PUT", &feedback, &token, Some(json!({ "rating": "down" }))).await;
    assert!((weight(&pool, memory_id).await - 0.8).abs() < 1e-9);
    let (status, _) = send(router.clone(), "DELETE", &feedback, &token, None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    assert!((weight(&pool, memory_id).await - 1.0).abs() < 1e-9);

    let (status, _) = send(router, "PUT", &feedback, &token, Some(json!({ "rating": "down", "reason": "rude" }))).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_reply_shows_which_memories_it_used(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let (token, session_id, message_id, memory_id) = reply_with_memory(router.clone(), &pool).await;

    let (_, list) = send(router.clone(), "GET", &format!("/api/sessions/{session_id}/messages"), &token, None).await;
    let reply = list["messages"].as_array().unwrap().iter().find(|m| m["id"] == message_id.to_string()).unwrap();
    assert_eq!(reply["memory_count"], 1);

    let (status, used) = send(router.clone(), "GET", &format!("/api/sessions/{session_id}/messages/{message_id}/memories"), &token, None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(used["memories"][0]["id"], memory_id.to_string());
    assert_eq!(used["memories"][0]["kind"], "preference");

    // Someone else can't read them.
    let (budi_token, _) = register_and_login(router.clone(), &pool, "budi@acme.test").await;
    let (status, _) = send(router, "GET", &format!("/api/sessions/{session_id}/messages/{message_id}/memories"), &budi_token, None).await;
    assert_ne!(status, StatusCode::OK);
}

#[sqlx::test(migrations = "../../migrations")]
async fn people_can_fix_confirm_and_retire_their_memories(pool: PgPool) {
    std::env::set_var("EMBEDDING_PROVIDER", "fake");
    let router = build_router(test_state(pool.clone()));
    let (token, _, _, memory_id) = reply_with_memory(router.clone(), &pool).await;
    let uri = format!("/api/memory/{memory_id}");

    let (status, _) = send(router.clone(), "PUT", &uri, &token, Some(json!({ "content": "Pescatarian since 2026", "kind": "fact" }))).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (content, kind, confirmed): (String, String, bool) =
        sqlx::query_as("SELECT content, kind, confirmed_at IS NOT NULL FROM memory_items WHERE id = $1").bind(memory_id).fetch_one(&pool).await.unwrap();
    assert_eq!((content.as_str(), kind.as_str(), confirmed), ("Pescatarian since 2026", "fact", true));

    let (status, _) = send(router.clone(), "PUT", &uri, &token, Some(json!({ "content": "  " }))).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let (status, _) = send(router.clone(), "PUT", &uri, &token, Some(json!({ "content": "x", "kind": "vibe" }))).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    let (status, _) = send(router.clone(), "POST", &format!("{uri}/confirm"), &token, None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    assert!((weight(&pool, memory_id).await - 1.1).abs() < 1e-9);

    // A memory a newer one replaced drops out of the list.
    sqlx::query("UPDATE memory_items SET archived_at = now() WHERE id = $1").bind(memory_id).execute(&pool).await.unwrap();
    let (_, list) = send(router.clone(), "GET", "/api/memory", &token, None).await;
    assert_eq!(list["memories"].as_array().unwrap().len(), 0);

    let (budi_token, _) = register_and_login(router.clone(), &pool, "budi@acme.test").await;
    let (status, _) = send(router, "POST", &format!("{uri}/confirm"), &budi_token, None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}
