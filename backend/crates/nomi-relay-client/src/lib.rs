//! Nomi's client for Nomi Relay's control plane (see relay/README.md): making and deleting a
//! project's backend, applying its migrations, reading its schema and types, and deploying its
//! functions. Only the Nomi server calls it, with the service credential `RELAY_CONTROL_KEY`;
//! apps talk to Relay's gateway with their own keys instead.

use std::time::Duration;

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

pub const ENV_CONTROL_URL: &str = "RELAY_CONTROL_URL";
pub const ENV_CONTROL_KEY: &str = "RELAY_CONTROL_KEY";
/// The project secrets Relay keeps for sign-in: forwarded when an admin sets them.
pub fn is_sign_in_secret(name: &str) -> bool {
    name == "NOMI_SMTP_PASSWORD" || name.starts_with("NOMI_OAUTH_")
}

/// Most control-plane calls are quick lookups or small writes.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
/// Applying migrations or building functions can take a while.
const LONG_REQUEST_TIMEOUT: Duration = Duration::from_secs(300);
/// How much of an unparseable answer is kept for the error.
const MAX_RAW_ERROR_CHARS: usize = 300;

/// How Nomi reaches Relay, from the server's environment.
#[derive(Debug, Clone)]
pub struct RelayConfig {
    /// `https://<relay>`, without a trailing slash.
    pub control_url: String,
    pub control_key: String,
}

impl RelayConfig {
    /// `None` when Relay isn't set up: projects are then built on Drizzle.
    pub fn from_env() -> Option<Self> {
        Self::from_values(std::env::var(ENV_CONTROL_URL).ok().as_deref(), std::env::var(ENV_CONTROL_KEY).ok().as_deref())
    }

    pub fn from_values(control_url: Option<&str>, control_key: Option<&str>) -> Option<Self> {
        let url = control_url.map(str::trim).filter(|v| !v.is_empty())?;
        let key = control_key.map(str::trim).filter(|v| !v.is_empty())?;
        Some(Self { control_url: url.trim_end_matches('/').to_string(), control_key: key.to_string() })
    }
}

/// What went wrong talking to Relay.
#[derive(Debug, Clone, PartialEq)]
pub enum RelayError {
    /// Relay refused the request (a migration failed, a lint rule, a conflict): its own words,
    /// and where it went wrong when it says (`{ "file", "line" }` for a migration).
    Rejected { status: u16, code: String, message: String, details: Value },
    NotFound(String),
    /// The control key is wrong or revoked.
    Unauthorized(String),
    /// Couldn't reach it, or it failed (5xx).
    Unavailable(String),
}

impl std::fmt::Display for RelayError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RelayError::Rejected { message, .. } => write!(f, "Relay refused it: {message}"),
            RelayError::NotFound(message) => write!(f, "Relay has no such thing: {message}"),
            RelayError::Unauthorized(message) => write!(f, "Relay rejected Nomi's key: {message}"),
            RelayError::Unavailable(message) => write!(f, "couldn't reach Relay: {message}"),
        }
    }
}

impl std::error::Error for RelayError {}

/// Relay's `{ data, error }` envelope, unwrapped.
fn unwrap_envelope(status: u16, body: &str) -> Result<Value, RelayError> {
    let parsed: Value = serde_json::from_str(body).unwrap_or_else(|_| json!({ "raw": body.chars().take(MAX_RAW_ERROR_CHARS).collect::<String>() }));
    let error = parsed.get("error").filter(|e| !e.is_null());
    let message = error
        .and_then(|e| e.get("message"))
        .and_then(Value::as_str)
        .map(str::to_string)
        .unwrap_or_else(|| body.chars().take(MAX_RAW_ERROR_CHARS).collect());
    match status {
        200..=299 => Ok(parsed.get("data").cloned().unwrap_or(parsed)),
        401 | 403 => Err(RelayError::Unauthorized(message)),
        404 => Err(RelayError::NotFound(message)),
        400..=499 => Err(RelayError::Rejected {
            status,
            code: error.and_then(|e| e.get("code")).and_then(Value::as_str).unwrap_or_default().to_string(),
            message,
            details: error.and_then(|e| e.get("details")).cloned().unwrap_or(Value::Null),
        }),
        _ => Err(RelayError::Unavailable(message)),
    }
}

