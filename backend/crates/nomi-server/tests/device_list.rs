//! The person's device list: each app registers under the device id it keeps, sign-ins made on a
//! device end when that device is signed out, and presence follows what the apps' connections say.
use axum::{body::Body, http::{Request, StatusCode}};
use http_body_util::BodyExt;
use nomi_realtime::{PresenceState, PresenceUpdate};
use nomi_server::app::{build_router, AppState};
use serde_json::{json, Value};
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;

const SECRET: &str = "test-secret-do-not-use-in-prod";
const DEVICE_HEADER: &str = "x-device-id";

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

async fn request(router: &axum::Router, method: &str, uri: &str, token: Option<&str>, device: Option<Uuid>, body: Option<Value>) -> (StatusCode, Value) {
    let mut builder = Request::builder().method(method).uri(uri);
    if let Some(token) = token {
        builder = builder.header("authorization", format!("Bearer {token}"));
    }
    if let Some(device) = device {
        builder = builder.header(DEVICE_HEADER, device.to_string());
    }
    let body = match body {
        Some(b) => {
            builder = builder.header("content-type", "application/json");
            Body::from(b.to_string())
        }
        None => Body::empty(),
    };
    let response = router.clone().oneshot(builder.body(body).unwrap()).await.unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    (status, if bytes.is_empty() { Value::Null } else { serde_json::from_slice(&bytes).unwrap_or(Value::Null) })
}

/// Signs up, then signs in on `device`; returns (access token, refresh token).
async fn sign_in(router: &axum::Router, email: &str, device: Uuid) -> (String, String) {
    let account = json!({ "email": email, "password": "correct-password", "org": { "mode": "create", "name": "Acme" } });
    let (status, _) = request(router, "POST", "/api/auth/register", None, None, Some(account)).await;
    assert!(status.is_success(), "{status}");
    let credentials = json!({ "email": email, "password": "correct-password" });
    let (_, body) = request(router, "POST", "/api/auth/login", None, Some(device), Some(credentials)).await;
    (body["access_token"].as_str().unwrap().to_string(), body["refresh_token"].as_str().unwrap().to_string())
}

fn phone(device: Uuid) -> Value {
    json!({
        "device_id": device, "platform": "android", "name": "Pixel 9", "brand": "google", "model": "Pixel 9",
        "os_name": "Android", "os_version": "15", "app_version": "1.4.0"
    })
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_device_registers_under_its_own_id_and_lists_with_its_details(pool: PgPool) {
    let router = build_router(test_state(pool));
    let device = Uuid::new_v4();
    let (token, _) = sign_in(&router, "devices-list@nomi.test", device).await;

    let (status, body) = request(&router, "POST", "/api/devices", Some(&token), Some(device), Some(phone(device))).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["id"], json!(device));
    // Registering again (every app start) keeps one row and takes the new details.
    let mut updated = phone(device);
    updated["app_version"] = json!("1.5.0");
    request(&router, "POST", "/api/devices", Some(&token), Some(device), Some(updated)).await;

    let (status, list) = request(&router, "GET", "/api/devices", Some(&token), Some(device), None).await;
    assert_eq!(status, StatusCode::OK);
    let list = list.as_array().unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0]["name"], "Pixel 9");
    assert_eq!(list[0]["brand"], "google");
    assert_eq!(list[0]["os_version"], "15");
    assert_eq!(list[0]["app_version"], "1.5.0");
    assert_eq!(list[0]["presence"], "offline");
    assert_eq!(list[0]["current"], true);

    let (_, from_elsewhere) = request(&router, "GET", "/api/devices", Some(&token), Some(Uuid::new_v4()), None).await;
    assert_eq!(from_elsewhere[0]["current"], false);
}

