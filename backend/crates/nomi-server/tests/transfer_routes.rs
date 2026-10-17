//! Moving a project to another organization: the project's name has to be typed to confirm,
//! and the receiving organization's owners hear about it.
use axum::{body::Body, http::{Request, StatusCode}};
use http_body_util::BodyExt;
use nomi_server::app::{build_router, AppState};
use serde_json::{json, Value};
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;
use wiremock::matchers::{body_partial_json, method, path, path_regex};
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


#[sqlx::test(migrations = "../../migrations")]
async fn a_transfer_is_confirmed_by_name_and_the_new_owners_hear_of_it(pool: PgPool) {
    let shipyard = MockServer::start().await;
    let router = build_router(test_state(pool.clone(), &shipyard));
    let ana = register(router.clone(), "ana@example.com", json!({ "mode": "create", "name": "Acme" })).await;
    let bo = register(router.clone(), "bo@example.com", json!({ "mode": "create", "name": "Beta" })).await;
    let project = new_project(router.clone(), &ana).await;
    sqlx::query("UPDATE projects SET name = 'Habit Quest' WHERE id = $1").bind(project).execute(&pool).await.unwrap();
    let beta: Uuid = sqlx::query_scalar("SELECT id FROM organizations WHERE name = 'Beta'").fetch_one(&pool).await.unwrap();
    let uri = format!("/api/projects/{project}/transfer");

    // The wrong name: nothing moves.
    let (status, _) = request(router.clone(), "POST", &uri, Some(&ana), Some(json!({ "target_org_id": beta, "confirm_name": "Habit" }))).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let org: Uuid = sqlx::query_scalar("SELECT org_id FROM projects WHERE id = $1").bind(project).fetch_one(&pool).await.unwrap();
    assert_ne!(org, beta);

    let (status, _) = request(router.clone(), "POST", &uri, Some(&ana), Some(json!({ "target_org_id": beta, "confirm_name": " Habit Quest " }))).await;
    assert_eq!(status, StatusCode::OK);
    let org: Uuid = sqlx::query_scalar("SELECT org_id FROM projects WHERE id = $1").bind(project).fetch_one(&pool).await.unwrap();
    assert_eq!(org, beta);

    // Beta's owner has it in their inbox; whoever moved it doesn't.
    let (_, inbox) = request(router.clone(), "GET", "/api/notifications", Some(&bo), None).await;
    let items = inbox["items"].as_array().unwrap();
    let moved = items.iter().find(|n| n["kind"] == "project").expect("a project notification");
    assert_eq!(moved["title"], "Habit Quest was moved to Beta");
    assert!(moved["body"].as_str().unwrap().contains("from Acme to Beta"), "{moved}");
    assert_eq!(moved["link"], format!("/projects/{project}/app"));
    let (_, inbox) = request(router, "GET", "/api/notifications", Some(&ana), None).await;
    assert!(inbox["items"].as_array().unwrap().iter().all(|n| n["kind"] != "project"));
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_published_project_moves_its_shipyard_app_first(pool: PgPool) {
    let shipyard = MockServer::start().await;
    let router = build_router(test_state(pool.clone(), &shipyard));
    let ana = register(router.clone(), "ana@example.com", json!({ "mode": "create", "name": "Acme" })).await;
    register(router.clone(), "bo@example.com", json!({ "mode": "create", "name": "Beta" })).await;
    let project = new_project(router.clone(), &ana).await;
    sqlx::query("UPDATE projects SET name = 'Todo', shipyard_app = 'todo-1a2b3c' WHERE id = $1").bind(project).execute(&pool).await.unwrap();
    let (acme, beta): (Uuid, Uuid) = sqlx::query_as("SELECT (SELECT org_id FROM projects WHERE id = $1), (SELECT id FROM organizations WHERE name = 'Beta')")
        .bind(project)
        .fetch_one(&pool)
        .await
        .unwrap();
    let uri = format!("/api/projects/{project}/transfer");
    let transfer = format!("/orgs/{acme}/apps/todo-1a2b3c/transfer");
    Mock::given(method("PUT")).and(path_regex(r"^/orgs/[^/]+$")).respond_with(ResponseTemplate::new(200).set_body_json(json!({ "data": {}, "error": null }))).mount(&shipyard).await;

    // Shipyard refuses (a deployment is running): nothing moves in Nomi either.
    let busy = Mock::given(method("POST"))
        .and(path(transfer.clone()))
        .respond_with(ResponseTemplate::new(409).set_body_json(json!({ "data": null, "error": { "code": "CONFLICT", "message": "a deployment is running" } })))
        .expect(1)
        .mount_as_scoped(&shipyard)
        .await;
    let (status, _) = request(router.clone(), "POST", &uri, Some(&ana), Some(json!({ "target_org_id": beta, "confirm_name": "Todo" }))).await;
    assert_eq!(status, StatusCode::CONFLICT);
    let org: Uuid = sqlx::query_scalar("SELECT org_id FROM projects WHERE id = $1").bind(project).fetch_one(&pool).await.unwrap();
    assert_eq!(org, acme);
    drop(busy);

    // Shipyard moves the app to Beta's organization, then the project follows.
    Mock::given(method("POST"))
        .and(path(transfer))
        .and(body_partial_json(json!({ "to_org": beta.to_string() })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "data": {
            "app": "todo-1a2b3c", "from_org": acme.to_string(), "to_org": beta.to_string(), "environments": [], "redeploying": [], "skipped": []
        }, "error": null })))
        .expect(1)
        .mount(&shipyard)
        .await;
    let (status, _) = request(router, "POST", &uri, Some(&ana), Some(json!({ "target_org_id": beta, "confirm_name": "Todo" }))).await;
    assert_eq!(status, StatusCode::OK);
    let (org, app): (Uuid, Option<String>) = sqlx::query_as("SELECT org_id, shipyard_app FROM projects WHERE id = $1").bind(project).fetch_one(&pool).await.unwrap();
    assert_eq!((org, app.as_deref()), (beta, Some("todo-1a2b3c")));
}
