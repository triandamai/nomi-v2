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
    std::env::set_var("GOOGLE_APP_ORIGINS", "http://new.app.test, http://localhost:5174");
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

    workspace_connection(&app, &server).await;
    callbacks_follow_the_app_started_on(&app, &server).await;
}

fn redirect_of(url: &str) -> String {
    reqwest::Url::parse(url).unwrap().query_pairs().find(|(k, _)| k == "redirect_uri").unwrap().1.to_string()
}

/// A flow started on the new app comes back to the new app (and trades its code with that same
/// address); one started on the app, or with no origin, comes back to the configured page.
async fn callbacks_follow_the_app_started_on(app: &App, google_at: &MockServer) {
    let start = |origin: Option<&str>| json!({ "origin": origin });
    for (origin, expected) in [
        (None, "http://app.test/auth/google/callback"),
        (Some("http://app.test"), "http://app.test/auth/google/callback"),
        (Some("http://new.app.test/"), "http://new.app.test/auth/google/callback"),
        (Some("http://localhost:5174"), "http://localhost:5174/auth/google/callback"),
    ] {
        let (status, started) = app.call("POST", "/api/auth/google/start", None, Some(start(origin))).await;
        assert_eq!(status, StatusCode::OK, "{origin:?}: {started}");
        assert_eq!(redirect_of(started["url"].as_str().unwrap()), expected, "{origin:?}");
    }
    // Anywhere else is refused, not quietly sent to the configured page.
    for origin in ["http://evil.test", "javascript:alert(1)", "http://new.app.test:8080"] {
        assert_eq!(app.call("POST", "/api/auth/google/start", None, Some(start(Some(origin)))).await.0, StatusCode::BAD_REQUEST, "{origin}");
    }

    // The code is traded with the address the flow was sent to.
    Mock::given(method("POST"))
        .and(path("/token"))
        .and(body_string_contains("code=new-app-code"))
        .and(body_string_contains("redirect_uri=http%3A%2F%2Fnew.app.test%2Fauth%2Fgoogle%2Fcallback"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"access_token": "at-new-app-code", "expires_in": 3600, "scope": "openid email"})))
        .mount(google_at)
        .await;
    let (_, started) = app.call("POST", "/api/auth/google/start", None, Some(start(Some("http://new.app.test")))).await;
    let (status, signed_in) = app.call("POST", "/api/auth/google/callback", None, Some(json!({"code": "new-app-code", "state": state_of(started["url"].as_str().unwrap())}))).await;
    // Only a token call with the new app's address is answered, so OK means it was sent.
    assert_eq!(status, StatusCode::OK, "{signed_in}");

    // Adding Google from Settings, and connecting Workspace, follow the app too.
    let cy = app.person("cy@acme.test").await;
    let (status, started) = app.call("POST", "/api/auth/google/link", Some(&cy.token), Some(start(Some("http://new.app.test")))).await;
    assert_eq!(status, StatusCode::OK, "{started}");
    assert_eq!(redirect_of(started["url"].as_str().unwrap()), "http://new.app.test/auth/google/callback");
    assert_eq!(app.call("POST", "/api/auth/google/link", Some(&cy.token), Some(start(Some("http://evil.test")))).await.0, StatusCode::BAD_REQUEST);
    let (status, started) = app.send("POST", "/api/connections/google/start", &cy, json!({"services": ["gmail"], "origin": "http://new.app.test"})).await;
    assert_eq!(status, StatusCode::OK, "{started}");
    assert_eq!(redirect_of(started["url"].as_str().unwrap()), "http://new.app.test/connections/google/callback");
    let (status, started) = app.send("POST", "/api/connections/google/start", &cy, json!({"services": ["gmail"]})).await;
    assert_eq!(status, StatusCode::OK, "{started}");
    assert_eq!(redirect_of(started["url"].as_str().unwrap()), "http://app.test/connections/google/callback");
    assert_eq!(app.send("POST", "/api/connections/google/start", &cy, json!({"services": ["gmail"], "origin": "http://evil.test"})).await.0, StatusCode::BAD_REQUEST);
}

/// Connecting Google for the Workspace agent, on the same fake Google (the OAuth client is
/// read once per process, so it runs inside the sign-in test).
async fn workspace_connection(app: &App, google_at: &MockServer) {
    let ana = app.person("ana@acme.test").await;

    let (status, before) = app.get("/api/connections/google", &ana).await;
    assert_eq!(status, StatusCode::OK, "{before}");
    assert_eq!(before["configured"], true);
    assert!(before["email"].is_null());
    assert_eq!(app.send("POST", "/api/connections/google/start", &ana, json!({"services": []})).await.0, StatusCode::BAD_REQUEST);

    let (status, started) = app.send("POST", "/api/connections/google/start", &ana, json!({"services": ["gmail", "sheets"]})).await;
    assert_eq!(status, StatusCode::OK, "{started}");
    let state = state_of(started["url"].as_str().unwrap());

    Mock::given(method("POST"))
        .and(path("/token"))
        .and(body_string_contains("code=ws-code"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"access_token": "ws-token", "expires_in": 3600, "refresh_token": "ws-refresh", "scope": "openid email https://www.googleapis.com/auth/gmail.modify"})))
        .mount(google_at)
        .await;
    Mock::given(method("GET")).and(path("/v1/userinfo")).respond_with(ResponseTemplate::new(200).set_body_json(json!({"sub": "g1", "email": "ana@gmail.test", "email_verified": true}))).mount(google_at).await;
    let (status, done) = app.send("POST", "/api/connections/google/callback", &ana, json!({"code": "ws-code", "state": state})).await;
    assert_eq!(status, StatusCode::OK, "{done}");
    assert_eq!(done["services"], json!(["gmail"]));

    let (_, overview) = app.get("/api/connections", &ana).await;
    assert_eq!(overview["google"]["email"], "ana@gmail.test");
    assert!(overview["hevy"].is_null());
    assert_eq!(app.call("DELETE", "/api/connections/google", Some(&ana.token), None).await.0, StatusCode::NO_CONTENT);
    let (_, after) = app.get("/api/connections/google", &ana).await;
    assert!(after["email"].is_null());
    let (status, _) = app.send("POST", "/api/connections/google/callback", &ana, json!({"code": "ws-code", "state": "used"})).await;
    assert!(status.is_client_error());

    assert_eq!(app.get("/api/connections/hevy", &ana).await.0, StatusCode::OK);
    let _ = app.call("DELETE", "/api/connections/hevy", Some(&ana.token), None).await;
}
