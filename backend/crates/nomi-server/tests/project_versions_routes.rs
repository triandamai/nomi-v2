//! A project's versions and restoring one, renaming, duplicating and remixing a project, the
//! "needs you" / last publish / live fields on its summary, and searching chats, memory and files.

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use nomi_server::app::{build_router, AppState};
use serde_json::{json, Value};
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;

const SECRET: &str = "test-secret-do-not-use-in-prod";

fn test_state(pool: PgPool) -> AppState {
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
        shipyard: None,
    }
}

async fn call(
    router: &axum::Router,
    method: &str,
    uri: &str,
    body: Value,
    token: &str,
) -> (StatusCode, Value) {
    let request = Request::builder()
        .method(method)
        .uri(uri)
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .body(if body.is_null() {
            Body::empty()
        } else {
            Body::from(body.to_string())
        })
        .unwrap();
    let response = router.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    )
}

async fn file(
    router: &axum::Router,
    token: &str,
    project: &str,
    path: &str,
) -> (StatusCode, String) {
    let request = Request::builder()
        .uri(format!("/api/projects/{project}/files/{path}"))
        .header("authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap();
    let response = router.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    (status, String::from_utf8_lossy(&bytes).to_string())
}

/// Signs up `email` with a workspace picked for projects; their token and that workspace.
async fn person(router: &axum::Router, email: &str) -> (String, String) {
    let body = json!({"email": email, "password": "correct-password", "org": {"mode": "create", "name": "Acme"}});
    let request = Request::builder()
        .method("POST")
        .uri("/api/auth/register")
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap();
    router.clone().oneshot(request).await.unwrap();
    let request = Request::builder()
        .method("POST")
        .uri("/api/auth/login")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({"email": email, "password": "correct-password"}).to_string(),
        ))
        .unwrap();
    let response = router.clone().oneshot(request).await.unwrap();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let token = serde_json::from_slice::<Value>(&bytes).unwrap()["access_token"]
        .as_str()
        .unwrap()
        .to_string();
    let (_, orgs) = call(router, "GET", "/api/orgs", Value::Null, &token).await;
    let org = orgs["orgs"][0]["id"].as_str().unwrap().to_string();
    let (status, _) = call(
        router,
        "PUT",
        "/api/preferences/project-org",
        json!({ "org_id": org }),
        &token,
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    (token, org)
}

async fn new_project(router: &axum::Router, token: &str) -> (String, String) {
    let (status, body) = call(router, "POST", "/api/projects", Value::Null, token).await;
    assert_eq!(status, StatusCode::CREATED);
    (
        body["project_id"].as_str().unwrap().to_string(),
        body["session_id"].as_str().unwrap().to_string(),
    )
}

