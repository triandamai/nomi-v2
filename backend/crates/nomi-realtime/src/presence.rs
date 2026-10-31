//! Which of a person's devices are on, in MQTT: every presence connection (a native app's realtime
//! socket, each browser tab's activity socket) publishes its state on
//! `presence/{user_id}/{client_id}`, and leaves an `offline` last will the broker sends if it
//! drops without saying goodbye. The MQTT client id is the device id the app keeps for good.

use rumqttc::{LastWill, QoS};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Every presence topic, for the server that keeps the device list.
pub const PRESENCE_FILTER: &str = "presence/+/+";
/// The longest tab id a browser may add to its device id.
const MAX_TAB_ID_LEN: usize = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PresenceState {
    /// Connected, with the app in front.
    Online,
    /// Connected, with the app in the background (or the tab hidden).
    Away,
    Offline,
}

impl PresenceState {
    pub fn as_str(self) -> &'static str {
        match self {
            PresenceState::Online => "online",
            PresenceState::Away => "away",
            PresenceState::Offline => "offline",
        }
    }
}

/// What a presence connection publishes about itself.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PresenceUpdate {
    pub device_id: Uuid,
    /// This connection, of the ones that have had its client id: a reconnect replaces the old
    /// connection, whose `offline` may arrive after the new one's `online`.
    pub connection: Uuid,
    pub state: PresenceState,
}

pub fn presence_topic(user_id: Uuid, client_id: &str) -> String {
    format!("presence/{user_id}/{client_id}")
}

/// The person and client id of a presence topic.
pub fn parse_presence_topic(topic: &str) -> Option<(Uuid, &str)> {
    let rest = topic.strip_prefix("presence/")?;
    let (user, client_id) = rest.split_once('/')?;
    if client_id.is_empty() || client_id.contains('/') {
        return None;
    }
    Some((Uuid::parse_str(user).ok()?, client_id))
}

/// The broker's word for a presence connection that dropped: `offline`.
pub fn offline_will(user_id: Uuid, client_id: &str, device_id: Uuid, connection: Uuid) -> LastWill {
    let payload = serde_json::to_vec(&PresenceUpdate { device_id, connection, state: PresenceState::Offline }).expect("presence always serializes");
    LastWill::new(presence_topic(user_id, client_id), payload, QoS::AtLeastOnce, false)
}

/// The MQTT client id of one of a device's connections: the device id itself, or with a suffix
/// when a device holds several at once (browser tabs, a chat's own relay). The broker allows one
/// connection per client id, so a reconnect replaces a stale one.
pub fn device_client_id(device_id: Uuid, suffix: Option<&str>) -> String {
    match suffix {
        Some(suffix) => format!("{device_id}-{suffix}"),
        None => device_id.to_string(),
    }
}

/// A tab id a browser may send: short, letters, digits and dashes.
pub fn valid_tab_id(tab_id: &str) -> bool {
    !tab_id.is_empty() && tab_id.len() <= MAX_TAB_ID_LEN && tab_id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_device_connection_is_named_after_the_device() {
        let device = Uuid::nil();
        assert_eq!(device_client_id(device, None), device.to_string());
        assert_eq!(device_client_id(device, Some("t1")), format!("{device}-t1"));
    }

    #[test]
    fn presence_topics_round_trip() {
        let user = Uuid::new_v4();
        let topic = presence_topic(user, "abc-t1");
        assert_eq!(parse_presence_topic(&topic), Some((user, "abc-t1")));
        assert_eq!(parse_presence_topic("presence/not-a-uuid/x"), None);
        assert_eq!(parse_presence_topic(&format!("presence/{user}/")), None);
        assert_eq!(parse_presence_topic(&format!("presence/{user}/a/b")), None);
    }

    #[test]
    fn presence_goes_out_in_the_shape_the_server_reads() {
        let update = PresenceUpdate { device_id: Uuid::nil(), connection: Uuid::nil(), state: PresenceState::Away };
        assert_eq!(serde_json::to_value(&update).unwrap(), serde_json::json!({ "device_id": Uuid::nil(), "connection": Uuid::nil(), "state": "away" }));
    }

    #[test]
    fn tab_ids_are_short_and_plain() {
        assert!(valid_tab_id("k3j-9x"));
        assert!(!valid_tab_id(""));
        assert!(!valid_tab_id("a/b"));
        assert!(!valid_tab_id(&"x".repeat(MAX_TAB_ID_LEN + 1)));
    }
}
