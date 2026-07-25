use axum::{routing::get, Router};
use sqlx::PgPool;

use crate::auth::extractor::AuthClaims;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub jwt_secret: String,
}

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route(
            "/api/whoami",
            get(|AuthClaims(claims): AuthClaims| async move { axum::Json(claims) }),
        )
        .with_state(state)
}
