//! Nomi's configuration as set in Admin → Configuration: attachments, file storage, Google,
//! GitHub, email, the agent loop, Relay and app domains. Most fields have the environment variable
//! they replace, which stays the fallback for anything not saved here; a field without one is only
//! ever read from here. Secrets are encrypted with the settings key and never
//! shown back. The server and the worker read the saved values (cached for a few seconds), so a
//! save takes effect without a restart.

use std::collections::{BTreeMap, HashMap};
use std::sync::{Mutex, OnceLock, RwLock};
use std::time::{Duration, Instant};

use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::crypto::{decrypt, encrypt, parse_key};

/// How long a process trusts what it last read before reading again: how soon a save made in
/// one process (the server) reaches another (the worker).
const CACHE_SECONDS: u64 = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Text,
    /// A whole number above zero.
    Number,
    /// Encrypted, and never shown back.
    Secret,
    Choice(&'static [&'static str]),
    /// A domain name such as `nomi.app`: saved lowercased, without a trailing dot.
    Domain,
}

#[derive(Debug)]
pub struct Field {
    pub name: &'static str,
    /// The environment variable it replaces, and falls back to. `None`: saved here or unset.
    pub env: Option<&'static str>,
    pub kind: Kind,
}

#[derive(Debug)]
pub struct SectionDef {
    pub name: &'static str,
    pub fields: &'static [Field],
}

const fn field(name: &'static str, env: &'static str, kind: Kind) -> Field {
    Field { name, env: Some(env), kind }
}

/// A field with no environment variable behind it: only what's saved in Admin → Configuration.
const fn saved_only(name: &'static str, kind: Kind) -> Field {
    Field { name, env: None, kind }
}

impl Field {
    /// What `env` has for this field's variable, trimmed; `None` when it has no variable.
    fn from_env(&self, env: &impl Fn(&str) -> Option<String>) -> Option<String> {
        self.env.and_then(env).map(|v| v.trim().to_string()).filter(|v| !v.is_empty())
    }
}

pub const ATTACHMENTS: SectionDef = SectionDef {
    name: "attachments",
    fields: &[
        field("max_file_mb", "ATTACHMENT_MAX_FILE_MB", Kind::Number),
        field("max_video_mb", "ATTACHMENT_MAX_VIDEO_MB", Kind::Number),
        field("max_message_mb", "ATTACHMENT_MAX_MESSAGE_MB", Kind::Number),
        field("max_files", "ATTACHMENT_MAX_FILES", Kind::Number),
        field("quota_mb", "ATTACHMENT_QUOTA_MB", Kind::Number),
    ],
};

pub const STORAGE: SectionDef = SectionDef {
    name: "storage",
    fields: &[
        field("bucket", "S3_BUCKET", Kind::Text),
        field("region", "S3_REGION", Kind::Text),
        field("access_key_id", "AWS_ACCESS_KEY_ID", Kind::Text),
        field("secret_access_key", "AWS_SECRET_ACCESS_KEY", Kind::Secret),
        field("endpoint_url", "S3_ENDPOINT_URL", Kind::Text),
        field("public_url_base", "S3_PUBLIC_URL_BASE", Kind::Text),
    ],
};

pub const GOOGLE: SectionDef = SectionDef {
    name: "google",
    fields: &[
        field("client_id", "GOOGLE_CLIENT_ID", Kind::Text),
        field("client_secret", "GOOGLE_CLIENT_SECRET", Kind::Secret),
        field("redirect_uri", "GOOGLE_REDIRECT_URI", Kind::Text),
        field("signin_redirect_uri", "GOOGLE_SIGNIN_REDIRECT_URI", Kind::Text),
    ],
};

/// Nomi's GitHub OAuth app, for people to connect GitHub and push a project to a repository.
/// Without it they can still connect with a token of their own.
pub const GITHUB: SectionDef = SectionDef {
    name: "github",
    fields: &[
        field("client_id", "GITHUB_CLIENT_ID", Kind::Text),
        field("client_secret", "GITHUB_CLIENT_SECRET", Kind::Secret),
        field("redirect_uri", "GITHUB_REDIRECT_URI", Kind::Text),
    ],
};

pub const SMTP_SECURITY: &[&str] = &["starttls", "tls", "none"];

pub const SMTP: SectionDef = SectionDef {
    name: "smtp",
    fields: &[
        field("host", "SMTP_HOST", Kind::Text),
        field("port", "SMTP_PORT", Kind::Number),
        field("username", "SMTP_USERNAME", Kind::Text),
        field("password", "SMTP_PASSWORD", Kind::Secret),
        field("security", "SMTP_SECURITY", Kind::Choice(SMTP_SECURITY)),
        field("from", "MAIL_FROM", Kind::Text),
    ],
};