async fn put(router: &axum::Router, token: &str, project: &str, path: &str, content: &str) {
    let (status, _) = call(
        router,
        "PUT",
        &format!("/api/projects/{project}/files/{path}"),
        json!({ "content": content }),
        token,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
}

#[sqlx::test(migrations = "../../migrations")]
async fn versions_are_listed_and_any_one_is_restored_as_a_new_version(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let (token, _) = person(&router, "owner@example.com").await;
    let (project, _) = new_project(&router, &token).await;

    put(&router, &token, &project, "index.html", "v1").await;
    put(&router, &token, &project, "about.html", "about").await;
    put(&router, &token, &project, "index.html", "v3").await;
    let (status, _) = call(
        &router,
        "DELETE",
        &format!("/api/projects/{project}/files/about.html"),
        Value::Null,
        &token,
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let (status, body) = call(
        &router,
        "GET",
        &format!("/api/projects/{project}/versions"),
        Value::Null,
        &token,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let versions = body["versions"].as_array().unwrap();
    assert_eq!(versions.len(), 4);
    assert_eq!(body["current"], 4);
    assert_eq!(versions[0]["version"], 4);
    assert_eq!(versions[0]["current"], true);
    assert_eq!(versions[0]["files"], 1);
    assert_eq!(versions[0]["by"]["name"], "owner");
    assert_eq!(versions[2]["files"], 2);
    assert!(versions.iter().all(|v| v["restorable"] == true));

    // Paging.
    let (_, page) = call(
        &router,
        "GET",
        &format!("/api/projects/{project}/versions?before=3&limit=1"),
        Value::Null,
        &token,
    )
    .await;
    assert_eq!(page["versions"][0]["version"], 2);
    assert_eq!(page["has_more"], true);

    let (status, body) = call(
        &router,
        "POST",
        &format!("/api/projects/{project}/versions/2/restore"),
        Value::Null,
        &token,
    )
    .await;
    assert_eq!(status, StatusCode::ACCEPTED);
    assert_eq!(body["version"], 5);
    assert_eq!(file(&router, &token, &project, "index.html").await.1, "v1");
    assert_eq!(
        file(&router, &token, &project, "about.html").await,
        (StatusCode::OK, "about".to_string())
    );
    let (_, body) = call(
        &router,
        "GET",
        &format!("/api/projects/{project}/versions"),
        Value::Null,
        &token,
    )
    .await;
    assert_eq!(body["versions"][0]["restored_from"], 2);

    // And back again: version 4 had no about.html.
    let (status, _) = call(
        &router,
        "POST",
        &format!("/api/projects/{project}/versions/4/restore"),
        Value::Null,
        &token,
    )
    .await;
    assert_eq!(status, StatusCode::ACCEPTED);
    assert_eq!(
        file(&router, &token, &project, "about.html").await.0,
        StatusCode::NOT_FOUND
    );
    assert_eq!(file(&router, &token, &project, "index.html").await.1, "v3");

    let (status, _) = call(
        &router,
        "POST",
        &format!("/api/projects/{project}/versions/99/restore"),
        Value::Null,
        &token,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    // A version from before versions kept their files can't be restored.
    let id: Uuid = project.parse().unwrap();
    sqlx::query("INSERT INTO project_versions (project_id, version) VALUES ($1, 100)")
        .bind(id)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO project_version_files (project_id, version, path, content_type, size_bytes) VALUES ($1, 100, 'x.html', 'text/html', 1)")
        .bind(id)
        .execute(&pool)
        .await
        .unwrap();
    let (status, _) = call(
        &router,
        "POST",
        &format!("/api/projects/{project}/versions/100/restore"),
        Value::Null,
        &token,
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);

    // Someone else's project is not found.
    let (stranger, _) = person(&router, "stranger@example.com").await;
    let (status, _) = call(
        &router,
        "GET",
        &format!("/api/projects/{project}/versions"),
        Value::Null,
        &stranger,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (status, _) = call(
        &router,
        "POST",
        &format!("/api/projects/{project}/versions/2/restore"),
        Value::Null,
        &stranger,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[sqlx::test(migrations = "../../migrations")]
async fn files_saved_before_versions_are_kept_on_the_next_change(pool: PgPool) {
    let state = test_state(pool.clone());
    let storage = state.project_storage.clone();
    let router = build_router(state);
    let (token, _) = person(&router, "owner@example.com").await;
    let (project, _) = new_project(&router, &token).await;
    let id: Uuid = project.parse().unwrap();
    // A file from before: in storage, with no content hash.
    storage
        .put_bytes(
            &nomi_agent_coding::project_file_key(id, "old.css"),
            b"body{}".to_vec(),
            "text/css",
        )
        .await
        .unwrap();
    sqlx::query("INSERT INTO project_files (project_id, path, content_type, size_bytes) VALUES ($1, 'old.css', 'text/css', 6)")
        .bind(id)
        .execute(&pool)
        .await
        .unwrap();

    put(&router, &token, &project, "index.html", "hi").await;
    let (_, body) = call(
        &router,
        "GET",
        &format!("/api/projects/{project}/versions"),
        Value::Null,
        &token,
    )
    .await;
    assert_eq!(body["versions"][0]["restorable"], true);
    assert_eq!(body["versions"][0]["files"], 2);
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_project_is_renamed_duplicated_and_remixed_from_the_showcase(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let (token, org) = person(&router, "owner@example.com").await;
    let (project, _) = new_project(&router, &token).await;
    put(&router, &token, &project, "index.html", "<h1>Kopi</h1>").await;

    let (status, body) = call(
        &router,
        "PATCH",
        &format!("/api/projects/{project}"),
        json!({ "name": "  Coffee shop ", "description": "orders" }),
        &token,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["name"], "Coffee shop");
    assert_eq!(body["description"], "orders");
    let (status, body) = call(
        &router,
        "PATCH",
        &format!("/api/projects/{project}"),
        json!({ "description": "" }),
        &token,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["name"], "Coffee shop");
    assert_eq!(body["description"], Value::Null);
    let (status, _) = call(
        &router,
        "PATCH",
        &format!("/api/projects/{project}"),
        json!({ "name": " " }),
        &token,
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let (status, _) = call(
        &router,
        "PATCH",
        &format!("/api/projects/{project}"),
        json!({ "name": "x".repeat(81) }),
        &token,
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    let (status, copy) = call(
        &router,
        "POST",
        &format!("/api/projects/{project}/duplicate"),
        json!({ "name": "Coffee shop 2" }),
        &token,
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let copy_id = copy["project_id"].as_str().unwrap();
    assert_eq!(
        file(&router, &token, copy_id, "index.html").await.1,
        "<h1>Kopi</h1>"
    );
    let (_, list) = call(
        &router,
        "GET",
        &format!("/api/projects?org_id={org}"),
        Value::Null,
        &token,
    )
    .await;
    let names: Vec<_> = list["projects"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["name"].as_str().unwrap().to_string())
        .collect();
    assert!(
        names.contains(&"Coffee shop 2".to_string()) && names.contains(&"Coffee shop".to_string()),
        "{names:?}"
    );

    // Someone outside can't rename, duplicate, or remix what isn't on show.
    let (other, _) = person(&router, "other@example.com").await;
    let (status, _) = call(
        &router,
        "PATCH",
        &format!("/api/projects/{project}"),
        json!({ "name": "Mine" }),
        &other,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (status, _) = call(
        &router,
        "POST",
        &format!("/api/projects/{project}/duplicate"),
        Value::Null,
        &other,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (status, _) = call(
        &router,
        "POST",
        &format!("/api/showcase/{project}/remix"),
        Value::Null,
        &other,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    sqlx::query("UPDATE projects SET showcase = true, shipyard_app = 'coffee' WHERE id = $1::uuid")
        .bind(&project)
        .execute(&pool)
        .await
        .unwrap();
    let (status, remix) = call(
        &router,
        "POST",
        &format!("/api/showcase/{project}/remix"),
        Value::Null,
        &other,
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let remix_id = remix["project_id"].as_str().unwrap();
    assert_eq!(
        file(&router, &other, remix_id, "index.html").await.1,
        "<h1>Kopi</h1>"
    );
    let (_, list) = call(&router, "GET", "/api/projects", Value::Null, &other).await;
    assert_eq!(list["projects"][0]["name"], "Coffee shop");
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_summary_says_what_waits_on_you_and_how_its_publishes_went(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let (token, _) = person(&router, "owner@example.com").await;
    let (project, session) = new_project(&router, &token).await;
    let summary = |list: &Value| list["projects"][0].clone();

    let (_, list) = call(&router, "GET", "/api/projects", Value::Null, &token).await;
    assert_eq!(summary(&list)["waiting_on_you"], 0);
    assert_eq!(summary(&list)["last_deploy"], Value::Null);
    assert_eq!(summary(&list)["live_status"], Value::Null);

    // Koda asks; nobody has answered yet.
    sqlx::query("INSERT INTO messages (session_id, content, content_blocks) VALUES ($1::uuid, 'Two questions', $2)")
        .bind(&session)
        .bind(json!([{ "kind": "questions", "intro": null, "questions": [] }]))
        .execute(&pool)
        .await
        .unwrap();
    let (_, list) = call(&router, "GET", "/api/projects", Value::Null, &token).await;
    assert_eq!(summary(&list)["waiting_on_you"], 1);
    sqlx::query(
        "INSERT INTO messages (session_id, sender_channel_identity_id, content, created_at) \
         SELECT $1::uuid, ci.id, 'blue', now() + interval '1 second' FROM sessions s JOIN channel_identities ci ON ci.user_id = s.user_id WHERE s.id = $1::uuid LIMIT 1",
    )
    .bind(&session)
    .execute(&pool)
    .await
    .unwrap();
    let (_, list) = call(&router, "GET", "/api/projects", Value::Null, &token).await;
    assert_eq!(summary(&list)["waiting_on_you"], 0);

    // Published to production, then it crashed.
    sqlx::query("UPDATE projects SET shipyard_app = 'coffee' WHERE id = $1::uuid")
        .bind(&project)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO project_publishes (project_id, environment, status, manifest, finished_at, created_at) \
         VALUES ($1::uuid, 'production', 'succeeded', '{}', now() - interval '5 minutes', now() - interval '6 minutes')",
    )
    .bind(&project)
    .execute(&pool)
    .await
    .unwrap();
    let (_, list) = call(&router, "GET", "/api/projects", Value::Null, &token).await;
    assert_eq!(summary(&list)["last_deploy"]["status"], "ok");
    assert_eq!(summary(&list)["live_status"], "up");

    sqlx::query("INSERT INTO shipyard_events (id, type, project_id, environment, created_at) VALUES ('ev1', 'service.crashed', $1::uuid, 'production', now())")
        .bind(&project)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO project_publishes (project_id, environment, status, manifest, finished_at) VALUES ($1::uuid, 'staging', 'failed', '{}', now())")
        .bind(&project)
        .execute(&pool)
        .await
        .unwrap();
    let (_, list) = call(&router, "GET", "/api/projects", Value::Null, &token).await;
    assert_eq!(summary(&list)["live_status"], "down");
    assert_eq!(summary(&list)["last_deploy"]["status"], "failed");
}

#[sqlx::test(migrations = "../../migrations")]
async fn search_finds_the_persons_chats_memory_and_files(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let (token, _) = person(&router, "owner@example.com").await;
    let (project, session) = new_project(&router, &token).await;
    put(&router, &token, &project, "src/espresso.ts", "export {}").await;
    sqlx::query("UPDATE sessions SET title = 'Coffee orders' WHERE id = $1::uuid")
        .bind(&session)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO messages (session_id, content) VALUES ($1::uuid, 'The espresso costs 25k at the shop')")
        .bind(&session)
        .execute(&pool)
        .await
        .unwrap();
    let embedding = format!("[{}]", vec!["0.01"; 1536].join(","));
    sqlx::query(
        "INSERT INTO memory_items (user_id, content, embedding) \
         SELECT s.user_id, 'Prefers a double ESPRESSO in the morning', $2::vector FROM sessions s WHERE s.id = $1::uuid",
    )
    .bind(&session)
    .bind(&embedding)
    .execute(&pool)
    .await
    .unwrap();

    let (status, body) = call(
        &router,
        "GET",
        "/api/search?q=espresso",
        Value::Null,
        &token,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let results = body["results"].as_array().unwrap();
    let kinds: Vec<_> = results
        .iter()
        .map(|r| r["kind"].as_str().unwrap())
        .collect();
    assert_eq!(results.len(), 3, "{body}");
    assert_eq!(kinds[0], "file", "a file-name hit ranks first: {body}");
    assert!(kinds.contains(&"chat") && kinds.contains(&"memory"));
    let chat = results.iter().find(|r| r["kind"] == "chat").unwrap();
    assert_eq!(chat["session_id"], session.as_str());
    assert_eq!(chat["project_id"], project.as_str());
    assert!(chat["snippet"].as_str().unwrap().contains("espresso"));

    let (_, body) = call(
        &router,
        "GET",
        "/api/search?q=COFFEE&kinds=chat",
        Value::Null,
        &token,
    )
    .await;
    assert_eq!(body["results"][0]["title"], "Coffee orders");
    let (_, body) = call(
        &router,
        "GET",
        "/api/search?q=espresso&kinds=memory&limit=1",
        Value::Null,
        &token,
    )
    .await;
    assert_eq!(body["results"].as_array().unwrap().len(), 1);
    assert_eq!(body["results"][0]["kind"], "memory");
    // Wildcards are literal.
    let (_, body) = call(&router, "GET", "/api/search?q=%25", Value::Null, &token).await;
    assert_eq!(body["results"].as_array().unwrap().len(), 0);
    let (_, body) = call(&router, "GET", "/api/search?q=", Value::Null, &token).await;
    assert_eq!(body["results"].as_array().unwrap().len(), 0);
    let (status, _) = call(
        &router,
        "GET",
        &format!("/api/search?q={}", "a".repeat(201)),
        Value::Null,
        &token,
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    // Nobody else's.
    let (other, _) = person(&router, "other@example.com").await;
    let (_, body) = call(
        &router,
        "GET",
        "/api/search?q=espresso",
        Value::Null,
        &other,
    )
    .await;
    assert_eq!(body["results"].as_array().unwrap().len(), 0);
}
