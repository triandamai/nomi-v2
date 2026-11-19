//! Reading back the plans agents write with `write_plan` (see engine.rs's `write_agent_plan`).

use sqlx::pool::PoolConnection;
use sqlx::Postgres;
use uuid::Uuid;

pub struct SavedPlan {
    pub title: String,
    pub version: i32,
    /// `None` when the plan lives in S3 and couldn't be fetched.
    pub content: Option<String>,
}

/// The newest plan in this chat written by an agent of `agent_type` (e.g. Rena's, as opposed to
/// notes Koda wrote while building). Delegated and default-agent runs write under the chat's own
/// id rather than an agent session of theirs, so they never match.
pub async fn latest_plan_by(
    conn: &mut PoolConnection<Postgres>,
    s3: Option<&nomi_storage::S3Config>,
    session_id: Uuid,
    agent_type: &str,
) -> Option<SavedPlan> {
    let (title, version, content, key): (String, i32, Option<String>, Option<String>) = sqlx::query_as(
        "SELECT p.title, p.version, p.content, p.content_s3_key FROM agent_plans p \
         JOIN agent_sessions a ON a.id = p.agent_session_id \
         WHERE p.session_id = $1 AND a.agent_type = $2 ORDER BY p.created_at DESC LIMIT 1",
    )
    .bind(session_id)
    .bind(agent_type)
    .fetch_optional(&mut **conn)
    .await
    .ok()
    .flatten()?;
    let content = match (content, key, s3) {
        (Some(inline), _, _) => Some(inline),
        (None, Some(key), Some(s3)) => s3.get_object(&key).await.ok().flatten(),
        _ => None,
    };
    Some(SavedPlan { title, version, content })
}

/// The plan's checklist items still to do (`- [ ] …` lines), which say what "done" means.
pub fn open_checklist_items(content: &str) -> Vec<&str> {
    content
        .lines()
        .filter_map(|line| match task_item(line) {
            Some((box_at, false)) => Some(line[box_at + 3..].trim()),
            _ => None,
        })
        .collect()
}

/// (ticked, all) task-list items in a plan's markdown, as the chat's checklist counts them.
pub fn checklist_progress(markdown: &str) -> (usize, usize) {
    markdown.split('\n').filter_map(task_item).fold((0, 0), |(done, all), (_, ticked)| (done + usize::from(ticked), all + 1))
}

/// For a task-list line (`- [ ] …`, `* [x] …`, `+ [X] …`): where its box starts, and whether it's
/// ticked.
pub fn task_item(line: &str) -> Option<(usize, bool)> {
    let indent = line.len() - line.trim_start().len();
    let rest = &line[indent..];
    if !(rest.starts_with(['-', '*', '+']) && rest[1..].starts_with(char::is_whitespace)) {
        return None;
    }
    let after = rest[1..].trim_start();
    let ticked = match after.get(..3) {
        Some("[ ]") => false,
        Some("[x]" | "[X]") => true,
        _ => return None,
    };
    if !after[3..].starts_with(char::is_whitespace) || after[3..].trim().is_empty() {
        return None;
    }
    Some((indent + 1 + (rest[1..].len() - after.len()), ticked))
}

/// How a proposed plan is taken up: carried out, or only looked into first.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunMode {
    Run,
    Research,
}

impl RunMode {
    /// `run` or `research`; anything else is not a mode.
    pub fn parse(mode: &str) -> Option<Self> {
        match mode {
            "run" => Some(Self::Run),
            "research" => Some(Self::Research),
            _ => None,
        }
    }

    /// What the person asks the crew, as they would have typed it.
    pub fn request(self, title: &str) -> String {
        let template = match self {
            Self::Run => crate::prompts::PLAN_RUN_REQUEST,
            Self::Research => crate::prompts::PLAN_RESEARCH_REQUEST,
        };
        template.replace("{title}", title)
    }
}

/// What a plan's checklist is estimated at per step still open, when nothing better is known.
pub const PLAN_STEP_ESTIMATE_SECONDS: u32 = 60;

/// How the build that followed a plan stands, as far as its steps are concerned.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BuildStand<'a> {
    /// The delegation's status: `pending`, `claimed`, `processing`, `completed`, `failed`, `cancelled`.
    pub status: &'a str,
    /// It finished, but its checks on the final files failed.
    pub failing: bool,
    pub error: Option<&'a str>,
}