pub const AGENT_LOOP: SectionDef = SectionDef {
    name: "agent_loop",
    fields: &[
        field("turn_concurrency", "TURN_CONCURRENCY", Kind::Number),
        field("turn_time_limit_secs", "TURN_TIME_LIMIT_SECS", Kind::Number),
        field("delegation_time_limit_secs", "DELEGATION_TIME_LIMIT_SECS", Kind::Number),
    ],
};

/// Where the images a project's own Relay runs are: the repository of Relay's and its functions
/// runtime's images in Shipyard's registry (`<host>/<prefix>/relay`), PostgREST's full image and
/// the tag the first two are run at.
pub const RELAY: SectionDef = SectionDef {
    name: "relay",
    fields: &[
        field("relay_image", "RELAY_IMAGE", Kind::Text),
        field("functions_image", "RELAY_FUNCTIONS_IMAGE", Kind::Text),
        field("postgrest_image", "RELAY_POSTGREST_IMAGE", Kind::Text),
        field("version", "RELAY_IMAGE_VERSION", Kind::Text),
        field("sdk_package", "RELAY_SDK_PACKAGE", Kind::Text),
        field("sdk_version", "RELAY_SDK_VERSION", Kind::Text),
        field("cli_package", "RELAY_CLI_PACKAGE", Kind::Text),
        field("cli_version", "RELAY_CLI_VERSION", Kind::Text),
    ],
};

/// The domain every app Nomi publishes is addressed under: `<name>.<apps_base_domain>`. Only
/// set here; unset, Shipyard picks the address under its own apps domain, as before.
pub const DOMAINS: SectionDef = SectionDef {
    name: "domains",
    fields: &[saved_only("apps_base_domain", Kind::Domain)],
};

pub const XENDIT: SectionDef = SectionDef {
    name: "xendit",
    fields: &[
        field("secret_key", "XENDIT_SECRET_KEY", Kind::Secret),
        field("webhook_token", "XENDIT_WEBHOOK_TOKEN", Kind::Secret),
        field("public_key", "XENDIT_PUBLIC_KEY", Kind::Text),
        field("base_url", "XENDIT_BASE_URL", Kind::Text),
    ],
};

pub const SECTIONS: [&SectionDef; 9] = [&ATTACHMENTS, &STORAGE, &GOOGLE, &GITHUB, &SMTP, &AGENT_LOOP, &RELAY, &DOMAINS, &XENDIT];

pub fn section_named(name: &str) -> Option<&'static SectionDef> {
    SECTIONS.into_iter().find(|s| s.name == name)
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("{0}")]
    Invalid(String),
    #[error("secrets can't be saved: SETTINGS_ENCRYPTION_KEY isn't set")]
    NoKey,
    #[error(transparent)]
    Db(#[from] sqlx::Error),
}

/// What's saved for one section, before the environment fills the gaps.
#[derive(Debug, Clone)]
pub struct Saved {
    def: &'static SectionDef,
    values: BTreeMap<&'static str, String>,
}

impl Saved {
    /// Every field: the saved value, or else what `env` has for its variable.
    pub fn with_env(self, env: impl Fn(&str) -> Option<String>) -> Section {
        let mut values = BTreeMap::new();
        for field in self.def.fields {
            let value = self.values.get(field.name).cloned().or_else(|| field.from_env(&env));
            if let Some(value) = value {
                values.insert(field.name, value);
            }
        }
        Section { values }
    }
}

/// One section's values as the code uses them: saved, else from the environment.
#[derive(Debug, Clone, Default)]
pub struct Section {
    values: BTreeMap<&'static str, String>,
}

impl Section {
    pub fn get(&self, field: &str) -> Option<String> {
        self.values.get(field).cloned()
    }

    /// A number field, when set to a whole number above zero.
    pub fn number(&self, field: &str) -> Option<u64> {
        self.values.get(field).and_then(|v| v.trim().parse().ok()).filter(|n| *n > 0)
    }
}

fn process_env(name: &str) -> Option<String> {
    std::env::var(name).ok()
}

/// The settings key from the environment, read on use like the other admin settings.
pub fn settings_key() -> Option<[u8; 32]> {
    std::env::var("SETTINGS_ENCRYPTION_KEY").ok().and_then(|hex| parse_key(&hex))
}

