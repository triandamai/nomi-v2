//! A person's own topic, `user/{id}/stream`: their native apps keep it open all the time and
//! show new inbox items and chat replies in the island / bubble / Live Activity.

use std::sync::OnceLock;

use nomi_realtime::{MqttPublisher, UserEvent};
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

pub const CHAT_REPLY_PREVIEW_CHARS: usize = 140;

static PUBLISHER: OnceLock<MqttPublisher> = OnceLock::new();

/// Connects the publisher; until this runs (in tests, say) nothing is sent.
pub fn init(broker_host: &str, broker_port: u16) {
    let _ = PUBLISHER.get_or_init(|| MqttPublisher::connect(broker_host, broker_port, &format!("nomi-user-events-{}", Uuid::new_v4())));
}

pub fn topic(user_id: Uuid) -> String {
    format!("user/{user_id}/stream")
}

/// The first `CHAT_REPLY_PREVIEW_CHARS` characters of a reply, cut on a character boundary.
pub fn preview(text: &str) -> String {
    text.chars().take(CHAT_REPLY_PREVIEW_CHARS).collect()
}

pub fn publish(user_id: Uuid, event: &UserEvent) {
    let Some(publisher) = PUBLISHER.get() else { return };
    if let Err(e) = publisher.try_publish_to(&topic(user_id), event) {
        tracing::debug!(error = %e, %user_id, "couldn't tell the person's apps about an event");
    }
}

/// Tells the person a chat replied: on their user topic (through `mqtt`, the worker's own
/// publisher) for apps with a socket open, and by push for phones: an alert titled after the chat,
/// and an update to the iPhone's Live Activity.
pub fn announce_chat_reply(pool: &PgPool, mqtt: &MqttPublisher, user_id: Uuid, session_id: Uuid, message_id: Uuid, reply: &str) {
    let preview = preview(reply);
    if let Err(e) = mqtt.try_publish_to(&topic(user_id), &UserEvent::ChatReply { session_id, message_id, preview: preview.clone() }) {
        tracing::debug!(error = %e, %user_id, "couldn't tell the person's apps about a chat reply");
    }
    let sender = crate::push::sender();
    if !sender.is_enabled() {
        return;
    }
    let pool = pool.clone();
    tokio::spawn(async move {
        let title: Option<String> =
            sqlx::query_scalar("SELECT title FROM sessions WHERE id = $1").bind(session_id).fetch_optional(&pool).await.ok().flatten().flatten();
        let alert = crate::push::PushMessage::chat_reply(title, session_id, message_id, &preview);
        crate::push::dispatch(pool.clone(), sender.clone(), user_id, alert).await;
        let activity = crate::push::PushMessage::live_activity_update(json!({ "phase": "replied", "preview": preview }));
        crate::push::dispatch(pool, sender, user_id, activity).await;
    });
}
