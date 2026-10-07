//! The tool registry's switches and settings (Admin → Tools). Every tool an agent can be given
//! is listed there; an admin turns each on or off and edits its settings. The engine only offers
//! agents the tools that are on, and refuses a call to one that's off.

use std::collections::{HashMap, HashSet};

use serde_json::Value;
use sqlx::PgConnection;

use crate::engine::{COMPLETE_TASK_TOOL_NAME, DELEGATE_TOOL_NAME};

/// Tools the crew can't work without (finishing a task, handing it to another agent): listed,
/// but never switched off.
pub const REQUIRED_TOOLS: [&str; 2] = [COMPLETE_TASK_TOOL_NAME, DELEGATE_TOOL_NAME];

/// Which tools are switched off right now. Read once per turn: cheap, and an admin's change
/// applies from the next turn on.
#[derive(Debug, Default, Clone)]
pub struct ToolSwitches {
    off: HashSet<String>,
}

impl ToolSwitches {
    pub async fn load(conn: &mut PgConnection) -> Self {
        let off: Vec<String> = sqlx::query_scalar("SELECT name FROM tool_settings WHERE NOT enabled")
            .fetch_all(conn)
            .await
            .unwrap_or_else(|e| {
                tracing::warn!(error = %e, "tools: couldn't read tool switches; treating every tool as on");
                Vec::new()
            });
        Self { off: off.into_iter().collect() }
    }

    pub fn allows(&self, name: &str) -> bool {
        REQUIRED_TOOLS.contains(&name) || !self.off.contains(name)
    }
}

/// One tool's saved row: its switch, its settings, and (still encrypted) its secrets.
#[derive(Debug, Clone, Default)]
pub struct ToolRow {
    pub enabled: bool,
    pub config: Value,
    pub secrets_encrypted: Option<Vec<u8>>,
}

/// The saved row for `name`, or the defaults (on, no settings) when nothing was saved.
pub async fn load_row(conn: &mut PgConnection, name: &str) -> Result<ToolRow, sqlx::Error> {
    let row: Option<(bool, Value, Option<Vec<u8>>)> =
        sqlx::query_as("SELECT enabled, config, secrets_encrypted FROM tool_settings WHERE name = $1").bind(name).fetch_optional(conn).await?;
    Ok(match row {
        Some((enabled, config, secrets_encrypted)) => ToolRow { enabled, config, secrets_encrypted },
        None => ToolRow { enabled: true, config: Value::Object(Default::default()), secrets_encrypted: None },
    })
}

pub async fn set_enabled(conn: &mut PgConnection, name: &str, enabled: bool) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO tool_settings (name, enabled) VALUES ($1, $2) \
         ON CONFLICT (name) DO UPDATE SET enabled = EXCLUDED.enabled, updated_at = now()",
    )
    .bind(name)
    .bind(enabled)
    .execute(conn)
    .await?;
    Ok(())
}

/// Saves a tool's settings; `secrets` (already encrypted) replaces the stored ones when given.
pub async fn save_settings(conn: &mut PgConnection, name: &str, config: &Value, secrets: Option<Vec<u8>>) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO tool_settings (name, config, secrets_encrypted) VALUES ($1, $2, $3) \
         ON CONFLICT (name) DO UPDATE SET config = EXCLUDED.config, \
             secrets_encrypted = COALESCE(EXCLUDED.secrets_encrypted, tool_settings.secrets_encrypted), updated_at = now()",
    )
    .bind(name)
    .bind(config)
    .bind(secrets)
    .execute(conn)
    .await?;
    Ok(())
}

/// The key that encrypts tool secrets (the same one that protects stored model keys).
pub fn settings_key() -> Option<[u8; 32]> {
    std::env::var("SETTINGS_ENCRYPTION_KEY").ok().and_then(|k| nomi_settings::crypto::parse_key(k.trim()))
}

/// A tool's secrets (name → value), decrypted. Empty when none are saved or they can't be read.
pub fn decrypt_secrets(encrypted: Option<&[u8]>) -> HashMap<String, String> {
    match settings_key() {
        Some(key) => decrypt_secrets_with(&key, encrypted),
        None => HashMap::new(),
    }
}

pub fn decrypt_secrets_with(key: &[u8; 32], encrypted: Option<&[u8]>) -> HashMap<String, String> {
    let Some(data) = encrypted else { return HashMap::new() };
    nomi_settings::crypto::decrypt(key, data).ok().and_then(|json| serde_json::from_str(&json).ok()).unwrap_or_default()
}

/// Encrypts a tool's secrets for storage.
pub fn encrypt_secrets_with(key: &[u8; 32], secrets: &HashMap<String, String>) -> Vec<u8> {
    nomi_settings::crypto::encrypt(key, &serde_json::to_string(secrets).unwrap_or_default())
}
