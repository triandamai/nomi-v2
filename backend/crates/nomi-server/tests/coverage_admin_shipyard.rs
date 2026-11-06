//! Admin → Integrations → Shipyard beyond what admin_integrations_routes covers: the apps on
//! Shipyard and linking them to projects, webhook tests and deliveries, environments and secrets.

mod support;

use axum::http::StatusCode;
use serde_json::json;
use sqlx::PgPool;
use support::App;
use wiremock::matchers::{method, path, path_regex};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn ok(data: serde_json::Value) -> ResponseTemplate {
    ResponseTemplate::new(200).set_body_json(json!({ "data": data, "error": null }))
}

async fn app_with_shipyard(pool: PgPool) -> (App, MockServer) {
    let server = MockServer::start().await;
    let mut state = support::state(pool.clone());
    state.shipyard = Some(nomi_shipyard::ShipyardConfig { base_url: server.uri(), key: "manage".into(), reveal_key: Some("reveal".into()), webhook_secret: None });
    (App::with_state(pool, state), server)
}

#[sqlx::test(migrations = "../../migrations")]
async fn apps_on_shipyard_are_listed_and_linked_to_projects(pool: PgPool) {
    let (app, shipyard) = app_with_shipyard(pool.clone()).await;
    let admin = app.admin("root@nomi.test").await;
    let owner = app.person("owner@acme.test").await;
    let viewer = app.person("viewer@acme.test").await;
    let (linked, _) = app.project(&owner, "Shop").await;
    let (unlinked, _) = app.project(&owner, "Blog").await;
    sqlx::query("UPDATE projects SET shipyard_app = 'shop' WHERE id = $1").bind(linked).execute(&pool).await.unwrap();
    let org: uuid::Uuid = sqlx::query_scalar("SELECT org_id FROM projects WHERE id = $1").bind(linked).fetch_one(&pool).await.unwrap();

    Mock::given(method("GET")).and(path(format!("/orgs/{org}"))).respond_with(ok(json!({"apps": [
        {"key": "shop", "name": "Shop", "environments": ["production", "staging"]},
        {"key": "shop-backend", "name": "Shop backend", "environments": ["production"]},
        {"key": "blog", "environments": ["production"]},
        {"key": "", "name": "nameless"}
    ]}))).mount(&shipyard).await;
    Mock::given(method("GET")).and(path_regex(r"^/orgs/[^/]+$")).respond_with(ResponseTemplate::new(404).set_body_json(json!({"data": null, "error": {"code": "not_found", "message": "no org"}}))).mount(&shipyard).await;

    assert_eq!(app.get("/api/admin/shipyard/apps", &viewer).await.0, StatusCode::FORBIDDEN);
    let (status, overview) = app.get("/api/admin/shipyard/apps", &admin).await;
    assert_eq!(status, StatusCode::OK, "{overview}");
    let apps = overview["apps"].as_array().unwrap();
    assert_eq!(apps.len(), 3);
    assert_eq!(apps[0]["key"], "blog", "orphans first");
    assert_eq!(apps.iter().find(|a| a["key"] == "shop-backend").unwrap()["project"]["name"], "Shop", "a backend goes with its app's project");
    assert!(overview["candidates"][0]["projects"].as_array().unwrap().iter().any(|p| p["id"] == unlinked.to_string()));

    // Link "blog" to the Blog project; "shop" is taken; an app that isn't there is refused.
    let link = |app_key: &str, project: Option<uuid::Uuid>| json!({"org_id": org, "app": app_key, "project_id": project});
    let (status, body) = app.send("POST", "/api/admin/shipyard/apps/link", &admin, link("blog", Some(unlinked))).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["name"], "Blog");
    assert_eq!(app.send("POST", "/api/admin/shipyard/apps/link", &admin, link("shop", Some(unlinked))).await.0, StatusCode::CONFLICT);
    assert_eq!(app.send("POST", "/api/admin/shipyard/apps/link", &admin, link("ghost", None)).await.0, StatusCode::NOT_FOUND);
    // Unlinked and with no project given: a new project, owned by the organization's owner.
    sqlx::query("UPDATE projects SET shipyard_app = NULL WHERE id = $1").bind(linked).execute(&pool).await.unwrap();
    let (status, made) = app.send("POST", "/api/admin/shipyard/apps/link", &admin, link("shop", None)).await;
    assert_eq!(status, StatusCode::OK, "{made}");
    assert_eq!(made["name"], "Shop");
    let owner_of_new: uuid::Uuid = sqlx::query_scalar("SELECT user_id FROM projects WHERE id = $1").bind(made["id"].as_str().unwrap().parse::<uuid::Uuid>().unwrap()).fetch_one(&pool).await.unwrap();
    assert_eq!(owner_of_new, owner.id);
}

