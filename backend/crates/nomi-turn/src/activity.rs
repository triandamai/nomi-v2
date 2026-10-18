//! Telling the person how a chat ended up, wherever they are in the app: when the crew is done
//! with a chat for now (finished, failed, or waiting on them), a `ChatActivity` goes to their own
//! MQTT topic. The app subscribes to it once while it's open and shows a snackbar with a way back
//! (frontend: $lib/userActivity). The workers call `announce` where a turn ends.

use nomi_realtime::{MqttPublisher, UserEnvelope};
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

/// How the turn that just ended went.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ended {
    Completed,
    Failed,
}

/// What a block of the crew's leaves waiting on the person, if anything.
fn waiting_on(block: &Value) -> Option<&'static str> {
    match block.get("kind")?.as_str()? {
        "questions" => Some("question"),
        "approval_request" if block.get("status").and_then(|s| s.as_str()) == Some("pending") => Some("approval"),
        "workspace_connect" | "connector_connect" => Some("connect"),
        "view" if matches!(block.get("view").and_then(|v| v.as_str()), Some("decision" | "form")) => Some("question"),
        _ => None,
    }
}

/// Where the chat stands now that a turn in it ended, or `None` while the crew is still at it.
/// Something waiting on the person is said right away, even with other work still going (a
/// delegated agent paused for approval counts as running); otherwise only once nothing in the
/// chat is still queued or running, so a chat with several crew members says it once, at the end.
pub async fn chat_activity(pool: &PgPool, session_id: Uuid, ended: Ended) -> Result<Option<UserEnvelope>, sqlx::Error> {
    // The crew's messages since the person last wrote, oldest first.
    let replies: Vec<(Option<Value>, Option<String>)> = sqlx::query_as(
        "SELECT content_blocks, agent_display_name FROM messages \
         WHERE session_id = $1 AND sender_channel_identity_id IS NULL \
           AND created_at > COALESCE((SELECT max(created_at) FROM messages WHERE session_id = $1 AND sender_channel_identity_id IS NOT NULL), '-infinity') \
         ORDER BY created_at, id",
    )
    .bind(session_id)
    .fetch_all(pool)
    .await?;
    let waiting = replies
        .iter()
        .rev()
        .find_map(|(blocks, _)| blocks.as_ref().and_then(|b| b.as_array()).and_then(|blocks| blocks.iter().find_map(waiting_on)));

    if waiting.is_none() {
        let busy: bool = sqlx::query_scalar(
            "SELECT EXISTS (SELECT 1 FROM turn_jobs WHERE session_id = $1 AND status IN ('pending', 'processing')) \
                 OR EXISTS (SELECT 1 FROM agent_delegations WHERE session_id = $1 AND status IN ('pending', 'claimed', 'processing') \
                            AND created_at > now() - interval '30 minutes') \
                 OR EXISTS (SELECT 1 FROM approval_resumes r JOIN messages m ON m.id = r.message_id \
                            WHERE m.session_id = $1 AND r.status IN ('pending', 'processing'))",
        )
        .bind(session_id)
        .fetch_one(pool)
        .await?;
        if busy {
            return Ok(None);
        }
    }

    let (title, project_id): (Option<String>, Option<Uuid>) = sqlx::query_as(
        "SELECT COALESCE(p.name, s.title), p.id FROM sessions s LEFT JOIN projects p ON p.session_id = s.id WHERE s.id = $1",
    )
    .bind(session_id)
    .fetch_one(pool)
    .await?;
    let outcome = match (waiting, ended) {
        (Some(_), _) => "waiting",
        (None, Ended::Failed) => "failed",
        (None, Ended::Completed) => "done",
    };
    Ok(Some(UserEnvelope::ChatActivity {
        session_id,
        title,
        project_id,
        outcome: outcome.to_string(),
        waiting: waiting.map(str::to_string),
        agent: replies.iter().rev().find_map(|(_, agent)| agent.clone()),
    }))
}

/// Tells the person how the chat ended up, when the crew is done with it for now. Best effort.
pub async fn announce(pool: &PgPool, mqtt: &MqttPublisher, session_id: Uuid, user_id: Uuid, ended: Ended) {
    if let Ok(Some(envelope)) = chat_activity(pool, session_id, ended).await {
        let _ = mqtt.publish_to_user(user_id, &envelope).await;
    }
}
