//! The server's side of `nomi_agent_core::relay_ops`: agents reach a project's Nomi Relay backend
//! through Relay's control plane (`nomi-relay-client`), as the person they work for, with the
//! same project checks the Deploy sheet uses. Anyone who can open the project works on its `dev`
//! backend; only its admins read or change `production`, as with its production database.

use async_trait::async_trait;
use axum::http::StatusCode;
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

use nomi_agent_core::relay_ops::{ProjectFile, RelayEnv, RelayOps, SharedRelayOps, NOT_SET_UP};
use nomi_relay_client::{Relay, RelayConfig, RelayError, SourceFile};
use nomi_shipyard::ShipyardConfig;
use nomi_storage::ProjectStore;

use crate::project_ops::ShipyardAccess;

/// Where the published app finds its Relay backend, and its keys: stored as the project's secrets
/// (production's values), so every publish binds them by name.
pub const SECRET_URL: &str = "NOMI_URL";
pub const SECRET_ANON_KEY: &str = "NOMI_ANON_KEY";
pub const SECRET_SERVICE_KEY: &str = "NOMI_SERVICE_KEY";
const PRODUCTION: &str = "production";

use crate::routes::deploy::{project_role, require_admin, ApiError};

fn message((_, message): ApiError) -> String {
    message
}

/// Relay's answer as an API error, like `deploy::shipyard_error` does for Shipyard: what Relay
/// refused is the caller's to fix; a broken key or an unreachable Relay is Nomi's.
pub(crate) fn relay_error(e: RelayError) -> ApiError {
    match e {
        RelayError::Rejected { status: 409, message, .. } => (StatusCode::CONFLICT, message),
        RelayError::Rejected { message, details, .. } => (StatusCode::UNPROCESSABLE_ENTITY, with_location(&message, &details)),
        RelayError::NotFound(message) => (StatusCode::NOT_FOUND, message),
        RelayError::Unauthorized(message) => {
            tracing::error!(%message, "Relay refused Nomi's control key");
            (StatusCode::BAD_GATEWAY, "Relay refused Nomi's key".to_string())
        }
        RelayError::Unavailable(message) => {
            tracing::warn!(%message, "Relay unavailable");
            (StatusCode::BAD_GATEWAY, "Relay couldn't be reached. Try again in a moment.".to_string())
        }
    }
}

/// `0002_task.sql line 3: <message>` when Relay said where a migration failed.
fn with_location(message: &str, details: &Value) -> String {
    let file = details.get("file").and_then(Value::as_str);
    let line = details.get("line").and_then(Value::as_u64);
    match (file, line) {
        (Some(file), Some(line)) => format!("{file} line {line}: {message}"),
        (Some(file), None) => format!("{file}: {message}"),
        _ => message.to_string(),
    }
}

fn source_files(files: &[ProjectFile]) -> Vec<SourceFile> {
    files.iter().map(|f| SourceFile { path: f.path.clone(), content: f.content.clone() }).collect()
}

/// What storing a project's secrets on Shipyard takes.
struct ShipyardHandle {
    pool: PgPool,
    project_storage: ProjectStore,
    http_client: reqwest::Client,
    config: ShipyardConfig,
}

impl ShipyardAccess for ShipyardHandle {
    fn pool(&self) -> &PgPool {
        &self.pool
    }
    fn project_storage(&self) -> &ProjectStore {
        &self.project_storage
    }
    fn http_client(&self) -> &reqwest::Client {
        &self.http_client
    }
    fn shipyard(&self) -> Option<&ShipyardConfig> {
        Some(&self.config)
    }
}

pub struct ServerRelayOps {
    pool: PgPool,
    http_client: reqwest::Client,
    config: Option<RelayConfig>,
    shipyard: Option<ShipyardHandle>,
}

impl ServerRelayOps {
    pub fn new(pool: PgPool, http_client: reqwest::Client, config: Option<RelayConfig>) -> Self {
        Self { pool, http_client, config, shipyard: None }
    }

    /// Where Relay's keys are kept for the published app. Without it (a Nomi without Shipyard),
    /// backends are still made, but nothing is stored for publishing.
    pub fn with_shipyard(mut self, project_storage: ProjectStore, config: ShipyardConfig) -> Self {
        self.shipyard = Some(ShipyardHandle { pool: self.pool.clone(), project_storage, http_client: self.http_client.clone(), config });
        self
    }

