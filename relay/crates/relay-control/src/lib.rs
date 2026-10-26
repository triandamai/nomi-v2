//! Relay's control plane: what the Nomi backend asks of it, service to service — making and
//! deleting a project's backend, applying its migrations, reading its schema and types.

pub mod backend;
pub mod migrations;
pub mod schema;
pub mod state;

use std::collections::HashMap;

use relay_core::{Env, Tier};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use tokio::sync::Mutex;
use uuid::Uuid;

/// Connections to one dedicated project's database.
const DEDICATED_POOL_SIZE: u32 = 5;

/// What went wrong, in the words the control plane answers with.
#[derive(Debug)]
pub enum RelayError {
    NotFound(String),
    /// The request can't be done as asked (a migration failed, lint, a bad tier): its code,
    /// message and, for a migration, `{ file, line }`.
    Rejected { code: &'static str, message: String, details: serde_json::Value },
    Internal(String),
}

impl RelayError {
    pub fn rejected(code: &'static str, message: impl Into<String>) -> Self {
        RelayError::Rejected { code, message: message.into(), details: serde_json::Value::Null }
    }
}

impl std::fmt::Display for RelayError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RelayError::NotFound(m) | RelayError::Internal(m) => f.write_str(m),
            RelayError::Rejected { message, .. } => f.write_str(message),
        }
    }
}

impl From<sqlx::Error> for RelayError {
    fn from(e: sqlx::Error) -> Self {
        RelayError::Internal(e.to_string())
    }
}

/// Relay's state and the databases projects live in.
pub struct Relay {
    pub state: PgPool,
    shared: HashMap<Env, PgPool>,
    pub encryption_key: [u8; 32],
    pub public_domain: String,
    dedicated: Mutex<HashMap<String, PgPool>>,
}

/// Where one environment of a project lives: the database and the schema its tables are in.
pub struct Placement {
    pub pool: PgPool,
    pub schema: String,
    pub tier: Tier,
}

impl Relay {
    pub fn new(state: PgPool, shared: HashMap<Env, PgPool>, encryption_key: [u8; 32], public_domain: String) -> Self {
        Self { state, shared, encryption_key, public_domain, dedicated: Mutex::new(HashMap::new()) }
    }

    pub(crate) fn shared(&self, env: Env) -> Result<PgPool, RelayError> {
        self.shared.get(&env).cloned().ok_or_else(|| RelayError::Internal(format!("no shared database for {}", env.as_str())))
    }

    pub(crate) async fn dedicated(&self, url: &str) -> Result<PgPool, RelayError> {
        let mut pools = self.dedicated.lock().await;
        if let Some(pool) = pools.get(url) {
            return Ok(pool.clone());
        }
        let pool = PgPoolOptions::new().max_connections(DEDICATED_POOL_SIZE).connect(url).await?;
        pools.insert(url.to_string(), pool.clone());
        Ok(pool)
    }

    /// Where `project`'s `env` backend lives.
    pub async fn placement(&self, project: Uuid, env: Env) -> Result<Placement, RelayError> {
        let row = state::environment(self, project, env).await?.ok_or_else(|| RelayError::NotFound("no backend for this project".into()))?;
        let pool = match &row.database_url {
            Some(url) => self.dedicated(url).await?,
            None => self.shared(env)?,
        };
        Ok(Placement { pool, schema: relay_core::names::schema_for(project, row.tier), tier: row.tier })
    }
}
