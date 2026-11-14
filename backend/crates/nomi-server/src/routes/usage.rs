//! Billing & usage: the person's own token usage and spend, a month at a time.

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};

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

/// Rupiah to the dollar when Admin → Payment doesn't say.
pub const DEFAULT_USD_TO_IDR: u64 = 16_000;

/// The rate spend (kept in dollars) is shown in rupiah at: Admin → Payment's, else the default.
pub async fn usd_to_idr() -> u64 {
    nomi_settings::config::current(&nomi_settings::config::XENDIT).await.number("usd_to_idr").unwrap_or(DEFAULT_USD_TO_IDR)
}

#[derive(Serialize)]
pub struct MonthUsageView {
    #[serde(flatten)]
    pub usage: MonthUsage,
    /// How they paid last ("BANK_TRANSFER · BCA"); None when they never have.
    pub payment_method: Option<String>,
    /// What they paid for plans in the month, in rupiah.
    pub paid_idr: i64,
    /// Rupiah to the dollar, to show `spend_usd` in rupiah.
    pub usd_to_idr: u64,
}

/// One month of usage, spend and the plan's allowance, with what was paid.
pub async fn usage_month(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Query(query): Query<UsageQuery>,
) -> Result<Json<MonthUsageView>, (StatusCode, String)> {
    let usage = nomi_usage::month(&state.pool, claims.sub, query.month.as_deref()).await.map_err(internal)?;
    let paid_idr: i64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(amount_idr), 0)::bigint FROM plan_payments WHERE user_id = $1 AND status = 'paid' \
           AND paid_at >= (($3 || '-01')::timestamp AT TIME ZONE $2) \
           AND paid_at < ((($3 || '-01')::timestamp + interval '1 month') AT TIME ZONE $2)",
    )
    .bind(claims.sub)
    .bind(&usage.timezone)
    .bind(&usage.month)
    .fetch_one(&state.pool)
    .await
    .map_err(internal)?;
    let payment_method: Option<String> = sqlx::query_scalar(
        "SELECT payment_method FROM plan_payments WHERE user_id = $1 AND status = 'paid' AND payment_method IS NOT NULL ORDER BY paid_at DESC LIMIT 1",
    )
    .bind(claims.sub)
    .fetch_optional(&state.pool)
    .await
    .map_err(internal)?;
    Ok(Json(MonthUsageView { usage, payment_method, paid_idr, usd_to_idr: usd_to_idr().await }))
}

/// This month's allowance and how much is used, for the navigation drawer.
pub async fn usage_brief(State(state): State<AppState>, AuthClaims(claims): AuthClaims) -> Result<Json<Brief>, (StatusCode, String)> {
    nomi_usage::brief(&state.pool, claims.sub).await.map(Json).map_err(internal)
}
