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
