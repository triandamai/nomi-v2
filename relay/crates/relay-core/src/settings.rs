//! What Relay is set up with from Nomi's admin, kept sealed in its state database (only the
//! three variables in `config` come from the environment).

use serde::{Deserialize, Serialize};

use crate::Env;

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
}
