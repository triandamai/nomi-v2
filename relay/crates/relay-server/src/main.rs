//! `relay-server`: reads its settings (see `relay_core::config`), brings its state up to date,
//! and serves the control plane.

use std::collections::HashMap;
use std::sync::Arc;

use relay_control::Relay;
use relay_core::{Config, Env};
use sqlx::postgres::PgPoolOptions;

/// Connections to Relay's state and to each shared database.
const POOL_SIZE: u32 = 10;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().with_env_filter(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into())).init();
    let config = Config::from_env().unwrap_or_else(|e| panic!("{e}"));
    let connect = |url: String| async move { PgPoolOptions::new().max_connections(POOL_SIZE).connect(&url).await.expect("couldn't reach a database") };
    let state = connect(config.database_url.clone()).await;
    sqlx::migrate!("../../migrations").run(&state).await.expect("couldn't bring Relay's state up to date");
    let shared = HashMap::from([
        (Env::Dev, connect(config.shared_dev_url.clone()).await),
        (Env::Production, connect(config.shared_production_url.clone()).await),
    ]);
    let relay = Arc::new(Relay::new(state, shared, config.encryption_key, config.public_domain.clone()));
    let app = relay_server::app::router(relay, config.control_key.clone());
    let listener = tokio::net::TcpListener::bind(&config.bind).await.expect("couldn't bind");
    tracing::info!(bind = %config.bind, "relay listening");
    axum::serve(listener, app).await.expect("server failed");
}
