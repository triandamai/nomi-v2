
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
        push: std::sync::Arc::new(nomi_server::push::NoopPush),
        shipyard: None,
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
        serde_json::from_slice(&bytes).unwrap_or(Value::Null)
    };
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

/// Starts a chat the way the app does: create it, then send its first message.
async fn start_chat(router: axum::Router, token: &str) -> String {
    let (_, created) = json_request(router.clone(), "POST", "/api/sessions", Value::Null, Some(token)).await;
    let session_id = created["session_id"].as_str().unwrap().to_string();
    let (status, _) = json_request(router, "POST", &format!("/api/sessions/{session_id}/messages"), json!({ "text": "hi" }), Some(token)).await;
    assert_eq!(status, StatusCode::ACCEPTED);
    session_id
}

/// Adds `count` said messages to the chat, a minute apart.
async fn add_messages(pool: &PgPool, session_id: &str, count: i32) {
    for i in 0..count {
        sqlx::query("INSERT INTO messages (session_id, content, created_at) VALUES ($1::uuid, $2, now() - make_interval(mins => $3))")
            .bind(session_id)
            .bind(format!("reply {i}"))
            .bind(count - i)
            .execute(pool)
            .await
            .unwrap();
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn summarising_a_long_chat_is_accepted(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register_and_login(router.clone(), "long@example.com").await;
    let session_id = start_chat(router.clone(), &token).await;
    add_messages(&pool, &session_id, 12).await;

    let (status, body) = json_request(router, "POST", &format!("/api/sessions/{session_id}/summarize"), Value::Null, Some(&token)).await;
    assert_eq!(status, StatusCode::ACCEPTED);
    assert_eq!(body["summarizing"], true);
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_second_summary_while_one_runs_is_refused(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register_and_login(router.clone(), "twice@example.com").await;
    let session_id = start_chat(router.clone(), &token).await;
    add_messages(&pool, &session_id, 12).await;
    sqlx::query("UPDATE sessions SET summarizing_since = now() WHERE id = $1::uuid").bind(&session_id).execute(&pool).await.unwrap();

    let (status, body) = json_request(router, "POST", &format!("/api/sessions/{session_id}/summarize"), Value::Null, Some(&token)).await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["error"], "already_summarizing");
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_summary_that_was_lost_does_not_block_a_new_one(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register_and_login(router.clone(), "lost@example.com").await;
    let session_id = start_chat(router.clone(), &token).await;
    add_messages(&pool, &session_id, 12).await;
    sqlx::query("UPDATE sessions SET summarizing_since = now() - interval '1 day' WHERE id = $1::uuid").bind(&session_id).execute(&pool).await.unwrap();

    let (status, _) = json_request(router, "POST", &format!("/api/sessions/{session_id}/summarize"), Value::Null, Some(&token)).await;
    assert_eq!(status, StatusCode::ACCEPTED);
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_short_chat_has_nothing_to_summarise_and_is_left_unlocked(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register_and_login(router.clone(), "short@example.com").await;
    let session_id = start_chat(router.clone(), &token).await;

    let (status, body) = json_request(router, "POST", &format!("/api/sessions/{session_id}/summarize"), Value::Null, Some(&token)).await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["error"], "nothing_to_summarise");
    let since: Option<chrono::DateTime<chrono::Utc>> = sqlx::query_scalar("SELECT summarizing_since FROM sessions WHERE id = $1::uuid").bind(&session_id).fetch_one(&pool).await.unwrap();
    assert!(since.is_none());
}

#[sqlx::test(migrations = "../../migrations")]
async fn someone_elses_chat_cannot_be_summarised(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let owner = register_and_login(router.clone(), "own@example.com").await;
    let other = register_and_login(router.clone(), "other@example.com").await;
    let session_id = start_chat(router.clone(), &owner).await;
    add_messages(&pool, &session_id, 12).await;

    let (status, _) = json_request(router.clone(), "POST", &format!("/api/sessions/{session_id}/summarize"), Value::Null, Some(&other)).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (status, _) = json_request(router, "POST", &format!("/api/sessions/{session_id}/summarize"), Value::Null, None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}
