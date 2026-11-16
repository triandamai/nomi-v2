//! The catalog of apps, a person's MCP servers, and a connection's health.
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
        tool_catalog: std::sync::Arc::new(nomi_agent_core::ToolCatalog::empty()),
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

#[sqlx::test(migrations = "../../migrations")]
async fn the_catalog_marks_only_what_connects_today(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register(router.clone(), "catalog@example.com").await;
    let (status, body) = request(
        router.clone(),
        "GET",
        "/api/connections/catalog",
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let apps = body.as_array().unwrap();
    let hevy = apps.iter().find(|a| a["id"] == "hevy").unwrap();
    assert_eq!(hevy["available"], true);
    assert_eq!(hevy["for_agent"], "health");
    assert_eq!(hevy["category"], "health");
    let notion = apps.iter().find(|a| a["id"] == "notion").unwrap();
    assert_eq!(notion["available"], false);
    assert_eq!(notion["access"], json!(["read", "write"]));
    let (status, _) = request(router, "GET", "/api/connections/catalog", None, None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[sqlx::test(migrations = "../../migrations")]
async fn mcp_servers_are_kept_by_address_and_never_as_commands(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register(router.clone(), "mcp@example.com").await;
    let other = register(router.clone(), "mcp-other@example.com").await;

    let (status, body) = request(
        router.clone(),
        "GET",
        "/api/connections/mcp",
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    // Sora, who calls these servers, is on unless an admin switched her off.
    assert_eq!(body, json!({ "servers": [], "callable": true }));

    let (status, added) = request(
        router.clone(),
        "POST",
        "/api/connections/mcp",
        Some(&token),
        Some(json!({ "name": " Notes ", "url": "https://mcp.example.com/sse" })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(added["name"], "Notes");
    assert_eq!(added["url"], "https://mcp.example.com/sse");

    for (body, code) in [
        (
            json!({ "name": "Local", "command": "npx some-mcp-server" }),
            "command_not_supported",
        ),
        (
            json!({ "name": "", "url": "https://x.example.com" }),
            "missing_name",
        ),
        (json!({ "name": "No url" }), "missing_url"),
        (
            json!({ "name": "File", "url": "file:///etc/passwd" }),
            "invalid_url",
        ),
    ] {
        let (status, err) = request(
            router.clone(),
            "POST",
            "/api/connections/mcp",
            Some(&token),
            Some(body),
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(err["error"], code);
    }
    let (status, err) = request(
        router.clone(),
        "POST",
        "/api/connections/mcp",
        Some(&token),
        Some(json!({ "name": "Again", "url": "https://mcp.example.com/sse" })),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(err["error"], "already_added");

    // Someone else's list is their own, and they can't remove this one.
    let (_, theirs) = request(
        router.clone(),
        "GET",
        "/api/connections/mcp",
        Some(&other),
        None,
    )
    .await;
    assert_eq!(theirs["servers"], json!([]));
    let uri = format!("/api/connections/mcp/{}", added["id"].as_str().unwrap());
    let (status, _) = request(router.clone(), "DELETE", &uri, Some(&other), None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    let (status, _) = request(router.clone(), "DELETE", &uri, Some(&token), None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (_, body) = request(router, "GET", "/api/connections/mcp", Some(&token), None).await;
    assert_eq!(body["servers"], json!([]));
}

#[sqlx::test(migrations = "../../migrations")]
async fn too_many_mcp_servers_are_refused(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register(router.clone(), "mcp-many@example.com").await;
    for i in 0..nomi_server::routes::connections_catalog::MAX_MCP_SERVERS {
        let (status, _) = request(
            router.clone(),
            "POST",
            "/api/connections/mcp",
            Some(&token),
            Some(json!({ "name": "S", "url": format!("https://s{i}.example.com") })),
        )
        .await;
        assert_eq!(status, StatusCode::CREATED);
    }
    let (status, err) = request(
        router,
        "POST",
        "/api/connections/mcp",
        Some(&token),
        Some(json!({ "name": "S", "url": "https://one-more.example.com" })),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(err["error"], "too_many");
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_lapsed_connection_says_so_with_who_uses_it(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register(router.clone(), "health-state@example.com").await;
    let me: uuid::Uuid = sqlx::query_scalar("SELECT user_id FROM web_credentials WHERE email = $1")
        .bind("health-state@example.com")
        .fetch_one(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO workspace_connections (user_id, google_email, services, access_token_encrypted, expires_at, expired_at, last_used_at) VALUES ($1, 'a@x', '{gmail,drive}', $2, now(), now(), now())")
        .bind(me)
        .bind(nomi_settings::crypto::encrypt(&nomi_test_support::TEST_SETTINGS_KEY, "a"))
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO health_connections (user_id, provider, api_key_encrypted, account_name) VALUES ($1, 'hevy', $2, 'ana')")
        .bind(me)
        .bind(nomi_settings::crypto::encrypt(&nomi_test_support::TEST_SETTINGS_KEY, "k"))
        .execute(&pool)
        .await
        .unwrap();

    let (status, body) = request(
        router.clone(),
        "GET",
        "/api/connections",
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["google"]["status"], "expired");
    assert!(body["google"]["expired_at"].is_string());
    assert!(body["google"]["last_used_at"].is_string());
    assert_eq!(body["google"]["used_by"], json!(["workspace"]));
    assert_eq!(
        body["google"]["service_access"],
        json!([{ "service": "gmail", "access": ["read", "write", "ask"] }, { "service": "drive", "access": ["read"] }])
    );
    assert_eq!(body["hevy"]["status"], "ok");
    assert_eq!(body["hevy"]["used_by"], json!(["health"]));

    sqlx::query("INSERT INTO github_connections (user_id, login, kind, token_encrypted, expired_at) VALUES ($1, 'ana', 'token', $2, now())")
        .bind(me)
        .bind(nomi_settings::crypto::encrypt(&nomi_test_support::TEST_SETTINGS_KEY, "t"))
        .execute(&pool)
        .await
        .unwrap();
    let (_, github) = request(router, "GET", "/api/connections/github", Some(&token), None).await;
    assert_eq!(github["connection"]["status"], "expired");
    assert_eq!(github["connection"]["used_by"], json!([]));
}
