//! Shipyard (https://shipyard.trian.space/docs/nomi) is where Nomi projects are published. This
//! crate talks to its integration API: organizations and apps, publishing a project's source
//! with a manifest, following publishes and deployments, logs and resource usage, the app's
//! development database and its webhooks.
//!
//! Two keys: `SHIPYARD_KEY` (scope `manage`) for everything, and `SHIPYARD_REVEAL_KEY` (adds
//! `secrets:reveal`) used only by server code that shows a person their own database login.
//! Secret values never reach a model.

pub mod bundle;
pub mod manifest;
pub mod signature;

use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

pub const ENV_URL: &str = "SHIPYARD_URL";
pub const ENV_KEY: &str = "SHIPYARD_KEY";
pub const ENV_REVEAL_KEY: &str = "SHIPYARD_REVEAL_KEY";
pub const ENV_WEBHOOK_SECRET: &str = "SHIPYARD_WEBHOOK_SECRET";

/// How Nomi reaches Shipyard, from the server's environment.
#[derive(Debug, Clone)]
pub struct ShipyardConfig {
    /// `https://<shipyard>/openapi/v1/integration`
    pub base_url: String,
    pub key: String,
    pub reveal_key: Option<String>,
    /// Set when the webhook endpoint was registered by hand; otherwise Nomi registers its own.
    pub webhook_secret: Option<String>,
}

fn env(name: &str) -> Option<String> {
    std::env::var(name).ok().map(|v| v.trim().to_string()).filter(|v| !v.is_empty())
}

impl ShipyardConfig {
    /// `None` when Shipyard isn't set up: publishing is then unavailable.
    pub fn from_env() -> Option<Self> {
        Some(Self {
            base_url: env(ENV_URL)?.trim_end_matches('/').to_string(),
            key: env(ENV_KEY)?,
            reveal_key: env(ENV_REVEAL_KEY),
            webhook_secret: env(ENV_WEBHOOK_SECRET),
        })
    }
}

/// What went wrong talking to Shipyard.
#[derive(Debug, Clone, PartialEq)]
pub enum ShipyardError {
    /// Shipyard refused the request (`BAD_REQUEST`, `CONFLICT`, ...); its own words.
    Rejected { status: u16, code: String, message: String },
    NotFound(String),
    /// The key is wrong, revoked, or lacks the scope.
    Unauthorized(String),
    /// Couldn't reach it, or it failed (5xx).
    Unavailable(String),
}

impl std::fmt::Display for ShipyardError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShipyardError::Rejected { message, .. } => write!(f, "Shipyard refused it: {message}"),
            ShipyardError::NotFound(message) => write!(f, "Shipyard has no such thing: {message}"),
            ShipyardError::Unauthorized(message) => write!(f, "Shipyard rejected Nomi's key: {message}"),
            ShipyardError::Unavailable(message) => write!(f, "couldn't reach Shipyard: {message}"),
        }
    }
}

impl std::error::Error for ShipyardError {}

/// Shipyard's `{ data, error }` envelope, unwrapped.
fn unwrap_envelope(status: u16, body: &str) -> Result<Value, ShipyardError> {
    let parsed: Value = serde_json::from_str(body).unwrap_or_else(|_| json!({ "raw": body.chars().take(500).collect::<String>() }));
    let error = parsed.get("error").filter(|e| !e.is_null());
    let message = error
        .and_then(|e| e.get("message"))
        .and_then(Value::as_str)
        .map(str::to_string)
        .unwrap_or_else(|| body.chars().take(300).collect());
    let code = error.and_then(|e| e.get("code")).and_then(Value::as_str).unwrap_or_default().to_string();
    match status {
        200..=299 => Ok(parsed.get("data").cloned().unwrap_or(parsed)),
        401 | 403 => Err(ShipyardError::Unauthorized(message)),
        404 => Err(ShipyardError::NotFound(message)),
        400..=499 => Err(ShipyardError::Rejected { status, code, message }),
        _ => Err(ShipyardError::Unavailable(message)),
    }
}

/// A publish Shipyard accepted (`202`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishAccepted {
    pub publish_id: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub warnings: Vec<String>,
}

