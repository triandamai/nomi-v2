//! How Nomi reaches Nomi Relay: Relay's address and control key, set in the admin (Relay page)
//! and kept encrypted with the settings key. `RELAY_CONTROL_URL` / `RELAY_CONTROL_KEY` are the
//! fallback for a Nomi set up before the page existed. Read on every use, so the server and the
//! worker see a saved connection at once.

use nomi_relay_client::RelayConfig;
use nomi_settings::crypto::{decrypt, encrypt, parse_key};
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

/// Where the connection in use came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Source {
    Admin,
    Environment,
    None,
}

/// What the admin page shows: never the key itself.
#[derive(Debug, Clone, Serialize)]
pub struct ConnectionView {
    pub source: Source,
    pub control_url: String,
    pub key_set: bool,
}

/// The settings key from the environment, as the server and worker read it at start.
pub fn settings_key_from_env() -> Option<[u8; 32]> {
    std::env::var("SETTINGS_ENCRYPTION_KEY").ok().and_then(|hex| parse_key(&hex))
}

async fn stored(pool: &PgPool, settings_key: Option<&[u8; 32]>) -> Option<RelayConfig> {
    let key = settings_key?;
    let row: Option<(String, Vec<u8>)> = match sqlx::query_as("SELECT control_url, control_key_encrypted FROM relay_connection").fetch_optional(pool).await {
        Ok(row) => row,
        Err(e) => {
            tracing::warn!(error = %e, "couldn't read the Relay connection");
            return None;
        }
    };
    let (url, sealed) = row?;
    match decrypt(key, &sealed) {
        Ok(control_key) => RelayConfig::from_values(Some(&url), Some(&control_key)),
        Err(_) => {
            tracing::error!("the stored Relay control key can't be decrypted with this settings key");
            None
        }
    }
}

/// The connection to use: the admin's, else `fallback` (the environment's).
pub async fn load_or(pool: &PgPool, settings_key: Option<&[u8; 32]>, fallback: Option<RelayConfig>) -> Option<RelayConfig> {
    match stored(pool, settings_key).await {
        Some(config) => Some(config),
        None => fallback,
    }
}

/// The connection to use, with the environment as the fallback.
pub async fn load(pool: &PgPool, settings_key: Option<&[u8; 32]>) -> Option<RelayConfig> {
    load_or(pool, settings_key, RelayConfig::from_env()).await
}

pub async fn view(pool: &PgPool, settings_key: Option<&[u8; 32]>, fallback: Option<&RelayConfig>) -> ConnectionView {
    match (stored(pool, settings_key).await, fallback) {
        (Some(config), _) => ConnectionView { source: Source::Admin, control_url: config.control_url, key_set: true },
        (None, Some(config)) => ConnectionView { source: Source::Environment, control_url: config.control_url.clone(), key_set: true },
        (None, None) => ConnectionView { source: Source::None, control_url: String::new(), key_set: false },
    }
}

/// Saves Relay's address and, when given, its key; an empty or missing key keeps the stored one.
pub async fn save(pool: &PgPool, settings_key: &[u8; 32], control_url: &str, control_key: Option<&str>, user_id: Uuid) -> Result<(), String> {
    let url = control_url.trim().trim_end_matches('/');
    let parsed = reqwest::Url::parse(url).map_err(|_| "Relay's address must be a URL like https://relay.example.com".to_string())?;
    if !matches!(parsed.scheme(), "http" | "https") {
        return Err("Relay's address must start with http:// or https://".to_string());
    }
    let sealed = match control_key.map(str::trim).filter(|k| !k.is_empty()) {
        Some(key) => encrypt(settings_key, key),
        None => sqlx::query_scalar::<_, Vec<u8>>("SELECT control_key_encrypted FROM relay_connection")
            .fetch_optional(pool)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "Relay's control key is needed the first time".to_string())?,
    };
    sqlx::query(
        "INSERT INTO relay_connection (id, control_url, control_key_encrypted, updated_by) VALUES (true, $1, $2, $3) \
         ON CONFLICT (id) DO UPDATE SET control_url = EXCLUDED.control_url, control_key_encrypted = EXCLUDED.control_key_encrypted, \
         updated_by = EXCLUDED.updated_by, updated_at = now()",
    )
    .bind(url)
    .bind(sealed)
    .bind(user_id)
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;
    Ok(())
}
