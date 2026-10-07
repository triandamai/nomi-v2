
use axum::{body::Body, http::{Request, StatusCode}};
use http_body_util::BodyExt;
use nomi_server::app::{build_router, AppState};
use serde_json::{json, Value};
use sqlx::PgPool;
use tower::ServiceExt;

use std::sync::Arc;

use nomi_mail::MemoryMailer;
use nomi_server::sign_in_codes::EmailCodes;
use nomi_test_support::TEST_SETTINGS_KEY;

const SECRET: &str = "test-secret-do-not-use-in-prod";

fn test_state(pool: PgPool, mailer: Arc<MemoryMailer>) -> AppState {
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
        email_codes: EmailCodes::Required(mailer),
    }
}

async fn call(router: axum::Router, method: &str, uri: &str, body: Value) -> (StatusCode, Value) {
    let request = Request::builder().method(method).uri(uri).header("content-type", "application/json").body(Body::from(body.to_string())).unwrap();
    let response = router.oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    (status, serde_json::from_slice(&bytes).unwrap_or(Value::Null))
}

/// The code in the last email sent to `to`.
fn code_sent_to(mailer: &MemoryMailer, to: &str) -> String {
    let email = mailer.last_to(to).expect("an email was sent");
    email.subject.rsplit(' ').next().unwrap().to_string()
}

fn setup(pool: PgPool) -> (axum::Router, Arc<MemoryMailer>) {
    let mailer = Arc::new(MemoryMailer::default());
    (build_router(test_state(pool, mailer.clone())), mailer)
}

async fn register(router: &axum::Router, email: &str, language: &str) -> (StatusCode, Value) {
    call(
        router.clone(),
        "POST",
        "/api/auth/register",
        json!({ "email": email, "password": "correct-password", "org": { "mode": "create", "name": "Acme" }, "language": language }),
    )
    .await
}

async fn login(router: &axum::Router, email: &str, password: &str) -> (StatusCode, Value) {
    call(router.clone(), "POST", "/api/auth/login", json!({ "email": email, "password": password })).await
}

async fn verify(router: &axum::Router, challenge: &Value, code: &str) -> (StatusCode, Value) {
    call(router.clone(), "POST", "/api/auth/verify", json!({ "challenge_id": challenge, "code": code })).await
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_new_account_confirms_its_email_with_a_code_before_it_is_signed_in(pool: PgPool) {
    let (router, mailer) = setup(pool.clone());
    let (status, body) = register(&router, "new@example.com", "id").await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.get("access_token").is_none(), "no tokens before the code");
    let v = &body["verification"];
    assert_eq!(v["purpose"], "register");
    assert_eq!(v["email"], "n**@example.com");
    assert!(v["expires_in"].as_i64().unwrap() > 500);

    let email = mailer.last_to("new@example.com").unwrap();
    assert!(email.subject.starts_with("Konfirmasi"), "in the sign-up page's language: {}", email.subject);
    let verified: Option<chrono::DateTime<chrono::Utc>> =
        sqlx::query_scalar("SELECT email_verified_at FROM web_credentials WHERE email = 'new@example.com'").fetch_one(&pool).await.unwrap();
    assert!(verified.is_none());

    let code = code_sent_to(&mailer, "new@example.com");
    let (status, tokens) = verify(&router, &v["challenge_id"], &code).await;
    assert_eq!(status, StatusCode::OK);
    assert!(tokens["access_token"].is_string() && tokens["refresh_token"].is_string());
    let verified: Option<chrono::DateTime<chrono::Utc>> =
        sqlx::query_scalar("SELECT email_verified_at FROM web_credentials WHERE email = 'new@example.com'").fetch_one(&pool).await.unwrap();
    assert!(verified.is_some());

    let (status, body) = verify(&router, &v["challenge_id"], &code).await;
    assert_eq!(status, StatusCode::GONE, "a code works once");
    assert_eq!(body["error"], "expired");
}

