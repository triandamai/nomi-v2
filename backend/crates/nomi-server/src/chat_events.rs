//! What a delete in a chat tells every app that might be showing it (web, Android, iOS, desktop):
//! the chat's own topic for devices that have it open, and the person's topic for their chat
//! lists. Best effort, like the other realtime announcements: a missed event is caught up the next
//! time the chat or the list loads.

use nomi_realtime::{user_topic, StreamEnvelope, UserEnvelope};
use uuid::Uuid;

fn chat_topic(session_id: Uuid) -> String {
    format!("chat/{session_id}/stream")
}

/// `message_ids` left the chat.
pub fn messages_deleted(session_id: Uuid, message_ids: Vec<Uuid>) {
    if message_ids.is_empty() {
        return;
    }
    let Some(publisher) = crate::project_events::publisher() else { return };
    if let Err(e) = publisher.try_publish_to(&chat_topic(session_id), &StreamEnvelope::MessagesDeleted { message_ids }) {
        tracing::debug!(error = %e, %session_id, "couldn't announce deleted messages");
    }
}

/// A message changed in place (its text was edited).
pub fn message_updated(session_id: Uuid, message_id: Uuid) {
    let Some(publisher) = crate::project_events::publisher() else { return };
    if let Err(e) = publisher.try_publish_to(&chat_topic(session_id), &StreamEnvelope::MessageUpdated { message_id }) {
        tracing::debug!(error = %e, %session_id, "couldn't announce an edited message");
    }
}

/// The chat is gone: devices showing it leave it, and the owner's devices drop it from their lists.
pub fn chat_deleted(owner_id: Uuid, session_id: Uuid) {
    let Some(publisher) = crate::project_events::publisher() else { return };
    if let Err(e) = publisher.try_publish_to(&chat_topic(session_id), &StreamEnvelope::SessionDeleted) {
        tracing::debug!(error = %e, %session_id, "couldn't announce a deleted chat");
    }
    if let Err(e) = publisher.try_publish_to(&user_topic(owner_id), &UserEnvelope::ChatDeleted { session_id }) {
        tracing::debug!(error = %e, %session_id, "couldn't tell the person's devices about a deleted chat");
    }
}
