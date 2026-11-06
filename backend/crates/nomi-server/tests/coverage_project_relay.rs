//! A project's Relay backend as its preview and data sheet see it, with Relay faked.

mod support;

use axum::http::StatusCode;
use serde_json::json;
use sqlx::PgPool;
use support::App;
use uuid::Uuid;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn envelope(data: serde_json::Value) -> ResponseTemplate {
    ResponseTemplate::new(200).set_body_json(json!({ "data": data, "error": null }))
}

async fn relay(project: Uuid) -> MockServer {
    let server = MockServer::start().await;
    let base = format!("/control/projects/{project}");
    Mock::given(method("GET")).and(path(base.clone())).respond_with(envelope(json!({
        "id": project, "slug": "todo", "tier": "free",
        "environments": {
            "dev": {"url": "https://todo-dev.relay.test", "anon_key": "dev-anon", "service_key": "dev-service"},
            "production": {"url": "https://todo.relay.test", "anon_key": "prod-anon", "service_key": "prod-service"}
        }
    }))).mount(&server).await;
    for env in ["dev", "production"] {
        Mock::given(method("GET")).and(path(format!("{base}/environments/{env}/schema"))).respond_with(envelope(json!({"tables": [{"name": "todos"}]}))).mount(&server).await;
        Mock::given(method("GET")).and(path(format!("{base}/environments/{env}/auth"))).respond_with(envelope(json!({"providers": ["email"]}))).mount(&server).await;
        Mock::given(method("POST")).and(path(format!("{base}/environments/{env}/query"))).respond_with(envelope(json!({"columns": ["n"], "rows": [[1]], "truncated": false}))).mount(&server).await;
    }
    server
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_relay_project_s_data_sheet_preview_and_sql(pool: PgPool) {
    let app = App::new(pool.clone());
    let owner = app.person("owner@acme.test").await;
    let stranger = app.person("stranger@acme.test").await;
    let (project, _) = app.project(&owner, "Todo").await;
    let base = format!("/api/projects/{project}/relay");

    // Kept in Shipyard's Postgres: the sheet uses its own database view.
    assert_eq!(app.get(&base, &owner).await, (StatusCode::OK, json!({"backend": "drizzle"})));
    let (status, _) = app.send("POST", &format!("{base}/query"), &owner, json!({"sql": "select 1"})).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(app.get(&format!("{base}/preview"), &owner).await.0, StatusCode::NOT_FOUND);

    sqlx::query("UPDATE projects SET backend = 'relay' WHERE id = $1").bind(project).execute(&pool).await.unwrap();
    // On Relay, but Relay isn't set up on this Nomi.
    std::env::remove_var("RELAY_CONTROL_URL");
    std::env::remove_var("RELAY_URL");
    assert_eq!(app.send("POST", &format!("{base}/query"), &owner, json!({"sql": "select 1"})).await.0, StatusCode::SERVICE_UNAVAILABLE);

    let server = relay(project).await;
    nomi_server::relay_connection::save(&pool, &nomi_test_support::TEST_SETTINGS_KEY, &server.uri(), Some("relay-key"), owner.id).await.unwrap();

    let (status, sheet) = app.get(&base, &owner).await;
    assert_eq!(status, StatusCode::OK, "{sheet}");
    assert_eq!(sheet["backend"], "relay");
    assert_eq!(sheet["url"], "https://todo-dev.relay.test");
    assert_eq!(sheet["auth"]["providers"], json!(["email"]));
    let (_, prod) = app.get(&format!("{base}?source=production"), &owner).await;
    assert_eq!(prod["anon_key"], "prod-anon");

    let (status, rows) = app.send("POST", &format!("{base}/query"), &owner, json!({"sql": "select 1 as n"})).await;
    assert_eq!(status, StatusCode::OK, "{rows}");
    assert_eq!(rows["rows"], json!([[1]]));

    let (status, access) = app.get(&format!("{base}/preview"), &owner).await;
    assert_eq!(status, StatusCode::OK, "{access}");
    assert_eq!(access["service_key"], "dev-service", "an admin's preview gets the service key");
    assert_ne!(app.get(&format!("{base}/preview"), &stranger).await.0, StatusCode::OK);
    assert_eq!(app.get(&base, &stranger).await.0, StatusCode::NOT_FOUND);

    // Not a backend Nomi set up itself: nothing to retry.
    assert_eq!(app.call("POST", &format!("{base}/retry"), Some(&owner.token), None).await.0, StatusCode::NOT_FOUND);
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_backend_still_being_set_up_shows_its_step_and_retries_only_after_failing(pool: PgPool) {
    let app = App::new(pool.clone());
    let owner = app.person("owner@acme.test").await;
    let (project, _) = app.project(&owner, "Todo").await;
    sqlx::query("UPDATE projects SET backend = 'relay' WHERE id = $1").bind(project).execute(&pool).await.unwrap();
    nomi_server::project_relays::begin(&pool, &nomi_test_support::TEST_SETTINGS_KEY, project, "dev", "control", "1").await.unwrap();
    let base = format!("/api/projects/{project}/relay");

    let (status, sheet) = app.get(&base, &owner).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(sheet["setup"]["state"], "creating");
    assert_eq!(app.call("POST", &format!("{base}/retry"), Some(&owner.token), None).await.0, StatusCode::CONFLICT);

    sqlx::query("UPDATE project_relays SET state = 'failed', error = 'boom' WHERE project_id = $1").bind(project).execute(&pool).await.unwrap();
    let (_, sheet) = app.get(&base, &owner).await;
    assert_eq!(sheet["setup"]["error"], "boom");
    // Failed, but publishing (Shipyard) isn't set up here to make it again.
    assert_eq!(app.call("POST", &format!("{base}/retry"), Some(&owner.token), None).await.0, StatusCode::SERVICE_UNAVAILABLE);
}
