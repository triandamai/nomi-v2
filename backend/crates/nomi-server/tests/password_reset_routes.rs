use axum::{body::Body, http::{Request, StatusCode}};
use http_body_util::BodyExt;
use nomi_mail::MemoryMailer;
use nomi_server::app::{build_router, AppState};
use nomi_server::sign_in_codes::EmailCodes;
use nomi_test_support::TEST_SETTINGS_KEY;
use serde_json::{json, Value};
use sqlx::PgPool;
use std::sync::Arc;
use tower::ServiceExt;

const SECRET: &str = "test-secret-do-not-use-in-prod";
const APP_URL: &str = "https://nomi.test";

fn setup(pool: PgPool) -> (axum::Router, Arc<MemoryMailer>) {
    std::env::set_var("APP_URL", APP_URL);
    let mailer = Arc::new(MemoryMailer::default());
    let state = AppState {
        pool,
        jwt_secret: SECRET.to_string(),
        http_client: reqwest::Client::new(),
        settings_key: TEST_SETTINGS_KEY,
        mqtt_broker_host: nomi_test_support::TEST_MQTT_BROKER_HOST.to_string(),
        mqtt_broker_port: nomi_test_support::TEST_MQTT_BROKER_PORT,
        s3: None,
        project_storage: nomi_test_support::test_project_storage(),
        tool_catalog: Arc::new(nomi_agent_core::ToolCatalog::empty()),
        email_codes: EmailCodes::Off,
        push: Arc::new(nomi_server::push::NoopPush),
        shipyard: None,
    };
    // Reset links go out through the state's mailer when codes are required; with codes off they
    // use the configured one, so these tests turn codes on to capture the email.
    let state = AppState { email_codes: EmailCodes::Required(mailer.clone()), ..state };
    (build_router(state), mailer)
}

async fn call(router: &axum::Router, method: &str, uri: &str, body: Value) -> (StatusCode, Value) {
    let request = Request::builder().method(method).uri(uri).header("content-type", "application/json").body(Body::from(body.to_string())).unwrap();
    let response = router.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    (status, serde_json::from_slice(&bytes).unwrap_or(Value::Null))
}

async fn account(pool: &PgPool, router: &axum::Router, email: &str) {
    let (status, _) = call(router, "POST", "/api/auth/register", json!({ "email": email, "password": "old-password", "org": { "mode": "create", "name": "Acme" } })).await;
    assert_eq!(status, StatusCode::OK);
    sqlx::query("UPDATE sign_in_challenges SET used_at = now()").execute(pool).await.unwrap();
}

/// The token in the link of the last email sent to `to`.
fn token_sent_to(mailer: &MemoryMailer, to: &str) -> String {
    let email = mailer.last_to(to).expect("a reset email was sent");
    assert!(email.subject.starts_with("Reset your Nomi password"));
    let after = email.text.split(&format!("{APP_URL}/reset-password?token=")).nth(1).expect("the link is in the email");
    after.chars().take_while(char::is_ascii_hexdigit).collect()
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_reset_link_chooses_a_new_password_once(pool: PgPool) {
    let (router, mailer) = setup(pool.clone());
    account(&pool, &router, "ana@example.com").await;
    let sent_before = mailer.sent().len();

    let (status, _) = call(&router, "POST", "/api/auth/forgot", json!({ "email": "Ana@Example.com", "language": "en" })).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    assert_eq!(mailer.sent().len(), sent_before + 1);
    let token = token_sent_to(&mailer, "ana@example.com");

    assert_eq!(call(&router, "GET", &format!("/api/auth/reset/{token}"), Value::Null).await.0, StatusCode::NO_CONTENT);
    let (status, body) = call(&router, "POST", "/api/auth/reset", json!({ "token": token, "password": "short" })).await;
    assert_eq!((status, body["error"].as_str()), (StatusCode::BAD_REQUEST, Some("too_short")));

    let (status, _) = call(&router, "POST", "/api/auth/reset", json!({ "token": token, "password": "brand-new-password" })).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (status, body) = call(&router, "POST", "/api/auth/reset", json!({ "token": token, "password": "another-password" })).await;
    assert_eq!((status, body["error"].as_str()), (StatusCode::GONE, Some("expired")), "a link works once");
    assert_eq!(call(&router, "GET", &format!("/api/auth/reset/{token}"), Value::Null).await.0, StatusCode::GONE);

    let (status, _) = call(&router, "POST", "/api/auth/login", json!({ "email": "ana@example.com", "password": "old-password" })).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED, "the old password is gone");
    let (status, _) = call(&router, "POST", "/api/auth/login", json!({ "email": "ana@example.com", "password": "brand-new-password" })).await;
    assert_eq!(status, StatusCode::OK);
}

#[sqlx::test(migrations = "../../migrations")]
async fn asking_for_an_unknown_email_looks_the_same_and_sends_nothing(pool: PgPool) {
    let (router, mailer) = setup(pool);
    let (status, _) = call(&router, "POST", "/api/auth/forgot", json!({ "email": "nobody@example.com" })).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    assert!(mailer.sent().is_empty());
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_newer_link_replaces_the_older_one_and_resetting_signs_out_everywhere(pool: PgPool) {
    let (router, mailer) = setup(pool.clone());
    account(&pool, &router, "ana@example.com").await;
    call(&router, "POST", "/api/auth/forgot", json!({ "email": "ana@example.com" })).await;
    let first = token_sent_to(&mailer, "ana@example.com");
    call(&router, "POST", "/api/auth/forgot", json!({ "email": "ana@example.com" })).await;
    let second = token_sent_to(&mailer, "ana@example.com");
    assert_ne!(first, second);
    assert_eq!(call(&router, "GET", &format!("/api/auth/reset/{first}"), Value::Null).await.0, StatusCode::GONE);

    sqlx::query("INSERT INTO refresh_tokens (user_id, token_hash, expires_at) SELECT user_id, 'h', now() + interval '1 day' FROM web_credentials")
        .execute(&pool)
        .await
        .unwrap();
    let (status, _) = call(&router, "POST", "/api/auth/reset", json!({ "token": second, "password": "brand-new-password" })).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let open: i64 = sqlx::query_scalar("SELECT count(*) FROM refresh_tokens WHERE revoked_at IS NULL").fetch_one(&pool).await.unwrap();
    assert_eq!(open, 0);
}

#[sqlx::test(migrations = "../../migrations")]
async fn an_expired_link_is_refused(pool: PgPool) {
    let (router, mailer) = setup(pool.clone());
    account(&pool, &router, "ana@example.com").await;
    call(&router, "POST", "/api/auth/forgot", json!({ "email": "ana@example.com" })).await;
    let token = token_sent_to(&mailer, "ana@example.com");
    sqlx::query("UPDATE password_resets SET expires_at = now() - interval '1 minute'").execute(&pool).await.unwrap();
    let (status, _) = call(&router, "POST", "/api/auth/reset", json!({ "token": token, "password": "brand-new-password" })).await;
    assert_eq!(status, StatusCode::GONE);
}
