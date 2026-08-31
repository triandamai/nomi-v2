use std::env::var;

use nomi_realtime::MqttPublisher;
use nomi_settings as settings;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "nomi_orchestrator=debug,nomi_server=debug,tower_http=debug,info".into()),
        )
        .init();

    let database_url = var("DATABASE_URL").expect("DATABASE_URL must be set");
    let jwt_secret = var("JWT_SECRET").expect("JWT_SECRET must be set");
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

    let http_client = reqwest::Client::new();

    let s3 = nomi_storage::build_from_env().await;
    if s3.is_some() {
        tracing::info!("S3 avatar storage configured");
    } else {
        tracing::info!("S3_BUCKET not set — avatar upload disabled");
    }

    // Embedded by default so a single `cargo run` (or single production instance) is enough to
    // process turns — no separate `cargo run --bin worker` process required. Set
    // RUN_WORKER_INLINE=false to disable this and run the worker as its own process(es) instead,
    // e.g. for horizontal worker scaling independent of HTTP server instances (see
    // docs/superpowers/specs/2026-08-11-mqtt-turn-queue-design.md for why the two were split
    // into separate binaries in the first place — that separation still works, this default
    // just avoids requiring it for the common single-instance case).
    let run_worker_inline = var("RUN_WORKER_INLINE").map(|v| v != "false" && v != "0").unwrap_or(true);
    if run_worker_inline {
        let worker_mqtt_client_id = format!("nomi-orchestrator-worker-{}", uuid::Uuid::new_v4());
        let worker_mqtt = MqttPublisher::connect(&mqtt_broker_host, mqtt_broker_port, &worker_mqtt_client_id);
        let worker_pool = pool.clone();
        let worker_http_client = http_client.clone();
        let worker_database_url = database_url.clone();
        tokio::spawn(async move {
            nomi_server::worker::run(worker_pool, worker_mqtt, settings_key, worker_http_client, worker_database_url).await;
        });

        let delegation_mqtt_client_id = format!("nomi-orchestrator-delegation-worker-{}", uuid::Uuid::new_v4());
        let delegation_mqtt = MqttPublisher::connect(&mqtt_broker_host, mqtt_broker_port, &delegation_mqtt_client_id);
        let delegation_pool = pool.clone();
        let delegation_http_client = http_client.clone();
        let delegation_database_url = database_url.clone();
        tokio::spawn(async move {
            nomi_server::delegation_worker::run(delegation_pool, delegation_mqtt, settings_key, delegation_http_client, delegation_database_url).await;
        });
        tracing::info!("embedded worker enabled (set RUN_WORKER_INLINE=false to disable)");
    } else {
        tracing::info!("embedded worker disabled (RUN_WORKER_INLINE=false); run `cargo run --bin worker` separately");
    }

    let state = nomi_server::app::AppState {
        pool,
        jwt_secret,
        http_client,
        settings_key,
        mqtt_broker_host,
        mqtt_broker_port,
        s3,
    };
    let app = nomi_server::app::build_router(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.expect("failed to bind to port 8080");
    tracing::info!("listening on 0.0.0.0:8080");
    axum::serve(listener, app).await.expect("server error");
}
