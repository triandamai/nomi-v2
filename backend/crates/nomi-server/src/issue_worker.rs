//! The crew working the issue board: an issue moved into Ready to dev is queued, and this worker
//! picks it up. Each issue gets a chat of its own inside the project (`chat_project()` maps it
//! back), started with the issue addressed to the crew member assigned to it, as the person who
//! queued it. Separate chats let several issues run at once, up to Admin → Configuration →
//! Agent loop → `issues_per_project` (1 when empty).
//!
//! The worker follows each one: when the crew member hands it on (`delegate_to_agent`, or a
//! hand-off inside the turn) the issue's assignee follows, and once the chat's turn and every
//! hand-over in it have ended the issue is marked done (it stays in Ready to dev, still open,
//! for the person to check: the crew ends by asking whether it is) or failed (with "Try again" on
//! the board). When the person answers in the issue's chat that it isn't done yet, the crew's
//! new turn there makes it `working` again; once someone moves it to Done (or Closed) it's
//! closed and its chat is read-only.

use std::time::Duration;

use chrono::NaiveDate;
use sqlx::PgPool;
use uuid::Uuid;

use crate::routes::project_issues::ISSUES_CHANGED;

/// How often queued issues are picked up and running ones followed.
const ISSUE_PICKUP_EVERY: Duration = Duration::from_secs(5);
/// Issues of one project the crew works at once, unless Admin → Configuration says otherwise…
pub const DEFAULT_ISSUES_PER_PROJECT: u64 = 1;
/// …and never more than this.
pub const MAX_ISSUES_PER_PROJECT: u64 = 5;
/// Issues picked up in one sweep, across every project.
const PICKUPS_PER_SWEEP: i64 = 20;
/// How long a picked-up issue may go without its chat's turn before it counts as failed (the
/// server stopped between picking it up and starting the chat).
const START_GRACE_SECS: f64 = 120.0;

/// What a sweep did, for the logs and tests.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Swept {
    pub picked_up: usize,
    pub handed_on: usize,
    pub finished: usize,
    /// Finished issues the crew took up again after a follow-up in their chat.
    pub resumed: usize,
}

/// The limit from Admin → Configuration, kept between 1 and `MAX_ISSUES_PER_PROJECT`.
pub async fn issues_per_project() -> u64 {
    let agent_loop = nomi_settings::config::current(&nomi_settings::config::AGENT_LOOP).await;
    clamp_per_project(agent_loop.number("issues_per_project"))
}

pub fn clamp_per_project(configured: Option<u64>) -> u64 {
    configured.unwrap_or(DEFAULT_ISSUES_PER_PROJECT).clamp(1, MAX_ISSUES_PER_PROJECT)
}

pub async fn run(pool: PgPool) {
    let mut tick = tokio::time::interval(ISSUE_PICKUP_EVERY);
    tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    tracing::info!("issue board: worker started");
    loop {
        tick.tick().await;
        let limit = issues_per_project().await;
        match sweep(&pool, limit).await {
            Ok(swept) if swept != Swept::default() => tracing::info!(?swept, "issue board: swept"),
            Ok(_) => {}
            Err(e) => tracing::warn!(error = %e, "issue board: sweep failed"),
        }
    }
}

/// Follows the issues being worked, then picks up queued ones while their projects have room.
pub async fn sweep(pool: &PgPool, per_project: u64) -> Result<Swept, sqlx::Error> {
    let resumed = resume_followed_up(pool).await?;
    let mut swept = follow_working(pool).await?;
    swept.resumed = resumed;
    swept.picked_up = pick_up(pool, per_project).await?;
    Ok(swept)
}

