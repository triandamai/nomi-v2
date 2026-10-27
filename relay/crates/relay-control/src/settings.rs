//! Relay's settings, as Nomi's admin sets them: checked before they're kept (each shared
//! database reachable with PostgREST's role in it, each PostgREST answering), sealed in Relay's
//! state, and written where PostgREST reads its own (its role's `pgrst.*` settings in each shared
//! database), so PostgREST needs nothing but its database URL.

use std::collections::HashMap;
use std::time::Duration;

use relay_core::crypto::{decrypt, encrypt};
use relay_core::keys::new_signing_secret;
use relay_auth::mail::Outbox;
use relay_core::settings::{mask_database_url, Settings, Smtp, StorageSettings};
use relay_core::Env;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

use crate::{postgrest, Live, Relay, RelayError};

/// How long a check waits for a database or a PostgREST.
const CHECK_TIMEOUT: Duration = Duration::from_secs(5);
/// The claim in the gateway's token that names the database role.
const ROLE_CLAIM_KEY: &str = ".db_role";
pub const SETTINGS_INVALID: &str = "SETTINGS_INVALID";
const SHARED_SENDER: &str = "shared sender";
const FILE_STORAGE: &str = "file storage";

/// What the admin sends. Database URLs left out (or empty) keep the stored ones: the form never
/// has their passwords.
#[derive(Debug, Clone, Deserialize)]
pub struct SettingsInput {
    pub public_domain: String,
    #[serde(default)]
    pub shared_dev_url: Option<String>,
    #[serde(default)]
    pub shared_production_url: Option<String>,
    pub postgrest_dev_url: String,
    pub postgrest_production_url: String,
    #[serde(default)]
    pub postgrest_role: Option<String>,
    #[serde(default)]
    pub preview_origins: Vec<String>,
    /// Left out keeps the stored sender; an empty host removes it; an empty password keeps the
    /// stored one.
    #[serde(default)]
    pub shared_smtp: Option<Smtp>,
    /// Left out keeps the stored bucket; an empty bucket name removes it; an empty secret keeps
    /// the stored one.
    #[serde(default)]
    pub storage: Option<StorageSettings>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Check {
    pub name: String,
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// What the admin sees: the settings with database URLs masked, and whether each part answers.
#[derive(Debug, Clone, Serialize)]
pub struct SettingsView {
    pub configured: bool,
    pub public_domain: String,
    pub shared_dev_url: String,
    pub shared_production_url: String,
    pub postgrest_dev_url: String,
    pub postgrest_production_url: String,
    pub postgrest_role: String,
    pub preview_origins: Vec<String>,
    /// Without its password.
    pub shared_smtp: Option<Smtp>,
    /// Without its secret.
    pub storage: Option<StorageSettings>,
    pub checks: Vec<Check>,
}

fn unsealable(e: impl std::fmt::Display) -> RelayError {
    RelayError::Internal(format!("Relay's saved settings can't be read: {e}"))
}

/// The saved settings, if any.
pub async fn load(relay: &Relay) -> Result<Option<Settings>, RelayError> {
    let sealed: Option<Vec<u8>> = sqlx::query_scalar("SELECT sealed FROM relay.settings").fetch_optional(&relay.state).await?;
    let Some(sealed) = sealed else { return Ok(None) };
    let text = decrypt(&relay.encryption_key, &sealed).map_err(unsealable)?;
    serde_json::from_str(&text).map(Some).map_err(unsealable)
}

fn database_name(env: Env) -> String {
    format!("{} database", env.as_str())
}

fn postgrest_name(env: Env) -> String {
    format!("{} PostgREST", env.as_str())
}

/// Reachable, and PostgREST's role exists there.
async fn check_database(pool: &PgPool, role: &str) -> Result<(), String> {
    let found = tokio::time::timeout(CHECK_TIMEOUT, sqlx::query_scalar::<_, bool>("SELECT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = $1)").bind(role).fetch_one(pool))
        .await
        .map_err(|_| "didn't answer in time".to_string())?
        .map_err(|e| e.to_string())?;
    if found {
        Ok(())
    } else {
        Err(format!("connected, but PostgREST's role {role} doesn't exist there"))
    }
}

/// Answers HTTP at all (PostgREST answers its root even without a token).
async fn check_postgrest(http: &reqwest::Client, url: &str) -> Result<(), String> {
    if url.is_empty() {
        return Err("not set".into());
    }
    http.get(url).timeout(CHECK_TIMEOUT).send().await.map(|_| ()).map_err(|e| e.to_string())
}

fn check(name: String, result: Result<(), String>) -> Check {
    Check { name, ok: result.is_ok(), message: result.err() }
}

async fn checks(settings: &Settings, pools: &HashMap<Env, PgPool>, outbox: &dyn Outbox) -> Vec<Check> {
    let http = reqwest::Client::new();
    let role = settings.postgrest_role().unwrap_or_default();
    let mut out = Vec::new();
    for env in Env::ALL {
        let database = match pools.get(&env) {
            Some(pool) if !role.is_empty() => check_database(pool, role).await,
            Some(_) => Err("PostgREST's role isn't set".into()),
            None => Err("not set".into()),
        };
        out.push(check(database_name(env), database));
        out.push(check(postgrest_name(env), check_postgrest(&http, settings.postgrest_url(env)).await));
    }
    if let Some(smtp) = &settings.shared_smtp {
        out.push(check(SHARED_SENDER.to_string(), outbox.check(smtp).await));
    }
    if let Some(storage) = &settings.storage {
        let answered = match crate::files::store_for(storage) {
            Ok(store) => tokio::time::timeout(CHECK_TIMEOUT, crate::files::check(store.as_ref())).await.unwrap_or_else(|_| Err("didn't answer in time".into())),
            Err(e) => Err(e),
        };
        out.push(check(FILE_STORAGE.to_string(), answered));
    }
    out
}

fn view_of(settings: &Settings, configured: bool, checks: Vec<Check>) -> SettingsView {
    SettingsView {
        configured,
        public_domain: settings.public_domain.clone(),
        shared_dev_url: mask_database_url(&settings.shared_dev_url),
        shared_production_url: mask_database_url(&settings.shared_production_url),
        postgrest_dev_url: settings.postgrest_dev_url.clone(),
        postgrest_production_url: settings.postgrest_production_url.clone(),
        postgrest_role: settings.postgrest_role.clone(),
        preview_origins: settings.preview_origins.clone(),
        shared_smtp: settings.shared_smtp.clone().map(|smtp| Smtp { password: String::new(), ..smtp }),
        storage: settings.storage.clone().map(|storage| StorageSettings { secret_access_key: String::new(), ..storage }),
        checks,
    }
}

/// The settings as they are now, checked live.
pub async fn view(relay: &Relay) -> SettingsView {
    match relay.live() {
        Ok(live) => view_of(&live.settings, true, checks(&live.settings, &live.shared, relay.outbox.as_ref()).await),
        Err(_) => view_of(&Settings::default(), false, Vec::new()),
    }
}

fn keep_or(given: Option<String>, stored: &str) -> String {
    given.map(|v| v.trim().to_string()).filter(|v| !v.is_empty()).unwrap_or_else(|| stored.to_string())
}

fn merge_sender(given: Option<Smtp>, stored: Option<Smtp>) -> Option<Smtp> {
    let Some(given) = given else { return stored };
    if given.host.trim().is_empty() {
        return None;
    }
    let password = if given.password.is_empty() { stored.map(|s| s.password).unwrap_or_default() } else { given.password };
    Some(Smtp { host: given.host.trim().to_string(), from: given.from.trim().to_string(), password, ..given })
}

fn merge_storage(given: Option<StorageSettings>, stored: Option<StorageSettings>) -> Option<StorageSettings> {
    let Some(given) = given else { return stored };
    if given.bucket.trim().is_empty() {
        return None;
    }
    let secret = if given.secret_access_key.is_empty() { stored.map(|s| s.secret_access_key).unwrap_or_default() } else { given.secret_access_key };
    Some(StorageSettings {
        endpoint: given.endpoint.trim().trim_end_matches('/').to_string(),
        region: given.region.trim().to_string(),
        bucket: given.bucket.trim().to_string(),
        access_key_id: given.access_key_id.trim().to_string(),
        secret_access_key: secret,
        path_style: given.path_style,
    })
}

fn quoted(identifier: &str) -> String {
    format!("\"{}\"", identifier.replace('"', "\"\""))
}

fn literal(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

/// Writes PostgREST's secret and claim key onto its role in this database, and the schemas it
/// serves, then asks it to reload.
async fn point_postgrest(pool: &PgPool, settings: &Settings, role: &str) -> Result<(), RelayError> {
    let database: String = sqlx::query_scalar("SELECT current_database()").fetch_one(pool).await?;
    let target = format!("ALTER ROLE {} IN DATABASE {}", quoted(role), quoted(&database));
    sqlx::query(&format!("{target} SET pgrst.jwt_secret = {}", literal(&settings.postgrest_jwt_secret))).execute(pool).await?;
    sqlx::query(&format!("{target} SET pgrst.jwt_role_claim_key = {}", literal(ROLE_CLAIM_KEY))).execute(pool).await?;
    postgrest::refresh_schemas(pool, role).await
}

/// Checks the new settings and, when every check passes, keeps and serves them. A refusal says
/// which checks failed, and the old settings keep serving.
pub async fn save(relay: &Relay, input: SettingsInput) -> Result<SettingsView, RelayError> {
    let stored = load(relay).await?.unwrap_or_default();
    let settings = Settings {
        public_domain: input.public_domain.trim().to_string(),
        shared_dev_url: keep_or(input.shared_dev_url, &stored.shared_dev_url),
        shared_production_url: keep_or(input.shared_production_url, &stored.shared_production_url),
        postgrest_dev_url: input.postgrest_dev_url.trim().trim_end_matches('/').to_string(),
        postgrest_production_url: input.postgrest_production_url.trim().trim_end_matches('/').to_string(),
        postgrest_role: keep_or(input.postgrest_role, &stored.postgrest_role),
        preview_origins: input.preview_origins.into_iter().map(|o| o.trim().to_string()).filter(|o| !o.is_empty()).collect(),
        postgrest_jwt_secret: Some(stored.postgrest_jwt_secret.clone()).filter(|s| !s.is_empty()).unwrap_or_else(new_signing_secret),
        shared_smtp: merge_sender(input.shared_smtp, stored.shared_smtp.clone()),
        storage: merge_storage(input.storage, stored.storage.clone()),
    };
    let mut pools = HashMap::new();
    for env in Env::ALL {
        let url = settings.shared_url(env);
        if url.is_empty() {
            continue;
        }
        if let Ok(pool) = PgPoolOptions::new().max_connections(1).acquire_timeout(CHECK_TIMEOUT).connect_lazy(url) {
            pools.insert(env, pool);
        }
    }
    let mut found = checks(&settings, &pools, relay.outbox.as_ref()).await;
    if settings.public_domain.is_empty() {
        found.push(Check { name: "public domain".into(), ok: false, message: Some("not set".into()) });
    }
    if found.iter().any(|c| !c.ok) {
        let failed: Vec<&str> = found.iter().filter(|c| !c.ok).map(|c| c.name.as_str()).collect();
        return Err(RelayError::Rejected {
            code: SETTINGS_INVALID,
            message: format!("These didn't work: {}", failed.join(", ")),
            details: json!({ "checks": found }),
        });
    }
    let role = settings.postgrest_role.clone();
    for pool in pools.values() {
        point_postgrest(pool, &settings, &role).await?;
    }
    let text = serde_json::to_string(&settings).map_err(|e| RelayError::Internal(e.to_string()))?;
    sqlx::query("INSERT INTO relay.settings (id, sealed) VALUES (true, $1) ON CONFLICT (id) DO UPDATE SET sealed = EXCLUDED.sealed, updated_at = now()")
        .bind(encrypt(&relay.encryption_key, &text))
        .execute(&relay.state)
        .await?;
    relay.replace(Live::connect(settings.clone())?);
    Ok(view_of(&settings, true, found))
}
