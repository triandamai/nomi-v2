//! A project's database for its admins: a dump as a SQL file, and fixing a row by its primary
//! key with the columns checked against the table. Shipyard is faked; the "project database" is
//! this test's own database.

mod support;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::json;
use sqlx::PgPool;
use support::App;
use wiremock::matchers::{method, path, path_regex};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn ok(data: serde_json::Value) -> ResponseTemplate {
    ResponseTemplate::new(200).set_body_json(json!({ "data": data, "error": null }))
}

async fn shipyard(pool: &PgPool) -> MockServer {
    let server = MockServer::start().await;
    let name: String = sqlx::query_scalar("SELECT current_database()")
        .fetch_one(pool)
        .await
        .unwrap();
    let base = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://nomi:nomi@localhost:5432/nomi".into());
    let mut url = reqwest::Url::parse(&base).unwrap();
    url.set_path(&name);
    Mock::given(method("PUT"))
        .and(path_regex(r"^/orgs/[^/]+$"))
        .respond_with(ok(json!({"created": true})))
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path_regex(r"^/orgs/[^/]+/apps/[^/]+$"))
        .respond_with(ok(json!({"created": true})))
        .mount(&server)
        .await;
    Mock::given(method("PUT"))
        .and(path_regex(r"/dev-database$"))
        .respond_with(ok(json!({"owner_url_ref": "dev-ref"})))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/secrets/dev-ref/reveal"))
        .respond_with(ok(json!({"value": url.to_string()})))
        .mount(&server)
        .await;
    server
}

fn with_shipyard(pool: PgPool, server: &MockServer) -> App {
    let mut state = support::state(pool.clone());
    state.shipyard = Some(nomi_shipyard::ShipyardConfig {
        base_url: server.uri(),
        key: "ship_manage".into(),
        reveal_key: Some("ship_reveal".into()),
        webhook_secret: None,
    });
    App::with_state(pool, state)
}

#[sqlx::test(migrations = "../../migrations")]
async fn admins_fix_a_row_by_its_primary_key(pool: PgPool) {
    let server = shipyard(&pool).await;
    let app = with_shipyard(pool.clone(), &server);
    let owner = app.person("owner@dbedit.test").await;
    let other = app.person("other@dbedit.test").await;
    let (project, _) = app.project(&owner, "Todo").await;
    sqlx::query("CREATE TABLE task (id serial PRIMARY KEY, title text NOT NULL, done boolean DEFAULT false, meta jsonb, shout text GENERATED ALWAYS AS (upper(title)) STORED)")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("CREATE TABLE note (body text)")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO task (title) VALUES ('buy milk')")
        .execute(&pool)
        .await
        .unwrap();
    let rows = format!("/api/projects/{project}/database/tables/public/task/rows");

    // The tables say which columns pick out a row.
    let (_, tables) = app
        .get(&format!("/api/projects/{project}/database/tables"), &owner)
        .await;
    let listed = |name: &str| {
        tables
            .as_array()
            .unwrap()
            .iter()
            .find(|t| t["name"] == name)
            .cloned()
            .unwrap()
    };
    assert_eq!(listed("task")["primary_key"], json!(["id"]));
    assert_eq!(listed("note")["primary_key"], json!([]));

    assert_eq!(
        app.send(
            "PATCH",
            &rows,
            &other,
            json!({"key": {"id": 1}, "changes": {"title": "x"}})
        )
        .await
        .0,
        StatusCode::NOT_FOUND
    );

    let (status, edited) = app.send("PATCH", &rows, &owner, json!({"key": {"id": 1}, "changes": {"title": "buy oat milk", "done": true, "meta": {"aisle": 3}}})).await;
    assert_eq!(status, StatusCode::OK, "{edited}");
    assert_eq!(edited["row"]["title"], "buy oat milk");
    assert_eq!(edited["row"]["done"], true);
    assert_eq!(edited["row"]["meta"], json!({"aisle": 3}));
    assert_eq!(edited["row"]["shout"], "BUY OAT MILK");
    let (_, cleared) = app
        .send(
            "PATCH",
            &rows,
            &owner,
            json!({"key": {"id": "1"}, "changes": {"meta": null}}),
        )
        .await;
    assert!(cleared["row"]["meta"].is_null(), "{cleared}");

    let refused = [
        (
            json!({"key": {"id": 1}, "changes": {}}),
            StatusCode::BAD_REQUEST,
        ),
        (
            json!({"key": {"title": "buy oat milk"}, "changes": {"done": false}}),
            StatusCode::BAD_REQUEST,
        ),
        (
            json!({"key": {"id": 1}, "changes": {"nope": 1}}),
            StatusCode::BAD_REQUEST,
        ),
        (
            json!({"key": {"id": 1}, "changes": {"shout": "x"}}),
            StatusCode::BAD_REQUEST,
        ),
        (
            json!({"key": {"id": null}, "changes": {"done": false}}),
            StatusCode::BAD_REQUEST,
        ),
        (
            json!({"key": {"id": 1}, "changes": {"done": "maybe"}}),
            StatusCode::UNPROCESSABLE_ENTITY,
        ),
        (
            json!({"key": {"id": 1}, "changes": {"title": null}}),
            StatusCode::UNPROCESSABLE_ENTITY,
        ),
        (
            json!({"key": {"id": 99}, "changes": {"done": false}}),
            StatusCode::NOT_FOUND,
        ),
    ];
    for (body, expected) in refused {
        let (status, answer) = app.send("PATCH", &rows, &owner, body.clone()).await;
        assert_eq!(status, expected, "{body} → {answer}");
    }
    // A table name that isn't one, and one without a primary key.
    let (status, _) = app
        .send(
            "PATCH",
            &format!("/api/projects/{project}/database/tables/public/task%3B%20drop/rows"),
            &owner,
            json!({"key": {"id": 1}, "changes": {"done": true}}),
        )
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (status, _) = app
        .send(
            "PATCH",
            &format!("/api/projects/{project}/database/tables/public/note/rows"),
            &owner,
            json!({"key": {"body": "a"}, "changes": {"body": "b"}}),
        )
        .await;
    assert_eq!(status, StatusCode::CONFLICT);
    let title: String = sqlx::query_scalar("SELECT title FROM task WHERE id = 1")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(title, "buy oat milk");
}