#[sqlx::test(migrations = "../../migrations")]
async fn signing_a_device_out_ends_the_sign_ins_made_on_it(pool: PgPool) {
    let router = build_router(test_state(pool));
    let laptop = Uuid::new_v4();
    let (token, laptop_refresh) = sign_in(&router, "devices-out@nomi.test", laptop).await;
    request(&router, "POST", "/api/devices", Some(&token), Some(laptop), Some(phone(laptop))).await;
    // A second sign-in, on another device, stays.
    let tablet = Uuid::new_v4();
    let credentials = json!({ "email": "devices-out@nomi.test", "password": "correct-password" });
    let (_, tablet_login) = request(&router, "POST", "/api/auth/login", None, Some(tablet), Some(credentials)).await;
    let tablet_refresh = tablet_login["refresh_token"].as_str().unwrap().to_string();

    let (status, _) = request(&router, "DELETE", &format!("/api/devices/{laptop}"), Some(&token), Some(tablet), None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (status, _) = request(&router, "POST", "/api/auth/refresh", None, None, Some(json!({ "refresh_token": laptop_refresh }))).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    let (status, _) = request(&router, "POST", "/api/auth/refresh", None, None, Some(json!({ "refresh_token": tablet_refresh }))).await;
    assert_eq!(status, StatusCode::OK);
    let (_, list) = request(&router, "GET", "/api/devices", Some(&token), Some(tablet), None).await;
    assert!(list.as_array().unwrap().is_empty());
}

#[sqlx::test(migrations = "../../migrations")]
async fn someone_elses_device_cant_be_signed_out(pool: PgPool) {
    let router = build_router(test_state(pool));
    let theirs = Uuid::new_v4();
    let (owner, _) = sign_in(&router, "devices-owner@nomi.test", theirs).await;
    request(&router, "POST", "/api/devices", Some(&owner), Some(theirs), Some(phone(theirs))).await;
    let (stranger, _) = sign_in(&router, "devices-stranger@nomi.test", Uuid::new_v4()).await;

    let (status, _) = request(&router, "DELETE", &format!("/api/devices/{theirs}"), Some(&stranger), None, None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

async fn presence_of(pool: &PgPool, device: Uuid) -> String {
    sqlx::query_scalar("SELECT presence FROM devices WHERE id = $1").bind(device).fetch_one(pool).await.unwrap()
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_device_is_online_while_a_connection_is_in_front_and_offline_when_none_is_left(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let browser = Uuid::new_v4();
    let (token, _) = sign_in(&router, "devices-presence@nomi.test", browser).await;
    request(&router, "POST", "/api/devices", Some(&token), Some(browser), Some(phone(browser))).await;
    let user: Uuid = sqlx::query_scalar("SELECT user_id FROM devices WHERE id = $1").bind(browser).fetch_one(&pool).await.unwrap();
    let (tab_a, tab_b) = (format!("{browser}-a"), format!("{browser}-b"));
    let (conn_a, conn_b) = (Uuid::new_v4(), Uuid::new_v4());
    let say = |connection: Uuid, state: PresenceState| PresenceUpdate { device_id: browser, connection, state };

    nomi_server::presence::apply(&pool, user, &tab_a, &say(conn_a, PresenceState::Online)).await.unwrap();
    assert_eq!(presence_of(&pool, browser).await, "online");
    nomi_server::presence::apply(&pool, user, &tab_b, &say(conn_b, PresenceState::Away)).await.unwrap();
    nomi_server::presence::apply(&pool, user, &tab_a, &say(conn_a, PresenceState::Away)).await.unwrap();
    assert_eq!(presence_of(&pool, browser).await, "away");
    nomi_server::presence::apply(&pool, user, &tab_a, &say(conn_a, PresenceState::Offline)).await.unwrap();
    assert_eq!(presence_of(&pool, browser).await, "away", "the other tab is still open");
    nomi_server::presence::apply(&pool, user, &tab_b, &say(conn_b, PresenceState::Offline)).await.unwrap();
    assert_eq!(presence_of(&pool, browser).await, "offline");
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_replaced_connections_late_goodbye_leaves_its_replacement_online(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let device = Uuid::new_v4();
    let (token, _) = sign_in(&router, "devices-takeover@nomi.test", device).await;
    request(&router, "POST", "/api/devices", Some(&token), Some(device), Some(phone(device))).await;
    let user: Uuid = sqlx::query_scalar("SELECT user_id FROM devices WHERE id = $1").bind(device).fetch_one(&pool).await.unwrap();
    let client_id = device.to_string();
    let (old, new) = (Uuid::new_v4(), Uuid::new_v4());

    nomi_server::presence::apply(&pool, user, &client_id, &PresenceUpdate { device_id: device, connection: old, state: PresenceState::Online }).await.unwrap();
    nomi_server::presence::apply(&pool, user, &client_id, &PresenceUpdate { device_id: device, connection: new, state: PresenceState::Online }).await.unwrap();
    nomi_server::presence::apply(&pool, user, &client_id, &PresenceUpdate { device_id: device, connection: old, state: PresenceState::Offline }).await.unwrap();
    assert_eq!(presence_of(&pool, device).await, "online");
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_device_someone_else_registered_stays_dark(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let device = Uuid::new_v4();
    let (token, _) = sign_in(&router, "devices-mine@nomi.test", device).await;
    request(&router, "POST", "/api/devices", Some(&token), Some(device), Some(phone(device))).await;
    let stranger = Uuid::new_v4();

    nomi_server::presence::apply(&pool, stranger, &device.to_string(), &PresenceUpdate { device_id: device, connection: Uuid::new_v4(), state: PresenceState::Online }).await.unwrap();
    assert_eq!(presence_of(&pool, device).await, "offline");
}