/// Open issues the crew finished (or couldn't) whose chat has a newer turn: the person followed
/// up ("not yet, the button is still grey"), so the crew is working on it again, on that turn.
async fn resume_followed_up(pool: &PgPool) -> Result<usize, sqlx::Error> {
    let resumed: Vec<(Uuid, i32)> = sqlx::query_as(
        "WITH latest AS ( \
             SELECT i.id, (SELECT j.id FROM turn_jobs j WHERE j.session_id = i.session_id ORDER BY j.created_at DESC LIMIT 1) AS job \
             FROM project_issues i \
             WHERE i.crew_state IN ('done', 'failed') AND i.status = 'open' AND i.session_id IS NOT NULL \
         ) \
         UPDATE project_issues i SET crew_state = 'working', turn_job_id = l.job, picked_up_at = now(), updated_at = now() \
         FROM latest l WHERE i.id = l.id AND l.job IS NOT NULL AND l.job IS DISTINCT FROM i.turn_job_id \
         RETURNING i.project_id, i.number",
    )
    .fetch_all(pool)
    .await?;
    for (project_id, number) in &resumed {
        tracing::info!(%project_id, issue = number, "issue board: followed up, working again");
        crate::project_events::changed(*project_id, ISSUES_CHANGED);
    }
    Ok(resumed.len())
}

#[derive(sqlx::FromRow)]
struct Working {
    id: Uuid,
    project_id: Uuid,
    number: i32,
    crew_agent: String,
    /// The chat's first turn: pending, processing, completed or failed; null when there's none.
    turn_status: Option<String>,
    /// Whoever was handed the work last in the issue's chat.
    latest_agent: Option<String>,
    /// Whether a hand-over in the chat is still waiting or running.
    handing_over: bool,
    /// How the last hand-over ended.
    last_handover: Option<String>,
    started_late: bool,
}

/// What a working issue is now: still `working`, `done` or `failed`.
pub fn outcome(turn_status: Option<&str>, handing_over: bool, last_handover: Option<&str>, started_late: bool) -> &'static str {
    match turn_status {
        None if started_late => "failed",
        None | Some("pending") | Some("processing") => "working",
        Some("failed") => "failed",
        _ if handing_over => "working",
        _ if matches!(last_handover, Some("failed") | Some("cancelled")) => "failed",
        _ => "done",
    }
}

async fn follow_working(pool: &PgPool) -> Result<Swept, sqlx::Error> {
    let rows: Vec<Working> = sqlx::query_as(
        "SELECT i.id, i.project_id, i.number, i.crew_agent, j.status AS turn_status, \
            (SELECT who FROM ( \
                SELECT d.target_agent_type AS who, d.created_at AS at FROM agent_delegations d WHERE d.session_id = i.session_id \
                UNION ALL \
                SELECT a.agent_type, a.started_at FROM agent_sessions a WHERE a.session_id = i.session_id \
             ) w ORDER BY at DESC LIMIT 1) AS latest_agent, \
            EXISTS (SELECT 1 FROM agent_delegations d WHERE d.session_id = i.session_id AND d.status IN ('pending', 'processing')) AS handing_over, \
            (SELECT d.status FROM agent_delegations d WHERE d.session_id = i.session_id ORDER BY d.created_at DESC LIMIT 1) AS last_handover, \
            (i.picked_up_at < now() - make_interval(secs => $1)) AS started_late \
         FROM project_issues i LEFT JOIN turn_jobs j ON j.id = i.turn_job_id \
         WHERE i.crew_state = 'working'",
    )
    .bind(START_GRACE_SECS)
    .fetch_all(pool)
    .await?;

    let mut swept = Swept::default();
    for row in rows {
        let mut changed = false;
        if let Some(latest) = row.latest_agent.as_deref().filter(|a| *a != row.crew_agent && nomi_agent_core::crew::crew_name(a).is_some()) {
            let handed = sqlx::query("UPDATE project_issues SET assigned_by_agent = crew_agent, crew_agent = $2, updated_at = now() WHERE id = $1 AND crew_agent <> $2")
                .bind(row.id)
                .bind(latest)
                .execute(pool)
                .await?;
            if handed.rows_affected() > 0 {
                tracing::info!(
                    project_id = %row.project_id,
                    issue = row.number,
                    from = %nomi_agent_core::crew::display_name_for(&row.crew_agent),
                    to = %nomi_agent_core::crew::display_name_for(latest),
                    "issue board: handed on"
                );
                swept.handed_on += 1;
                changed = true;
            }
        }
        let now = outcome(row.turn_status.as_deref(), row.handing_over, row.last_handover.as_deref(), row.started_late);
        if now != "working" {
            sqlx::query("UPDATE project_issues SET crew_state = $2, updated_at = now() WHERE id = $1 AND crew_state = 'working'")
                .bind(row.id)
                .bind(now)
                .execute(pool)
                .await?;
            tracing::info!(project_id = %row.project_id, issue = row.number, state = now, "issue board: finished");
            swept.finished += 1;
            changed = true;
        }
        if changed {
            crate::project_events::changed(row.project_id, ISSUES_CHANGED);
        }
    }
    Ok(swept)
}

