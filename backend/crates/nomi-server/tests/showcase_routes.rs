//! The showcase: a project's admins show a live app (with one line about it), anyone can see the
//! page without signing in, and taking production offline takes the app off it.
use axum::{body::Body, http::{Request, StatusCode}};
use http_body_util::BodyExt;
use nomi_server::app::{build_router, AppState};
use serde_json::{json, Value};
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;
use wiremock::matchers::{method, path_regex};
use wiremock::{Mock, MockServer, ResponseTemplate};

const SECRET: &str = "test-secret-do-not-use-in-prod";

fn test_state(pool: PgPool, shipyard: &MockServer) -> AppState {
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
        shipyard: Some(nomi_shipyard::ShipyardConfig {
            base_url: shipyard.uri(),
            key: "ship_int_manage".into(),
            reveal_key: None,
            webhook_secret: Some("whsec_test".into()),
        }),
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

async fn register(router: axum::Router, email: &str, org: Value) -> String {
    request(router.clone(), "POST", "/api/auth/register", None, Some(json!({ "email": email, "password": "correct-password", "org": org }))).await;
    let (_, body) = request(router, "POST", "/api/auth/login", None, Some(json!({ "email": email, "password": "correct-password" }))).await;
    body["access_token"].as_str().unwrap().to_string()
}

async fn new_project(router: axum::Router, token: &str) -> Uuid {
    let (_, orgs) = request(router.clone(), "GET", "/api/orgs", Some(token), None).await;
    let org_id = orgs["orgs"][0]["id"].as_str().unwrap();
    request(router.clone(), "PUT", "/api/preferences/project-org", Some(token), Some(json!({ "org_id": org_id }))).await;
    let (status, created) = request(router, "POST", "/api/projects", Some(token), None).await;
    assert_eq!(status, StatusCode::CREATED);
    created["project_id"].as_str().unwrap().parse().unwrap()
}

/// Shipyard reports `url` as the address of `project_id`'s production environment.
async fn live_on_shipyard(shipyard: &MockServer, project_id: Uuid, url: &str) {
    Mock::given(method("GET"))
        .and(path_regex(format!(r"/apps/todo-{project_id}/envs/production$")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "data": { "services": [
            { "name": "db", "type": "database", "urls": [] },
            { "name": "web", "urls": [url], "live": true },
        ] }, "error": null })))
        .mount(shipyard)
        .await;
}

async fn publish_live(pool: &PgPool, project_id: Uuid, url: &str) {
    publish_named(pool, project_id, "Todo", url).await;
}