#[sqlx::test(migrations = "../../migrations")]
async fn every_password_sign_in_waits_for_the_emailed_code(pool: PgPool) {
    let (router, mailer) = setup(pool);
    let (_, body) = register(&router, "ana@example.com", "en").await;
    verify(&router, &body["verification"]["challenge_id"], &code_sent_to(&mailer, "ana@example.com")).await;

    let (status, body) = login(&router, "ana@example.com", "wrong-password").await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"], "invalid credentials");
    assert_eq!(mailer.sent().len(), 1, "no code for a wrong password");

    let (status, body) = login(&router, "ana@example.com", "correct-password").await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.get("access_token").is_none());
    let challenge = &body["verification"]["challenge_id"];
    assert_eq!(body["verification"]["purpose"], "login");
    let email = mailer.last_to("ana@example.com").unwrap();
    assert!(email.subject.starts_with("Your Nomi sign-in code"));

    let code = code_sent_to(&mailer, "ana@example.com");
    let wrong = if code == "000000" { "111111" } else { "000000" };
    let (status, body) = verify(&router, challenge, wrong).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"], "wrong_code");
    assert_eq!(body["attempts_left"], 4);

    let (status, tokens) = verify(&router, challenge, &format!("{} {}", &code[..3], &code[3..])).await;
    assert_eq!(status, StatusCode::OK, "spaces in the code are fine");
    assert!(tokens["access_token"].is_string());
}

#[sqlx::test(migrations = "../../migrations")]
async fn five_wrong_codes_end_the_sign_in(pool: PgPool) {
    let (router, mailer) = setup(pool);
    register(&router, "ana@example.com", "en").await;
    let (_, body) = login(&router, "ana@example.com", "correct-password").await;
    let challenge = body["verification"]["challenge_id"].clone();
    let code = code_sent_to(&mailer, "ana@example.com");
    let wrong = if code == "000000" { "111111" } else { "000000" };
    for _ in 0..4 {
        assert_eq!(verify(&router, &challenge, wrong).await.0, StatusCode::BAD_REQUEST);
    }
    let (status, body) = verify(&router, &challenge, wrong).await;
    assert_eq!((status, body["error"].as_str()), (StatusCode::GONE, Some("too_many_attempts")));
    let (status, _) = verify(&router, &challenge, &code).await;
    assert_eq!(status, StatusCode::GONE, "even the right code no longer works");
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_new_code_waits_a_minute_and_replaces_the_old_one(pool: PgPool) {
    let (router, mailer) = setup(pool.clone());
    register(&router, "ana@example.com", "en").await;
    let (_, body) = login(&router, "ana@example.com", "correct-password").await;
    let challenge = body["verification"]["challenge_id"].clone();
    let first = code_sent_to(&mailer, "ana@example.com");

    let (status, body) = call(router.clone(), "POST", "/api/auth/resend", json!({ "challenge_id": challenge })).await;
    assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(body["error"], "wait");
    assert!(body["seconds"].as_i64().unwrap() > 0);

    sqlx::query("UPDATE sign_in_challenges SET last_sent_at = now() - interval '2 minutes'").execute(&pool).await.unwrap();
    let (status, body) = call(router.clone(), "POST", "/api/auth/resend", json!({ "challenge_id": challenge })).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["challenge_id"], challenge);
    assert!(body["resend_in"].as_i64().unwrap() > 50);
    let second = code_sent_to(&mailer, "ana@example.com");

    if first != second {
        assert_eq!(verify(&router, &challenge, &first).await.0, StatusCode::BAD_REQUEST, "the old code stops working");
    }
    let (status, info) = call(router.clone(), "GET", &format!("/api/auth/challenge/{}", challenge.as_str().unwrap()), Value::Null).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(info["email"], "a**@example.com");
    assert_eq!(verify(&router, &challenge, &second).await.0, StatusCode::OK);
}

#[sqlx::test(migrations = "../../migrations")]
async fn signing_in_again_retires_the_earlier_code(pool: PgPool) {
    let (router, mailer) = setup(pool);
    register(&router, "ana@example.com", "en").await;
    let (_, first) = login(&router, "ana@example.com", "correct-password").await;
    let first_code = code_sent_to(&mailer, "ana@example.com");
    let (_, second) = login(&router, "ana@example.com", "correct-password").await;
    assert_ne!(first["verification"]["challenge_id"], second["verification"]["challenge_id"]);
    let (status, _) = verify(&router, &first["verification"]["challenge_id"], &first_code).await;
    assert_eq!(status, StatusCode::GONE);
    let (status, _) = call(router, "GET", &format!("/api/auth/challenge/{}", first["verification"]["challenge_id"].as_str().unwrap()), Value::Null).await;
    assert_eq!(status, StatusCode::GONE);
}