/// One step of a plan, as the plan card draws it.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct PlanStep {
    pub text: String,
    /// The crew member doing it (the plan's own, when the plan names no one else).
    pub owner_agent_type: Option<String>,
    /// `waiting`, `running`, `done` or `failed`.
    pub status: &'static str,
    pub status_detail: Option<String>,
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    pub finished_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// The steps of a plan written as a markdown checklist, with where each stands: ticked ones are
/// done; of the rest, while a build runs the first is running; once it failed or was stopped the
/// first is failed (with the build's error); once it finished without failing checks every step
/// is done. No per-step times: a checklist doesn't record them.
pub fn steps_from_checklist(markdown: &str, owner_agent_type: Option<&str>, build: Option<BuildStand<'_>>) -> Vec<PlanStep> {
    let all_done = matches!(build, Some(b) if b.status == "completed" && !b.failing);
    let stuck = matches!(build, Some(b) if matches!(b.status, "failed" | "cancelled") || (b.status == "completed" && b.failing));
    let running = matches!(build, Some(b) if !matches!(b.status, "completed" | "failed" | "cancelled"));
    let mut open_seen = false;
    markdown
        .lines()
        .filter_map(|line| {
            let (box_at, ticked) = task_item(line)?;
            let text = line[box_at + 3..].trim().to_string();
            let first_open = !ticked && !all_done && !std::mem::replace(&mut open_seen, true);
            let (status, status_detail) = match (ticked || all_done, first_open) {
                (true, _) => ("done", None),
                (false, true) if stuck => ("failed", build.and_then(|b| b.error).map(str::to_string)),
                (false, true) if running => ("running", None),
                _ => ("waiting", None),
            };
            Some(PlanStep { text, owner_agent_type: owner_agent_type.map(str::to_string), status, status_detail, started_at: None, finished_at: None })
        })
        .collect()
}

/// How long the steps still open are expected to take; `None` when the plan has no steps.
pub fn estimate_seconds(steps: &[PlanStep]) -> Option<u32> {
    if steps.is_empty() {
        return None;
    }
    let open = steps.iter().filter(|s| s.status != "done").count() as u32;
    Some(open * PLAN_STEP_ESTIMATE_SECONDS)
}

#[cfg(test)]
mod tests {
    use super::*;

    const PLAN: &str = "# Cafe site\n- [x] Pick the colours\n- [ ] Home shows the menu\n- [ ] Contact page\nNotes - [ ] not a step";

    fn statuses(steps: &[PlanStep]) -> Vec<&'static str> {
        steps.iter().map(|s| s.status).collect()
    }

    #[test]
    fn a_checklist_becomes_steps_owned_by_the_plans_agent() {
        let steps = steps_from_checklist(PLAN, Some("planning"), None);
        assert_eq!(steps.iter().map(|s| s.text.as_str()).collect::<Vec<_>>(), vec!["Pick the colours", "Home shows the menu", "Contact page"]);
        assert_eq!(statuses(&steps), vec!["done", "waiting", "waiting"]);
        assert!(steps.iter().all(|s| s.owner_agent_type.as_deref() == Some("planning") && s.started_at.is_none() && s.status_detail.is_none()));
        assert!(steps_from_checklist("no list here", None, None).is_empty());
    }

    #[test]
    fn a_running_build_puts_the_first_open_step_in_progress() {
        let build = BuildStand { status: "processing", failing: false, error: None };
        assert_eq!(statuses(&steps_from_checklist(PLAN, None, Some(build))), vec!["done", "running", "waiting"]);
    }

    #[test]
    fn a_failed_or_stopped_build_fails_the_first_open_step_with_its_error() {
        let build = BuildStand { status: "cancelled", failing: false, error: Some("Stopped by the user") };
        let steps = steps_from_checklist(PLAN, None, Some(build));
        assert_eq!(statuses(&steps), vec!["done", "failed", "waiting"]);
        assert_eq!(steps[1].status_detail.as_deref(), Some("Stopped by the user"));
        let failing = BuildStand { status: "completed", failing: true, error: None };
        assert_eq!(statuses(&steps_from_checklist(PLAN, None, Some(failing))), vec!["done", "failed", "waiting"]);
    }

    #[test]
    fn a_finished_build_leaves_every_step_done() {
        let build = BuildStand { status: "completed", failing: false, error: None };
        assert_eq!(statuses(&steps_from_checklist(PLAN, None, Some(build))), vec!["done", "done", "done"]);
    }

    #[test]
    fn only_run_and_research_are_modes_and_each_asks_for_its_own_thing() {
        assert_eq!(RunMode::parse("run"), Some(RunMode::Run));
        assert_eq!(RunMode::parse("research"), Some(RunMode::Research));
        assert_eq!(RunMode::parse("RUN"), None);
        assert_eq!(RunMode::parse(""), None);
        assert_eq!(RunMode::parse("explore"), None);
        let run = RunMode::Run.request("Cafe site");
        let research = RunMode::Research.request("Cafe site");
        assert!(run.contains("Cafe site") && research.contains("Cafe site"));
        assert_ne!(run, research);
        assert!(research.contains("Do not run"));
    }

    #[test]
    fn the_estimate_counts_the_steps_still_open() {
        let steps = steps_from_checklist(PLAN, None, None);
        assert_eq!(estimate_seconds(&steps), Some(2 * PLAN_STEP_ESTIMATE_SECONDS));
        let done = steps_from_checklist("- [x] a\n- [x] b", None, None);
        assert_eq!(estimate_seconds(&done), Some(0));
        assert_eq!(estimate_seconds(&[]), None);
    }

    #[test]
    fn only_unticked_items_are_left_to_do() {
        let plan = "## Steps\n- [x] Pick the colours\n- [ ] Home shows the menu\n  * [ ] Contact page\n+  [ ]  Gallery \n- [ ]\n-[ ] no space\nSome prose - [ ] not an item";
        assert_eq!(open_checklist_items(plan), vec!["Home shows the menu", "Contact page", "Gallery"]);
        assert_eq!(checklist_progress(plan), (1, 4));
    }
}
