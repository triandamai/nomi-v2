//! The shared Relay older projects still use: a saved connection, kept encrypted, with the
//! environment as the fallback.
use nomi_agent_core::relay_ops::RelayOps;
use nomi_relay_client::RelayConfig;
use nomi_server::relay_connection::{load_or, save};
use nomi_server::relay_ops::ServerRelayOps;
use sqlx::PgPool;
use uuid::Uuid;

const KEY: [u8; 32] = [3u8; 32];

async fn admin(pool: &PgPool) -> Uuid {
    sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(pool).await.unwrap()
}

fn from_env() -> Option<RelayConfig> {
    RelayConfig::from_values(Some("http://relay-from-env"), Some("env-key"))
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_saved_connection_wins_over_the_environment_and_is_sealed(pool: PgPool) {
    let user = admin(&pool).await;
    assert_eq!(load_or(&pool, Some(&KEY), from_env()).await.unwrap().control_url, "http://relay-from-env");

    save(&pool, &KEY, "https://relay.example/", Some("stored-key"), user).await.unwrap();
    let config = load_or(&pool, Some(&KEY), from_env()).await.unwrap();
    assert_eq!(config.control_url, "https://relay.example");
    assert_eq!(config.control_key, "stored-key");
    let sealed: Vec<u8> = sqlx::query_scalar("SELECT control_key_encrypted FROM relay_connection").fetch_one(&pool).await.unwrap();
    assert!(!String::from_utf8_lossy(&sealed).contains("stored-key"));
}

#[sqlx::test(migrations = "../../migrations")]
async fn saving_without_a_key_keeps_the_stored_one_and_a_first_save_needs_one(pool: PgPool) {
    let user = admin(&pool).await;
    assert!(save(&pool, &KEY, "https://relay.example", None, user).await.is_err());
    save(&pool, &KEY, "https://relay.example", Some("stored-key"), user).await.unwrap();
    save(&pool, &KEY, "https://relay2.example", Some(""), user).await.unwrap();
    let config = load_or(&pool, Some(&KEY), None).await.unwrap();
    assert_eq!(config.control_url, "https://relay2.example");
    assert_eq!(config.control_key, "stored-key");
    assert!(save(&pool, &KEY, "not a url", None, user).await.is_err());
}

#[sqlx::test(migrations = "../../migrations")]
async fn agents_see_a_connection_as_soon_as_its_saved(pool: PgPool) {
    let user = admin(&pool).await;
    let ops = ServerRelayOps::stored(pool.clone(), reqwest::Client::new(), Some(KEY), None);
    assert!(!ops.is_available().await);
    save(&pool, &KEY, "https://relay.example", Some("stored-key"), user).await.unwrap();
    assert!(ops.is_available().await);
}
