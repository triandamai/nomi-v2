//! Files through the gateway: `nomi.storage` uploads, downloads, lists, removes and signs them
//! under each bucket's rules, and public buckets serve without a key. An in-memory store stands
//! in for S3.
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use object_store::memory::InMemory;
use relay_control::backend::{ensure_backend, BackendKeys};
use relay_control::{state, storage, Relay};
use relay_core::backend::StorageConfig;
use relay_core::keys::user_token;
use relay_core::{Env, Tier};
use relay_server::app::router;
use serde_json::{json, Value};
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;

const DOMAIN: &str = "api.relay.test";
const ENV: Env = Env::Dev;

struct World {
    router: axum::Router,
    relay: Arc<Relay>,
    keys: BackendKeys,
}

struct Answer {
    status: StatusCode,
    headers: axum::http::HeaderMap,
    bytes: Vec<u8>,
}

impl Answer {
    fn json(&self) -> Value {
        serde_json::from_slice(&self.bytes).unwrap_or(Value::Null)
    }
}

async fn world(pool: &PgPool, buckets: Value) -> World {
    let shared = HashMap::from([(Env::Dev, pool.clone()), (Env::Production, pool.clone())]);
    let relay = Arc::new(Relay::new(pool.clone(), shared, [9u8; 32], DOMAIN.into()).with_store(Arc::new(InMemory::new())));
    let keys = ensure_backend(&relay, Uuid::new_v4(), "Photos", Tier::Free, None).await.unwrap();
    let config: StorageConfig = serde_json::from_value(json!({ "buckets": buckets })).unwrap();
    storage::save(&relay, keys.id, ENV, &config).await.unwrap();
    World { router: router(relay.clone(), "control-key".into()), relay, keys }
}

fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
}

impl World {
    fn host(&self) -> String {
        format!("{}-dev.{DOMAIN}", self.keys.slug)
    }

    fn anon(&self) -> String {
        self.keys.environments[ENV.as_str()].anon_key.clone()
    }

    fn service(&self) -> String {
        self.keys.environments[ENV.as_str()].service_key.clone()
    }

    async fn user(&self, user: Uuid) -> String {
        let row = state::environment(&self.relay, self.keys.id, ENV).await.unwrap().unwrap();
        user_token(&row.signing_secret, self.keys.id, ENV, user, Uuid::new_v4(), None, now(), 900)
    }

    async fn call(&self, method: &str, path: &str, bearer: Option<&str>, headers: &[(&str, &str)], body: Body) -> Answer {
        let mut request = Request::builder().method(method).uri(path).header("host", self.host()).header("apikey", self.anon());
        if let Some(bearer) = bearer {
            request = request.header("authorization", format!("Bearer {bearer}"));
        }
        for (name, value) in headers {
            request = request.header(*name, *value);
        }
        self.send(request.body(body).unwrap()).await
    }

    async fn send(&self, request: Request<Body>) -> Answer {
        let response = self.router.clone().oneshot(request).await.unwrap();
        let status = response.status();
        let headers = response.headers().clone();
        let bytes = response.into_body().collect().await.unwrap().to_bytes().to_vec();
        Answer { status, headers, bytes }
    }

