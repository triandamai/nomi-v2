//! OAuth providers against stand-ins: wiremock plays an OpenID Connect issuer (discovery, JWKS,
//! token endpoint) and GitHub. The keys in `tests/keys/` exist only for these tests.
use std::collections::BTreeMap;

use jsonwebtoken::{encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use relay_auth::config::{AuthConfig, GithubConfig, OidcConfig, Secrets};
use relay_auth::oauth::{apple_client_secret, ClientSecret, GithubProvider, OidcProvider, Profile, Provider, Start};
use relay_auth::AuthError;
use serde_json::{json, Value};
use wiremock::matchers::{body_string_contains, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const ISSUER_KEY: &str = include_str!("keys/issuer_rsa.pem");
const ISSUER_N: &str = include_str!("keys/issuer_rsa.n");
const APPLE_P8: &str = include_str!("keys/apple.p8");
const APPLE_PUB: &str = include_str!("keys/apple_pub.pem");
const KID: &str = "test-key";
const CLIENT: &str = "bakery-web";
const NONCE: &str = "nonce-123";
const CALLBACK: &str = "https://bakery-dev.api.relay.test/auth/callback/oidc:acme";

fn now() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs()
}

fn id_token(issuer: &str, claims: Value) -> String {
    let mut header = Header::new(Algorithm::RS256);
    header.kid = Some(KID.into());
    let mut base = json!({ "iss": issuer, "aud": CLIENT, "sub": "acme-user-1", "exp": now() + 300, "iat": now(), "nonce": NONCE });
    base.as_object_mut().unwrap().extend(claims.as_object().unwrap().clone());
    encode(&header, &base, &EncodingKey::from_rsa_pem(ISSUER_KEY.as_bytes()).unwrap()).unwrap()
}

async fn issuer(token_claims: Value) -> MockServer {
    let server = MockServer::start().await;
    let base = server.uri();
    Mock::given(method("GET"))
        .and(path("/.well-known/openid-configuration"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "issuer": base, "authorization_endpoint": format!("{base}/authorize"), "token_endpoint": format!("{base}/token"), "jwks_uri": format!("{base}/jwks")
        })))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/jwks"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "keys": [{ "kty": "RSA", "kid": KID, "alg": "RS256", "use": "sig", "n": ISSUER_N.trim(), "e": "AQAB" }] })))
        .mount(&server)
        .await;
    let token = id_token(&base, token_claims);
    Mock::given(method("POST"))
        .and(path("/token"))
        .and(body_string_contains("grant_type=authorization_code"))
        .and(body_string_contains("code=provider-code"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "access_token": "at", "token_type": "Bearer", "id_token": token })))
        .mount(&server)
        .await;
    server
}

fn oidc(server: &MockServer) -> OidcProvider {
    OidcProvider { name: "oidc:acme".into(), issuer: server.uri(), client_id: CLIENT.into(), secret: ClientSecret::Plain("acme-secret".into()), scopes: "openid email profile".into(), form_post: false }
}

fn start() -> Start {
    Start { state: "state-abc".into(), nonce: NONCE.into(), redirect_uri: CALLBACK.into(), scopes: None }
}

#[tokio::test]
async fn an_oidc_sign_in_sends_people_to_the_issuer_and_reads_a_checked_id_token() {
    let server = issuer(json!({ "email": "Ana@Example.test", "email_verified": true })).await;
    let provider = Provider::Oidc(oidc(&server));
    let http = reqwest::Client::new();
    let url = reqwest::Url::parse(&provider.authorize_url(&http, &start()).await.unwrap()).unwrap();
    assert_eq!(url.path(), "/authorize");
    let query: BTreeMap<_, _> = url.query_pairs().into_owned().collect();
    assert_eq!(query["client_id"], CLIENT);
    assert_eq!(query["redirect_uri"], CALLBACK);
    assert_eq!(query["state"], "state-abc");
    assert_eq!(query["nonce"], NONCE);
    assert_eq!(query["response_type"], "code");
    assert_eq!(query["scope"], "openid email profile");

    let profile = provider.exchange(&http, "provider-code", CALLBACK, NONCE).await.unwrap();
    assert_eq!(profile, Profile { provider_user_id: "acme-user-1".into(), email: Some("ana@example.test".into()), email_verified: true });
}

