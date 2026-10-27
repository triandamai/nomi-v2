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
const DEV: &str = "dev";
const BACKEND_JSON: &str = "nomi/backend.json";

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

/// Whether the project keeps its data on Relay rather than in Shipyard's Postgres (Drizzle).
pub(crate) async fn on_relay(pool: &PgPool, project_id: Uuid) -> Result<bool, ApiError> {
    let backend: Option<String> = sqlx::query_scalar("SELECT backend FROM projects WHERE id = $1")
        .bind(project_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("couldn't read the project: {e}")))?;
    Ok(backend.ok_or((StatusCode::NOT_FOUND, "project not found".to_string()))? == "relay")
}

/// Relay's name for a data sheet source: the preview's `dev`, or the published `production`.
pub(crate) fn relay_env(production: bool) -> &'static str {
    if production {
        PRODUCTION
    } else {
        DEV
    }
}

/// The project's backend in `env` for its data sheet: where it is, its anon key, its tables
/// with their columns and policies, its buckets, live tables, functions and sign-in.
pub async fn overview(http_client: &reqwest::Client, config: &RelayConfig, project_id: Uuid, env: &str) -> Result<Value, ApiError> {
    let relay = Relay::new(http_client, config);
    let found = relay.backend(project_id).await.map_err(relay_error)?;
    let keys = found.environment(env).ok_or((StatusCode::NOT_FOUND, format!("the project's backend has no {env} environment")))?;
    let mut answer = relay.describe(project_id, env).await.map_err(relay_error)?;
    answer["backend"] = Value::from("relay");
    answer["url"] = Value::from(keys.url.clone());
    answer["anon_key"] = Value::from(keys.anon_key.clone());
    answer["auth"] = relay.auth(project_id, env).await.map_err(relay_error)?;
    Ok(answer)
}

/// Before a publish of a Relay project: applies the production migrations it doesn't have yet,
/// from the files being published. A failure stops the publish (the running version keeps
/// serving) and says which file and line. A project not on Relay passes straight through.
pub async fn apply_before_publish(
    pool: &PgPool,
    http_client: &reqwest::Client,
    config: Option<&RelayConfig>,
    project_id: Uuid,
    files: &[(String, Vec<u8>)],
) -> Result<(), ApiError> {
    let backend: String = sqlx::query_scalar("SELECT backend FROM projects WHERE id = $1")
        .bind(project_id)
        .fetch_one(pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("couldn't read the project: {e}")))?;
    if backend != "relay" {
        return Ok(());
    }
    let config = config.ok_or((StatusCode::SERVICE_UNAVAILABLE, "this project is on Relay, and Relay isn't set up on this Nomi".to_string()))?;
    let relay = Relay::new(http_client, config);
    // The published app's sign-in settings first, so its users can sign in the moment it's live.
    if let Some((_, bytes)) = files.iter().find(|(path, _)| path == BACKEND_JSON) {
        let parsed: Value = serde_json::from_slice(bytes).map_err(|e| (StatusCode::UNPROCESSABLE_ENTITY, format!("{BACKEND_JSON} isn't valid JSON: {e}")))?;
        if let Some(auth) = parsed.get("auth") {
            relay.put_auth(project_id, PRODUCTION, auth).await.map_err(relay_error)?;
        }
    }
    let backend: Option<Value> = files.iter().find(|(path, _)| path == BACKEND_JSON).and_then(|(_, bytes)| serde_json::from_slice(bytes).ok());
    if let Some(storage) = backend.as_ref().and_then(|b| b.get("storage")) {
        relay.put_storage(project_id, PRODUCTION, storage).await.map_err(relay_error)?;
    }
    let migrations: Vec<SourceFile> = files
        .iter()
        .filter(|(path, _)| path.starts_with("nomi/migrations/") && path.ends_with(".sql"))
        .map(|(path, bytes)| SourceFile { path: path.clone(), content: String::from_utf8_lossy(bytes).into_owned() })
        .collect();
    relay.apply_migrations(project_id, PRODUCTION, &migrations).await.map_err(relay_error)?;
    // After the migrations, so every listed table exists.
    if let Some(tables) = backend.as_ref().and_then(|b| b.get("realtime")) {
        relay.put_realtime(project_id, PRODUCTION, tables).await.map_err(relay_error)?;
    }
    // Last, so they meet the schema they were written for; a failed check stops the publish.
    deploy_functions_for(&relay, project_id, PRODUCTION, files).await
}

