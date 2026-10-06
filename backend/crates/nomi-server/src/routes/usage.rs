//! Billing & usage: the person's own token usage and spend, a month at a time.

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;

use crate::app::AppState;
use nomi_auth::extractor::AuthClaims;
use nomi_usage::{Brief, MonthUsage};

#[derive(Deserialize)]
pub struct UsageQuery {
    /// `YYYY-MM`; defaults to the current month in the person's timezone.
    pub month: Option<String>,
}

fn internal(e: sqlx::Error) -> (StatusCode, String) {
    tracing::error!(error = %e, "failed to read llm usage");
    (StatusCode::INTERNAL_SERVER_ERROR, "failed to read usage".to_string())
}

/// One month of usage, spend and the plan's allowance.
pub async fn usage_month(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Query(query): Query<UsageQuery>,
) -> Result<Json<MonthUsage>, (StatusCode, String)> {
    nomi_usage::month(&state.pool, claims.sub, query.month.as_deref()).await.map(Json).map_err(internal)
}

/// This month's allowance and how much is used, for the navigation drawer.
pub async fn usage_brief(State(state): State<AppState>, AuthClaims(claims): AuthClaims) -> Result<Json<Brief>, (StatusCode, String)> {
    nomi_usage::brief(&state.pool, claims.sub).await.map(Json).map_err(internal)
}
