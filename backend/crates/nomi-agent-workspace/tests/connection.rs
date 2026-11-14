//! The Google connection outside the tools: configuration, the OAuth flow's failures, status,
//! token refresh and disconnecting.

use chrono::{Duration, Utc};
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;
use wiremock::matchers::{body_string_contains, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use nomi_agent_workspace::connection::{self, AccessError, GoogleConfig};

const KEY: [u8; 32] = [3u8; 32];

#[test]
fn config_comes_from_all_three_fields_and_derives_the_sign_in_callback() {
    let fields = |pairs: &'static [(&'static str, &'static str)]| move |f: &str| pairs.iter().find(|(k, _)| *k == f).map(|(_, v)| v.to_string());
    assert!(GoogleConfig::from_lookup(fields(&[("client_id", "id"), ("client_secret", "s")])).is_none());
    assert!(GoogleConfig::from_lookup(fields(&[("client_id", " "), ("client_secret", "s"), ("redirect_uri", "https://nomi.app/cb")])).is_none());
    let config = GoogleConfig::from_lookup(fields(&[("client_id", " id "), ("client_secret", "s"), ("redirect_uri", "https://nomi.app/connections/google/callback")])).unwrap();
    assert_eq!(config.client_id, "id");
    assert_eq!(config.signin_redirect_uri, "https://nomi.app/auth/google/callback");
    assert_eq!(config.token_url, "https://oauth2.googleapis.com/token");
    let custom = GoogleConfig::from_lookup(fields(&[("client_id", "id"), ("client_secret", "s"), ("redirect_uri", "https://nomi.app/cb"), ("signin_redirect_uri", "https://login.nomi.app/cb")])).unwrap();
    assert_eq!(custom.signin_redirect_uri, "https://login.nomi.app/cb");
    assert_eq!(GoogleConfig::new("a".into(), "b".into(), "not a url".into()).signin_redirect_uri, "");
}

#[test]
fn callbacks_go_to_the_app_the_flow_started_on() {
    use nomi_agent_workspace::connection::{Callback, OriginNotAllowed};
    let fields = |pairs: &'static [(&'static str, &'static str)]| move |f: &str| pairs.iter().find(|(k, _)| *k == f).map(|(_, v)| v.to_string());
    let config = GoogleConfig::from_lookup(fields(&[
        ("client_id", "id"),
        ("client_secret", "s"),
        ("redirect_uri", "https://nomi.app/connections/google/callback"),
        ("app_origins", "https://new.nomi.app/path, http://localhost:5174 not-a-url"),
    ]))
    .unwrap();
    assert_eq!(config.app_origins, ["https://new.nomi.app", "http://localhost:5174"]);
    assert_eq!(config.callback_for(Callback::SignIn, None).unwrap(), "https://nomi.app/auth/google/callback");
    assert_eq!(config.callback_for(Callback::Workspace, Some(" ")).unwrap(), "https://nomi.app/connections/google/callback");
    assert_eq!(config.callback_for(Callback::SignIn, Some("https://nomi.app")).unwrap(), "https://nomi.app/auth/google/callback");
    assert_eq!(config.callback_for(Callback::SignIn, Some("https://new.nomi.app/")).unwrap(), "https://new.nomi.app/auth/google/callback");
    assert_eq!(config.callback_for(Callback::Workspace, Some("http://localhost:5174")).unwrap(), "http://localhost:5174/connections/google/callback");
    assert_eq!(config.callback_for(Callback::SignIn, Some("https://evil.app")), Err(OriginNotAllowed("https://evil.app".into())));
    assert!(config.callback_for(Callback::SignIn, Some("nomi.app")).is_err());
}

#[test]
fn only_known_services_have_scopes() {
    for service in connection::SERVICES {
        assert!(connection::service_scope(service).unwrap().starts_with("https://www.googleapis.com/auth/"));
    }
    assert_eq!(connection::service_scope("photos"), None);
}

async fn seed_user(pool: &PgPool) -> Uuid {
    sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(pool).await.unwrap()
}

async fn state_of(url: &str) -> String {
    reqwest::Url::parse(url).unwrap().query_pairs().find(|(k, _)| k == "state").unwrap().1.to_string()
}

#[sqlx::test(migrations = "../../migrations")]
async fn connecting_needs_a_service_and_a_code_google_accepts(pool: PgPool) {
    let server = MockServer::start().await;
    let config = GoogleConfig::all_at(&server.uri());
    let user = seed_user(&pool).await;
    let mut conn = pool.acquire().await.unwrap();
    assert_eq!(connection::start_authorization(&mut conn, &config, user, &["photos".into()], None, None).await.unwrap_err(), "pick at least one service");

    Mock::given(method("POST")).and(path("/token")).respond_with(ResponseTemplate::new(400).set_body_string("invalid_grant")).mount(&server).await;
    let url = connection::start_authorization(&mut conn, &config, user, &["gmail".into()], None, None).await.unwrap();
    let error = connection::complete_authorization(&mut conn, &reqwest::Client::new(), &config, &KEY, user, "bad", &state_of(&url).await).await.unwrap_err();
    assert!(error.contains("didn't accept"), "{error}");
    // The state is used up either way.
    let again = connection::complete_authorization(&mut conn, &reqwest::Client::new(), &config, &KEY, user, "bad", &state_of(&url).await).await.unwrap_err();
    assert!(again.contains("expired"));
}

#[sqlx::test(migrations = "../../migrations")]
async fn status_refresh_and_disconnect(pool: PgPool) {
    let server = MockServer::start().await;
    let config = GoogleConfig::all_at(&server.uri());
    let user = seed_user(&pool).await;
    let org: Uuid = sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Acme') RETURNING id").fetch_one(&pool).await.unwrap();
    let session: Uuid = sqlx::query_scalar("INSERT INTO sessions (org_id, channel, chat_id) VALUES ($1, 'telegram', 'x') RETURNING id").bind(org).fetch_one(&pool).await.unwrap();
    let mut conn = pool.acquire().await.unwrap();
    assert_eq!(connection::status(&mut conn, user).await.unwrap(), None);

    Mock::given(method("POST"))
        .and(path("/token"))
        .and(body_string_contains("grant_type=authorization_code"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"access_token": "a1", "expires_in": 3600, "refresh_token": "r1", "scope": "openid email https://www.googleapis.com/auth/gmail.modify"})))
        .mount(&server)
        .await;
    Mock::given(method("GET")).and(path("/v1/userinfo")).respond_with(ResponseTemplate::new(200).set_body_json(json!({"email": "ana@example.com"}))).mount(&server).await;
    let url = connection::start_authorization(&mut conn, &config, user, &["gmail".into(), "sheets".into()], Some(session), None).await.unwrap();
    let connected = connection::complete_authorization(&mut conn, &reqwest::Client::new(), &config, &KEY, user, "c", &state_of(&url).await).await.unwrap();
    assert_eq!(connected.services, vec!["gmail".to_string()], "sheets was unticked on Google's screen");
    assert_eq!(connected.resume_session_id, Some(session));
    let shown = connection::status(&mut conn, user).await.unwrap().unwrap();
    assert_eq!(shown.email, "ana@example.com");

    // A live token is used as it is.
    let http = reqwest::Client::new();
    assert_eq!(connection::access_token(&mut conn, &http, &config, &KEY, user, "gmail").await.unwrap(), "a1");
    assert_eq!(connection::access_token(&mut conn, &http, &config, &KEY, user, "sheets").await.unwrap_err(), AccessError::ServiceNotAllowed("sheets".into()));
    assert_eq!(connection::access_token(&mut conn, &http, &config, &KEY, seed_user(&pool).await, "gmail").await.unwrap_err(), AccessError::NotConnected);

    // About to expire: refreshed; a refresh Google fails is an error, not a disconnect.
    sqlx::query("UPDATE workspace_connections SET expires_at = $2 WHERE user_id = $1").bind(user).bind(Utc::now() + Duration::seconds(10)).execute(&mut *conn).await.unwrap();
    let refresh = Mock::given(method("POST"))
        .and(path("/token"))
        .and(body_string_contains("grant_type=refresh_token"))
        .respond_with(ResponseTemplate::new(503))
        .mount_as_scoped(&server)
        .await;
    assert!(matches!(connection::access_token(&mut conn, &http, &config, &KEY, user, "gmail").await, Err(AccessError::Other(_))));
    drop(refresh);
    assert!(connection::status(&mut conn, user).await.unwrap().is_some());

    // A connection with a token this key can't open needs connecting again.
    assert_eq!(connection::access_token(&mut conn, &http, &config, &[0u8; 32], user, "gmail").await.unwrap_err(), AccessError::NotConnected);

    // Disconnecting revokes with Google and forgets the account; twice is a no-op.
    Mock::given(method("POST")).and(path("/revoke")).and(body_string_contains("token=r1")).respond_with(ResponseTemplate::new(200)).expect(1).mount(&server).await;
    assert!(connection::disconnect(&mut conn, &http, Some(&config), &KEY, user).await.unwrap());
    assert!(!connection::disconnect(&mut conn, &http, Some(&config), &KEY, user).await.unwrap());
    assert_eq!(connection::status(&mut conn, user).await.unwrap(), None);
}

