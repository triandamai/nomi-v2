//! What Relay is set up with from Nomi's admin, kept sealed in its state database (only the
//! three variables in `config` come from the environment).

use serde::{Deserialize, Serialize};

use crate::config::Standalone;
use crate::Env;

/// The role PostgREST logs in as next to a standalone Relay.
pub const AUTHENTICATOR: &str = "authenticator";
/// What a standalone Relay's PostgREST JWT secret is derived under, so it's the same after every
/// restart without being stored.
const JWT_SECRET_LABEL: &str = "pgrst.jwt_secret";

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Settings {
    /// Apps reach their backend at `https://<slug>.<public_domain>`.
    pub public_domain: String,
    /// The shared databases free projects live in, one per environment.
    pub shared_dev_url: String,
    pub shared_production_url: String,
    /// The PostgREST serving each shared database.
    pub postgrest_dev_url: String,
    pub postgrest_production_url: String,
    /// The role PostgREST logs in as and switches from; empty until set.
    pub postgrest_role: String,
    /// Host suffixes of Nomi's previews, which may call a dev backend from the browser.
    pub preview_origins: Vec<String>,
    /// Signs the gateway's tokens for PostgREST. Relay makes it and writes it where PostgREST
    /// reads it (its role's `pgrst.jwt_secret`); nobody types it.
    pub postgrest_jwt_secret: String,
    /// Nomi's shared sender for sign-in codes; projects without their own SMTP use it.
    #[serde(default)]
    pub shared_smtp: Option<Smtp>,
    /// The S3 bucket every project's files go in, each under its own prefix; without it, storage
    /// answers that it isn't set up.
    #[serde(default)]
    pub storage: Option<StorageSettings>,
    /// The functions runtime (`relay/functions-runtime`) projects' functions run in; without it,
    /// functions answer that they aren't set up.
    #[serde(default)]
    pub functions: Option<FunctionsRuntime>,
}

/// Where the functions runtime is and the secret it and Relay share (its
/// `RELAY_FUNCTIONS_SECRET`).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct FunctionsRuntime {
    pub url: String,
    #[serde(default)]
    pub secret: String,
}

/// An S3-compatible bucket (AWS, R2, MinIO, …).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct StorageSettings {
    /// Empty for AWS itself.
    #[serde(default)]
    pub endpoint: String,
    pub region: String,
    pub bucket: String,
    pub access_key_id: String,
    #[serde(default)]
    pub secret_access_key: String,
    /// `endpoint/bucket/key` instead of `bucket.endpoint/key` (MinIO and most self-hosted).
    #[serde(default)]
    pub path_style: bool,
}

/// An SMTP server email goes out through: Relay's shared sender, or a project's own.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Smtp {
    pub host: String,
    pub port: u16,
    #[serde(default)]
    pub user: String,
    #[serde(default)]
    pub password: String,
    /// `Name <address>`.
    pub from: String,
    /// `starttls` (default), `tls` or `none`.
    #[serde(default)]
    pub security: String,
}

impl Settings {
    pub fn shared_url(&self, env: Env) -> &str {
        match env {
            Env::Dev => &self.shared_dev_url,
            Env::Production => &self.shared_production_url,
        }
    }

    pub fn postgrest_url(&self, env: Env) -> &str {
        match env {
            Env::Dev => &self.postgrest_dev_url,
            Env::Production => &self.postgrest_production_url,
        }
    }

    pub fn postgrest_role(&self) -> Option<&str> {
        Some(self.postgrest_role.as_str()).filter(|r| !r.is_empty())
    }

    /// A standalone Relay's settings: what was saved through the control API (public domain,
    /// preview origins, sender, storage), with everything about where its data lives taken from
    /// its environment. Both environments' database is its own, served by its own PostgREST.
    pub fn standalone(saved: Settings, database_url: &str, encryption_key: &[u8; 32], standalone: &Standalone) -> Settings {
        Settings {
            shared_dev_url: database_url.to_string(),
            shared_production_url: database_url.to_string(),
            postgrest_dev_url: standalone.postgrest_url.clone(),
            postgrest_production_url: standalone.postgrest_url.clone(),
            postgrest_role: AUTHENTICATOR.to_string(),
            postgrest_jwt_secret: crate::keys::role_password(encryption_key, JWT_SECRET_LABEL),
            functions: standalone.functions.clone().or(saved.functions.clone()),
            ..saved
        }
    }
}

