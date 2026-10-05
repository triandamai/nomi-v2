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

fn sample_agent_body() -> Value {
    json!({
        "name": "Weather Bot",
        "system_prompt": "You report the weather.",
        "intent_label": "weather",
        "intent_description": "the user is asking about the weather",
        "granted_tools": [],
        "supports_todos": false,
        "supports_plans": false,
        "can_delegate": false,
    })
}

#[sqlx::test(migrations = "../../migrations")]
async fn create_then_list_dynamic_agents(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register_admin_and_login(router.clone(), &pool, "admin1@example.com").await;

    let (status, create_body) =
        json_request(router.clone(), "POST", "/api/admin/dynamic-agents", sample_agent_body(), Some(&token)).await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(create_body["name"], "Weather Bot");
    assert_eq!(create_body["intent_label"], "weather");
    assert_eq!(create_body["is_active"], true);

    let (status, list_body) = json_request(router, "GET", "/api/admin/dynamic-agents", Value::Null, Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(list_body.as_array().unwrap().len(), 1);
}

#[sqlx::test(migrations = "../../migrations")]
async fn create_rejects_an_unrecognized_granted_tool(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register_admin_and_login(router.clone(), &pool, "admin2@example.com").await;

    let mut body = sample_agent_body();
    body["granted_tools"] = json!(["not_a_real_tool"]);

    let (status, _) = json_request(router, "POST", "/api/admin/dynamic-agents", body, Some(&token)).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[sqlx::test(migrations = "../../migrations")]
async fn non_admin_cannot_list_or_create(pool: PgPool) {
    let router = build_router(test_state(pool));
    register_via_api(router.clone(), "regular@example.com").await;
    let token = login_via_api(router.clone(), "regular@example.com").await;

    let (status, _) = json_request(router.clone(), "GET", "/api/admin/dynamic-agents", Value::Null, Some(&token)).await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    let (status, _) = json_request(router, "POST", "/api/admin/dynamic-agents", sample_agent_body(), Some(&token)).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}

#[sqlx::test(migrations = "../../migrations")]
async fn toggle_active_flips_is_active(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register_admin_and_login(router.clone(), &pool, "admin3@example.com").await;

    let (_, create_body) =
        json_request(router.clone(), "POST", "/api/admin/dynamic-agents", sample_agent_body(), Some(&token)).await;
    let id = create_body["id"].as_str().unwrap();
    assert_eq!(create_body["is_active"], true);

    let (status, toggled) = json_request(
        router.clone(),
        "POST",
        &format!("/api/admin/dynamic-agents/{id}/toggle-active"),
        Value::Null,
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(toggled["is_active"], false);

    let (status, toggled_again) = json_request(
        router,
        "POST",
        &format!("/api/admin/dynamic-agents/{id}/toggle-active"),
        Value::Null,
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(toggled_again["is_active"], true);
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_dynamic_agent_keeps_its_chosen_look_and_joins_everyones_crew(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let admin = register_admin_and_login(router.clone(), &pool, "admin@example.com").await;

    let mut body = sample_agent_body();
    body["shape"] = json!("puffy7");
    body["tone"] = json!("dusk");
    body["motion"] = json!("wobble");
    let (status, created) = json_request(router.clone(), "POST", "/api/admin/dynamic-agents", body, Some(&admin)).await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!((created["shape"].as_str(), created["tone"].as_str(), created["motion"].as_str()), (Some("puffy7"), Some("dusk"), Some("wobble")));

    // Any signed-in user sees the whole crew: Nomi first, the built-ins, then the dynamic agent.
    register_via_api(router.clone(), "member@example.com").await;
    let member = login_via_api(router.clone(), "member@example.com").await;
    let (status, crew) = json_request(router, "GET", "/api/agents", Value::Null, Some(&member)).await;
    assert_eq!(status, StatusCode::OK);
    let agents = crew["agents"].as_array().unwrap();
    assert_eq!(agents[0]["name"], "Nomi");
    assert!(agents.iter().any(|a| a["agent_type"] == "money" && a["role"] == "Transactions, budgets, subscriptions"));
    let dynamic = agents.iter().find(|a| a["is_dynamic"] == true).expect("the dynamic agent is in the crew");
    assert_eq!(dynamic["agent_type"], created["id"]);
    assert_eq!(dynamic["shape"], "puffy7");
    assert_eq!(dynamic["state"], "idle");
    assert_eq!(agents[0]["status"], "Ready when you are");
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_crew_shows_what_each_agent_is_doing_for_you(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    register_via_api(router.clone(), "busy@example.com").await;
    let token = login_via_api(router.clone(), "busy@example.com").await;
    let user_id: uuid::Uuid = sqlx::query_scalar("SELECT user_id FROM web_credentials WHERE email = 'busy@example.com'").fetch_one(&pool).await.unwrap();
    let session_id: uuid::Uuid = sqlx::query_scalar(
        "INSERT INTO sessions (org_id, channel, chat_id) VALUES ((SELECT id FROM organizations LIMIT 1), 'web', 'c') RETURNING id",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    for (target, status, task) in [("coding", "processing", "Write the hero section"), ("money", "completed", "Find idle subscriptions")] {
        sqlx::query(
            "INSERT INTO agent_delegations (session_id, user_id, requesting_agent_type, target_agent_type, task, status) VALUES ($1, $2, 'chitchat', $3, $4, $5)",
        )
        .bind(session_id)
        .bind(user_id)
        .bind(target)
        .bind(task)
        .bind(status)
        .execute(&pool)
        .await
        .unwrap();
    }

    let (_, crew) = json_request(router, "GET", "/api/agents", Value::Null, Some(&token)).await;
    let find = |t: &str| crew["agents"].as_array().unwrap().iter().find(|a| a["agent_type"] == t).unwrap().clone();
    assert_eq!(find("coding")["state"], "working");
    assert_eq!(find("coding")["status"], "Write the hero section");
    assert_eq!(find("money")["state"], "done");
    assert_eq!(find("money")["status"], "Finished · Find idle subscriptions");
    assert_eq!(find("planning")["state"], "idle");
}

#[sqlx::test(migrations = "../../migrations")]
async fn an_unknown_shape_is_rejected(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let admin = register_admin_and_login(router.clone(), &pool, "admin@example.com").await;
    let mut body = sample_agent_body();
    body["shape"] = json!("dodecahedron");
    let (status, _) = json_request(router, "POST", "/api/admin/dynamic-agents", body, Some(&admin)).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}
