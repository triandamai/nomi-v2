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

async fn register_via_api(router: axum::Router, email: &str) {
    json_request(
        router,
        "POST",
        "/api/auth/register",
        json!({ "email": email, "password": "correct-password", "org": { "mode": "create", "name": "Acme" } }),
        None,
    )
    .await;
}

async fn login_via_api(router: axum::Router, email: &str) -> String {
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
async fn non_admin_is_forbidden_from_viewing_the_dashboard(pool: PgPool) {
    let router = build_router(test_state(pool));
    register_via_api(router.clone(), "regular@example.com").await;
    let token = login_via_api(router.clone(), "regular@example.com").await;

    let (status, _) = json_request(router, "GET", "/api/admin/dashboard", Value::Null, Some(&token)).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}

#[sqlx::test(migrations = "../../migrations")]
async fn non_admin_is_forbidden_from_listing_agents(pool: PgPool) {
    let router = build_router(test_state(pool));
    register_via_api(router.clone(), "regular@example.com").await;
    let token = login_via_api(router.clone(), "regular@example.com").await;

    let (status, _) = json_request(router, "GET", "/api/admin/agents", Value::Null, Some(&token)).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}

#[sqlx::test(migrations = "../../migrations")]
async fn dashboard_returns_zero_counts_on_a_fresh_install(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let admin_token = register_admin_and_login(router.clone(), &pool, "admin@example.com").await;

    let (status, body) = json_request(router, "GET", "/api/admin/dashboard", Value::Null, Some(&admin_token)).await;
    assert_eq!(status, StatusCode::OK);
    // The admin's own registration is the only user row, and nothing else has happened yet —
    // COALESCE(SUM(...), 0) must report 0, not null or an error, for the token fields.
    assert_eq!(body["total_users"], 1);
    assert_eq!(body["running_agents"], 0);
    assert_eq!(body["tokens_today"], 0);
    assert_eq!(body["tokens_all_time"], 0);
}

#[sqlx::test(migrations = "../../migrations")]
async fn dashboard_reports_correct_counts(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let admin_token = register_admin_and_login(router.clone(), &pool, "admin@example.com").await;
    register_via_api(router.clone(), "regular@example.com").await;

    let org_id: Uuid = sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Acme') RETURNING id")
        .fetch_one(&pool)
        .await
        .unwrap();
    let session_id: Uuid = sqlx::query_scalar(
        "INSERT INTO sessions (org_id, channel, chat_id) VALUES ($1, 'telegram', 'chat-1') RETURNING id",
    )
    .bind(org_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    let user_id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(&pool).await.unwrap();
    let identity_id: Uuid = sqlx::query_scalar(
        "INSERT INTO channel_identities (user_id, channel, channel_user_id) VALUES ($1, 'telegram', 'tg-1') RETURNING id",
    )
    .bind(user_id)
    .fetch_one(&pool)
    .await
    .unwrap();

    // One active session, one completed — running_agents must count only the active one.
    sqlx::query(
        "INSERT INTO agent_sessions (session_id, sender_channel_identity_id, agent_type, status) VALUES ($1, $2, 'chitchat', 'active')",
    )
    .bind(session_id)
    .bind(identity_id)
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO agent_sessions (session_id, sender_channel_identity_id, agent_type, status) VALUES ($1, $2, 'money', 'completed')",
    )
    .bind(session_id)
    .bind(identity_id)
    .execute(&pool)
    .await
    .unwrap();

    // One AgentReplied event today, one backdated two days — proves the today/all-time split.
    sqlx::query(
        "INSERT INTO agent_events (session_id, agent_type, event_type, payload, created_at) \
         VALUES ($1, 'chitchat', 'AgentReplied', $2, now())",
    )
    .bind(session_id)
    .bind(json!({"input_tokens": 100, "output_tokens": 50}))
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO agent_events (session_id, agent_type, event_type, payload, created_at) \
         VALUES ($1, 'chitchat', 'AgentReplied', $2, now() - interval '2 days')",
    )
    .bind(session_id)
    .bind(json!({"input_tokens": 200, "output_tokens": 75}))
    .execute(&pool)
    .await
    .unwrap();

    let (status, body) = json_request(router, "GET", "/api/admin/dashboard", Value::Null, Some(&admin_token)).await;
    assert_eq!(status, StatusCode::OK);
    // Three distinct users exist by this point: the admin (registered above), "regular@example.com"
    // (registered above), and the bare user row inserted directly for the agent session's identity.
    assert_eq!(body["total_users"], 3);
    assert_eq!(body["running_agents"], 1);
    assert_eq!(body["tokens_today"], 150);
    assert_eq!(body["tokens_all_time"], 425);
}

