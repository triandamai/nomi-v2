//! A project's token usage and spend: every call made in its own chat, in a chat filed into it
//! (`sessions.in_project_id`) or in one of its issues' chats (`project_issues.session_id`), whoever
//! made it. Moving a chat into a project brings its calls
//! with it. Calls from before calls carried their chat count toward no project; `counted_since`
//! says when counting began.

use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{counted_tokens_sql, timezone_and_current_month, valid_month, ModelUsage};

/// One person's part of a project's month.
#[derive(Debug, Clone, Serialize, PartialEq, sqlx::FromRow)]
pub struct PersonUsage {
    pub user_id: Uuid,
    /// Their display name, else their email.
    pub name: Option<String>,
    pub tokens: i64,
    pub calls: i64,
    pub spend_usd: f64,
}

/// One month of a project's usage, in the viewer's timezone.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ProjectUsage {
    /// `YYYY-MM`
    pub month: String,
    pub current_month: String,
    pub timezone: String,
    /// When calls began to carry their chat: nothing before it counts toward a project. `None`
    /// until the first such call.
    pub counted_since: Option<DateTime<Utc>>,
    /// Every token, cached input at [`crate::CACHED_INPUT_WEIGHT`], whoever's key paid.
    pub tokens: i64,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cached_input_tokens: i64,
    /// The part of `tokens` sent with someone's own API key: counted, never billed.
    pub own_key_tokens: i64,
    pub calls: i64,
    pub spend_usd: f64,
    pub by_model: Vec<ModelUsage>,
    pub by_person: Vec<PersonUsage>,
}

/// The project's calls in the month: `$1` the project, `$2` the timezone, `$3` the `YYYY-MM`.
const PROJECT_MONTH: &str = "SELECT * FROM llm_usage \
    WHERE session_id IN (SELECT session_id FROM projects WHERE id = $1 \
                         UNION SELECT id FROM sessions WHERE in_project_id = $1 \
                         UNION SELECT session_id FROM project_issues WHERE project_id = $1 AND session_id IS NOT NULL) \
      AND created_at >= (($3 || '-01')::timestamp AT TIME ZONE $2) \
      AND created_at < ((($3 || '-01')::timestamp + interval '1 month') AT TIME ZONE $2)";

/// `project_id`'s usage in `month` (`YYYY-MM`; anything else, or nothing, means this month), with
/// months in `viewer_id`'s timezone. The caller checks the viewer may open the project, and asks for
/// `by_person` (`with_people`) only for someone who may see who spent what.
pub async fn project_month(pool: &PgPool, project_id: Uuid, viewer_id: Uuid, month: Option<&str>, with_people: bool) -> Result<ProjectUsage, sqlx::Error> {
    let (tz, current_month) = timezone_and_current_month(pool, viewer_id).await?;
    let month = month.filter(|m| valid_month(m)).unwrap_or(&current_month).to_string();
    let counted = counted_tokens_sql("u.");

    #[allow(clippy::type_complexity)]
    let (tokens, input_tokens, output_tokens, cached_input_tokens, own_key_tokens, calls, spend_usd): (i64, i64, i64, i64, i64, i64, f64) = sqlx::query_as(&format!(
        "SELECT COALESCE(ROUND(SUM({counted})), 0)::bigint, \
                COALESCE(SUM(u.input_tokens), 0)::bigint, \
                COALESCE(SUM(u.output_tokens), 0)::bigint, \
                COALESCE(SUM(u.cached_input_tokens), 0)::bigint, \
                COALESCE(ROUND(SUM({counted}) FILTER (WHERE u.source = 'own_key')), 0)::bigint, \
                count(*), \
                COALESCE(SUM(u.cost_usd), 0)::float8 \
         FROM ({PROJECT_MONTH}) u"
    ))
    .bind(project_id)
    .bind(&tz)
    .bind(&month)
    .fetch_one(pool)
    .await?;

    let by_model: Vec<ModelUsage> = sqlx::query_as(&format!(
        "SELECT u.admin_model_id, COALESCE(m.label, (array_agg(u.model_label ORDER BY u.created_at DESC))[1]) AS label, \
                u.provider, u.model_id, u.source, \
                ROUND(SUM({counted}))::bigint AS tokens, count(*) AS calls, SUM(u.cost_usd)::float8 AS spend_usd \
         FROM ({PROJECT_MONTH}) u LEFT JOIN admin_llm_models m ON m.id = u.admin_model_id \
         GROUP BY u.admin_model_id, m.id, m.label, u.provider, u.model_id, u.source ORDER BY tokens DESC"
    ))
    .bind(project_id)
    .bind(&tz)
    .bind(&month)
    .fetch_all(pool)
    .await?;

    let by_person: Vec<PersonUsage> = if !with_people {
        Vec::new()
    } else {
        sqlx::query_as(&format!(
        "SELECT u.user_id, COALESCE(NULLIF(p.display_name, ''), w.email, g.email) AS name, \
                ROUND(SUM({counted}))::bigint AS tokens, count(*) AS calls, SUM(u.cost_usd)::float8 AS spend_usd \
         FROM ({PROJECT_MONTH}) u \
         LEFT JOIN user_profiles p ON p.user_id = u.user_id \
         LEFT JOIN web_credentials w ON w.user_id = u.user_id \
         LEFT JOIN google_identities g ON g.user_id = u.user_id \
         GROUP BY u.user_id, p.display_name, w.email, g.email ORDER BY tokens DESC"
    ))
    .bind(project_id)
    .bind(&tz)
    .bind(&month)
    .fetch_all(pool)
    .await?
    };

    let counted_since: Option<DateTime<Utc>> = sqlx::query_scalar("SELECT min(created_at) FROM llm_usage WHERE session_id IS NOT NULL").fetch_one(pool).await?;

    Ok(ProjectUsage {
        month,
        current_month,
        timezone: tz,
        counted_since,
        tokens,
        input_tokens,
        output_tokens,
        cached_input_tokens,
        own_key_tokens,
        calls,
        spend_usd,
        by_model,
        by_person,
    })
}
