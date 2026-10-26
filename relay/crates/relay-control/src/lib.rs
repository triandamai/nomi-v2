//! Relay's control plane: what the Nomi backend asks of it, service to service — making and
//! deleting a project's backend, applying its migrations, reading its schema and types.

pub mod backend;
pub mod migrations;
pub mod postgrest;
pub mod schema;
pub mod settings;
pub mod state;

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use relay_auth::mail::{Outbox, SmtpOutbox};
use relay_core::settings::Settings;
use relay_core::{Env, Tier};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use tokio::sync::Mutex;
use uuid::Uuid;

/// Connections to one dedicated project's database.
const DEDICATED_POOL_SIZE: u32 = 5;

/// The answer until Relay's settings have been saved.
pub const NOT_CONFIGURED: &str = "NOT_CONFIGURED";

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

/// Where one environment of a project lives: the database and the schema its tables are in.
pub struct Placement {
    pub pool: PgPool,
    pub schema: String,
    pub tier: Tier,
}

/// Connections to each shared database.
const SHARED_POOL_SIZE: u32 = 10;

/// Relay's state, and what its saved settings point at.
pub struct Relay {
    pub state: PgPool,
    pub encryption_key: [u8; 32],
    live: RwLock<Option<Arc<Live>>>,
    dedicated: Mutex<HashMap<String, PgPool>>,
    /// Where sign-in codes go out (SMTP; a recording outbox in tests).
    pub outbox: Arc<dyn Outbox>,
}

/// Relay's settings and the shared databases they name. Swapped whole when settings are saved.
pub struct Live {
    pub settings: Settings,
    shared: HashMap<Env, PgPool>,
}

impl Live {
    /// Connects lazily, so Relay starts even while a shared database is down.
    pub fn connect(settings: Settings) -> Result<Self, RelayError> {
        let mut shared = HashMap::new();
        for env in Env::ALL {
            let pool = PgPoolOptions::new().max_connections(SHARED_POOL_SIZE).connect_lazy(settings.shared_url(env))?;
            shared.insert(env, pool);
        }
        Ok(Self { settings, shared })
    }

    pub fn shared(&self, env: Env) -> Result<PgPool, RelayError> {
        self.shared.get(&env).cloned().ok_or_else(|| RelayError::Internal(format!("no shared database for {}", env.as_str())))
    }
}

impl Relay {
    /// A Relay already pointed at `shared`, as the tests (and only they) make one.
    pub fn new(state: PgPool, shared: HashMap<Env, PgPool>, encryption_key: [u8; 32], public_domain: String) -> Self {
        let live = Live { settings: Settings { public_domain, ..Settings::default() }, shared };
        Self { state, encryption_key, live: RwLock::new(Some(Arc::new(live))), dedicated: Mutex::new(HashMap::new()), outbox: Arc::new(SmtpOutbox) }
    }

    /// A Relay whose settings haven't been saved yet.
    pub fn unconfigured(state: PgPool, encryption_key: [u8; 32]) -> Self {
        Self { state, encryption_key, live: RwLock::new(None), dedicated: Mutex::new(HashMap::new()), outbox: Arc::new(SmtpOutbox) }
    }

    /// Relay as its saved settings describe it, or unconfigured when there are none.
    pub async fn start(state: PgPool, encryption_key: [u8; 32]) -> Result<Self, RelayError> {
        let relay = Self::unconfigured(state, encryption_key);
        if let Some(saved) = settings::load(&relay).await? {
            relay.replace(Live::connect(saved)?);
        }
        Ok(relay)
    }

    /// Changes the settings in place (tests point the gateway at a stand-in PostgREST this way).
    pub fn with_settings(self, change: impl FnOnce(&mut Settings)) -> Self {
        {
            let mut live = self.live.write().expect("settings lock");
            if let Some(current) = live.take() {
                let mut settings = current.settings.clone();
                change(&mut settings);
                *live = Some(Arc::new(Live { settings, shared: current.shared.clone() }));
            }
        }
        self
    }

    pub fn with_outbox(mut self, outbox: Arc<dyn Outbox>) -> Self {
        self.outbox = outbox;
        self
    }

    pub fn with_postgrest_role(self, role: impl Into<String>) -> Self {
        let role = role.into();
        self.with_settings(|s| s.postgrest_role = role)
    }

    /// The current settings and shared databases; `NOT_CONFIGURED` until saved from Nomi's admin.
    pub fn live(&self) -> Result<Arc<Live>, RelayError> {
        self.live
            .read()
            .expect("settings lock")
            .clone()
            .ok_or_else(|| RelayError::Rejected { code: NOT_CONFIGURED, message: "Relay isn't set up yet: save its settings in Nomi's admin".into(), details: serde_json::Value::Null })
    }

    pub(crate) fn replace(&self, live: Live) {
        *self.live.write().expect("settings lock") = Some(Arc::new(live));
    }

    pub(crate) fn shared(&self, env: Env) -> Result<PgPool, RelayError> {
        self.live()?.shared(env)
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
