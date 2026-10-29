//! OAuth through the gateway, the way `nomi-relay` drives it, with wiremock as the issuer.
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use base64::Engine;
use http_body_util::BodyExt;
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use relay_auth::mail::RecordingOutbox;
use relay_control::backend::{ensure_backend, BackendKeys};
use relay_control::Relay;
use relay_core::settings::Smtp;
use relay_core::{Env, Tier};
use relay_server::app::router;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const DOMAIN: &str = "api.relay.test";
const CONTROL: &str = "control-key";
const ISSUER_KEY: &str = include_str!("../../relay-auth/tests/keys/issuer_rsa.pem");
const ISSUER_N: &str = include_str!("../../relay-auth/tests/keys/issuer_rsa.n");
const CLIENT: &str = "bakery-web";
const APP: &str = "https://bakery.example";
const VERIFIER: &str = "a-pkce-verifier-that-is-long-enough-0123456789";

fn challenge(verifier: &str) -> String {
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()))
}

fn now() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs()
}

struct World {
    router: axum::Router,
    issuer: MockServer,
    keys: BackendKeys,
    outbox: Arc<RecordingOutbox>,
}

async fn world(pool: &PgPool) -> World {
    let shared = HashMap::from([(Env::Dev, pool.clone()), (Env::Production, pool.clone())]);
    let outbox = Arc::new(RecordingOutbox::default());
    let sender = Smtp { host: "smtp.test".into(), port: 587, user: String::new(), password: String::new(), from: "Nomi <c@nomi.test>".into(), security: "none".into() };
    let relay = Arc::new(Relay::new(pool.clone(), shared, [9u8; 32], DOMAIN.into()).with_settings(|s| s.shared_smtp = Some(sender)).with_outbox(outbox.clone()));
    let keys = ensure_backend(&relay, Uuid::new_v4(), "Bakery", Tier::Free, None).await.unwrap();
    let issuer = MockServer::start().await;
    let base = issuer.uri();
    Mock::given(method("GET")).and(path("/.well-known/openid-configuration")).respond_with(ResponseTemplate::new(200).set_body_json(json!({
        "issuer": base, "authorization_endpoint": format!("{base}/authorize"), "token_endpoint": format!("{base}/token"), "jwks_uri": format!("{base}/jwks")
    }))).mount(&issuer).await;
    Mock::given(method("GET")).and(path("/jwks")).respond_with(ResponseTemplate::new(200).set_body_json(json!({ "keys": [{ "kty": "RSA", "kid": "k", "alg": "RS256", "n": ISSUER_N.trim(), "e": "AQAB" }] }))).mount(&issuer).await;
    let w = World { router: router(relay, CONTROL.into()), issuer, keys, outbox };
    let config = json!({ "methods": ["email_code", "oidc:acme"], "session_ttl_days": 30, "redirect_urls": [APP], "oidc": [{ "id": "acme", "issuer": base, "client_id": CLIENT }] });
    assert_eq!(w.control("PUT", "/environments/dev/auth", Some(config)).await.0, StatusCode::OK);
    assert_eq!(w.control("PUT", "/auth/secrets/NOMI_OAUTH_OIDC_ACME_SECRET", Some(json!({ "value": "acme-secret" }))).await.0, StatusCode::OK);
    w
}

impl World {
    fn host(&self) -> String {
        format!("{}-dev.{DOMAIN}", self.keys.slug)
    }

    fn anon(&self) -> String {
        self.keys.environments["dev"].anon_key.clone()
    }

    async fn control(&self, method: &str, rest: &str, body: Option<Value>) -> (StatusCode, Value) {
        let path = format!("/control/projects/{}{rest}", self.keys.id);
        let request = Request::builder().method(method).uri(path).header("host", DOMAIN).header("authorization", format!("Bearer {CONTROL}"));
        let request = match body {
            Some(b) => request.header("content-type", "application/json").body(Body::from(b.to_string())).unwrap(),
            None => request.body(Body::empty()).unwrap(),
        };
        send(&self.router, request).await
    }

    async fn app(&self, method: &str, uri: &str, bearer: Option<&str>, body: Option<Value>) -> (StatusCode, Value, Option<String>) {
        let mut request = Request::builder().method(method).uri(uri).header("host", self.host()).header("apikey", self.anon());
        if let Some(b) = bearer {
            request = request.header("authorization", format!("Bearer {b}"));
        }
        let request = match body {
            Some(b) => request.header("content-type", "application/json").body(Body::from(b.to_string())).unwrap(),
            None => request.body(Body::empty()).unwrap(),
        };
        let response = self.router.clone().oneshot(request).await.unwrap();
        let status = response.status();
        let location = response.headers().get("location").map(|l| l.to_str().unwrap().to_string());
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        (status, serde_json::from_slice(&bytes).unwrap_or(Value::Null), location)
    }

