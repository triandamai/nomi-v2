//! What happens in a project, newest first: prompts, builds and the files they changed, publishes, rollbacks, people joining.
use axum::{body::Body, http::{Request, StatusCode}};
use http_body_util::BodyExt;
use nomi_server::app::{build_router, AppState};
use serde_json::{json, Value};
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;
use wiremock::MockServer;

const SECRET: &str = "test-secret-do-not-use-in-prod";

fn test_state(pool: PgPool, shipyard: Option<&MockServer>) -> AppState {
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
        shipyard: shipyard.map(|server| nomi_shipyard::ShipyardConfig {
            base_url: server.uri(),
            key: "ship_int_manage".into(),
            reveal_key: Some("ship_int_reveal".into()),
            webhook_secret: None,
        }),
    }
}

async fn request(router: axum::Router, method: &str, uri: &str, token: Option<&str>, body: Option<Value>) -> (StatusCode, Value) {
    let mut builder = Request::builder().method(method).uri(uri);
    if let Some(token) = token {
        builder = builder.header("authorization", format!("Bearer {token}"));
    }
    let body = match body {
        Some(b) => {
            builder = builder.header("content-type", "application/json");
            Body::from(b.to_string())
        }
        None => Body::empty(),
    };
    let response = router.oneshot(builder.body(body).unwrap()).await.unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    (status, if bytes.is_empty() { Value::Null } else { serde_json::from_slice(&bytes).unwrap_or(Value::Null) })
}

async fn register(router: axum::Router, email: &str) -> String {
    request(
        router.clone(),
        "POST",
        "/api/auth/register",
        None,
        Some(json!({ "email": email, "password": "correct-password", "org": { "mode": "create", "name": "Acme" } })),
    )
    .await;
    let (_, body) = request(router, "POST", "/api/auth/login", None, Some(json!({ "email": email, "password": "correct-password" }))).await;
    body["access_token"].as_str().unwrap().to_string()
}

async fn user_id(pool: &PgPool, email: &str) -> Uuid {
    sqlx::query_scalar("SELECT user_id FROM web_credentials WHERE email = $1").bind(email).fetch_one(pool).await.unwrap()
}


/// Ana's project in her organization, with Ben as an active member of that organization (so he
/// can open it), and Cy in an organization of his own (so he can't).
struct Team {
    router: axum::Router,
    project_id: Uuid,
    ana: String,
    ben: String,
    cy: String,
}

async fn team(pool: &PgPool) -> Team {
    let router = build_router(test_state(pool.clone(), None));
    let ana = register(router.clone(), "ana@acme.test").await;
    let ben = register(router.clone(), "ben@acme.test").await;
    let cy = register(router.clone(), "cy@other.test").await;
    let (_, orgs) = request(router.clone(), "GET", "/api/orgs", Some(&ana), None).await;
    let org_id: Uuid = orgs["orgs"][0]["id"].as_str().unwrap().parse().unwrap();
    request(router.clone(), "PUT", "/api/preferences/project-org", Some(&ana), Some(json!({ "org_id": org_id }))).await;
    let (_, created) = request(router.clone(), "POST", "/api/projects", Some(&ana), None).await;
    let project_id: Uuid = created["project_id"].as_str().unwrap().parse().unwrap();
    let ben_id = user_id(pool, "ben@acme.test").await;
    sqlx::query("INSERT INTO memberships (org_id, user_id, role) VALUES ($1, $2, 'member')").bind(org_id).bind(ben_id).execute(pool).await.unwrap();
    Team { router, project_id, ana, ben, cy }
}

fn names(body: &Value) -> Vec<String> {
    body["people"].as_array().unwrap().iter().map(|p| p["name"].as_str().unwrap().to_string()).collect()
}


async fn session_of(pool: &PgPool, project_id: Uuid) -> Uuid {
    sqlx::query_scalar("SELECT session_id FROM projects WHERE id = $1").bind(project_id).fetch_one(pool).await.unwrap()
}

/// Ana's web identity, so a message counts as hers.
async fn identity_of(pool: &PgPool, email: &str) -> Uuid {
    let user = user_id(pool, email).await;
    sqlx::query_scalar("SELECT id FROM channel_identities WHERE user_id = $1 ORDER BY created_at LIMIT 1")
        .bind(user)
        .fetch_optional(pool)
        .await
        .unwrap()
        .unwrap_or_else(|| panic!("{email} has no channel identity"))
}

async fn said(pool: &PgPool, session: Uuid, from: Option<Uuid>, content: &str, blocks: Option<Value>, minutes_ago: i32) {
    sqlx::query(
        "INSERT INTO messages (session_id, sender_channel_identity_id, content, content_blocks, created_at) \
         VALUES ($1, $2, $3, $4, now() - make_interval(mins => $5))",
    )
    .bind(session)
    .bind(from)
    .bind(content)
    .bind(blocks)
    .bind(minutes_ago)
    .execute(pool)
    .await
    .unwrap();
}

