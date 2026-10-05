//! Home's three status cards: "While you were out", "Today" and "Plans in progress".

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::Json;
use chrono::{DateTime, Duration, NaiveTime, SubsecRound, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::app::AppState;
use nomi_auth::extractor::AuthClaims;

/// Each card on Home shows this many; "Show more" opens the section's own page.
pub const PREVIEW_LIMIT: usize = 4;
/// The most a section lists in full, across all its pages.
const LIST_CAP: i64 = 200;
const DEFAULT_PER_PAGE: usize = 20;
const MAX_PER_PAGE: usize = 50;
/// Without a previous visit, "while you were out" looks back this far.
const DEFAULT_LOOKBACK_HOURS: i64 = 24;
/// A gap longer than this starts a new visit (see migration 0033).
const NEW_VISIT_AFTER_MINUTES: i64 = 30;

#[derive(Serialize)]
pub struct OutItem {
    /// "finished", "failed", "stopped", "needs_you", "reminder" or "reply".
    pub kind: String,
    /// Agent type or display name, for the shape and tint.
    pub agent: String,
    pub title: String,
    pub detail: String,
    pub session_id: Option<Uuid>,
    pub at: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct TodayItem {
    pub id: Uuid,
    pub run_at: DateTime<Utc>,
    pub label: String,
    pub agent: String,
    pub recurrence: Option<String>,
}

#[derive(Serialize)]
pub struct PlanItem {
    /// "todo" (a to-do list in a chat) or "plan" (a written plan).
    pub kind: String,
    pub title: String,
    pub agent: String,
    pub done: i64,
    pub total: i64,
    pub session_id: Uuid,
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct HomeSummary {
    pub since: DateTime<Utc>,
    pub timezone: String,
    pub while_you_were_out: Vec<OutItem>,
    pub today: Vec<TodayItem>,
    pub plans: Vec<PlanItem>,
    /// How many each section holds in full; the lists above carry at most PREVIEW_LIMIT.
    pub while_you_were_out_total: usize,
    pub today_total: usize,
    pub plans_total: usize,
}

/// One page of a Home section (GET /api/home/{updates,today,plans}).
#[derive(Serialize)]
pub struct SectionPage<T> {
    pub since: DateTime<Utc>,
    pub timezone: String,
    pub items: Vec<T>,
    pub total: usize,
    pub page: usize,
    pub per_page: usize,
}

#[derive(Deserialize)]
pub struct PageQuery {
    pub page: Option<usize>,
    pub per_page: Option<usize>,
}

/// `(page, per_page)` clamped to sane values; pages count from 1.
pub fn page_params(query: &PageQuery) -> (usize, usize) {
    let per_page = query
        .per_page
        .unwrap_or(DEFAULT_PER_PAGE)
        .clamp(1, MAX_PER_PAGE);
    (query.page.unwrap_or(1).max(1), per_page)
}

/// The items on `page` (1-based) of `per_page`; past the end is empty.
pub fn page_of<T>(items: Vec<T>, page: usize, per_page: usize) -> Vec<T> {
    items
        .into_iter()
        .skip((page - 1).saturating_mul(per_page))
        .take(per_page)
        .collect()
}

fn clip(text: &str, max: usize) -> String {
    let flat = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if flat.chars().count() <= max {
        return flat;
    }
    let mut clipped: String = flat.chars().take(max - 1).collect();
    clipped.push('…');
    clipped
}

/// Markdown emphasis and code marks stripped, for a one-line preview.
fn plain(text: &str) -> String {
    text.replace(['*', '`', '#', '_'], "")
}

fn capitalize(agent_type: &str) -> String {
    if agent_type == "chitchat" {
        return "Nomi".to_string();
    }
    let mut chars = agent_type.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

/// `(done, total)` task-list items in a plan's markdown (`- [x]` / `- [ ]`).
pub fn checklist_progress(markdown: &str) -> (i64, i64) {
    let mut done = 0;
    let mut total = 0;
    for line in markdown.lines() {
        let item = line
            .trim_start()
            .trim_start_matches(['-', '*', '+'])
            .trim_start();
        if line.trim_start().starts_with(['-', '*', '+']) {
            if item.starts_with("[ ]") {
                total += 1;
            } else if item.starts_with("[x]") || item.starts_with("[X]") {
                total += 1;
                done += 1;
            }
        }
    }
    (done, total)
}

type DelegationRow = (
    String,
    String,
    String,
    Option<String>,
    Option<String>,
    Uuid,
    DateTime<Utc>,
);
type ReplyRow = (Uuid, Option<String>, Option<String>, String, DateTime<Utc>);
type TodayRow = (Uuid, DateTime<Utc>, String, String, Option<String>);
type TodoRow = (
    Uuid,
    Option<String>,
    Option<String>,
    serde_json::Value,
    DateTime<Utc>,
);

fn internal(e: sqlx::Error) -> (StatusCode, &'static str) {
    tracing::error!(error = %e, "failed to load the home summary");
    (StatusCode::INTERNAL_SERVER_ERROR, "failed to load home")
}

/// Records this visit and returns the moment "while you were out" counts from.
async fn visit_since(pool: &sqlx::PgPool, user_id: Uuid) -> Result<DateTime<Utc>, sqlx::Error> {
    // Postgres keeps microseconds; match it so a stored time reads back identical.
    let now = Utc::now().trunc_subsecs(6);
    let row: Option<(DateTime<Utc>, DateTime<Utc>)> =
        sqlx::query_as("SELECT seen_at, previous_seen_at FROM user_home_visits WHERE user_id = $1")
            .bind(user_id)
            .fetch_optional(pool)
            .await?;
    let since = match row {
        None => {
            let since = now - Duration::hours(DEFAULT_LOOKBACK_HOURS);
            sqlx::query("INSERT INTO user_home_visits (user_id, seen_at, previous_seen_at) VALUES ($1, $2, $3) ON CONFLICT (user_id) DO NOTHING")
                .bind(user_id)
                .bind(now)
                .bind(since)
                .execute(pool)
                .await?;
            since
        }
        Some((seen_at, _)) if now - seen_at > Duration::minutes(NEW_VISIT_AFTER_MINUTES) => {
            sqlx::query("UPDATE user_home_visits SET previous_seen_at = seen_at, seen_at = $2 WHERE user_id = $1")
                .bind(user_id)
                .bind(now)
                .execute(pool)
                .await?;
            seen_at
        }
        Some((_, previous)) => {
            sqlx::query("UPDATE user_home_visits SET seen_at = $2 WHERE user_id = $1")
                .bind(user_id)
                .bind(now)
                .execute(pool)
                .await?;
            previous
        }
    };
    Ok(since.max(now - Duration::days(7)))
}

/// Like `visit_since`, without recording a visit: a section's own page reads the same window
/// Home showed.
async fn peek_since(pool: &sqlx::PgPool, user_id: Uuid) -> Result<DateTime<Utc>, sqlx::Error> {
    let now = Utc::now().trunc_subsecs(6);
    let row: Option<(DateTime<Utc>, DateTime<Utc>)> =
        sqlx::query_as("SELECT seen_at, previous_seen_at FROM user_home_visits WHERE user_id = $1")
            .bind(user_id)
            .fetch_optional(pool)
            .await?;
    let since = match row {
        None => now - Duration::hours(DEFAULT_LOOKBACK_HOURS),
        Some((seen_at, _)) if now - seen_at > Duration::minutes(NEW_VISIT_AFTER_MINUTES) => seen_at,
        Some((_, previous)) => previous,
    };
    Ok(since.max(now - Duration::days(7)))
}

async fn user_timezone(
    pool: &sqlx::PgPool,
    user_id: Uuid,
) -> Result<(String, chrono_tz::Tz), sqlx::Error> {
    let timezone: String =
        sqlx::query_scalar("SELECT timezone FROM user_preferences WHERE user_id = $1")
            .bind(user_id)
            .fetch_optional(pool)
            .await?
            .unwrap_or_else(|| "UTC".to_string());
    let tz: chrono_tz::Tz = timezone.parse().unwrap_or(chrono_tz::UTC);
    Ok((timezone, tz))
}

/// Everything that moved since `since`: waiting on the user first, then newest.
async fn while_you_were_out(
    pool: &sqlx::PgPool,
    user_id: Uuid,
    org_id: Uuid,
    since: DateTime<Utc>,
) -> Result<Vec<OutItem>, sqlx::Error> {
    let mut out: Vec<OutItem> = Vec::new();

    // Approvals waiting on the user, wherever they were raised.
    let waiting: Vec<(String, Uuid, String, DateTime<Utc>)> = sqlx::query_as(
        "SELECT a.agent_type, a.session_id, COALESCE(m.content_blocks->0->>'description', 'An action'), m.created_at \
         FROM agent_sessions a JOIN channel_identities ci ON ci.id = a.sender_channel_identity_id \
         JOIN messages m ON m.id = (a.state->>'pending_approval_message_id')::uuid \
         WHERE ci.user_id = $1 AND a.status IN ('active', 'awaiting_approval') AND (a.state->>'paused_for_approval')::boolean = true \
         ORDER BY m.created_at DESC LIMIT $2",
    )
    .bind(user_id)
    .bind(LIST_CAP)
    .fetch_all(pool)
    .await
    ?;
    for (agent_type, session_id, description, at) in waiting {
        out.push(OutItem {
            kind: "needs_you".into(),
            title: format!("{} needs your OK", capitalize(&agent_type)),
            detail: clip(&description, 90),
            agent: agent_type,
            session_id: Some(session_id),
            at,
        });
    }

    let delegations: Vec<DelegationRow> = sqlx::query_as(
        "SELECT target_agent_type, status, task, result, error, session_id, completed_at FROM agent_delegations \
         WHERE user_id = $1 AND completed_at > $2 AND status IN ('completed', 'failed', 'cancelled') \
         ORDER BY completed_at DESC LIMIT $3",
    )
    .bind(user_id)
    .bind(since)
    .bind(LIST_CAP)
    .fetch_all(pool)
    .await
    ?;
    for (agent_type, status, task, result, error, session_id, at) in delegations {
        let name = capitalize(&agent_type);
        let (kind, title, detail) = match status.as_str() {
            "completed" => (
                "finished",
                format!("{name} finished {}", clip(&task, 60)),
                result.unwrap_or_default(),
            ),
            "failed" => (
                "failed",
                format!("{name} couldn't finish {}", clip(&task, 52)),
                error.unwrap_or_default(),
            ),
            _ => (
                "stopped",
                format!("{name} stopped {}", clip(&task, 60)),
                "You stopped this one".to_string(),
            ),
        };
        out.push(OutItem {
            kind: kind.into(),
            agent: agent_type,
            title,
            detail: clip(&plain(&detail), 90),
            session_id: Some(session_id),
            at,
        });
    }

    let rang: Vec<(String, Uuid, DateTime<Utc>)> = sqlx::query_as(
        "SELECT title, session_id, last_fired_at FROM reminders \
         WHERE user_id = $1 AND last_fired_at > $2 ORDER BY last_fired_at DESC LIMIT $3",
    )
    .bind(user_id)
    .bind(since)
    .bind(LIST_CAP)
    .fetch_all(pool)
    .await?;
    for (title, session_id, at) in rang {
        out.push(OutItem {
            kind: "reminder".into(),
            title: format!("Reminder: {}", clip(&title, 60)),
            detail: "Went off in your Reminders chat".to_string(),
            agent: "reminders".into(),
            session_id: Some(session_id),
            at,
        });
    }

    let fired: Vec<(String, String, Uuid, DateTime<Utc>)> = sqlx::query_as(
        "SELECT target_agent_type, label, session_id, last_fired_at FROM scheduled_jobs \
         WHERE user_id = $1 AND last_fired_at > $2 ORDER BY last_fired_at DESC LIMIT $3",
    )
    .bind(user_id)
    .bind(since)
    .bind(LIST_CAP)
    .fetch_all(pool)
    .await?;
    for (agent_type, label, session_id, at) in fired {
        out.push(OutItem {
            kind: "reminder".into(),
            title: format!("Reminder: {}", clip(&label, 60)),
            detail: format!("{} followed up in your chat", capitalize(&agent_type)),
            agent: agent_type,
            session_id: Some(session_id),
            at,
        });
    }

    // Chats where the crew said something the user hasn't answered yet.
    let replies: Vec<ReplyRow> = sqlx::query_as(
        "SELECT DISTINCT ON (m.session_id) m.session_id, s.title, m.agent_display_name, m.content, m.created_at \
         FROM messages m JOIN sessions s ON s.id = m.session_id \
         WHERE s.org_id = $1 AND m.sender_channel_identity_id IS NULL AND m.created_at > $2 \
           AND m.content_blocks IS NULL AND m.content NOT LIKE '🧠%' \
           AND NOT EXISTS (SELECT 1 FROM messages u WHERE u.session_id = m.session_id \
                           AND u.sender_channel_identity_id IS NOT NULL AND u.created_at > m.created_at) \
         ORDER BY m.session_id, m.created_at DESC",
    )
    .bind(org_id)
    .bind(since)
    .fetch_all(pool)
    .await
    ?;
    for (session_id, title, agent, content, at) in replies {
        if out.iter().any(|item| item.session_id == Some(session_id)) {
            continue;
        }
        let agent = agent.unwrap_or_else(|| "Nomi".to_string());
        out.push(OutItem {
            kind: "reply".into(),
            title: format!(
                "{agent} replied in {}",
                clip(title.as_deref().unwrap_or("a chat"), 40)
            ),
            detail: clip(&plain(&content), 90),
            agent,
            session_id: Some(session_id),
            at,
        });
    }

    // Waiting on the user first, then newest.
    out.sort_by(|a, b| {
        (b.kind == "needs_you")
            .cmp(&(a.kind == "needs_you"))
            .then(b.at.cmp(&a.at))
    });

    Ok(out)
}

/// What's scheduled for today in the user's timezone, in time order.
async fn today(
    pool: &sqlx::PgPool,
    user_id: Uuid,
    tz: chrono_tz::Tz,
) -> Result<Vec<TodayItem>, sqlx::Error> {
    // Today, in the user's timezone.
    let local_today = Utc::now().with_timezone(&tz).date_naive();
    let day_start = tz
        .from_local_datetime(&local_today.and_time(NaiveTime::MIN))
        .earliest()
        .map(|t| t.with_timezone(&Utc));
    let day_start = day_start.unwrap_or_else(Utc::now);
    let today: Vec<TodayRow> = sqlx::query_as(
        "SELECT id, run_at, label, target_agent_type, recurrence FROM scheduled_jobs \
         WHERE user_id = $1 AND status = 'active' AND run_at >= $2 AND run_at < $2 + interval '1 day' ORDER BY run_at",
    )
    .bind(user_id)
    .bind(day_start)
    .fetch_all(pool)
    .await
    ?;
    let reminders_today: Vec<TodayRow> = sqlx::query_as(
        "SELECT id, due_at, title, 'reminders', recurrence FROM reminders \
         WHERE user_id = $1 AND status = 'active' AND due_at >= $2 AND due_at < $2 + interval '1 day'",
    )
    .bind(user_id)
    .bind(day_start)
    .fetch_all(pool)
    .await
    ?;
    let mut today: Vec<TodayItem> = today
        .into_iter()
        .chain(reminders_today)
        .map(|(id, run_at, label, agent, recurrence)| TodayItem {
            id,
            run_at,
            label,
            agent,
            recurrence,
        })
        .collect();
    today.sort_by_key(|item| item.run_at);

    Ok(today)
}

/// Plans still in progress, latest first.
async fn plans(
    pool: &sqlx::PgPool,
    user_id: Uuid,
    org_id: Uuid,
) -> Result<Vec<PlanItem>, sqlx::Error> {
    // Plans in progress: the latest to-do list in each chat, and each written plan's latest
    // version, while they still have open items.
    let mut plans: Vec<PlanItem> = Vec::new();
    let todos: Vec<TodoRow> = sqlx::query_as(
        "SELECT DISTINCT ON (m.session_id) m.session_id, s.title, m.agent_display_name, m.content_blocks->0->'items', m.created_at \
         FROM messages m JOIN sessions s ON s.id = m.session_id \
         WHERE s.org_id = $1 AND m.content_blocks->0->>'kind' = 'todo_list' \
         ORDER BY m.session_id, m.created_at DESC",
    )
    .bind(org_id)
    .fetch_all(pool)
    .await
    ?;
    for (session_id, title, agent, items, updated_at) in todos {
        let items = items.as_array().cloned().unwrap_or_default();
        let total = items.len() as i64;
        let done = items.iter().filter(|i| i["status"] == "done").count() as i64;
        if total > 0 && done < total {
            plans.push(PlanItem {
                kind: "todo".into(),
                title: title.unwrap_or_else(|| "To-do".to_string()),
                agent: agent.unwrap_or_else(|| "Nomi".to_string()),
                done,
                total,
                session_id,
                updated_at,
            });
        }
    }
    let written: Vec<(Uuid, String, Option<String>, DateTime<Utc>)> = sqlx::query_as(
        "SELECT DISTINCT ON (p.agent_session_id) p.session_id, p.title, p.content, p.created_at \
         FROM agent_plans p JOIN sessions s ON s.id = p.session_id \
         WHERE p.user_id = $1 AND s.org_id = $2 ORDER BY p.agent_session_id, p.version DESC",
    )
    .bind(user_id)
    .bind(org_id)
    .fetch_all(pool)
    .await?;
    for (session_id, title, content, updated_at) in written {
        let (done, total) = checklist_progress(content.as_deref().unwrap_or_default());
        if total > 0 && done < total {
            plans.push(PlanItem {
                kind: "plan".into(),
                title,
                agent: "planning".into(),
                done,
                total,
                session_id,
                updated_at,
            });
        }
    }
    plans.sort_by_key(|plan| std::cmp::Reverse(plan.updated_at));
    Ok(plans)
}

pub async fn home_summary(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
) -> Result<Json<HomeSummary>, (StatusCode, &'static str)> {
    let pool = &state.pool;
    let user_id = claims.sub;
    let org_id = claims.active_org_id;
    let since = visit_since(pool, user_id).await.map_err(internal)?;
    let (timezone, tz) = user_timezone(pool, user_id).await.map_err(internal)?;

    let mut out = while_you_were_out(pool, user_id, org_id, since)
        .await
        .map_err(internal)?;
    let mut today = today(pool, user_id, tz).await.map_err(internal)?;
    let mut plans = plans(pool, user_id, org_id).await.map_err(internal)?;
    let (while_you_were_out_total, today_total, plans_total) =
        (out.len(), today.len(), plans.len());
    out.truncate(PREVIEW_LIMIT);
    today.truncate(PREVIEW_LIMIT);
    plans.truncate(PREVIEW_LIMIT);

    Ok(Json(HomeSummary {
        since,
        timezone,
        while_you_were_out: out,
        today,
        plans,
        while_you_were_out_total,
        today_total,
        plans_total,
    }))
}

fn section_page<T>(
    since: DateTime<Utc>,
    timezone: String,
    items: Vec<T>,
    query: &PageQuery,
) -> SectionPage<T> {
    let (page, per_page) = page_params(query);
    let total = items.len();
    SectionPage {
        since,
        timezone,
        items: page_of(items, page, per_page),
        total,
        page,
        per_page,
    }
}

pub async fn updates_page(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Query(query): Query<PageQuery>,
) -> Result<Json<SectionPage<OutItem>>, (StatusCode, &'static str)> {
    let pool = &state.pool;
    let since = peek_since(pool, claims.sub).await.map_err(internal)?;
    let (timezone, _) = user_timezone(pool, claims.sub).await.map_err(internal)?;
    let items = while_you_were_out(pool, claims.sub, claims.active_org_id, since)
        .await
        .map_err(internal)?;
    Ok(Json(section_page(since, timezone, items, &query)))
}

pub async fn today_page(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Query(query): Query<PageQuery>,
) -> Result<Json<SectionPage<TodayItem>>, (StatusCode, &'static str)> {
    let pool = &state.pool;
    let since = peek_since(pool, claims.sub).await.map_err(internal)?;
    let (timezone, tz) = user_timezone(pool, claims.sub).await.map_err(internal)?;
    let items = today(pool, claims.sub, tz).await.map_err(internal)?;
    Ok(Json(section_page(since, timezone, items, &query)))
}

pub async fn plans_page(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Query(query): Query<PageQuery>,
) -> Result<Json<SectionPage<PlanItem>>, (StatusCode, &'static str)> {
    let pool = &state.pool;
    let since = peek_since(pool, claims.sub).await.map_err(internal)?;
    let (timezone, _) = user_timezone(pool, claims.sub).await.map_err(internal)?;
    let items = plans(pool, claims.sub, claims.active_org_id)
        .await
        .map_err(internal)?;
    Ok(Json(section_page(since, timezone, items, &query)))
}

#[cfg(test)]
mod tests {
    use super::{checklist_progress, page_of, page_params, PageQuery};

    #[test]
    fn pages_count_from_one_and_clamp() {
        assert_eq!(
            page_params(&PageQuery {
                page: None,
                per_page: None
            }),
            (1, 20)
        );
        assert_eq!(
            page_params(&PageQuery {
                page: Some(0),
                per_page: Some(500)
            }),
            (1, 50)
        );
        let items: Vec<i32> = (1..=9).collect();
        assert_eq!(page_of(items.clone(), 1, 4), vec![1, 2, 3, 4]);
        assert_eq!(page_of(items.clone(), 3, 4), vec![9]);
        assert!(page_of(items, 4, 4).is_empty());
    }

    #[test]
    fn counts_task_list_items() {
        assert_eq!(
            checklist_progress(
                "# Plan\n- [x] one\n- [X] two\n- [ ] three\n* [ ] four\n- plain bullet"
            ),
            (2, 4)
        );
        assert_eq!(checklist_progress("no tasks here"), (0, 0));
    }
}
