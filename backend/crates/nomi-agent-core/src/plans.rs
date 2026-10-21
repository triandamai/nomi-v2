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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_unticked_items_are_left_to_do() {
        let plan = "## Steps\n- [x] Pick the colours\n- [ ] Home shows the menu\n  * [ ] Contact page\n+  [ ]  Gallery \n- [ ]\n-[ ] no space\nSome prose - [ ] not an item";
        assert_eq!(open_checklist_items(plan), vec!["Home shows the menu", "Contact page", "Gallery"]);
        assert_eq!(checklist_progress(plan), (1, 4));
    }
}