#[sqlx::test(migrations = "../../migrations")]
async fn agents_endpoint_groups_running_sessions_by_user_and_excludes_non_active(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let admin_token = register_admin_and_login(router.clone(), &pool, "admin@example.com").await;

    let org_id: Uuid = sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Acme') RETURNING id")
        .fetch_one(&pool)
        .await
        .unwrap();
    let session_id: Uuid = sqlx::query_scalar(
        "INSERT INTO sessions (org_id, channel, chat_id) VALUES ($1, 'telegram', 'chat-1') RETURNING id",
    )
    .bind(org_id)
    .fetch_one(&pool)
    .await
    .unwrap();

    // User A: has a web account, one active agent session.
    let user_a: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(&pool).await.unwrap();
    sqlx::query("INSERT INTO web_credentials (user_id, email, password_hash) VALUES ($1, 'a@example.com', 'x')")
        .bind(user_a)
        .execute(&pool)
        .await
        .unwrap();
    let identity_a: Uuid = sqlx::query_scalar(
        "INSERT INTO channel_identities (user_id, channel, channel_user_id) VALUES ($1, 'telegram', 'a-tg') RETURNING id",
    )
    .bind(user_a)
    .fetch_one(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO agent_sessions (session_id, sender_channel_identity_id, agent_type, status) VALUES ($1, $2, 'chitchat', 'active')",
    )
    .bind(session_id)
    .bind(identity_a)
    .execute(&pool)
    .await
    .unwrap();

    // User B: no web account (channel-only), one active session and one completed (excluded).
    let user_b: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(&pool).await.unwrap();
    let identity_b: Uuid = sqlx::query_scalar(
        "INSERT INTO channel_identities (user_id, channel, channel_user_id) VALUES ($1, 'telegram', 'b-tg') RETURNING id",
    )
    .bind(user_b)
    .fetch_one(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO agent_sessions (session_id, sender_channel_identity_id, agent_type, status) VALUES ($1, $2, 'money', 'active')",
    )
    .bind(session_id)
    .bind(identity_b)
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO agent_sessions (session_id, sender_channel_identity_id, agent_type, status) VALUES ($1, $2, 'personality', 'completed')",
    )
    .bind(session_id)
    .bind(identity_b)
    .execute(&pool)
    .await
    .unwrap();

    let (status, body) = json_request(router, "GET", "/api/admin/agents", Value::Null, Some(&admin_token)).await;
    assert_eq!(status, StatusCode::OK);
    let users = body["users"].as_array().unwrap();
    assert_eq!(users.len(), 2);

    let group_a = users.iter().find(|g| g["user_id"] == user_a.to_string()).unwrap();
    assert_eq!(group_a["label"], "a@example.com");
    assert_eq!(group_a["agents"].as_array().unwrap().len(), 1);

    let group_b = users.iter().find(|g| g["user_id"] == user_b.to_string()).unwrap();
    assert_eq!(group_b["label"], "telegram:b-tg");
    assert_eq!(group_b["agents"].as_array().unwrap().len(), 1);
    assert_eq!(group_b["agents"][0]["agent_type"], "money");
}
