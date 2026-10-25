//! Publishing projects on Shipyard (a mock here): publish, webhook events, rollback, taking a
//! project down, and who may open and change a project.
use std::io::Read;

use axum::{body::Body, http::{Request, StatusCode}};
use http_body_util::BodyExt;
use nomi_server::app::{build_router, AppState};
use serde_json::{json, Value};
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;
use wiremock::matchers::{method, path_regex};
use wiremock::{Mock, MockServer, ResponseTemplate};

const SECRET: &str = "test-secret-do-not-use-in-prod";
const WEBHOOK_SECRET: &str = "whsec_test";

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
            webhook_secret: Some(WEBHOOK_SECRET.into()),
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

async fn webhook(router: axum::Router, body: &Value, secret: &str) -> StatusCode {
    let raw = body.to_string();
    let now = chrono::Utc::now().timestamp();
    let signature = format!("t={now},v1={}", nomi_shipyard::signature::sign(secret, now, raw.as_bytes()));
    let request = Request::builder()
        .method("POST")
        .uri("/api/hooks/shipyard")
        .header("content-type", "application/json")
        .header("shipyard-signature", signature)
        .body(Body::from(raw))
        .unwrap();
    router.oneshot(request).await.unwrap().status()
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

/// A SvelteKit project with a Postgres database, in the person's own organization.
async fn project_with_files(router: axum::Router, token: &str) -> (Uuid, Uuid) {
    let (_, orgs) = request(router.clone(), "GET", "/api/orgs", Some(token), None).await;
    let org_id: Uuid = orgs["orgs"][0]["id"].as_str().unwrap().parse().unwrap();
    request(router.clone(), "PUT", "/api/preferences/project-org", Some(token), Some(json!({ "org_id": org_id }))).await;
    let (status, created) = request(router.clone(), "POST", "/api/projects", Some(token), None).await;
    assert_eq!(status, StatusCode::CREATED);
    let project_id: Uuid = created["project_id"].as_str().unwrap().parse().unwrap();
    for (file, content) in [
        ("package.json", r#"{"name":"todo","scripts":{"build":"vite build"}}"#),
        ("drizzle.config.ts", "export default { dialect: 'postgresql' }"),
        ("src/routes/+page.svelte", "<h1>Todo</h1>"),
        (".env", "SECRET=1"),
    ] {
        let (status, _) =
            request(router.clone(), "PUT", &format!("/api/projects/{project_id}/files/{file}"), Some(token), Some(json!({ "content": content }))).await;
        assert_eq!(status, StatusCode::OK, "{file}");
    }
    (project_id, org_id)
}

async fn mount_shipyard(server: &MockServer) {
    Mock::given(method("PUT"))
        .and(path_regex(r"^/orgs/[^/]+$"))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({ "data": { "created": true }, "error": null })))
        .mount(server)
        .await;
    Mock::given(method("PUT"))
        .and(path_regex(r"^/orgs/[^/]+/apps/[^/]+$"))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({ "data": { "created": true }, "error": null })))
        .mount(server)
        .await;
}

