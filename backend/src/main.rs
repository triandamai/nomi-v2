use std::env::var;
use std::sync::Arc;

use nomi_orchestrator::bootstrap::{build_embedding_provider_from_settings_or_env, build_llm_provider_from_settings_or_env};
use nomi_orchestrator::settings;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "nomi_orchestrator=debug,tower_http=debug,info".into()),
        )
        .init();

    let database_url = var("DATABASE_URL").expect("DATABASE_URL must be set");
    let jwt_secret = var("JWT_SECRET").expect("JWT_SECRET must be set");
    let settings_key = settings::crypto::parse_key(
        &var("SETTINGS_ENCRYPTION_KEY").expect("SETTINGS_ENCRYPTION_KEY must be set"),
    )
    .expect("SETTINGS_ENCRYPTION_KEY must be 64 hex characters (32 bytes)");

    let pool = sqlx::PgPool::connect(&database_url).await.expect("failed to connect to database");
    tracing::info!("connected to database");
    sqlx::migrate!().run(&pool).await.expect("failed to run migrations");
    tracing::info!("migrations up to date");

    let http_client = reqwest::Client::new();

    let provider = build_llm_provider_from_settings_or_env(&pool, &settings_key, http_client.clone()).await;
    let embedding_provider =
        build_embedding_provider_from_settings_or_env(&pool, &settings_key, http_client.clone()).await;
    tracing::info!("llm and embedding providers ready");

    let state = nomi_orchestrator::app::AppState {
        pool,
        jwt_secret,
        provider: Arc::new(tokio::sync::RwLock::new(provider)),
        embedding_provider: Arc::new(tokio::sync::RwLock::new(embedding_provider)),
        http_client,
        settings_key,
    };
    let app = nomi_orchestrator::app::build_router(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.expect("failed to bind to port 8080");
    tracing::info!("listening on 0.0.0.0:8080");
    axum::serve(listener, app).await.expect("server error");
}
