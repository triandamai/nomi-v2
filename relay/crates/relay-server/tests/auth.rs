//! Sign-in through the gateway, in the shapes `nomi-relay`'s `auth` reads.
use std::collections::HashMap;
use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use relay_auth::mail::RecordingOutbox;
use relay_control::backend::{ensure_backend, BackendKeys};
use relay_control::Relay;
use relay_core::settings::Smtp;
use relay_core::{Env, Tier};
use relay_server::app::router;
use serde_json::{json, Value};
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;

const DOMAIN: &str = "api.relay.test";
const CONTROL: &str = "control-key";

struct World {
    router: axum::Router,
    outbox: Arc<RecordingOutbox>,
    keys: BackendKeys,
    other: BackendKeys,
}

fn shared_sender() -> Smtp {
    Smtp { host: "smtp.shared.test".into(), port: 587, user: "nomi".into(), password: "pw".into(), from: "Nomi <codes@nomi.test>".into(), security: "starttls".into() }
}

async fn world(pool: &PgPool) -> World {
    let shared = HashMap::from([(Env::Dev, pool.clone()), (Env::Production, pool.clone())]);
    let outbox = Arc::new(RecordingOutbox::default());
    let relay = Relay::new(pool.clone(), shared, [9u8; 32], DOMAIN.into()).with_settings(|s| s.shared_smtp = Some(shared_sender())).with_outbox(outbox.clone());
    let relay = Arc::new(relay);
    let keys = ensure_backend(&relay, Uuid::new_v4(), "Bakery", Tier::Free, None).await.unwrap();
    let other = ensure_backend(&relay, Uuid::new_v4(), "Florist", Tier::Free, None).await.unwrap();
    World { router: router(relay, CONTROL.into()), outbox, keys, other }
}

impl World {
    fn host(&self) -> String {
        format!("{}-dev.{DOMAIN}", self.keys.slug)
    }

    fn key(&self, which: &str) -> String {
        let dev = &self.keys.environments["dev"];
        if which == "service" { dev.service_key.clone() } else { dev.anon_key.clone() }
    }

    async fn call(&self, method: &str, path: &str, bearer: Option<&str>, body: Option<Value>) -> (StatusCode, Value) {
        let mut request = Request::builder().method(method).uri(path).header("host", self.host()).header("apikey", self.key("anon"));
        if let Some(bearer) = bearer {
            request = request.header("authorization", format!("Bearer {bearer}"));
        }
        let request = match body {
            Some(body) => request.header("content-type", "application/json").body(Body::from(body.to_string())).unwrap(),
            None => request.body(Body::empty()).unwrap(),
        };
        let response = self.router.clone().oneshot(request).await.unwrap();
        let status = response.status();
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        (status, serde_json::from_slice(&bytes).unwrap_or(Value::Null))
    }

