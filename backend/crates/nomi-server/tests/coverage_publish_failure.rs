//! A publish that fails on Shipyard: the webhook marks it failed, and Nomi asks Shipyard which
//! deployment failed, at which step, and what the app printed, to tell the project's chat.

mod support;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::json;
use sqlx::PgPool;
use support::App;
use uuid::Uuid;
use wiremock::matchers::{method, path_regex};
use wiremock::{Mock, MockServer, ResponseTemplate};

const SECRET: &str = "whsec_test";

fn ok(data: serde_json::Value) -> ResponseTemplate {
    ResponseTemplate::new(200).set_body_json(json!({ "data": data, "error": null }))
}

fn signed(body: &serde_json::Value) -> Request<Body> {
    let raw = body.to_string();
    let now = chrono::Utc::now().timestamp();
    let signature = format!("t={now},v1={}", nomi_shipyard::signature::sign(SECRET, now, raw.as_bytes()));
    Request::builder()
        .method("POST")
        .uri("/api/hooks/shipyard")
        .header("content-type", "application/json")
        .header("shipyard-signature", signature)
        .body(Body::from(raw))
        .unwrap()
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_failed_publish_tells_the_chat_what_failed(pool: PgPool) {
    let shipyard = MockServer::start().await;
    Mock::given(method("GET")).and(path_regex(r"/publishes/pub-9$")).respond_with(ok(json!({"status": "failed", "deployments": [{"service": "web", "deployment_id": "dep-1", "status": "failed"}]}))).mount(&shipyard).await;
    Mock::given(method("GET")).and(path_regex(r"/deployments/dep-1$")).respond_with(ok(json!({"failure": {"step": "healthcheck", "phase": "deploy", "message": "the app didn't answer", "tail": ["listening on 3000"]}}))).mount(&shipyard).await;
    Mock::given(method("GET")).and(path_regex(r"/services/web/logs")).respond_with(ok(json!({"lines": ["Error: DATABASE_URL is not set"]}))).mount(&shipyard).await;
    Mock::given(method("GET")).and(path_regex(r"/envs/production$")).respond_with(ok(json!({"services": [{"name": "web", "urls": ["https://bakery.apps.test"]}]}))).mount(&shipyard).await;

    let mut state = support::state(pool.clone());
    state.shipyard = Some(nomi_shipyard::ShipyardConfig { base_url: shipyard.uri(), key: "manage".into(), reveal_key: None, webhook_secret: Some(SECRET.into()) });
    let app = App::with_state(pool.clone(), state);
    let owner = app.person("owner@acme.test").await;
    let (project, session) = app.project(&owner, "Bakery").await;
    sqlx::query("UPDATE projects SET shipyard_app = 'bakery' WHERE id = $1").bind(project).execute(&pool).await.unwrap();
    let org: Uuid = sqlx::query_scalar("SELECT org_id FROM projects WHERE id = $1").bind(project).fetch_one(&pool).await.unwrap();
    let publish: Uuid = sqlx::query_scalar("INSERT INTO project_publishes (project_id, shipyard_publish_id, manifest) VALUES ($1, 'pub-9', '{}') RETURNING id")
        .bind(project)
        .fetch_one(&pool)
        .await
        .unwrap();

    let event = json!({
        "id": "evt-fail", "type": "publish.failed", "created_at": chrono::Utc::now(),
        "org": { "id": "x", "external_id": org }, "app": "bakery", "environment": "production",
        "data": { "publish_id": "pub-9", "error": "deploy failed" }
    });
    assert_eq!(app.raw(signed(&event)).await.0, StatusCode::NO_CONTENT);

    let mut status = String::new();
    let mut told = 0i64;
    for _ in 0..100 {
        status = sqlx::query_scalar("SELECT status FROM project_publishes WHERE id = $1").bind(publish).fetch_one(&pool).await.unwrap();
        told = sqlx::query_scalar("SELECT count(*) FROM messages WHERE session_id = $1").bind(session).fetch_one(&pool).await.unwrap();
        if status == "failed" && told > 0 {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(30)).await;
    }
    assert_eq!(status, "failed");
    assert!(told > 0, "the project's chat hears about it");
    let asked: Vec<String> = shipyard.received_requests().await.unwrap().iter().map(|r| r.url.path().to_string()).collect();
    assert!(asked.iter().any(|p| p.ends_with("/deployments/dep-1")), "{asked:?}");

    // A host for a service, from the environment Shipyard reports.
    let http = reqwest::Client::new();
    let config = nomi_shipyard::ShipyardConfig { base_url: shipyard.uri(), key: "manage".into(), reveal_key: None, webhook_secret: None };
    let client = nomi_shipyard::Shipyard::new(&http, &config);
    assert_eq!(nomi_server::shipyard::service_host(&client, "o", "bakery", "production", "web").await.as_deref(), Some("bakery.apps.test"));
    assert_eq!(nomi_server::shipyard::service_host(&client, "o", "bakery", "production", "worker").await, None);
}