    /// Relay and Shipyard from the environment (RELAY_CONTROL_URL, RELAY_CONTROL_KEY, SHIPYARD_*).
    pub fn shared(pool: PgPool, project_storage: ProjectStore) -> SharedRelayOps {
        let ops = Self::new(pool, crate::http_client(), RelayConfig::from_env());
        let ops = match ShipyardConfig::from_env() {
            Some(config) => ops.with_shipyard(project_storage, config),
            None => ops,
        };
        std::sync::Arc::new(ops)
    }

    fn relay(&self) -> Result<Relay<'_>, String> {
        self.config.as_ref().map(|config| Relay::new(&self.http_client, config)).ok_or_else(|| NOT_SET_UP.to_string())
    }

    /// Anyone who can open the project works on `dev`; `production`, read or changed, is its admins'.
    async fn check_access(&self, user_id: Uuid, project_id: Uuid, env: RelayEnv) -> Result<(), String> {
        match env {
            RelayEnv::Dev => project_role(&self.pool, project_id, user_id).await.map(|_| ()).map_err(message),
            RelayEnv::Production => require_admin(&self.pool, project_id, user_id).await.map_err(message),
        }
    }
}

#[async_trait]
impl RelayOps for ServerRelayOps {
    fn is_available(&self) -> bool {
        self.config.is_some()
    }

    async fn ensure_backend(&self, user_id: Uuid, project_id: Uuid) -> Result<Uuid, String> {
        let relay = self.relay()?;
        self.check_access(user_id, project_id, RelayEnv::Dev).await?;
        let name: String = sqlx::query_scalar("SELECT name FROM projects WHERE id = $1")
            .bind(project_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| format!("couldn't read the project: {e}"))?;
        let backend = relay.ensure_backend(project_id, &name).await.map_err(|e| message(relay_error(e)))?;
        sqlx::query("UPDATE projects SET relay_project_id = $1, updated_at = now() WHERE id = $2")
            .bind(backend.id)
            .bind(project_id)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("couldn't remember the Relay backend: {e}"))?;
        match (&self.shipyard, backend.environment(PRODUCTION)) {
            (Some(shipyard), Some(production)) => {
                for (name, value) in [(SECRET_URL, &production.url), (SECRET_ANON_KEY, &production.anon_key), (SECRET_SERVICE_KEY, &production.service_key)] {
                    crate::routes::project_secrets::store(shipyard, project_id, name, value, user_id).await.map_err(message)?;
                }
            }
            (None, _) => tracing::info!(%project_id, "Relay backend made; no Shipyard here to keep its keys for publishing"),
            (Some(_), None) => return Err("Relay answered without a production environment".to_string()),
        }
        Ok(backend.id)
    }

    async fn apply_migrations(&self, user_id: Uuid, project_id: Uuid, env: RelayEnv, files: &[ProjectFile]) -> Result<String, String> {
        let relay = self.relay()?;
        self.check_access(user_id, project_id, env).await?;
        let answer = relay.apply_migrations(project_id, env.as_str(), &source_files(files)).await.map_err(|e| message(relay_error(e)))?;
        let applied: Vec<&str> = answer
            .get("applied")
            .and_then(Value::as_array)
            .map(|names| names.iter().filter_map(Value::as_str).collect())
            .unwrap_or_default();
        Ok(if applied.is_empty() {
            format!("Nothing to apply: {} already has every migration.", env.as_str())
        } else {
            format!("Applied to {}: {}.", env.as_str(), applied.join(", "))
        })
    }

    async fn describe_backend(&self, user_id: Uuid, project_id: Uuid, env: RelayEnv) -> Result<String, String> {
        let relay = self.relay()?;
        self.check_access(user_id, project_id, env).await?;
        let schema = relay.describe(project_id, env.as_str()).await.map_err(|e| message(relay_error(e)))?;
        serde_json::to_string_pretty(&schema).map_err(|e| e.to_string())
    }

    async fn deploy_functions(&self, user_id: Uuid, project_id: Uuid, env: RelayEnv, files: &[ProjectFile]) -> Result<String, String> {
        let relay = self.relay()?;
        self.check_access(user_id, project_id, env).await?;
        let answer = relay.deploy_functions(project_id, env.as_str(), &source_files(files)).await.map_err(|e| message(relay_error(e)))?;
        serde_json::to_string_pretty(&answer).map_err(|e| e.to_string())
    }

    async fn types(&self, user_id: Uuid, project_id: Uuid, env: RelayEnv) -> Result<String, String> {
        let relay = self.relay()?;
        self.check_access(user_id, project_id, env).await?;
        relay.types(project_id, env.as_str()).await.map_err(|e| message(relay_error(e)))
    }
}
