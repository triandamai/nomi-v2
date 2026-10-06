//! Token usage and spend, per person: every call to a language model is metered as it finishes
//! ([`MeteredProvider`]), and [`month`] / [`brief`] read it back for Billing & usage and the
//! navigation drawer.
//!
//! Spend uses the admin model's price at the time of the call. Calls made with the person's own
//! API key count as usage but never as spend, and don't use up the plan's allowance.

use std::sync::Arc;

use async_trait::async_trait;
use futures_util::StreamExt;
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

use nomi_llm::{LlmError, LlmEventStream, LlmProvider, LlmRequest, StreamEvent};

/// Whose model a provider calls, as recorded on each usage row.
#[derive(Debug, Clone, PartialEq)]
pub struct ModelTag {
    /// The admin model, when Nomi's own model answers. `None` for the person's own key, or the
    /// environment fallback.
    pub admin_model_id: Option<Uuid>,
    /// The person's own API key: counted, never billed.
    pub own_key: bool,
    pub label: String,
    pub provider: String,
    pub model_id: String,
}

/// A provider that records each finished call's tokens for `user_id`.
pub struct MeteredProvider {
    inner: Arc<dyn LlmProvider>,
    pool: PgPool,
    user_id: Uuid,
    tag: Arc<ModelTag>,
}

impl MeteredProvider {
    pub fn new(inner: Arc<dyn LlmProvider>, pool: PgPool, user_id: Uuid, tag: ModelTag) -> Self {
        Self { inner, pool, user_id, tag: Arc::new(tag) }
    }
}

#[async_trait]
impl LlmProvider for MeteredProvider {
    async fn complete_stream(&self, request: LlmRequest) -> Result<LlmEventStream, LlmError> {
        let mut events = self.inner.complete_stream(request).await?;
        let (pool, user_id, tag) = (self.pool.clone(), self.user_id, self.tag.clone());
        Ok(Box::pin(async_stream::stream! {
            while let Some(event) = events.next().await {
                if let Ok(StreamEvent::Done { input_tokens, output_tokens, .. }) = &event {
                    // Recorded before the caller sees the end of the stream, so a finished turn's
                    // usage is always there to read. A failed write never fails the reply.
                    if let Err(e) = record(&pool, user_id, &tag, *input_tokens, *output_tokens).await {
                        tracing::warn!(error = %e, %user_id, "failed to record llm usage");
                    }
                }
                yield event;
            }
        }))
    }
}

/// Records one call. Spend comes from the admin model's current price.
pub async fn record(pool: &PgPool, user_id: Uuid, tag: &ModelTag, input_tokens: u32, output_tokens: u32) -> Result<(), sqlx::Error> {
    if input_tokens == 0 && output_tokens == 0 {
        return Ok(());
    }
    sqlx::query(
        "INSERT INTO llm_usage (user_id, admin_model_id, source, model_label, provider, model_id, input_tokens, output_tokens, cost_usd) \
         SELECT $1, $2, $3, $4, $5, $6, $7, $8, \
                CASE WHEN $3 = 'nomi' THEN \
                    ($7::numeric * COALESCE(m.input_usd_per_mtok, 0) + $8::numeric * COALESCE(m.output_usd_per_mtok, 0)) / 1000000 \
                ELSE 0 END \
         FROM (SELECT 1) one LEFT JOIN admin_llm_models m ON m.id = $2",
    )
    .bind(user_id)
    .bind(tag.admin_model_id)
    .bind(if tag.own_key { "own_key" } else { "nomi" })
    .bind(&tag.label)
    .bind(&tag.provider)
    .bind(&tag.model_id)
    .bind(input_tokens as i32)
    .bind(output_tokens as i32)
    .execute(pool)
    .await?;
    Ok(())
}

/// The plan someone is on. Everyone is on Free until Pro launches.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Plan {
    pub id: &'static str,
    /// Tokens of Nomi's own models included each month.
    pub monthly_tokens: i64,
}

/// The Free plan's monthly allowance: `FREE_MONTHLY_TOKENS`, else one million.
pub fn plan_for(_user_id: Uuid) -> Plan {
    let monthly_tokens = std::env::var("FREE_MONTHLY_TOKENS").ok().and_then(|v| v.trim().parse().ok()).filter(|n: &i64| *n > 0).unwrap_or(1_000_000);
    Plan { id: "free", monthly_tokens }
}

/// For the navigation drawer: this month's allowance and how much of it is used.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Brief {
    pub plan: Plan,
    pub month: String,
    /// Tokens of Nomi's own models this month (own-key usage doesn't count).
    pub tokens_used: i64,
}

/// One month of usage, for Billing & usage.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct MonthUsage {
    pub plan: Plan,
    /// `YYYY-MM`, in the person's timezone.
    pub month: String,
    pub current_month: String,
    pub timezone: String,
    /// Tokens of Nomi's own models: what the plan's allowance counts.
    pub tokens_used: i64,
    pub input_tokens: i64,
    pub output_tokens: i64,
    /// Tokens sent with the person's own API key.
    pub own_key_tokens: i64,
    pub calls: i64,
    pub spend_usd: f64,
    pub by_day: Vec<DayUsage>,
    pub by_model: Vec<ModelUsage>,
}

