//! What Relay needs from its environment to start: its state database, the key its saved secrets
//! are sealed with, and the key Nomi calls it with. Everything else is set from Nomi's admin and
//! kept in its state (see `settings`).
//!
//! With `RELAY_PROJECT` set, Relay runs standalone: one project, one environment, in its own
//! database, as Nomi runs it on Shipyard next to each project. Its PostgREST and functions
//! runtime then come from the environment too.

use uuid::Uuid;

use crate::settings::FunctionsRuntime;
use crate::Env;

pub const ENV_DATABASE_URL: &str = "RELAY_DATABASE_URL";
pub const ENV_CONTROL_KEY: &str = "RELAY_CONTROL_KEY";
pub const ENV_ENCRYPTION_KEY: &str = "RELAY_ENCRYPTION_KEY";
pub const ENV_BIND: &str = "RELAY_BIND";
pub const DEFAULT_BIND: &str = "0.0.0.0:8080";
pub const ENV_PROJECT: &str = "RELAY_PROJECT";
pub const ENV_ENV: &str = "RELAY_ENV";
pub const ENV_POSTGREST_URL: &str = "RELAY_POSTGREST_URL";
pub const ENV_PGRST_PASSWORD: &str = "RELAY_PGRST_PASSWORD";
pub const ENV_FUNCTIONS_URL: &str = "RELAY_FUNCTIONS_URL";
pub const ENV_FUNCTIONS_SECRET: &str = "RELAY_FUNCTIONS_SECRET";
const KEY_BYTES: usize = 32;

#[derive(Clone)]
pub struct Config {
    /// Relay's own state: projects, environments, settings, sealed secrets.
    pub database_url: String,
    /// What the Nomi backend sends as `Authorization: Bearer …` on control-plane calls.
    pub control_key: String,
    pub encryption_key: [u8; KEY_BYTES],
    pub bind: String,
    /// Set when Relay serves one project on its own (`RELAY_PROJECT`).
    pub standalone: Option<Standalone>,
}

/// One project in one environment, in Relay's own database.
#[derive(Clone)]
pub struct Standalone {
    pub project: Uuid,
    /// `RELAY_ENV`; production when unset.
    pub env: Env,
    /// The PostgREST next to it (`RELAY_POSTGREST_URL`).
    pub postgrest_url: String,
    /// The password PostgREST logs in with as the authenticator (`RELAY_PGRST_PASSWORD`).
    pub pgrst_password: String,
    /// The functions runtime next to it, when both its address and secret are set.
    pub functions: Option<FunctionsRuntime>,
}

/// Shows where Relay listens without printing its keys or database password.
impl std::fmt::Debug for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Config").field("bind", &self.bind).field("standalone", &self.standalone.as_ref().map(|s| s.project)).finish_non_exhaustive()
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
        let standalone = match optional(ENV_PROJECT) {
            None => None,
            Some(project) => Some(Standalone {
                project: Uuid::parse_str(&project).map_err(|_| format!("{ENV_PROJECT} must be a project id (a UUID)"))?,
                env: match optional(ENV_ENV) {
                    None => Env::Production,
                    Some(env) => Env::parse(&env).ok_or_else(|| format!("{ENV_ENV} must be dev or production"))?,
                },
                postgrest_url: need(ENV_POSTGREST_URL)?.trim_end_matches('/').to_string(),
                pgrst_password: need(ENV_PGRST_PASSWORD)?,
                functions: match (optional(ENV_FUNCTIONS_URL), optional(ENV_FUNCTIONS_SECRET)) {
                    (Some(url), Some(secret)) => Some(FunctionsRuntime { url: url.trim_end_matches('/').to_string(), secret }),
                    _ => None,
                },
            }),
        };
        Ok(Self {
            database_url: need(ENV_DATABASE_URL)?,
            control_key: need(ENV_CONTROL_KEY)?,
            encryption_key,
            bind: optional(ENV_BIND).unwrap_or_else(|| DEFAULT_BIND.to_string()),
            standalone,
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

    const PROJECT: &str = "6dfd4200-0000-4000-8000-000000000001";

    fn standalone(extra: &[(&'static str, &'static str)]) -> Result<Config, String> {
        let mut all = values();
        all.extend_from_slice(extra);
        Config::from_lookup(lookup(all))
    }

    #[test]
    fn without_relay_project_it_is_not_standalone() {
        assert!(Config::from_lookup(lookup(values())).unwrap().standalone.is_none());
    }

    #[test]
    fn relay_project_reads_the_standalone_settings() {
        let base = [(ENV_PROJECT, PROJECT), (ENV_POSTGREST_URL, "http://postgrest:3000/"), (ENV_PGRST_PASSWORD, "pw")];
        let production = standalone(&base).unwrap().standalone.unwrap();
        assert_eq!(production.project.to_string(), PROJECT);
        assert_eq!(production.env, Env::Production);
        assert_eq!(production.postgrest_url, "http://postgrest:3000");
        assert_eq!(production.pgrst_password, "pw");
        assert!(production.functions.is_none());

        let mut dev = base.to_vec();
        dev.extend([(ENV_ENV, "dev"), (ENV_FUNCTIONS_URL, "http://functions:8000"), (ENV_FUNCTIONS_SECRET, "s")]);
        let dev = standalone(&dev).unwrap().standalone.unwrap();
        assert_eq!(dev.env, Env::Dev);
        assert_eq!(dev.functions, Some(FunctionsRuntime { url: "http://functions:8000".into(), secret: "s".into() }));

        let mut half = base.to_vec();
        half.push((ENV_FUNCTIONS_URL, "http://functions:8000"));
        assert!(standalone(&half).unwrap().standalone.unwrap().functions.is_none(), "a runtime needs its secret too");
    }

    #[test]
    fn standalone_needs_postgrest_and_its_password() {
        assert!(standalone(&[(ENV_PROJECT, PROJECT), (ENV_PGRST_PASSWORD, "pw")]).unwrap_err().contains(ENV_POSTGREST_URL));
        assert!(standalone(&[(ENV_PROJECT, PROJECT), (ENV_POSTGREST_URL, "http://p")]).unwrap_err().contains(ENV_PGRST_PASSWORD));
        let full = [(ENV_POSTGREST_URL, "http://p"), (ENV_PGRST_PASSWORD, "pw")];
        let mut bad_id = full.to_vec();
        bad_id.push((ENV_PROJECT, "bakery"));
        assert!(standalone(&bad_id).unwrap_err().contains(ENV_PROJECT));
        let mut bad_env = full.to_vec();
        bad_env.extend([(ENV_PROJECT, PROJECT), (ENV_ENV, "staging")]);
        assert!(standalone(&bad_env).unwrap_err().contains(ENV_ENV));
    }
}