/// Rolling a Relay project back to an earlier publish: its functions go back too, deployed again
/// from that publish's source (migrations never go back).
pub async fn functions_on_rollback(
    pool: &PgPool,
    http_client: &reqwest::Client,
    config: Option<&RelayConfig>,
    project_id: Uuid,
    env: &str,
    source_zip: &[u8],
) -> Result<(), ApiError> {
    let backend: Option<String> = sqlx::query_scalar("SELECT backend FROM projects WHERE id = $1")
        .bind(project_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("couldn't read the project: {e}")))?;
    if backend.as_deref() != Some("relay") {
        return Ok(());
    }
    let config = config.ok_or((StatusCode::SERVICE_UNAVAILABLE, "this project is on Relay, and Relay isn't set up on this Nomi".to_string()))?;
    let mut files = nomi_shipyard::bundle::files_under(source_zip, "nomi/functions/").map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("couldn't read that publish's source: {e}")))?;
    files.extend(nomi_shipyard::bundle::files_under(source_zip, BACKEND_JSON).unwrap_or_default());
    deploy_functions_for(&Relay::new(http_client, config), project_id, env, &files).await
}

/// Deploys the functions among `files` to `env`, when there are any.
pub async fn deploy_functions_for(relay: &Relay<'_>, project_id: Uuid, env: &str, files: &[(String, Vec<u8>)]) -> Result<(), ApiError> {
    let sources = function_sources(files);
    if sources.is_empty() {
        return Ok(());
    }
    let settings = files
        .iter()
        .find(|(path, _)| path == BACKEND_JSON)
        .and_then(|(_, bytes)| serde_json::from_slice::<Value>(bytes).ok())
        .and_then(|b| b.get("functions").cloned())
        .unwrap_or_else(|| serde_json::json!({}));
    relay.deploy_functions(project_id, env, &sources, &settings).await.map(|_| ()).map_err(relay_error)
}

/// What to tell Koda after a functions deploy.
fn deploy_summary(env: &str, answer: &Value) -> String {
    let version = answer.get("version").and_then(Value::as_i64).unwrap_or_default();
    let names: Vec<&str> = answer.get("functions").and_then(Value::as_object).map(|f| f.keys().map(String::as_str).collect()).unwrap_or_default();
    format!("Deployed to {env} as build {version}: {}.", if names.is_empty() { "no functions".to_string() } else { names.join(", ") })
}

/// The files of `nomi/functions/` among those being published, helpers included.
pub fn function_sources(files: &[(String, Vec<u8>)]) -> Vec<SourceFile> {
    const EXTENSIONS: [&str; 5] = [".ts", ".js", ".mts", ".mjs", ".json"];
    let mut sources: Vec<SourceFile> = files
        .iter()
        .filter(|(path, _)| path.starts_with("nomi/functions/") && EXTENSIONS.iter().any(|ext| path.ends_with(ext)))
        .map(|(path, bytes)| SourceFile { path: path.clone(), content: String::from_utf8_lossy(bytes).into_owned() })
        .collect();
    sources.sort_by(|a, b| a.path.cmp(&b.path));
    sources
}

/// The names in `backend.json`'s `storage` block, for telling Koda what was saved.
fn bucket_names(storage: &Value) -> String {
    let names: Vec<&str> = storage.get("buckets").and_then(Value::as_array).into_iter().flatten().filter_map(|b| b.get("name").and_then(Value::as_str)).collect();
    if names.is_empty() { "none".to_string() } else { names.join(", ") }
}

