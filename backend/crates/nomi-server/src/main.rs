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

    let http_client = nomi_server::http_client();

    let s3 = nomi_storage::build_from_env().await;
    if s3.is_some() {
        tracing::info!("S3 storage configured for avatars, attachments and project files");
    } else {
        tracing::info!("S3_BUCKET not set — avatar upload disabled, attachments and project files kept on disk");
    }

    // Notifications are emailed through the same SMTP settings as sign-in codes.
    nomi_server::notifications::init_mailer(nomi_mail::from_env());

    // Chat attachments go to the same bucket as avatars, or to disk without one.
    nomi_storage::blob::init_attachment_store(s3.clone());

    let project_storage = nomi_storage::build_project_store(s3.clone());

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
        let worker_s3 = s3.clone();
        let worker_http_client = http_client.clone();
        let worker_database_url = database_url.clone();
        let worker_project_storage = project_storage.clone();
        tokio::spawn(async move {
            nomi_server::worker::run(worker_pool, worker_mqtt, worker_s3, settings_key, worker_http_client, worker_database_url, worker_project_storage).await;
        });

        let delegation_mqtt_client_id = format!("nomi-orchestrator-delegation-worker-{}", uuid::Uuid::new_v4());
        let delegation_mqtt = MqttPublisher::connect(&mqtt_broker_host, mqtt_broker_port, &delegation_mqtt_client_id);
        let delegation_pool = pool.clone();
        let delegation_s3 = s3.clone();
        let delegation_http_client = http_client.clone();
        let delegation_database_url = database_url.clone();
        let delegation_project_storage = project_storage.clone();
        tokio::spawn(async move {
            nomi_server::delegation_worker::run(delegation_pool, delegation_mqtt, delegation_s3, settings_key, delegation_http_client, delegation_database_url, delegation_project_storage).await;
        });

        let scheduler_mqtt_client_id = format!("nomi-orchestrator-scheduler-{}", uuid::Uuid::new_v4());
        let scheduler_mqtt = MqttPublisher::connect(&mqtt_broker_host, mqtt_broker_port, &scheduler_mqtt_client_id);
        let scheduler_pool = pool.clone();
        let scheduler_s3 = s3.clone();
        let scheduler_http_client = http_client.clone();
        let scheduler_project_storage = project_storage.clone();
        let notification: std::sync::Arc<dyn nomi_agent_core::NotificationDelivery> = std::sync::Arc::new(nomi_agent_core::LogOnlyDelivery);
        tokio::spawn(async move {
            nomi_server::scheduler_worker::run(scheduler_pool, scheduler_mqtt, scheduler_s3, settings_key, scheduler_http_client, scheduler_project_storage, notification).await;
        });
        let reminders_mqtt_client_id = format!("nomi-orchestrator-reminders-{}", uuid::Uuid::new_v4());
        let reminders_mqtt = MqttPublisher::connect(&mqtt_broker_host, mqtt_broker_port, &reminders_mqtt_client_id);
        let reminders_pool = pool.clone();
        tokio::spawn(async move {
            nomi_server::reminders_worker::run(reminders_pool, reminders_mqtt).await;
        });
        let memory_pool = pool.clone();
        let memory_http_client = http_client.clone();
        tokio::spawn(async move {
            nomi_server::memory_worker::run(memory_pool, settings_key, memory_http_client).await;
        });
        let attachment_pool = pool.clone();
        let attachment_http_client = http_client.clone();
        tokio::spawn(async move {
            nomi_server::attachment_worker::run(attachment_pool, settings_key, attachment_http_client).await;
        });
        tracing::info!("embedded worker enabled (set RUN_WORKER_INLINE=false to disable)");
    } else {
        tracing::info!("embedded worker disabled (RUN_WORKER_INLINE=false); run `cargo run --bin worker` separately");
    }

    let tool_catalog = nomi_server::build_tool_catalog(project_storage.clone());

    let state = nomi_server::app::AppState {
        pool,
        jwt_secret,
        http_client,
        settings_key,
        mqtt_broker_host,
        mqtt_broker_port,
        s3,
        project_storage,
        tool_catalog,
        email_codes: nomi_server::sign_in_codes::EmailCodes::from_env(),
    };
    let app = nomi_server::app::build_router(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.expect("failed to bind to port 8080");
    match nomi_agent_workspace::connection::GoogleConfig::from_env() {
        // Both must be listed exactly under the Google OAuth client's authorized redirect URIs.
        Some(google) => tracing::info!(
            sign_in_redirect_uri = %google.signin_redirect_uri,
            workspace_redirect_uri = %google.redirect_uri,
            "Google sign-in and Workspace enabled"
        ),
        None => tracing::info!("Google not configured (GOOGLE_CLIENT_ID, GOOGLE_CLIENT_SECRET, GOOGLE_REDIRECT_URI)"),
    }
    tracing::info!("listening on 0.0.0.0:8080");
    axum::serve(listener, app).await.expect("server error");
}
