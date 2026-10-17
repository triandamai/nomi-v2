//! Admin → Integrations against a mock Shipyard: webhook endpoints, organizations, secrets and
//! catching up on events, each behind the `integrations` access level it needs.
use axum::{body::Body, http::{Request, StatusCode}};
use http_body_util::BodyExt;
use nomi_server::app::{build_router, AppState};
use serde_json::{json, Value};
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;
use wiremock::matchers::{body_json, method, path, path_regex};
use wiremock::{Mock, MockServer, ResponseTemplate};

const SECRET: &str = "test-secret-do-not-use-in-prod";

fn test_state(pool: PgPool, server: &MockServer) -> AppState {
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
        shipyard: Some(nomi_shipyard::ShipyardConfig {
            base_url: server.uri(),
            key: "ship_int_manage".into(),
            reveal_key: None,
            webhook_secret: None,
        }),
    }
}

async fn request(router: axum::Router, method: &str, uri: &str, token: &str, body: Option<Value>) -> (StatusCode, Value) {
    let mut builder = Request::builder().method(method).uri(uri).header("authorization", format!("Bearer {token}"));
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

async fn login(router: axum::Router, pool: &PgPool, email: &str, admin: bool) -> String {
    let register = Request::builder()
        .method("POST")
        .uri("/api/auth/register")
        .header("content-type", "application/json")
        .body(Body::from(json!({ "email": email, "password": "correct-password", "org": { "mode": "create", "name": "Acme" } }).to_string()))
        .unwrap();
    router.clone().oneshot(register).await.unwrap();
    if admin {
        sqlx::query("UPDATE users SET is_platform_admin = true WHERE id = (SELECT user_id FROM web_credentials WHERE email = $1)")
            .bind(email)
            .execute(pool)
            .await
            .unwrap();
    }
    let login = Request::builder()
        .method("POST")
        .uri("/api/auth/login")
        .header("content-type", "application/json")
        .body(Body::from(json!({ "email": email, "password": "correct-password" }).to_string()))
        .unwrap();
    let response = router.oneshot(login).await.unwrap();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body: Value = serde_json::from_slice(&bytes).unwrap();
    body["access_token"].as_str().unwrap().to_string()
}

fn ok(data: Value) -> ResponseTemplate {
    ResponseTemplate::new(200).set_body_json(json!({ "data": data, "error": null }))
}

#[sqlx::test(migrations = "../../migrations")]
async fn webhooks_can_be_switched_off_and_given_a_new_secret(pool: PgPool) {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/webhooks"))
        .respond_with(ok(json!([{ "id": "ep1", "url": "https://ops.test/events", "events": ["*"], "enabled": true, "org": null, "created_at": "2026-01-01T00:00:00Z" }])))
        .mount(&server)
        .await;
    Mock::given(method("PATCH"))
        .and(path("/webhooks/ep1"))
        .and(body_json(json!({ "enabled": false })))
        .respond_with(ok(json!({ "id": "ep1", "enabled": false })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/webhooks/ep1/rotate-secret"))
        .respond_with(ok(json!({ "id": "ep1", "secret": "whsec_new" })))
        .mount(&server)
        .await;
    let router = build_router(test_state(pool.clone(), &server));
    let token = login(router.clone(), &pool, "admin@example.com", true).await;

    let (status, body) = request(router.clone(), "GET", "/api/admin/shipyard/webhooks", &token, None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["endpoints"][0]["id"], "ep1");

    let (status, _) = request(router.clone(), "PATCH", "/api/admin/shipyard/webhooks/ep1", &token, Some(json!({ "enabled": false }))).await;
    assert_eq!(status, StatusCode::OK);

    // Someone else's endpoint: the secret is shown once.
    let (status, body) = request(router.clone(), "POST", "/api/admin/shipyard/webhooks/ep1/rotate", &token, None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["secret"], "whsec_new");

    // Nomi's own (it registered it): it keeps the secret and doesn't show it.
    sqlx::query("INSERT INTO shipyard_webhook (endpoint_id, url, secret_encrypted) VALUES ('ep1', 'https://ops.test/events', $1)")
        .bind(nomi_settings::crypto::encrypt(&nomi_test_support::TEST_SETTINGS_KEY, "whsec_old"))
        .execute(&pool)
        .await
        .unwrap();
    let (_, body) = request(router, "POST", "/api/admin/shipyard/webhooks/ep1/rotate", &token, None).await;
    assert_eq!(body["secret"], Value::Null);
    let stored: Vec<u8> = sqlx::query_scalar("SELECT secret_encrypted FROM shipyard_webhook").fetch_one(&pool).await.unwrap();
    assert_eq!(nomi_settings::crypto::decrypt(&nomi_test_support::TEST_SETTINGS_KEY, &stored).unwrap(), "whsec_new");
}

#[sqlx::test(migrations = "../../migrations")]
async fn secrets_need_manage_and_never_come_back(pool: PgPool) {
    let server = MockServer::start().await;
    Mock::given(method("PUT"))
        .and(path_regex(r"^/orgs/[^/]+/apps/shop/secrets/STRIPE_KEY$"))
        .and(body_json(json!({ "value": "sk_test_123", "environment": "production" })))
        .respond_with(ok(json!({ "ref": "sec_1", "name": "STRIPE_KEY" })))
        .expect(1)
        .mount(&server)
        .await;
    let router = build_router(test_state(pool.clone(), &server));
    let admin = login(router.clone(), &pool, "admin@example.com", true).await;
    let org = Uuid::new_v4();
    let uri = format!("/api/admin/shipyard/orgs/{org}/apps/shop/secrets/STRIPE_KEY");

    let (status, body) = request(router.clone(), "PUT", &uri, &admin, Some(json!({ "value": "sk_test_123", "environment": "production" }))).await;
    assert_eq!(status, StatusCode::OK);
    assert!(!body.to_string().contains("sk_test_123"));

    let (status, _) = request(router.clone(), "PUT", &format!("/api/admin/shipyard/orgs/{org}/apps/shop/secrets/bad-name"), &admin, Some(json!({ "value": "x" }))).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    // Write on integrations isn't enough for secrets.
    login(router.clone(), &pool, "staff@example.com", false).await;
    let staff_id: Uuid = sqlx::query_scalar("SELECT user_id FROM web_credentials WHERE email = 'staff@example.com'").fetch_one(&pool).await.unwrap();
    let admin_id: Uuid = sqlx::query_scalar("SELECT user_id FROM web_credentials WHERE email = 'admin@example.com'").fetch_one(&pool).await.unwrap();
    nomi_auth::grants::set_admin_access(&pool, staff_id, &[("integrations".into(), "write".into())], admin_id).await.unwrap();
    // Signing in again picks up the new access.
    let staff = login(router.clone(), &pool, "staff@example.com", false).await;
    let (status, _) = request(router, "PUT", &uri, &staff, Some(json!({ "value": "sk_test_123" }))).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}

#[sqlx::test(migrations = "../../migrations")]
async fn catching_up_records_missed_events_once(pool: PgPool) {
    let server = MockServer::start().await;
    let router = build_router(test_state(pool.clone(), &server));
    let token = login(router.clone(), &pool, "admin@example.com", true).await;
    let org: Uuid = sqlx::query_scalar("UPDATE organizations SET shipyard_synced_at = now() WHERE name = 'Acme' RETURNING id").fetch_one(&pool).await.unwrap();
    Mock::given(method("GET"))
        .and(path(format!("/orgs/{org}/events")))
        .respond_with(ok(json!({
            "events": [
                { "id": "evt_1", "type": "deployment.succeeded", "created_at": "2026-01-01T00:00:00Z", "data": {} },
                { "id": "evt_2", "type": "service.crashed", "created_at": "2026-01-01T00:01:00Z", "data": {} }
            ],
            "next_after": "evt_2"
        })))
        .mount(&server)
        .await;

    let (status, body) = request(router.clone(), "POST", "/api/admin/shipyard/events/catch-up", &token, None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["new_events"], 2);
    let cursor: String = sqlx::query_scalar("SELECT after_id FROM shipyard_event_cursors WHERE org_id = $1").bind(org).fetch_one(&pool).await.unwrap();
    assert_eq!(cursor, "evt_2");

    let (_, body) = request(router.clone(), "POST", "/api/admin/shipyard/events/catch-up", &token, None).await;
    assert_eq!(body["new_events"], 0);

    let (status, body) = request(router, "GET", "/api/admin/shipyard/events", &token, None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["events"].as_array().unwrap().len(), 2);
}

#[sqlx::test(migrations = "../../migrations")]
async fn removing_an_org_from_shipyard_unpublishes_its_projects(pool: PgPool) {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path_regex(r"^/orgs/[^/]+$"))
        .respond_with(ok(json!({ "deleted": true })))
        .expect(1)
        .mount(&server)
        .await;
    let router = build_router(test_state(pool.clone(), &server));
    let token = login(router.clone(), &pool, "admin@example.com", true).await;
    let org: Uuid = sqlx::query_scalar("UPDATE organizations SET shipyard_synced_at = now() WHERE name = 'Acme' RETURNING id").fetch_one(&pool).await.unwrap();

    let (status, body) = request(router.clone(), "GET", "/api/admin/shipyard/orgs", &token, None).await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.as_array().unwrap().iter().any(|o| o["id"] == org.to_string()));

    let (status, _) = request(router, "DELETE", &format!("/api/admin/shipyard/orgs/{org}"), &token, None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let synced: Option<chrono::DateTime<chrono::Utc>> =
        sqlx::query_scalar("SELECT shipyard_synced_at FROM organizations WHERE id = $1").bind(org).fetch_one(&pool).await.unwrap();
    assert!(synced.is_none());
}

async fn signed_webhook(router: axum::Router, secret: &str) -> StatusCode {
    let raw = json!({ "id": "ping-1", "type": "ping" }).to_string();
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

/// SHIPYARD_WEBHOOK_SECRET set but no endpoint on Shipyard: the page says so, registering puts
/// one there, and events signed with either secret are accepted.
#[sqlx::test(migrations = "../../migrations")]
async fn a_missing_webhook_shows_up_and_can_be_registered_despite_an_env_secret(pool: PgPool) {
    std::env::set_var("APP_URL", "https://nomi.test");
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/webhooks"))
        .respond_with(ok(json!([])))
        .up_to_n_times(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/webhooks"))
        .respond_with(ok(json!([{ "id": "ep9", "url": "https://nomi.test/hooks/shipyard", "events": ["*"], "enabled": true, "org": null, "created_at": "2026-01-01T00:00:00Z" }])))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/webhooks"))
        .and(body_json(json!({ "url": "https://nomi.test/hooks/shipyard", "events": ["*"] })))
        .respond_with(ok(json!({ "id": "ep9", "url": "https://nomi.test/hooks/shipyard", "events": ["*"], "org": null, "secret": "whsec_new" })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/webhooks/ep9/test"))
        .respond_with(ok(json!({ "ok": true })))
        .mount(&server)
        .await;
    let mut state = test_state(pool.clone(), &server);
    state.shipyard.as_mut().unwrap().webhook_secret = Some("whsec_env".into());
    let router = build_router(state);
    let token = login(router.clone(), &pool, "admin@example.com", true).await;

    let (status, body) = request(router.clone(), "GET", "/api/admin/shipyard", &token, None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["webhook_registered"], false);
    assert_eq!(body["webhook_from_env"], true);
    assert_eq!(body["webhook_target"], "https://nomi.test/hooks/shipyard");

    let (status, body) = request(router.clone(), "POST", "/api/admin/shipyard/webhook", &token, None).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["webhook_registered"], true);
    assert_eq!(body["webhook_url"], "https://nomi.test/hooks/shipyard");

    let (_, body) = request(router.clone(), "GET", "/api/admin/shipyard/webhooks", &token, None).await;
    assert_eq!(body["own_id"], "ep9");

    assert_eq!(signed_webhook(router.clone(), "whsec_new").await, StatusCode::NO_CONTENT);
    assert_eq!(signed_webhook(router.clone(), "whsec_env").await, StatusCode::NO_CONTENT);
    assert_eq!(signed_webhook(router, "whsec_wrong").await, StatusCode::UNAUTHORIZED);
}