/// The manifest and zip entries of a publish request Shipyard received.
fn publish_parts(body: &[u8]) -> (Value, Vec<String>) {
    let text = String::from_utf8_lossy(body);
    let manifest_start = text.find('{').unwrap();
    let manifest_end = text[manifest_start..].find("\r\n--").unwrap() + manifest_start;
    let manifest: Value = serde_json::from_str(&text[manifest_start..manifest_end]).unwrap();
    // The zip starts at its local file header signature.
    let zip_start = body.windows(4).position(|w| w == b"PK\x03\x04").unwrap();
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(body[zip_start..].to_vec())).unwrap();
    let names = (0..archive.len()).map(|i| archive.by_index(i).unwrap().name().to_string()).collect();
    let mut dockerfile = String::new();
    archive.by_name("Dockerfile").unwrap().read_to_string(&mut dockerfile).unwrap();
    assert!(dockerfile.contains(r#"CMD ["node", "build"]"#));
    (manifest, names)
}

#[sqlx::test(migrations = "../../migrations")]
async fn publishing_follows_webhooks_and_rolls_back(pool: PgPool) {
    let shipyard = MockServer::start().await;
    mount_shipyard(&shipyard).await;
    Mock::given(method("POST"))
        .and(path_regex(r"^/orgs/[^/]+/apps/[^/]+/envs/production/publish$"))
        .respond_with(ResponseTemplate::new(202).set_body_json(json!({
            "data": { "publish_id": "pub-1", "status": "running", "url": "https://todo.apps.test", "warnings": [] }, "error": null
        })))
        .up_to_n_times(1)
        .mount(&shipyard)
        .await;
    Mock::given(method("POST"))
        .and(path_regex(r"^/orgs/[^/]+/apps/[^/]+/envs/production/publish$"))
        .respond_with(ResponseTemplate::new(202).set_body_json(json!({
            "data": { "publish_id": "pub-2", "status": "running", "url": "https://todo.apps.test", "warnings": [] }, "error": null
        })))
        .mount(&shipyard)
        .await;

    let router = build_router(test_state(pool.clone(), Some(&shipyard)));
    let token = register(router.clone(), "ana@acme.test").await;
    let (project_id, org_id) = project_with_files(router.clone(), &token).await;

    let (status, first) = request(router.clone(), "POST", &format!("/api/projects/{project_id}/deploy/publish"), Some(&token), None).await;
    assert_eq!(status, StatusCode::ACCEPTED, "{first}");
    assert_eq!(first["status"], "running");
    assert_eq!(first["can_roll_back"], true);

    // Another publish while one runs is refused.
    let (status, _) = request(router.clone(), "POST", &format!("/api/projects/{project_id}/deploy/publish"), Some(&token), None).await;
    assert_eq!(status, StatusCode::CONFLICT);

    let received = shipyard.received_requests().await.unwrap();
    let org_put = received.iter().find(|r| r.method.as_str() == "PUT" && !r.url.path().contains("/apps/")).unwrap();
    assert_eq!(org_put.url.path(), format!("/orgs/{org_id}"));
    let publish_request = received.iter().find(|r| r.url.path().ends_with("/publish")).unwrap();
    assert_eq!(publish_request.headers.get("authorization").unwrap(), "Bearer ship_int_manage");
    let (manifest, names) = publish_parts(&publish_request.body);
    assert_eq!(manifest["database"]["engine"], "postgres");
    assert_eq!(manifest["services"][0]["env"]["DATABASE_URL"], json!({ "database": "url" }));
    assert!(names.contains(&"package.json".to_string()));
    assert!(names.contains(&"Dockerfile".to_string()));
    assert!(!names.contains(&".env".to_string()), "never send .env files");

    let app: String = sqlx::query_scalar("SELECT shipyard_app FROM projects WHERE id = $1").bind(project_id).fetch_one(&pool).await.unwrap();
    assert!(app.starts_with("new-project-"));

    // A forged event is refused; a signed one finishes the publish (once, however often it comes).
    let event = json!({
        "id": "evt-1", "type": "publish.succeeded", "created_at": chrono::Utc::now(),
        "org": { "id": "x", "external_id": org_id }, "app": app, "environment": "production",
        "data": { "publish_id": "pub-1", "urls": ["https://todo.apps.test"], "error": null }
    });
    assert_eq!(webhook(router.clone(), &event, "whsec_wrong").await, StatusCode::UNAUTHORIZED);
    assert_eq!(webhook(router.clone(), &event, WEBHOOK_SECRET).await, StatusCode::NO_CONTENT);
    assert_eq!(webhook(router.clone(), &event, WEBHOOK_SECRET).await, StatusCode::NO_CONTENT);
    let mut overview = Value::Null;
    for _ in 0..50 {
        let (_, body) = request(router.clone(), "GET", &format!("/api/projects/{project_id}/deploy"), Some(&token), None).await;
        overview = body;
        if overview["publishes"][0]["status"] == "succeeded" {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
    assert_eq!(overview["publishes"][0]["status"], "succeeded");
    assert_eq!(overview["url"], "https://todo.apps.test");
    assert_eq!(overview["database"], "postgres");
    assert_eq!(overview["role"], "admin");
    let stored: i64 = sqlx::query_scalar("SELECT count(*) FROM shipyard_events WHERE project_id = $1").bind(project_id).fetch_one(&pool).await.unwrap();
    assert_eq!(stored, 1);

    // A failed crash alert notifies the project's admin.
    let crash = json!({
        "id": "evt-2", "type": "service.crashed", "created_at": chrono::Utc::now(), "app": app,
        "environment": "production", "service": "web", "data": { "exit_code": 137 }
    });
    assert_eq!(webhook(router.clone(), &crash, WEBHOOK_SECRET).await, StatusCode::NO_CONTENT);
    let owner = user_id(&pool, "ana@acme.test").await;
    let mut notified = 0i64;
    for _ in 0..50 {
        notified = sqlx::query_scalar("SELECT count(*) FROM notifications WHERE user_id = $1 AND kind = 'deploy'")
            .bind(owner)
            .fetch_one(&pool)
            .await
            .unwrap();
        if notified > 0 {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
    assert_eq!(notified, 1);

    // Rolling back sends the first publish's source and manifest again.
    let first_id = first["id"].as_str().unwrap();
    let (status, rolled) =
        request(router.clone(), "POST", &format!("/api/projects/{project_id}/deploy/publishes/{first_id}/rollback"), Some(&token), None).await;
    assert_eq!(status, StatusCode::ACCEPTED, "{rolled}");
    assert_eq!(rolled["rollback_of"], first_id);
    let received = shipyard.received_requests().await.unwrap();
    let publishes: Vec<_> = received.iter().filter(|r| r.url.path().ends_with("/publish")).collect();
    assert_eq!(publishes.len(), 2);
    assert_eq!(publish_parts(&publishes[1].body).0, manifest);
}

#[sqlx::test(migrations = "../../migrations")]
async fn secrets_go_to_shipyard_and_are_bound_by_name_on_publish(pool: PgPool) {
    let shipyard = MockServer::start().await;
    mount_shipyard(&shipyard).await;
    Mock::given(method("PUT"))
        .and(path_regex(r"^/orgs/[^/]+/apps/[^/]+/secrets/[A-Z_]+$"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "data": { "ref": "sec-1" }, "error": null })))
        .mount(&shipyard)
        .await;
    Mock::given(method("DELETE"))
        .and(path_regex(r"^/orgs/[^/]+/apps/[^/]+/secrets/[A-Z_]+$"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "data": { "deleted": true }, "error": null })))
        .mount(&shipyard)
        .await;
    Mock::given(method("POST"))
        .and(path_regex(r"^/orgs/[^/]+/apps/[^/]+/envs/production/publish$"))
        .respond_with(ResponseTemplate::new(202).set_body_json(json!({
            "data": { "publish_id": "pub-1", "status": "running", "url": "https://todo.apps.test", "warnings": [] }, "error": null
        })))
        .mount(&shipyard)
        .await;
    let router = build_router(test_state(pool.clone(), Some(&shipyard)));
    let token = register(router.clone(), "ana@acme.test").await;
    let (project_id, _) = project_with_files(router.clone(), &token).await;
    let secrets = format!("/api/projects/{project_id}/secrets");

    let (status, body) = request(router.clone(), "PUT", &format!("{secrets}/DATABASE_URL"), Some(&token), Some(json!({ "value": "x" }))).await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");
    let (status, _) = request(router.clone(), "PUT", &format!("{secrets}/stripe"), Some(&token), Some(json!({ "value": "x" }))).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    for name in ["STRIPE_KEY", "OPENAI_API_KEY"] {
        let (status, body) = request(router.clone(), "PUT", &format!("{secrets}/{name}"), Some(&token), Some(json!({ "value": "sk_live_123" }))).await;
        assert_eq!(status, StatusCode::NO_CONTENT, "{body}");
    }
    // The value went to Shipyard, for every environment of the app.
    let received = shipyard.received_requests().await.unwrap();
    let put = received.iter().find(|r| r.url.path().ends_with("/secrets/STRIPE_KEY")).unwrap();
    let sent: Value = serde_json::from_slice(&put.body).unwrap();
    assert_eq!(sent["value"], "sk_live_123");
    assert_eq!(sent["environment"], Value::Null);

    // Only the names come back.
    let (status, listed) = request(router.clone(), "GET", &secrets, Some(&token), None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(listed["can_edit"], true);
    assert_eq!(listed["secrets"].as_array().unwrap().len(), 2);
    assert_eq!(listed["secrets"][1]["name"], "STRIPE_KEY");
    assert!(!listed.to_string().contains("sk_live_123"));

    let (status, _) = request(router.clone(), "DELETE", &format!("{secrets}/OPENAI_API_KEY"), Some(&token), None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (status, _) = request(router.clone(), "POST", &format!("/api/projects/{project_id}/deploy/publish"), Some(&token), None).await;
    assert_eq!(status, StatusCode::ACCEPTED);
    let received = shipyard.received_requests().await.unwrap();
    let publish_request = received.iter().find(|r| r.url.path().ends_with("/publish")).unwrap();
    let (manifest, _) = publish_parts(&publish_request.body);
    let env = &manifest["services"][0]["env"];
    assert_eq!(env["STRIPE_KEY"], json!({ "secret": "STRIPE_KEY" }));
    assert!(env.get("OPENAI_API_KEY").is_none());
    assert!(!String::from_utf8_lossy(&publish_request.body).contains("sk_live_123"));
}

#[sqlx::test(migrations = "../../migrations")]
async fn publishing_without_shipyard_is_unavailable(pool: PgPool) {
    let router = build_router(test_state(pool.clone(), None));
    let token = register(router.clone(), "ana@acme.test").await;
    let (project_id, _) = project_with_files(router.clone(), &token).await;
    let (status, overview) = request(router.clone(), "GET", &format!("/api/projects/{project_id}/deploy"), Some(&token), None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(overview["available"], false);
    let (status, _) = request(router, "POST", &format!("/api/projects/{project_id}/deploy/publish"), Some(&token), None).await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_project_can_be_limited_to_its_members(pool: PgPool) {
    let router = build_router(test_state(pool.clone(), None));
    let owner_token = register(router.clone(), "ana@acme.test").await;
    let teammate_token = register(router.clone(), "budi@acme.test").await;
    let (project_id, org_id) = project_with_files(router.clone(), &owner_token).await;
    let teammate = user_id(&pool, "budi@acme.test").await;
    sqlx::query("INSERT INTO memberships (org_id, user_id, role, status) VALUES ($1, $2, 'member', 'active')")
        .bind(org_id)
        .bind(teammate)
        .execute(&pool)
        .await
        .unwrap();

    // Open to the organization: the teammate sees it, but as a member can't change who's in it.
    let (status, members) = request(router.clone(), "GET", &format!("/api/projects/{project_id}/members"), Some(&teammate_token), None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(members["access"], "org");
    assert_eq!(members["my_role"], "member");
    assert_eq!(members["people"].as_array().unwrap().len(), 2);
    let (status, _) =
        request(router.clone(), "PUT", &format!("/api/projects/{project_id}/access"), Some(&teammate_token), Some(json!({ "access": "members" }))).await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    // Limited to its members: the teammate loses it until they're added.
    let (status, members) =
        request(router.clone(), "PUT", &format!("/api/projects/{project_id}/access"), Some(&owner_token), Some(json!({ "access": "members" }))).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(members["access"], "members");
    let (status, _) = request(router.clone(), "GET", &format!("/api/projects/{project_id}"), Some(&teammate_token), None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    let (status, _) = request(
        router.clone(),
        "PUT",
        &format!("/api/projects/{project_id}/members/{teammate}"),
        Some(&owner_token),
        Some(json!({ "role": "member" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let (status, _) = request(router.clone(), "GET", &format!("/api/projects/{project_id}"), Some(&teammate_token), None).await;
    assert_eq!(status, StatusCode::OK);
    let (status, _) = request(router.clone(), "DELETE", &format!("/api/projects/{project_id}"), Some(&teammate_token), None).await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    // Someone outside the organization can't be added.
    let outsider: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(&pool).await.unwrap();
    let (status, _) = request(
        router.clone(),
        "PUT",
        &format!("/api/projects/{project_id}/members/{outsider}"),
        Some(&owner_token),
        Some(json!({ "role": "member" })),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);

    // The owner deletes the project (never published, so Shipyard isn't needed).
    let (status, _) = request(router.clone(), "DELETE", &format!("/api/projects/{project_id}"), Some(&owner_token), None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let left: i64 = sqlx::query_scalar("SELECT count(*) FROM projects WHERE id = $1").bind(project_id).fetch_one(&pool).await.unwrap();
    assert_eq!(left, 0);
}