fn table_names(tables: &Value) -> String {
    let names: Vec<&str> = tables.as_array().into_iter().flatten().filter_map(Value::as_str).collect();
    if names.is_empty() { "no tables".to_string() } else { names.join(", ") }
}

/// What to tell Koda after sign-in settings were saved to `env`.
fn sign_in_summary(env: &str, status: &Value) -> String {
    let missing: Vec<&str> = status.get("missing_secrets").and_then(Value::as_array).into_iter().flatten().filter_map(Value::as_str).collect();
    if missing.is_empty() {
        format!("Sign-in settings saved to {env}.")
    } else {
        format!(
            "Sign-in settings saved to {env}. Still needs the project secret{} {}: ask the person to add {} in the project's Secrets (you never see the values).",
            if missing.len() == 1 { "" } else { "s" },
            missing.join(", "),
            if missing.len() == 1 { "it" } else { "them" }
        )
    }
}

/// What a project's preview needs to reach its dev backend: the address and anon key for
/// everyone who can open the project, the service key only for its admins.
#[derive(Debug, serde::Serialize)]
pub struct PreviewAccess {
    pub url: String,
    pub anon_key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_key: Option<String>,
}

/// The dev backend's access for `user_id`'s preview of the project; 404 when they can't open
/// the project or it isn't on Relay.
pub async fn preview_access(
    pool: &PgPool,
    http_client: &reqwest::Client,
    config: Option<&RelayConfig>,
    project_id: Uuid,
    user_id: Uuid,
) -> Result<PreviewAccess, ApiError> {
    let role = project_role(pool, project_id, user_id).await?;
    let backend: String = sqlx::query_scalar("SELECT backend FROM projects WHERE id = $1")
        .bind(project_id)
        .fetch_one(pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("couldn't read the project: {e}")))?;
    if backend != "relay" {
        return Err((StatusCode::NOT_FOUND, "this project isn't on Relay".to_string()));
    }
    let config = config.ok_or((StatusCode::SERVICE_UNAVAILABLE, "this project is on Relay, and Relay isn't set up on this Nomi".to_string()))?;
    let found = Relay::new(http_client, config).backend(project_id).await.map_err(relay_error)?;
    let dev = found.environment(DEV).ok_or((StatusCode::NOT_FOUND, "the project's backend has no dev environment".to_string()))?;
    Ok(PreviewAccess { url: dev.url.clone(), anon_key: dev.anon_key.clone(), service_key: (role == "admin").then(|| dev.service_key.clone()) })
}

fn source_files(files: &[ProjectFile]) -> Vec<SourceFile> {
    files.iter().map(|f| SourceFile { path: f.path.clone(), content: f.content.clone() }).collect()
}

/// What storing a project's secrets on Shipyard takes.
pub(crate) struct ShipyardHandle {
    pool: PgPool,
    project_storage: ProjectStore,
    http_client: reqwest::Client,
    pub(crate) config: ShipyardConfig,
    settings_key: Option<[u8; 32]>,
}

impl ShipyardHandle {
    /// An owned copy of what `state` reaches Shipyard with, for work that outlives the request.
    pub(crate) fn of(state: &impl ShipyardAccess) -> Option<Self> {
        Some(Self {
            pool: state.pool().clone(),
            project_storage: state.project_storage().clone(),
            http_client: state.http_client().clone(),
            config: state.shipyard()?.clone(),
            settings_key: state.settings_key(),
        })
    }
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
    fn settings_key(&self) -> Option<[u8; 32]> {
        self.settings_key
    }
}

/// Where the Relay connection comes from: fixed (tests), or the admin's stored one read on every
/// call, with the environment's as the fallback.
enum Connection {
    Fixed(Option<RelayConfig>),
    Stored { settings_key: Option<[u8; 32]>, fallback: Option<RelayConfig> },
}

pub struct ServerRelayOps {
    pool: PgPool,
    http_client: reqwest::Client,
    connection: Connection,
    shipyard: Option<ShipyardHandle>,
}