#[tokio::test]
async fn an_id_token_for_someone_else_late_replayed_or_forged_is_refused() {
    let http = reqwest::Client::new();
    for claims in [json!({ "aud": "another-app" }), json!({ "exp": now() - 600 }), json!({ "nonce": "another-flow" })] {
        let server = issuer(claims.clone()).await;
        let refused = Provider::Oidc(oidc(&server)).exchange(&http, "provider-code", CALLBACK, NONCE).await;
        assert!(matches!(refused, Err(AuthError::Unauthorized { code: "BAD_ID_TOKEN", .. })), "{claims}: {refused:?}");
    }
    let server = issuer(json!({})).await;
    let forged = MockServer::start().await;
    // Same discovery, but the token endpoint hands back a token signed by a key the JWKS doesn't have.
    let base = forged.uri();
    Mock::given(method("GET")).and(path("/.well-known/openid-configuration")).respond_with(ResponseTemplate::new(200).set_body_json(json!({
        "issuer": base, "authorization_endpoint": format!("{base}/authorize"), "token_endpoint": format!("{base}/token"), "jwks_uri": format!("{}/jwks", server.uri())
    }))).mount(&forged).await;
    let mut header = Header::new(Algorithm::HS256);
    header.kid = Some(KID.into());
    let fake = encode(&header, &json!({ "iss": base, "aud": CLIENT, "sub": "x", "exp": now() + 300, "nonce": NONCE }), &EncodingKey::from_secret(b"guess")).unwrap();
    Mock::given(method("POST")).and(path("/token")).respond_with(ResponseTemplate::new(200).set_body_json(json!({ "id_token": fake }))).mount(&forged).await;
    let refused = Provider::Oidc(oidc(&forged)).exchange(&http, "provider-code", CALLBACK, NONCE).await;
    assert!(matches!(refused, Err(AuthError::Unauthorized { code: "BAD_ID_TOKEN", .. })), "{refused:?}");
}

#[tokio::test]
async fn an_unverified_email_is_reported_as_unverified() {
    let server = issuer(json!({ "email": "ana@example.test", "email_verified": "false" })).await;
    let profile = Provider::Oidc(oidc(&server)).exchange(&reqwest::Client::new(), "provider-code", CALLBACK, NONCE).await.unwrap();
    assert!(!profile.email_verified);
}

#[tokio::test]
async fn github_signs_in_with_the_primary_verified_email() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/login/oauth/access_token"))
        .and(header("accept", "application/json"))
        .and(body_string_contains("client_secret=gh-secret"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "access_token": "gho_token", "token_type": "bearer" })))
        .mount(&server)
        .await;
    Mock::given(method("GET")).and(path("/user")).and(header("authorization", "Bearer gho_token")).respond_with(ResponseTemplate::new(200).set_body_json(json!({ "id": 4242, "login": "ana" }))).mount(&server).await;
    Mock::given(method("GET"))
        .and(path("/user/emails"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            { "email": "old@example.test", "primary": false, "verified": true },
            { "email": "Ana@Example.test", "primary": true, "verified": true }
        ])))
        .mount(&server)
        .await;
    let provider = Provider::Github(GithubProvider { client_id: "gh-client".into(), client_secret: "gh-secret".into(), api_base: server.uri(), web_base: server.uri() });
    let http = reqwest::Client::new();
    let url = provider.authorize_url(&http, &start()).await.unwrap();
    assert!(url.starts_with(&format!("{}/login/oauth/authorize?", server.uri())) && url.contains("client_id=gh-client") && url.contains("scope=read%3Auser+user%3Aemail"), "{url}");
    let profile = provider.exchange(&http, "provider-code", CALLBACK, NONCE).await.unwrap();
    assert_eq!(profile, Profile { provider_user_id: "4242".into(), email: Some("ana@example.test".into()), email_verified: true });
}

