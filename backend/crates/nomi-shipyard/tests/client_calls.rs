//! Every Shipyard client call against a fake Shipyard: the path and method each one uses, the
//! envelope it reads, and how errors come back.

use nomi_shipyard::{DomainName, MetricsRange, MetricsStep, Shipyard, ShipyardConfig, ShipyardError};
use serde_json::json;
use wiremock::matchers::{any, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn ok(data: serde_json::Value) -> ResponseTemplate {
    ResponseTemplate::new(200).set_body_json(json!({ "data": data, "error": null }))
}

async fn server() -> MockServer {
    let server = MockServer::start().await;
    Mock::given(method("POST")).and(path("/webhooks")).respond_with(ok(json!({"id": "ep1", "secret": "whsec_1"}))).mount(&server).await;
    Mock::given(method("POST")).and(path("/webhooks/ep1/rotate-secret")).respond_with(ok(json!({"secret": "whsec_2"}))).mount(&server).await;
    Mock::given(method("POST")).and(path("/webhooks/ep1/rotate")).respond_with(ok(json!({"secret": "whsec_2"}))).mount(&server).await;
    Mock::given(method("POST")).and(path("/secrets/ref1/reveal")).respond_with(ok(json!({"value": "postgres://x"}))).mount(&server).await;
    Mock::given(any()).respond_with(ok(json!({"ok": true, "lines": ["GET / 200", {"message": "boom", "level": "error", "at": "10:00"}, {"no": "message"}, 3], "deployments": []}))).mount(&server).await;
    server
}

fn config(server: &MockServer) -> ShipyardConfig {
    ShipyardConfig { base_url: server.uri(), key: "manage".into(), reveal_key: Some("reveal".into()), webhook_secret: None }
}

#[tokio::test]
async fn every_call_reaches_shipyard_and_reads_its_envelope() {
    let server = server().await;
    let config = config(&server);
    let http = reqwest::Client::new();
    let s = Shipyard::new(&http, &config);
    let (o, a, e) = ("org1", "shop", "production");

    s.put_org(o, "Acme").await.unwrap();
    s.get_org(o).await.unwrap();
    s.delete_org(o).await.unwrap();
    s.put_app(o, a, "Shop", true).await.unwrap();
    s.get_app(o, a).await.unwrap();
    s.delete_app(o, a).await.unwrap();
    s.ensure_dev_database(o, a).await.unwrap();
    s.get_publish(o, a, e, "pub1").await.unwrap();
    s.get_env(o, a, e).await.unwrap();
    s.check_domain(o, DomainName::Hostname("shop.example.com"), Some((a, e))).await.unwrap();
    s.check_domain(o, DomainName::Subdomain("shop"), None).await.unwrap();
    s.get_domain(o, a, e).await.unwrap();
    s.set_domain(o, a, e, DomainName::Subdomain("shop")).await.unwrap();
    s.transfer_app(o, a, "org2", Some(a)).await.unwrap();
    s.transfer_app(o, a, "org2", None).await.unwrap();
    s.delete_env(o, a, "staging").await.unwrap();
    s.get_deployment(o, a, e, "d1").await.unwrap();
    let _ = s.deployment_failure(o, a, e, "d1").await;
    s.list_deployments(o, a, e, Some("web"), 5).await.unwrap();
    s.list_deployments(o, a, e, None, 5).await.unwrap();
    s.restart_env(o, a, e).await.unwrap();
    s.rollback_service(o, a, e, "web", Some("d0")).await.unwrap();
    s.rollback_service(o, a, e, "web", None).await.unwrap();
    let logs = s.service_logs(o, a, e, "web", 50, Some("2026-11-01T00:00:00Z")).await.unwrap();
    assert_eq!(nomi_shipyard::log_lines(&logs), vec!["GET / 200".to_string(), "10:00 error boom".to_string()]);
    s.usage(o, a, e).await.unwrap();
    for range in ["1h", "24h", "7d"] {
        let range = MetricsRange::parse(range).unwrap();
        s.metrics(o, a, e, range, Some(range.default_step())).await.unwrap();
        s.metrics(o, a, e, range, None).await.unwrap();
    }
    assert!(MetricsRange::parse("1y").is_none());
    assert!(MetricsStep::parse("nope").is_none());
    s.list_secrets(o, a).await.unwrap();
    assert_eq!(s.reveal("ref1").await.unwrap(), "postgres://x");
    s.set_secret(o, a, "API_KEY", Some("v"), Some(e)).await.unwrap();
    s.set_secret(o, a, "API_KEY", None, None).await.unwrap();
    s.generate_secret(o, a, "SESSION_SECRET", "random", None).await.unwrap();
    s.delete_secret(o, a, "API_KEY", Some(e)).await.unwrap();
    s.delete_secret(o, a, "API_KEY", None).await.unwrap();
    s.duplicate_env(o, a, e, "staging", true).await.unwrap();
    s.scale_env(o, a, e, 2).await.unwrap();
    assert_eq!(s.create_webhook("https://nomi.app/hooks/shipyard").await.unwrap(), ("ep1".into(), "whsec_1".into()));
    s.create_webhook_for("https://x.test/h", &["deployment.*".into()], Some(o)).await.unwrap();
    s.list_webhooks().await.unwrap();
    s.update_webhook("ep1", &json!({"enabled": false})).await.unwrap();
    let _ = s.rotate_webhook_secret("ep1").await;
    s.webhook_deliveries("ep1", Some("failed"), 10).await.unwrap();
    s.webhook_deliveries("ep1", None, 10).await.unwrap();
    s.redeliver("dl1").await.unwrap();
    s.test_webhook("ep1").await.unwrap();
    s.delete_webhook("ep1").await.unwrap();
    s.org_events(o, Some("ev1"), 20).await.unwrap();
    s.org_events(o, None, 20).await.unwrap();
    s.event_types().await.unwrap();
    assert!(server.received_requests().await.unwrap().len() > 40);
}

#[tokio::test]
async fn shipyard_errors_come_back_by_kind() {
    let server = MockServer::start().await;
    let err = |status: u16, code: &str| ResponseTemplate::new(status).set_body_json(json!({ "data": null, "error": { "code": code, "message": format!("{code} said") } }));
    Mock::given(path("/orgs/missing")).respond_with(err(404, "NOT_FOUND")).mount(&server).await;
    Mock::given(path("/orgs/locked")).respond_with(err(401, "UNAUTHORIZED")).mount(&server).await;
    Mock::given(path("/orgs/busy")).respond_with(err(409, "CONFLICT")).mount(&server).await;
    Mock::given(path("/orgs/down")).respond_with(ResponseTemplate::new(503).set_body_string("down")).mount(&server).await;
    let config = config(&server);
    let http = reqwest::Client::new();
    let s = Shipyard::new(&http, &config);
    assert!(matches!(s.get_org("missing").await, Err(ShipyardError::NotFound(_))));
    assert!(matches!(s.get_org("locked").await, Err(ShipyardError::Unauthorized(_))));
    let conflict = s.get_org("busy").await.unwrap_err();
    assert!(matches!(conflict, ShipyardError::Rejected { status: 409, .. }));
    assert!(conflict.to_string().contains("CONFLICT said"));
    assert!(matches!(s.get_org("down").await, Err(ShipyardError::Unavailable(_))));

    let no_reveal = ShipyardConfig { reveal_key: None, ..config.clone() };
    assert!(matches!(Shipyard::new(&http, &no_reveal).reveal("r").await, Err(ShipyardError::Unauthorized(_))));
    let nowhere = ShipyardConfig { base_url: "http://127.0.0.1:9".into(), ..config };
    assert!(matches!(Shipyard::new(&http, &nowhere).get_org("x").await, Err(ShipyardError::Unavailable(_))));
}

#[test]
fn app_slugs_are_short_and_safe() {
    let slug = nomi_shipyard::app_slug("My Bakery & Café!!", "4f2c1a9b8e7d");
    assert!(slug.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-'), "{slug}");
    std::env::remove_var("SHIPYARD_URL");
    let _ = ShipyardConfig::from_env();
}
