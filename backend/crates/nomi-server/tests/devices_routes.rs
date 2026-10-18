//! Devices: native apps register on sign-in and token refresh, and are told about inbox items
//! and chat replies by push when no socket can stay open.
use std::sync::{Arc, Mutex};

use axum::{body::Body, http::{Request, StatusCode}};
use http_body_util::BodyExt;
use nomi_server::app::{build_router, AppState};
use nomi_server::push::{dispatch, DeviceTarget, PushError, PushKind, PushMessage, PushSender};
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
    let (status, _) = request(
        router.clone(),
        "POST",
        "/api/auth/register",
        None,
        Some(json!({ "email": email, "password": "correct-password", "org": { "mode": "create", "name": "Acme" } })),
    )
    .await;
    assert!(status.is_success(), "{status}");
    let (_, body) = request(router, "POST", "/api/auth/login", None, Some(json!({ "email": email, "password": "correct-password" }))).await;
    body["access_token"].as_str().unwrap().to_string()
}

async fn user_id(pool: &PgPool, email: &str) -> Uuid {
    sqlx::query_scalar("SELECT user_id FROM web_credentials WHERE email = $1").bind(email).fetch_one(pool).await.unwrap()
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_phone_registers_once_per_token_and_only_its_owner_can_remove_it(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let ana = register(router.clone(), "ana@acme.test").await;

    let phone = json!({ "platform": "android", "push_token": "fcm-ana-1", "app_version": "1.0.0" });
    let (status, body) = request(router.clone(), "POST", "/api/devices", Some(&ana), Some(phone.clone())).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let id = body["id"].as_str().expect("an id").to_string();

    // Signing in again (or a token refresh with the same token) keeps the same row.
    let again = json!({ "platform": "android", "push_token": "fcm-ana-1", "app_version": "1.0.1" });
    let (status, body) = request(router.clone(), "POST", "/api/devices", Some(&ana), Some(again)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["id"].as_str(), Some(id.as_str()));
    let version: String = sqlx::query_scalar("SELECT app_version FROM devices WHERE id = $1::uuid").bind(&id).fetch_one(&pool).await.unwrap();
    assert_eq!(version, "1.0.1");

    // A desktop registers without a token, and keeps one row across sign-ins too.
    let mac = json!({ "platform": "macos", "app_version": "1.0.0" });
    let (_, first) = request(router.clone(), "POST", "/api/devices", Some(&ana), Some(mac.clone())).await;
    let (_, second) = request(router.clone(), "POST", "/api/devices", Some(&ana), Some(mac)).await;
    assert!(first["id"].is_string());
    assert_eq!(first["id"], second["id"]);

    let (status, _) =
        request(router.clone(), "POST", "/api/devices", Some(&ana), Some(json!({ "platform": "toaster", "app_version": "1" }))).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let (status, _) = request(router.clone(), "POST", "/api/devices", None, Some(phone)).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    // Someone else can't remove Ana's phone.
    let budi = register(router.clone(), "budi@acme.test").await;
    let (status, _) = request(router.clone(), "DELETE", &format!("/api/devices/{id}"), Some(&budi), None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    let (status, _) = request(router.clone(), "DELETE", &format!("/api/devices/{id}"), Some(&ana), None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let left: i64 = sqlx::query_scalar("SELECT count(*) FROM devices WHERE id = $1::uuid").bind(&id).fetch_one(&pool).await.unwrap();
    assert_eq!(left, 0);
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_phone_handed_to_someone_else_stops_getting_the_first_persons_pushes(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let ana = register(router.clone(), "ana@acme.test").await;
    let budi = register(router.clone(), "budi@acme.test").await;
    let phone = json!({ "platform": "ios", "push_token": "apns-shared", "app_version": "1.0.0" });
    request(router.clone(), "POST", "/api/devices", Some(&ana), Some(phone.clone())).await;
    request(router.clone(), "POST", "/api/devices", Some(&budi), Some(phone)).await;

    let owners: Vec<Uuid> = sqlx::query_scalar("SELECT user_id FROM devices WHERE push_token = 'apns-shared'").fetch_all(&pool).await.unwrap();
    assert_eq!(owners, vec![user_id(&pool, "budi@acme.test").await]);
}

/// Records what it was asked to send, and reports one token as no longer valid.
struct RecordingPush {
    invalid_token: String,
    sent: Mutex<Vec<(String, String)>>,
}

#[async_trait::async_trait]
impl PushSender for RecordingPush {
    async fn send(&self, device: &DeviceTarget, _message: &PushMessage) -> Result<(), PushError> {
        self.sent.lock().unwrap().push((device.platform.clone(), device.token.clone()));
        if device.token == self.invalid_token {
            Err(PushError::Invalid)
        } else {
            Ok(())
        }
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn dispatch_reaches_phones_and_forgets_tokens_the_provider_rejects(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let ana = register(router.clone(), "ana@acme.test").await;
    for device in [
        json!({ "platform": "android", "push_token": "fcm-good", "app_version": "1" }),
        json!({ "platform": "ios", "push_token": "apns-gone", "app_version": "1" }),
        json!({ "platform": "macos", "app_version": "1" }),
    ] {
        let (status, _) = request(router.clone(), "POST", "/api/devices", Some(&ana), Some(device)).await;
        assert_eq!(status, StatusCode::OK);
    }
    let ana_id = user_id(&pool, "ana@acme.test").await;

    let sender = Arc::new(RecordingPush { invalid_token: "apns-gone".into(), sent: Mutex::new(Vec::new()) });
    let message = PushMessage { title: "Hi".into(), body: "There".into(), data: Default::default(), kind: PushKind::Alert };
    dispatch(pool.clone(), sender.clone(), ana_id, message).await;

    let mut sent = sender.sent.lock().unwrap().clone();
    sent.sort();
    assert_eq!(sent, vec![("android".to_string(), "fcm-good".to_string()), ("ios".to_string(), "apns-gone".to_string())], "desktops get no push");
    let tokens: Vec<Option<String>> =
        sqlx::query_scalar("SELECT push_token FROM devices WHERE user_id = $1 ORDER BY platform").bind(ana_id).fetch_all(&pool).await.unwrap();
    assert_eq!(tokens, vec![Some("fcm-good".to_string()), None], "the rejected phone is gone, the desktop stays");
}

#[sqlx::test(migrations = "../../migrations")]
async fn live_activity_pushes_go_to_iphones_by_their_activity_token(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let ana = register(router.clone(), "ana@acme.test").await;
    for device in [
        json!({ "platform": "android", "push_token": "fcm-1", "app_version": "1" }),
        json!({ "platform": "ios", "push_token": "apns-1", "live_activity_start_token": "activity-1", "app_version": "1" }),
        json!({ "platform": "ios", "push_token": "apns-2", "app_version": "1" }),
    ] {
        request(router.clone(), "POST", "/api/devices", Some(&ana), Some(device)).await;
    }
    let ana_id = user_id(&pool, "ana@acme.test").await;

    let sender = Arc::new(RecordingPush { invalid_token: "activity-1".into(), sent: Mutex::new(Vec::new()) });
    let message = PushMessage::live_activity_update(json!({ "phase": "replied", "preview": "Done" }));
    dispatch(pool.clone(), sender.clone(), ana_id, message).await;

    assert_eq!(*sender.sent.lock().unwrap(), vec![("ios".to_string(), "activity-1".to_string())]);
    // A rejected activity token is forgotten, but the phone still gets alerts.
    let rows: Vec<(Option<String>, Option<String>)> =
        sqlx::query_as("SELECT push_token, live_activity_start_token FROM devices WHERE user_id = $1 AND platform = 'ios' ORDER BY push_token")
            .bind(ana_id)
            .fetch_all(&pool)
            .await
            .unwrap();
    assert_eq!(rows, vec![(Some("apns-1".into()), None), (Some("apns-2".into()), None)]);
}