/// One file of a project's `nomi/migrations/` or `nomi/functions/`.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SourceFile {
    pub path: String,
    pub content: String,
}

/// One environment of a project's backend: where the app reaches it and its keys. The anon key
/// is public; the service key is for the app's server only.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct EnvironmentKeys {
    pub url: String,
    pub anon_key: String,
    pub service_key: String,
}

/// A project's backend on Relay.
#[derive(Debug, Clone, Deserialize)]
pub struct Backend {
    pub id: Uuid,
    /// Keyed by `dev` and `production`.
    pub environments: BTreeMap<String, EnvironmentKeys>,
}

impl Backend {
    pub fn environment(&self, env: &str) -> Option<&EnvironmentKeys> {
        self.environments.get(env)
    }
}

pub struct Relay<'a> {
    pub http: &'a reqwest::Client,
    pub config: &'a RelayConfig,
}

impl<'a> Relay<'a> {
    pub fn new(http: &'a reqwest::Client, config: &'a RelayConfig) -> Self {
        Self { http, config }
    }

    async fn call(&self, method: reqwest::Method, path: &str, body: Option<Value>, timeout: Duration) -> Result<Value, RelayError> {
        let mut request = self
            .http
            .request(method, format!("{}{path}", self.config.control_url))
            .timeout(timeout)
            .bearer_auth(&self.config.control_key)
            .header("accept", "application/json");
        if let Some(body) = body {
            request = request.json(&body);
        }
        let response = request.send().await.map_err(|e| RelayError::Unavailable(e.to_string()))?;
        let status = response.status().as_u16();
        let body = response.text().await.unwrap_or_default();
        unwrap_envelope(status, &body)
    }

    fn project_path(project_id: Uuid) -> String {
        format!("/control/projects/{project_id}")
    }

    /// `env` is `dev` or `production`; it's one of Relay's names, so it's used as is.
    fn env_path(project_id: Uuid, env: &str) -> String {
        format!("{}/environments/{env}", Self::project_path(project_id))
    }

    /// Makes the project's backend (database objects, `nomi` schema, keys), or does nothing if it
    /// exists. Idempotent: the same keys every time.
    pub async fn ensure_backend(&self, project_id: Uuid, name: &str) -> Result<Backend, RelayError> {
        let data = self.call(reqwest::Method::PUT, &Self::project_path(project_id), Some(json!({ "name": name })), REQUEST_TIMEOUT).await?;
        serde_json::from_value(data.clone()).map_err(|e| RelayError::Unavailable(format!("Relay's answer wasn't a backend ({e}): {data}")))
    }

    /// The project's backend as it is, changing nothing: `NotFound` when it has none.
    pub async fn backend(&self, project_id: Uuid) -> Result<Backend, RelayError> {
        let data = self.call(reqwest::Method::GET, &Self::project_path(project_id), None, REQUEST_TIMEOUT).await?;
        serde_json::from_value(data.clone()).map_err(|e| RelayError::Unavailable(format!("Relay's answer wasn't a backend ({e}): {data}")))
    }

    /// Relay's settings (database URLs without passwords) and whether each part answers.
    pub async fn settings(&self) -> Result<Value, RelayError> {
        self.call(reqwest::Method::GET, "/control/settings", None, REQUEST_TIMEOUT).await
    }

    /// Checks and saves Relay's settings; a refusal names each check that failed (in `details`).
    pub async fn save_settings(&self, settings: &Value) -> Result<Value, RelayError> {
        self.call(reqwest::Method::PUT, "/control/settings", Some(settings.clone()), REQUEST_TIMEOUT).await
    }

    /// Drops exactly the project's database objects and rows.
    pub async fn delete_backend(&self, project_id: Uuid) -> Result<Value, RelayError> {
        self.call(reqwest::Method::DELETE, &Self::project_path(project_id), None, REQUEST_TIMEOUT).await
    }

    /// Lints and applies the migrations `env` doesn't have yet. Answers what ran.
    pub async fn apply_migrations(&self, project_id: Uuid, env: &str, files: &[SourceFile]) -> Result<Value, RelayError> {
        let path = format!("{}/migrations", Self::env_path(project_id, env));
        self.call(reqwest::Method::POST, &path, Some(json!({ "files": files })), LONG_REQUEST_TIMEOUT).await
    }

