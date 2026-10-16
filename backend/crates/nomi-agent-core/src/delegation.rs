use sqlx::pool::PoolConnection;
use sqlx::Postgres;
use uuid::Uuid;

use nomi_realtime::{MqttPublisher, StreamEnvelope};

pub async fn create_delegation(
    conn: &mut PoolConnection<Postgres>,
    mqtt: Option<(&MqttPublisher, Uuid)>,
    session_id: Uuid,
    requesting_agent_type: &str,
    target_agent_type: &str,
    task: &str,
    user_id: Uuid,
) -> Result<String, String> {
    // The same agent already has this chat's work queued or in hand: two runs racing on one
    // task (and talking over each other in the chat) helps nobody.
    let busy: Option<String> = sqlx::query_scalar(
        "SELECT task FROM agent_delegations WHERE session_id = $1 AND target_agent_type = $2 \
         AND status IN ('pending', 'claimed', 'processing') AND created_at > now() - interval '30 minutes' \
         ORDER BY created_at DESC LIMIT 1",
    )
    .bind(session_id)
    .bind(target_agent_type)
    .fetch_optional(&mut **conn)
    .await
    .map_err(|e| format!("failed to check delegations: {e}"))?;
    if let Some(task_in_hand) = busy {
        return Ok(format!(
            "{target_agent_type} is already working on \"{task_in_hand}\" in this chat. Don't delegate again: tell the user \
             it's in progress and that its result will appear here. If the new request changes that work, say so and \
             it will be picked up after."
        ));
    }

    let id: Uuid = sqlx::query_scalar(
        "INSERT INTO agent_delegations (session_id, user_id, requesting_agent_type, target_agent_type, task) \
         VALUES ($1, $2, $3, $4, $5) RETURNING id",
    )
    .bind(session_id)
    .bind(user_id)
    .bind(requesting_agent_type)
    .bind(target_agent_type)
    .bind(task)
    .fetch_one(&mut **conn)
    .await
    .map_err(|e| format!("failed to create delegation: {e}"))?;

    let _ = sqlx::query("SELECT pg_notify('agent_delegations_channel', $1)")
        .bind(id.to_string())
        .execute(&mut **conn)
        .await;

    if let Some((publisher, _)) = mqtt {
        let _ = publisher.publish(session_id, &StreamEnvelope::AgentDelegationUpdated { delegation_id: id }).await;
    }

    Ok(format!(
        "Delegated to {target_agent_type}. Tell the user you'll follow up once it's done — do not wait for the result now."
    ))
}
