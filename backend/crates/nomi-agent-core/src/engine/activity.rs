//! What the person sees of an agent at work: its phase and what it's working on, activity
//! messages, and the log of its tool calls.

use super::*;

/// The plain text a posted card stands for (a view's own text).
pub(super) fn card_text(block: &crate::content_block::ContentBlock) -> String {
    match block {
        crate::content_block::ContentBlock::View(view) => view.plain_text(),
        _ => String::new(),
    }
}

pub(super) async fn log_tool_call(
    conn: &mut PoolConnection<Postgres>,
    session_id: Uuid,
    agent_session_id: Uuid,
    agent_type: &str,
    tool_name: &str,
    input: &serde_json::Value,
    result: &str,
    is_error: bool,
) {
    if is_error {
        tracing::warn!(
            %session_id,
            %agent_session_id,
            agent = %agent_type,
            tool = %tool_name,
            result = %result,
            "agent tool execution failed"
        );
    } else {
        tracing::info!(
            %session_id,
            %agent_session_id,
            agent = %agent_type,
            tool = %tool_name,
            "agent tool execution succeeded"
        );
    }
    let _ = sqlx::query(
        "INSERT INTO agent_events (session_id, agent_session_id, agent_type, event_type, payload) VALUES ($1, (SELECT id FROM agent_sessions WHERE id = $2), $3, 'ToolCalled', $4)",
    )
    .bind(session_id)
    .bind(agent_session_id)
    .bind(agent_type)
    .bind(serde_json::json!({"tool_name": tool_name, "input": input, "result": result, "is_error": is_error}))
    .execute(&mut **conn)
    .await;
}

/// Best-effort (never fails the turn, matches `post_activity_message`'s convention): updates
/// `agent_sessions.current_phase`/`current_phase_detail` and, when `mqtt` is available, pushes
/// the same change live over `StreamEnvelope::AgentPhaseChanged`. Silently a no-op for the
/// default agent's turns, where `agent_session_id` is the `session_id` sentinel (see
/// `nomi_turn::run_locked_turn`'s call site comment) — there is no real `agent_sessions` row
/// with that id, so the UPDATE affects zero rows and nothing is published.
pub(super) async fn update_agent_phase(
    conn: &mut PoolConnection<Postgres>,
    mqtt: Option<&MqttPublisher>,
    session_id: Uuid,
    agent_session_id: Uuid,
    phase: &str,
    detail: Option<&str>,
) {
    update_agent_phase_about(conn, mqtt, session_id, agent_session_id, phase, detail, None).await;
}

/// The longest subject a phase carries; past it the start of the text stands for the rest.
pub(super) const PHASE_SUBJECT_MAX_CHARS: usize = 160;

/// The inputs, in order, that name what a tool is working on: a file, a page, a search.
pub(super) const PHASE_SUBJECT_KEYS: [&str; 5] = ["path", "file_path", "url", "query", "name"];

/// What a tool call is about, from its input (`PHASE_SUBJECT_KEYS`), for the phase's `subject`.
pub(super) fn tool_subject(input: &serde_json::Value) -> Option<String> {
    let text = PHASE_SUBJECT_KEYS.iter().find_map(|key| input.get(*key).and_then(|v| v.as_str()).map(str::trim).filter(|v| !v.is_empty()))?;
    Some(text.chars().take(PHASE_SUBJECT_MAX_CHARS).collect())
}

