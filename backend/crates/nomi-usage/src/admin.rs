//! Admin → Usage: every person's tokens for one month (UTC), per model and per embedding model.
//!
//! Chat models are told apart by the admin model's id (so a renamed model keeps its history and
//! shows its current name) and by the model actually called (an admin can point a model at a
//! different provider). Calls with a person's own key are grouped by provider and model.

use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct AdminUsage {
    /// `YYYY-MM`, UTC.
    pub month: String,
    pub current_month: String,
    pub chat: ChatTotals,
    pub embedding: EmbeddingTotals,
    pub by_day: Vec<AdminDay>,
    pub models: Vec<AdminModelUsage>,
    pub embedding_models: Vec<EmbeddingModelUsage>,
}

#[derive(Debug, Clone, Serialize, PartialEq, sqlx::FromRow)]
pub struct ChatTotals {
    pub tokens: i64,
    pub input_tokens: i64,
    pub output_tokens: i64,
    /// Input served from providers' prompt caches (in `input_tokens` too, billed for less).
    pub cached_input_tokens: i64,
    /// Tokens sent with people's own API keys (in `tokens` too; never billed).
    pub own_key_tokens: i64,
    pub calls: i64,
    pub people: i64,
    pub spend_usd: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq, sqlx::FromRow)]
pub struct EmbeddingTotals {
    pub tokens: i64,
    pub calls: i64,
    /// Calls whose tokens were worked out from the text (the provider didn't report them).
    pub estimated_calls: i64,
}

#[derive(Debug, Clone, Serialize, PartialEq, sqlx::FromRow)]
pub struct AdminDay {
    /// `YYYY-MM-DD`
    pub date: String,
    pub chat_tokens: i64,
    pub embedding_tokens: i64,
}

#[derive(Debug, Clone, Serialize, PartialEq, sqlx::FromRow)]
pub struct AdminModelUsage {
    /// The admin model, for Nomi's own models; `None` for people's own keys (and the
    /// environment fallback).
    pub admin_model_id: Option<Uuid>,
    /// The model's current name; for a deleted or own-key model, the last name it was used under.
    pub label: String,
    /// The admin model has since been deleted.
    pub deleted: bool,
    pub provider: String,
    pub model_id: String,
    /// `nomi` or `own_key`.
    pub source: String,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cached_input_tokens: i64,
    pub tokens: i64,
    pub calls: i64,
    pub people: i64,
    pub spend_usd: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq, sqlx::FromRow)]
pub struct EmbeddingModelUsage {
    pub provider: String,
    pub model_id: String,
    pub tokens: i64,
    pub calls: i64,
    pub estimated_calls: i64,
    pub people: i64,
    /// The embedding model Nomi uses now.
    pub current: bool,
}

/// The month's bounds in UTC: `$1` is `YYYY-MM`.
const IN_MONTH: &str = "created_at >= (($1 || '-01')::timestamp AT TIME ZONE 'UTC') \
    AND created_at < ((($1 || '-01')::timestamp + interval '1 month') AT TIME ZONE 'UTC')";

/// `month` is `YYYY-MM`; anything else (or nothing) means this month. `current_embedding` is the
/// provider and model Nomi embeds with now, to mark it.
pub async fn report(pool: &PgPool, month: Option<&str>, current_embedding: Option<(&str, &str)>) -> Result<AdminUsage, sqlx::Error> {
    let current_month: String = sqlx::query_scalar("SELECT to_char(now() AT TIME ZONE 'UTC', 'YYYY-MM')").fetch_one(pool).await?;
    let month = month.filter(|m| crate::valid_month(m)).unwrap_or(&current_month).to_string();

    let chat: ChatTotals = sqlx::query_as(&format!(
        "SELECT COALESCE(SUM(input_tokens + output_tokens), 0)::bigint AS tokens, \
                COALESCE(SUM(input_tokens), 0)::bigint AS input_tokens, \
                COALESCE(SUM(output_tokens), 0)::bigint AS output_tokens, \
                COALESCE(SUM(cached_input_tokens), 0)::bigint AS cached_input_tokens, \
                COALESCE(SUM(input_tokens + output_tokens) FILTER (WHERE source = 'own_key'), 0)::bigint AS own_key_tokens, \
                count(*) AS calls, count(DISTINCT user_id) AS people, COALESCE(SUM(cost_usd), 0)::float8 AS spend_usd \
         FROM llm_usage WHERE {IN_MONTH}"
    ))
    .bind(&month)
    .fetch_one(pool)
    .await?;

    let embedding: EmbeddingTotals = sqlx::query_as(&format!(
        "SELECT COALESCE(SUM(tokens), 0)::bigint AS tokens, count(*) AS calls, count(*) FILTER (WHERE estimated) AS estimated_calls \
         FROM embedding_usage WHERE {IN_MONTH}"
    ))
    .bind(&month)
    .fetch_one(pool)
    .await?;

    let by_day: Vec<AdminDay> = sqlx::query_as(&format!(
        "SELECT to_char(day, 'YYYY-MM-DD') AS date, SUM(chat)::bigint AS chat_tokens, SUM(embedding)::bigint AS embedding_tokens FROM ( \
            SELECT (created_at AT TIME ZONE 'UTC')::date AS day, input_tokens + output_tokens AS chat, 0 AS embedding FROM llm_usage WHERE {IN_MONTH} \
            UNION ALL \
            SELECT (created_at AT TIME ZONE 'UTC')::date, 0, tokens FROM embedding_usage WHERE {IN_MONTH} \
         ) d GROUP BY day ORDER BY day"
    ))
    .bind(&month)
    .fetch_all(pool)
    .await?;

    let models: Vec<AdminModelUsage> = sqlx::query_as(&format!(
        "SELECT u.admin_model_id, \
                COALESCE(m.label, (array_agg(u.model_label ORDER BY u.created_at DESC))[1]) AS label, \
                (u.admin_model_id IS NOT NULL AND m.id IS NULL) AS deleted, \
                u.provider, u.model_id, u.source, \
                SUM(u.input_tokens)::bigint AS input_tokens, SUM(u.output_tokens)::bigint AS output_tokens, \
                SUM(u.cached_input_tokens)::bigint AS cached_input_tokens, \
                SUM(u.input_tokens + u.output_tokens)::bigint AS tokens, count(*) AS calls, \
                count(DISTINCT u.user_id) AS people, SUM(u.cost_usd)::float8 AS spend_usd \
         FROM (SELECT * FROM llm_usage WHERE {IN_MONTH}) u \
         LEFT JOIN admin_llm_models m ON m.id = u.admin_model_id \
         GROUP BY u.admin_model_id, m.id, m.label, u.provider, u.model_id, u.source \
         ORDER BY tokens DESC, label"
    ))
    .bind(&month)
    .fetch_all(pool)
    .await?;

    let (current_provider, current_model) = current_embedding.unwrap_or(("", ""));
    let embedding_models: Vec<EmbeddingModelUsage> = sqlx::query_as(&format!(
        "SELECT provider, model_id, SUM(tokens)::bigint AS tokens, count(*) AS calls, \
                count(*) FILTER (WHERE estimated) AS estimated_calls, count(DISTINCT user_id) AS people, \
                (provider = $2 AND model_id = $3) AS current \
         FROM embedding_usage WHERE {IN_MONTH} GROUP BY provider, model_id ORDER BY tokens DESC"
    ))
    .bind(&month)
    .bind(current_provider)
    .bind(current_model)
    .fetch_all(pool)
    .await?;

    Ok(AdminUsage { month, current_month, chat, embedding, by_day, models, embedding_models })
}
