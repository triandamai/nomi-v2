//! What happens in a project, newest first, for Members → Activity: people's prompts in the
//! project chat, Koda's builds and the files they changed, publishes and rollbacks, people
//! joining. Read from what Nomi already records (messages, delegations, publishes, members), so
//! the feed covers history from before it existed and can't drift from what really happened.

use axum::extract::{Path, Query, State};
use axum::Json;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use nomi_auth::extractor::AuthClaims;

use crate::app::AppState;
use crate::routes::deploy::{internal, project_role, ApiError};

const DEFAULT_LIMIT: i64 = 50;
const MAX_LIMIT: i64 = 100;
/// A prompt is shown up to this many characters.
const PROMPT_PREVIEW_CHARS: i32 = 140;
const CODING_AGENT: &str = "coding";

#[derive(Deserialize)]
pub struct ActivityQuery {
    pub before: Option<DateTime<Utc>>,
    pub limit: Option<i64>,
}

#[derive(Serialize)]
pub struct Actor {
    pub name: Option<String>,
    pub avatar_url: Option<String>,
    /// The crew member, when it was one ("Koda").
    pub agent: Option<String>,
}

#[derive(Serialize)]
pub struct ActivityItem {
    /// prompt, build_started, build_finished, build_failed, build_stopped, files_changed,
    /// publishing, published, publish_failed, rolled_back, joined.
    pub kind: String,
    pub at: DateTime<Utc>,
    pub actor: Option<Actor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phase: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub step: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub files: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rollback_of_version: Option<i64>,
}

#[derive(Serialize)]
pub struct Activity {
    pub items: Vec<ActivityItem>,
    pub has_more: bool,
}

#[derive(sqlx::FromRow)]
struct Row {
    kind: String,
    at: DateTime<Utc>,
    person: Option<String>,
    avatar_url: Option<String>,
    agent: Option<String>,
    text: Option<String>,
    version: Option<i64>,
    environment: Option<String>,
    status: Option<String>,
    phase: Option<String>,
    step: Option<String>,
    files: Option<i64>,
    rollback_of_version: Option<i64>,
}