/// `postgres://user@host:port/db`: where a database is, without its password or options.
pub fn mask_database_url(url: &str) -> String {
    let Ok(mut parsed) = url::Url::parse(url) else { return String::new() };
    if parsed.cannot_be_a_base() || parsed.host().is_none() {
        return String::new();
    }
    let _ = parsed.set_password(None);
    parsed.set_query(None);
    parsed.set_fragment(None);
    parsed.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_database_url_is_shown_without_its_password() {
        assert_eq!(mask_database_url("postgres://relay:s3cret@db.internal:5432/shared_dev?sslmode=require"), "postgres://relay@db.internal:5432/shared_dev");
        assert_eq!(mask_database_url("postgres://relay@db/x"), "postgres://relay@db/x");
        assert_eq!(mask_database_url("not a url"), "");
        assert_eq!(mask_database_url(""), "");
    }

    #[test]
    fn each_environment_has_its_database_and_postgrest() {
        let settings = Settings {
            shared_dev_url: "postgres://d".into(),
            shared_production_url: "postgres://p".into(),
            postgrest_dev_url: "http://pd".into(),
            postgrest_production_url: "http://pp".into(),
            ..Settings::default()
        };
        assert_eq!(settings.shared_url(Env::Dev), "postgres://d");
        assert_eq!(settings.postgrest_url(Env::Production), "http://pp");
        assert_eq!(settings.postgrest_role(), None);
        assert_eq!(Settings { postgrest_role: "authenticator".into(), ..Settings::default() }.postgrest_role(), Some("authenticator"));
    }

    #[test]
    fn standalone_settings_point_everything_at_its_own_database() {
        let standalone = Standalone {
            project: uuid::Uuid::nil(),
            env: Env::Production,
            postgrest_url: "http://postgrest:3000".into(),
            pgrst_password: "pw".into(),
            functions: None,
        };
        let saved = Settings {
            public_domain: "app-backend.apps.test".into(),
            shared_dev_url: "postgres://elsewhere".into(),
            preview_origins: vec![".webcontainer-api.io".into()],
            functions: Some(FunctionsRuntime { url: "http://saved".into(), secret: "s".into() }),
            ..Settings::default()
        };
        let key = [7u8; 32];
        let settings = Settings::standalone(saved.clone(), "postgres://own/db", &key, &standalone);
        assert_eq!(settings.shared_url(Env::Dev), "postgres://own/db");
        assert_eq!(settings.shared_url(Env::Production), "postgres://own/db");
        assert_eq!(settings.postgrest_url(Env::Dev), "http://postgrest:3000");
        assert_eq!(settings.postgrest_url(Env::Production), "http://postgrest:3000");
        assert_eq!(settings.postgrest_role(), Some(AUTHENTICATOR));
        assert_eq!(settings.public_domain, "app-backend.apps.test");
        assert_eq!(settings.preview_origins, vec![".webcontainer-api.io".to_string()]);
        assert_eq!(settings.functions, saved.functions, "a runtime saved earlier is kept when the environment names none");

        let again = Settings::standalone(Settings::default(), "postgres://own/db", &key, &standalone);
        assert_eq!(settings.postgrest_jwt_secret, again.postgrest_jwt_secret, "the same after a restart");
        assert!(settings.postgrest_jwt_secret.len() >= 32);
        assert_ne!(Settings::standalone(Settings::default(), "postgres://own/db", &[8u8; 32], &standalone).postgrest_jwt_secret, settings.postgrest_jwt_secret);

        let with_runtime = Standalone { functions: Some(FunctionsRuntime { url: "http://functions:8000".into(), secret: "f".into() }), ..standalone };
        assert_eq!(Settings::standalone(saved, "postgres://own/db", &key, &with_runtime).functions.unwrap().url, "http://functions:8000");
    }
}
