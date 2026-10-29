//! Relay's settings, set from Nomi's admin: checked before they're kept, sealed in Relay's state,
//! and written onto PostgREST's role in each shared database.
use relay_control::settings::{save, view, SettingsInput};
use relay_control::{Relay, RelayError};
use relay_core::settings::StorageSettings;
use relay_core::Env;
use sqlx::PgPool;
use wiremock::matchers::method;
use wiremock::{Mock, MockServer, ResponseTemplate};

const AUTHENTICATOR: &str = "relay_test_authenticator";
const KEY: [u8; 32] = [9u8; 32];

async fn ensure_authenticator(pool: &PgPool) {
    if let Err(e) = sqlx::query(&format!("CREATE ROLE {AUTHENTICATOR} NOLOGIN NOINHERIT")).execute(pool).await {
        let code = e.as_database_error().and_then(|d| d.code()).map(|c| c.to_string());
        assert!(matches!(code.as_deref(), Some("42710") | Some("23505")), "{e}");
    }
}

/// The test database's own URL, which stands in for both shared databases.
async fn own_url(pool: &PgPool) -> String {
    let base = std::env::var("DATABASE_URL").expect("DATABASE_URL");
    let database: String = sqlx::query_scalar("SELECT current_database()").fetch_one(pool).await.unwrap();
    let (server, _) = base.rsplit_once('/').unwrap();
    format!("{server}/{database}")
}

/// The same database URL with a password that doesn't work, whatever the login is.
fn with_wrong_password(url: &str) -> String {
    let (scheme, rest) = url.split_once("://").unwrap();
    let (login, host) = rest.split_once('@').expect("DATABASE_URL has a login");
    let user = login.split_once(':').map_or(login, |(user, _)| user);
    format!("{scheme}://{user}:wrong-password@{host}")
}

async fn postgrest() -> MockServer {
    let server = MockServer::start().await;
    Mock::given(method("GET")).respond_with(ResponseTemplate::new(200)).mount(&server).await;
    server
}

fn input(url: &str, postgrest: &str) -> SettingsInput {
    SettingsInput {
        public_domain: "api.relay.test".into(),
        shared_dev_url: Some(url.into()),
        shared_production_url: Some(url.into()),
        postgrest_dev_url: postgrest.into(),
        postgrest_production_url: postgrest.into(),
        postgrest_role: Some(AUTHENTICATOR.into()),
        preview_origins: vec![".preview.nomi.test".into()],
        shared_smtp: None,
        storage: None,
        functions: None,
    }
}

/// An S3 stand-in answering every request with `status` (an empty listing when it's 200).
async fn s3(status: u16) -> MockServer {
    let server = MockServer::start().await;
    let body = "<?xml version=\"1.0\" encoding=\"UTF-8\"?><ListBucketResult><Name>files</Name><Prefix></Prefix><KeyCount>0</KeyCount><MaxKeys>1</MaxKeys><IsTruncated>false</IsTruncated></ListBucketResult>";
    Mock::given(method("GET")).respond_with(ResponseTemplate::new(status).set_body_string(body).insert_header("content-type", "application/xml")).mount(&server).await;
    server
}

fn file_storage(endpoint: &str, secret: &str) -> StorageSettings {
    StorageSettings {
        endpoint: endpoint.into(),
        region: "us-east-1".into(),
        bucket: "files".into(),
        access_key_id: "AKIATEST".into(),
        secret_access_key: secret.into(),
        path_style: true,
    }
}

async fn role_settings(pool: &PgPool) -> Vec<String> {
    sqlx::query_scalar::<_, Vec<String>>(
        "SELECT s.setconfig FROM pg_db_role_setting s JOIN pg_roles r ON r.oid = s.setrole \
         WHERE r.rolname = $1 AND s.setdatabase = (SELECT oid FROM pg_database WHERE datname = current_database())",
    )
    .bind(AUTHENTICATOR)
    .fetch_optional(pool)
    .await
    .unwrap()
    .unwrap_or_default()
}

#[sqlx::test(migrations = "../../migrations")]
async fn before_its_settings_are_saved_relay_says_so(pool: PgPool) {
    let relay = Relay::unconfigured(pool.clone(), KEY);
    assert!(matches!(relay.live(), Err(RelayError::Rejected { code: "NOT_CONFIGURED", .. })));
    assert!(!view(&relay).await.configured);
}

