use axum::{routing::{delete, get, post}, Router};
use sqlx::PgPool;
use tower_http::trace::TraceLayer;

use crate::auth::extractor::AuthClaims;
use crate::routes::auth as auth_routes;
use crate::routes::sessions as sessions_routes;
use crate::routes::settings as settings_routes;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub jwt_secret: String,
    pub http_client: reqwest::Client,
    pub settings_key: [u8; 32],
    pub mqtt_broker_host: String,
    pub mqtt_broker_port: u16,
}

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/api/auth/register", post(auth_routes::register))
        .route("/api/auth/login", post(auth_routes::login_handler))
        .route("/api/auth/refresh", post(auth_routes::refresh_handler))
        .route("/api/auth/logout", post(auth_routes::logout_handler))
        .route(
            "/api/whoami",
            get(|AuthClaims(claims): AuthClaims| async move { axum::Json(claims) }),
        )
        .route("/api/orgs/:org_id/members/:user_id", delete(auth_routes::remove_member))
        .route(
            "/api/sessions",
            post(sessions_routes::create_session).get(sessions_routes::list_sessions),
        )
        .route(
            "/api/sessions/:id/messages",
            get(sessions_routes::list_messages).post(sessions_routes::send_message),
        )
        .route("/api/sessions/:id/ws", get(sessions_routes::session_stream))
        .route(
            "/api/admin/settings/llm",
            get(settings_routes::get_llm_settings).put(settings_routes::put_llm_settings),
        )
        .route(
            "/api/admin/settings/embedding",
            get(settings_routes::get_embedding_settings).put(settings_routes::put_embedding_settings),
        )
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
