//! A site's Monitor beyond Shipyard's traffic: where visits came from and the forms sent (as the
//! published site reports them), and uptime (as Nomi's own look at the address finds it).

mod support;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::json;
use sqlx::PgPool;
use support::App;
use uuid::Uuid;
use wiremock::matchers::{method, path_regex};
use wiremock::{Mock, MockServer, ResponseTemplate};

async fn report(app: &App, project: Uuid, body: &str) -> StatusCode {
    let request = Request::builder()
        .method("POST")
        .uri(format!("/api/hooks/sites/{project}"))
        .header("content-type", "text/plain")
        .body(Body::from(body.to_string()))
        .unwrap();
    app.raw(request).await.0
}

#[sqlx::test(migrations = "../../migrations")]
async fn sources_form_sends_and_uptime_join_the_metrics(pool: PgPool) {
    let shipyard = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path_regex(r"^/orgs/[^/]+/apps/[^/]+/envs/production/metrics$"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"data": {"range": "1h", "buckets": [], "totals": {"visitors": 3}}, "error": null})))
        .mount(&shipyard)
        .await;
    let site = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&site)
        .await;

    let mut state = support::state(pool.clone());
    state.shipyard = Some(nomi_shipyard::ShipyardConfig {
        base_url: shipyard.uri(),
        key: "ship_manage".into(),
        reveal_key: None,
        webhook_secret: None,
    });
    let app = App::with_state(pool.clone(), state);
    let owner = app.person("owner@visits.test").await;
    let (project, _) = app.project(&owner, "Studio").await;

    // Not live: reports are dropped.
    assert_eq!(
        report(&app, project, r#"{"k":"v","r":"","h":"studio"}"#).await,
        StatusCode::NO_CONTENT
    );
    sqlx::query("UPDATE projects SET shipyard_app = 'studio' WHERE id = $1")
        .bind(project)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO project_publishes (project_id, environment, status, url, manifest) VALUES ($1, 'production', 'succeeded', $2, '{}')")
        .bind(project)
        .bind(site.uri())
        .execute(&pool)
        .await
        .unwrap();

    let metrics = format!("/api/projects/{project}/deploy/metrics?range=1h");
    let (status, before) = app.get(&metrics, &owner).await;
    assert_eq!(status, StatusCode::OK, "{before}");
    assert_eq!(before["totals"]["visitors"], 3);
    assert!(
        before["sources"].is_null() && before["form_sends"].is_null() && before["uptime"].is_null(),
        "nothing measured yet: {before}"
    );

    for body in [
        r#"{"k":"v","r":"https://l.instagram.com/","h":"studio.test"}"#,
        r#"{"k":"v","r":"https://www.instagram.com/p/1","h":"studio.test"}"#,
        r#"{"k":"v","r":"","h":"studio.test"}"#,
        r#"{"k":"v","r":"https://studio.test/about","h":"studio.test"}"#,
        r#"{"k":"f","r":"https://studio.test/","h":"studio.test"}"#,
        r#"{"k":"x"}"#,
        "not json",
    ] {
        assert_eq!(report(&app, project, body).await, StatusCode::NO_CONTENT);
    }
    assert_eq!(
        report(&app, project, &"x".repeat(5000)).await,
        StatusCode::NO_CONTENT
    );

    assert_eq!(
        nomi_server::routes::site_visits::probe_due(&pool, &reqwest::Client::new()).await,
        1
    );
    assert_eq!(
        nomi_server::routes::site_visits::probe_due(&pool, &reqwest::Client::new()).await,
        0,
        "looked at a moment ago"
    );

    let (_, after) = app.get(&metrics, &owner).await;
    let sources = after["sources"].as_array().unwrap();
    assert_eq!(sources.len(), 2, "{after}");
    assert_eq!(sources[0]["name"], "Instagram");
    assert!((sources[0]["share"].as_f64().unwrap() - 2.0 / 3.0).abs() < 1e-9);
    assert_eq!(sources[1]["name"], "Direct");
    assert_eq!(after["form_sends"], 1);
    assert_eq!(after["uptime"]["share"], 1.0);
    assert!(after["uptime"]["since"].is_string());
}