#[sqlx::test(migrations = "../../migrations")]
async fn saved_settings_serve_now_and_after_a_restart_without_showing_passwords(pool: PgPool) {
    ensure_authenticator(&pool).await;
    let url = own_url(&pool).await;
    let pgrst = postgrest().await;
    let relay = Relay::unconfigured(pool.clone(), KEY);

    let saved = save(&relay, input(&url, &pgrst.uri())).await.unwrap();
    assert!(saved.configured);
    assert!(saved.checks.iter().all(|c| c.ok), "{:?}", saved.checks);
    assert!(!serde_json::to_string(&saved).unwrap().contains("nomi:nomi@"), "a password was answered");
    assert_eq!(relay.live().unwrap().settings.public_domain, "api.relay.test");

    let restarted = Relay::start(pool.clone(), KEY).await.unwrap();
    let live = restarted.live().unwrap();
    assert_eq!(live.settings.postgrest_url(Env::Dev), pgrst.uri());
    assert_eq!(live.settings.postgrest_jwt_secret, relay.live().unwrap().settings.postgrest_jwt_secret);
    assert!(live.settings.postgrest_jwt_secret.len() >= 32);
    let sealed: Vec<u8> = sqlx::query_scalar("SELECT sealed FROM relay.settings").fetch_one(&pool).await.unwrap();
    assert!(!String::from_utf8_lossy(&sealed).contains("postgres://"), "settings must be sealed");
}

#[sqlx::test(migrations = "../../migrations")]
async fn saving_again_without_database_urls_keeps_them_and_the_secret(pool: PgPool) {
    ensure_authenticator(&pool).await;
    let url = own_url(&pool).await;
    let pgrst = postgrest().await;
    let relay = Relay::unconfigured(pool.clone(), KEY);
    save(&relay, input(&url, &pgrst.uri())).await.unwrap();
    let secret = relay.live().unwrap().settings.postgrest_jwt_secret.clone();

    let mut again = input(&url, &pgrst.uri());
    again.shared_dev_url = None;
    again.shared_production_url = Some(String::new());
    again.public_domain = "relay.example".into();
    save(&relay, again).await.unwrap();
    let live = relay.live().unwrap();
    assert_eq!(live.settings.shared_url(Env::Dev), url);
    assert_eq!(live.settings.shared_url(Env::Production), url);
    assert_eq!(live.settings.public_domain, "relay.example");
    assert_eq!(live.settings.postgrest_jwt_secret, secret);
}

#[sqlx::test(migrations = "../../migrations")]
async fn settings_that_dont_work_are_refused_and_the_old_ones_keep_serving(pool: PgPool) {
    ensure_authenticator(&pool).await;
    let url = own_url(&pool).await;
    let pgrst = postgrest().await;
    let relay = Relay::unconfigured(pool.clone(), KEY);
    save(&relay, input(&url, &pgrst.uri())).await.unwrap();

    let mut broken = input(&with_wrong_password(&url), &pgrst.uri());
    broken.postgrest_production_url = "http://127.0.0.1:9".into();
    broken.public_domain = "changed.example".into();
    let Err(RelayError::Rejected { code, details, .. }) = save(&relay, broken).await else { panic!("expected a refusal") };
    assert_eq!(code, "SETTINGS_INVALID");
    let failed: Vec<&str> = details["checks"].as_array().unwrap().iter().filter(|c| c["ok"] == false).map(|c| c["name"].as_str().unwrap()).collect();
    assert!(failed.contains(&"dev database") && failed.contains(&"production PostgREST"), "{failed:?}");
    assert_eq!(relay.live().unwrap().settings.public_domain, "api.relay.test");

    let mut no_role = input(&url, &pgrst.uri());
    no_role.postgrest_role = Some("nobody_by_this_name".into());
    assert!(matches!(save(&relay, no_role).await, Err(RelayError::Rejected { code: "SETTINGS_INVALID", .. })));
}

