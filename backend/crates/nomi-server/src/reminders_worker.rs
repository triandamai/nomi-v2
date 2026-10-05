//! Posts due reminders into their chats (nomi_agent_reminders::fire_due) and announces each new
//! message over MQTT. No model call: a reminder is the user's own words coming back on time.

use std::time::Duration;

use sqlx::PgPool;

use nomi_realtime::{MqttPublisher, StreamEnvelope};

const POLL_INTERVAL: Duration = Duration::from_secs(15);
const BATCH: i64 = 50;

pub async fn run(pool: PgPool, mqtt: MqttPublisher) {
    tracing::info!("reminders worker: started");
    loop {
        match nomi_agent_reminders::fire_due(&pool, BATCH).await {
            Ok(posted) => {
                for (session_id, message_id) in posted {
                    let _ = mqtt.publish(session_id, &StreamEnvelope::MessageCreated { message_id }).await;
                    tracing::info!(%session_id, %message_id, "reminders worker: reminder posted");
                }
            }
            Err(e) => tracing::error!(error = %e, "reminders worker: failed to fire due reminders"),
        }
        tokio::time::sleep(POLL_INTERVAL).await;
    }
}
