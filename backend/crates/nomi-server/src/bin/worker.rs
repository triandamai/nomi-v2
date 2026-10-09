use std::env::var;

use nomi_realtime::MqttPublisher;
use nomi_settings as settings;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "worker=debug,nomi_server=debug,info".into()),
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
    sqlx::migrate!("../../migrations").run(&pool).await.expect("failed to run migrations");
    tracing::info!("migrations up to date");

    let http_client = nomi_server::http_client();
    let mqtt_client_id = format!("nomi-worker-{}", uuid::Uuid::new_v4());
    let mqtt = MqttPublisher::connect(&mqtt_broker_host, mqtt_broker_port, &mqtt_client_id);

    let s3 = nomi_storage::build_from_env().await;
    let project_storage = nomi_storage::build_local_fs_store();
    // Turns open attached files from the same place the server stores them.
    nomi_storage::blob::init_attachment_store(s3.clone());

    // Memory learning, chat summaries and tidying run beside the turn worker, as they do inline.
    tokio::spawn(nomi_server::memory_worker::run(pool.clone(), settings_key, http_client.clone()));
    // Attached images, audio and video are read by a model in the background.
    tokio::spawn(nomi_server::attachment_worker::run(pool.clone(), settings_key, http_client.clone()));

    nomi_server::worker::run(pool, mqtt, s3, settings_key, http_client, database_url, project_storage).await;
}
