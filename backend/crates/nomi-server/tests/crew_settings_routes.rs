//! A person's own crew: switching members off, permissions, soul.md, activity and own members.
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
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
        tool_catalog: nomi_server::build_tool_catalog(nomi_test_support::test_project_storage()),
        email_codes: nomi_server::sign_in_codes::EmailCodes::Off,
        push: std::sync::Arc::new(nomi_server::push::NoopPush),
        shipyard: None,
    }
}

async fn request(
    router: axum::Router,
    method: &str,
    uri: &str,
    token: Option<&str>,
    body: Option<Value>,
) -> (StatusCode, Value) {
    let mut builder = Request::builder().method(method).uri(uri);
    if let Some(token) = token {
        builder = builder.header("authorization", format!("Bearer {token}"));
    }
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
    (
        status,
        if bytes.is_empty() {
            Value::Null
        } else {
            serde_json::from_slice(&bytes).unwrap_or(Value::Null)
        },
    )
}

async fn login(router: axum::Router, email: &str) -> String {
    let (_, body) = request(
        router,
        "POST",
        "/api/auth/login",
        None,
        Some(json!({ "email": email, "password": "correct-password" })),
    )
    .await;
    body["access_token"].as_str().unwrap().to_string()
}

async fn register(router: axum::Router, email: &str) -> String {
    let (status, _) = request(
        router.clone(),
        "POST",
        "/api/auth/register",
        None,
        Some(json!({ "email": email, "password": "correct-password", "org": { "mode": "create", "name": "Acme" } })),
    )
    .await;
    assert!(status.is_success(), "{status}");
    login(router, email).await
}

async fn user_id(pool: &PgPool, email: &str) -> uuid::Uuid {
    sqlx::query_scalar("SELECT user_id FROM web_credentials WHERE email = $1")
        .bind(email)
        .fetch_one(pool)
        .await
        .unwrap()
}

