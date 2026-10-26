//! A project's sign-in settings for one environment: which methods are on, where OAuth may send
//! people back, how long sessions last, and its own SMTP server. Kept sealed in Relay's state.

use relay_core::crypto::{decrypt, encrypt};
use relay_core::settings::Smtp;
use relay_core::Env;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::AuthError;

pub const DEFAULT_SESSION_DAYS: u32 = 30;
pub const EMAIL_CODE: &str = "email_code";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuthConfig {
    pub methods: Vec<String>,
    #[serde(default)]
    pub redirect_urls: Vec<String>,
    pub session_ttl_days: u32,
    /// The project's own sender; `None` uses Relay's shared one.
    #[serde(default)]
    pub smtp: Option<Smtp>,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self { methods: vec![EMAIL_CODE.to_string()], redirect_urls: Vec::new(), session_ttl_days: DEFAULT_SESSION_DAYS, smtp: None }
    }
}

impl AuthConfig {
    pub fn allows(&self, method: &str) -> bool {
        self.methods.iter().any(|m| m == method)
    }
}

pub async fn load(state: &PgPool, key: &[u8; 32], project: Uuid, env: Env) -> Result<AuthConfig, AuthError> {
    let sealed: Option<Vec<u8>> = sqlx::query_scalar("SELECT sealed FROM relay.auth_configs WHERE project_id = $1 AND env = $2")
        .bind(project)
        .bind(env.as_str())
        .fetch_optional(state)
        .await?;
    let Some(sealed) = sealed else { return Ok(AuthConfig::default()) };
    let text = decrypt(key, &sealed).map_err(|e| AuthError::Internal(e.to_string()))?;
    serde_json::from_str(&text).map_err(|e| AuthError::Internal(e.to_string()))
}

pub async fn save(state: &PgPool, key: &[u8; 32], project: Uuid, env: Env, config: &AuthConfig) -> Result<(), AuthError> {
    let text = serde_json::to_string(config).map_err(|e| AuthError::Internal(e.to_string()))?;
    sqlx::query(
        "INSERT INTO relay.auth_configs (project_id, env, sealed) VALUES ($1, $2, $3) \
         ON CONFLICT (project_id, env) DO UPDATE SET sealed = EXCLUDED.sealed",
    )
    .bind(project)
    .bind(env.as_str())
    .bind(encrypt(key, &text))
    .execute(state)
    .await?;
    Ok(())
}
