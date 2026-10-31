
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

/// An agent's reply in the chat, as the crew writes them.
async fn reply(pool: &PgPool, session_id: &str, text: &str) -> Uuid {
    sqlx::query_scalar("INSERT INTO messages (session_id, content, agent_display_name, created_at) VALUES ($1::uuid, $2, 'Dana', now() + interval '1 second') RETURNING id")
        .bind(session_id)
        .bind(text)
        .fetch_one(pool)
        .await
        .unwrap()
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_shared_chat_opens_for_anyone_with_amounts_masked(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register_and_login(router.clone(), "sharer@example.com").await;
    let session_id = start_chat(router.clone(), &token).await;
    reply(&pool, &session_id, "Food came to Rp 1.250.000 this month.").await;

    let (status, share) = json_request(router.clone(), "POST", &format!("/api/sessions/{session_id}/shares"), json!({ "scope": "chat", "expires_in_days": 7 }), Some(&token)).await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(share["mask"], true);
    assert_eq!(share["chat_messages"], 2);
    assert_eq!(share["preview"], "Food came to Rp ••• this month.");
    assert_eq!(share["preview_by"], "Dana");
    assert!(share["expires_at"].is_string());

    let link = format!("/api/shared/{}", share["token"].as_str().unwrap());
    let (status, shared) = json_request(router.clone(), "GET", &link, Value::Null, None).await;
    assert_eq!(status, StatusCode::OK);
    let messages = shared["messages"].as_array().unwrap();
    assert_eq!(messages.len(), 2);
    assert_eq!(messages[0]["sender"], "user");
    assert_eq!(messages[1]["content"], "Food came to Rp ••• this month.");

    // Unmasked, the same link shows the amounts.
    let path = format!("/api/sessions/{session_id}/shares/{}", share["id"].as_str().unwrap());
    let (status, _) = json_request(router.clone(), "PUT", &path, json!({ "scope": "chat", "mask": false }), Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    let (_, shared) = json_request(router, "GET", &link, Value::Null, None).await;
    assert_eq!(shared["messages"][1]["content"], "Food came to Rp 1.250.000 this month.");
    assert!(shared["expires_at"].is_null());
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_shared_reply_shows_only_that_reply(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register_and_login(router.clone(), "reply-sharer@example.com").await;
    let session_id = start_chat(router.clone(), &token).await;
    let message_id = reply(&pool, &session_id, "Here's the plan.").await;

    let (status, _) = json_request(router.clone(), "POST", &format!("/api/sessions/{session_id}/shares"), json!({ "scope": "reply" }), Some(&token)).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let (_, share) = json_request(router.clone(), "POST", &format!("/api/sessions/{session_id}/shares"), json!({ "scope": "reply", "message_id": message_id }), Some(&token)).await;
    let (_, shared) = json_request(router, "GET", &format!("/api/shared/{}", share["token"].as_str().unwrap()), Value::Null, None).await;
    let messages = shared["messages"].as_array().unwrap();
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0]["content"], "Here's the plan.");
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_link_for_signed_in_people_asks_for_a_sign_in(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register_and_login(router.clone(), "private-sharer@example.com").await;
    let reader = register_and_login(router.clone(), "reader@example.com").await;
    let session_id = start_chat(router.clone(), &token).await;

    let (_, share) = json_request(router.clone(), "POST", &format!("/api/sessions/{session_id}/shares"), json!({ "scope": "chat", "anyone": false }), Some(&token)).await;
    let link = format!("/api/shared/{}", share["token"].as_str().unwrap());
    let (status, _) = json_request(router.clone(), "GET", &link, Value::Null, None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    let (status, _) = json_request(router, "GET", &link, Value::Null, Some(&reader)).await;
    assert_eq!(status, StatusCode::OK);
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_revoked_or_expired_link_is_gone_and_only_the_owner_shares(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register_and_login(router.clone(), "owner-share@example.com").await;
    let outsider = register_and_login(router.clone(), "outsider-share@example.com").await;
    let session_id = start_chat(router.clone(), &token).await;
    let shares = format!("/api/sessions/{session_id}/shares");

    let (status, _) = json_request(router.clone(), "POST", &shares, json!({ "scope": "chat" }), Some(&outsider)).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (status, _) = json_request(router.clone(), "POST", &shares, json!({ "scope": "chat", "expires_in_days": 3 }), Some(&token)).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    let (_, revoked) = json_request(router.clone(), "POST", &shares, json!({ "scope": "chat" }), Some(&token)).await;
    let (status, _) = json_request(router.clone(), "DELETE", &format!("{shares}/{}", revoked["id"].as_str().unwrap()), Value::Null, Some(&token)).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (status, _) = json_request(router.clone(), "GET", &format!("/api/shared/{}", revoked["token"].as_str().unwrap()), Value::Null, None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    let (_, expired) = json_request(router.clone(), "POST", &shares, json!({ "scope": "chat", "expires_in_days": 1 }), Some(&token)).await;
    sqlx::query("UPDATE chat_shares SET expires_at = now() - interval '1 minute' WHERE token = $1").bind(expired["token"].as_str().unwrap()).execute(&pool).await.unwrap();
    let (status, _) = json_request(router, "GET", &format!("/api/shared/{}", expired["token"].as_str().unwrap()), Value::Null, None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_person_lists_their_live_links_newest_first(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register_and_login(router.clone(), "lister@example.com").await;
    let other = register_and_login(router.clone(), "someone-else@example.com").await;
    let session_id = start_chat(router.clone(), &token).await;
    let shares = format!("/api/sessions/{session_id}/shares");
    let theirs = start_chat(router.clone(), &other).await;
    json_request(router.clone(), "POST", &format!("/api/sessions/{theirs}/shares"), json!({ "scope": "chat" }), Some(&other)).await;

    let (_, first) = json_request(router.clone(), "POST", &shares, json!({ "scope": "chat", "anyone": false }), Some(&token)).await;
    let (_, revoked) = json_request(router.clone(), "POST", &shares, json!({ "scope": "chat" }), Some(&token)).await;
    json_request(router.clone(), "DELETE", &format!("{shares}/{}", revoked["id"].as_str().unwrap()), Value::Null, Some(&token)).await;
    let (_, expired) = json_request(router.clone(), "POST", &shares, json!({ "scope": "chat", "expires_in_days": 1 }), Some(&token)).await;
    sqlx::query("UPDATE chat_shares SET expires_at = now() - interval '1 minute' WHERE id = $1")
        .bind(Uuid::parse_str(expired["id"].as_str().unwrap()).unwrap())
        .execute(&pool)
        .await
        .unwrap();
    let (_, last) = json_request(router.clone(), "POST", &shares, json!({ "scope": "chat", "expires_in_days": 7 }), Some(&token)).await;

    let (status, body) = json_request(router, "GET", "/api/shares", Value::Null, Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    let ids: Vec<&str> = body["shares"].as_array().unwrap().iter().map(|s| s["id"].as_str().unwrap()).collect();
    assert_eq!(ids, vec![last["id"].as_str().unwrap(), first["id"].as_str().unwrap()]);
    assert_eq!(body["shares"][1]["anyone"], false);
    assert_eq!(body["shares"][1]["session_id"], session_id.as_str());
    assert!(body["shares"][0]["expires_at"].is_string());
}