pub struct Shipyard<'a> {
    pub http: &'a reqwest::Client,
    pub config: &'a ShipyardConfig,
}

fn seg(value: &str) -> String {
    value
        .bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => (b as char).to_string(),
            _ => format!("%{b:02X}"),
        })
        .collect()
}

impl<'a> Shipyard<'a> {
    pub fn new(http: &'a reqwest::Client, config: &'a ShipyardConfig) -> Self {
        Self { http, config }
    }

    async fn send(&self, request: reqwest::RequestBuilder, key: &str) -> Result<Value, ShipyardError> {
        let response = request
            .bearer_auth(key)
            .header("accept", "application/json")
            .send()
            .await
            .map_err(|e| ShipyardError::Unavailable(e.to_string()))?;
        let status = response.status().as_u16();
        let body = response.text().await.unwrap_or_default();
        unwrap_envelope(status, &body)
    }

    fn url(&self, path: &str) -> String {
        format!("{}{path}", self.config.base_url)
    }

    async fn call(&self, method: reqwest::Method, path: &str, body: Option<Value>) -> Result<Value, ShipyardError> {
        let mut request = self.http.request(method, self.url(path)).timeout(Duration::from_secs(30));
        if let Some(body) = body {
            request = request.json(&body);
        }
        self.send(request, &self.config.key).await
    }

    fn app_path(org: &str, app: &str) -> String {
        format!("/orgs/{}/apps/{}", seg(org), seg(app))
    }

    fn env_path(org: &str, app: &str, env: &str) -> String {
        format!("{}/envs/{}", Self::app_path(org, app), seg(env))
    }

    /// Creates the organization, or updates its name. Idempotent.
    pub async fn put_org(&self, org: &str, name: &str) -> Result<Value, ShipyardError> {
        self.call(reqwest::Method::PUT, &format!("/orgs/{}", seg(org)), Some(json!({ "name": name }))).await
    }

    /// The organization and its apps with their environments.
    pub async fn get_org(&self, org: &str) -> Result<Value, ShipyardError> {
        self.call(reqwest::Method::GET, &format!("/orgs/{}", seg(org)), None).await
    }

    pub async fn delete_org(&self, org: &str) -> Result<Value, ShipyardError> {
        self.call(reqwest::Method::DELETE, &format!("/orgs/{}", seg(org)), None).await
    }

    /// Creates the app (with its development database when asked), or renames it. Idempotent.
    pub async fn put_app(&self, org: &str, app: &str, name: &str, dev_database: bool) -> Result<Value, ShipyardError> {
        self.call(reqwest::Method::PUT, &Self::app_path(org, app), Some(json!({ "name": name, "dev_database": dev_database })))
            .await
    }

    /// Terminates the app everywhere: every environment, its data, the dev database, secrets.
    pub async fn delete_app(&self, org: &str, app: &str) -> Result<Value, ShipyardError> {
        self.call(reqwest::Method::DELETE, &Self::app_path(org, app), None).await
    }

    /// The development database, created when missing.
    pub async fn ensure_dev_database(&self, org: &str, app: &str) -> Result<Value, ShipyardError> {
        self.call(reqwest::Method::PUT, &format!("{}/dev-database", Self::app_path(org, app)), Some(json!({}))).await
    }

    /// Sends the source zip and manifest. `source` None re-applies the manifest to the last upload.
    pub async fn publish(&self, org: &str, app: &str, env: &str, manifest: &Value, source: Option<Vec<u8>>) -> Result<PublishAccepted, ShipyardError> {
        let manifest_part = reqwest::multipart::Part::text(manifest.to_string())
            .mime_str("application/json")
            .map_err(|e| ShipyardError::Unavailable(e.to_string()))?;
        let mut form = reqwest::multipart::Form::new().part("manifest", manifest_part);
        if let Some(bytes) = source {
            let part = reqwest::multipart::Part::bytes(bytes)
                .file_name("source.zip")
                .mime_str("application/zip")
                .map_err(|e| ShipyardError::Unavailable(e.to_string()))?;
            form = form.part("source", part);
        }
        let request = self
            .http
            .post(self.url(&format!("{}/publish", Self::env_path(org, app, env))))
            .timeout(Duration::from_secs(300))
            .multipart(form);
        let data = self.send(request, &self.config.key).await?;
        serde_json::from_value(data).map_err(|e| ShipyardError::Unavailable(format!("unexpected publish response: {e}")))
    }