    /// The provider says `sub` signed in, with this email, for the next token request.
    async fn provider_answers(&self, nonce: &str, sub: &str, email: &str, verified: bool) {
        let mut header = Header::new(Algorithm::RS256);
        header.kid = Some("k".into());
        let claims = json!({ "iss": self.issuer.uri(), "aud": CLIENT, "sub": sub, "exp": now() + 300, "nonce": nonce, "email": email, "email_verified": verified });
        let token = encode(&header, &claims, &EncodingKey::from_rsa_pem(ISSUER_KEY.as_bytes()).unwrap()).unwrap();
        self.issuer.reset().await;
        let base = self.issuer.uri();
        Mock::given(method("GET")).and(path("/.well-known/openid-configuration")).respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "issuer": base, "authorization_endpoint": format!("{base}/authorize"), "token_endpoint": format!("{base}/token"), "jwks_uri": format!("{base}/jwks")
        }))).mount(&self.issuer).await;
        Mock::given(method("GET")).and(path("/jwks")).respond_with(ResponseTemplate::new(200).set_body_json(json!({ "keys": [{ "kty": "RSA", "kid": "k", "alg": "RS256", "n": ISSUER_N.trim(), "e": "AQAB" }] }))).mount(&self.issuer).await;
        Mock::given(method("POST")).and(path("/token")).respond_with(ResponseTemplate::new(200).set_body_json(json!({ "id_token": token }))).mount(&self.issuer).await;
    }

    /// Starts a sign-in and answers its state and nonce from where Relay sent the browser.
    async fn authorize(&self, redirect_to: &str) -> (StatusCode, BTreeMap<String, String>, Value) {
        let uri = format!("/auth/authorize/oidc:acme?redirect_to={}&code_challenge={}&code_challenge_method=S256&apikey={}", urlencode(redirect_to), challenge(VERIFIER), self.anon());
        let request = Request::get(uri).header("host", self.host()).body(Body::empty()).unwrap();
        let response = self.router.clone().oneshot(request).await.unwrap();
        let status = response.status();
        let location = response.headers().get("location").map(|l| l.to_str().unwrap().to_string());
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let query = location.map(|l| reqwest::Url::parse(&l).unwrap().query_pairs().into_owned().collect()).unwrap_or_default();
        (status, query, serde_json::from_slice(&bytes).unwrap_or(Value::Null))
    }

    /// The provider sends the browser back; answers where Relay sends it next.
    async fn callback(&self, state: &str) -> (StatusCode, Option<String>) {
        let request = Request::get(format!("/auth/callback/oidc:acme?code=provider-code&state={state}")).header("host", self.host()).body(Body::empty()).unwrap();
        let response = self.router.clone().oneshot(request).await.unwrap();
        (response.status(), response.headers().get("location").map(|l| l.to_str().unwrap().to_string()))
    }

    async fn trade(&self, code: &str, verifier: &str) -> (StatusCode, Value) {
        let (status, body, _) = self.app("POST", "/auth/token?grant_type=pkce", Some(&self.anon()), Some(json!({ "code": code, "code_verifier": verifier }))).await;
        (status, body)
    }

    async fn sign_in_with_acme(&self, sub: &str, email: &str, verified: bool) -> Value {
        let (status, query, body) = self.authorize(&format!("{APP}/after")).await;
        assert_eq!(status, StatusCode::SEE_OTHER, "{body}");
        self.provider_answers(&query["nonce"], sub, email, verified).await;
        let (status, location) = self.callback(&query["state"]).await;
        assert_eq!(status, StatusCode::SEE_OTHER);
        let back = reqwest::Url::parse(&location.unwrap()).unwrap();
        let code: String = back.query_pairs().find(|(k, _)| k == "code").unwrap().1.into_owned();
        let (status, session) = self.trade(&code, VERIFIER).await;
        assert_eq!(status, StatusCode::OK, "{session}");
        session
    }
}

fn urlencode(text: &str) -> String {
    url::form_urlencoded::byte_serialize(text.as_bytes()).collect()
}