/// What's saved for `def`. Without `key`, secrets are skipped (and the environment's used).
pub async fn load(pool: &PgPool, key: Option<&[u8; 32]>, def: &'static SectionDef) -> Result<Saved, sqlx::Error> {
    let rows: Vec<(String, Option<String>, Option<Vec<u8>>)> =
        sqlx::query_as("SELECT field, value, value_encrypted FROM app_config WHERE section = $1").bind(def.name).fetch_all(pool).await?;
    let mut values = BTreeMap::new();
    for (name, value, sealed) in rows {
        let Some(field) = def.fields.iter().find(|f| f.name == name) else { continue };
        let value = match (value, sealed, key) {
            (Some(value), _, _) => Some(value),
            (None, Some(sealed), Some(key)) => decrypt(key, &sealed).ok(),
            _ => None,
        };
        if let Some(value) = value {
            values.insert(field.name, value);
        }
    }
    Ok(Saved { def, values })
}

fn check(field: &Field, value: &str) -> Result<(), ConfigError> {
    let fail = |why: &str| Err(ConfigError::Invalid(format!("{}: {why}", field.name)));
    match field.kind {
        Kind::Number if !value.parse::<u64>().is_ok_and(|n| n > 0) => fail("use a whole number above 0"),
        Kind::Choice(choices) if !choices.contains(&value) => fail(&format!("use one of {}", choices.join(", "))),
        Kind::Domain if !is_domain(value) => fail("use a domain name such as apps.example.com, without https:// or a path"),
        _ => Ok(()),
    }
}

/// The longest domain name DNS allows, and the longest label in one.
const DOMAIN_MAX: usize = 253;
const LABEL_MAX: usize = 63;

/// A domain name of two or more labels, each `a-z 0-9 -` not starting or ending with `-`, and not
/// an IP address.
fn is_domain(value: &str) -> bool {
    let labels: Vec<&str> = value.split('.').collect();
    let label_ok = |l: &&str| {
        !l.is_empty() && l.len() <= LABEL_MAX && !l.starts_with('-') && !l.ends_with('-') && l.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
    };
    value.len() <= DOMAIN_MAX
        && labels.len() >= 2
        && labels.iter().all(label_ok)
        && !labels.last().is_some_and(|tld| tld.bytes().all(|b| b.is_ascii_digit()))
}

/// `value` as it's saved: a domain lowercased, without a trailing dot; anything else as given.
fn normalize(field: &Field, value: &str) -> String {
    match field.kind {
        Kind::Domain => value.trim_end_matches('.').to_lowercase(),
        _ => value.to_string(),
    }
}

/// Saves `changes` to `def`, all or nothing. An empty value removes the saved one (the
/// environment applies again), except for a secret, where empty keeps what's saved.
pub async fn save(pool: &PgPool, key: &[u8; 32], def: &'static SectionDef, changes: &BTreeMap<String, String>, user: Uuid) -> Result<(), ConfigError> {
    let mut checked = Vec::new();
    for (name, value) in changes {
        let field = def.fields.iter().find(|f| f.name == name).ok_or_else(|| ConfigError::Invalid(format!("{name} isn't part of {}", def.name)))?;
        let value = normalize(field, value.trim());
        if !value.is_empty() {
            check(field, &value)?;
        }
        checked.push((field, value));
    }
    let mut tx = pool.begin().await?;
    for (field, value) in checked {
        let value = value.as_str();
        match (value.is_empty(), field.kind) {
            (true, Kind::Secret) => {}
            (true, _) => {
                sqlx::query("DELETE FROM app_config WHERE section = $1 AND field = $2").bind(def.name).bind(field.name).execute(&mut *tx).await?;
            }
            (false, kind) => {
                let (plain, sealed) = if kind == Kind::Secret { (None, Some(encrypt(key, value))) } else { (Some(value.to_string()), None) };
                sqlx::query(
                    "INSERT INTO app_config (section, field, value, value_encrypted, updated_by) VALUES ($1, $2, $3, $4, $5) \
                     ON CONFLICT (section, field) DO UPDATE SET value = $3, value_encrypted = $4, updated_by = $5, updated_at = now()",
                )
                .bind(def.name)
                .bind(field.name)
                .bind(plain)
                .bind(sealed)
                .bind(user)
                .execute(&mut *tx)
                .await?;
            }
        }
    }
    tx.commit().await?;
    forget();
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Source {
    Saved,
    Environment,
    None,
}

/// One field as the admin page shows it: never a secret's value.
#[derive(Debug, Serialize)]
pub struct FieldView {
    pub name: &'static str,
    pub env: Option<&'static str>,
    pub kind: &'static str,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub choices: Vec<&'static str>,
    pub value: Option<String>,
    pub set: bool,
    pub source: Source,
}

pub async fn view(pool: &PgPool, key: Option<&[u8; 32]>, def: &'static SectionDef, env: impl Fn(&str) -> Option<String>) -> Result<Vec<FieldView>, sqlx::Error> {
    let saved = load(pool, key, def).await?;
    // A secret saved under a key this process can't read still counts as saved.
    let sealed: Vec<String> =
        sqlx::query_scalar("SELECT field FROM app_config WHERE section = $1 AND value_encrypted IS NOT NULL").bind(def.name).fetch_all(pool).await?;
    Ok(def
        .fields
        .iter()
        .map(|field| {
            let from_env = field.from_env(&env);
            let saved_value = saved.values.get(field.name);
            let source = if saved_value.is_some() || sealed.iter().any(|s| s == field.name) {
                Source::Saved
            } else if from_env.is_some() {
                Source::Environment
            } else {
                Source::None
            };
            let secret = field.kind == Kind::Secret;
            FieldView {
                name: field.name,
                env: field.env,
                kind: match field.kind {
                    Kind::Text => "text",
                    Kind::Number => "number",
                    Kind::Secret => "secret",
                    Kind::Choice(_) => "choice",
                    Kind::Domain => "domain",
                },
                choices: match field.kind {
                    Kind::Choice(choices) => choices.to_vec(),
                    _ => Vec::new(),
                },
                value: if secret { None } else { saved_value.cloned().or(from_env) },
                set: source != Source::None,
                source,
            }
        })
        .collect())
}

fn pool_slot() -> &'static RwLock<Option<PgPool>> {
    static POOL: OnceLock<RwLock<Option<PgPool>>> = OnceLock::new();
    POOL.get_or_init(|| RwLock::new(None))
}

