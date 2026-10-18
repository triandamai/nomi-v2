//! A project's web address: reading it, checking a name as it's typed, and moving production to
//! a free one (or being told why not, with free names to pick from).
use axum::{body::Body, http::{Request, StatusCode}};
use http_body_util::BodyExt;
use nomi_server::app::{build_router, AppState};
use serde_json::{json, Value};
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;
use wiremock::matchers::{body_partial_json, method, path_regex, query_param};
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
        push: std::sync::Arc::new(nomi_server::push::NoopPush),
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


async fn publish_live(pool: &PgPool, project_id: Uuid, url: &str) {
    sqlx::query("UPDATE projects SET shipyard_app = $2, name = 'Todo' WHERE id = $1").bind(project_id).bind(format!("todo-{project_id}")).execute(pool).await.unwrap();
    sqlx::query("INSERT INTO project_publishes (project_id, environment, status, url, manifest) VALUES ($1, 'production', 'succeeded', $2, '{}')")
        .bind(project_id)
        .bind(url)
        .execute(pool)
        .await
        .unwrap();
}

fn ok(data: Value) -> ResponseTemplate {
    ResponseTemplate::new(200).set_body_json(json!({ "data": data, "error": null }))
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_address_can_be_checked_and_changed_by_admins(pool: PgPool) {
    let shipyard = MockServer::start().await;
    Mock::given(method("PUT")).and(path_regex(r"^/orgs/[^/]+$")).respond_with(ok(json!({}))).mount(&shipyard).await;
    let router = build_router(test_state(pool.clone(), &shipyard));
    let owner = register(router.clone(), "ana@example.com", json!({ "mode": "create", "name": "Acme" })).await;
    let project = new_project(router.clone(), &owner).await;
    let uri = format!("/api/projects/{project}/domain");

    // Never published: no address, and none to change.
    let (status, body) = request(router.clone(), "GET", &uri, Some(&owner), None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, json!({ "hostname": null, "subdomain": null, "url": null, "can_edit": true }));
    let (status, _) = request(router.clone(), "PUT", &uri, Some(&owner), Some(json!({ "subdomain": "my-todo" }))).await;
    assert_eq!(status, StatusCode::CONFLICT);

    publish_live(&pool, project, "https://todo.apps.example.dev").await;
    sqlx::query("UPDATE projects SET showcase = true, showcase_url = 'https://todo.apps.example.dev' WHERE id = $1").bind(project).execute(&pool).await.unwrap();
    let env = format!(r"/apps/todo-{project}/envs/production/domain$");
    Mock::given(method("GET"))
        .and(path_regex(&env))
        .respond_with(ok(json!({ "hostname": "todo.apps.example.dev", "subdomain": "todo", "url": "https://todo.apps.example.dev" })))
        .mount(&shipyard)
        .await;
    let (_, body) = request(router.clone(), "GET", &uri, Some(&owner), None).await;
    assert_eq!(body["hostname"], "todo.apps.example.dev");

    // Checking asks about this app's production, so its own name counts as free.
    Mock::given(method("GET"))
        .and(path_regex(r"/domains/check$"))
        .and(query_param("subdomain", "shop"))
        .and(query_param("app", format!("todo-{project}")))
        .and(query_param("env", "production"))
        .respond_with(ok(json!({ "subdomain": "shop", "hostname": "shop.apps.example.dev", "available": false, "reason": "taken",
            "message": "'shop' is already in use", "suggestions": [{ "subdomain": "shop-app", "hostname": "shop-app.apps.example.dev" }] })))
        .mount(&shipyard)
        .await;
    let (status, body) = request(router.clone(), "GET", &format!("{uri}/check?subdomain=%20Shop%20"), Some(&owner), None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["available"], false);
    assert_eq!(body["suggestions"][0]["subdomain"], "shop-app");

    // Taken a moment later: nothing changes, and the answer says why with free names.
    Mock::given(method("PUT"))
        .and(path_regex(&env))
        .and(body_partial_json(json!({ "subdomain": "shop" })))
        .respond_with(ResponseTemplate::new(409).set_body_json(json!({ "data": { "available": false }, "error": { "code": "CONFLICT", "message": "'shop' is already in use" } })))
        .mount(&shipyard)
        .await;
    let (status, body) = request(router.clone(), "PUT", &uri, Some(&owner), Some(json!({ "subdomain": "shop" }))).await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["message"], "'shop' is already in use");
    assert_eq!(body["suggestions"][0]["subdomain"], "shop-app");

    // A free name: production moves, and the recorded address and the showcase follow.
    Mock::given(method("PUT"))
        .and(path_regex(&env))
        .and(body_partial_json(json!({ "subdomain": "my-todo" })))
        .respond_with(ok(json!({ "hostname": "my-todo.apps.example.dev", "url": "https://my-todo.apps.example.dev", "previous_hostname": "todo.apps.example.dev", "changed": true })))
        .expect(1)
        .mount(&shipyard)
        .await;
    let (status, body) = request(router.clone(), "PUT", &uri, Some(&owner), Some(json!({ "subdomain": "my-todo" }))).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["changed"], true);
    let (publish_url, showcase_url): (String, String) = sqlx::query_as(
        "SELECT (SELECT url FROM project_publishes WHERE project_id = $1 ORDER BY created_at DESC LIMIT 1), showcase_url FROM projects WHERE id = $1",
    )
    .bind(project)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!((publish_url.as_str(), showcase_url.as_str()), ("https://my-todo.apps.example.dev", "https://my-todo.apps.example.dev"));

    // Someone else can't change it.
    let stranger = register(router.clone(), "bo@example.com", json!({ "mode": "create", "name": "Elsewhere" })).await;
    let (status, _) = request(router, "PUT", &uri, Some(&stranger), Some(json!({ "subdomain": "mine" }))).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}