#[sqlx::test(migrations = "../../migrations")]
async fn postgrest_gets_its_secret_and_claim_key_from_the_database(pool: PgPool) {
    ensure_authenticator(&pool).await;
    let url = own_url(&pool).await;
    let pgrst = postgrest().await;
    let relay = Relay::unconfigured(pool.clone(), KEY);
    save(&relay, input(&url, &pgrst.uri())).await.unwrap();
    let secret = relay.live().unwrap().settings.postgrest_jwt_secret.clone();
    let settings = role_settings(&pool).await;
    assert!(settings.contains(&format!("pgrst.jwt_secret={secret}")), "{settings:?}");
    assert!(settings.contains(&"pgrst.jwt_role_claim_key=.db_role".to_string()), "{settings:?}");
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_shared_sender_is_kept_masked_and_its_password_stays_when_left_out(pool: PgPool) {
    use relay_core::settings::Smtp;
    ensure_authenticator(&pool).await;
    let url = own_url(&pool).await;
    let pgrst = postgrest().await;
    let relay = Relay::unconfigured(pool.clone(), KEY).with_outbox(std::sync::Arc::new(relay_auth::mail::RecordingOutbox::default()));
    let sender = Smtp { host: "smtp.nomi.test".into(), port: 587, user: "codes".into(), password: "smtp-secret".into(), from: "Nomi <codes@nomi.test>".into(), security: "starttls".into() };
    let mut first = input(&url, &pgrst.uri());
    first.shared_smtp = Some(sender.clone());
    let saved = save(&relay, first).await.unwrap();
    assert!(saved.checks.iter().any(|c| c.name == "shared sender" && c.ok), "{:?}", saved.checks);
    let shown = serde_json::to_string(&saved).unwrap();
    assert!(shown.contains("smtp.nomi.test") && !shown.contains("smtp-secret"), "{shown}");

    let mut again = input(&url, &pgrst.uri());
    again.shared_smtp = Some(Smtp { password: String::new(), from: "Nomi codes <codes@nomi.test>".into(), ..sender });
    save(&relay, again).await.unwrap();
    let kept = relay.live().unwrap().settings.shared_smtp.clone().unwrap();
    assert_eq!(kept.password, "smtp-secret");
    assert_eq!(kept.from, "Nomi codes <codes@nomi.test>");

    save(&relay, input(&url, &pgrst.uri())).await.unwrap();
    assert!(relay.live().unwrap().settings.shared_smtp.is_some(), "leaving the sender out keeps it");
}

#[sqlx::test(migrations = "../../migrations")]
async fn file_storage_is_checked_and_kept_without_showing_its_secret(pool: PgPool) {
    ensure_authenticator(&pool).await;
    let url = own_url(&pool).await;
    let pgrst = postgrest().await;
    let bucket = s3(200).await;
    let relay = Relay::unconfigured(pool.clone(), KEY);
    assert!(relay.store().is_none(), "no storage before it's set");

    let saved = save(&relay, SettingsInput { storage: Some(file_storage(&bucket.uri(), "s3cret")), ..input(&url, &pgrst.uri()) }).await.unwrap();

    let shown = saved.storage.as_ref().unwrap();
    assert_eq!(shown.bucket, "files");
    assert_eq!(shown.secret_access_key, "", "the secret is never shown");
    assert!(saved.checks.iter().any(|c| c.name == "file storage" && c.ok), "{:?}", saved.checks);
    assert!(relay.store().is_some());

    // Saving again with the secret left empty keeps it.
    save(&relay, SettingsInput { storage: Some(file_storage(&bucket.uri(), "")), ..input(&url, &pgrst.uri()) }).await.unwrap();
    let kept = relay_control::settings::load(&relay).await.unwrap().unwrap();
    assert_eq!(kept.storage.unwrap().secret_access_key, "s3cret");

    // Left out keeps it; an empty bucket removes it.
    save(&relay, input(&url, &pgrst.uri())).await.unwrap();
    assert!(relay.store().is_some());
    save(&relay, SettingsInput { storage: Some(StorageSettings { bucket: String::new(), ..file_storage(&bucket.uri(), "") }), ..input(&url, &pgrst.uri()) }).await.unwrap();
    assert!(relay.store().is_none());
}

#[sqlx::test(migrations = "../../migrations")]
async fn file_storage_that_refuses_is_named_and_nothing_is_saved(pool: PgPool) {
    ensure_authenticator(&pool).await;
    let url = own_url(&pool).await;
    let pgrst = postgrest().await;
    let bucket = s3(403).await;
    let relay = Relay::unconfigured(pool.clone(), KEY);

    let refused = save(&relay, SettingsInput { storage: Some(file_storage(&bucket.uri(), "wrong")), ..input(&url, &pgrst.uri()) }).await.unwrap_err();

    let RelayError::Rejected { message, .. } = refused else { panic!("not a refusal") };
    assert!(message.contains("file storage"), "{message}");
    assert!(relay.live().is_err());
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_functions_runtime_is_checked_with_its_secret_and_the_secret_never_shown(pool: PgPool) {
    use relay_core::settings::FunctionsRuntime;
    use wiremock::matchers::{header, path};
    ensure_authenticator(&pool).await;
    let url = own_url(&pool).await;
    let pgrst = postgrest().await;
    let runtime = MockServer::start().await;
    Mock::given(method("GET")).and(path("/health")).and(header("x-relay-runtime", "right-secret")).respond_with(ResponseTemplate::new(200)).mount(&runtime).await;
    Mock::given(method("GET")).and(path("/health")).respond_with(ResponseTemplate::new(401)).mount(&runtime).await;
    let relay = Relay::unconfigured(pool.clone(), KEY);

    let wrong = save(&relay, SettingsInput { functions: Some(FunctionsRuntime { url: runtime.uri(), secret: "wrong".into() }), ..input(&url, &pgrst.uri()) }).await.unwrap_err();
    let RelayError::Rejected { message, .. } = wrong else { panic!("not a refusal") };
    assert!(message.contains("functions runtime"), "{message}");

    let saved = save(&relay, SettingsInput { functions: Some(FunctionsRuntime { url: format!("{}/", runtime.uri()), secret: "right-secret".into() }), ..input(&url, &pgrst.uri()) }).await.unwrap();
    assert_eq!(saved.functions.as_ref().unwrap().secret, "");
    assert_eq!(saved.functions.as_ref().unwrap().url, runtime.uri());
    save(&relay, SettingsInput { functions: Some(FunctionsRuntime { url: runtime.uri(), secret: String::new() }), ..input(&url, &pgrst.uri()) }).await.unwrap();
    assert_eq!(relay_control::settings::load(&relay).await.unwrap().unwrap().functions.unwrap().secret, "right-secret");
}