    /// The backend's tables, columns, policies, buckets, auth providers and functions.
    pub async fn describe(&self, project_id: Uuid, env: &str) -> Result<Value, RelayError> {
        self.call(reqwest::Method::GET, &format!("{}/schema", Self::env_path(project_id, env)), None, REQUEST_TIMEOUT).await
    }

    /// `src/lib/nomi.types.ts` for `env`'s schema.
    pub async fn types(&self, project_id: Uuid, env: &str) -> Result<String, RelayError> {
        let data = self.call(reqwest::Method::GET, &format!("{}/types", Self::env_path(project_id, env)), None, REQUEST_TIMEOUT).await?;
        data.get("types")
            .and_then(Value::as_str)
            .map(str::to_string)
            .ok_or_else(|| RelayError::Unavailable(format!("Relay answered without types: {data}")))
    }

    /// Replaces `env`'s sign-in settings (`nomi/backend.json`'s `auth`).
    pub async fn put_auth(&self, project_id: Uuid, env: &str, auth: &Value) -> Result<Value, RelayError> {
        self.call(reqwest::Method::PUT, &format!("{}/auth", Self::env_path(project_id, env)), Some(auth.clone()), REQUEST_TIMEOUT).await
    }

    /// Replaces `env`'s buckets (`nomi/backend.json`'s `storage`).
    pub async fn put_storage(&self, project_id: Uuid, env: &str, storage: &Value) -> Result<Value, RelayError> {
        self.call(reqwest::Method::PUT, &format!("{}/storage", Self::env_path(project_id, env)), Some(storage.clone()), REQUEST_TIMEOUT).await
    }

    /// Replaces `env`'s realtime tables (`nomi/backend.json`'s `realtime`).
    pub async fn put_realtime(&self, project_id: Uuid, env: &str, tables: &Value) -> Result<Value, RelayError> {
        self.call(reqwest::Method::PUT, &format!("{}/realtime", Self::env_path(project_id, env)), Some(json!({ "tables": tables })), REQUEST_TIMEOUT).await
    }

    /// `env`'s sign-in settings, without secrets, and `missing_secrets`.
    pub async fn auth(&self, project_id: Uuid, env: &str) -> Result<Value, RelayError> {
        self.call(reqwest::Method::GET, &format!("{}/auth", Self::env_path(project_id, env)), None, REQUEST_TIMEOUT).await
    }

    /// Sets (or, with `None`, removes) one of the project's sign-in secrets.
    pub async fn put_auth_secret(&self, project_id: Uuid, name: &str, value: Option<&str>) -> Result<Value, RelayError> {
        let path = format!("{}/auth/secrets/{name}", Self::project_path(project_id));
        match value {
            Some(value) => self.call(reqwest::Method::PUT, &path, Some(json!({ "value": value })), REQUEST_TIMEOUT).await,
            None => self.call(reqwest::Method::DELETE, &path, None, REQUEST_TIMEOUT).await,
        }
    }

