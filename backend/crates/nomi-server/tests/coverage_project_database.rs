//! A project's databases for its admins: where they are, the login on request, the tables, and
//! SQL run as the owner. Shipyard is faked; the "project database" is this test's own database.

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

/// This test's database, as a login Shipyard would reveal.
async fn own_database_url(pool: &PgPool) -> String {
    let name: String = sqlx::query_scalar("SELECT current_database()").fetch_one(pool).await.unwrap();
    let base = std::env::var("DATABASE_URL").unwrap_or_else(|_| "postgres://nomi:nomi@localhost:5432/nomi".into());
    let mut url = reqwest::Url::parse(&base).unwrap();
    url.set_path(&name);
    url.to_string()
}

async fn shipyard(pool: &PgPool) -> MockServer {
    let server = MockServer::start().await;
    let url = own_database_url(pool).await;
    Mock::given(method("PUT")).and(path_regex(r"^/orgs/[^/]+$")).respond_with(ok(json!({"created": true}))).mount(&server).await;
    Mock::given(method("PUT")).and(path_regex(r"^/orgs/[^/]+/apps/[^/]+$")).respond_with(ok(json!({"created": true}))).mount(&server).await;
    Mock::given(method("PUT")).and(path_regex(r"/dev-database$")).respond_with(ok(json!({"owner_url_ref": "dev-ref"}))).mount(&server).await;
    Mock::given(method("GET")).and(path_regex(r"/secrets$"))
        .respond_with(ok(json!({"secrets": [{"name": "DATABASE_URL", "environment": "staging", "ref": "nope"}, {"name": "DATABASE_URL", "environment": "production", "ref": "prod-ref"}]})))
        .mount(&server).await;
    Mock::given(method("POST")).and(path("/secrets/dev-ref/reveal")).respond_with(ok(json!({"value": url.clone()}))).mount(&server).await;
    Mock::given(method("POST")).and(path("/secrets/prod-ref/reveal")).respond_with(ok(json!({"value": url}))).mount(&server).await;
    server
}

fn with_shipyard(pool: PgPool, server: &MockServer) -> App {
    with_shipyard_and_storage(pool, server).0
}

fn with_shipyard_and_storage(pool: PgPool, server: &MockServer) -> (App, nomi_storage::ProjectStore) {
    let mut state = support::state(pool.clone());
    state.shipyard = Some(nomi_shipyard::ShipyardConfig {
        base_url: server.uri(),
        key: "ship_manage".into(),
        reveal_key: Some("ship_reveal".into()),
        webhook_secret: None,
    });
    let storage = state.project_storage.clone();
    (App::with_state(pool, state), storage)
}

#[sqlx::test(migrations = "../../migrations")]
async fn admins_see_the_database_its_tables_and_run_sql(pool: PgPool) {
    let server = shipyard(&pool).await;
    let app = with_shipyard(pool.clone(), &server);
    let owner = app.person("owner@acme.test").await;
    let other = app.person("other@acme.test").await;
    let (project, _) = app.project(&owner, "Todo").await;
    let base = format!("/api/projects/{project}/database");
    sqlx::query("CREATE TABLE task (id serial primary key, title text)").execute(&pool).await.unwrap();

    assert_eq!(app.get(&base, &other).await.0, StatusCode::NOT_FOUND);

    let (status, info) = app.get(&base, &owner).await;
    assert_eq!(status, StatusCode::OK, "{info}");
    assert_eq!(info["source"], "development");
    assert_eq!(info["host"], "localhost");
    assert!(info["database"].as_str().unwrap().starts_with("_sqlx_test"), "{info}");

    let (status, login) = app.call("POST", &format!("{base}/credentials"), Some(&owner.token), None).await;
    assert_eq!(status, StatusCode::OK);
    assert!(login["url"].as_str().unwrap().starts_with("postgres://"));

    let (status, tables) = app.get(&format!("{base}/tables"), &owner).await;
    assert_eq!(status, StatusCode::OK);
    assert!(tables.as_array().unwrap().iter().any(|t| t["name"] == "task"), "{tables}");

    let query = format!("{base}/query");
    let (status, wrote) = app.send("POST", &query, &owner, json!({"sql": "INSERT INTO task (title) VALUES ('a'), ('b');"})).await;
    assert_eq!(status, StatusCode::OK, "{wrote}");
    assert_eq!(wrote["rows_affected"], 2);
    let (status, read) = app.send("POST", &query, &owner, json!({"sql": "-- newest first\nSELECT id, title FROM task ORDER BY id DESC"})).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(read["columns"], json!(["id", "title"]));
    assert_eq!(read["rows"][0][1], "b");
    assert_eq!(read["truncated"], false);
    let (status, many) = app.send("POST", &query, &owner, json!({"sql": "SELECT g FROM generate_series(1, 600) g"})).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(many["rows"].as_array().unwrap().len(), 500);
    assert_eq!(many["truncated"], true);

    let (status, error) = app.send("POST", &query, &owner, json!({"sql": "SELECT nope FROM task"})).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(error.as_str().unwrap().contains("nope"), "{error}");
    assert_eq!(app.send("POST", &query, &owner, json!({"sql": " ; "})).await.0, StatusCode::BAD_REQUEST);

    // Production reads the published environment's own login.
    let (status, prod) = app.get(&format!("{base}?source=production"), &owner).await;
    assert_eq!(status, StatusCode::OK, "{prod}");
    assert_eq!(prod["source"], "production");
}

#[sqlx::test(migrations = "../../migrations")]
async fn projects_without_a_database_or_a_publish_say_so(pool: PgPool) {
    let server = shipyard(&pool).await;
    let (app, storage) = with_shipyard_and_storage(pool.clone(), &server);
    let owner = app.person("owner@acme.test").await;
    let (project, _) = app.project(&owner, "Static site").await;

    // Not published: no production database yet.
    let (status, body) = app.get(&format!("/api/projects/{project}/database?source=production"), &owner).await;
    assert_eq!((status, body), (StatusCode::NOT_FOUND, json!("this project hasn't been published yet")));

    // A package.json without a database: there's nothing to open.
    let key = nomi_agent_coding::project_file_key(project, "package.json");
    storage.put_object(&key, r#"{"dependencies":{"svelte":"5"}}"#, "application/json").await.unwrap();
    let (status, body) = app.get(&format!("/api/projects/{project}/database"), &owner).await;
    assert_eq!((status, body), (StatusCode::NOT_FOUND, json!("This project does not use a database.")));

    // Without Shipyard set up at all.
    let plain = App::new(pool);
    let owner = plain.login("owner@acme.test").await;
    assert_ne!(plain.get(&format!("/api/projects/{project}/database/tables"), &owner).await.0, StatusCode::OK);
}
