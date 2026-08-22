use std::env::var;

use nomi_orchestrator::realtime::MqttPublisher;
use nomi_orchestrator::settings;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "nomi_orchestrator=debug,info".into()),
        )
        .init();

    let database_url = var("DATABASE_URL").expect("DATABASE_URL must be set");
    let settings_key = settings::crypto::parse_key(
        &var("SETTINGS_ENCRYPTION_KEY").expect("SETTINGS_ENCRYPTION_KEY must be set"),
    )
    .expect("SETTINGS_ENCRYPTION_KEY must be 64 hex characters (32 bytes)");
    let mqtt_broker_host = var("MQTT_BROKER_HOST").unwrap_or_else(|_| "localhost".to_string());
    let mqtt_broker_port: u16 = var("MQTT_BROKER_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(1883);

    let pool = sqlx::PgPool::connect(&database_url).await.expect("failed to connect to database");
    tracing::info!("connected to database");
    sqlx::migrate!().run(&pool).await.expect("failed to run migrations");
    tracing::info!("migrations up to date");

    let http_client = reqwest::Client::new();
    let mqtt_client_id = format!("nomi-worker-{}", uuid::Uuid::new_v4());
    let mqtt = MqttPublisher::connect(&mqtt_broker_host, mqtt_broker_port, &mqtt_client_id);

    nomi_orchestrator::worker::run(pool, mqtt, settings_key, http_client, database_url).await;
}
