//! The server's side of `nomi_agent_core::project_ops`: agents publish a project through the
//! same code the Deploy sheet uses, as the person they work for, from the web server or a worker.

use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use nomi_agent_core::project_ops::{ProjectOps, SharedProjectOps};
use nomi_shipyard::ShipyardConfig;
use nomi_storage::ProjectStore;

/// What reaching a project on Shipyard takes, without the rest of the web server's state.
pub(crate) trait ShipyardAccess {
    fn pool(&self) -> &PgPool;
    fn project_storage(&self) -> &ProjectStore;
    fn http_client(&self) -> &reqwest::Client;
    fn shipyard(&self) -> Option<&ShipyardConfig>;
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
}

pub struct ServerProjectOps {
    pool: PgPool,
    project_storage: ProjectStore,
    http_client: reqwest::Client,
    shipyard: Option<ShipyardConfig>,
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
}

impl ServerProjectOps {
    /// Shipyard from the environment (SHIPYARD_*), like the web server's own.
    pub fn shared(pool: PgPool, project_storage: ProjectStore) -> SharedProjectOps {
        std::sync::Arc::new(Self { pool, project_storage, http_client: crate::http_client(), shipyard: ShipyardConfig::from_env() })
    }
}

#[async_trait]
impl ProjectOps for ServerProjectOps {
    async fn publish(&self, user_id: Uuid, project_id: Uuid, environment: &str) -> Result<String, String> {
        crate::routes::deploy::require_admin(&self.pool, project_id, user_id).await.map_err(|(_, message)| message)?;
        let row = crate::routes::deploy::start_publish(self, project_id, user_id, environment).await.map_err(|(_, message)| message)?;
        Ok(match row.url {
            Some(url) => format!(
                "Publishing to {} started (files version {}). It takes a few minutes; it will be live at {url}. \
                 The Deployment sheet shows the progress.",
                row.environment, row.files_version
            ),
            None => format!("Publishing to {} started. It takes a few minutes; the Deployment sheet shows the progress.", row.environment),
        })
    }
}
