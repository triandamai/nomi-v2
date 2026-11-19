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
    let (_, login_body) = json_request(router, "POST", "/api/auth/login", json!({"email": email, "password": "correct-password"}), None).await;
    login_body["access_token"].as_str().unwrap().to_string()
}

#[sqlx::test(migrations = "../../migrations")]
async fn resolving_an_approval_that_is_not_pending_returns_409(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register_and_login(router.clone(), "user@example.com").await;

    // Create a session and a plain message with no pending approval state on any agent_sessions row.
    let (_, create_body) = json_request(router.clone(), "POST", "/api/sessions", Value::Null, Some(&token)).await;
    let session_id = create_body["session_id"].as_str().unwrap();
    let message_id: uuid::Uuid = sqlx::query_scalar("INSERT INTO messages (session_id, content) VALUES ($1, 'hi') RETURNING id")
        .bind(uuid::Uuid::parse_str(session_id).unwrap())
        .fetch_one(&pool)
        .await
        .unwrap();

    let (status, _) = json_request(
        router,
        "PUT",
        &format!("/api/sessions/{session_id}/messages/{message_id}/approval"),
        json!({"decision": "approve"}),
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
}

#[sqlx::test(migrations = "../../migrations")]
async fn an_invalid_decision_value_is_rejected(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register_and_login(router.clone(), "user2@example.com").await;

    let (_, create_body) = json_request(router.clone(), "POST", "/api/sessions", Value::Null, Some(&token)).await;
    let session_id = create_body["session_id"].as_str().unwrap();
    let message_id: uuid::Uuid = sqlx::query_scalar("INSERT INTO messages (session_id, content) VALUES ($1, 'hi') RETURNING id")
        .bind(uuid::Uuid::parse_str(session_id).unwrap())
        .fetch_one(&pool)
        .await
        .unwrap();

    let (status, _) = json_request(
        router,
        "PUT",
        &format!("/api/sessions/{session_id}/messages/{message_id}/approval"),
        json!({"decision": "maybe"}),
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

/// A session of `token`'s owner with an approval card in `status`, decided `decided_secs_ago`
/// seconds ago, and the deny the turn is still waiting on (as the route queues it).
async fn denied_card(pool: &PgPool, router: &axum::Router, token: &str, decided_secs_ago: i64) -> (String, uuid::Uuid) {
    let (_, create_body) = json_request(router.clone(), "POST", "/api/sessions", Value::Null, Some(token)).await;
    let session_id = create_body["session_id"].as_str().unwrap().to_string();
    let message_id: uuid::Uuid = sqlx::query_scalar(
        "INSERT INTO messages (session_id, content, content_blocks) VALUES ($1, 'card', $2) RETURNING id",
    )
    .bind(uuid::Uuid::parse_str(&session_id).unwrap())
    .bind(json!([{
        "kind": "approval_request", "id": uuid::Uuid::new_v4(), "tool_name": "gmail.send", "description": "Send",
        "input": {"to": "bu.sri@example.com"}, "status": "denied",
        "decided_at": chrono::Utc::now() - chrono::Duration::seconds(decided_secs_ago)
    }]))
    .fetch_one(pool)
    .await
    .unwrap();
    sqlx::query("INSERT INTO approval_resumes (message_id, decision, run_after) VALUES ($1, 'deny', now() + make_interval(secs => $2))")
        .bind(message_id)
        .bind((nomi_turn::approval::UNDO_WINDOW_SECONDS - decided_secs_ago) as f64)
        .execute(pool)
        .await
        .unwrap();
    (session_id, message_id)
}

#[sqlx::test(migrations = "../../migrations")]
async fn undoing_a_deny_inside_the_window_puts_the_card_back_and_twice_is_harmless(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register_and_login(router.clone(), "undo@example.com").await;
    let (session_id, message_id) = denied_card(&pool, &router, &token, 1).await;
    let uri = format!("/api/sessions/{session_id}/messages/{message_id}/approval/undo");

    let (status, body) = json_request(router.clone(), "POST", &uri, Value::Null, Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "pending");
    let card: Value = sqlx::query_scalar("SELECT content_blocks->0 FROM messages WHERE id = $1").bind(message_id).fetch_one(&pool).await.unwrap();
    assert_eq!(card["status"], "pending");
    assert!(card["decided_at"].is_null());
    let resume: String = sqlx::query_scalar("SELECT status FROM approval_resumes WHERE message_id = $1").bind(message_id).fetch_one(&pool).await.unwrap();
    assert_eq!(resume, "cancelled");

    let (again, _) = json_request(router, "POST", &uri, Value::Null, Some(&token)).await;
    assert_eq!(again, StatusCode::OK);
    let still: String = sqlx::query_scalar("SELECT status FROM approval_resumes WHERE message_id = $1").bind(message_id).fetch_one(&pool).await.unwrap();
    assert_eq!(still, "cancelled");
}

#[sqlx::test(migrations = "../../migrations")]
async fn undoing_a_deny_after_the_window_is_a_409_and_changes_nothing(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register_and_login(router.clone(), "late@example.com").await;
    let (session_id, message_id) = denied_card(&pool, &router, &token, 30).await;

    let (status, _) = json_request(router, "POST", &format!("/api/sessions/{session_id}/messages/{message_id}/approval/undo"), Value::Null, Some(&token)).await;
    assert_eq!(status, StatusCode::CONFLICT);
    let card: Value = sqlx::query_scalar("SELECT content_blocks->0 FROM messages WHERE id = $1").bind(message_id).fetch_one(&pool).await.unwrap();
    assert_eq!(card["status"], "denied");
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_scoped_remember_needs_the_recipient_scope_and_edits_need_an_approval(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register_and_login(router.clone(), "scope@example.com").await;
    let (_, create_body) = json_request(router.clone(), "POST", "/api/sessions", Value::Null, Some(&token)).await;
    let session_id = create_body["session_id"].as_str().unwrap();
    let message_id: uuid::Uuid = sqlx::query_scalar("INSERT INTO messages (session_id, content) VALUES ($1, 'hi') RETURNING id")
        .bind(uuid::Uuid::parse_str(session_id).unwrap())
        .fetch_one(&pool)
        .await
        .unwrap();
    let uri = format!("/api/sessions/{session_id}/messages/{message_id}/approval");

    let (status, _) = json_request(router.clone(), "PUT", &uri, json!({"decision": "approve", "remember": {"scope": "channel", "value": "x"}}), Some(&token)).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let (status, _) = json_request(router.clone(), "PUT", &uri, json!({"decision": "deny", "input": {"to": "a@b.co"}}), Some(&token)).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let (status, _) = json_request(router.clone(), "PUT", &uri, json!({"decision": "approve", "input": "not an object"}), Some(&token)).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    // The old shape still reaches the pending check.
    let (status, _) = json_request(router, "PUT", &uri, json!({"decision": "approve", "remember": true}), Some(&token)).await;
    assert_eq!(status, StatusCode::CONFLICT);
}

#[sqlx::test(migrations = "../../migrations")]
async fn discarding_a_stop_clears_the_partial_work_and_twice_is_a_no_op(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register_and_login(router.clone(), "discard@example.com").await;
    let (_, create_body) = json_request(router.clone(), "POST", "/api/sessions", Value::Null, Some(&token)).await;
    let session_id = uuid::Uuid::parse_str(create_body["session_id"].as_str().unwrap()).unwrap();
    let user_id: uuid::Uuid = sqlx::query_scalar("SELECT user_id FROM sessions WHERE id = $1").bind(session_id).fetch_one(&pool).await.unwrap();
    let task_id: uuid::Uuid = sqlx::query_scalar(
        "INSERT INTO background_tasks (session_id, user_id, agent_type, label, command, status, output) \
         VALUES ($1, $2, 'coding', 'build', 'npm run build', 'cancelled', 'half a build') RETURNING id",
    )
    .bind(session_id)
    .bind(user_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    let message_id: uuid::Uuid = sqlx::query_scalar("INSERT INTO messages (session_id, content, content_blocks) VALUES ($1, 'stopped', $2) RETURNING id")
        .bind(session_id)
        .bind(json!([{"kind": "stopped", "tasks": [{"agent": "coding", "title": "build", "state": "stopped", "background_task_id": task_id}]}]))
        .fetch_one(&pool)
        .await
        .unwrap();
    let uri = format!("/api/sessions/{session_id}/stopped/{message_id}/discard");

    let (status, body) = json_request(router.clone(), "POST", &uri, Value::Null, Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["discarded"], 1);
    let output: Option<String> = sqlx::query_scalar("SELECT output FROM background_tasks WHERE id = $1").bind(task_id).fetch_one(&pool).await.unwrap();
    assert_eq!(output, None);
    let card: Value = sqlx::query_scalar("SELECT content_blocks->0 FROM messages WHERE id = $1").bind(message_id).fetch_one(&pool).await.unwrap();
    assert_eq!(card["discarded"], true);

    let (again, body) = json_request(router, "POST", &uri, Value::Null, Some(&token)).await;
    assert_eq!(again, StatusCode::OK);
    assert_eq!(body["discarded"], 0);
}
