//! Design previews published through Shipyard's static artifacts: the chat's organization and
//! static project, the files that go up, and the address that comes back.

use std::io::Read;

use nomi_agent_core::design_artifact::ArtifactFile;
use nomi_agent_core::static_publish::{PublishRequest, StaticPublisher};
use nomi_server::static_publisher::{project_slug, ShipyardStaticPublisher, PROVIDER};
use nomi_shipyard::ShipyardConfig;
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

async fn chat(pool: &PgPool) -> (Uuid, Uuid, Uuid) {
    let org_id: Uuid = sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Roti Pagi') RETURNING id").fetch_one(pool).await.unwrap();
    let session_id = sqlx::query_scalar("INSERT INTO sessions (org_id, channel, chat_id) VALUES ($1, 'telegram', $2) RETURNING id")
        .bind(org_id)
        .bind(Uuid::new_v4().to_string())
        .fetch_one(pool)
        .await
        .unwrap();
    let user_id = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(pool).await.unwrap();
    (org_id, session_id, user_id)
}

fn files() -> Vec<ArtifactFile> {
    vec![ArtifactFile { path: "index.html".into(), content: "<html>Hi</html>".into() }, ArtifactFile { path: "build/app.js".into(), content: "let a = 1".into() }]
}

fn publisher(pool: &PgPool, server: &MockServer) -> ShipyardStaticPublisher {
    let config = ShipyardConfig { base_url: format!("{}/openapi/v1/integration", server.uri()), key: "manage".into(), reveal_key: None, webhook_secret: None };
    ShipyardStaticPublisher::new(pool.clone(), reqwest::Client::new(), config)
}

fn answer(status: &str) -> ResponseTemplate {
    ResponseTemplate::new(200).set_body_json(json!({"success": true, "data": {"project_id": "p1", "project_slug": "x", "deployment_id": "d1", "status": status, "preview_url": "/dpl/d1", "alias": "latest", "alias_url": "/alias/p1/latest"}}))
}

async fn shipyard(org_id: Uuid, publish: ResponseTemplate) -> MockServer {
    let server = MockServer::start().await;
    Mock::given(method("PUT")).and(path(format!("/openapi/v1/integration/orgs/{org_id}"))).respond_with(ResponseTemplate::new(200).set_body_json(json!({"data": {}}))).mount(&server).await;
    Mock::given(method("POST")).and(path(format!("/openapi/v1/integration/orgs/{org_id}/static-publish"))).respond_with(publish).mount(&server).await;
    server
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_preview_goes_to_the_chats_organization_and_gets_its_pinned_address(pool: PgPool) {
    let (org_id, session_id, user_id) = chat(&pool).await;
    let server = shipyard(org_id, answer("READY")).await;
    let files = files();
    let request = PublishRequest { user_id, session_id, version: 3, title: "Editorial", files: &files };
    let published = publisher(&pool, &server).publish(&request).await.unwrap();
    assert_eq!(published.provider, PROVIDER);
    assert_eq!(published.url, format!("{}/dpl/d1", server.uri()), "the deployment's own address, not the alias that moves");

    let requests = server.received_requests().await.unwrap();
    assert_eq!(requests[0].method.as_str(), "PUT", "the organization is made sure of first");
    assert_eq!(
        requests[0].body_json::<serde_json::Value>().unwrap(),
        json!({"name": "Roti Pagi", "quota": {"max_projects": -1, "max_members": -1, "max_orgs": -1}})
    );
    let upload = &requests[1].body;
    let body = String::from_utf8_lossy(upload).to_string();
    assert!(body.contains(&format!("name=\"project_slug\"\r\n\r\n{}", project_slug(session_id))), "{body}");
    assert!(body.contains("name=\"alias\"\r\n\r\nlatest") && body.contains("name=\"spa_fallback\"\r\n\r\nfalse"));
    let start = upload.windows(2).position(|w| w == b"PK").expect("a zip in the form");
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(upload[start..].to_vec())).unwrap();
    let mut script = String::new();
    archive.by_name("build/app.js").unwrap().read_to_string(&mut script).unwrap();
    assert_eq!(script, "let a = 1", "every file goes up, whatever its folder");
    let synced: Option<chrono::DateTime<chrono::Utc>> = sqlx::query_scalar("SELECT shipyard_synced_at FROM organizations WHERE id = $1").bind(org_id).fetch_one(&pool).await.unwrap();
    assert!(synced.is_some());
}

#[sqlx::test(migrations = "../../migrations")]
async fn shipyard_failing_or_not_serving_yet_is_said_in_words(pool: PgPool) {
    let (org_id, session_id, user_id) = chat(&pool).await;
    let files = files();
    let request = PublishRequest { user_id, session_id, version: 1, title: "Editorial", files: &files };

    let down = shipyard(org_id, ResponseTemplate::new(503).set_body_string("down")).await;
    assert!(publisher(&pool, &down).publish(&request).await.unwrap_err().starts_with("couldn't reach Shipyard"));

    let building = shipyard(org_id, answer("BUILDING")).await;
    assert_eq!(publisher(&pool, &building).publish(&request).await.unwrap_err(), "Shipyard isn't serving it yet (BUILDING)");

    let nowhere = PublishRequest { session_id: Uuid::new_v4(), ..request };
    assert_eq!(publisher(&pool, &building).publish(&nowhere).await.unwrap_err(), "this chat couldn't be found");
}