fn cache() -> &'static Mutex<HashMap<&'static str, (Instant, Section)>> {
    static CACHE: OnceLock<Mutex<HashMap<&'static str, (Instant, Section)>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Where `current` reads saved values from: the server and the worker call this once at start.
pub fn install(pool: PgPool) {
    *pool_slot().write().unwrap_or_else(|e| e.into_inner()) = Some(pool);
}

/// Drops what this process cached, so the next read sees the database.
pub fn forget() {
    cache().lock().unwrap_or_else(|e| e.into_inner()).clear();
}

/// `def` as the code should use it now: saved values (when `install` was called), else the
/// process environment. Never fails: a database that can't be read means the environment.
pub async fn current(def: &'static SectionDef) -> Section {
    if let Some((at, section)) = cache().lock().unwrap_or_else(|e| e.into_inner()).get(def.name) {
        if at.elapsed() < Duration::from_secs(CACHE_SECONDS) {
            return section.clone();
        }
    }
    let pool = pool_slot().read().unwrap_or_else(|e| e.into_inner()).clone();
    let saved = match pool {
        Some(pool) => load(&pool, settings_key().as_ref(), def).await.ok(),
        None => None,
    };
    let section = saved.unwrap_or(Saved { def, values: BTreeMap::new() }).with_env(process_env);
    cache().lock().unwrap_or_else(|e| e.into_inner()).insert(def.name, (Instant::now(), section.clone()));
    section
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_field_has_a_unique_name_and_variable() {
        let mut envs: Vec<&str> = SECTIONS.iter().flat_map(|s| s.fields.iter().filter_map(|f| f.env)).collect();
        let count = envs.len();
        envs.sort();
        envs.dedup();
        assert_eq!(envs.len(), count);
        for section in SECTIONS {
            assert_eq!(section_named(section.name).map(|s| s.name), Some(section.name));
        }
    }

    #[test]
    fn the_apps_base_domain_comes_only_from_what_is_saved() {
        let saved = Saved { def: &DOMAINS, values: BTreeMap::new() };
        let section = saved.with_env(|_| Some("from-env.example.com".to_string()));
        assert_eq!(section.get("apps_base_domain"), None);
    }

    #[test]
    fn a_domain_is_saved_lowercased_and_must_be_a_bare_domain_name() {
        let field = &DOMAINS.fields[0];
        assert_eq!(normalize(field, "Nomi.App."), "nomi.app");
        for good in ["nomi.app", "apps.nomi.dev", "a-b.example.co.uk"] {
            assert!(check(field, good).is_ok(), "{good}");
        }
        for bad in ["nomi", "https://nomi.app", "nomi.app/x", "-nomi.app", "nomi..app", "*.nomi.app", "10.0.0.1", "no mi.app"] {
            assert!(check(field, bad).is_err(), "{bad}");
        }
    }
}