/// Every source as one row shape, newest first. `$1` project, `$2` before (or null), `$3`
/// limit + 1, `$4` the coding agent's name, `$5` prompt preview length.
const ACTIVITY_SQL: &str = "
WITH p AS (SELECT id, session_id FROM projects WHERE id = $1),
people AS (
    SELECT u.id, COALESCE(NULLIF(up.display_name, ''), split_part(COALESCE(w.email, g.email, ''), '@', 1)) AS name, up.avatar_url
    FROM users u
    LEFT JOIN user_profiles up ON up.user_id = u.id
    LEFT JOIN web_credentials w ON w.user_id = u.id
    LEFT JOIN google_identities g ON g.user_id = u.id
),
builds AS (
    SELECT d.* FROM agent_delegations d, p WHERE d.session_id = p.session_id AND d.target_agent_type = 'coding'
),
events AS (
    SELECT 'prompt' AS kind, m.created_at AS at, pe.name AS person, pe.avatar_url, NULL::text AS agent,
           left(m.content, $5) AS text, NULL::bigint AS version, NULL::text AS environment, NULL::text AS status,
           NULL::text AS phase, NULL::text AS step, NULL::bigint AS files, NULL::bigint AS rollback_of_version
    FROM messages m JOIN p ON m.session_id = p.session_id
    JOIN channel_identities ci ON ci.id = m.sender_channel_identity_id
    LEFT JOIN people pe ON pe.id = ci.user_id
    UNION ALL
    SELECT 'build_started', b.claimed_at, NULL, NULL, $4, NULL, NULL, NULL, b.status, NULL, NULL, NULL, NULL
    FROM builds b WHERE b.claimed_at IS NOT NULL
    UNION ALL
    SELECT CASE b.status WHEN 'completed' THEN 'build_finished' WHEN 'failed' THEN 'build_failed' ELSE 'build_stopped' END,
           b.completed_at, NULL, NULL, $4, NULL, NULL, NULL, b.status, NULL, NULL, NULL, NULL
    FROM builds b WHERE b.completed_at IS NOT NULL AND b.status IN ('completed', 'failed', 'cancelled')
    UNION ALL
    SELECT 'files_changed', changed.last_at, NULL, NULL, $4, NULL, NULL, NULL, NULL, NULL, NULL, changed.files, NULL
    FROM builds b
    CROSS JOIN LATERAL (
        SELECT sum(CASE blk->>'kind' WHEN 'file_writes' THEN jsonb_array_length(COALESCE(blk->'files', '[]'::jsonb)) ELSE 1 END)::bigint AS files,
               max(m.created_at) AS last_at
        FROM messages m
        CROSS JOIN LATERAL jsonb_array_elements(CASE WHEN jsonb_typeof(m.content_blocks) = 'array' THEN m.content_blocks ELSE '[]'::jsonb END) AS blk
        WHERE m.session_id = b.session_id AND m.created_at BETWEEN b.claimed_at AND b.completed_at
          AND blk->>'kind' IN ('file_write', 'file_writes', 'file_delete')
    ) AS changed
    WHERE b.claimed_at IS NOT NULL AND b.completed_at IS NOT NULL AND changed.files > 0
    UNION ALL
    SELECT CASE WHEN pp.rollback_of IS NOT NULL THEN 'rolled_back'
                WHEN pp.status = 'running' THEN 'publishing'
                WHEN pp.status = 'failed' THEN 'publish_failed'
                ELSE 'published' END,
           COALESCE(pp.finished_at, pp.created_at), pe.name, pe.avatar_url, pp.started_by_agent, NULL,
           pp.files_version, pp.environment, pp.status, pp.failure->>'phase', pp.failure->>'step', NULL, rb.files_version
    FROM project_publishes pp JOIN p ON pp.project_id = p.id
    LEFT JOIN people pe ON pe.id = pp.created_by
    LEFT JOIN project_publishes rb ON rb.id = pp.rollback_of
    UNION ALL
    SELECT 'joined', pm.created_at, pe.name, pe.avatar_url, NULL, NULL, NULL, NULL, pm.role, NULL, NULL, NULL, NULL
    FROM project_members pm JOIN p ON pm.project_id = p.id
    LEFT JOIN people pe ON pe.id = pm.user_id
)
SELECT kind, at, person, avatar_url, agent, text, version, environment, status, phase, step, files, rollback_of_version
FROM events WHERE at IS NOT NULL AND ($2::timestamptz IS NULL OR at < $2)
ORDER BY at DESC
LIMIT $3";

/// `GET /api/projects/:id/activity?before=&limit=`
pub async fn list(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(project_id): Path<Uuid>,
    Query(query): Query<ActivityQuery>,
) -> Result<Json<Activity>, ApiError> {
    project_role(&state.pool, project_id, claims.sub).await?;
    let limit = query.limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT);
    let coding_agent = nomi_agent_core::crew::crew_name(CODING_AGENT).unwrap_or(CODING_AGENT);
    let mut rows: Vec<Row> = sqlx::query_as(ACTIVITY_SQL)
        .bind(project_id)
        .bind(query.before)
        .bind(limit + 1)
        .bind(coding_agent)
        .bind(PROMPT_PREVIEW_CHARS)
        .fetch_all(&state.pool)
        .await
        .map_err(internal("failed to load the project's activity"))?;
    let has_more = rows.len() as i64 > limit;
    rows.truncate(limit as usize);
    let items = rows
        .into_iter()
        .map(|row| {
            let actor = (row.person.is_some() || row.agent.is_some()).then(|| Actor { name: row.person, avatar_url: row.avatar_url, agent: row.agent });
            ActivityItem {
                kind: row.kind,
                at: row.at,
                actor,
                text: row.text,
                version: row.version,
                environment: row.environment,
                status: row.status,
                phase: row.phase,
                step: row.step,
                files: row.files,
                rollback_of_version: row.rollback_of_version,
            }
        })
        .collect();
    Ok(Json(Activity { items, has_more }))
}
