//! What Relay needs from its environment to start: its state database, the key its saved secrets
//! are sealed with, and the key Nomi calls it with. Everything else is set from Nomi's admin and
//! kept in its state (see `settings`).

pub const ENV_DATABASE_URL: &str = "RELAY_DATABASE_URL";
pub const ENV_CONTROL_KEY: &str = "RELAY_CONTROL_KEY";
pub const ENV_ENCRYPTION_KEY: &str = "RELAY_ENCRYPTION_KEY";
pub const ENV_BIND: &str = "RELAY_BIND";
pub const DEFAULT_BIND: &str = "0.0.0.0:8080";
const KEY_BYTES: usize = 32;

#[derive(Clone)]
pub struct Config {
    /// Relay's own state: projects, environments, settings, sealed secrets.
    pub database_url: String,
    /// What the Nomi backend sends as `Authorization: Bearer …` on control-plane calls.
    pub control_key: String,
    pub encryption_key: [u8; KEY_BYTES],
    pub bind: String,
}

/// Shows where Relay listens without printing its keys or database password.
impl std::fmt::Debug for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Config").field("bind", &self.bind).finish_non_exhaustive()
    }
}

impl Config {
    pub fn from_env() -> Result<Self, String> {
        Self::from_lookup(|name| std::env::var(name).ok())
    }

    pub fn from_lookup(get: impl Fn(&str) -> Option<String>) -> Result<Self, String> {
        let optional = |name: &str| get(name).map(|v| v.trim().to_string()).filter(|v| !v.is_empty());
        let need = |name: &str| optional(name).ok_or_else(|| format!("{name} must be set"));
        let key_hex = need(ENV_ENCRYPTION_KEY)?;
        let encryption_key: [u8; KEY_BYTES] = hex::decode(&key_hex)
            .ok()
            .and_then(|bytes| bytes.try_into().ok())
            .ok_or_else(|| format!("{ENV_ENCRYPTION_KEY} must be {KEY_BYTES} bytes of hex"))?;
        Ok(Self {
            database_url: need(ENV_DATABASE_URL)?,
            control_key: need(ENV_CONTROL_KEY)?,
            encryption_key,
            bind: optional(ENV_BIND).unwrap_or_else(|| DEFAULT_BIND.to_string()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn values() -> Vec<(&'static str, &'static str)> {
        vec![
            (ENV_DATABASE_URL, "postgres://relay@localhost/relay"),
            (ENV_CONTROL_KEY, "control-key"),
            (ENV_ENCRYPTION_KEY, "0707070707070707070707070707070707070707070707070707070707070707"),
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
    }

    #[test]
    fn three_variables_are_enough() {
        let config = Config::from_lookup(lookup(values())).unwrap();
        assert_eq!(config.database_url, "postgres://relay@localhost/relay");
        assert_eq!(config.control_key, "control-key");
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
