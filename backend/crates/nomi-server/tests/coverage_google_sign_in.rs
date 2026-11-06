//! Sign in with Google through the routes, against a fake Google (GOOGLE_FAKE_BASE_URL). Its own
//! test binary: the OAuth client comes from the environment, read once per process.

mod support;

use axum::http::StatusCode;
use serde_json::json;
use sqlx::PgPool;
use support::App;
use wiremock::matchers::{body_string_contains, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn state_of(url: &str) -> String {
    reqwest::Url::parse(url).unwrap().query_pairs().find(|(k, _)| k == "state").unwrap().1.to_string()
}

async fn google(server: &MockServer, code: &str, sub: &str, email: &str, verified: bool) {
    Mock::given(method("POST"))
        .and(path("/token"))
        .and(body_string_contains(format!("code={code}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"access_token": format!("at-{code}"), "expires_in": 3600, "scope": "openid email"})))
        .mount(server)
        .await;
    Mock::given(method("GET"))
        .and(path("/v1/userinfo"))
        .and(wiremock::matchers::header("authorization", format!("Bearer at-{code}").as_str()))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"sub": sub, "email": email, "email_verified": verified, "name": "Ana K", "given_name": "Ana"})))
        .mount(server)
        .await;
}

#[sqlx::test(migrations = "../../migrations")]
async fn google_signs_people_up_in_links_and_unlinks(pool: PgPool) {
    let server = MockServer::start().await;
    std::env::set_var("GOOGLE_CLIENT_ID", "client");
    std::env::set_var("GOOGLE_CLIENT_SECRET", "secret");
    std::env::set_var("GOOGLE_REDIRECT_URI", "http://app.test/connections/google/callback");
    std::env::set_var("GOOGLE_FAKE_BASE_URL", server.uri());
    let app = App::new(pool.clone());

    let (status, available) = app.call("GET", "/api/auth/google/available", None, None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(available["available"], true);
    assert_eq!(available["sign_in_redirect_uri"], "http://app.test/auth/google/callback");

    // A new person: signing in creates their account.
    google(&server, "new-code", "g-ana", "ana@gmail.test", true).await;
    let (status, started) = app.call("POST", "/api/auth/google/start", None, Some(json!({}))).await;
    assert_eq!(status, StatusCode::OK, "{started}");
    let url = started["url"].as_str().unwrap();
    assert!(url.starts_with(&server.uri()));
    let (status, signed_in) = app.call("POST", "/api/auth/google/callback", None, Some(json!({"code": "new-code", "state": state_of(url)}))).await;
    assert_eq!(status, StatusCode::OK, "{signed_in}");
    assert_eq!(signed_in["is_new"], true);
    assert_eq!(signed_in["email"], "ana@gmail.test");
    let token = signed_in["access_token"].as_str().unwrap().to_string();

    // The same state can't be used twice.
    let (status, _) = app.call("POST", "/api/auth/google/callback", None, Some(json!({"code": "new-code", "state": state_of(url)}))).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    let (status, methods) = app.call("GET", "/api/auth/google", Some(&token), None).await;
    assert_eq!(status, StatusCode::OK, "{methods}");
    assert_eq!(methods["configured"], true);
    // Google is the only way in: it can't be removed.
    assert_eq!(app.call("DELETE", "/api/auth/google", Some(&token), None).await.0, StatusCode::CONFLICT);

    // Someone with a password adds Google, then removes it.
    let ben = app.person("ben@acme.test").await;
    google(&server, "link-code", "g-ben", "ben@gmail.test", true).await;
    let (status, started) = app.call("POST", "/api/auth/google/link", Some(&ben.token), None).await;
    assert_eq!(status, StatusCode::OK);
    let (status, linked) = app.call("POST", "/api/auth/google/callback", None, Some(json!({"code": "link-code", "state": state_of(started["url"].as_str().unwrap())}))).await;
    assert_eq!(status, StatusCode::OK, "{linked}");
    assert_eq!(linked["linked"], true);
    assert!(linked["access_token"].is_null());
    assert_eq!(app.call("DELETE", "/api/auth/google", Some(&ben.token), None).await.0, StatusCode::NO_CONTENT);

    // A Google account already tied to someone else can't be linked again.
    google(&server, "taken-code", "g-ana", "ana@gmail.test", true).await;
    let (_, started) = app.call("POST", "/api/auth/google/link", Some(&ben.token), None).await;
    let (status, _) = app.call("POST", "/api/auth/google/callback", None, Some(json!({"code": "taken-code", "state": state_of(started["url"].as_str().unwrap())}))).await;
    assert_eq!(status, StatusCode::CONFLICT);

    // An unverified email is refused; a code Google won't take is refused.
    google(&server, "unverified", "g-eve", "eve@gmail.test", false).await;
    let (_, started) = app.call("POST", "/api/auth/google/start", None, None).await;
    assert_eq!(app.call("POST", "/api/auth/google/callback", None, Some(json!({"code": "unverified", "state": state_of(started["url"].as_str().unwrap())}))).await.0, StatusCode::BAD_REQUEST);
    let (_, started) = app.call("POST", "/api/auth/google/start", None, Some(json!({"invite_code": "  "}))).await;
    let (status, _) = app.call("POST", "/api/auth/google/callback", None, Some(json!({"code": "unknown", "state": state_of(started["url"].as_str().unwrap())}))).await;
    assert!(status.is_client_error() || status.is_server_error());
}
