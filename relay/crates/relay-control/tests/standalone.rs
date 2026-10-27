//! A standalone Relay: one project, in its own database, set up from its environment, as Nomi
//! runs one next to each project on Shipyard.
use relay_control::backend::ensure_backend;
use relay_control::migrations::apply;
use relay_control::settings::{save, SettingsInput};
use relay_control::{Relay, RelayError};
use relay_core::config::Standalone;
use relay_core::settings::AUTHENTICATOR;
use relay_core::{Env, Tier};
use relay_migrate::MigrationFile;
use sqlx::PgPool;
use uuid::Uuid;
use wiremock::matchers::method;
use wiremock::{Mock, MockServer, ResponseTemplate};

const KEY: [u8; 32] = [4u8; 32];
const PUBLIC_DOMAIN: &str = "app-backend.apps.test";

/// The test database's own URL: a standalone Relay's state and its project share it.
async fn own_url(pool: &PgPool) -> String {
    let base = std::env::var("DATABASE_URL").expect("DATABASE_URL");
    let database: String = sqlx::query_scalar("SELECT current_database()").fetch_one(pool).await.unwrap();
    let (server, _) = base.rsplit_once('/').unwrap();
    format!("{server}/{database}")
}

async fn postgrest() -> MockServer {
    let server = MockServer::start().await;
    Mock::given(method("GET")).respond_with(ResponseTemplate::new(200)).mount(&server).await;
    server
}

async fn standalone(pool: &PgPool, postgrest_url: &str) -> (Relay, Uuid) {
    let project = Uuid::new_v4();
    let config = Standalone { project, env: Env::Production, postgrest_url: postgrest_url.to_string(), pgrst_password: "pgrst-pw".into(), functions: None };
    let relay = Relay::start_standalone(pool.clone(), KEY, own_url(pool).await, config).await.unwrap();
    (relay, project)
}

/// What Nomi sends after deploying it: its address and preview origins, nothing about databases.
fn from_nomi() -> SettingsInput {
    SettingsInput {
        public_domain: PUBLIC_DOMAIN.into(),
        shared_dev_url: None,
        shared_production_url: None,
        postgrest_dev_url: String::new(),
        postgrest_production_url: String::new(),
        postgrest_role: None,
        preview_origins: vec![".webcontainer-api.io".into()],
        shared_smtp: None,
        storage: None,
        functions: None,
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_standalone_relay_creates_the_authenticator_and_points_postgrest(pool: PgPool) {
    let postgrest = postgrest().await;
    let (relay, _) = standalone(&pool, &postgrest.uri()).await;

    let can_login: bool = sqlx::query_scalar("SELECT rolcanlogin FROM pg_roles WHERE rolname = $1").bind(AUTHENTICATOR).fetch_one(&pool).await.unwrap();
    assert!(can_login);
    let live = relay.live().expect("serves with nothing saved");
    assert_eq!(live.settings.postgrest_role(), Some(AUTHENTICATOR));
    let database: String = sqlx::query_scalar("SELECT current_database()").fetch_one(&pool).await.unwrap();
    let configured: Vec<String> = sqlx::query_scalar(
        "SELECT unnest(s.setconfig) FROM pg_db_role_setting s JOIN pg_roles r ON r.oid = s.setrole JOIN pg_database d ON d.oid = s.setdatabase WHERE r.rolname = $1 AND d.datname = $2",
    )
    .bind(AUTHENTICATOR)
    .bind(&database)
    .fetch_all(&pool)
    .await
    .unwrap();
    assert!(configured.contains(&format!("pgrst.jwt_secret={}", live.settings.postgrest_jwt_secret)), "{configured:?}");
}

#[sqlx::test(migrations = "../../migrations")]
async fn saving_settings_keeps_the_environment_values(pool: PgPool) {
    let postgrest = postgrest().await;
    let (relay, _) = standalone(&pool, &postgrest.uri()).await;
    let own = own_url(&pool).await;

    let saved = save(&relay, from_nomi()).await.unwrap();

    assert!(saved.configured);
    let live = relay.live().unwrap();
    assert_eq!(live.settings.public_domain, PUBLIC_DOMAIN);
    assert_eq!(live.settings.shared_url(Env::Production), own);
    assert_eq!(live.settings.shared_url(Env::Dev), own);
    assert_eq!(live.settings.postgrest_url(Env::Production), postgrest.uri());
    assert_eq!(live.settings.preview_origins, vec![".webcontainer-api.io".to_string()]);
    let jwt = live.settings.postgrest_jwt_secret.clone();

    // A restart reads the saved settings back with the environment over them, and the same secret.
    let restarted = Relay::start_standalone(pool.clone(), KEY, own.clone(), relay.standalone().unwrap().config.clone()).await.unwrap();
    let again = restarted.live().unwrap();
    assert_eq!(again.settings.public_domain, PUBLIC_DOMAIN);
    assert_eq!(again.settings.postgrest_jwt_secret, jwt);
}

#[sqlx::test(migrations = "../../migrations")]
async fn it_serves_only_its_own_project(pool: PgPool) {
    let postgrest = postgrest().await;
    let (relay, project) = standalone(&pool, &postgrest.uri()).await;
    save(&relay, from_nomi()).await.unwrap();

    let keys = ensure_backend(&relay, project, "Barbershop", Tier::Free, None).await.unwrap();
    assert_eq!(keys.tier, "dedicated", "standalone holds its project in its own database");
    assert_eq!(keys.environments["production"].url, format!("https://{PUBLIC_DOMAIN}"));

    let other = ensure_backend(&relay, Uuid::new_v4(), "Someone else", Tier::Free, None).await;
    assert!(matches!(other, Err(RelayError::Rejected { code: "CONFLICT", .. })), "{other:?}");
    let projects: i64 = sqlx::query_scalar("SELECT count(*) FROM relay.projects").fetch_one(&pool).await.unwrap();
    assert_eq!(projects, 1);
}

#[sqlx::test(migrations = "../../migrations")]
async fn migrations_land_in_public(pool: PgPool) {
    let postgrest = postgrest().await;
    let (relay, project) = standalone(&pool, &postgrest.uri()).await;
    save(&relay, from_nomi()).await.unwrap();
    ensure_backend(&relay, project, "Barbershop", Tier::Free, None).await.unwrap();

    let files = vec![MigrationFile { path: "nomi/migrations/0001_init.sql".into(), content: "create table task (id int primary key);\n".into() }];
    let applied = apply(&relay, project, Env::Production, &files).await.unwrap();

    assert_eq!(applied, vec!["0001_init.sql"]);
    let found: bool = sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM pg_tables WHERE schemaname = 'public' AND tablename = 'task')").fetch_one(&pool).await.unwrap();
    assert!(found);
}
