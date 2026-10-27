//! The server's side of `nomi_agent_core::project_ops`: agents publish a project and look into
//! its running app through the same code the Deploy, Monitor and Database sheets use, as the
//! person they work for (with the same checks), from the web server or a worker.

use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::routes::deploy::{client, project_role, require_admin, shipyard_error, target, ApiError};
use crate::routes::project_database::{self as database, Access};

/// How many recent publishes the status names.
const STATUS_PUBLISHES: i64 = 3;
/// The most of Shipyard's environment description passed on.
const MAX_ENV_DESCRIPTION_CHARS: usize = 4000;

fn message((_, message): ApiError) -> String {
    message
}

fn database_source(source: DataSource) -> database::Source {
    match source {
        DataSource::Development => database::Source::Development,
        DataSource::Production => database::Source::Production,
    }
}

use nomi_agent_core::project_ops::{DataSource, ProjectOps, QueryRows, SharedProjectOps, TableInfo};
use nomi_relay_client::{Relay, RelayConfig};
use nomi_shipyard::ShipyardConfig;
use nomi_storage::ProjectStore;
use serde_json::Value;

use crate::relay_ops::relay_error;

/// What reaching a project on Shipyard takes, without the rest of the web server's state.
pub(crate) trait ShipyardAccess {
    fn pool(&self) -> &PgPool;
    fn project_storage(&self) -> &ProjectStore;
    fn http_client(&self) -> &reqwest::Client;
    fn shipyard(&self) -> Option<&ShipyardConfig>;
    /// The key that seals what Nomi keeps about projects' Relays; the environment's by default.
    fn settings_key(&self) -> Option<[u8; 32]> {
        crate::relay_connection::settings_key_from_env()
    }
}

impl ShipyardAccess for crate::app::AppState {
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
        self.shipyard.as_ref()
    }
    fn settings_key(&self) -> Option<[u8; 32]> {
        Some(self.settings_key)
    }
}

pub struct ServerProjectOps {
    pool: PgPool,
    project_storage: ProjectStore,
    http_client: reqwest::Client,
    shipyard: Option<ShipyardConfig>,
    /// Opens the Relay connection saved in the admin, for projects that keep their data on Relay.
    relay_settings_key: Option<[u8; 32]>,
}

impl ShipyardAccess for ServerProjectOps {
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
        self.shipyard.as_ref()
    }
    fn settings_key(&self) -> Option<[u8; 32]> {
        self.relay_settings_key
    }
}

impl ServerProjectOps {
    /// Shipyard from the environment (SHIPYARD_*), like the web server's own.
    pub fn shared(pool: PgPool, project_storage: ProjectStore) -> SharedProjectOps {
        std::sync::Arc::new(Self {
            pool,
            project_storage,
            http_client: crate::http_client(),
            shipyard: ShipyardConfig::from_env(),
            relay_settings_key: crate::relay_connection::settings_key_from_env(),
        })
    }

    /// Without Shipyard, with Relay as saved in the admin (sealed with `relay_settings_key`).
    pub fn with_relay(pool: PgPool, project_storage: ProjectStore, http_client: reqwest::Client, relay_settings_key: Option<[u8; 32]>) -> Self {
        Self { pool, project_storage, http_client, shipyard: None, relay_settings_key }
    }

    /// Relay, when the project keeps its data there: Iris reads it through Relay as the
    /// project's owner instead of Shipyard's Postgres.
    async fn relay_for(&self, project_id: Uuid, env: &str) -> Result<Option<RelayConfig>, String> {
        if !crate::relay_ops::on_relay(&self.pool, project_id).await.map_err(message)? {
            return Ok(None);
        }
        let shared = crate::relay_connection::load_or(&self.pool, self.relay_settings_key.as_ref(), RelayConfig::from_env()).await;
        crate::project_relays::config_for(&self.pool, self.relay_settings_key.as_ref(), shared, project_id, env)
            .await?
            .map(Some)
            .ok_or_else(|| "this project is on Relay, and Relay isn't set up on this Nomi".to_string())
    }
}

fn relay_env(source: DataSource) -> &'static str {
    crate::relay_ops::relay_env(source == DataSource::Production)
}

#[async_trait]
impl ProjectOps for ServerProjectOps {
    async fn publish(&self, user_id: Uuid, project_id: Uuid, environment: &str, agent: Option<&str>) -> Result<String, String> {
        require_admin(&self.pool, project_id, user_id).await.map_err(message)?;
        let row = match crate::routes::deploy::begin_publish(self, project_id, user_id, environment, agent).await.map_err(message)? {
            crate::routes::deploy::Started::Publishing(row) => row,
            // Not started yet: the app's own database is made first, and the publish follows by itself.
            crate::routes::deploy::Started::Preparing(note) => {
                return Ok(format!("{note} Nomi posts in this chat when the app is live, or why it couldn't be. Don't publish again and don't give the person a link yet."));
            }
        };
        // Accepted isn't live: the address only works once Shipyard says the publish succeeded,
        // and Nomi posts it in the chat then (or why it failed).
        Ok(format!(
            "Publishing files version {} to {} started. It takes a few minutes. Don't give the person a link yet: \
             Nomi posts the live address in this chat when it's up, or why it failed. The Deployment sheet shows the progress.",
            row.files_version, row.environment
        ))
    }