    pub async fn get_publish(&self, org: &str, app: &str, env: &str, publish_id: &str) -> Result<Value, ShipyardError> {
        self.call(reqwest::Method::GET, &format!("{}/publishes/{}", Self::env_path(org, app, env), seg(publish_id)), None).await
    }

    /// Services with status, replicas, URLs and last deployment, plus the last publish.
    pub async fn get_env(&self, org: &str, app: &str, env: &str) -> Result<Value, ShipyardError> {
        self.call(reqwest::Method::GET, &Self::env_path(org, app, env), None).await
    }

    /// Takes the environment down: services, database and data, volumes, domains.
    pub async fn delete_env(&self, org: &str, app: &str, env: &str) -> Result<Value, ShipyardError> {
        self.call(reqwest::Method::DELETE, &Self::env_path(org, app, env), None).await
    }

    /// Status, steps and the build/deploy log of one deployment (secret values masked).
    pub async fn get_deployment(&self, org: &str, app: &str, env: &str, deployment_id: &str) -> Result<Value, ShipyardError> {
        self.call(reqwest::Method::GET, &format!("{}/deployments/{}", Self::env_path(org, app, env), seg(deployment_id)), None)
            .await
    }

    /// Runtime output of a service's replicas: `{ service, lines: ["<time> <line>"] }`.
    pub async fn service_logs(&self, org: &str, app: &str, env: &str, service: &str, tail: u32, since: Option<&str>) -> Result<Value, ShipyardError> {
        let mut path = format!("{}/services/{}/logs?tail={}", Self::env_path(org, app, env), seg(service), tail.clamp(1, 5000));
        if let Some(since) = since {
            path.push_str(&format!("&since={}", seg(since)));
        }
        self.call(reqwest::Method::GET, &path, None).await
    }

    /// CPU, memory, network and disk of each running container.
    pub async fn usage(&self, org: &str, app: &str, env: &str) -> Result<Value, ShipyardError> {
        self.call(reqwest::Method::GET, &format!("{}/usage", Self::env_path(org, app, env)), None).await
    }

    /// The app's secrets: names, refs, environments and where they're used. Never values.
    pub async fn list_secrets(&self, org: &str, app: &str) -> Result<Value, ShipyardError> {
        self.call(reqwest::Method::GET, &format!("{}/secrets", Self::app_path(org, app)), None).await
    }

    /// The value behind a secret ref. Needs the reveal key; server-side only.
    pub async fn reveal(&self, secret_ref: &str) -> Result<String, ShipyardError> {
        let key = self
            .config
            .reveal_key
            .as_deref()
            .ok_or_else(|| ShipyardError::Unauthorized(format!("{ENV_REVEAL_KEY} is not set")))?;
        let request = self.http.post(self.url(&format!("/secrets/{}/reveal", seg(secret_ref)))).timeout(Duration::from_secs(30));
        let data = self.send(request, key).await?;
        data.get("value")
            .and_then(Value::as_str)
            .map(str::to_string)
            .ok_or_else(|| ShipyardError::Unavailable("reveal returned no value".into()))
    }

    /// Registers a webhook endpoint for every event; the secret is only returned here.
    pub async fn create_webhook(&self, url: &str) -> Result<(String, String), ShipyardError> {
        let data = self.call(reqwest::Method::POST, "/webhooks", Some(json!({ "url": url, "events": ["*"] }))).await?;
        let id = data.get("id").and_then(Value::as_str).unwrap_or_default().to_string();
        let secret = data
            .get("secret")
            .and_then(Value::as_str)
            .ok_or_else(|| ShipyardError::Unavailable("webhook registration returned no secret".into()))?
            .to_string();
        Ok((id, secret))
    }

    pub async fn list_webhooks(&self) -> Result<Value, ShipyardError> {
        self.call(reqwest::Method::GET, "/webhooks", None).await
    }