#[derive(sqlx::FromRow)]
struct PickedUp {
    org_id: Option<Uuid>,
    number: i32,
    title: String,
    body: String,
    kind: String,
    severity: String,
    points: Option<i16>,
    start_on: Option<NaiveDate>,
    due_on: Option<NaiveDate>,
    crew_agent: String,
    queued_by: Option<Uuid>,
}

/// Queued issues, in Ready to dev order per project, while their project has room. With room for
/// one, an issue also waits while the crew is busy in the project's own chat: they'd be changing
/// the same files.
async fn pick_up(pool: &PgPool, per_project: u64) -> Result<usize, sqlx::Error> {
    let per_project = per_project as i64;
    let candidates: Vec<(Uuid, Uuid)> = sqlx::query_as(
        "WITH running AS ( \
             SELECT project_id, count(*) AS n FROM project_issues WHERE crew_state = 'working' GROUP BY project_id \
         ), queued AS ( \
             SELECT i.id, i.project_id, row_number() OVER (PARTITION BY i.project_id ORDER BY i.position, i.number) AS place \
             FROM project_issues i JOIN project_issue_columns c ON c.id = i.column_id AND c.fixed = 'ready' \
             WHERE i.crew_state = 'queued' AND i.status = 'open' \
         ) \
         SELECT q.id, q.project_id FROM queued q LEFT JOIN running r ON r.project_id = q.project_id \
         WHERE q.place <= $1 - COALESCE(r.n, 0) \
           AND ($1 > 1 OR NOT EXISTS ( \
               SELECT 1 FROM turn_jobs j JOIN projects p ON p.session_id = j.session_id \
               WHERE p.id = q.project_id AND j.status IN ('pending', 'processing'))) \
         ORDER BY q.place LIMIT $2",
    )
    .bind(per_project)
    .bind(PICKUPS_PER_SWEEP)
    .fetch_all(pool)
    .await?;

    let mut picked = 0;
    for (issue_id, project_id) in candidates {
        let Some(issue) = claim(pool, issue_id, project_id, per_project).await? else { continue };
        let started = start_chat(pool, issue_id, &issue).await;
        // The board hears once the issue's chat exists, so its card links to it.
        crate::project_events::changed(project_id, ISSUES_CHANGED);
        match started {
            Ok(session_id) => {
                tracing::info!(%project_id, issue = issue.number, %session_id, crew = %nomi_agent_core::crew::display_name_for(&issue.crew_agent), "issue board: picked up");
                picked += 1;
            }
            Err(e) => {
                tracing::warn!(%project_id, issue = issue.number, error = %e, "issue board: couldn't start the issue's chat");
                sqlx::query("UPDATE project_issues SET crew_state = 'failed', updated_at = now() WHERE id = $1").bind(issue_id).execute(pool).await?;
                crate::project_events::changed(project_id, ISSUES_CHANGED);
            }
        }
    }
    Ok(picked)
}