impl ServerRelayOps {
    pub fn new(pool: PgPool, http_client: reqwest::Client, config: Option<RelayConfig>) -> Self {
        Self { pool, http_client, connection: Connection::Fixed(config), shipyard: None }
    }

    /// Reads the connection saved in the admin on every call, falling back to `fallback`.
    pub fn stored(pool: PgPool, http_client: reqwest::Client, settings_key: Option<[u8; 32]>, fallback: Option<RelayConfig>) -> Self {
        Self { pool, http_client, connection: Connection::Stored { settings_key, fallback }, shipyard: None }
    }

    /// Where Relay's keys are kept for the published app. Without it (a Nomi without Shipyard),
    /// backends are still made, but nothing is stored for publishing.
    pub fn with_shipyard(mut self, project_storage: ProjectStore, config: ShipyardConfig) -> Self {
        self.shipyard = Some(ShipyardHandle { pool: self.pool.clone(), project_storage, http_client: self.http_client.clone(), config, settings_key: self.settings_key() });
        self
    }

    /// Relay as set in the admin (or RELAY_CONTROL_URL / RELAY_CONTROL_KEY), Shipyard from SHIPYARD_*.
    pub fn shared(pool: PgPool, project_storage: ProjectStore) -> SharedRelayOps {
        let ops = Self::stored(pool, crate::http_client(), crate::relay_connection::settings_key_from_env(), RelayConfig::from_env());
        let ops = match ShipyardConfig::from_env() {
            Some(config) => ops.with_shipyard(project_storage, config),
            None => ops,
        };
        std::sync::Arc::new(ops)
    }

    async fn config(&self) -> Option<RelayConfig> {
        match &self.connection {
            Connection::Fixed(config) => config.clone(),
            Connection::Stored { settings_key, fallback } => crate::relay_connection::load_or(&self.pool, settings_key.as_ref(), fallback.clone()).await,
        }
    }

    /// The key that seals the projects' Relay control keys.
    fn settings_key(&self) -> Option<[u8; 32]> {
        match &self.connection {
            Connection::Stored { settings_key, .. } => *settings_key,
            Connection::Fixed(_) => None,
        }
    }

