//! The Money page: one money month of the user's transactions (from the day their month starts,
//! see nomi_agent_money::period), totalled and broken down.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use chrono::{DateTime, Datelike, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::app::AppState;
use nomi_agent_money::period::{self, Period};
use nomi_auth::extractor::AuthClaims;

const TRANSACTION_LIMIT: i64 = 200;

#[derive(Deserialize)]
pub struct MoneyQuery {
    /// `YYYY-MM`: the money month that starts in that month. Defaults to the current one.
    pub month: Option<String>,
}

#[derive(Serialize)]
pub struct CategoryTotal {
    pub category: String,
    pub cents: i64,
    pub count: i64,
}

#[derive(Serialize)]
pub struct DayTotal {
    pub date: NaiveDate,
    pub cents: i64,
}

#[derive(Serialize)]
pub struct Transaction {
    pub id: Uuid,
    pub occurred_at: DateTime<Utc>,
    pub amount_cents: i64,
    pub category: String,
    pub description: String,
    /// "expense" or "income".
    pub kind: String,
    pub items: Vec<TransactionItem>,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct TransactionItem {
    #[serde(skip)]
    pub transaction_id: Uuid,
    pub name: String,
    pub quantity: f64,
    pub unit_amount_cents: Option<i64>,
    pub amount_cents: i64,
    pub category: Option<String>,
}

#[derive(Serialize)]
pub struct Budget {
    pub category: String,
    pub limit_cents: i64,
    /// Spent in the summary's month.
    pub spent_cents: i64,
}

#[derive(Serialize)]
pub struct MoneySummary {
    /// The money month shown, by the month it starts in (`YYYY-MM`).
    pub month: String,
    /// The day money months start (1 = calendar months).
    pub start_day: u32,
    /// Its first and last day.
    pub period_start: NaiveDate,
    pub period_end: NaiveDate,
    pub timezone: String,
    /// Spending (expenses) in the period.
    pub total_cents: i64,
    /// Money in (income) in the period.
    pub income_cents: i64,
    pub previous_total_cents: i64,
    pub transaction_count: i64,
    pub by_category: Vec<CategoryTotal>,
    pub by_day: Vec<DayTotal>,
    pub transactions: Vec<Transaction>,
    /// Months that have transactions, newest first (at most a year).
    pub months: Vec<String>,
    pub budgets: Vec<Budget>,
}

fn internal(e: sqlx::Error) -> (StatusCode, &'static str) {
    tracing::error!(error = %e, "failed to load money summary");
    (StatusCode::INTERNAL_SERVER_ERROR, "failed to load money")
}

pub async fn money_summary(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Query(query): Query<MoneyQuery>,
) -> Result<Json<MoneySummary>, (StatusCode, &'static str)> {
    let pool = &state.pool;
    let user_id = claims.sub;
    let mut conn = pool.acquire().await.map_err(internal)?;
    let (tz, start_day) = period::settings(&mut conn, user_id).await.map_err(internal)?;
    let timezone = tz.name().to_string();

    let current = match query.month.as_deref().and_then(|m| NaiveDate::parse_from_str(&format!("{m}-01"), "%Y-%m-%d").ok()) {
        Some(date) => Period::starting_in(tz, start_day, date.year(), date.month()),
        None => Period::containing(tz, start_day, Utc::now()),
    };
    let (start, end) = (current.start, current.end);
    let prev_start = current.shifted(tz, -1).start;

    let (total_cents, income_cents, transaction_count): (i64, i64, i64) = sqlx::query_as(
        "SELECT COALESCE(SUM(amount_cents) FILTER (WHERE kind = 'expense'), 0)::bigint, \
                COALESCE(SUM(amount_cents) FILTER (WHERE kind = 'income'), 0)::bigint, COUNT(*) \
         FROM money_transactions WHERE user_id = $1 AND occurred_at >= $2 AND occurred_at < $3",
    )
    .bind(user_id)
    .bind(start)
    .bind(end)
    .fetch_one(pool)
    .await
    .map_err(internal)?;
    let previous_total_cents: i64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(amount_cents), 0)::bigint FROM money_transactions WHERE user_id = $1 AND kind = 'expense' AND occurred_at >= $2 AND occurred_at < $3",
    )
    .bind(user_id)
    .bind(prev_start)
    .bind(start)
    .fetch_one(pool)
    .await
    .map_err(internal)?;

    let by_category: Vec<(String, i64, i64)> = sqlx::query_as(
        "SELECT category, SUM(amount_cents)::bigint, COUNT(*) FROM money_transactions \
         WHERE user_id = $1 AND kind = 'expense' AND occurred_at >= $2 AND occurred_at < $3 GROUP BY category ORDER BY 2 DESC",
    )
    .bind(user_id)
    .bind(start)
    .bind(end)
    .fetch_all(pool)
    .await
    .map_err(internal)?;

    let by_day: Vec<(NaiveDate, i64)> = sqlx::query_as(
        "SELECT (occurred_at AT TIME ZONE $4)::date, SUM(amount_cents)::bigint FROM money_transactions \
         WHERE user_id = $1 AND kind = 'expense' AND occurred_at >= $2 AND occurred_at < $3 GROUP BY 1 ORDER BY 1",
    )
    .bind(user_id)
    .bind(start)
    .bind(end)
    .bind(&timezone)
    .fetch_all(pool)
    .await
    .map_err(internal)?;

    let transactions: Vec<(Uuid, DateTime<Utc>, i64, String, String, String)> = sqlx::query_as(
        "SELECT id, occurred_at, amount_cents, category, description, kind FROM money_transactions \
         WHERE user_id = $1 AND occurred_at >= $2 AND occurred_at < $3 ORDER BY occurred_at DESC LIMIT $4",
    )
    .bind(user_id)
    .bind(start)
    .bind(end)
    .bind(TRANSACTION_LIMIT)
    .fetch_all(pool)
    .await
    .map_err(internal)?;

    let ids: Vec<Uuid> = transactions.iter().map(|t| t.0).collect();
    let mut items: Vec<TransactionItem> = sqlx::query_as(
        "SELECT transaction_id, name, quantity::float8 AS quantity, unit_amount_cents, amount_cents, category \
         FROM money_transaction_items WHERE transaction_id = ANY($1) ORDER BY transaction_id, position",
    )
    .bind(&ids)
    .fetch_all(pool)
    .await
    .map_err(internal)?;

    // Money months with transactions, by the month each starts in: shifting back by the start
    // day lands every date in its period's starting month.
    let months: Vec<String> = sqlx::query_scalar(
        "SELECT to_char(date_trunc('month', (occurred_at AT TIME ZONE $2) - make_interval(days => $3 - 1)), 'YYYY-MM') AS m \
         FROM money_transactions WHERE user_id = $1 GROUP BY m ORDER BY m DESC LIMIT 12",
    )
    .bind(user_id)
    .bind(&timezone)
    .bind(start_day as i32)
    .fetch_all(pool)
    .await
    .map_err(internal)?;

    let budgets = nomi_agent_money::budgets_with_spending(&mut conn, user_id, start, end)
        .await
        .map_err(internal)?
        .into_iter()
        .map(|(category, limit_cents, spent_cents)| Budget { category, limit_cents, spent_cents })
        .collect();

    Ok(Json(MoneySummary {
        month: current.key(),
        start_day,
        period_start: current.first_day(),
        period_end: current.last_day(),
        timezone,
        total_cents,
        income_cents,
        previous_total_cents,
        transaction_count,
        by_category: by_category.into_iter().map(|(category, cents, count)| CategoryTotal { category, cents, count }).collect(),
        by_day: by_day.into_iter().map(|(date, cents)| DayTotal { date, cents }).collect(),
        transactions: transactions
            .into_iter()
            .map(|(id, occurred_at, amount_cents, category, description, kind)| {
                let (mine, rest): (Vec<_>, Vec<_>) = std::mem::take(&mut items).into_iter().partition(|i| i.transaction_id == id);
                items = rest;
                Transaction { id, occurred_at, amount_cents, category, description, kind, items: mine }
            })
            .collect(),
        months,
        budgets,
    }))
}

