//! A project card's build progress ("3 of 6", "writing routes/menu/+page.svelte"): what the
//! crew already reports while it works on a project. The step comes from the newest plan's
//! checklist in the project's chat, the target from the latest tool call of the agent running now.

use std::collections::HashMap;

use serde::Serialize;
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

use super::projects::ProjectSummary;

/// The longest target shown on a card.
const DOING_MAX_CHARS: usize = 160;
/// The tool inputs, in order, that name what a tool is working on: a file, a page, a search.
const SUBJECT_KEYS: [&str; 5] = ["path", "file_path", "url", "query", "name"];

/// `build` on a project summary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BuildProgress {
    /// The step being worked on, counting from 1 (0 while there is no plan to count in).
    pub step: usize,
    /// How many steps the plan has (0 without a plan).
    pub steps: usize,
    pub doing: Option<String>,
}

/// Progress from a checklist's (ticked, all) counts: the step at hand is the one after those done.
pub fn build_progress(done: usize, total: usize, doing: Option<String>) -> BuildProgress {
    let step = if total == 0 { 0 } else { (done + 1).min(total) };
    BuildProgress { step, steps: total, doing }
}

/// What a tool call is working on: its target (a path, a page, a search), else the tool's name.
pub fn doing_from(tool: &str, input: &Value) -> Option<String> {
    let target = SUBJECT_KEYS.iter().find_map(|key| input.get(*key).and_then(Value::as_str).map(str::trim).filter(|v| !v.is_empty()));
    let text = target.unwrap_or_else(|| tool.trim());
    (!text.is_empty()).then(|| text.chars().take(DOING_MAX_CHARS).collect())
}

/// Whether the crew is at work: a turn is being run for the project's chats. An agent session
/// that stays `active` while it waits for the person's next message is not work.
pub fn at_work(turn_running: bool, _agent_waiting_on_person: bool) -> bool {
    turn_running
}

#[derive(sqlx::FromRow)]
struct Building {
    project_id: Uuid,
    turn_running: bool,
    agent_active: bool,
    tool: Option<Value>,
    plan_content: Option<String>,
}

/// Fills `build` on the summaries whose crew is at work right now; the rest stay null.
pub async fn attach(pool: &PgPool, projects: &mut [ProjectSummary]) -> Result<(), sqlx::Error> {
    if projects.is_empty() {
        return Ok(());
    }
    let ids: Vec<Uuid> = projects.iter().map(|p| p.id).collect();
    let rows: Vec<Building> = sqlx::query_as(
        "SELECT p.id AS project_id, \
                EXISTS (SELECT 1 FROM turn_jobs j WHERE j.status = 'processing' \
                        AND j.session_id IN (SELECT s.id FROM sessions s WHERE s.id = p.session_id OR s.in_project_id = p.id)) AS turn_running, \
                COALESCE(a.last_tool IS NOT NULL OR a.active, false) AS agent_active, a.last_tool AS tool, pl.content AS plan_content \
         FROM projects p \
         LEFT JOIN LATERAL ( \
             SELECT true AS active, (SELECT e.payload FROM agent_events e WHERE e.agent_session_id = ag.id AND e.event_type = 'ToolCalled' \
                     ORDER BY e.created_at DESC LIMIT 1) AS last_tool \
             FROM agent_sessions ag \
             WHERE ag.status = 'active' AND ag.session_id IN (SELECT s.id FROM sessions s WHERE s.id = p.session_id OR s.in_project_id = p.id) \
             ORDER BY ag.last_activity_at DESC LIMIT 1 \
         ) a ON true \
         LEFT JOIN LATERAL ( \
             SELECT content FROM agent_plans WHERE session_id = p.session_id ORDER BY created_at DESC LIMIT 1 \
         ) pl ON true \
         WHERE p.id = ANY($1)",
    )
    .bind(&ids)
    .fetch_all(pool)
    .await?;
    let mut by_project: HashMap<Uuid, Building> = rows.into_iter().map(|row| (row.project_id, row)).collect();
    for project in projects {
        if let Some(row) = by_project.remove(&project.id).filter(|row| at_work(row.turn_running, row.agent_active)) {
            let (done, total) = row.plan_content.as_deref().map(nomi_agent_core::plans::checklist_progress).unwrap_or((0, 0));
            let doing = row.tool.as_ref().and_then(|payload| {
                let tool = payload.get("tool_name").and_then(Value::as_str)?;
                doing_from(tool, payload.get("input").unwrap_or(&Value::Null))
            });
            project.build = serde_json::to_value(build_progress(done, total, doing)).ok();
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn the_step_at_hand_is_the_one_after_those_done() {
        assert_eq!(build_progress(2, 6, Some("a".into())), BuildProgress { step: 3, steps: 6, doing: Some("a".into()) });
        assert_eq!(build_progress(0, 6, None), BuildProgress { step: 1, steps: 6, doing: None });
        // Everything ticked but still working: the last step, not one past it.
        assert_eq!(build_progress(6, 6, None).step, 6);
        // No plan to count in.
        assert_eq!(build_progress(0, 0, None), BuildProgress { step: 0, steps: 0, doing: None });
    }

    #[test]
    fn an_agent_waiting_on_the_person_is_not_building() {
        assert!(!at_work(false, true));
        assert!(!at_work(false, false));
        assert!(at_work(true, true));
        assert!(at_work(true, false));
    }

    #[test]
    fn doing_names_the_target_of_the_latest_tool_call() {
        assert_eq!(doing_from("write_file", &json!({ "path": "routes/menu/+page.svelte", "content": "x" })).as_deref(), Some("routes/menu/+page.svelte"));
        assert_eq!(doing_from("web_search", &json!({ "query": " svelte 5 runes " })).as_deref(), Some("svelte 5 runes"));
        // No target in the input: the tool's own name.
        assert_eq!(doing_from("run_tests", &json!({})).as_deref(), Some("run_tests"));
        assert_eq!(doing_from("run_tests", &Value::Null).as_deref(), Some("run_tests"));
        assert_eq!(doing_from("", &Value::Null), None);
        let long = "x".repeat(DOING_MAX_CHARS * 2);
        assert_eq!(doing_from("read_file", &json!({ "path": long })).map(|s| s.chars().count()), Some(DOING_MAX_CHARS));
    }
}
