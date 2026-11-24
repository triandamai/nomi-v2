use std::env::var;

use nomi_settings as settings;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,worker=info,nomi_server=info,nomi_turn=info,nomi_agent_core=info,tower_http=info".into()),
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

    // Admin → Configuration is read from here on; the environment fills what isn't saved there.
    nomi_settings::config::install(pool.clone());
    let s3 = nomi_server::configuration::storage().await;
    let project_storage = nomi_storage::build_project_store(s3.clone());
    // Turns open attached files from the same place the server stores them.
    nomi_storage::blob::init_attachment_store(s3.clone());
    // Quota notices from turns are emailed through the same SMTP settings as the server's.
    nomi_server::notifications::init_mailer(nomi_mail::configured());
    nomi_server::project_events::init(&mqtt_broker_host, mqtt_broker_port);
    nomi_server::user_events::init(&mqtt_broker_host, mqtt_broker_port);
    // Chat replies and quota notices reach phones through the same push setup as the server's.
    nomi_server::push::init(nomi_server::push::from_env(http_client.clone()));
    nomi_server::public_url::warn_if_unknown();

    // Every loop, as the server runs them inline.
    nomi_worker::spawn::spawn_all(nomi_worker::spawn::WorkerContext {
        pool,
        mqtt_host: mqtt_broker_host,
        mqtt_port: mqtt_broker_port,
        client_id_prefix: "nomi-worker",
        s3,
        settings_key,
        http_client,
        database_url,
        project_storage,
    });
    std::future::pending::<()>().await;
}