async fn publish_named(pool: &PgPool, project_id: Uuid, name: &str, url: &str) {
    sqlx::query("UPDATE projects SET shipyard_app = $2, name = $3 WHERE id = $1").bind(project_id).bind(format!("todo-{project_id}")).bind(name).execute(pool).await.unwrap();
    sqlx::query("INSERT INTO project_publishes (project_id, environment, status, url, manifest) VALUES ($1, 'production', 'succeeded', $2, '{}')")
        .bind(project_id)
        .bind(url)
        .execute(pool)
        .await
        .unwrap();
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_live_app_can_be_shown_and_anyone_sees_it(pool: PgPool) {
    let shipyard = MockServer::start().await;
    Mock::given(method("PUT")).and(path_regex(r"^/orgs/")).respond_with(ResponseTemplate::new(200).set_body_json(json!({ "data": {}, "error": null }))).mount(&shipyard).await;
    Mock::given(method("DELETE")).and(path_regex(r"/envs/production$")).respond_with(ResponseTemplate::new(200).set_body_json(json!({ "data": {}, "error": null }))).expect(1).mount(&shipyard).await;
    let router = build_router(test_state(pool.clone(), &shipyard));
    let owner = register(router.clone(), "ana@example.com", json!({ "mode": "create", "name": "Acme Studio" })).await;
    let project = new_project(router.clone(), &owner).await;
    let uri = format!("/api/projects/{project}/showcase");

    // Not published yet: it can't be shown.
    let (status, body) = request(router.clone(), "GET", &uri, Some(&owner), None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, json!({ "showcase": false, "blurb": null, "url": null, "can_edit": true }));
    let (status, _) = request(router.clone(), "PUT", &uri, Some(&owner), Some(json!({ "showcase": true }))).await;
    assert_eq!(status, StatusCode::CONFLICT);

    publish_live(&pool, project, "https://todo.apps.example.dev").await;
    live_on_shipyard(&shipyard, project, "https://todo.apps.example.dev").await;
    let (status, _) = request(router.clone(), "PUT", &uri, Some(&owner), Some(json!({ "showcase": true, "blurb": "x".repeat(141) }))).await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "a description that's too long");
    let (status, body) = request(router.clone(), "PUT", &uri, Some(&owner), Some(json!({ "showcase": true, "blurb": "  A to-do list   for two. " }))).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["showcase"], true);
    assert_eq!(body["blurb"], "A to-do list for two.");

    // Public: no sign-in.
    let (status, apps) = request(router.clone(), "GET", "/api/showcase", None, None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(apps.as_array().unwrap().len(), 1);
    assert_eq!(apps[0]["name"], "Todo");
    assert_eq!(apps[0]["url"], "https://todo.apps.example.dev");
    assert_eq!(apps[0]["team"], "Acme Studio");

    // Someone else can't change it.
    let stranger = register(router.clone(), "bo@example.com", json!({ "mode": "create", "name": "Elsewhere" })).await;
    let (status, _) = request(router.clone(), "PUT", &uri, Some(&stranger), Some(json!({ "showcase": false }))).await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    // Production offline: off the showcase.
    let (status, _) = request(router.clone(), "DELETE", &format!("/api/projects/{project}/deploy/environments/production"), Some(&owner), None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (_, apps) = request(router.clone(), "GET", "/api/showcase", None, None).await;
    assert_eq!(apps, json!([]));
    let (_, body) = request(router, "GET", &uri, Some(&owner), None).await;
    assert_eq!(body["showcase"], false);
    let stored: Option<String> = sqlx::query_scalar("SELECT showcase_url FROM projects WHERE id = $1").bind(project).fetch_one(&pool).await.unwrap();
    assert_eq!(stored, None);
}

#[sqlx::test(migrations = "../../migrations")]
async fn each_app_is_shown_at_the_address_shipyard_reports_for_it(pool: PgPool) {
    let shipyard = MockServer::start().await;
    Mock::given(method("PUT")).and(path_regex(r"^/orgs/")).respond_with(ResponseTemplate::new(200).set_body_json(json!({ "data": {}, "error": null }))).mount(&shipyard).await;
    let router = build_router(test_state(pool.clone(), &shipyard));
    let owner = register(router.clone(), "ana@example.com", json!({ "mode": "create", "name": "Acme" })).await;
    let habits = new_project(router.clone(), &owner).await;
    let users = new_project(router.clone(), &owner).await;
    let other = new_project(router.clone(), &owner).await;

    // The publish records name the same address for both; Shipyard knows better.
    publish_named(&pool, users, "Users", "https://users.apps.example.dev").await;
    publish_named(&pool, habits, "Habits", "https://users.apps.example.dev").await;
    live_on_shipyard(&shipyard, users, "https://users.apps.example.dev").await;
    live_on_shipyard(&shipyard, habits, "https://habits.apps.example.dev").await;
    for project in [users, habits] {
        let (status, body) = request(router.clone(), "PUT", &format!("/api/projects/{project}/showcase"), Some(&owner), Some(json!({ "showcase": true }))).await;
        assert_eq!(status, StatusCode::OK, "{body}");
    }
    let (_, body) = request(router.clone(), "GET", &format!("/api/projects/{habits}/showcase"), Some(&owner), None).await;
    assert_eq!(body["url"], "https://habits.apps.example.dev");
    let (_, apps) = request(router.clone(), "GET", "/api/showcase", None, None).await;
    let mut shown: Vec<(String, String)> = apps.as_array().unwrap().iter().map(|a| (a["name"].as_str().unwrap().into(), a["url"].as_str().unwrap().into())).collect();
    shown.sort();
    assert_eq!(shown, vec![("Habits".into(), "https://habits.apps.example.dev".into()), ("Users".into(), "https://users.apps.example.dev".into())]);

    // An address already shown for another project isn't shown twice.
    publish_named(&pool, other, "Other", "https://users.apps.example.dev").await;
    live_on_shipyard(&shipyard, other, "https://users.apps.example.dev").await;
    let (status, _) = request(router.clone(), "PUT", &format!("/api/projects/{other}/showcase"), Some(&owner), Some(json!({ "showcase": true }))).await;
    assert_eq!(status, StatusCode::CONFLICT);

    // Shipyard says it isn't live: it can't be shown.
    let offline = new_project(router.clone(), &owner).await;
    publish_named(&pool, offline, "Offline", "https://offline.apps.example.dev").await;
    Mock::given(method("GET"))
        .and(path_regex(format!(r"/apps/todo-{offline}/envs/production$")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "data": { "services": [{ "name": "web", "urls": [] }] }, "error": null })))
        .mount(&shipyard)
        .await;
    let (status, _) = request(router, "PUT", &format!("/api/projects/{offline}/showcase"), Some(&owner), Some(json!({ "showcase": true }))).await;
    assert_eq!(status, StatusCode::CONFLICT);
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_personal_workspace_isnt_named(pool: PgPool) {
    let shipyard = MockServer::start().await;
    Mock::given(method("PUT")).and(path_regex(r"^/orgs/")).respond_with(ResponseTemplate::new(200).set_body_json(json!({ "data": {}, "error": null }))).mount(&shipyard).await;
    let router = build_router(test_state(pool.clone(), &shipyard));
    let owner = register(router.clone(), "ana@example.com", json!({ "mode": "create", "name": "Acme" })).await;
    let project = new_project(router.clone(), &owner).await;
    sqlx::query("UPDATE organizations SET is_personal = true WHERE id = (SELECT org_id FROM projects WHERE id = $1)").bind(project).execute(&pool).await.unwrap();
    publish_live(&pool, project, "https://todo.apps.example.dev").await;
    live_on_shipyard(&shipyard, project, "https://todo.apps.example.dev").await;
    request(router.clone(), "PUT", &format!("/api/projects/{project}/showcase"), Some(&owner), Some(json!({ "showcase": true }))).await;
    let (_, apps) = request(router, "GET", "/api/showcase", None, None).await;
    assert_eq!(apps[0]["team"], Value::Null);
}