/// [`update_agent_phase`], saying what the phase is about (`subject`) on the live event.
pub(super) async fn update_agent_phase_about(
    conn: &mut PoolConnection<Postgres>,
    mqtt: Option<&MqttPublisher>,
    session_id: Uuid,
    agent_session_id: Uuid,
    phase: &str,
    detail: Option<&str>,
    subject: Option<String>,
) {
    let updated: Result<Option<(String, Option<String>)>, _> = sqlx::query_as(&format!(
        "UPDATE agent_sessions a SET current_phase = $1, current_phase_detail = $2 WHERE a.id = $3 RETURNING {}",
        crate::crew::SESSION_AGENT_COLUMNS
    ))
    .bind(phase)
    .bind(detail)
    .bind(agent_session_id)
    .fetch_optional(&mut **conn)
    .await;

    if let (Ok(Some((agent_type, dynamic_name))), Some(publisher)) = (updated, mqtt) {
        let envelope = StreamEnvelope::AgentPhaseChanged {
            agent_session_id,
            phase: phase.to_string(),
            detail: detail.map(|d| d.to_string()),
            agent_display_name: Some(crate::crew::session_agent_name(&agent_type, dynamic_name)),
            subject,
        };
        let _ = publisher.publish(session_id, &envelope).await;
    }
}

/// The newest recording (a voice block) `agent_display_name` posted in the chat since `since`,
/// with the words it says.
/// The newest recording or view this agent posted in this turn, which can stand as its reply:
/// its id and its words (what the recording says, or the view as text).
pub(super) async fn answer_posted_since(
    conn: &mut PoolConnection<Postgres>,
    session_id: Uuid,
    agent_display_name: &str,
    since: chrono::DateTime<chrono::Utc>,
) -> Option<(Uuid, String)> {
    sqlx::query_as(
        "SELECT id, CASE WHEN content_blocks->0->>'kind' = 'voice' THEN content_blocks->0->>'text' ELSE content END FROM messages \
         WHERE session_id = $1 AND agent_display_name = $2 AND created_at >= $3 AND content_blocks->0->>'kind' IN ('voice', 'view') \
         ORDER BY created_at DESC LIMIT 1",
    )
    .bind(session_id)
    .bind(agent_display_name)
    .bind(since)
    .fetch_optional(&mut **conn)
    .await
    .ok()
    .flatten()
    .map(|(id, text): (Uuid, Option<String>)| (id, text.unwrap_or_default()))
}

/// Inserts an activity message (an agent's "thought", or a description of a tool call it just
/// made) so a user watching a long-running build sees it appear like any other chat message —
/// see `SubAgent::surfaces_activity`. Best-effort: never fails the turn.
pub async fn post_activity_message(
    conn: &mut PoolConnection<Postgres>,
    mqtt: Option<&MqttPublisher>,
    session_id: Uuid,
    agent_display_name: &str,
    content: &str,
    block: Option<&crate::content_block::ContentBlock>,
) -> Option<Uuid> {
    let content_blocks = block.map(|b| serde_json::json!([b]));
    let message_id: Option<Uuid> = sqlx::query_scalar(
        "INSERT INTO messages (session_id, sender_channel_identity_id, content, content_blocks, agent_display_name) VALUES ($1, NULL, $2, $3, $4) RETURNING id",
    )
    .bind(session_id)
    .bind(content)
    .bind(&content_blocks)
    .bind(agent_display_name)
    .fetch_one(&mut **conn)
    .await
    .ok();

    if let (Some(id), Some(publisher)) = (message_id, mqtt) {
        let _ = publisher.publish(session_id, &StreamEnvelope::MessageCreated { message_id: id }).await;
    }
    message_id
}

#[cfg(test)]
mod tool_subject_tests {
    use super::*;

    #[test]
    fn a_tool_is_about_its_file_page_or_search_in_that_order() {
        assert_eq!(tool_subject(&serde_json::json!({ "path": "src/app.ts", "query": "x" })).as_deref(), Some("src/app.ts"));
        assert_eq!(tool_subject(&serde_json::json!({ "query": " kos murah " })).as_deref(), Some("kos murah"));
        assert_eq!(tool_subject(&serde_json::json!({ "path": "  ", "content": "x" })), None);
    }

    #[test]
    fn a_long_subject_is_cut_to_its_start() {
        let long = "a".repeat(PHASE_SUBJECT_MAX_CHARS * 2);
        assert_eq!(tool_subject(&serde_json::json!({ "query": long })).map(|s| s.chars().count()), Some(PHASE_SUBJECT_MAX_CHARS));
    }
}
