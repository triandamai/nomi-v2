use std::sync::Arc;

use axum::{routing::{delete, get, post}, Router};
use sqlx::PgPool;

use crate::auth::extractor::AuthClaims;
use crate::embedding::EmbeddingProvider;
use crate::llm::LlmProvider;
use crate::routes::auth as auth_routes;
use crate::routes::sessions as sessions_routes;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub jwt_secret: String,
    pub provider: Arc<dyn LlmProvider>,
    pub embedding_provider: Arc<dyn EmbeddingProvider>,
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
        .with_state(state)
}