#[test]
fn apples_client_secret_is_signed_from_the_p8_for_five_minutes() {
    let secret = apple_client_secret("TEAM123", "KEY456", "com.bakery.web", APPLE_P8, 1_000).unwrap();
    let header = jsonwebtoken::decode_header(&secret).unwrap();
    assert_eq!((header.alg, header.kid.as_deref()), (Algorithm::ES256, Some("KEY456")));
    let mut validation = Validation::new(Algorithm::ES256);
    validation.set_audience(&["https://appleid.apple.com"]);
    validation.validate_exp = false;
    let claims = jsonwebtoken::decode::<Value>(&secret, &DecodingKey::from_ec_pem(APPLE_PUB.as_bytes()).unwrap(), &validation).unwrap().claims;
    assert_eq!((claims["iss"].as_str(), claims["sub"].as_str()), (Some("TEAM123"), Some("com.bakery.web")));
    assert_eq!(claims["exp"].as_u64().unwrap() - claims["iat"].as_u64().unwrap(), 300);
}

#[tokio::test]
async fn apple_comes_back_by_form_post_and_gets_its_signed_secret() {
    let server = issuer(json!({ "email": "ana@privaterelay.appleid.com", "email_verified": "true" })).await;
    let apple = OidcProvider { name: "apple".into(), secret: ClientSecret::Apple { team_id: "TEAM123".into(), key_id: "KEY456".into(), p8: APPLE_P8.into() }, form_post: true, scopes: "name email".into(), ..oidc(&server) };
    let provider = Provider::Oidc(apple);
    let http = reqwest::Client::new();
    let url = provider.authorize_url(&http, &start()).await.unwrap();
    assert!(url.contains("response_mode=form_post"), "{url}");
    assert!(provider.exchange(&http, "provider-code", CALLBACK, NONCE).await.unwrap().email_verified);
    let sent = server.received_requests().await.unwrap().into_iter().find(|r| r.url.path() == "/token").unwrap();
    let form: BTreeMap<String, String> = url::form_urlencoded::parse(&sent.body).into_owned().collect();
    assert_eq!(jsonwebtoken::decode_header(&form["client_secret"]).unwrap().alg, Algorithm::ES256);
}

#[test]
fn a_provider_must_be_switched_on_and_have_its_secret() {
    let config = AuthConfig {
        methods: vec!["github".into(), "oidc:acme".into()],
        github: Some(GithubConfig { client_id: "gh-client".into(), api_base: None, web_base: None }),
        oidc: vec![OidcConfig { id: "acme".into(), issuer: "https://login.acme.test".into(), client_id: CLIENT.into() }],
        ..AuthConfig::default()
    };
    let mut secrets = Secrets::default();
    assert!(matches!(Provider::resolve(&config, &secrets, "google"), Err(AuthError::BadRequest { code: "METHOD_OFF", .. })));
    assert!(matches!(Provider::resolve(&config, &secrets, "github"), Err(AuthError::BadRequest { code: "PROVIDER_NOT_READY", .. })));
    secrets.0.insert("NOMI_OAUTH_GITHUB_SECRET".into(), "gh-secret".into());
    secrets.0.insert("NOMI_OAUTH_OIDC_ACME_SECRET".into(), "acme-secret".into());
    assert!(matches!(Provider::resolve(&config, &secrets, "github"), Ok(Provider::Github(_))));
    assert!(matches!(Provider::resolve(&config, &secrets, "oidc:acme"), Ok(Provider::Oidc(_))));
    assert_eq!(config.missing_secrets(&Secrets::default()), vec!["NOMI_OAUTH_GITHUB_SECRET".to_string(), "NOMI_OAUTH_OIDC_ACME_SECRET".to_string()]);
}

#[test]
fn a_backend_json_auth_block_can_leave_out_what_has_a_default() {
    let config: AuthConfig = serde_json::from_value(json!({ "github": { "client_id": "gh" } })).unwrap();
    assert_eq!(config.methods, vec!["email_code".to_string()]);
    assert_eq!(config.session_ttl_days, relay_auth::config::DEFAULT_SESSION_DAYS);
    assert_eq!(config.github.unwrap().client_id, "gh");
}
