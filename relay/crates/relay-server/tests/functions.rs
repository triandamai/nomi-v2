//! Apps calling their functions: `/functions/<name>` through the gateway to the functions runtime
//! (wiremock stands in for it), with the caller checked and their claims passed on.
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::Body;
use axum::http::{Request, StatusCode};
use base64::Engine;
use http_body_util::BodyExt;
use relay_control::backend::{ensure_backend, BackendKeys};
use relay_control::functions::{self, SourceFile};
use relay_control::{state, Relay};
use relay_core::backend::{Access, FunctionSettings};
use relay_core::keys::user_token;
use relay_core::settings::FunctionsRuntime;
use relay_core::{Env, Tier};
use relay_server::app::router;
use serde_json::{json, Value};
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;
use wiremock::matchers::{header, method, path, path_regex};
use wiremock::{Mock, MockServer, ResponseTemplate};

const DOMAIN: &str = "api.relay.test";
const SECRET: &str = "runtime-secret";
const ENV: Env = Env::Dev;

struct World {
    router: axum::Router,
    relay: Arc<Relay>,
    keys: BackendKeys,
    runtime: MockServer,
}

async fn world(pool: &PgPool, runtime_url: Option<String>) -> World {
    let runtime = MockServer::start().await;
    Mock::given(method("PUT")).and(path_regex("^/builds/")).respond_with(ResponseTemplate::new(200).set_body_json(json!({ "ok": true }))).mount(&runtime).await;
    let url = runtime_url.unwrap_or_else(|| runtime.uri());
    let shared = HashMap::from([(Env::Dev, pool.clone()), (Env::Production, pool.clone())]);
    let relay = Arc::new(Relay::new(pool.clone(), shared, [9u8; 32], DOMAIN.into()).with_settings(|s| s.functions = Some(FunctionsRuntime { url, secret: SECRET.into() })));
    let keys = ensure_backend(&relay, Uuid::new_v4(), "Shop", Tier::Free, None).await.unwrap();
    World { router: router(relay.clone(), "control-key".into()), relay, keys, runtime }
}

impl World {
    async fn deploy(&self, settings: BTreeMap<String, FunctionSettings>) {
        let files = vec![
            SourceFile { path: "nomi/functions/place-order.ts".into(), content: "export default () => new Response('x')".into() },
            SourceFile { path: "nomi/functions/stripe-hook.ts".into(), content: "export default () => new Response('x')".into() },
        ];
        // Pointed at the stand-in when the world's runtime URL is elsewhere.
        functions::deploy(&self.relay, self.keys.id, ENV, &files, &settings).await.unwrap();
    }

    fn host(&self) -> String {
        format!("{}-dev.{DOMAIN}", self.keys.slug)
    }

    fn anon(&self) -> String {
        self.keys.environments[ENV.as_str()].anon_key.clone()
    }

    async fn user(&self, user: Uuid) -> String {
        let row = state::environment(&self.relay, self.keys.id, ENV).await.unwrap().unwrap();
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        user_token(&row.signing_secret, self.keys.id, ENV, user, Uuid::new_v4(), Some("ana@shop.test"), now, 900)
    }

    async fn send(&self, request: Request<Body>) -> (StatusCode, axum::http::HeaderMap, Vec<u8>) {
        let response = self.router.clone().oneshot(request).await.unwrap();
        let status = response.status();
        let headers = response.headers().clone();
        (status, headers, response.into_body().collect().await.unwrap().to_bytes().to_vec())
    }
}

fn decode_claims(value: &str) -> Value {
    serde_json::from_slice(&base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(value).unwrap()).unwrap()
}

