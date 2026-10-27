//! Admin → Configuration: only people given the area see it, only its managers change it, and
//! a secret is never sent back.
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

async fn request(router: axum::Router, method: &str, uri: &str, token: &str, body: Option<Value>) -> (StatusCode, Value) {
    let mut builder = Request::builder().method(method).uri(uri).header("authorization", format!("Bearer {token}"));
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
    (status, if bytes.is_empty() { Value::Null } else { serde_json::from_slice(&bytes).unwrap_or(Value::Null) })
}

async fn login(router: axum::Router, pool: &PgPool, email: &str, admin: bool) -> String {
    let register = Request::builder()
        .method("POST")
        .uri("/api/auth/register")
        .header("content-type", "application/json")
        .body(Body::from(json!({ "email": email, "password": "correct-password", "org": { "mode": "create", "name": "Acme" } }).to_string()))
        .unwrap();
    router.clone().oneshot(register).await.unwrap();
    if admin {
        sqlx::query("UPDATE users SET is_platform_admin = true WHERE id = (SELECT user_id FROM web_credentials WHERE email = $1)")
            .bind(email)
            .execute(pool)
            .await
            .unwrap();
    }
    let login = Request::builder()
        .method("POST")
        .uri("/api/auth/login")
        .header("content-type", "application/json")
        .body(Body::from(json!({ "email": email, "password": "correct-password" }).to_string()))
        .unwrap();
    let response = router.oneshot(login).await.unwrap();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body: Value = serde_json::from_slice(&bytes).unwrap();
    body["access_token"].as_str().unwrap().to_string()
}

async fn grant(pool: &PgPool, email: &str, actions: &[&str]) {
    sqlx::query(
        "INSERT INTO user_permissions (user_id, scope_type, resource, actions) \
         SELECT user_id, 'admin', 'configuration', $2 FROM web_credentials WHERE email = $1",
    )
    .bind(email)
    .bind(actions.iter().map(|a| a.to_string()).collect::<Vec<_>>())
    .execute(pool)
    .await
    .unwrap();
}

fn section<'a>(body: &'a Value, name: &str) -> &'a Value {
    body["sections"].as_array().unwrap().iter().find(|s| s["name"] == name).unwrap()
}

fn field<'a>(section: &'a Value, name: &str) -> &'a Value {
    section["fields"].as_array().unwrap().iter().find(|f| f["name"] == name).unwrap()
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_manager_saves_a_section_and_secrets_never_come_back(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = login(router.clone(), &pool, "root@nomi.test", true).await;

    let (status, saved) = request(
        router.clone(),
        "PUT",
        "/api/admin/configuration/smtp",
        &token,
        Some(json!({ "values": { "host": "smtp.example.com", "port": "587", "password": "hunter2" } })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{saved}");

    let (status, body) = request(router.clone(), "GET", "/api/admin/configuration", &token, None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["can_manage"], true);
    let smtp = section(&body, "smtp");
    assert_eq!(field(smtp, "host")["value"], "smtp.example.com");
    assert_eq!(field(smtp, "host")["source"], "saved");
    assert_eq!(field(smtp, "password")["value"], Value::Null);
    assert_eq!(field(smtp, "password")["set"], true);
    assert_eq!(field(smtp, "password")["env"], "SMTP_PASSWORD");
    assert!(!body.to_string().contains("hunter2"));
    for name in ["attachments", "storage", "google", "smtp", "agent_loop"] {
        section(&body, name);
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn bad_values_are_refused_with_the_reason(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = login(router.clone(), &pool, "root@nomi.test", true).await;

    let (status, body) = request(router.clone(), "PUT", "/api/admin/configuration/agent_loop", &token, Some(json!({ "values": { "turn_concurrency": "lots" } }))).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(body.to_string().contains("turn_concurrency"), "{body}");

    // A bucket needs both keys, saved here or in the environment.
    let (status, body) = request(router.clone(), "PUT", "/api/admin/configuration/storage", &token, Some(json!({ "values": { "bucket": "nomi-files" } }))).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(body.to_string().contains("access key"), "{body}");

    let (status, _) = request(router.clone(), "PUT", "/api/admin/configuration/nope", &token, Some(json!({ "values": {} }))).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[sqlx::test(migrations = "../../migrations")]
async fn only_people_given_configuration_see_it_and_only_managers_change_it(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let nobody = login(router.clone(), &pool, "nobody@nomi.test", false).await;
    assert_eq!(request(router.clone(), "GET", "/api/admin/configuration", &nobody, None).await.0, StatusCode::FORBIDDEN);

    login(router.clone(), &pool, "viewer@nomi.test", false).await;
    grant(&pool, "viewer@nomi.test", &["view"]).await;
    let viewer = login(router.clone(), &pool, "viewer@nomi.test", false).await;
    let (status, body) = request(router.clone(), "GET", "/api/admin/configuration", &viewer, None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["can_manage"], false);
    let change = Some(json!({ "values": { "max_files": "20" } }));
    assert_eq!(request(router.clone(), "PUT", "/api/admin/configuration/attachments", &viewer, change).await.0, StatusCode::FORBIDDEN);
}
