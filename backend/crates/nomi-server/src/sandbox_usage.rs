//! Sandbox metering: how many minutes a person's projects have run this month, the monthly
//! allowance, and a daily check of our totals against the provider's. Minutes belong to the
//! project's owner. Months are calendar months in UTC; a session that spans a month's edge counts
//! only the part inside the month. Zero-length sessions (they only advance the idempotency
//! generation) add nothing.

use std::time::Duration;

use nomi_sandbox::{SandboxId, SandboxProvider};
use sqlx::{PgConnection, PgPool};
use uuid::Uuid;

use crate::configuration::SandboxConfig;

/// Seconds in a minute; metered time is kept in seconds and floored to whole minutes.
const SECS_PER_MINUTE: i64 = 60;
/// How far back a sandbox must have been active to be checked against the provider.
const DRIFT_WINDOW_HOURS: i32 = 24;
/// How far our total and the provider's may differ before it is worth a log line.
pub const DRIFT_TOLERANCE: Duration = Duration::from_secs(5 * 60);
/// How often the drift check runs.
pub const DRIFT_INTERVAL: Duration = Duration::from_secs(24 * 60 * 60);

/// Seconds of a session inside the current UTC month (open sessions run to now).
const SECONDS_IN_MONTH: &str = "GREATEST(0, EXTRACT(EPOCH FROM ( \
    LEAST(COALESCE(stopped_at, now()), (date_trunc('month', now() AT TIME ZONE 'UTC') + interval '1 month') AT TIME ZONE 'UTC') \
    - GREATEST(started_at, date_trunc('month', now() AT TIME ZONE 'UTC') AT TIME ZONE 'UTC'))))";

/// Keeps sessions that ended before this month out of the sum, so the (user, started_at) index
/// can narrow the rows read.
const MONTH_NOT_OVER: &str = "COALESCE(stopped_at, now()) >= date_trunc('month', now() AT TIME ZONE 'UTC') AT TIME ZONE 'UTC'";

/// Whole minutes this person's projects have run in the sandbox this month. Open sessions count
/// up to now.
pub async fn minutes_this_month(conn: &mut PgConnection, user_id: Uuid) -> Result<i64, sqlx::Error> {
    let seconds: f64 = sqlx::query_scalar(&format!("SELECT COALESCE(SUM({SECONDS_IN_MONTH}), 0)::float8 FROM sandbox_sessions WHERE user_id = $1 AND {MONTH_NOT_OVER}"))
        .bind(user_id)
        .fetch_one(conn)
        .await?;
    Ok(seconds as i64 / SECS_PER_MINUTE)
}

/// Whole minutes every project has run in the sandbox this month, for the admin dashboard.
pub async fn minutes_this_month_all(conn: &mut PgConnection) -> Result<i64, sqlx::Error> {
    let seconds: f64 = sqlx::query_scalar(&format!("SELECT COALESCE(SUM({SECONDS_IN_MONTH}), 0)::float8 FROM sandbox_sessions WHERE {MONTH_NOT_OVER}"))
        .fetch_one(conn)
        .await?;
    Ok(seconds as i64 / SECS_PER_MINUTE)
}

/// Whether the person still has minutes left this month.
pub async fn within_allowance(conn: &mut PgConnection, user_id: Uuid, cfg: &SandboxConfig) -> Result<bool, sqlx::Error> {
    Ok(minutes_this_month(conn, user_id).await? < i64::from(cfg.monthly_minutes_per_user))
}

/// Compare our run time with the provider's for the sandboxes that ran in the last day, and log
/// the ones that disagree. Nothing is billed from this. Our figure is the project's metered time;
/// a project that has had several sandboxes will show some drift. Returns the projects logged.
pub async fn reconcile_usage_drift(pool: &PgPool, provider: &dyn SandboxProvider) -> Vec<Uuid> {
    let rows: Result<Vec<(Uuid, String, f64)>, sqlx::Error> = sqlx::query_as(
        "SELECT s.project_id, s.sandbox_id, COALESCE(SUM(EXTRACT(EPOCH FROM (COALESCE(x.stopped_at, now()) - x.started_at))), 0)::float8 \
         FROM project_sandboxes s JOIN sandbox_sessions x ON x.project_id = s.project_id \
         WHERE s.sandbox_id IS NOT NULL \
         AND EXISTS (SELECT 1 FROM sandbox_sessions a WHERE a.project_id = s.project_id \
                     AND (a.stopped_at IS NULL OR a.stopped_at > now() - make_interval(hours => $1))) \
         GROUP BY s.project_id, s.sandbox_id",
    )
    .bind(DRIFT_WINDOW_HOURS)
    .fetch_all(pool)
    .await;
    let rows = match rows {
        Ok(rows) => rows,
        Err(e) => {
            tracing::warn!(error = %e, "sandbox usage drift: reading our totals failed");
            return Vec::new();
        }
    };
    let mut drifted = Vec::new();
    for (project_id, sandbox_id, ours) in rows {
        let theirs = match provider.usage(&SandboxId(sandbox_id)).await {
            Ok(Some(theirs)) => theirs,
            Ok(None) => continue,
            Err(e) => {
                tracing::debug!(%project_id, error = %e, "sandbox usage drift: provider usage unavailable");
                continue;
            }
        };
        let ours = Duration::from_secs_f64(ours.max(0.0));
        let gap = ours.abs_diff(theirs);
        if gap > DRIFT_TOLERANCE {
            tracing::warn!(%project_id, our_secs = ours.as_secs(), provider_secs = theirs.as_secs(), gap_secs = gap.as_secs(), "sandbox usage drift");
            drifted.push(project_id);
        }
    }
    drifted
}
