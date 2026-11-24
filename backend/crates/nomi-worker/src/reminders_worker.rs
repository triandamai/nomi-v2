//! Posts due reminders into their chats (nomi_agent_reminders::fire_due), announces each new
//! message over MQTT, and pushes it to the person's phones unless they turned reminder pushes off
//! (`push_reminders`). No model call: a reminder is the user's own words coming back on time.
//! Quiet hours are kept by `fire_due`, which holds a reminder until they end.

use std::sync::Arc;
use std::time::Duration;

use sqlx::PgPool;

use nomi_agent_reminders::Fired;
use nomi_realtime::{MqttPublisher, StreamEnvelope};

use nomi_server::push::{PushMessage, PushSender};

const POLL_INTERVAL: Duration = Duration::from_secs(15);
const BATCH: i64 = 50;

/// Whether the person wants reminders pushed to their phones (on until they turn it off).
pub async fn wants_reminder_pushes(pool: &PgPool, user_id: uuid::Uuid) -> bool {
    sqlx::query_scalar::<_, bool>("SELECT push_reminders FROM user_preferences WHERE user_id = $1")
        .bind(user_id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()
        .unwrap_or(true)
}

/// Pushes a reminder that went off to its person's phones, when they allow reminder pushes.
pub async fn push_reminder(pool: &PgPool, sender: Arc<dyn PushSender>, fired: &Fired) {
    if !sender.is_enabled() || !wants_reminder_pushes(pool, fired.user_id).await {
        return;
    }
    let message = PushMessage::reminder(
        &fired.title,
        fired.notes.as_deref(),
        fired.session_id,
        fired.message_id,
        fired.reminder_id,
    );
    nomi_server::push::dispatch(pool.clone(), sender, fired.user_id, message).await;
}

pub async fn run(pool: PgPool, mqtt: MqttPublisher) {
    tracing::info!("reminders worker: started");
    loop {
        match nomi_agent_reminders::fire_due(&pool, BATCH).await {
            Ok(posted) => {
                for fired in posted {
                    let _ = mqtt
                        .publish(
                            fired.session_id,
                            &StreamEnvelope::MessageCreated {
                                message_id: fired.message_id,
                            },
                        )
                        .await;
                    tracing::info!(session_id = %fired.session_id, message_id = %fired.message_id, "reminders worker: reminder posted");
                    let pool = pool.clone();
                    tokio::spawn(async move {
                        push_reminder(&pool, nomi_server::push::sender(), &fired).await
                    });
                }
            }
            Err(e) => tracing::error!(error = %e, "reminders worker: failed to fire due reminders"),
        }
        tokio::time::sleep(POLL_INTERVAL).await;
    }
}
