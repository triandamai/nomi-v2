//! Relay's settings, from its environment.

pub const ENV_DATABASE_URL: &str = "RELAY_DATABASE_URL";
pub const ENV_SHARED_DEV_URL: &str = "RELAY_SHARED_DEV_URL";
pub const ENV_SHARED_PRODUCTION_URL: &str = "RELAY_SHARED_PRODUCTION_URL";
pub const ENV_CONTROL_KEY: &str = "RELAY_CONTROL_KEY";
pub const ENV_ENCRYPTION_KEY: &str = "RELAY_ENCRYPTION_KEY";
pub const ENV_PUBLIC_DOMAIN: &str = "RELAY_PUBLIC_DOMAIN";
pub const ENV_BIND: &str = "RELAY_BIND";
pub const ENV_POSTGREST_DEV_URL: &str = "RELAY_POSTGREST_DEV_URL";
pub const ENV_POSTGREST_PRODUCTION_URL: &str = "RELAY_POSTGREST_PRODUCTION_URL";
pub const ENV_POSTGREST_JWT_SECRET: &str = "RELAY_POSTGREST_JWT_SECRET";
pub const ENV_POSTGREST_ROLE: &str = "RELAY_POSTGREST_ROLE";
pub const ENV_PREVIEW_ORIGINS: &str = "RELAY_PREVIEW_ORIGINS";
pub const DEFAULT_POSTGREST_ROLE: &str = "authenticator";
pub const DEFAULT_BIND: &str = "0.0.0.0:8080";
const KEY_BYTES: usize = 32;

#[derive(Clone)]
pub struct Config {
    /// Relay's own state: projects, environments, sealed secrets.
    pub database_url: String,
    /// The shared databases free projects live in, one per environment.
    pub shared_dev_url: String,
    pub shared_production_url: String,
    /// What the Nomi backend sends as `Authorization: Bearer …` on control-plane calls.
    pub control_key: String,
    pub encryption_key: [u8; KEY_BYTES],
    /// Apps reach their backend at `https://<slug>.<public_domain>`.
    pub public_domain: String,
    pub bind: String,
    /// The PostgREST serving each shared database.
    pub postgrest_dev_url: String,
    pub postgrest_production_url: String,
    /// Signs the short tokens the gateway sends PostgREST; only Relay and PostgREST know it.
    pub postgrest_jwt_secret: String,
    /// The role PostgREST logs in as and switches from.
    pub postgrest_role: String,
    /// Host suffixes of Nomi's previews, which may call a dev backend from the browser.
    pub preview_origins: Vec<String>,
}

/// Shows where Relay runs without printing its keys or database passwords.
impl std::fmt::Debug for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Config").field("public_domain", &self.public_domain).field("bind", &self.bind).finish_non_exhaustive()
    }
}

impl Config {
    pub fn from_env() -> Result<Self, String> {
        Self::from_lookup(|name| std::env::var(name).ok())
    }

    pub fn from_lookup(get: impl Fn(&str) -> Option<String>) -> Result<Self, String> {
        let need = |name: &str| get(name).map(|v| v.trim().to_string()).filter(|v| !v.is_empty()).ok_or_else(|| format!("{name} must be set"));
        let optional = |name: &str| get(name).map(|v| v.trim().to_string()).filter(|v| !v.is_empty());
        let key_hex = need(ENV_ENCRYPTION_KEY)?;
        let encryption_key: [u8; KEY_BYTES] = hex::decode(&key_hex)
            .ok()
            .and_then(|bytes| bytes.try_into().ok())
            .ok_or_else(|| format!("{ENV_ENCRYPTION_KEY} must be {KEY_BYTES} bytes of hex"))?;
        Ok(Self {
            database_url: need(ENV_DATABASE_URL)?,
            shared_dev_url: need(ENV_SHARED_DEV_URL)?,
            shared_production_url: need(ENV_SHARED_PRODUCTION_URL)?,
            control_key: need(ENV_CONTROL_KEY)?,
            encryption_key,
            public_domain: need(ENV_PUBLIC_DOMAIN)?,
            bind: optional(ENV_BIND).unwrap_or_else(|| DEFAULT_BIND.to_string()),
            postgrest_dev_url: need(ENV_POSTGREST_DEV_URL)?,
            postgrest_production_url: need(ENV_POSTGREST_PRODUCTION_URL)?,
            postgrest_jwt_secret: need(ENV_POSTGREST_JWT_SECRET)?,
            postgrest_role: optional(ENV_POSTGREST_ROLE).unwrap_or_else(|| DEFAULT_POSTGREST_ROLE.to_string()),
            preview_origins: optional(ENV_PREVIEW_ORIGINS)
                .map(|list| list.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect())
                .unwrap_or_default(),
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    fn values() -> Vec<(&'static str, &'static str)> {
        vec![
            (ENV_DATABASE_URL, "postgres://relay@localhost/relay"),
            (ENV_SHARED_DEV_URL, "postgres://relay@localhost/shared_dev"),
            (ENV_SHARED_PRODUCTION_URL, "postgres://relay@localhost/shared_prod"),
            (ENV_CONTROL_KEY, "control-key"),
            (ENV_ENCRYPTION_KEY, "0707070707070707070707070707070707070707070707070707070707070707"),
            (ENV_PUBLIC_DOMAIN, "api.relay.test"),
            (ENV_POSTGREST_DEV_URL, "http://postgrest-dev:3000"),
            (ENV_POSTGREST_PRODUCTION_URL, "http://postgrest-production:3000"),
            (ENV_POSTGREST_JWT_SECRET, "postgrest-secret"),
            (ENV_PREVIEW_ORIGINS, ".preview.nomi.app, .nomi.test"),
        ]
    }

    fn lookup(values: Vec<(&'static str, &'static str)>) -> impl Fn(&str) -> Option<String> {
        move |name| values.iter().find(|(k, _)| *k == name).map(|(_, v)| v.to_string())
    }

    #[test]
    fn config_reads_every_setting_and_defaults_the_bind_address() {
        let config = Config::from_lookup(lookup(values())).unwrap();
        assert_eq!(config.encryption_key, [7u8; 32]);
        assert_eq!(config.bind, DEFAULT_BIND);
        assert_eq!(config.public_domain, "api.relay.test");
        assert_eq!(config.postgrest_role, DEFAULT_POSTGREST_ROLE);
        assert_eq!(config.preview_origins, vec![".preview.nomi.app".to_string(), ".nomi.test".to_string()]);
    }

    #[test]
    fn a_missing_setting_or_a_short_key_is_named() {
        let without_key: Vec<_> = values().into_iter().filter(|(k, _)| *k != ENV_CONTROL_KEY).collect();
        assert!(Config::from_lookup(lookup(without_key)).unwrap_err().contains(ENV_CONTROL_KEY));
        let mut short = values();
        short.retain(|(k, _)| *k != ENV_ENCRYPTION_KEY);
        short.push((ENV_ENCRYPTION_KEY, "0707"));
        assert!(Config::from_lookup(lookup(short)).unwrap_err().contains(ENV_ENCRYPTION_KEY));
    }
}
