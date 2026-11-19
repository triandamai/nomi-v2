//! `GET /api/me/plan`: the plan someone is on and what's left of its monthly build allowance
//! ("1 build · 26 of 30 left in Oct" in the Deploy dialog). A build is a publish; the plan's
//! `max_builds_month` is how many a calendar month holds (empty: unlimited, like the plan's other
//! limits). Cancelled publishes and rollbacks (they build nothing new) aren't counted.

use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use chrono::{DateTime, Datelike, TimeZone, Utc};
use serde::Serialize;
use uuid::Uuid;

use nomi_auth::extractor::AuthClaims;
use nomi_usage::plans;

use crate::app::AppState;

const MONTHS_IN_YEAR: u32 = 12;

/// What a month's builds come to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct BuildAllowance {
    /// Null when the plan has no limit.
    pub limit: Option<i64>,
    pub used: i64,
    /// Null when the plan has no limit; never below zero.
    pub left: Option<i64>,
}

pub fn build_allowance(limit: Option<i32>, used: i64) -> BuildAllowance {
    let limit = limit.map(i64::from);
    BuildAllowance { limit, used, left: limit.map(|limit| (limit - used).max(0)) }
}

/// The calendar month `now` is in (UTC): when it started and when the next one does.
pub fn month_bounds(now: DateTime<Utc>) -> (DateTime<Utc>, DateTime<Utc>) {
    let first = |year: i32, month: u32| Utc.with_ymd_and_hms(year, month, 1, 0, 0, 0).single().expect("the first of a month exists");
    let start = first(now.year(), now.month());
    let next = if now.month() == MONTHS_IN_YEAR { first(now.year() + 1, 1) } else { first(now.year(), now.month() + 1) };
    (start, next)
}

#[derive(Serialize)]
pub struct PlanSummary {
    pub id: Uuid,
    pub slug: String,
    pub name: String,
}

#[derive(Serialize)]
pub struct BuildsThisMonth {
    #[serde(flatten)]
    pub allowance: BuildAllowance,
    /// When the count starts again.
    pub resets_at: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct MyPlan {
    pub plan: PlanSummary,
    pub monthly_tokens: i64,
    pub builds: BuildsThisMonth,
}

fn internal(e: sqlx::Error) -> (StatusCode, &'static str) {
    tracing::error!(error = %e, "failed to load the plan");
    (StatusCode::INTERNAL_SERVER_ERROR, "failed to load the plan")
}

pub async fn get(State(state): State<AppState>, AuthClaims(claims): AuthClaims) -> Result<Json<MyPlan>, (StatusCode, &'static str)> {
    let subscription = plans::subscription_for(&state.pool, claims.sub).await.map_err(internal)?;
    let limit: Option<i32> = sqlx::query_scalar("SELECT max_builds_month FROM plans WHERE id = $1")
        .bind(subscription.plan.id)
        .fetch_one(&state.pool)
        .await
        .map_err(internal)?;
    let (from, resets_at) = month_bounds(Utc::now());
    let used: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM project_publishes WHERE created_by = $1 AND rollback_of IS NULL AND status <> 'cancelled' AND created_at >= $2 AND created_at < $3",
    )
    .bind(claims.sub)
    .bind(from)
    .bind(resets_at)
    .fetch_one(&state.pool)
    .await
    .map_err(internal)?;
    Ok(Json(MyPlan {
        plan: PlanSummary { id: subscription.plan.id, slug: subscription.plan.slug.clone(), name: subscription.plan.name.clone() },
        monthly_tokens: subscription.monthly_tokens,
        builds: BuildsThisMonth { allowance: build_allowance(limit, used), resets_at },
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_limit_counts_down_and_stops_at_zero() {
        assert_eq!(build_allowance(Some(30), 4), BuildAllowance { limit: Some(30), used: 4, left: Some(26) });
        assert_eq!(build_allowance(Some(30), 30).left, Some(0));
        // Past the limit (an admin lowered it): still zero, never negative.
        assert_eq!(build_allowance(Some(3), 5), BuildAllowance { limit: Some(3), used: 5, left: Some(0) });
    }

    #[test]
    fn no_limit_means_unlimited() {
        assert_eq!(build_allowance(None, 12), BuildAllowance { limit: None, used: 12, left: None });
    }

    #[test]
    fn the_month_runs_from_the_first_to_the_first() {
        let mid = Utc.with_ymd_and_hms(2026, 10, 17, 9, 30, 0).unwrap();
        assert_eq!(month_bounds(mid), (Utc.with_ymd_and_hms(2026, 10, 1, 0, 0, 0).unwrap(), Utc.with_ymd_and_hms(2026, 11, 1, 0, 0, 0).unwrap()));
        let december = Utc.with_ymd_and_hms(2026, 12, 31, 23, 59, 59).unwrap();
        assert_eq!(month_bounds(december), (Utc.with_ymd_and_hms(2026, 12, 1, 0, 0, 0).unwrap(), Utc.with_ymd_and_hms(2027, 1, 1, 0, 0, 0).unwrap()));
    }
}
