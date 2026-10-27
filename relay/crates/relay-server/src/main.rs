//! `relay-server`: reads its three settings from the environment (see `relay_core::config`),
//! brings its state up to date, loads what was saved from Nomi's admin, and serves the control
//! plane and the gateway.

use std::sync::Arc;

use relay_control::Relay;
use relay_core::Config;
use sqlx::postgres::PgPoolOptions;

/// Connections to Relay's state.
const POOL_SIZE: u32 = 10;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().with_env_filter(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into())).init();
    let config = Config::from_env().unwrap_or_else(|e| panic!("{e}"));
    let state = PgPoolOptions::new().max_connections(POOL_SIZE).connect(&config.database_url).await.expect("couldn't reach Relay's state database");
    sqlx::migrate!("../../migrations").run(&state).await.expect("couldn't bring Relay's state up to date");
    let relay = match config.standalone.clone() {
        Some(standalone) => {
            tracing::info!(project = %standalone.project, env = standalone.env.as_str(), "relay serving one project");
            Relay::start_standalone(state, config.encryption_key, config.database_url.clone(), standalone).await.expect("couldn't set up standalone Relay")
        }
        None => Relay::start(state, config.encryption_key).await.expect("couldn't load Relay's settings"),
    };
    if relay.live().is_err() {
        tracing::warn!("Relay isn't set up yet: save its settings in Nomi's admin");
    }
    let relay = Arc::new(relay);
    relay_server::schedules::start(relay.clone());
    let app = relay_server::app::router(relay, config.control_key.clone());
    let listener = tokio::net::TcpListener::bind(&config.bind).await.expect("couldn't bind");
    tracing::info!(bind = %config.bind, "relay listening");
    axum::serve(listener, app).await.expect("server failed");
}
