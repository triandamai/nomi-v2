use std::sync::Arc;

#[allow(unused_imports)]
use axum::{routing::{delete, get, post, put}, Router};
use sqlx::PgPool;
use tokio::sync::RwLock;

use crate::auth::extractor::AuthClaims;
use crate::embedding::EmbeddingProvider;
use crate::llm::LlmProvider;
use crate::routes::auth as auth_routes;
use crate::routes::sessions as sessions_routes;
#[allow(unused_imports)]
use crate::routes::settings as settings_routes;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub jwt_secret: String,
    pub provider: Arc<RwLock<Arc<dyn LlmProvider>>>,
    pub embedding_provider: Arc<RwLock<Arc<dyn EmbeddingProvider>>>,
    pub http_client: reqwest::Client,
    pub settings_key: [u8; 32],
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
        // TODO(Task 5): uncomment once routes::settings has real handlers.
        // .route(
        //     "/api/admin/settings/llm",
        //     get(settings_routes::get_llm_settings).put(settings_routes::put_llm_settings),
        // )
        // .route(
        //     "/api/admin/settings/embedding",
        //     get(settings_routes::get_embedding_settings).put(settings_routes::put_embedding_settings),
        // )
        .with_state(state)
}