/// A project with a little history: Ana asked for a checkout page, Koda built it (changing three
/// files), Ana published version 2 which failed at build, then rolled back to version 1, and Ben
/// joined.
async fn history(pool: &PgPool, t: &Team) {
    let session = session_of(pool, t.project_id).await;
    let ana = user_id(pool, "ana@acme.test").await;
    // Sending through the API gives Ana her web identity, as in real use.
    request(t.router.clone(), "POST", &format!("/api/sessions/{session}/messages"), Some(&t.ana), Some(json!({ "text": "Add a checkout page" }))).await;
    sqlx::query("UPDATE messages SET created_at = now() - interval '60 minutes' WHERE session_id = $1").bind(session).execute(pool).await.unwrap();
    let _ = identity_of(pool, "ana@acme.test").await;
    sqlx::query(
        "INSERT INTO agent_delegations (session_id, user_id, requesting_agent_type, target_agent_type, task, status, claimed_at, completed_at, created_at) \
         VALUES ($1, $2, 'planning', 'coding', 'Project x: build checkout', 'completed', now() - interval '55 minutes', now() - interval '40 minutes', now() - interval '56 minutes')",
    )
    .bind(session)
    .bind(ana)
    .execute(pool)
    .await
    .unwrap();
    said(pool, session, None, "wrote", Some(json!([{ "kind": "file_writes", "files": [{ "path": "a" }, { "path": "b" }] }])), 50).await;
    said(pool, session, None, "removed", Some(json!([{ "kind": "file_delete", "project_id": t.project_id, "path": "c" }])), 45).await;
    said(pool, session, None, "Done: checkout page is ready.", None, 39).await;
    let first: Uuid = sqlx::query_scalar(
        "INSERT INTO project_publishes (project_id, environment, status, manifest, files_version, created_by, created_at, finished_at) \
         VALUES ($1, 'production', 'succeeded', '{}', 1, $2, now() - interval '3 days', now() - interval '3 days') RETURNING id",
    )
    .bind(t.project_id)
    .bind(ana)
    .fetch_one(pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO project_publishes (project_id, environment, status, manifest, files_version, created_by, failure, created_at, finished_at) \
         VALUES ($1, 'production', 'failed', '{}', 2, $2, '{\"phase\":\"build\",\"step\":\"build_image\",\"message\":\"boom\"}', now() - interval '30 minutes', now() - interval '29 minutes')",
    )
    .bind(t.project_id)
    .bind(ana)
    .execute(pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO project_publishes (project_id, environment, status, manifest, files_version, created_by, rollback_of, created_at, finished_at) \
         VALUES ($1, 'production', 'succeeded', '{}', 1, $2, $3, now() - interval '20 minutes', now() - interval '19 minutes')",
    )
    .bind(t.project_id)
    .bind(ana)
    .bind(first)
    .execute(pool)
    .await
    .unwrap();
    let ben = user_id(pool, "ben@acme.test").await;
    sqlx::query("INSERT INTO project_members (project_id, user_id, role, created_at) VALUES ($1, $2, 'member', now() - interval '10 minutes')")
        .bind(t.project_id)
        .bind(ben)
        .execute(pool)
        .await
        .unwrap();
}

fn kinds(body: &Value) -> Vec<String> {
    body["items"].as_array().unwrap().iter().map(|i| i["kind"].as_str().unwrap().to_string()).collect()
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_feed_tells_what_happened_newest_first(pool: PgPool) {
    let t = team(&pool).await;
    history(&pool, &t).await;

    let (status, body) = request(t.router.clone(), "GET", &format!("/api/projects/{}/activity", t.project_id), Some(&t.ben), None).await;

    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(
        kinds(&body),
        vec!["joined", "rolled_back", "publish_failed", "build_finished", "files_changed", "build_started", "prompt", "published"]
    );
    let item = |kind: &str| body["items"].as_array().unwrap().iter().find(|i| i["kind"] == kind).unwrap().clone();
    assert_eq!(item("prompt")["text"], "Add a checkout page");
    assert_eq!(item("prompt")["actor"]["name"], "ana");
    assert_eq!(item("files_changed")["files"], 3);
    assert_eq!(item("build_started")["actor"]["agent"], "Koda");
    assert_eq!(item("publish_failed")["version"], 2);
    assert_eq!(item("publish_failed")["phase"], "build");
    assert_eq!(item("publish_failed")["step"], "build_image");
    assert_eq!(item("rolled_back")["rollback_of_version"], 1);
    assert_eq!(item("joined")["actor"]["name"], "ben");
    assert_eq!(body["has_more"], false);
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_feed_pages_back_in_time(pool: PgPool) {
    let t = team(&pool).await;
    history(&pool, &t).await;
    let base = format!("/api/projects/{}/activity", t.project_id);

    let (_, first) = request(t.router.clone(), "GET", &format!("{base}?limit=3"), Some(&t.ana), None).await;
    assert_eq!(kinds(&first), vec!["joined", "rolled_back", "publish_failed"]);
    assert_eq!(first["has_more"], true);
    let before = first["items"][2]["at"].as_str().unwrap().to_string();
    let encoded: String = before.replace(':', "%3A").replace('+', "%2B");
    let (_, next) = request(t.router.clone(), "GET", &format!("{base}?limit=3&before={encoded}"), Some(&t.ana), None).await;
    assert_eq!(kinds(&next), vec!["build_finished", "files_changed", "build_started"]);
}

#[sqlx::test(migrations = "../../migrations")]
async fn only_people_who_can_open_the_project_see_its_feed(pool: PgPool) {
    let t = team(&pool).await;
    let (status, _) = request(t.router.clone(), "GET", &format!("/api/projects/{}/activity", t.project_id), Some(&t.cy), None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}
