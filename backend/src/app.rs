use axum::{
    routing::{delete, get, post},
    Router,
};
use sqlx::PgPool;

use crate::auth::extractor::AuthClaims;
use crate::routes::auth as auth_routes;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub jwt_secret: String,
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
        .with_state(state)
}
