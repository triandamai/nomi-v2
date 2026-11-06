//! Importing a project from GitHub and connecting Hevy, against fakes set through the
//! environment (one test, so nothing else in this binary sees them change).

mod support;

use std::io::Write;

use axum::http::StatusCode;
use serde_json::json;
use sqlx::PgPool;
use support::App;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn zip_of(entries: &[(&str, &str)]) -> Vec<u8> {
    let mut writer = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    for (name, body) in entries {
        writer.start_file(*name, zip::write::SimpleFileOptions::default()).unwrap();
        writer.write_all(body.as_bytes()).unwrap();
    }
    writer.finish().unwrap().into_inner()
}

#[sqlx::test(migrations = "../../migrations")]
async fn github_imports_and_hevy_keys_go_through_their_services(pool: PgPool) {
    let fake = MockServer::start().await;
    std::env::set_var("GITHUB_API_URL", fake.uri());
    std::env::set_var("HEVY_BASE_URL", fake.uri());
    let repo = zip_of(&[("nomi-app-abc123/package.json", r#"{"dependencies":{"svelte":"5"}}"#), ("nomi-app-abc123/src/App.svelte", "<h1>Hi</h1>")]);
    Mock::given(method("GET")).and(path("/repos/nomi/app/zipball/main")).respond_with(ResponseTemplate::new(200).set_body_bytes(repo)).mount(&fake).await;
    Mock::given(method("GET")).and(path("/repos/nomi/private/zipball")).respond_with(ResponseTemplate::new(404)).mount(&fake).await;
    Mock::given(method("GET")).and(path("/repos/nomi/busy/zipball")).respond_with(ResponseTemplate::new(429)).mount(&fake).await;
    Mock::given(method("GET")).and(path("/repos/nomi/broken/zipball")).respond_with(ResponseTemplate::new(500)).mount(&fake).await;
    Mock::given(method("GET")).and(path("/repos/nomi/notzip/zipball")).respond_with(ResponseTemplate::new(200).set_body_string("nope")).mount(&fake).await;
    Mock::given(method("GET")).and(path("/v1/user/info")).and(header("api-key", "good")).respond_with(ResponseTemplate::new(200).set_body_json(json!({"data": {"id": "u1", "name": "ana_lifts", "url": ""}}))).mount(&fake).await;
    Mock::given(method("GET")).and(path("/v1/user/info")).and(header("api-key", "limited")).respond_with(ResponseTemplate::new(400).set_body_json(json!({"error": "not on Pro"}))).mount(&fake).await;
    Mock::given(method("GET")).and(path("/v1/user/info")).and(header("api-key", "down")).respond_with(ResponseTemplate::new(503)).mount(&fake).await;
    Mock::given(method("GET")).and(path("/v1/user/info")).respond_with(ResponseTemplate::new(401)).mount(&fake).await;

    let app = App::new(pool);
    let ana = app.person("ana@acme.test").await;
    let (project, _) = app.project(&ana, "Imported").await;
    let import = format!("/api/projects/{project}/import/github");

    let (status, body) = app.send("POST", &import, &ana, json!({"repo": "https://github.com/nomi/app", "ref": "main"})).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["imported"], 2);
    assert_eq!(body["stack"], "svelte");
    for (repo, expected) in [("nomi/private", StatusCode::NOT_FOUND), ("nomi/busy", StatusCode::TOO_MANY_REQUESTS), ("nomi/broken", StatusCode::BAD_GATEWAY), ("nomi/notzip", StatusCode::UNPROCESSABLE_ENTITY)] {
        assert_eq!(app.send("POST", &import, &ana, json!({"repo": repo})).await.0, expected, "{repo}");
    }

    let hevy = "/api/connections/hevy";
    for (key, status, code) in [("  ", StatusCode::BAD_REQUEST, "missing_key"), ("wrong", StatusCode::BAD_REQUEST, "key_rejected"), ("limited", StatusCode::BAD_REQUEST, "rejected"), ("down", StatusCode::BAD_GATEWAY, "unreachable")] {
        let (got, body) = app.send("PUT", hevy, &ana, json!({"api_key": key})).await;
        assert_eq!((got, body["error"].as_str()), (status, Some(code)), "{key}");
    }
    let (status, body) = app.send("PUT", hevy, &ana, json!({"api_key": "good"})).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["connection"]["account_name"], "ana_lifts");
    assert_eq!(app.get(hevy, &ana).await.1["connection"]["account_name"], "ana_lifts");
    assert_eq!(app.call("DELETE", hevy, Some(&ana.token), None).await.0, StatusCode::NO_CONTENT);
    assert!(app.get(hevy, &ana).await.1["connection"].is_null());
}