    async fn status(&self, user_id: Uuid, project_id: Uuid, environment: &str) -> Result<String, String> {
        project_role(&self.pool, project_id, user_id).await.map_err(message)?;
        let publishes: Vec<(String, String, Option<String>, Option<String>, chrono::DateTime<chrono::Utc>, Option<String>)> = sqlx::query_as(
            "SELECT environment, status, url, error, created_at, \
                    NULLIF(concat_ws(' ', failure->>'phase', failure->>'step', failure->>'message'), '') \
             FROM project_publishes WHERE project_id = $1 AND environment = $2 \
             ORDER BY created_at DESC LIMIT $3",
        )
        .bind(project_id)
        .bind(environment)
        .bind(STATUS_PUBLISHES)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("couldn't read the publishes: {e}"))?;
        let mut lines = Vec::new();
        if publishes.is_empty() {
            lines.push(format!("Never published to {environment}."));
        }
        for (env, status, url, error, created_at, failure) in &publishes {
            let mut line = format!("Publish to {env} at {}: {status}", created_at.to_rfc3339());
            if let Some(url) = url {
                line.push_str(&format!(", {url}"));
            }
            if let Some(error) = error {
                line.push_str(&format!(" ({})", nomi_llm::redact::redacted(error)));
            }
            if let Some(failure) = failure {
                line.push_str(&format!(" [failed at: {}]", nomi_llm::redact::redacted(failure)));
            }
            lines.push(line);
        }
        let target = target(self, project_id).await.map_err(message)?;
        if let Some(app) = target.shipyard_app.as_deref() {
            match client(self).map_err(message)?.get_env(&target.org_key(), app, environment).await {
                Ok(live) => {
                    let described = nomi_llm::redact::redacted(live.to_string());
                    let clipped: String = described.chars().take(MAX_ENV_DESCRIPTION_CHARS).collect();
                    lines.push(format!("Running now (from Shipyard): {clipped}"));
                }
                Err(e) => lines.push(format!("Nothing running in {environment}: {}", message(shipyard_error(e)))),
            }
        }
        Ok(lines.join("\n"))
    }

    async fn logs(&self, user_id: Uuid, project_id: Uuid, environment: &str, tail: u32) -> Result<Vec<String>, String> {
        project_role(&self.pool, project_id, user_id).await.map_err(message)?;
        let target = target(self, project_id).await.map_err(message)?;
        let app = target.shipyard_app.as_deref().ok_or("this project hasn't been published yet")?;
        let logs = client(self)
            .map_err(message)?
            .service_logs(&target.org_key(), app, environment, "web", tail, None)
            .await
            .map_err(|e| message(shipyard_error(e)))?;
        Ok(logs
            .get("lines")
            .and_then(serde_json::Value::as_array)
            .map(|lines| lines.iter().filter_map(|l| l.as_str().map(str::to_string)).collect())
            .unwrap_or_default())
    }

    async fn tables(&self, user_id: Uuid, project_id: Uuid, source: DataSource) -> Result<Vec<TableInfo>, String> {
        require_admin(&self.pool, project_id, user_id).await.map_err(message)?;
        if let Some(config) = self.relay_for(project_id, relay_env(source)).await? {
            let schema = Relay::new(&self.http_client, &config).describe(project_id, relay_env(source)).await.map_err(|e| message(relay_error(e)))?;
            let names = schema.get("tables").and_then(Value::as_array).cloned().unwrap_or_default();
            return Ok(names
                .iter()
                .filter_map(|t| t.get("name").and_then(Value::as_str))
                .map(|name| TableInfo { schema: "public".into(), name: name.to_string(), rows_estimate: -1 })
                .collect());
        }
        let url = database::owner_url(self, project_id, database_source(source)).await.map_err(message)?;
        let tables = database::list_tables(&url).await.map_err(message)?;
        Ok(tables.into_iter().map(|t| TableInfo { schema: t.schema, name: t.name, rows_estimate: t.rows_estimate }).collect())
    }

    async fn query(&self, user_id: Uuid, project_id: Uuid, source: DataSource, sql: &str) -> Result<QueryRows, String> {
        require_admin(&self.pool, project_id, user_id).await.map_err(message)?;
        let sql = sql.trim().trim_end_matches(';').trim();
        if let Some(config) = self.relay_for(project_id, relay_env(source)).await? {
            let result = Relay::new(&self.http_client, &config).query(project_id, relay_env(source), sql, true).await.map_err(|e| message(relay_error(e)))?;
            let field = |name: &str| result.get(name).cloned().unwrap_or_default();
            return Ok(QueryRows {
                columns: serde_json::from_value(field("columns")).unwrap_or_default(),
                rows: serde_json::from_value(field("rows")).unwrap_or_default(),
                truncated: field("truncated").as_bool().unwrap_or_default(),
            });
        }
        let url = database::owner_url(self, project_id, database_source(source)).await.map_err(message)?;
        let result = database::execute_sql(&url, sql, Access::ReadOnly).await.map_err(message)?;
        Ok(QueryRows { columns: result.columns, rows: result.rows, truncated: result.truncated })
    }
}
