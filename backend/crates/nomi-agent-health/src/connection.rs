//! Each person's connected health and fitness apps (`health_connections`). A key is checked
//! with the app before it's saved, stored encrypted, and only ever used for its owner.

use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgConnection;
use uuid::Uuid;

use crate::hevy::{Hevy, HevyError};

pub const HEVY: &str = "hevy";

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct HealthConnection {
    pub provider: String,
    pub account_name: Option<String>,
    pub connected_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
}

/// The person's connection to `provider`, if any.
pub async fn status(conn: &mut PgConnection, user_id: Uuid, provider: &str) -> Result<Option<HealthConnection>, sqlx::Error> {
    let row: Option<(String, Option<String>, DateTime<Utc>, Option<DateTime<Utc>>)> = sqlx::query_as(
        "SELECT provider, account_name, created_at, last_used_at FROM health_connections WHERE user_id = $1 AND provider = $2",
    )
    .bind(user_id)
    .bind(provider)
    .fetch_optional(&mut *conn)
    .await?;
    Ok(row.map(|(provider, account_name, connected_at, last_used_at)| HealthConnection { provider, account_name, connected_at, last_used_at }))
}

/// Every app the person has connected (for the Connections list).
pub async fn connected_providers(conn: &mut PgConnection, user_id: Uuid) -> Result<Vec<String>, sqlx::Error> {
    sqlx::query_scalar("SELECT provider FROM health_connections WHERE user_id = $1").bind(user_id).fetch_all(&mut *conn).await
}

#[derive(Debug)]
pub enum ConnectError {
    /// Hevy didn't accept the key; the reason to show.
    Rejected(HevyError),
    Db(sqlx::Error),
}

/// Checks `api_key` with Hevy (at `base_url`, normally `hevy::base_url()`) and saves it for
/// `user_id`, replacing any earlier key.
pub async fn connect_hevy(
    conn: &mut PgConnection,
    http: &reqwest::Client,
    base_url: &str,
    key: &[u8; 32],
    user_id: Uuid,
    api_key: &str,
) -> Result<HealthConnection, ConnectError> {
    let api_key = api_key.trim();
    let hevy = Hevy { http, base_url: base_url.to_string(), api_key };
    let account = hevy.user_info().await.map_err(ConnectError::Rejected)?;
    let name = account.get("name").and_then(|v| v.as_str()).map(str::to_string);
    let id = account.get("id").and_then(|v| v.as_str()).map(str::to_string);
    sqlx::query(
        "INSERT INTO health_connections (user_id, provider, api_key_encrypted, account_name, account_id) VALUES ($1, $2, $3, $4, $5) \
         ON CONFLICT (user_id, provider) DO UPDATE SET api_key_encrypted = EXCLUDED.api_key_encrypted, account_name = EXCLUDED.account_name, \
             account_id = EXCLUDED.account_id, created_at = now(), last_used_at = NULL",
    )
    .bind(user_id)
    .bind(HEVY)
    .bind(nomi_settings::crypto::encrypt(key, api_key))
    .bind(&name)
    .bind(&id)
    .execute(&mut *conn)
    .await
    .map_err(ConnectError::Db)?;
    status(conn, user_id, HEVY).await.map_err(ConnectError::Db)?.ok_or(ConnectError::Db(sqlx::Error::RowNotFound))
}

/// Forgets the person's connection to `provider`. Returns whether there was one.
pub async fn disconnect(conn: &mut PgConnection, user_id: Uuid, provider: &str) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM health_connections WHERE user_id = $1 AND provider = $2").bind(user_id).bind(provider).execute(&mut *conn).await?;
    Ok(result.rows_affected() > 0)
}

/// The person's key for `provider`, decrypted, marking the connection as just used. `None` when
/// they haven't connected it (or it can't be read with this server's key).
pub async fn api_key(conn: &mut PgConnection, key: &[u8; 32], user_id: Uuid, provider: &str) -> Result<Option<String>, sqlx::Error> {
    let stored: Option<Vec<u8>> = sqlx::query_scalar(
        "UPDATE health_connections SET last_used_at = now() WHERE user_id = $1 AND provider = $2 RETURNING api_key_encrypted",
    )
    .bind(user_id)
    .bind(provider)
    .fetch_optional(&mut *conn)
    .await?;
    Ok(stored.and_then(|data| match nomi_settings::crypto::decrypt(key, &data) {
        Ok(api_key) => Some(api_key),
        Err(e) => {
            tracing::error!(error = %e, %user_id, provider, "couldn't decrypt a health connection key");
            None
        }
    }))
}
