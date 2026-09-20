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

async fn register_via_api(router: axum::Router, email: &str) {
    router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({ "email": email, "password": "correct-password", "org": { "mode": "create", "name": "Acme" } }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
}

async fn json_request_with_body(router: axum::Router, method: &str, uri: &str, bearer: Option<&str>, body: Value) -> (StatusCode, Value) {
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

async fn login_via_api(router: axum::Router, email: &str) -> String {
    let (_, body) = json_request_with_body(
        router,
        "POST",
        "/api/auth/login",
        None,
        json!({ "email": email, "password": "correct-password" }),
    )
    .await;
    body["access_token"].as_str().unwrap().to_string()
}

async fn make_platform_admin(pool: &PgPool, email: &str) {
    sqlx::query("UPDATE users SET is_platform_admin = true WHERE id = (SELECT user_id FROM web_credentials WHERE email = $1)")
        .bind(email)
        .execute(pool)
        .await
        .unwrap();
}

async fn register_admin_and_login(router: axum::Router, pool: &PgPool, email: &str) -> String {
    register_via_api(router.clone(), email).await;
    make_platform_admin(pool, email).await;
    login_via_api(router, email).await
}

#[sqlx::test(migrations = "../../migrations")]
async fn non_admin_is_forbidden(pool: PgPool) {
    let router = build_router(test_state(pool));
    register_via_api(router.clone(), "regular@example.com").await;
    let token = login_via_api(router.clone(), "regular@example.com").await;

    let (status, _) = json_request(router, "GET", "/api/admin/agent-events", Some(&token)).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}

#[sqlx::test(migrations = "../../migrations")]
async fn returns_recent_events_never_including_tool_input_or_result(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register_admin_and_login(router.clone(), &pool, "admin@example.com").await;

    let org_id: Uuid = sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Acme') RETURNING id").fetch_one(&pool).await.unwrap();
    let session_id: Uuid = sqlx::query_scalar("INSERT INTO sessions (org_id, channel, chat_id) VALUES ($1, 'telegram', 'c1') RETURNING id")
        .bind(org_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO agent_events (session_id, agent_type, event_type, payload) \
         VALUES ($1, 'money', 'ToolCalled', $2)",
    )
    .bind(session_id)
    .bind(json!({"tool_name": "list_transactions", "input": {"limit": 10}, "result": "sensitive data here", "is_error": false}))
    .execute(&pool)
    .await
    .unwrap();

    let (status, body) = json_request(router, "GET", "/api/admin/agent-events", Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    let items = body.as_array().unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["tool_name"], "list_transactions");
    assert_eq!(items[0]["is_error"], false);
    assert!(items[0].get("input").is_none());
    assert!(items[0].get("result").is_none());
    assert!(!body.to_string().contains("sensitive data here"));
}

#[sqlx::test(migrations = "../../migrations")]
async fn session_id_filters_to_one_session(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register_admin_and_login(router.clone(), &pool, "admin2@example.com").await;

    let org_id: Uuid = sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Acme') RETURNING id").fetch_one(&pool).await.unwrap();
    let session_a: Uuid = sqlx::query_scalar("INSERT INTO sessions (org_id, channel, chat_id) VALUES ($1, 'telegram', 'a') RETURNING id")
        .bind(org_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    let session_b: Uuid = sqlx::query_scalar("INSERT INTO sessions (org_id, channel, chat_id) VALUES ($1, 'telegram', 'b') RETURNING id")
        .bind(org_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO agent_events (session_id, agent_type, event_type) VALUES ($1, 'money', 'AgentSpawned')")
        .bind(session_a)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO agent_events (session_id, agent_type, event_type) VALUES ($1, 'money', 'AgentSpawned')")
        .bind(session_b)
        .execute(&pool)
        .await
        .unwrap();

    let (status, body) = json_request(router, "GET", &format!("/api/admin/agent-events?session_id={session_a}"), Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    let items = body.as_array().unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["session_id"], session_a.to_string());
}

#[sqlx::test(migrations = "../../migrations")]
async fn events_carry_a_resolved_agent_display_name(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register_admin_and_login(router.clone(), &pool, "admin3@example.com").await;

    let org_id: Uuid = sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Acme') RETURNING id").fetch_one(&pool).await.unwrap();
    let session_id: Uuid = sqlx::query_scalar("INSERT INTO sessions (org_id, channel, chat_id) VALUES ($1, 'telegram', 'c2') RETURNING id")
        .bind(org_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    let user_id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(&pool).await.unwrap();
    let dynamic_agent_id: Uuid = sqlx::query_scalar(
        "INSERT INTO dynamic_agents (name, system_prompt, intent_label, intent_description, created_by) \
         VALUES ('Weather Bot', 'You report the weather.', 'weather', 'weather questions', $1) RETURNING id",
    )
    .bind(user_id)
    .fetch_one(&pool)
    .await
    .unwrap();

    // chitchat: the one built-in agent whose display name deliberately diverges from a naive
    // Title-Case of its agent_type.
    sqlx::query("INSERT INTO agent_events (session_id, agent_type, event_type) VALUES ($1, 'chitchat', 'AgentReplied')")
        .bind(session_id)
        .execute(&pool)
        .await
        .unwrap();
    // A dynamic agent: agent_type stores the dynamic_agents.id as text, resolved via a join.
    sqlx::query("INSERT INTO agent_events (session_id, agent_type, event_type) VALUES ($1, $2, 'AgentSpawned')")
        .bind(session_id)
        .bind(dynamic_agent_id.to_string())
        .execute(&pool)
        .await
        .unwrap();

    let (status, body) = json_request(router, "GET", "/api/admin/agent-events", Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    let items = body.as_array().unwrap();
    assert_eq!(items.len(), 2);

    let chitchat_event = items.iter().find(|i| i["agent_type"] == "chitchat").unwrap();
    assert_eq!(chitchat_event["agent_display_name"], "Nomi");

    let dynamic_event = items.iter().find(|i| i["agent_type"] == dynamic_agent_id.to_string()).unwrap();
    assert_eq!(dynamic_event["agent_display_name"], "Weather Bot");
}
