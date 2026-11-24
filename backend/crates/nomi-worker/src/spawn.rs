//! Starting every loop, the same way from both binaries.

use std::sync::Arc;

use futures_util::future::BoxFuture;
use futures_util::FutureExt;
use nomi_realtime::MqttPublisher;
use sqlx::PgPool;

/// Every loop `spawn_all` starts, in the order it starts them.
pub const LOOPS: [&str; 10] = ["turns", "delegation", "background", "scheduler", "reminders", "memory", "issues", "billing", "sandboxes", "attachments"];

/// What the loops run with.
#[derive(Clone)]
pub struct WorkerContext {
    pub pool: PgPool,
    pub mqtt_host: String,
    pub mqtt_port: u16,
    /// "nomi-orchestrator" when the server runs the loops, "nomi-worker" standalone. A loop that
    /// publishes gets its own MQTT client, named "<prefix>-<loop>-<uuid>".
    pub client_id_prefix: &'static str,
    pub s3: Option<nomi_storage::S3Config>,
    pub settings_key: [u8; 32],
    pub http_client: reqwest::Client,
    pub database_url: String,
    pub project_storage: nomi_storage::ProjectStore,
}

impl WorkerContext {
    fn mqtt(&self, client: &str) -> MqttPublisher {
        let id = format!("{}-{client}-{}", self.client_id_prefix, uuid::Uuid::new_v4());
        MqttPublisher::connect(&self.mqtt_host, self.mqtt_port, &id)
    }
}

/// Every loop, by name, ready to spawn. Building them runs nothing.
pub fn loops(ctx: WorkerContext) -> Vec<(&'static str, BoxFuture<'static, ()>)> {
    let c = ctx;
    let key = c.settings_key;
    let notification: Arc<dyn nomi_agent_core::NotificationDelivery> = Arc::new(nomi_agent_core::LogOnlyDelivery);
    vec![
        (
            "turns",
            crate::worker::run(c.pool.clone(), c.mqtt("worker"), c.s3.clone(), key, c.http_client.clone(), c.database_url.clone(), c.project_storage.clone()).boxed(),
        ),
        (
            "delegation",
            crate::delegation_worker::run(c.pool.clone(), c.mqtt("delegation-worker"), c.s3.clone(), key, c.http_client.clone(), c.database_url.clone(), c.project_storage.clone())
                .boxed(),
        ),
        ("background", crate::background_worker::run(c.pool.clone(), c.mqtt("background-worker"), c.database_url.clone()).boxed()),
        (
            "scheduler",
            crate::scheduler_worker::run(c.pool.clone(), c.mqtt("scheduler"), c.s3.clone(), key, c.http_client.clone(), c.project_storage.clone(), notification).boxed(),
        ),
        ("reminders", crate::reminders_worker::run(c.pool.clone(), c.mqtt("reminders")).boxed()),
        ("memory", crate::memory_worker::run(c.pool.clone(), key, c.http_client.clone()).boxed()),
        ("issues", crate::issue_worker::run(c.pool.clone()).boxed()),
        // Renewal invoices and reminders for paid plans.
        ("billing", crate::billing_worker::run(c.pool.clone()).boxed()),
        // Server-side sandboxes follow what projects want; idle unless the admin switch is on.
        (
            "sandboxes",
            crate::sandbox_worker::run(c.pool.clone(), c.http_client.clone(), c.project_storage.clone(), key, nomi_shipyard::ShipyardConfig::from_env()).boxed(),
        ),
        ("attachments", crate::attachment_worker::run(c.pool.clone(), key, c.http_client.clone()).boxed()),
    ]
}

/// Spawns every loop on the current runtime and returns their names, in order.
pub fn spawn_all(ctx: WorkerContext) -> Vec<&'static str> {
    loops(ctx)
        .into_iter()
        .map(|(name, run)| {
            tokio::spawn(run);
            name
        })
        .collect()
}

/// Runs every loop until the first one ends (returns or panics), and gives its name. The
/// standalone worker exits then, so whatever supervises it starts a fresh process.
pub async fn first_to_end(loops: Vec<(&'static str, BoxFuture<'static, ()>)>) -> &'static str {
    let mut running = tokio::task::JoinSet::new();
    for (name, run) in loops {
        running.spawn(async move {
            // A panic still names its loop: it's caught here instead of only in the JoinError.
            let _ = std::panic::AssertUnwindSafe(run).catch_unwind().await;
            name
        });
    }
    running.join_next().await.and_then(Result::ok).unwrap_or("none")
}