fn webhook() -> BTreeMap<String, FunctionSettings> {
    BTreeMap::from([("stripe-hook".to_string(), FunctionSettings { auth: Access::Public, timeout: 10, schedule: None })])
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_signed_in_users_call_reaches_the_runtime_with_their_claims(pool: PgPool) {
    let w = world(&pool, None).await;
    w.deploy(BTreeMap::new()).await;
    Mock::given(method("POST"))
        .and(path(format!("/run/{}/dev/1/place-order/items", w.keys.id)))
        .and(header("x-relay-runtime", SECRET))
        .and(header("x-relay-timeout", "30"))
        .respond_with(ResponseTemplate::new(201).insert_header("x-order", "42").set_body_json(json!({ "id": 42 })))
        .expect(1)
        .mount(&w.runtime)
        .await;
    let user = Uuid::new_v4();
    let token = w.user(user).await;

    let (status, headers, body) = w
        .send(
            Request::post("/functions/place-order/items?express=1")
                .header("host", w.host())
                .header("apikey", w.anon())
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .header("x-nomi-claims", "forged")
                .body(Body::from(r#"{"cart":3}"#))
                .unwrap(),
        )
        .await;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(headers["x-order"], "42");
    assert_eq!(serde_json::from_slice::<Value>(&body).unwrap(), json!({ "id": 42 }));
    let sent = &w.runtime.received_requests().await.unwrap().into_iter().find(|r| r.url.path().starts_with("/run/")).unwrap();
    assert_eq!(sent.url.query(), Some("express=1"));
    assert_eq!(sent.body, br#"{"cart":3}"#);
    assert_eq!(sent.headers["authorization"], format!("Bearer {token}").as_str());
    let claims = decode_claims(sent.headers["x-nomi-claims"].to_str().unwrap());
    assert_eq!(claims["role"], "authenticated");
    assert_eq!(claims["sub"], user.to_string());
    assert_eq!(claims["email"], "ana@shop.test");
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_function_needs_a_signed_in_caller_unless_it_is_public(pool: PgPool) {
    let w = world(&pool, None).await;
    w.deploy(webhook()).await;
    Mock::given(method("POST")).and(path_regex("/stripe-hook$")).respond_with(ResponseTemplate::new(200)).expect(1).mount(&w.runtime).await;

    let anon = Request::post("/functions/place-order").header("host", w.host()).header("apikey", w.anon()).body(Body::empty()).unwrap();
    let (status, _, body) = w.send(anon).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(serde_json::from_slice::<Value>(&body).unwrap()["code"], "RELAY_SIGN_IN");

    // A webhook comes with no key at all.
    let hook = Request::post("/functions/stripe-hook").header("host", w.host()).header("stripe-signature", "t=1,v1=abc").body(Body::from("{}")).unwrap();
    assert_eq!(w.send(hook).await.0, StatusCode::OK);
    let sent = w.runtime.received_requests().await.unwrap().into_iter().find(|r| r.url.path().ends_with("/stripe-hook")).unwrap();
    assert_eq!(sent.headers["stripe-signature"], "t=1,v1=abc", "the provider's own headers reach the function");
    assert_eq!(decode_claims(sent.headers["x-nomi-claims"].to_str().unwrap())["role"], "anon");
}

#[sqlx::test(migrations = "../../migrations")]
async fn unknown_and_undeployed_functions_are_not_found(pool: PgPool) {
    let w = world(&pool, None).await;
    let call = |name: &str| Request::get(format!("/functions/{name}")).header("host", w.host()).header("apikey", w.anon()).body(Body::empty()).unwrap();
    let (status, _, body) = w.send(call("place-order")).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(serde_json::from_slice::<Value>(&body).unwrap()["code"], "RELAY_NO_FUNCTION");
    w.deploy(BTreeMap::new()).await;
    assert_eq!(w.send(call("refund")).await.0, StatusCode::NOT_FOUND);
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_runtime_that_lost_the_build_gets_it_again_and_the_call_goes_through(pool: PgPool) {
    let w = world(&pool, None).await;
    w.deploy(webhook()).await;
    Mock::given(method("GET")).and(path_regex("/stripe-hook$")).respond_with(ResponseTemplate::new(412).set_body_json(json!({ "code": "NOT_LOADED" }))).up_to_n_times(1).mount(&w.runtime).await;
    Mock::given(method("GET")).and(path_regex("/stripe-hook$")).respond_with(ResponseTemplate::new(200).set_body_string("served")).mount(&w.runtime).await;
    let before = w.runtime.received_requests().await.unwrap().iter().filter(|r| r.method.as_str() == "PUT").count();

    let (status, _, body) = w.send(Request::get("/functions/stripe-hook").header("host", w.host()).body(Body::empty()).unwrap()).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, b"served");
    let after = w.runtime.received_requests().await.unwrap().iter().filter(|r| r.method.as_str() == "PUT").count();
    assert_eq!(after, before + 1, "the build was sent again once");
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_runtime_that_stays_down_answers_503_with_retry_after(pool: PgPool) {
    let w = world(&pool, None).await;
    w.deploy(webhook()).await;
    // Now point Relay at an address nothing listens on.
    let relay = Arc::new(Relay::new(pool.clone(), HashMap::from([(Env::Dev, pool.clone()), (Env::Production, pool.clone())]), [9u8; 32], DOMAIN.into()).with_settings(|s| {
        s.functions = Some(FunctionsRuntime { url: "http://127.0.0.1:9".into(), secret: SECRET.into() })
    }));
    let router = router(relay, "control-key".into());

    let response = router.oneshot(Request::get("/functions/stripe-hook").header("host", w.host()).body(Body::empty()).unwrap()).await.unwrap();

    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(response.headers()["retry-after"], "2");
}

#[sqlx::test(migrations = "../../migrations")]
async fn without_a_runtime_set_functions_say_so(pool: PgPool) {
    let shared = HashMap::from([(Env::Dev, pool.clone()), (Env::Production, pool.clone())]);
    let relay = Arc::new(Relay::new(pool.clone(), shared, [9u8; 32], DOMAIN.into()));
    let keys = ensure_backend(&relay, Uuid::new_v4(), "Shop", Tier::Free, None).await.unwrap();
    let router = router(relay, "control-key".into());
    let request = Request::get("/functions/x").header("host", format!("{}-dev.{DOMAIN}", keys.slug)).body(Body::empty()).unwrap();
    let response = router.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
}