    async fn sign_in(&self, email: &str) -> Value {
        let (status, body) = self.call("POST", "/auth/otp", None, Some(json!({ "email": email, "create_user": true }))).await;
        assert_eq!(status, StatusCode::OK, "{body}");
        let code = self.outbox.sent().last().unwrap().1.subject.rsplit(' ').next().unwrap().to_string();
        let (status, session) = self.call("POST", "/auth/verify", None, Some(json!({ "email": email, "code": code }))).await;
        assert_eq!(status, StatusCode::OK, "{session}");
        session
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_person_signs_in_with_a_code_refreshes_and_signs_out(pool: PgPool) {
    let w = world(&pool).await;
    let session = w.sign_in("ana@example.test").await;
    assert_eq!(w.outbox.sent()[0].0.host, "smtp.shared.test");
    assert!(w.outbox.sent()[0].1.subject.contains("Bakery"));
    assert_eq!(session["token_type"], "bearer");
    assert!(session["expires_at"].as_u64().unwrap() > 0);
    assert_eq!(session["user"]["email"], "ana@example.test");
    let access = session["access_token"].as_str().unwrap();

    let (status, user) = w.call("GET", "/auth/user", Some(access), None).await;
    assert_eq!(status, StatusCode::OK, "{user}");
    assert_eq!(user["email_verified"], true);

    let (status, user) = w.call("PATCH", "/auth/user", Some(access), Some(json!({ "user_metadata": { "name": "Ana" } }))).await;
    assert_eq!(status, StatusCode::OK, "{user}");
    assert_eq!(user["user_metadata"]["name"], "Ana");
    let (status, refused) = w.call("PATCH", "/auth/user", Some(access), Some(json!({ "email": "new@example.test" }))).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(refused["code"], "EMAIL_CHANGE_UNSUPPORTED");

    let refresh = session["refresh_token"].as_str().unwrap();
    let (status, next) = w.call("POST", "/auth/token?grant_type=refresh_token", Some(&w.key("anon")), Some(json!({ "refresh_token": refresh }))).await;
    assert_eq!(status, StatusCode::OK, "{next}");
    let (status, reused) = w.call("POST", "/auth/token?grant_type=refresh_token", Some(&w.key("anon")), Some(json!({ "refresh_token": refresh }))).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert!(reused["message"].is_string() && reused["code"] == "REFRESH_TOKEN_REUSED");

    let fresh = w.sign_in("ana@example.test").await;
    let fresh_access = fresh["access_token"].as_str().unwrap();
    let (status, _) = w.call("POST", "/auth/logout", Some(fresh_access), None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    assert_eq!(w.call("GET", "/auth/user", Some(fresh_access), None).await.0, StatusCode::UNAUTHORIZED, "a signed-out session's token stops working on /auth");
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_token_from_another_project_is_refused(pool: PgPool) {
    let w = world(&pool).await;
    let session = w.sign_in("ana@example.test").await;
    let other_host = format!("{}-dev.{DOMAIN}", w.other.slug);
    let request = Request::get("/auth/user")
        .header("host", other_host)
        .header("apikey", w.other.environments["dev"].anon_key.clone())
        .header("authorization", format!("Bearer {}", session["access_token"].as_str().unwrap()))
        .body(Body::empty())
        .unwrap();
    assert_eq!(w.router.clone().oneshot(request).await.unwrap().status(), StatusCode::UNAUTHORIZED);
}

#[sqlx::test(migrations = "../../migrations")]
async fn user_management_needs_the_service_key(pool: PgPool) {
    let w = world(&pool).await;
    let session = w.sign_in("ana@example.test").await;
    let user_token = session["access_token"].as_str().unwrap().to_string();
    let anon = w.key("anon");
    let service = w.key("service");
    assert_eq!(w.call("GET", "/auth/admin/users", Some(&anon), None).await.0, StatusCode::FORBIDDEN);
    assert_eq!(w.call("GET", "/auth/admin/users", Some(&user_token), None).await.0, StatusCode::FORBIDDEN);

    let (status, made) = w.call("POST", "/auth/admin/users", Some(&service), Some(json!({ "email": "bo@example.test", "email_verified": true }))).await;
    assert_eq!(status, StatusCode::OK, "{made}");
    let id = made["id"].as_str().unwrap().to_string();
    let (_, listed) = w.call("GET", "/auth/admin/users?page=1&per_page=10", Some(&service), None).await;
    assert_eq!(listed["total"], 2);
    let (status, updated) = w.call("PATCH", &format!("/auth/admin/users/{id}"), Some(&service), Some(json!({ "user_metadata": { "plan": "pro" } }))).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(updated["user_metadata"]["plan"], "pro");
    assert_eq!(w.call("GET", &format!("/auth/admin/users/{id}"), Some(&service), None).await.0, StatusCode::OK);
    assert_eq!(w.call("DELETE", &format!("/auth/admin/users/{id}"), Some(&service), None).await.0, StatusCode::NO_CONTENT);
    assert_eq!(w.call("GET", &format!("/auth/admin/users/{id}"), Some(&service), None).await.0, StatusCode::NOT_FOUND);

    let ana = session["user"]["id"].as_str().unwrap();
    w.call("DELETE", &format!("/auth/admin/users/{ana}"), Some(&service), None).await;
    let (status, _) = w.call("POST", "/auth/token?grant_type=refresh_token", Some(&anon), Some(json!({ "refresh_token": session["refresh_token"] }))).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED, "a deleted user's session ends");
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_project_with_its_own_smtp_sends_through_it_and_without_any_sender_says_so(pool: PgPool) {
    let w = world(&pool).await;
    let own = json!({ "methods": ["email_code"], "session_ttl_days": 7, "smtp": { "host": "smtp.bakery.test", "port": 465, "user": "u", "password": "bakery-pw", "from": "Bakery <hi@bakery.test>", "security": "tls" } });
    let path = format!("/control/projects/{}/environments/dev/auth", w.keys.id);
    let put = Request::put(&path).header("host", DOMAIN).header("authorization", format!("Bearer {CONTROL}")).header("content-type", "application/json").body(Body::from(own.to_string())).unwrap();
    assert_eq!(w.router.clone().oneshot(put).await.unwrap().status(), StatusCode::OK);
    let get = Request::get(&path).header("host", DOMAIN).header("authorization", format!("Bearer {CONTROL}")).body(Body::empty()).unwrap();
    let shown = w.router.clone().oneshot(get).await.unwrap().into_body().collect().await.unwrap().to_bytes();
    let shown = String::from_utf8_lossy(&shown).to_string();
    assert!(shown.contains("smtp.bakery.test") && !shown.contains("bakery-pw"), "{shown}");

    w.sign_in("ana@example.test").await;
    assert_eq!(w.outbox.sent().last().unwrap().0.host, "smtp.bakery.test");

    let off = json!({ "methods": [], "session_ttl_days": 7 });
    let put = Request::put(&path).header("host", DOMAIN).header("authorization", format!("Bearer {CONTROL}")).header("content-type", "application/json").body(Body::from(off.to_string())).unwrap();
    w.router.clone().oneshot(put).await.unwrap();
    let (status, refused) = w.call("POST", "/auth/otp", None, Some(json!({ "email": "ana@example.test" }))).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(refused["code"], "METHOD_OFF");
}