#[sqlx::test(migrations = "../../migrations")]
async fn admins_download_a_dump(pool: PgPool) {
    let server = shipyard(&pool).await;
    let app = with_shipyard(pool.clone(), &server);
    let owner = app.person("owner@dbdump.test").await;
    let other = app.person("other@dbdump.test").await;
    let (project, _) = app.project(&owner, "Warung Kopi").await;
    sqlx::query("CREATE TYPE mood AS ENUM ('ok', 'great')")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("CREATE TABLE menu (id integer GENERATED ALWAYS AS IDENTITY PRIMARY KEY, name text NOT NULL UNIQUE, price numeric(10,2), feel mood)")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO menu (name, price, feel) VALUES ('kopi O''s', 12.5, 'great')")
        .execute(&pool)
        .await
        .unwrap();
    let uri = format!("/api/projects/{project}/database/dump");

    assert_eq!(app.get(&uri, &other).await.0, StatusCode::NOT_FOUND);
    let request = Request::builder()
        .uri(&uri)
        .header("authorization", format!("Bearer {}", owner.token))
        .body(Body::empty())
        .unwrap();
    let (status, headers, body) = app.raw(request).await;
    assert_eq!(status, StatusCode::OK);
    let disposition = headers["content-disposition"].to_str().unwrap();
    assert!(
        disposition.starts_with("attachment; filename=\"warung-kopi-development-")
            && disposition.ends_with(".sql\""),
        "{disposition}"
    );
    let sql = String::from_utf8(body).unwrap();
    assert!(
        sql.contains("CREATE TYPE \"public\".\"mood\" AS ENUM ('ok', 'great');"),
        "enum"
    );
    assert!(sql.contains("CREATE TABLE \"public\".\"menu\" (\n    \"id\" integer GENERATED ALWAYS AS IDENTITY NOT NULL"), "table");
    assert!(sql.contains("INSERT INTO \"public\".\"menu\" (\"id\", \"name\", \"price\", \"feel\") OVERRIDING SYSTEM VALUE VALUES ('1', 'kopi O''s', '12.50', 'great');"), "row");
    assert!(sql.trim_end().ends_with("COMMIT;"));

    // It restores: the whole database, into an empty one.
    use sqlx::{Connection, Executor};
    let restored = format!("restore_{}", uuid::Uuid::new_v4().simple());
    pool.execute(format!("CREATE DATABASE {restored}").as_str())
        .await
        .unwrap();
    let base = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://nomi:nomi@localhost:5432/nomi".into());
    let mut url = reqwest::Url::parse(&base).unwrap();
    url.set_path(&restored);
    let mut conn = sqlx::PgConnection::connect(url.as_str()).await.unwrap();
    let applied = conn.execute(sql.as_str()).await;
    let name: Option<String> = match &applied {
        Ok(_) => sqlx::query_scalar("SELECT name FROM menu WHERE id = 1")
            .fetch_optional(&mut conn)
            .await
            .unwrap(),
        Err(_) => None,
    };
    conn.close().await.unwrap();
    pool.execute(format!("DROP DATABASE {restored}").as_str())
        .await
        .unwrap();
    applied.unwrap();
    assert_eq!(name.as_deref(), Some("kopi O's"));
}