#[sqlx::test(migrations = "../../migrations")]
async fn webhooks_environments_and_secrets_go_through_to_shipyard(pool: PgPool) {
    let (app, shipyard) = app_with_shipyard(pool.clone()).await;
    let admin = app.admin("root@nomi.test").await;
    let owner = app.person("owner@acme.test").await;
    let (project, _) = app.project(&owner, "Shop").await;
    sqlx::query("UPDATE projects SET shipyard_app = 'shop' WHERE id = $1").bind(project).execute(&pool).await.unwrap();
    let org: uuid::Uuid = sqlx::query_scalar("SELECT org_id FROM projects WHERE id = $1").bind(project).fetch_one(&pool).await.unwrap();
    Mock::given(method("POST")).and(path_regex(r"/webhooks$")).respond_with(ok(json!({"id": "ep9", "secret": "whsec"}))).mount(&shipyard).await;
    Mock::given(wiremock::matchers::any()).respond_with(ok(json!({"ok": true, "secrets": [{"name": "API_KEY", "ref": "r1", "environment": "production"}]}))).mount(&shipyard).await;

    let calls: Vec<(&str, String, Option<serde_json::Value>)> = vec![
        ("POST", "/api/admin/shipyard/webhooks/ep1/test".into(), None),
        ("GET", "/api/admin/shipyard/webhooks/ep1/deliveries?status=failed".into(), None),
        ("POST", "/api/admin/shipyard/deliveries/d1/redeliver".into(), None),
        ("POST", format!("/api/admin/shipyard/orgs/{org}/sync"), None),
        ("GET", format!("/api/admin/shipyard/orgs/{org}/apps/shop"), None),
        ("GET", format!("/api/admin/shipyard/orgs/{org}/apps/shop/secrets"), None),
        ("POST", format!("/api/admin/shipyard/orgs/{org}/apps/shop/envs/production/duplicate"), Some(json!({"to": "staging"}))),
        ("DELETE", format!("/api/admin/shipyard/orgs/{org}/apps/shop/envs/staging"), None),
        ("DELETE", format!("/api/admin/shipyard/orgs/{org}/apps/shop/secrets/API_KEY?environment=production"), None),
    ];
    for (verb, uri, body) in calls {
        let (status, answer) = app.call(verb, &uri, Some(&admin.token), body).await;
        assert!(status.is_success(), "{verb} {uri}: {status} {answer}");
    }

    let (status, created) = app.send("POST", "/api/admin/shipyard/webhooks", &admin, json!({"url": "https://hooks.example.com/in", "events": ["deployment.*", " ", "deployment.*"]})).await;
    assert!(status.is_success(), "{status} {created}");
    for bad in [json!({"url": "ftp://files.example.com/x", "events": ["*"]}), json!({"url": "https://hooks.example.com", "events": []}), json!({"url": "https://hooks.example.com", "events": ["Bad Event!"]})] {
        assert_eq!(app.send("POST", "/api/admin/shipyard/webhooks", &admin, bad).await.0, StatusCode::BAD_REQUEST);
    }

    for bad_env in ["bad env", "-x", ""] {
        let (status, _) = app.send("POST", &format!("/api/admin/shipyard/orgs/{org}/apps/shop/envs/production/duplicate"), &admin, json!({"to": bad_env})).await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{bad_env:?}");
    }

    // Taking the app down unlinks it from its project.
    assert_eq!(app.call("DELETE", &format!("/api/admin/shipyard/orgs/{org}/apps/shop"), Some(&admin.token), None).await.0, StatusCode::NO_CONTENT);
    let app_key: Option<String> = sqlx::query_scalar("SELECT shipyard_app FROM projects WHERE id = $1").bind(project).fetch_one(&pool).await.unwrap();
    assert_eq!(app_key, None);
}
