//! Connecting GitHub (a token, or Nomi's OAuth app) and pushing a project to a repository, against
//! a mock GitHub. One test: GitHub's addresses come from the environment, which every test in this
//! binary would share.
use axum::{body::Body, http::{Request, StatusCode}};
use http_body_util::BodyExt;
use nomi_server::app::{build_router, AppState};
use serde_json::{json, Value};
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;
use wiremock::matchers::{body_string_contains, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

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

async fn request(router: axum::Router, method: &str, uri: &str, token: Option<&str>, body: Option<Value>) -> (StatusCode, Value) {
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
    (status, if bytes.is_empty() { Value::Null } else { serde_json::from_slice(&bytes).unwrap_or(Value::Null) })
}

async fn register(router: axum::Router, email: &str) -> String {
    request(
        router.clone(),
        "POST",
        "/api/auth/register",
        None,
        Some(json!({ "email": email, "password": "correct-password", "org": { "mode": "create", "name": "Acme" } })),
    )
    .await;
    let (_, body) = request(router, "POST", "/api/auth/login", None, Some(json!({ "email": email, "password": "correct-password" }))).await;
    body["access_token"].as_str().unwrap().to_string()
}

async fn project_with_files(router: axum::Router, token: &str) -> Uuid {
    let (_, orgs) = request(router.clone(), "GET", "/api/orgs", Some(token), None).await;
    let org_id = orgs["orgs"][0]["id"].as_str().unwrap();
    request(router.clone(), "PUT", "/api/preferences/project-org", Some(token), Some(json!({ "org_id": org_id }))).await;
    let (_, created) = request(router.clone(), "POST", "/api/projects", Some(token), None).await;
    let project_id: Uuid = created["project_id"].as_str().unwrap().parse().unwrap();
    for (file, content) in [("package.json", r#"{"name":"todo"}"#), ("src/routes/+page.svelte", "<h1>Todo</h1>"), (".env", "SECRET=1")] {
        let (status, _) = request(router.clone(), "PUT", &format!("/api/projects/{project_id}/files/{file}"), Some(token), Some(json!({ "content": content }))).await;
        assert_eq!(status, StatusCode::OK, "{file}");
    }
    project_id
}

fn ok(body: Value) -> ResponseTemplate {
    ResponseTemplate::new(200).set_body_json(body)
}

/// A repository whose Git Data API answers: blobs, a tree, a commit, the branch moved.
async fn mount_git(github: &MockServer, repo: &str) {
    let base = format!("/repos/ana-gh/{repo}/git");
    Mock::given(method("POST")).and(path(format!("{base}/blobs"))).respond_with(ResponseTemplate::new(201).set_body_json(json!({ "sha": "blob-sha" }))).mount(github).await;
    Mock::given(method("POST")).and(path(format!("{base}/trees"))).respond_with(ResponseTemplate::new(201).set_body_json(json!({ "sha": "tree-sha" }))).mount(github).await;
    Mock::given(method("POST")).and(path(format!("{base}/commits"))).respond_with(ResponseTemplate::new(201).set_body_json(json!({ "sha": format!("{repo}-commit") }))).mount(github).await;
    Mock::given(method("PATCH")).and(path(format!("{base}/refs/heads/main"))).respond_with(ok(json!({ "object": { "sha": format!("{repo}-commit") } }))).mount(github).await;
}

fn body_of(request: &wiremock::Request) -> Value {
    serde_json::from_slice(&request.body).unwrap()
}

#[sqlx::test(migrations = "../../migrations")]
async fn github_connects_and_a_project_is_pushed_to_it(pool: PgPool) {
    let github = MockServer::start().await;
    std::env::set_var("GITHUB_API_URL", github.uri());
    std::env::set_var("GITHUB_OAUTH_URL", github.uri());
    std::env::set_var("GITHUB_CLIENT_ID", "client-id");
    std::env::set_var("GITHUB_CLIENT_SECRET", "client-secret");
    std::env::set_var("GITHUB_REDIRECT_URI", "http://app.test/connections/github/callback");

    Mock::given(method("GET")).and(path("/user")).and(header("authorization", "Bearer pasted-token")).respond_with(ok(json!({ "login": "ana-gh" }))).mount(&github).await;
    Mock::given(method("GET")).and(path("/user")).and(header("authorization", "Bearer oauth-token")).respond_with(ok(json!({ "login": "ana-gh" }))).mount(&github).await;
    Mock::given(method("GET")).and(path("/user")).respond_with(ResponseTemplate::new(401).set_body_json(json!({ "message": "Bad credentials" }))).mount(&github).await;
    Mock::given(method("POST"))
        .and(path("/login/oauth/access_token"))
        .and(body_string_contains("code=good-code"))
        .respond_with(ok(json!({ "access_token": "oauth-token", "token_type": "bearer", "scope": "repo" })))
        .mount(&github)
        .await;

    let router = build_router(test_state(pool.clone()));
    let token = register(router.clone(), "ana@acme.test").await;
    let project_id = project_with_files(router.clone(), &token).await;
    let export = format!("/api/projects/{project_id}/export/github");

    // Not connected yet: pushing asks for GitHub first. Nomi has an OAuth app here.
    let (_, status) = request(router.clone(), "GET", "/api/connections/github", Some(&token), None).await;
    assert_eq!(status, json!({ "oauth": true, "connection": null }));
    let (code, _) = request(router.clone(), "POST", &export, Some(&token), Some(json!({ "repo": "ana-gh/todo" }))).await;
    assert_eq!(code, StatusCode::CONFLICT);

    // A token GitHub doesn't accept isn't kept; one it does is, as that GitHub account.
    let (code, _) = request(router.clone(), "PUT", "/api/connections/github", Some(&token), Some(json!({ "token": "wrong" }))).await;
    assert_eq!(code, StatusCode::BAD_REQUEST);
    let (code, connected) = request(router.clone(), "PUT", "/api/connections/github", Some(&token), Some(json!({ "token": " pasted-token " }))).await;
    assert_eq!(code, StatusCode::OK, "{connected}");
    assert_eq!(connected["connection"]["login"], "ana-gh");
    assert_eq!(connected["connection"]["kind"], "token");
    let stored: Vec<u8> = sqlx::query_scalar("SELECT token_encrypted FROM github_connections").fetch_one(&pool).await.unwrap();
    assert!(!String::from_utf8_lossy(&stored).contains("pasted-token"));

    // An existing repository: one commit of the project's files on its branch, .env left out.
    Mock::given(method("GET"))
        .and(path("/repos/ana-gh/todo"))
        .respond_with(ok(json!({ "default_branch": "main", "html_url": "https://github.com/ana-gh/todo" })))
        .mount(&github)
        .await;
    Mock::given(method("GET")).and(path("/repos/ana-gh/todo/git/ref/heads/main")).respond_with(ok(json!({ "object": { "sha": "base-sha" } }))).mount(&github).await;
    mount_git(&github, "todo").await;
    let (code, pushed) = request(router.clone(), "POST", &export, Some(&token), Some(json!({ "repo": "https://github.com/ana-gh/todo", "message": "First version" }))).await;
    assert_eq!(code, StatusCode::OK, "{pushed}");
    assert_eq!(pushed, json!({ "url": "https://github.com/ana-gh/todo", "commit": "todo-commit", "branch": "main", "created": false }));
    let received = github.received_requests().await.unwrap();
    let tree = body_of(received.iter().find(|r| r.url.path() == "/repos/ana-gh/todo/git/trees").unwrap());
    let paths: Vec<&str> = tree["tree"].as_array().unwrap().iter().map(|e| e["path"].as_str().unwrap()).collect();
    assert_eq!(paths, vec!["package.json", "src/routes/+page.svelte"]);
    let commit = body_of(received.iter().find(|r| r.url.path() == "/repos/ana-gh/todo/git/commits").unwrap());
    assert_eq!(commit, json!({ "message": "First version", "tree": "tree-sha", "parents": ["base-sha"] }));
    let moved = body_of(received.iter().find(|r| r.method.as_str() == "PATCH").unwrap());
    assert_eq!(moved, json!({ "sha": "todo-commit", "force": false }));

    // A repository that doesn't exist: refused, unless asked to make it (private by default).
    Mock::given(method("GET")).and(path("/repos/ana-gh/new-app")).respond_with(ResponseTemplate::new(404).set_body_json(json!({ "message": "Not Found" }))).mount(&github).await;
    Mock::given(method("POST"))
        .and(path("/user/repos"))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({ "default_branch": "main", "html_url": "https://github.com/ana-gh/new-app" })))
        .mount(&github)
        .await;
    Mock::given(method("GET")).and(path("/repos/ana-gh/new-app/git/ref/heads/main")).respond_with(ok(json!({ "object": { "sha": "readme-sha" } }))).mount(&github).await;
    mount_git(&github, "new-app").await;
    let (code, _) = request(router.clone(), "POST", &export, Some(&token), Some(json!({ "repo": "ana-gh/new-app" }))).await;
    assert_eq!(code, StatusCode::NOT_FOUND);
    let (code, made) = request(router.clone(), "POST", &export, Some(&token), Some(json!({ "repo": "ana-gh/new-app", "create": true }))).await;
    assert_eq!(code, StatusCode::OK, "{made}");
    assert_eq!(made["created"], true);
    let received = github.received_requests().await.unwrap();
    let asked = body_of(received.iter().find(|r| r.url.path() == "/user/repos").unwrap());
    assert_eq!(asked, json!({ "name": "new-app", "private": true, "auto_init": true }));

    // An empty repository gets its first commit through the contents API, then the rest on top.
    Mock::given(method("GET"))
        .and(path("/repos/ana-gh/empty"))
        .respond_with(ok(json!({ "default_branch": "main", "html_url": "https://github.com/ana-gh/empty" })))
        .mount(&github)
        .await;
    Mock::given(method("GET")).and(path("/repos/ana-gh/empty/git/ref/heads/main")).respond_with(ResponseTemplate::new(409).set_body_json(json!({ "message": "Git Repository is empty." }))).mount(&github).await;
    Mock::given(method("PUT")).and(path("/repos/ana-gh/empty/contents/package.json")).respond_with(ResponseTemplate::new(201).set_body_json(json!({ "commit": { "sha": "init-sha" } }))).mount(&github).await;
    mount_git(&github, "empty").await;
    let (code, filled) = request(router.clone(), "POST", &export, Some(&token), Some(json!({ "repo": "ana-gh/empty" }))).await;
    assert_eq!(code, StatusCode::OK, "{filled}");
    let received = github.received_requests().await.unwrap();
    let commit = body_of(received.iter().find(|r| r.url.path() == "/repos/ana-gh/empty/git/commits").unwrap());
    assert_eq!(commit["parents"], json!(["init-sha"]));

    // Only people who can open the project push it, and only to a real repository name.
    let outsider = register(router.clone(), "budi@other.test").await;
    let (code, _) = request(router.clone(), "POST", &export, Some(&outsider), Some(json!({ "repo": "ana-gh/todo" }))).await;
    assert_eq!(code, StatusCode::NOT_FOUND);
    let (code, _) = request(router.clone(), "POST", &export, Some(&token), Some(json!({ "repo": "not a repo" }))).await;
    assert_eq!(code, StatusCode::BAD_REQUEST);

    // Through Nomi's OAuth app: GitHub's page, then the code it sends back, once.
    let (code, started) = request(router.clone(), "POST", "/api/connections/github/start", Some(&token), None).await;
    assert_eq!(code, StatusCode::OK);
    let url = reqwest::Url::parse(started["url"].as_str().unwrap()).unwrap();
    assert_eq!(url.path(), "/login/oauth/authorize");
    let query: std::collections::HashMap<_, _> = url.query_pairs().into_owned().collect();
    assert_eq!(query["scope"], "repo");
    assert_eq!(query["redirect_uri"], "http://app.test/connections/github/callback");
    let state = query["state"].clone();
    let (code, _) = request(router.clone(), "POST", "/api/connections/github/callback", Some(&outsider), Some(json!({ "code": "good-code", "state": state }))).await;
    assert_eq!(code, StatusCode::BAD_REQUEST);
    let (code, via_oauth) = request(router.clone(), "POST", "/api/connections/github/callback", Some(&token), Some(json!({ "code": "good-code", "state": state }))).await;
    assert_eq!(code, StatusCode::OK, "{via_oauth}");
    assert_eq!(via_oauth["connection"]["kind"], "oauth");
    let (code, _) = request(router.clone(), "POST", "/api/connections/github/callback", Some(&token), Some(json!({ "code": "good-code", "state": state }))).await;
    assert_eq!(code, StatusCode::BAD_REQUEST);

    // Disconnecting forgets it.
    let (code, _) = request(router.clone(), "DELETE", "/api/connections/github", Some(&token), None).await;
    assert_eq!(code, StatusCode::NO_CONTENT);
    let (_, status) = request(router, "GET", "/api/connections/github", Some(&token), None).await;
    assert_eq!(status["connection"], Value::Null);
}
