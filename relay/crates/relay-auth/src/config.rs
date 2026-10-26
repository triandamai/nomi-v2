//! A project's sign-in settings for one environment: which methods are on, where OAuth may send
//! people back, how long sessions last, and its own SMTP server. Kept sealed in Relay's state.

use std::collections::BTreeMap;

use relay_core::crypto::{decrypt, encrypt};
use relay_core::settings::Smtp;
use relay_core::Env;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::AuthError;

pub const DEFAULT_SESSION_DAYS: u32 = 30;
pub const EMAIL_CODE: &str = "email_code";
pub const GOOGLE: &str = "google";
pub const GITHUB: &str = "github";
pub const APPLE: &str = "apple";
/// `oidc:<id>` names one of the project's OpenID Connect providers.
pub const OIDC_PREFIX: &str = "oidc:";
pub const SMTP_PASSWORD_SECRET: &str = "NOMI_SMTP_PASSWORD";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClientConfig {
    pub client_id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GithubConfig {
    pub client_id: String,
    /// GitHub Enterprise: its API and web addresses.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_base: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub web_base: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppleConfig {
    /// The Services ID.
    pub client_id: String,
    pub team_id: String,
    pub key_id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OidcConfig {
    pub id: String,
    pub issuer: String,
    pub client_id: String,
}

/// A project's sign-in secrets by name (`NOMI_OAUTH_GITHUB_SECRET`, …), sent by Nomi when an
/// admin sets them.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Secrets(pub BTreeMap<String, String>);

impl Secrets {
    pub fn get(&self, name: &str) -> Option<&str> {
        self.0.get(name).map(String::as_str).filter(|v| !v.is_empty())
    }
}

/// The project secret a provider's client secret (or Apple's `.p8`) is kept under.
pub fn secret_name(provider: &str) -> String {
    match provider {
        APPLE => "NOMI_OAUTH_APPLE_KEY".to_string(),
        _ => {
            let id = provider.strip_prefix(OIDC_PREFIX).map(|id| format!("OIDC_{id}")).unwrap_or_else(|| provider.to_string());
            let shouted: String = id.chars().map(|c| if c.is_ascii_alphanumeric() { c.to_ascii_uppercase() } else { '_' }).collect();
            format!("NOMI_OAUTH_{shouted}_SECRET")
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuthConfig {
    pub methods: Vec<String>,
    #[serde(default)]
    pub redirect_urls: Vec<String>,
    pub session_ttl_days: u32,
    /// The project's own sender; `None` uses Relay's shared one.
    #[serde(default)]
    pub smtp: Option<Smtp>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub google: Option<ClientConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub github: Option<GithubConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub apple: Option<AppleConfig>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub oidc: Vec<OidcConfig>,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            methods: vec![EMAIL_CODE.to_string()],
            redirect_urls: Vec::new(),
            session_ttl_days: DEFAULT_SESSION_DAYS,
            smtp: None,
            google: None,
            github: None,
            apple: None,
            oidc: Vec::new(),
        }
    }
}

impl AuthConfig {
    pub fn allows(&self, method: &str) -> bool {
        self.methods.iter().any(|m| m == method)
    }

    /// The secrets switched-on methods need that the project hasn't set, sorted.
    pub fn missing_secrets(&self, secrets: &Secrets) -> Vec<String> {
        let mut missing: Vec<String> = self.methods.iter().filter(|m| m.as_str() != EMAIL_CODE).map(|m| secret_name(m)).filter(|name| secrets.get(name).is_none()).collect();
        let smtp_needs_password = self.smtp.as_ref().is_some_and(|smtp| !smtp.user.is_empty() && smtp.password.is_empty());
        if smtp_needs_password && secrets.get(SMTP_PASSWORD_SECRET).is_none() {
            missing.push(SMTP_PASSWORD_SECRET.to_string());
        }
        missing.sort();
        missing.dedup();
        missing
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
