//! The Money page: one month of the user's transactions, totalled and broken down.

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::Json;
use chrono::{DateTime, Datelike, NaiveDate, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::app::AppState;
use nomi_auth::extractor::AuthClaims;

const TRANSACTION_LIMIT: i64 = 200;

#[derive(Deserialize)]
pub struct MoneyQuery {
    /// `YYYY-MM`; defaults to the current month in the user's timezone.
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
}

#[derive(Serialize)]
pub struct MoneySummary {
    pub month: String,
    pub timezone: String,
    pub total_cents: i64,
    pub previous_total_cents: i64,
    pub transaction_count: i64,
    pub by_category: Vec<CategoryTotal>,
    pub by_day: Vec<DayTotal>,
    pub transactions: Vec<Transaction>,
    /// Months that have transactions, newest first (at most a year).
    pub months: Vec<String>,
}

fn internal(e: sqlx::Error) -> (StatusCode, &'static str) {
    tracing::error!(error = %e, "failed to load money summary");
    (StatusCode::INTERNAL_SERVER_ERROR, "failed to load money")
}

fn month_start(tz: chrono_tz::Tz, year: i32, month: u32) -> DateTime<Utc> {
    let date = NaiveDate::from_ymd_opt(year, month, 1).unwrap_or_default();
    tz.from_local_datetime(&date.and_hms_opt(0, 0, 0).unwrap_or_default()).earliest().map(|t| t.with_timezone(&Utc)).unwrap_or_default()
}

fn shift_month(year: i32, month: u32, by: i32) -> (i32, u32) {
    let index = year * 12 + month as i32 - 1 + by;
    (index.div_euclid(12), (index.rem_euclid(12) + 1) as u32)
}

pub async fn money_summary(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Query(query): Query<MoneyQuery>,
) -> Result<Json<MoneySummary>, (StatusCode, &'static str)> {
    let pool = &state.pool;
    let user_id = claims.sub;
    let timezone: String = sqlx::query_scalar("SELECT timezone FROM user_preferences WHERE user_id = $1")
        .bind(user_id)
        .fetch_optional(pool)
        .await
        .map_err(internal)?
        .unwrap_or_else(|| "UTC".to_string());
    let tz: chrono_tz::Tz = timezone.parse().unwrap_or(chrono_tz::UTC);

    let now_local = Utc::now().with_timezone(&tz);
    let (year, month) = match query.month.as_deref().and_then(|m| NaiveDate::parse_from_str(&format!("{m}-01"), "%Y-%m-%d").ok()) {
        Some(date) => (date.year(), date.month()),
        None => (now_local.year(), now_local.month()),
    };
    let start = month_start(tz, year, month);
    let (next_year, next_month) = shift_month(year, month, 1);
    let end = month_start(tz, next_year, next_month);
    let (prev_year, prev_month) = shift_month(year, month, -1);
    let prev_start = month_start(tz, prev_year, prev_month);

    let (total_cents, transaction_count): (i64, i64) = sqlx::query_as(
        "SELECT COALESCE(SUM(amount_cents), 0)::bigint, COUNT(*) FROM mock_transactions WHERE user_id = $1 AND occurred_at >= $2 AND occurred_at < $3",
    )
    .bind(user_id)
    .bind(start)
    .bind(end)
    .fetch_one(pool)
    .await
    .map_err(internal)?;
    let previous_total_cents: i64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(amount_cents), 0)::bigint FROM mock_transactions WHERE user_id = $1 AND occurred_at >= $2 AND occurred_at < $3",
    )
    .bind(user_id)
    .bind(prev_start)
    .bind(start)
    .fetch_one(pool)
    .await
    .map_err(internal)?;

    let by_category: Vec<(String, i64, i64)> = sqlx::query_as(
        "SELECT category, SUM(amount_cents)::bigint, COUNT(*) FROM mock_transactions \
         WHERE user_id = $1 AND occurred_at >= $2 AND occurred_at < $3 GROUP BY category ORDER BY 2 DESC",
    )
    .bind(user_id)
    .bind(start)
    .bind(end)
    .fetch_all(pool)
    .await
    .map_err(internal)?;

    let by_day: Vec<(NaiveDate, i64)> = sqlx::query_as(
        "SELECT (occurred_at AT TIME ZONE $4)::date, SUM(amount_cents)::bigint FROM mock_transactions \
         WHERE user_id = $1 AND occurred_at >= $2 AND occurred_at < $3 GROUP BY 1 ORDER BY 1",
    )
    .bind(user_id)
    .bind(start)
    .bind(end)
    .bind(&timezone)
    .fetch_all(pool)
    .await
    .map_err(internal)?;

    let transactions: Vec<(Uuid, DateTime<Utc>, i64, String, String)> = sqlx::query_as(
        "SELECT id, occurred_at, amount_cents, category, description FROM mock_transactions \
         WHERE user_id = $1 AND occurred_at >= $2 AND occurred_at < $3 ORDER BY occurred_at DESC LIMIT $4",
    )
    .bind(user_id)
    .bind(start)
    .bind(end)
    .bind(TRANSACTION_LIMIT)
    .fetch_all(pool)
    .await
    .map_err(internal)?;

    let months: Vec<String> = sqlx::query_scalar(
        "SELECT to_char(date_trunc('month', occurred_at AT TIME ZONE $2), 'YYYY-MM') AS m FROM mock_transactions \
         WHERE user_id = $1 GROUP BY m ORDER BY m DESC LIMIT 12",
    )
    .bind(user_id)
    .bind(&timezone)
    .fetch_all(pool)
    .await
    .map_err(internal)?;

    Ok(Json(MoneySummary {
        month: format!("{year:04}-{month:02}"),
        timezone,
        total_cents,
        previous_total_cents,
        transaction_count,
        by_category: by_category.into_iter().map(|(category, cents, count)| CategoryTotal { category, cents, count }).collect(),
        by_day: by_day.into_iter().map(|(date, cents)| DayTotal { date, cents }).collect(),
        transactions: transactions
            .into_iter()
            .map(|(id, occurred_at, amount_cents, category, description)| Transaction { id, occurred_at, amount_cents, category, description })
            .collect(),
        months,
    }))
}

#[cfg(test)]
mod tests {
    use super::shift_month;

    #[test]
    fn shifting_months_wraps_years() {
        assert_eq!(shift_month(2026, 1, -1), (2025, 12));
        assert_eq!(shift_month(2026, 12, 1), (2027, 1));
        assert_eq!(shift_month(2026, 10, 0), (2026, 10));
    }
}