    pub async fn delete_webhook(&self, id: &str) -> Result<Value, ShipyardError> {
        self.call(reqwest::Method::DELETE, &format!("/webhooks/{}", seg(id)), None).await
    }

    /// Turns an endpoint on or off, or changes its URL or events.
    pub async fn update_webhook(&self, id: &str, patch: &Value) -> Result<Value, ShipyardError> {
        self.call(reqwest::Method::PATCH, &format!("/webhooks/{}", seg(id)), Some(patch.clone())).await
    }

    /// A new signing secret for the endpoint; returned only here.
    pub async fn rotate_webhook_secret(&self, id: &str) -> Result<String, ShipyardError> {
        let data = self.call(reqwest::Method::POST, &format!("/webhooks/{}/rotate-secret", seg(id)), None).await?;
        data.get("secret")
            .and_then(Value::as_str)
            .map(str::to_string)
            .ok_or_else(|| ShipyardError::Unavailable("rotation returned no secret".into()))
    }

    /// The endpoint's recent deliveries, optionally only those with one status (pending, delivered, failed).
    pub async fn webhook_deliveries(&self, id: &str, status: Option<&str>, limit: u32) -> Result<Value, ShipyardError> {
        let mut path = format!("/webhooks/{}/deliveries?limit={}", seg(id), limit.clamp(1, 200));
        if let Some(status) = status {
            path.push_str(&format!("&status={}", seg(status)));
        }
        self.call(reqwest::Method::GET, &path, None).await
    }

    /// Queues a delivery to be sent again.
    pub async fn redeliver(&self, delivery_id: &str) -> Result<Value, ShipyardError> {
        self.call(reqwest::Method::POST, &format!("/deliveries/{}/redeliver", seg(delivery_id)), None).await
    }

    /// The organization's events after `after` (exclusive), oldest first: `{ events, next_after }`.
    pub async fn org_events(&self, org: &str, after: Option<&str>, limit: u32) -> Result<Value, ShipyardError> {
        let mut path = format!("/orgs/{}/events?limit={}", seg(org), limit.clamp(1, 500));
        if let Some(after) = after {
            path.push_str(&format!("&after={}", seg(after)));
        }
        self.call(reqwest::Method::GET, &path, None).await
    }

    /// The event types Shipyard can send.
    pub async fn event_types(&self) -> Result<Value, ShipyardError> {
        self.call(reqwest::Method::GET, "/event-types", None).await
    }

    /// The app's environments with their services, and whether it has a dev database.
    pub async fn get_app(&self, org: &str, app: &str) -> Result<Value, ShipyardError> {
        self.call(reqwest::Method::GET, &Self::app_path(org, app), None).await
    }

    /// Creates `to` from `env`'s last manifest and source (its own database, secrets and URL) and deploys it.
    pub async fn duplicate_env(&self, org: &str, app: &str, env: &str, to: &str, copy_secrets: bool) -> Result<Value, ShipyardError> {
        self.call(
            reqwest::Method::POST,
            &format!("{}/duplicate", Self::env_path(org, app, env)),
            Some(json!({ "to": to, "copy_secrets": copy_secrets })),
        )
        .await
    }

    /// Creates or changes a secret, app-wide or for one environment. `value` None asks Shipyard
    /// to generate a password. The response has the ref, never the value.
    pub async fn set_secret(&self, org: &str, app: &str, name: &str, value: Option<&str>, environment: Option<&str>) -> Result<Value, ShipyardError> {
        let body = match value {
            Some(value) => json!({ "value": value, "environment": environment }),
            None => json!({ "generate": "password", "environment": environment }),
        };
        self.call(reqwest::Method::PUT, &format!("{}/secrets/{}", Self::app_path(org, app), seg(name)), Some(body)).await
    }

    pub async fn delete_secret(&self, org: &str, app: &str, name: &str, environment: Option<&str>) -> Result<Value, ShipyardError> {
        let mut path = format!("{}/secrets/{}", Self::app_path(org, app), seg(name));
        if let Some(environment) = environment {
            path.push_str(&format!("?environment={}", seg(environment)));
        }
        self.call(reqwest::Method::DELETE, &path, None).await
    }

