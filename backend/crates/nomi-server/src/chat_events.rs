//! What a delete in a chat tells every app that might be showing it (web, Android, iOS, desktop):
//! the chat's own topic for devices that have it open, and the person's topic for all of their
//! devices, whichever chat they show. Best effort, like the other realtime announcements: a missed
//! event is caught up the next time the chat or the list loads.

use nomi_realtime::{user_topic, StreamEnvelope, UserEnvelope};
use uuid::Uuid;

fn chat_topic(session_id: Uuid) -> String {
    format!("chat/{session_id}/stream")
}

/// `message_ids` left the chat: on the chat's topic for devices showing it, and on `user_id`'s
/// topic for all of their devices.
pub fn messages_deleted(user_id: Uuid, session_id: Uuid, message_ids: Vec<Uuid>) {
    if message_ids.is_empty() {
        return;
    }
    let Some(publisher) = crate::project_events::publisher() else { return };
    if let Err(e) = publisher.try_publish_to(&chat_topic(session_id), &StreamEnvelope::MessagesDeleted { message_ids: message_ids.clone() }) {
        tracing::debug!(error = %e, %session_id, "couldn't announce deleted messages");
    }
    if let Err(e) = publisher.try_publish_to(&user_topic(user_id), &UserEnvelope::MessagesDeleted { session_id, message_ids }) {
        tracing::debug!(error = %e, %session_id, "couldn't tell the person's devices about deleted messages");
    }
}

/// A message changed in place (its text was edited), told the same way as a delete.
pub fn message_updated(user_id: Uuid, session_id: Uuid, message_id: Uuid) {
    let Some(publisher) = crate::project_events::publisher() else { return };
    if let Err(e) = publisher.try_publish_to(&chat_topic(session_id), &StreamEnvelope::MessageUpdated { message_id }) {
        tracing::debug!(error = %e, %session_id, "couldn't announce an edited message");
    }
    if let Err(e) = publisher.try_publish_to(&user_topic(user_id), &UserEnvelope::MessageUpdated { session_id, message_id }) {
        tracing::debug!(error = %e, %session_id, "couldn't tell the person's devices about an edited message");
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