    /// The Relay for this project's `env`: its own when it has one, else the shared one.
    async fn config_for(&self, project_id: Uuid, env: RelayEnv) -> Result<RelayConfig, String> {
        // A development backend that went to sleep is woken, and waited for, before it's called.
        if let (RelayEnv::Dev, Some(shipyard), Some(key)) = (env, &self.shipyard, self.settings_key()) {
            let deps = crate::relay_backend::Deps { pool: &self.pool, http: &self.http_client, shipyard: &shipyard.config, settings_key: &key };
            crate::relay_backend::ensure_awake(&deps, project_id).await?;
        }
        let shared = self.config().await;
        let settings_key = match &self.connection {
            Connection::Stored { settings_key, .. } => settings_key.as_ref(),
            Connection::Fixed(_) => None,
        };
        crate::project_relays::config_for(&self.pool, settings_key, shared, project_id, env.as_str()).await?.ok_or_else(|| NOT_SET_UP.to_string())
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
    async fn is_available(&self) -> bool {
        if self.config().await.is_some() {
            return true;
        }
        match (&self.shipyard, self.settings_key()) {
            (Some(_), Some(key)) => crate::relay_backend::images_set(&self.pool, &key).await,
            _ => false,
        }
    }

    async fn ensure_backend(&self, user_id: Uuid, project_id: Uuid) -> Result<Uuid, String> {
        self.check_access(user_id, project_id, RelayEnv::Dev).await?;
        // With Shipyard, the project gets its own Relay for development, made in the background.
        let own = match (&self.shipyard, self.settings_key()) {
            (Some(shipyard), Some(key)) if crate::relay_backend::images_set(&self.pool, &key).await => {
                crate::relay_backend::start(self.pool.clone(), self.http_client.clone(), shipyard.config.clone(), key, project_id, "dev", true).await?;
                sqlx::query("UPDATE projects SET relay_project_id = $1, updated_at = now() WHERE id = $2")
                    .bind(project_id)
                    .bind(project_id)
                    .execute(&self.pool)
                    .await
                    .map_err(|e| format!("couldn't remember the Relay backend: {e}"))?;
                true
            }
            _ => false,
        };
        // Until a project's production Relay is made with its first publish, its published app
        // gets its keys from the shared Relay, when this Nomi has one.
        let Some(config) = self.config().await else {
            return if own { Ok(project_id) } else { Err(NOT_SET_UP.to_string()) };
        };
        let relay = Relay::new(&self.http_client, &config);
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
        Ok(if own { project_id } else { backend.id })
    }

    async fn apply_migrations(&self, user_id: Uuid, project_id: Uuid, env: RelayEnv, files: &[ProjectFile]) -> Result<String, String> {
        let config = self.config_for(project_id, env).await?;
        let relay = Relay::new(&self.http_client, &config);
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
        let config = self.config_for(project_id, env).await?;
        let relay = Relay::new(&self.http_client, &config);
        self.check_access(user_id, project_id, env).await?;
        let mut schema = relay.describe(project_id, env.as_str()).await.map_err(|e| message(relay_error(e)))?;
        // Sign-in too: which methods are on and which secrets they still need.
        if let (Ok(auth), Some(object)) = (relay.auth(project_id, env.as_str()).await, schema.as_object_mut()) {
            object.insert("auth".to_string(), auth);
        }
        serde_json::to_string_pretty(&schema).map_err(|e| e.to_string())
    }

    async fn deploy_functions(&self, user_id: Uuid, project_id: Uuid, env: RelayEnv, files: &[ProjectFile], settings: &Value) -> Result<String, String> {
        let config = self.config_for(project_id, env).await?;
        let relay = Relay::new(&self.http_client, &config);
        self.check_access(user_id, project_id, env).await?;
        let answer = relay.deploy_functions(project_id, env.as_str(), &source_files(files), settings).await.map_err(|e| message(relay_error(e)))?;
        Ok(deploy_summary(env.as_str(), &answer))
    }

    async fn types(&self, user_id: Uuid, project_id: Uuid, env: RelayEnv) -> Result<String, String> {
        let config = self.config_for(project_id, env).await?;
        let relay = Relay::new(&self.http_client, &config);
        self.check_access(user_id, project_id, env).await?;
        relay.types(project_id, env.as_str()).await.map_err(|e| message(relay_error(e)))
    }

    async fn apply_auth(&self, user_id: Uuid, project_id: Uuid, env: RelayEnv, auth: &Value) -> Result<String, String> {
        let config = self.config_for(project_id, env).await?;
        let relay = Relay::new(&self.http_client, &config);
        self.check_access(user_id, project_id, env).await?;
        relay.put_auth(project_id, env.as_str(), auth).await.map_err(|e| message(relay_error(e)))?;
        let status = relay.auth(project_id, env.as_str()).await.unwrap_or(Value::Null);
        Ok(sign_in_summary(env.as_str(), &status))
    }

    async fn apply_storage(&self, user_id: Uuid, project_id: Uuid, env: RelayEnv, storage: &Value) -> Result<String, String> {
        let config = self.config_for(project_id, env).await?;
        let relay = Relay::new(&self.http_client, &config);
        self.check_access(user_id, project_id, env).await?;
        relay.put_storage(project_id, env.as_str(), storage).await.map_err(|e| message(relay_error(e)))?;
        Ok(format!("Buckets saved to {}: {}.", env.as_str(), bucket_names(storage)))
    }

    async fn apply_realtime(&self, user_id: Uuid, project_id: Uuid, env: RelayEnv, tables: &Value) -> Result<String, String> {
        let config = self.config_for(project_id, env).await?;
        let relay = Relay::new(&self.http_client, &config);
        self.check_access(user_id, project_id, env).await?;
        relay.put_realtime(project_id, env.as_str(), tables).await.map_err(|e| message(relay_error(e)))?;
        Ok(format!("Live changes are on in {} for: {}.", env.as_str(), table_names(tables)))
    }
}