async fn send(router: &axum::Router, request: Request<Body>) -> (StatusCode, Value) {
    let response = router.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    (status, serde_json::from_slice(&bytes).unwrap_or(Value::Null))
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_sign_in_goes_to_the_provider_comes_back_with_a_code_and_trades_it_once(pool: PgPool) {
    let w = world(&pool).await;
    let (status, query, _) = w.authorize(&format!("{APP}/after")).await;
    assert_eq!(status, StatusCode::SEE_OTHER);
    assert_eq!(query["client_id"], CLIENT);
    assert_eq!(query["redirect_uri"], format!("https://{}/auth/callback/oidc:acme", w.host()));
    w.provider_answers(&query["nonce"], "acme-1", "Ana@Example.test", true).await;

    let (status, location) = w.callback(&query["state"]).await;
    assert_eq!(status, StatusCode::SEE_OTHER);
    let location = location.unwrap();
    assert!(location.starts_with(&format!("{APP}/after?code=")), "{location}");
    let code = location.rsplit('=').next().unwrap().to_string();
    assert_eq!(w.callback(&query["state"]).await.0, StatusCode::BAD_REQUEST, "a state works once");

    assert_eq!(w.trade(&code, "the-wrong-verifier-0000000000000000000000000").await.0, StatusCode::BAD_REQUEST);
    let (status, session) = w.trade(&code, VERIFIER).await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "a wrong verifier spends the code: {session}");

    let session = w.sign_in_with_acme("acme-1", "ana@example.test", true).await;
    assert_eq!(session["user"]["email"], "ana@example.test");
    assert_eq!(session["user"]["email_verified"], true);
    assert_eq!(session["user"]["identities"][0]["provider"], "oidc:acme");
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_redirect_to_another_site_is_refused_before_leaving(pool: PgPool) {
    let w = world(&pool).await;
    let (status, _, body) = w.authorize("https://evil.test/steal").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["code"], "REDIRECT_NOT_ALLOWED");
    assert_eq!(w.authorize("http://localhost:5173/after").await.0, StatusCode::SEE_OTHER, "dev allows localhost");
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_verified_email_joins_its_user_and_an_unverified_one_never_does(pool: PgPool) {
    let w = world(&pool).await;
    w.app("POST", "/auth/otp", None, Some(json!({ "email": "ana@example.test" }))).await;
    let code = w.outbox.sent().last().unwrap().1.subject.rsplit(' ').next().unwrap().to_string();
    let (_, by_code, _) = w.app("POST", "/auth/verify", None, Some(json!({ "email": "ana@example.test", "code": code }))).await;

    let joined = w.sign_in_with_acme("acme-1", "ana@example.test", true).await;
    assert_eq!(joined["user"]["id"], by_code["user"]["id"]);
    assert_eq!(joined["user"]["identities"].as_array().unwrap().len(), 2);

    let stranger = w.sign_in_with_acme("acme-2", "ana@example.test", false).await;
    assert_ne!(stranger["user"]["id"], by_code["user"]["id"]);
    assert!(stranger["user"]["email"].is_null());
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_signed_in_user_links_and_unlinks_but_keeps_one_way_in(pool: PgPool) {
    let w = world(&pool).await;
    let session = w.sign_in_with_acme("acme-1", "ana@example.test", true).await;
    let access = session["access_token"].as_str().unwrap().to_string();
    let only = session["user"]["identities"][0]["id"].as_str().unwrap().to_string();
    let (status, refused, _) = w.app("DELETE", &format!("/auth/identities/{only}"), Some(&access), None).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(refused["code"], "LAST_IDENTITY");

    let (status, started, _) = w.app("POST", "/auth/identities", Some(&access), Some(json!({ "provider": "oidc:acme", "redirect_to": format!("{APP}/settings"), "code_challenge": challenge(VERIFIER), "code_challenge_method": "S256" }))).await;
    assert_eq!(status, StatusCode::OK, "{started}");
    let url = reqwest::Url::parse(started["url"].as_str().unwrap()).unwrap();
    let query: BTreeMap<String, String> = url.query_pairs().into_owned().collect();
    w.provider_answers(&query["nonce"], "acme-work", "ana@work.test", true).await;
    let (status, location) = w.callback(&query["state"]).await;
    assert_eq!(status, StatusCode::SEE_OTHER);
    let code = location.unwrap().rsplit('=').next().unwrap().to_string();
    let (_, linked) = w.trade(&code, VERIFIER).await;
    assert_eq!(linked["user"]["id"], session["user"]["id"]);
    assert_eq!(linked["user"]["identities"].as_array().unwrap().len(), 2);

    let (status, _, _) = w.app("DELETE", &format!("/auth/identities/{only}"), Some(&access), None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
}

#[sqlx::test(migrations = "../../migrations")]
async fn relay_says_which_secrets_a_project_still_needs(pool: PgPool) {
    let w = world(&pool).await;
    let (_, shown) = w.control("GET", "/environments/dev/auth", None).await;
    assert_eq!(shown["data"]["missing_secrets"], json!([]));
    assert_eq!(w.control("DELETE", "/auth/secrets/NOMI_OAUTH_OIDC_ACME_SECRET", None).await.0, StatusCode::OK);
    let (_, shown) = w.control("GET", "/environments/dev/auth", None).await;
    assert_eq!(shown["data"]["missing_secrets"], json!(["NOMI_OAUTH_OIDC_ACME_SECRET"]));
    assert_eq!(w.authorize(&format!("{APP}/after")).await.2["code"], "PROVIDER_NOT_READY");
}