/// Marks the issue working, if it's still queued and its project still has room (another
/// server may be sweeping too: the project's lock makes them take turns).
async fn claim(pool: &PgPool, issue_id: Uuid, project_id: Uuid, per_project: i64) -> Result<Option<PickedUp>, sqlx::Error> {
    let mut tx = pool.begin().await?;
    sqlx::query("SELECT pg_advisory_xact_lock(hashtext('issue-board:' || $1::text))").bind(project_id).execute(&mut *tx).await?;
    let running: i64 = sqlx::query_scalar("SELECT count(*) FROM project_issues WHERE project_id = $1 AND crew_state = 'working'")
        .bind(project_id)
        .fetch_one(&mut *tx)
        .await?;
    if running >= per_project {
        return Ok(None);
    }
    let issue: Option<PickedUp> = sqlx::query_as(
        "UPDATE project_issues i SET crew_state = 'working', picked_up_at = now(), session_id = NULL, turn_job_id = NULL, \
            assigned_by_agent = NULL, updated_at = now() \
         FROM projects p WHERE p.id = i.project_id AND i.id = $1 AND i.crew_state = 'queued' \
         RETURNING p.org_id, i.number, i.title, i.body, i.kind, i.severity, i.points, i.start_on, i.due_on, i.crew_agent, i.queued_by",
    )
    .bind(issue_id)
    .fetch_optional(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(issue)
}

/// The issue as the message that starts its chat: addressed to its crew member by name, so the
/// turn goes to them.
pub fn issue_message(crew_name: &str, number: i32, title: &str, body: &str, details: &[String]) -> String {
    let mut text = format!("@{crew_name} Issue #{number}: {title}");
    if !details.is_empty() {
        text.push_str(&format!("\n\n{}", details.join(" · ")));
    }
    if !body.trim().is_empty() {
        text.push_str(&format!("\n\n{}", body.trim()));
    }
    text
}

fn details(issue: &PickedUp) -> Vec<String> {
    let mut details = vec![format!("Type: {}", issue.kind), format!("Severity: {}", issue.severity)];
    if let Some(points) = issue.points {
        details.push(format!("Points: {points}"));
    }
    if let Some(start) = issue.start_on {
        details.push(format!("Start: {start}"));
    }
    if let Some(due) = issue.due_on {
        details.push(format!("Deadline: {due}"));
    }
    details
}

/// Opens the issue's chat as the person who queued it, with the issue as its first message
/// (queued as a turn like anything they'd say), and links the chat to the issue.
async fn start_chat(pool: &PgPool, issue_id: Uuid, issue: &PickedUp) -> Result<Uuid, String> {
    let person = issue.queued_by.ok_or("nobody queued it (their account is gone)")?;
    nomi_turn::bootstrap::ensure_web_channel_identity(pool, person).await.map_err(|e| e.to_string())?;
    let crew_name = nomi_agent_core::crew::display_name_for(&issue.crew_agent);
    let text = issue_message(&crew_name, issue.number, &issue.title, &issue.body, &details(issue));
    let chat_id = Uuid::new_v4().to_string();
    let ingested = nomi_turn::ingest::ingest_inbound_message(pool, "web", "dm", &chat_id, &person.to_string(), &text, issue.org_id)
        .await
        .map_err(|e| e.to_string())?;
    let title = format!("#{} · {}", issue.number, issue.title);
    sqlx::query("UPDATE sessions SET title = $2 WHERE id = $1").bind(ingested.session_id).bind(&title).execute(pool).await.map_err(|e| e.to_string())?;
    sqlx::query("UPDATE project_issues SET session_id = $2, turn_job_id = $3 WHERE id = $1")
        .bind(issue_id)
        .bind(ingested.session_id)
        .bind(ingested.turn_job_id)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(ingested.session_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_limit_is_one_unless_set_and_never_more_than_five() {
        assert_eq!(clamp_per_project(None), 1);
        assert_eq!(clamp_per_project(Some(3)), 3);
        assert_eq!(clamp_per_project(Some(50)), MAX_ISSUES_PER_PROJECT);
    }

    #[test]
    fn an_issue_is_done_once_its_turn_and_every_hand_over_ended() {
        assert_eq!(outcome(Some("processing"), false, None, false), "working");
        assert_eq!(outcome(Some("completed"), true, Some("processing"), false), "working");
        assert_eq!(outcome(Some("completed"), false, Some("completed"), false), "done");
        assert_eq!(outcome(Some("completed"), false, None, false), "done");
        assert_eq!(outcome(Some("completed"), false, Some("failed"), false), "failed");
        assert_eq!(outcome(Some("failed"), false, None, false), "failed");
        assert_eq!(outcome(None, false, None, false), "working");
        assert_eq!(outcome(None, false, None, true), "failed");
    }

    #[test]
    fn the_issue_is_addressed_to_its_crew_member() {
        let text = issue_message("Koda", 12, "Login button does nothing", "On phones.\n", &["Type: bug".into(), "Points: 3".into()]);
        assert_eq!(text, "@Koda Issue #12: Login button does nothing\n\nType: bug · Points: 3\n\nOn phones.");
    }
}
