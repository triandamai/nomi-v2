//! A person's own topic, `user/{id}/stream`: their native apps keep it open all the time and
//! show new inbox items and chat replies in the island / bubble / Live Activity.

use std::sync::OnceLock;

use nomi_realtime::{MqttPublisher, UserEvent};
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
