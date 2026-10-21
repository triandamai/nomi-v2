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
        .filter_map(|line| {
            let line = line.trim_start();
            line.strip_prefix("- [ ]").or_else(|| line.strip_prefix("* [ ]")).map(str::trim)
        })
        .filter(|item| !item.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_unticked_items_are_left_to_do() {
        let plan = "## Steps\n- [x] Pick the colours\n- [ ] Home shows the menu\n  * [ ] Contact page\n- [ ]\nSome prose - [ ] not an item";
        assert_eq!(open_checklist_items(plan), vec!["Home shows the menu", "Contact page"]);
    }
}
