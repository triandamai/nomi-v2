//! An issue's own chat: the project's issue board sends an issue to the crew in a chat of its own
//! (nomi-server's `issue_worker`). Every agent working there is told which issue it is, who it was
//! assigned to, and which other issues of the project are being worked at the same time.

use uuid::Uuid;

/// What the agent's system prompt gets in an issue's chat; `None` anywhere else.
pub async fn prompt_for(conn: &mut sqlx::PgConnection, session_id: Uuid) -> Option<String> {
    let (project_id, number, title, crew_agent): (Uuid, i32, String, String) =
        sqlx::query_as("SELECT project_id, number, title, crew_agent FROM project_issues WHERE session_id = $1")
            .bind(session_id)
            .fetch_optional(&mut *conn)
            .await
            .ok()
            .flatten()?;
    let others: Vec<(i32, String, String)> = sqlx::query_as(
        "SELECT number, title, crew_agent FROM project_issues \
         WHERE project_id = $1 AND crew_state = 'working' AND number <> $2 ORDER BY number",
    )
    .bind(project_id)
    .bind(number)
    .fetch_all(&mut *conn)
    .await
    .unwrap_or_default();
    Some(prompt(number, &title, &crate::crew::display_name_for(&crew_agent), &others))
}

pub fn prompt(number: i32, title: &str, assignee: &str, others: &[(i32, String, String)]) -> String {
    let mut text = crate::prompts::ISSUE_CHAT_PROMPT
        .replace("{number}", &number.to_string())
        .replace("{title}", title)
        .replace("{assignee}", assignee);
    if !others.is_empty() {
        text.push_str("\n\n");
        text.push_str(crate::prompts::ISSUE_CHAT_OTHERS);
        for (n, t, agent) in others {
            text.push_str(&format!("\n- #{n} \"{t}\" ({})", crate::crew::display_name_for(agent)));
        }
    }
    text
}

#[cfg(test)]
mod tests {
    #[test]
    fn names_the_issue_its_assignee_and_the_others_running() {
        let text = super::prompt(12, "Login button does nothing", "Koda", &[(14, "Dark mode".into(), "coding".into())]);
        assert!(text.starts_with("This chat works issue #12 \"Login button does nothing\" from the project's issue board; Koda was assigned"), "{text}");
        assert!(text.ends_with("- #14 \"Dark mode\" (Koda)"), "{text}");
        assert!(!super::prompt(1, "x", "Dana", &[]).contains("Other issues"));
    }
}
