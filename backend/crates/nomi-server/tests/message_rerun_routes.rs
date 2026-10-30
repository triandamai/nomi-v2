
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

/// A chat whose first turn has finished: the person asked, Dana answered and logged lunch.
async fn answered_chat(router: axum::Router, pool: &PgPool, email: &str) -> (String, String, Uuid, Uuid) {
    let token = register_and_login(router.clone(), email).await;
    let session_id = start_chat(router, &token).await;
    sqlx::query("UPDATE turn_jobs SET status = 'done' WHERE session_id = $1::uuid").bind(&session_id).execute(pool).await.unwrap();
    let start: Uuid = sqlx::query_scalar("SELECT id FROM messages WHERE session_id = $1::uuid AND sender_channel_identity_id IS NOT NULL").bind(&session_id).fetch_one(pool).await.unwrap();
    let reply: Uuid = sqlx::query_scalar("INSERT INTO messages (session_id, content, agent_display_name, created_at) VALUES ($1::uuid, 'Logged lunch, Rp 50.000.', 'Dana', now() + interval '1 second') RETURNING id")
        .bind(&session_id)
        .fetch_one(pool)
        .await
        .unwrap();
    let user: Uuid = sqlx::query_scalar("SELECT user_id FROM web_credentials WHERE email = $1").bind(email).fetch_one(pool).await.unwrap();
    sqlx::query("INSERT INTO money_transactions (user_id, occurred_at, amount_cents, category, description, source, kind, source_message_id) VALUES ($1, now(), 5000000, 'food', 'Lunch', 'agent', 'expense', $2)")
        .bind(user)
        .bind(start)
        .execute(pool)
        .await
        .unwrap();
    (token, session_id, start, reply)
}

async fn queued_text(pool: &PgPool, session_id: &str) -> Option<String> {
    sqlx::query_scalar("SELECT text FROM turn_jobs WHERE session_id = $1::uuid AND status = 'pending'").bind(session_id).fetch_optional(pool).await.unwrap()
}

#[sqlx::test(migrations = "../../migrations")]
async fn editing_the_latest_message_keeps_the_earlier_take_and_asks_again(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let (token, session_id, start, reply) = answered_chat(router.clone(), &pool, "editor@example.com").await;

    let (status, body) = json_request(router.clone(), "PATCH", &format!("/api/sessions/{session_id}/messages/{start}"), json!({ "text": "  hi, dinner instead  " }), Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!((body["version"].as_i64(), body["total"].as_i64()), (Some(2), Some(2)));
    assert!(body["edited_at"].is_string());
    assert_eq!(queued_text(&pool, &session_id).await.as_deref(), Some("hi, dinner instead"));

    // The reply and the lunch it logged are taken back; the message reads as edited.
    let (_, list) = json_request(router.clone(), "GET", &format!("/api/sessions/{session_id}/messages"), Value::Null, Some(&token)).await;
    let messages = list["messages"].as_array().unwrap();
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0]["content"], "hi, dinner instead");
    assert!(messages[0]["edited_at"].is_string());
    assert_eq!(messages[0]["versions"], 1);
    assert!(!messages.iter().any(|m| m["id"] == reply.to_string()));
    let lunches: i64 = sqlx::query_scalar("SELECT count(*) FROM money_transactions WHERE source_message_id = $1").bind(start).fetch_one(&pool).await.unwrap();
    assert_eq!(lunches, 0);

    // The earlier take is kept, with the reply it got.
    let (_, versions) = json_request(router, "GET", &format!("/api/sessions/{session_id}/messages/{start}/versions"), Value::Null, Some(&token)).await;
    assert_eq!(versions["total"], 2);
    assert_eq!(versions["versions"][0]["version"], 1);
    assert_eq!(versions["versions"][0]["text"], "hi");
    assert_eq!(versions["versions"][0]["replies"][0]["content"], "Logged lunch, Rp 50.000.");
}

#[sqlx::test(migrations = "../../migrations")]
async fn regenerating_a_reply_asks_the_same_message_again(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let (token, session_id, start, reply) = answered_chat(router.clone(), &pool, "regen@example.com").await;

    let (status, body) = json_request(router.clone(), "POST", &format!("/api/sessions/{session_id}/messages/{reply}/regenerate"), Value::Null, Some(&token)).await;
    assert_eq!(status, StatusCode::ACCEPTED);
    assert_eq!(body["message_id"], start.to_string());
    assert_eq!(body["total"], 2);
    assert!(body["edited_at"].is_null());
    assert_eq!(queued_text(&pool, &session_id).await.as_deref(), Some("hi"));
    let left: i64 = sqlx::query_scalar("SELECT count(*) FROM messages WHERE session_id = $1::uuid").bind(&session_id).fetch_one(&pool).await.unwrap();
    assert_eq!(left, 1);

    // While the crew works, it can't run again; and the reply asked from a reply's id is the same turn.
    let (status, _) = json_request(router.clone(), "POST", &format!("/api/sessions/{session_id}/messages/{start}/regenerate"), Value::Null, Some(&token)).await;
    assert_eq!(status, StatusCode::CONFLICT);
    let (_, versions) = json_request(router, "GET", &format!("/api/sessions/{session_id}/messages/{start}/versions"), Value::Null, Some(&token)).await;
    assert_eq!(versions["versions"].as_array().unwrap().len(), 1);
}

#[sqlx::test(migrations = "../../migrations")]
async fn only_the_latest_message_of_ones_own_chat_runs_again(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let (token, session_id, start, reply) = answered_chat(router.clone(), &pool, "latest@example.com").await;
    let outsider = register_and_login(router.clone(), "outsider-rerun@example.com").await;

    let (status, _) = json_request(router.clone(), "PATCH", &format!("/api/sessions/{session_id}/messages/{reply}"), json!({ "text": "x" }), Some(&token)).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let (status, _) = json_request(router.clone(), "PATCH", &format!("/api/sessions/{session_id}/messages/{start}"), json!({ "text": "   " }), Some(&token)).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let (status, _) = json_request(router.clone(), "PATCH", &format!("/api/sessions/{session_id}/messages/{start}"), json!({ "text": "mine" }), Some(&outsider)).await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    // A later message was answered with this one in mind.
    let (status, _) = json_request(router.clone(), "POST", &format!("/api/sessions/{session_id}/messages"), json!({ "text": "and tomorrow?" }), Some(&token)).await;
    assert_eq!(status, StatusCode::ACCEPTED);
    sqlx::query("UPDATE turn_jobs SET status = 'done' WHERE session_id = $1::uuid").bind(&session_id).execute(&pool).await.unwrap();
    let (status, _) = json_request(router, "PATCH", &format!("/api/sessions/{session_id}/messages/{start}"), json!({ "text": "hi again" }), Some(&token)).await;
    assert_eq!(status, StatusCode::CONFLICT);
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_reply_asked_again_shorter_tells_the_crew_without_changing_the_message(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let (token, session_id, start, reply) = answered_chat(router.clone(), &pool, "shorter@example.com").await;

    let (status, _) = json_request(router, "POST", &format!("/api/sessions/{session_id}/messages/{reply}/regenerate"), json!({ "style": "shorter" }), Some(&token)).await;
    assert_eq!(status, StatusCode::ACCEPTED);
    assert_eq!(queued_text(&pool, &session_id).await.as_deref(), Some(format!("hi\n\n{}", nomi_agent_core::prompts::REGENERATE_SHORTER).as_str()));
    let content: String = sqlx::query_scalar("SELECT content FROM messages WHERE id = $1").bind(start).fetch_one(&pool).await.unwrap();
    assert_eq!(content, "hi");
}
