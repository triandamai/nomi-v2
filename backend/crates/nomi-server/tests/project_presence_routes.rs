//! Who has a project open, and who's typing in its chat: short-lived, never stored as messages.
use axum::{body::Body, http::{Request, StatusCode}};
use http_body_util::BodyExt;
use nomi_server::app::{build_router, AppState};
use serde_json::{json, Value};
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;
use wiremock::MockServer;

const SECRET: &str = "test-secret-do-not-use-in-prod";

fn test_state(pool: PgPool, shipyard: Option<&MockServer>) -> AppState {
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
        shipyard: shipyard.map(|server| nomi_shipyard::ShipyardConfig {
            base_url: server.uri(),
            key: "ship_int_manage".into(),
            reveal_key: Some("ship_int_reveal".into()),
            webhook_secret: None,
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

async fn user_id(pool: &PgPool, email: &str) -> Uuid {
    sqlx::query_scalar("SELECT user_id FROM web_credentials WHERE email = $1").bind(email).fetch_one(pool).await.unwrap()
}


/// Ana's project in her organization, with Ben as an active member of that organization (so he
/// can open it), and Cy in an organization of his own (so he can't).
struct Team {
    router: axum::Router,
    project_id: Uuid,
    ana: String,
    ben: String,
    cy: String,
}

async fn team(pool: &PgPool) -> Team {
    let router = build_router(test_state(pool.clone(), None));
    let ana = register(router.clone(), "ana@acme.test").await;
    let ben = register(router.clone(), "ben@acme.test").await;
    let cy = register(router.clone(), "cy@other.test").await;
    let (_, orgs) = request(router.clone(), "GET", "/api/orgs", Some(&ana), None).await;
    let org_id: Uuid = orgs["orgs"][0]["id"].as_str().unwrap().parse().unwrap();
    request(router.clone(), "PUT", "/api/preferences/project-org", Some(&ana), Some(json!({ "org_id": org_id }))).await;
    let (_, created) = request(router.clone(), "POST", "/api/projects", Some(&ana), None).await;
    let project_id: Uuid = created["project_id"].as_str().unwrap().parse().unwrap();
    let ben_id = user_id(pool, "ben@acme.test").await;
    sqlx::query("INSERT INTO memberships (org_id, user_id, role) VALUES ($1, $2, 'member')").bind(org_id).bind(ben_id).execute(pool).await.unwrap();
    Team { router, project_id, ana, ben, cy }
}

fn names(body: &Value) -> Vec<String> {
    body["people"].as_array().unwrap().iter().map(|p| p["name"].as_str().unwrap().to_string()).collect()
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_heartbeat_puts_someone_in_here_now_for_everyone_else(pool: PgPool) {
    let t = team(&pool).await;
    let path = format!("/api/projects/{}/presence", t.project_id);
    let (status, _) = request(t.router.clone(), "POST", &path, Some(&t.ana), None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    request(t.router.clone(), "POST", &path, Some(&t.ben), None).await;

    let (_, seen_by_ben) = request(t.router.clone(), "GET", &path, Some(&t.ben), None).await;
    let (_, seen_by_ana) = request(t.router.clone(), "GET", &path, Some(&t.ana), None).await;

    assert_eq!(names(&seen_by_ben), vec!["ana"]);
    assert_eq!(names(&seen_by_ana), vec!["ben"]);
    // So the page can tell its own typing pings from others'.
    assert_eq!(seen_by_ben["me"], json!(user_id(&pool, "ben@acme.test").await));
}

#[sqlx::test(migrations = "../../migrations")]
async fn someone_who_went_quiet_or_left_is_no_longer_here(pool: PgPool) {
    let t = team(&pool).await;
    let path = format!("/api/projects/{}/presence", t.project_id);
    request(t.router.clone(), "POST", &path, Some(&t.ana), None).await;
    sqlx::query("UPDATE project_presence SET last_seen = now() - interval '2 minutes'").execute(&pool).await.unwrap();
    let (_, quiet) = request(t.router.clone(), "GET", &path, Some(&t.ben), None).await;
    assert!(names(&quiet).is_empty());

    request(t.router.clone(), "POST", &path, Some(&t.ana), None).await;
    let (status, _) = request(t.router.clone(), "DELETE", &path, Some(&t.ana), None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (_, left) = request(t.router.clone(), "GET", &path, Some(&t.ben), None).await;
    assert!(names(&left).is_empty());
}

#[sqlx::test(migrations = "../../migrations")]
async fn typing_is_for_people_who_can_open_the_project(pool: PgPool) {
    let t = team(&pool).await;
    let typing = format!("/api/projects/{}/typing", t.project_id);
    let presence = format!("/api/projects/{}/presence", t.project_id);
    assert_eq!(request(t.router.clone(), "POST", &typing, Some(&t.ben), None).await.0, StatusCode::NO_CONTENT);
    assert_eq!(request(t.router.clone(), "POST", &typing, Some(&t.cy), None).await.0, StatusCode::NOT_FOUND);
    assert_eq!(request(t.router.clone(), "POST", &presence, Some(&t.cy), None).await.0, StatusCode::NOT_FOUND);
    assert_eq!(request(t.router.clone(), "GET", &presence, Some(&t.cy), None).await.0, StatusCode::NOT_FOUND);
}