    async fn upload(&self, bearer: &str, path: &str, content_type: &str, bytes: &'static [u8]) -> Answer {
        self.call("PUT", &format!("/storage/object/{path}"), Some(bearer), &[("content-type", content_type)], Body::from(bytes)).await
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn an_owner_uploads_to_their_folder_and_reads_it_back(pool: PgPool) {
    let w = world(&pool, json!([{ "name": "avatars", "read": "owner", "write": "owner" }])).await;
    let user = Uuid::new_v4();
    let token = w.user(user).await;

    let put = w.upload(&token, &format!("avatars/{user}/me%201.png"), "image/png", b"png-bytes").await;
    assert_eq!(put.status, StatusCode::OK, "{:?}", put.json());
    assert_eq!(put.json()["path"], format!("{user}/me 1.png"));

    let got = w.call("GET", &format!("/storage/object/avatars/{user}/me%201.png"), Some(&token), &[], Body::empty()).await;
    assert_eq!(got.status, StatusCode::OK);
    assert_eq!(got.bytes, b"png-bytes");
    assert_eq!(got.headers["content-type"], "image/png");

    let listed = w.call("GET", &format!("/storage/list/avatars?prefix={user}"), Some(&token), &[], Body::empty()).await;
    assert_eq!(listed.status, StatusCode::OK);
    assert_eq!(listed.json()[0]["name"], "me 1.png");
    assert_eq!(listed.json()[0]["size"], 9);
}

#[sqlx::test(migrations = "../../migrations")]
async fn someone_elses_folder_and_anonymous_callers_are_refused(pool: PgPool) {
    let w = world(&pool, json!([{ "name": "avatars", "read": "owner", "write": "owner" }])).await;
    let (owner, other) = (Uuid::new_v4(), Uuid::new_v4());
    w.upload(&w.user(owner).await, &format!("avatars/{owner}/a.png"), "image/png", b"x").await;

    let theirs = w.user(other).await;
    assert_eq!(w.upload(&theirs, &format!("avatars/{owner}/a.png"), "image/png", b"y").await.status, StatusCode::FORBIDDEN);
    let read = w.call("GET", &format!("/storage/object/avatars/{owner}/a.png"), Some(&theirs), &[], Body::empty()).await;
    assert_eq!(read.status, StatusCode::FORBIDDEN);
    assert_eq!(read.json()["code"], "RELAY_FORBIDDEN");
    let anon = w.call("GET", &format!("/storage/object/avatars/{owner}/a.png"), None, &[], Body::empty()).await;
    assert_eq!(anon.status, StatusCode::FORBIDDEN);
    let listed = w.call("GET", &format!("/storage/list/avatars?prefix={owner}"), Some(&theirs), &[], Body::empty()).await;
    assert_eq!(listed.status, StatusCode::FORBIDDEN);
    let removed = w.call("DELETE", "/storage/object/avatars", Some(&theirs), &[("content-type", "application/json")], Body::from(json!({ "paths": [format!("{owner}/a.png")] }).to_string())).await;
    assert_eq!(removed.status, StatusCode::FORBIDDEN);

    // The service key passes every rule.
    let service = w.service();
    assert_eq!(w.call("GET", &format!("/storage/object/avatars/{owner}/a.png"), Some(&service), &[], Body::empty()).await.status, StatusCode::OK);
}

#[sqlx::test(migrations = "../../migrations")]
async fn an_upload_never_overwrites_unless_asked_and_remove_deletes(pool: PgPool) {
    let w = world(&pool, json!([{ "name": "docs" }])).await;
    let token = w.user(Uuid::new_v4()).await;
    assert_eq!(w.upload(&token, "docs/a.txt", "text/plain", b"one").await.status, StatusCode::OK);

    let again = w.upload(&token, "docs/a.txt", "text/plain", b"two").await;
    assert_eq!(again.status, StatusCode::CONFLICT);
    assert_eq!(again.json()["code"], "RELAY_EXISTS");
    let replaced = w.call("PUT", "/storage/object/docs/a.txt", Some(&token), &[("content-type", "text/plain"), ("x-nomi-upsert", "true")], Body::from("two")).await;
    assert_eq!(replaced.status, StatusCode::OK);
    assert_eq!(w.call("GET", "/storage/object/docs/a.txt", Some(&token), &[], Body::empty()).await.bytes, b"two");

    let removed = w.call("DELETE", "/storage/object/docs", Some(&token), &[("content-type", "application/json")], Body::from(json!({ "paths": ["a.txt", "never-was.txt"] }).to_string())).await;
    assert_eq!(removed.status, StatusCode::OK);
    let gone = w.call("GET", "/storage/object/docs/a.txt", Some(&token), &[], Body::empty()).await;
    assert_eq!(gone.status, StatusCode::NOT_FOUND);
    assert_eq!(gone.json()["code"], "RELAY_NOT_FOUND");
}

#[sqlx::test(migrations = "../../migrations")]
async fn size_type_bucket_and_path_limits_hold(pool: PgPool) {
    let w = world(&pool, json!([{ "name": "pics", "max_size_mb": 1, "types": ["image/*"] }])).await;
    let token = w.user(Uuid::new_v4()).await;

    assert_eq!(w.upload(&token, "pics/a.txt", "text/plain", b"x").await.status, StatusCode::UNSUPPORTED_MEDIA_TYPE);
    let big: &'static [u8] = Box::leak(vec![0u8; 1024 * 1024 + 1].into_boxed_slice());
    let too_big = w.upload(&token, "pics/big.png", "image/png", big).await;
    assert_eq!(too_big.status, StatusCode::PAYLOAD_TOO_LARGE);
    assert_eq!(too_big.json()["code"], "RELAY_TOO_LARGE");
    assert_eq!(w.upload(&token, "nope/a.png", "image/png", b"x").await.status, StatusCode::NOT_FOUND);
    for path in ["pics/..%2Fescape.png", "pics/a/../b.png", "pics/a//b.png", "pics/"] {
        let refused = w.upload(&token, path, "image/png", b"x").await;
        assert_eq!(refused.status, StatusCode::BAD_REQUEST, "{path}");
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_signed_url_opens_one_file_without_a_key_until_it_expires(pool: PgPool) {
    let w = world(&pool, json!([{ "name": "invoices", "read": "owner", "write": "service" }])).await;
    let user = Uuid::new_v4();
    let service = w.service();
    w.upload(&service, &format!("invoices/{user}/may.pdf"), "application/pdf", b"%PDF").await;
    w.upload(&service, &format!("invoices/{user}/june.pdf"), "application/pdf", b"%PDF-june").await;
    let token = w.user(user).await;
    assert_eq!(w.upload(&token, &format!("invoices/{user}/x.pdf"), "application/pdf", b"x").await.status, StatusCode::FORBIDDEN, "write is service only");

    let signed = w.call("POST", &format!("/storage/sign/invoices/{user}/may.pdf"), Some(&token), &[("content-type", "application/json")], Body::from(json!({ "expires_in": 60 }).to_string())).await;
    assert_eq!(signed.status, StatusCode::OK, "{:?}", signed.json());
    let url = signed.json()["signed_url"].as_str().unwrap().to_string();
    assert!(url.starts_with(&format!("/storage/sign/invoices/{user}/may.pdf?token=")), "{url}");

    let open = |uri: String| Request::get(uri).header("host", w.host()).body(Body::empty()).unwrap();
    let got = w.send(open(url.clone())).await;
    assert_eq!(got.status, StatusCode::OK);
    assert_eq!(got.bytes, b"%PDF");

    let token_part = url.split_once("?token=").unwrap().1;
    let other_file = w.send(open(format!("/storage/sign/invoices/{user}/june.pdf?token={token_part}"))).await;
    assert_eq!(other_file.status, StatusCode::FORBIDDEN, "a token opens only its own file");
    assert_eq!(w.send(open(format!("/storage/sign/invoices/{user}/may.pdf?token=nonsense"))).await.status, StatusCode::FORBIDDEN);

    let someone_else = w.user(Uuid::new_v4()).await;
    let refused = w.call("POST", &format!("/storage/sign/invoices/{user}/may.pdf"), Some(&someone_else), &[("content-type", "application/json")], Body::from("{}")).await;
    assert_eq!(refused.status, StatusCode::FORBIDDEN, "signing needs read access");
}

#[sqlx::test(migrations = "../../migrations")]
async fn public_buckets_serve_without_a_key_and_private_ones_dont(pool: PgPool) {
    let w = world(&pool, json!([{ "name": "site", "public": true, "write": "service" }, { "name": "private", "write": "service" }])).await;
    let service = w.service();
    w.upload(&service, "site/logo.svg", "image/svg+xml", b"<svg/>").await;
    w.upload(&service, "private/a.txt", "text/plain", b"secret").await;

    let open = |uri: &str| Request::get(uri).header("host", w.host()).body(Body::empty()).unwrap();
    let logo = w.send(open("/storage/public/site/logo.svg")).await;
    assert_eq!(logo.status, StatusCode::OK);
    assert_eq!(logo.bytes, b"<svg/>");
    assert_eq!(w.send(open("/storage/public/private/a.txt")).await.status, StatusCode::NOT_FOUND);
}

#[sqlx::test(migrations = "../../migrations")]
async fn without_file_storage_set_relay_says_so(pool: PgPool) {
    let shared = HashMap::from([(Env::Dev, pool.clone()), (Env::Production, pool.clone())]);
    let relay = Arc::new(Relay::new(pool.clone(), shared, [9u8; 32], DOMAIN.into()));
    let keys = ensure_backend(&relay, Uuid::new_v4(), "Photos", Tier::Free, None).await.unwrap();
    let w = World { router: router(relay.clone(), "control-key".into()), relay, keys };
    let token = w.service();

    let answer = w.upload(&token, "docs/a.txt", "text/plain", b"x").await;

    assert_eq!(answer.status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(answer.json()["code"], "RELAY_NO_STORAGE");
}