    /// Builds and deploys the project's functions to `env`.
    pub async fn deploy_functions(&self, project_id: Uuid, env: &str, files: &[SourceFile]) -> Result<Value, RelayError> {
        let path = format!("{}/functions", Self::env_path(project_id, env));
        self.call(reqwest::Method::POST, &path, Some(json!({ "files": files })), LONG_REQUEST_TIMEOUT).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use wiremock::matchers::{body_json, header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn config(base: &str) -> RelayConfig {
        RelayConfig { control_url: base.to_string(), control_key: "relay_control_key".into() }
    }

    fn envelope(data: serde_json::Value) -> ResponseTemplate {
        ResponseTemplate::new(200).set_body_json(json!({ "data": data, "error": null }))
    }

    #[test]
    fn config_needs_both_url_and_key() {
        assert!(RelayConfig::from_values(Some("https://relay.test/"), None).is_none());
        assert!(RelayConfig::from_values(None, Some("k")).is_none());
        assert!(RelayConfig::from_values(Some("  "), Some("k")).is_none());
        let config = RelayConfig::from_values(Some("https://relay.test/"), Some(" k ")).unwrap();
        assert_eq!(config.control_url, "https://relay.test");
        assert_eq!(config.control_key, "k");
    }

    #[tokio::test]
    async fn ensure_backend_puts_the_project_and_answers_its_keys() {
        let server = MockServer::start().await;
        let project = Uuid::new_v4();
        Mock::given(method("PUT"))
            .and(path(format!("/control/projects/{project}")))
            .and(header("authorization", "Bearer relay_control_key"))
            .and(body_json(json!({ "name": "Todo" })))
            .respond_with(envelope(json!({
                "id": project,
                "slug": "todo-4f2c1a",
                "tier": "free",
                "environments": {
                    "dev": { "url": "https://todo-4f2c1a-dev.api.relay.test", "anon_key": "dev-anon", "service_key": "dev-service" },
                    "production": { "url": "https://todo-4f2c1a.api.relay.test", "anon_key": "prod-anon", "service_key": "prod-service" }
                }
            })))
            .expect(1)
            .mount(&server)
            .await;
        let http = reqwest::Client::new();
        let config = config(&server.uri());
        let backend = Relay::new(&http, &config).ensure_backend(project, "Todo").await.unwrap();
        assert_eq!(backend.id, project);
        let production = backend.environment("production").unwrap();
        assert_eq!(production.url, "https://todo-4f2c1a.api.relay.test");
        assert_eq!(production.anon_key, "prod-anon");
        assert_eq!(production.service_key, "prod-service");
        assert!(backend.environment("staging").is_none());
    }

    #[tokio::test]
    async fn backend_reads_the_keys_and_a_missing_one_is_not_found() {
        let server = MockServer::start().await;
        let (made, missing) = (Uuid::new_v4(), Uuid::new_v4());
        Mock::given(method("GET"))
            .and(path(format!("/control/projects/{made}")))
            .and(header("authorization", "Bearer relay_control_key"))
            .respond_with(envelope(json!({
                "id": made,
                "environments": { "dev": { "url": "https://todo-dev.api.relay.test", "anon_key": "dev-anon", "service_key": "dev-service" } }
            })))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path(format!("/control/projects/{missing}")))
            .respond_with(ResponseTemplate::new(404).set_body_json(json!({ "data": null, "error": { "code": "NOT_FOUND", "message": "no backend for this project", "details": null } })))
            .mount(&server)
            .await;
        let http = reqwest::Client::new();
        let config = config(&server.uri());
        let relay = Relay::new(&http, &config);
        assert_eq!(relay.backend(made).await.unwrap().environment("dev").unwrap().anon_key, "dev-anon");
        assert!(matches!(relay.backend(missing).await, Err(RelayError::NotFound(_))));
    }

    #[tokio::test]
    async fn sign_in_settings_and_secrets_go_to_the_project() {
        let server = MockServer::start().await;
        let project = Uuid::new_v4();
        let auth = json!({ "methods": ["email_code", "github"], "github": { "client_id": "gh" } });
        Mock::given(method("PUT")).and(path(format!("/control/projects/{project}/environments/dev/auth"))).and(body_json(auth.clone())).respond_with(envelope(json!({ "saved": true }))).expect(1).mount(&server).await;
        Mock::given(method("GET"))
            .and(path(format!("/control/projects/{project}/environments/dev/auth")))
            .respond_with(envelope(json!({ "methods": ["email_code", "github"], "missing_secrets": ["NOMI_OAUTH_GITHUB_SECRET"] })))
            .mount(&server)
            .await;
        Mock::given(method("PUT")).and(path(format!("/control/projects/{project}/auth/secrets/NOMI_OAUTH_GITHUB_SECRET"))).and(body_json(json!({ "value": "s3cret" }))).respond_with(envelope(json!({}))).expect(1).mount(&server).await;
        Mock::given(method("DELETE")).and(path(format!("/control/projects/{project}/auth/secrets/NOMI_OAUTH_GITHUB_SECRET"))).respond_with(envelope(json!({}))).expect(1).mount(&server).await;
        let http = reqwest::Client::new();
        let config = config(&server.uri());
        let relay = Relay::new(&http, &config);
        relay.put_auth(project, "dev", &auth).await.unwrap();
        assert_eq!(relay.auth(project, "dev").await.unwrap()["missing_secrets"][0], "NOMI_OAUTH_GITHUB_SECRET");
        relay.put_auth_secret(project, "NOMI_OAUTH_GITHUB_SECRET", Some("s3cret")).await.unwrap();
        relay.put_auth_secret(project, "NOMI_OAUTH_GITHUB_SECRET", None).await.unwrap();
    }

    #[tokio::test]
    async fn migrations_go_to_the_environment_and_come_back_as_relay_said() {
        let server = MockServer::start().await;
        let project = Uuid::new_v4();
        Mock::given(method("POST"))
            .and(path(format!("/control/projects/{project}/environments/dev/migrations")))
            .and(body_json(json!({ "files": [{ "path": "nomi/migrations/0001_init.sql", "content": "create table t (id int);" }] })))
            .respond_with(envelope(json!({ "applied": ["0001_init.sql"] })))
            .mount(&server)
            .await;
        let http = reqwest::Client::new();
        let config = config(&server.uri());
        let files = [SourceFile { path: "nomi/migrations/0001_init.sql".into(), content: "create table t (id int);".into() }];
        let answer = Relay::new(&http, &config).apply_migrations(project, "dev", &files).await.unwrap();
        assert_eq!(answer, json!({ "applied": ["0001_init.sql"] }));
    }

    #[tokio::test]
    async fn a_failed_migration_keeps_relay_s_words_and_where_it_failed() {
        let server = MockServer::start().await;
        let project = Uuid::new_v4();
        Mock::given(method("POST"))
            .and(path(format!("/control/projects/{project}/environments/dev/migrations")))
            .respond_with(ResponseTemplate::new(422).set_body_json(json!({
                "data": null,
                "error": { "code": "MIGRATION_FAILED", "message": "relation \"task\" already exists", "details": { "file": "0002_task.sql", "line": 3 } }
            })))
            .mount(&server)
            .await;
        let http = reqwest::Client::new();
        let config = config(&server.uri());
        let err = Relay::new(&http, &config).apply_migrations(project, "dev", &[]).await.unwrap_err();
        assert_eq!(
            err,
            RelayError::Rejected {
                status: 422,
                code: "MIGRATION_FAILED".into(),
                message: "relation \"task\" already exists".into(),
                details: json!({ "file": "0002_task.sql", "line": 3 }),
            }
        );
    }

    #[tokio::test]
    async fn types_and_schema_are_read_per_environment() {
        let server = MockServer::start().await;
        let project = Uuid::new_v4();
        Mock::given(method("GET"))
            .and(path(format!("/control/projects/{project}/environments/production/types")))
            .respond_with(envelope(json!({ "types": "export interface Database {}" })))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path(format!("/control/projects/{project}/environments/dev/schema")))
            .respond_with(envelope(json!({ "tables": [] })))
            .mount(&server)
            .await;
        let http = reqwest::Client::new();
        let config = config(&server.uri());
        let relay = Relay::new(&http, &config);
        assert_eq!(relay.types(project, "production").await.unwrap(), "export interface Database {}");
        assert_eq!(relay.describe(project, "dev").await.unwrap(), json!({ "tables": [] }));
    }

    #[test]
    fn errors_say_whose_fault_it_is() {
        assert!(matches!(unwrap_envelope(401, r#"{"data":null,"error":{"code":"UNAUTHORIZED","message":"bad key"}}"#), Err(RelayError::Unauthorized(_))));
        assert!(matches!(unwrap_envelope(404, r#"{"data":null,"error":{"code":"NOT_FOUND","message":"no such project"}}"#), Err(RelayError::NotFound(_))));
        assert!(matches!(unwrap_envelope(502, "upstream down"), Err(RelayError::Unavailable(_))));
    }

    #[tokio::test]
    async fn an_unreachable_relay_is_unavailable() {
        let http = reqwest::Client::new();
        let config = config("http://127.0.0.1:9");
        assert!(matches!(Relay::new(&http, &config).describe(Uuid::new_v4(), "dev").await, Err(RelayError::Unavailable(_))));
    }
}