#[sqlx::test(migrations = "../../migrations")]
async fn disconnecting_without_google_set_up_still_forgets_the_account(pool: PgPool) {
    let user = seed_user(&pool).await;
    let mut conn = pool.acquire().await.unwrap();
    sqlx::query("INSERT INTO workspace_connections (user_id, google_email, services, access_token_encrypted, expires_at) VALUES ($1, 'a@x', '{gmail}', $2, now())")
        .bind(user)
        .bind(nomi_settings::crypto::encrypt(&KEY, "a"))
        .execute(&mut *conn)
        .await
        .unwrap();
    assert!(connection::disconnect(&mut conn, &reqwest::Client::new(), None, &KEY, user).await.unwrap());
    // No refresh token: an expired connection can only be connected again.
    sqlx::query("INSERT INTO workspace_connections (user_id, google_email, services, access_token_encrypted, expires_at) VALUES ($1, 'a@x', '{gmail}', $2, now())")
        .bind(user)
        .bind(nomi_settings::crypto::encrypt(&KEY, "a"))
        .execute(&mut *conn)
        .await
        .unwrap();
    let config = GoogleConfig::all_at("http://127.0.0.1:9");
    assert_eq!(connection::access_token(&mut conn, &reqwest::Client::new(), &config, &KEY, user, "gmail").await.unwrap_err(), AccessError::NotConnected);
}