#[derive(Debug, Clone, Serialize, PartialEq, sqlx::FromRow)]
pub struct DayUsage {
    /// `YYYY-MM-DD`
    pub date: String,
    pub tokens: i64,
    pub spend_usd: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq, sqlx::FromRow)]
pub struct ModelUsage {
    pub label: String,
    pub provider: String,
    pub model_id: String,
    pub source: String,
    pub tokens: i64,
    pub calls: i64,
    pub spend_usd: f64,
}

async fn timezone_and_current_month(pool: &PgPool, user_id: Uuid) -> Result<(String, String), sqlx::Error> {
    sqlx::query_as(
        "SELECT tz, to_char(now() AT TIME ZONE tz, 'YYYY-MM') FROM \
         (SELECT COALESCE((SELECT timezone FROM user_preferences WHERE user_id = $1 \
                           AND timezone IN (SELECT name FROM pg_timezone_names)), 'UTC') AS tz) t",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await
}

/// The month's bounds in the person's timezone, as a SQL fragment's parameters: `$2` is the
/// timezone and `$3` the `YYYY-MM`.
const IN_MONTH: &str = "user_id = $1 \
    AND created_at >= (($3 || '-01')::timestamp AT TIME ZONE $2) \
    AND created_at < ((($3 || '-01')::timestamp + interval '1 month') AT TIME ZONE $2)";

fn valid_month(month: &str) -> bool {
    month.len() == 7 && chrono::NaiveDate::parse_from_str(&format!("{month}-01"), "%Y-%m-%d").is_ok()
}

pub async fn brief(pool: &PgPool, user_id: Uuid) -> Result<Brief, sqlx::Error> {
    let (tz, month) = timezone_and_current_month(pool, user_id).await?;
    let tokens_used: i64 = sqlx::query_scalar(&format!(
        "SELECT COALESCE(SUM(input_tokens + output_tokens), 0)::bigint FROM llm_usage WHERE {IN_MONTH} AND source = 'nomi'"
    ))
    .bind(user_id)
    .bind(&tz)
    .bind(&month)
    .fetch_one(pool)
    .await?;
    Ok(Brief { plan: plan_for(user_id), month, tokens_used })
}

/// `month` is `YYYY-MM`; anything else (or nothing) means this month.
pub async fn month(pool: &PgPool, user_id: Uuid, month: Option<&str>) -> Result<MonthUsage, sqlx::Error> {
    let (tz, current_month) = timezone_and_current_month(pool, user_id).await?;
    let month = month.filter(|m| valid_month(m)).unwrap_or(&current_month).to_string();

    let (tokens_used, input_tokens, output_tokens, own_key_tokens, calls, spend_usd): (i64, i64, i64, i64, i64, f64) = sqlx::query_as(&format!(
        "SELECT \
            COALESCE(SUM(input_tokens + output_tokens) FILTER (WHERE source = 'nomi'), 0)::bigint, \
            COALESCE(SUM(input_tokens), 0)::bigint, \
            COALESCE(SUM(output_tokens), 0)::bigint, \
            COALESCE(SUM(input_tokens + output_tokens) FILTER (WHERE source = 'own_key'), 0)::bigint, \
            count(*), \
            COALESCE(SUM(cost_usd), 0)::float8 \
         FROM llm_usage WHERE {IN_MONTH}"
    ))
    .bind(user_id)
    .bind(&tz)
    .bind(&month)
    .fetch_one(pool)
    .await?;

    let by_day: Vec<DayUsage> = sqlx::query_as(&format!(
        "SELECT to_char((created_at AT TIME ZONE $2)::date, 'YYYY-MM-DD') AS date, \
                SUM(input_tokens + output_tokens)::bigint AS tokens, SUM(cost_usd)::float8 AS spend_usd \
         FROM llm_usage WHERE {IN_MONTH} GROUP BY 1 ORDER BY 1"
    ))
    .bind(user_id)
    .bind(&tz)
    .bind(&month)
    .fetch_all(pool)
    .await?;

    let by_model: Vec<ModelUsage> = sqlx::query_as(&format!(
        "SELECT model_label AS label, provider, model_id, source, \
                SUM(input_tokens + output_tokens)::bigint AS tokens, count(*) AS calls, SUM(cost_usd)::float8 AS spend_usd \
         FROM llm_usage WHERE {IN_MONTH} GROUP BY 1, 2, 3, 4 ORDER BY tokens DESC"
    ))
    .bind(user_id)
    .bind(&tz)
    .bind(&month)
    .fetch_all(pool)
    .await?;

    Ok(MonthUsage {
        plan: plan_for(user_id),
        month,
        current_month,
        timezone: tz,
        tokens_used,
        input_tokens,
        output_tokens,
        own_key_tokens,
        calls,
        spend_usd,
        by_day,
        by_model,
    })
}
