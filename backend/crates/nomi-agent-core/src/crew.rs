//! The names people see for the built-in crew. Agents are still identified by their
//! `agent_type` ("money", "reminders"…) everywhere in code and data; these are what the chat,
//! the crew page, emails and hand-off messages call them.

/// The name of a built-in crew member, by `agent_type`.
pub fn crew_name(agent_type: &str) -> Option<&'static str> {
    Some(match agent_type {
        "chitchat" => "Nomi",
        "money" => "Dana",
        "reminders" => "Kala",
        "files" => "Maya",
        "planning" => "Rena",
        "coding" => "Koda",
        "workspace" => "Tara",
        "health" => "Raga",
        _ => return None,
    })
}

/// What to call an agent by its type: its crew name, else its type in Title Case.
pub fn display_name_for(agent_type: &str) -> String {
    if let Some(name) = crew_name(agent_type) {
        return name.to_string();
    }
    let mut chars = agent_type.chars();
    chars.next().map(|first| first.to_uppercase().chain(chars).collect()).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_the_crew_and_title_cases_the_rest() {
        assert_eq!(display_name_for("money"), "Dana");
        assert_eq!(display_name_for("chitchat"), "Nomi");
        assert_eq!(display_name_for("supervisor"), "Supervisor");
    }
}

/// What the rest of the crew is doing in this chat right now, and what it just finished, so an
/// agent never redoes or talks over another's work. `None` when nobody else is busy.
pub async fn crew_board(conn: &mut sqlx::PgConnection, session_id: uuid::Uuid, me: &str) -> Option<String> {
    let rows: Vec<(String, String, String, Option<String>, Option<String>, i64)> = sqlx::query_as(
        "SELECT d.target_agent_type, d.task, d.status, d.result, \
                (SELECT s.current_phase FROM agent_sessions s WHERE s.session_id = d.session_id AND s.agent_type = d.target_agent_type \
                 ORDER BY s.last_activity_at DESC LIMIT 1), \
                EXTRACT(EPOCH FROM (now() - COALESCE(d.completed_at, d.created_at)))::bigint \
         FROM agent_delegations d \
         WHERE d.session_id = $1 AND d.target_agent_type <> $2 \
           AND (d.status IN ('pending', 'claimed', 'processing') OR d.completed_at > now() - interval '20 minutes') \
         ORDER BY d.created_at DESC LIMIT 8",
    )
    .bind(session_id)
    .bind(me)
    .fetch_all(conn)
    .await
    .ok()?;
    if rows.is_empty() {
        return None;
    }
    let clip = |text: &str, max: usize| -> String {
        let text = text.trim().replace('\n', " ");
        if text.chars().count() <= max { text } else { format!("{}…", text.chars().take(max).collect::<String>()) }
    };
    let mut lines = Vec::with_capacity(rows.len());
    for (agent, task, status, result, phase, ago) in rows {
        let who = display_name_for(&agent);
        let minutes = (ago / 60).max(0);
        let line = match status.as_str() {
            "pending" => format!("- {who}: queued to do \"{}\"", clip(&task, 160)),
            "claimed" | "processing" => {
                let doing = phase.filter(|p| p != "waiting").map(|p| format!(", now {}", p.replace('_', " "))).unwrap_or_default();
                format!("- {who}: working on \"{}\" (started {minutes} min ago{doing})", clip(&task, 160))
            }
            "completed" => format!("- {who}: finished \"{}\" {minutes} min ago: {}", clip(&task, 100), clip(result.as_deref().unwrap_or(""), 200)),
            "failed" => format!("- {who}: failed \"{}\" {minutes} min ago", clip(&task, 100)),
            _ => continue,
        };
        lines.push(line);
    }
    if lines.is_empty() {
        return None;
    }
    Some(format!(
        "## The crew right now\n{}\n\nWork with this: don't start what another agent is already doing or just finished, \
         and don't hand them the same task again. If your part depends on theirs, say you'll pick it up when they're \
         done rather than guessing their result.",
        lines.join("\n")
    ))
}