fn bad_request(message: String) -> (StatusCode, String) {
    (StatusCode::BAD_REQUEST, message)
}

/// Adds a transaction from the Money page. Body: `{kind?, amount?, category, description,
/// occurred_at?, items?}` (see nomi_agent_money::log_transaction).
pub async fn add_transaction(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Json(input): Json<serde_json::Value>,
) -> Result<StatusCode, (StatusCode, String)> {
    let mut conn = state.pool.acquire().await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    nomi_agent_money::log_transaction(&mut conn, claims.sub, &input, "manual").await.map_err(bad_request)?;
    Ok(StatusCode::CREATED)
}

/// Sets a category's monthly budget from the Money page. Body: `{category, monthly_limit}`.
pub async fn set_budget(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Json(input): Json<serde_json::Value>,
) -> Result<StatusCode, (StatusCode, String)> {
    let mut conn = state.pool.acquire().await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    nomi_agent_money::set_budget(&mut conn, claims.sub, &input).await.map_err(bad_request)?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
pub struct PeriodRequest {
    pub start_day: i64,
}

/// `PUT /api/money/period`: the day money months start (1–28).
pub async fn set_period(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Json(req): Json<PeriodRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    let mut conn = state.pool.acquire().await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    period::set_start_day(&mut conn, claims.sub, req.start_day).await.map_err(bad_request)?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn delete_budget(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(category): Path<String>,
) -> Result<StatusCode, (StatusCode, &'static str)> {
    let deleted = sqlx::query("DELETE FROM money_budgets WHERE user_id = $1 AND category = $2")
        .bind(claims.sub)
        .bind(category.to_lowercase())
        .execute(&state.pool)
        .await
        .map_err(internal)?
        .rows_affected();
    if deleted == 0 {
        return Err((StatusCode::NOT_FOUND, "budget not found"));
    }
    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    use nomi_agent_money::period::shift_month;

    #[test]
    fn shifting_months_wraps_years() {
        assert_eq!(shift_month(2026, 1, -1), (2025, 12));
        assert_eq!(shift_month(2026, 12, 1), (2027, 1));
        assert_eq!(shift_month(2026, 10, 0), (2026, 10));
    }
}
