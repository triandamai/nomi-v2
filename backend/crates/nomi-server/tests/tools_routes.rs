
use axum::{body::Body, http::{Request, StatusCode}};
use http_body_util::BodyExt;
use nomi_server::app::{build_router, AppState};
use serde_json::{json, Value};
use sqlx::PgPool;
use tower::ServiceExt;

use nomi_test_support::TEST_SETTINGS_KEY;

const SECRET: &str = "test-secret-do-not-use-in-prod";

fn test_state(pool: PgPool) -> AppState {
    AppState {
        pool,
        jwt_secret: SECRET.to_string(),
        http_client: reqwest::Client::new(),
        settings_key: TEST_SETTINGS_KEY,
        mqtt_broker_host: nomi_test_support::TEST_MQTT_BROKER_HOST.to_string(),
        mqtt_broker_port: nomi_test_support::TEST_MQTT_BROKER_PORT,
        s3: None,
        project_storage: nomi_test_support::test_project_storage(),
        tool_catalog: std::sync::Arc::new(nomi_agent_core::ToolCatalog::empty()),
        email_codes: nomi_server::sign_in_codes::EmailCodes::Off,
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

/// Registers, promotes to platform admin, then logs in — login must happen
/// *after* promotion, since permissions are baked into the JWT at login time.
async fn register_admin_and_login(router: axum::Router, pool: &PgPool, email: &str) -> String {
    register_via_api(router.clone(), email).await;
    make_platform_admin(pool, email).await;
    login_via_api(router, email).await
}

fn find_tool<'a>(body: &'a Value, name: &str) -> &'a Value {
    body["groups"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|g| g["tools"].as_array().unwrap().iter())
        .find(|t| t["name"] == name)
        .unwrap_or_else(|| panic!("{name} is listed"))
}

#[sqlx::test(migrations = "../../migrations")]
async fn non_admin_cannot_see_or_change_tools(pool: PgPool) {
    let router = build_router(test_state(pool));
    register_via_api(router.clone(), "regular-tools@example.com").await;
    let token = login_via_api(router.clone(), "regular-tools@example.com").await;

    let (status, _) = json_request(router.clone(), "GET", "/api/admin/tools", Value::Null, Some(&token)).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    let (status, _) = json_request(router, "PUT", "/api/admin/tools/web_search", json!({"enabled": false}), Some(&token)).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}

#[sqlx::test(migrations = "../../migrations")]
async fn every_tool_is_listed_once_with_its_switch(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register_admin_and_login(router.clone(), &pool, "tools-admin@example.com").await;

    let (status, body) = json_request(router.clone(), "GET", "/api/admin/tools", Value::Null, Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["groups"][0]["key"], "crew");
    let switchable: Vec<&str> = body["groups"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|g| g["tools"].as_array().unwrap().iter())
        .filter(|t| t["agent_only"] == false)
        .map(|t| t["name"].as_str().unwrap())
        .collect();
    let unique: std::collections::HashSet<&&str> = switchable.iter().collect();
    assert_eq!(unique.len(), switchable.len(), "each switch is listed once");
    assert!(body["groups"].as_array().unwrap().iter().any(|g| g["kind"] == "agent"));

    let complete = find_tool(&body, "complete_task");
    assert_eq!(complete["required"], true);
    let search = find_tool(&body, "web_search");
    assert_eq!(search["enabled"], true);
    assert_eq!(search["configurable"], true);
    assert_eq!(search["settings"]["provider"], "tavily", "Tavily is the default");

    let (status, _) = json_request(router.clone(), "PUT", "/api/admin/tools/read_web_page", json!({"enabled": false}), Some(&token)).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (_, body) = json_request(router.clone(), "GET", "/api/admin/tools", Value::Null, Some(&token)).await;
    assert_eq!(find_tool(&body, "read_web_page")["enabled"], false);

    let (status, _) = json_request(router.clone(), "PUT", "/api/admin/tools/complete_task", json!({"enabled": false}), Some(&token)).await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "required tools stay on");
    let (status, _) = json_request(router, "PUT", "/api/admin/tools/no_such_tool", json!({"enabled": false}), Some(&token)).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[sqlx::test(migrations = "../../migrations")]
async fn an_agents_own_tools_are_shown_but_not_switchable(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register_admin_and_login(router.clone(), &pool, "tools-agent@example.com").await;

    let (_, body) = json_request(router.clone(), "GET", "/api/admin/tools", Value::Null, Some(&token)).await;
    let money = body["groups"].as_array().unwrap().iter().find(|g| g["key"] == "money").expect("money's tools are listed");
    assert_eq!(money["kind"], "agent");
    let tool = &money["tools"][0];
    assert_eq!(tool["agent_only"], true);
    assert_eq!(tool["enabled"], true);
    assert_eq!(find_tool(&body, "web_search")["agent_only"], false);

    let uri = format!("/api/admin/tools/{}", tool["name"].as_str().unwrap());
    let (status, _) = json_request(router, "PUT", &uri, json!({"enabled": false}), Some(&token)).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let off: i64 = sqlx::query_scalar("SELECT count(*) FROM tool_settings").fetch_one(&pool).await.unwrap();
    assert_eq!(off, 0);
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_saved_search_key_is_kept_secret_and_kept_when_left_blank(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register_admin_and_login(router.clone(), &pool, "tools-key@example.com").await;

    let (status, _) = json_request(
        router.clone(),
        "PUT",
        "/api/admin/tools/web_search/settings",
        json!({"provider": "serper", "max_results": 50, "api_key": "serper-secret-123"}),
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (_, body) = json_request(router.clone(), "GET", "/api/admin/tools", Value::Null, Some(&token)).await;
    assert!(!body.to_string().contains("serper-secret-123"), "the key never goes back to the browser");
    let search = find_tool(&body, "web_search");
    assert_eq!(search["settings"]["provider"], "serper");
    assert_eq!(search["settings"]["max_results"], 10, "clamped to the limit");
    assert_eq!(search["ready"], true);
    let serper = search["settings"]["providers"].as_array().unwrap().iter().find(|p| p["id"] == "serper").unwrap();
    assert_eq!(serper["key_source"], "saved");

    // Saving again without a key keeps it.
    json_request(router.clone(), "PUT", "/api/admin/tools/web_search/settings", json!({"provider": "serper", "max_results": 4}), Some(&token)).await;
    let (_, body) = json_request(router.clone(), "GET", "/api/admin/tools", Value::Null, Some(&token)).await;
    assert_eq!(find_tool(&body, "web_search")["ready"], true);

    let (status, _) = json_request(
        router.clone(),
        "PUT",
        "/api/admin/tools/web_search/settings",
        json!({"provider": "searxng", "base_url": "not a url"}),
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let (status, _) =
        json_request(router, "PUT", "/api/admin/tools/web_search/settings", json!({"provider": "google"}), Some(&token)).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[sqlx::test(migrations = "../../migrations")]
async fn testing_search_without_a_key_says_what_is_missing(pool: PgPool) {
    std::env::remove_var("TAVILY_API_KEY");
    let router = build_router(test_state(pool.clone()));
    let token = register_admin_and_login(router.clone(), &pool, "tools-test@example.com").await;

    let (status, body) = json_request(router, "POST", "/api/admin/tools/web_search/test", json!({"query": "hi"}), Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["ok"], false);
    assert!(body["error"].as_str().unwrap().contains("API key"));
}