fn member<'a>(crew: &'a Value, agent_type: &str) -> Option<&'a Value> {
    crew["agents"]
        .as_array()
        .unwrap()
        .iter()
        .find(|a| a["agent_type"] == agent_type)
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_member_switches_off_and_on_but_nomi_stays_on(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register(router.clone(), "crew-switch@example.com").await;

    let (status, body) = request(
        router.clone(),
        "PUT",
        "/api/agents/money",
        Some(&token),
        Some(json!({ "enabled": false })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, json!({ "agent_type": "money", "enabled": false }));
    let (_, crew) = request(router.clone(), "GET", "/api/agents", Some(&token), None).await;
    let money = member(&crew, "money").unwrap();
    assert_eq!(money["enabled"], false);
    assert_eq!(money["state"], "paused");
    assert_eq!(member(&crew, "chitchat").unwrap()["is_default"], true);

    let (status, _) = request(
        router.clone(),
        "PUT",
        "/api/agents/money",
        Some(&token),
        Some(json!({ "enabled": true })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let (_, crew) = request(router.clone(), "GET", "/api/agents", Some(&token), None).await;
    assert_eq!(member(&crew, "money").unwrap()["enabled"], true);

    let (status, _) = request(
        router.clone(),
        "PUT",
        "/api/agents/chitchat",
        Some(&token),
        Some(json!({ "enabled": false })),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let (status, _) = request(
        router.clone(),
        "PUT",
        "/api/agents/nobody",
        Some(&token),
        Some(json!({ "enabled": false })),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (status, _) = request(
        router,
        "PUT",
        "/api/agents/money",
        None,
        Some(json!({ "enabled": false })),
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[sqlx::test(migrations = "../../migrations")]
async fn permissions_list_each_tool_and_save_levels(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register(router.clone(), "crew-perms@example.com").await;

    let (status, body) = request(
        router.clone(),
        "GET",
        "/api/agents/money/permissions",
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let log = body
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["key"] == "log_transaction")
        .cloned()
        .unwrap();
    assert_eq!(log["label"], "Log transaction");
    assert_eq!(log["level"], log["default_level"]);

    let (status, body) = request(
        router.clone(),
        "PUT",
        "/api/agents/money/permissions",
        Some(&token),
        Some(json!([{ "key": "log_transaction", "level": "never" }, { "key": "list_transactions", "level": "allow" }])),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let level = |key: &str| {
        body.as_array()
            .unwrap()
            .iter()
            .find(|p| p["key"] == key)
            .unwrap()["level"]
            .clone()
    };
    assert_eq!(level("log_transaction"), "never");
    assert_eq!(level("list_transactions"), "allow");

    let (status, _) = request(
        router.clone(),
        "PUT",
        "/api/agents/money/permissions",
        Some(&token),
        Some(json!([{ "key": "rm_rf", "level": "allow" }])),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let (status, _) = request(
        router,
        "PUT",
        "/api/agents/money/permissions",
        Some(&token),
        Some(json!([{ "key": "log_transaction", "level": "maybe" }])),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[sqlx::test(migrations = "../../migrations")]
async fn soul_saves_versions_and_rolls_back(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register(router.clone(), "crew-soul@example.com").await;

    let (status, body) = request(
        router.clone(),
        "GET",
        "/api/agents/money/soul",
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, json!({ "text": "", "version": 0 }));

    let (_, body) = request(
        router.clone(),
        "PUT",
        "/api/agents/money/soul",
        Some(&token),
        Some(json!({ "text": "  Be frugal. " })),
    )
    .await;
    assert_eq!(body, json!({ "text": "Be frugal.", "version": 1 }));
    let (_, body) = request(
        router.clone(),
        "PUT",
        "/api/agents/money/soul",
        Some(&token),
        Some(json!({ "text": "Be generous." })),
    )
    .await;
    assert_eq!(body["version"], 2);

    let (status, body) = request(
        router.clone(),
        "GET",
        "/api/agents/money/soul/versions",
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let versions = body["versions"].as_array().unwrap();
    assert_eq!(versions.len(), 2);
    assert_eq!(versions[0]["version"], 2);
    assert_eq!(versions[0]["is_current"], true);

    let (status, body) = request(
        router.clone(),
        "POST",
        "/api/agents/money/soul/rollback",
        Some(&token),
        Some(json!({ "version": 1 })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, json!({ "text": "Be frugal.", "version": 1 }));
    let (_, body) = request(
        router.clone(),
        "GET",
        "/api/agents/money/soul",
        Some(&token),
        None,
    )
    .await;
    assert_eq!(body["version"], 1);

    let (status, _) = request(
        router.clone(),
        "POST",
        "/api/agents/money/soul/rollback",
        Some(&token),
        Some(json!({ "version": 9 })),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let too_long = "x".repeat(8_001);
    let (status, _) = request(
        router,
        "PUT",
        "/api/agents/money/soul",
        Some(&token),
        Some(json!({ "text": too_long })),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[sqlx::test(migrations = "../../migrations")]
async fn activity_lists_tasks_and_tool_calls_from_the_persons_chats(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register(router.clone(), "crew-activity@example.com").await;
    let me = user_id(&pool, "crew-activity@example.com").await;
    let org: uuid::Uuid =
        sqlx::query_scalar("SELECT org_id FROM memberships WHERE user_id = $1 LIMIT 1")
            .bind(me)
            .fetch_one(&pool)
            .await
            .unwrap();
    let session: uuid::Uuid = sqlx::query_scalar("INSERT INTO sessions (org_id, channel, chat_id, user_id, title) VALUES ($1, 'web', 'crew-activity', $2, 'Spending') RETURNING id")
        .bind(org)
        .bind(me)
        .fetch_one(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO agent_delegations (session_id, user_id, requesting_agent_type, target_agent_type, task, status) VALUES ($1, $2, 'chitchat', 'money', 'Sum March', 'completed')")
        .bind(session)
        .bind(me)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO agent_events (session_id, agent_type, event_type, payload) VALUES ($1, 'money', 'ToolCalled', '{\"tool_name\": \"list_transactions\", \"is_error\": false}')")
        .bind(session)
        .execute(&pool)
        .await
        .unwrap();

    let (status, body) = request(
        router,
        "GET",
        "/api/agents/money/activity",
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let items = body["items"].as_array().unwrap();
    assert_eq!(items.len(), 2);
    assert!(items.iter().any(|i| i["kind"] == "task"
        && i["text"] == "Sum March"
        && i["status"] == "completed"
        && i["chat_title"] == "Spending"));
    assert!(items
        .iter()
        .any(|i| i["kind"] == "tool" && i["text"] == "list_transactions" && i["status"] == "ok"));
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_person_makes_a_member_of_their_own(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register(router.clone(), "crew-own@example.com").await;
    let other = register(router.clone(), "crew-other@example.com").await;

    let (status, body) = request(
        router.clone(),
        "GET",
        "/api/agents/templates",
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(body["templates"]
        .as_array()
        .unwrap()
        .iter()
        .any(|t| t["key"] == "budget_coach"));

    let (status, made) = request(
        router.clone(),
        "POST",
        "/api/agents",
        Some(&token),
        Some(json!({ "template": "budget_coach", "name": "Penny" })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{made}");
    assert_eq!(made["name"], "Penny");
    assert_eq!(made["soul_version"], 1);
    let agent_type = made["agent_type"].as_str().unwrap().to_string();

    let (_, crew) = request(router.clone(), "GET", "/api/agents", Some(&token), None).await;
    let penny = member(&crew, &agent_type).unwrap();
    assert_eq!(penny["is_own"], true);
    assert_eq!(penny["is_dynamic"], true);
    let (_, crew) = request(router.clone(), "GET", "/api/agents", Some(&other), None).await;
    assert!(member(&crew, &agent_type).is_none(), "nobody else sees it");
    let (status, _) = request(
        router.clone(),
        "GET",
        &format!("/api/agents/{agent_type}/soul"),
        Some(&other),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    let (_, perms) = request(
        router.clone(),
        "GET",
        &format!("/api/agents/{agent_type}/permissions"),
        Some(&token),
        None,
    )
    .await;
    let keys: Vec<&str> = perms
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["key"].as_str().unwrap())
        .collect();
    assert!(
        keys.contains(&"set_budget") && keys.contains(&"list_budgets"),
        "{keys:?}"
    );
    let (_, soul) = request(
        router.clone(),
        "GET",
        &format!("/api/agents/{agent_type}/soul"),
        Some(&token),
        None,
    )
    .await;
    assert!(soul["text"].as_str().unwrap().starts_with("# Budget coach"));

    let (status, blank) = request(
        router.clone(),
        "POST",
        "/api/agents",
        Some(&token),
        Some(json!({ "name": "Scout", "role": "Finds weekend plans" })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(blank["soul_version"], 0);

    let (status, _) = request(
        router.clone(),
        "POST",
        "/api/agents",
        Some(&token),
        Some(json!({ "name": "Dana", "role": "x" })),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "a built-in's name");
    let (status, _) = request(
        router.clone(),
        "POST",
        "/api/agents",
        Some(&token),
        Some(json!({ "name": "penny", "role": "x" })),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "already theirs");
    let (status, _) = request(
        router.clone(),
        "POST",
        "/api/agents",
        Some(&token),
        Some(json!({ "name": "Nobody" })),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let (status, _) = request(
        router.clone(),
        "POST",
        "/api/agents",
        Some(&token),
        Some(json!({ "template": "nope" })),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let (status, _) = request(
        router.clone(),
        "POST",
        "/api/agents",
        Some(&token),
        Some(json!({ "name": "Tooly", "role": "x", "tools": ["not_a_tool"] })),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    let (status, _) = request(
        router.clone(),
        "DELETE",
        "/api/agents/money",
        Some(&token),
        None,
    )
    .await;
    assert_eq!(
        status,
        StatusCode::NOT_FOUND,
        "only one's own members are deleted"
    );
    let (status, _) = request(
        router.clone(),
        "DELETE",
        &format!("/api/agents/{agent_type}"),
        Some(&other),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (status, _) = request(
        router.clone(),
        "DELETE",
        &format!("/api/agents/{agent_type}"),
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (_, crew) = request(router, "GET", "/api/agents", Some(&token), None).await;
    assert!(member(&crew, &agent_type).is_none());
    let left: i64 =
        sqlx::query_scalar("SELECT count(*) FROM user_agent_soul_versions WHERE agent_type = $1")
            .bind(&agent_type)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(left, 0);
}

#[sqlx::test(migrations = "../../migrations")]
async fn admins_do_not_see_peoples_own_members(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    register(router.clone(), "crew-admin@example.com").await;
    sqlx::query("UPDATE users SET is_platform_admin = true WHERE id = (SELECT user_id FROM web_credentials WHERE email = $1)")
        .bind("crew-admin@example.com")
        .execute(&pool)
        .await
        .unwrap();
    let token = login(router.clone(), "crew-admin@example.com").await;
    let (status, _) = request(
        router.clone(),
        "POST",
        "/api/agents",
        Some(&token),
        Some(json!({ "template": "writing_buddy" })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let (status, body) = request(
        router,
        "GET",
        "/api/admin/dynamic-agents",
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body.as_array().unwrap().len(), 0);
}