    /// Sends a signed `ping` to the endpoint and returns the result.
    pub async fn test_webhook(&self, id: &str) -> Result<Value, ShipyardError> {
        self.call(reqwest::Method::POST, &format!("/webhooks/{}/test", seg(id)), None).await
    }
}

/// A Shipyard app slug for a project: its name in `a-z0-9-` and a piece of its id, so two
/// projects with the same name don't collide (1–63 chars, no leading or trailing `-`).
pub fn app_slug(name: &str, project_id: &str) -> String {
    let mut base = String::new();
    for c in name.to_lowercase().chars() {
        if c.is_ascii_alphanumeric() {
            base.push(c);
        } else if !base.ends_with('-') && !base.is_empty() {
            base.push('-');
        }
    }
    let base: String = base.trim_matches('-').chars().take(40).collect();
    let base = base.trim_end_matches('-');
    let suffix: String = project_id.chars().filter(char::is_ascii_alphanumeric).take(6).collect::<String>().to_lowercase();
    if base.is_empty() {
        format!("app-{suffix}")
    } else {
        format!("{base}-{suffix}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn config(base: &str) -> ShipyardConfig {
        ShipyardConfig { base_url: base.to_string(), key: "ship_int_manage".into(), reveal_key: Some("ship_int_reveal".into()), webhook_secret: None }
    }

    #[test]
    fn slugs_are_valid_shipyard_names() {
        assert_eq!(app_slug("Todo App!", "3f9a2c1b-0000"), "todo-app-3f9a2c");
        assert_eq!(app_slug("  ", "ABCDEF12"), "app-abcdef");
        assert_eq!(app_slug("Ünïcode — test", "123456"), "n-code-test-123456");
        let long = app_slug(&"x".repeat(200), "abcdef");
        assert!(long.len() <= 63);
    }

    #[test]
    fn envelope_errors_carry_shipyard_words() {
        let err = unwrap_envelope(400, r#"{"data":null,"error":{"code":"BAD_REQUEST","message":"Missing secret STRIPE_KEY"}}"#).unwrap_err();
        assert_eq!(err, ShipyardError::Rejected { status: 400, code: "BAD_REQUEST".into(), message: "Missing secret STRIPE_KEY".into() });
        assert!(matches!(unwrap_envelope(503, "down"), Err(ShipyardError::Unavailable(_))));
        assert_eq!(unwrap_envelope(200, r#"{"data":{"a":1},"error":null}"#).unwrap(), json!({ "a": 1 }));
    }

    #[tokio::test]
    async fn publish_sends_manifest_and_source() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/orgs/org-1/apps/todo-abc/envs/production/publish"))
            .and(header("authorization", "Bearer ship_int_manage"))
            .respond_with(ResponseTemplate::new(202).set_body_json(json!({
                "data": { "publish_id": "p1", "status": "running", "url": "https://todo.apps.dev", "warnings": [] },
                "error": null
            })))
            .expect(1)
            .mount(&server)
            .await;
        let http = reqwest::Client::new();
        let config = config(&server.uri());
        let accepted = Shipyard::new(&http, &config)
            .publish("org-1", "todo-abc", "production", &json!({ "services": [] }), Some(vec![1, 2, 3]))
            .await
            .unwrap();
        assert_eq!(accepted.publish_id, "p1");
        assert_eq!(accepted.url.as_deref(), Some("https://todo.apps.dev"));
        let received = &server.received_requests().await.unwrap()[0];
        let body = String::from_utf8_lossy(&received.body);
        assert!(body.contains("name=\"manifest\""));
        assert!(body.contains("name=\"source\""));
    }

    #[tokio::test]
    async fn reveal_uses_the_reveal_key() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/secrets/sec_1/reveal"))
            .and(header("authorization", "Bearer ship_int_reveal"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "data": { "ref": "sec_1", "value": "postgres://x" }, "error": null })))
            .mount(&server)
            .await;
        let http = reqwest::Client::new();
        let config = config(&server.uri());
        assert_eq!(Shipyard::new(&http, &config).reveal("sec_1").await.unwrap(), "postgres://x");
    }
}
