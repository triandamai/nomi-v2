//! Publishing a static artifact (`POST /orgs/{org}/static-publish`) against a fake Shipyard:
//! what is sent, the answer that is read, and where its addresses point.

use nomi_shipyard::{Shipyard, ShipyardConfig, ShipyardError, StaticPublish};
use serde_json::json;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const DEPLOYMENT: &str = "0193309a-5001-7c00-9abc-987654321def";
const PROJECT: &str = "0193309a-4f5b-7b00-8cb5-123456789abc";

fn config(base_url: &str) -> ShipyardConfig {
    ShipyardConfig { base_url: base_url.to_string(), key: "manage".into(), reveal_key: None, webhook_secret: None }
}

fn ready() -> ResponseTemplate {
    ResponseTemplate::new(200).set_body_json(json!({
        "success": true,
        "data": {
            "project_id": PROJECT, "project_slug": "design-abc", "deployment_id": DEPLOYMENT, "status": "READY",
            "preview_url": format!("/dpl/{DEPLOYMENT}"), "alias": "latest", "alias_url": format!("/alias/{PROJECT}/latest")
        }
    }))
}

#[tokio::test]
async fn the_artifact_goes_up_with_its_project_alias_and_fallback() {
    let server = MockServer::start().await;
    Mock::given(method("POST")).and(path("/orgs/org-1/static-publish")).and(header("authorization", "Bearer manage")).respond_with(ready()).mount(&server).await;
    let config = config(&server.uri());
    let http = reqwest::Client::new();
    let publish = StaticPublish { project_slug: "design-abc", alias: "latest", spa_fallback: false };
    let published = Shipyard::new(&http, &config).static_publish("org-1", &publish, b"zip-bytes".to_vec()).await.unwrap();

    assert_eq!((published.deployment_id.as_str(), published.project_id.as_str(), published.status.as_str()), (DEPLOYMENT, PROJECT, "READY"));
    assert_eq!(published.preview_url, format!("/dpl/{DEPLOYMENT}"));
    assert_eq!(published.alias_url.as_deref(), Some(format!("/alias/{PROJECT}/latest").as_str()));
    assert!(published.is_ready());

    let requests = server.received_requests().await.unwrap();
    let body = String::from_utf8_lossy(&requests[0].body).to_string();
    for part in ["name=\"project_slug\"\r\n\r\ndesign-abc", "name=\"alias\"\r\n\r\nlatest", "name=\"spa_fallback\"\r\n\r\nfalse", "name=\"artifact\"; filename=\"artifact.zip\"", "zip-bytes"] {
        assert!(body.contains(part), "the form has {part:?}: {body}");
    }
}

#[tokio::test]
async fn a_refusal_comes_back_in_shipyards_words() {
    let server = MockServer::start().await;
    let refused = ResponseTemplate::new(422).set_body_json(json!({"success": false, "error": {"code": "INVALID", "message": "the artifact has no index.html"}}));
    Mock::given(method("POST")).and(path("/orgs/org-1/static-publish")).respond_with(refused).mount(&server).await;
    let config = config(&server.uri());
    let http = reqwest::Client::new();
    let publish = StaticPublish { project_slug: "design-abc", alias: "latest", spa_fallback: true };
    let err = Shipyard::new(&http, &config).static_publish("org-1", &publish, Vec::new()).await.unwrap_err();
    assert!(matches!(err, ShipyardError::Rejected { status: 422, .. }), "{err:?}");
    assert_eq!(err.to_string(), "Shipyard refused it: the artifact has no index.html");
}

#[tokio::test]
async fn an_answer_without_a_deployment_is_not_a_publish() {
    let server = MockServer::start().await;
    Mock::given(method("POST")).respond_with(ResponseTemplate::new(200).set_body_json(json!({"success": true, "data": {"status": "READY"}}))).mount(&server).await;
    let config = config(&server.uri());
    let http = reqwest::Client::new();
    let publish = StaticPublish { project_slug: "design-abc", alias: "latest", spa_fallback: false };
    assert!(matches!(Shipyard::new(&http, &config).static_publish("org-1", &publish, Vec::new()).await, Err(ShipyardError::Unavailable(_))));
}

#[test]
fn shipyards_paths_are_addresses_on_its_own_host() {
    let config = config("https://ship.example.com/openapi/v1/integration");
    assert_eq!(config.origin().as_deref(), Some("https://ship.example.com"));
    assert_eq!(config.address("/dpl/d1").as_deref(), Some("https://ship.example.com/dpl/d1"));
    assert_eq!(config.address("https://cdn.example.com/dpl/d1").as_deref(), Some("https://cdn.example.com/dpl/d1"), "an absolute address is kept");
    assert_eq!(config.address("dpl/d1"), None, "neither a path nor an address");
    assert_eq!(config.address("//evil.example/dpl"), None, "not another host");
    let with_port = self::config("http://localhost:8080/openapi/v1/integration");
    assert_eq!(with_port.address("/dpl/d1").as_deref(), Some("http://localhost:8080/dpl/d1"));
    assert_eq!(self::config("not an address").origin(), None);
}

#[test]
fn a_static_artifact_keeps_every_file() {
    use std::io::Read;
    let bytes = nomi_shipyard::bundle::zip_static(&[("index.html".into(), b"<html>".to_vec()), ("build/app.js".into(), b"js".to_vec())]).unwrap();
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes)).unwrap();
    assert_eq!(archive.len(), 2, "nothing is left out of a preview");
    let mut content = String::new();
    archive.by_name("build/app.js").unwrap().read_to_string(&mut content).unwrap();
    assert_eq!(content, "js");
}
