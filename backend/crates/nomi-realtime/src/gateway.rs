//! The one socket a native app keeps open: it asks for chat/project topics by name, always gets
//! its own user topic, and receives every event wrapped with the topic it came from.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Topic {
    User(Uuid),
    Chat(Uuid),
    Project(Uuid),
}

impl Topic {
    pub fn mqtt_topic(&self) -> String {
        match self {
            Topic::User(id) => format!("user/{id}/stream"),
            Topic::Chat(id) => format!("chat/{id}/stream"),
            Topic::Project(id) => format!("project/{id}/stream"),
        }
    }

    /// What the client sees. The user topic carries no id: a socket only ever has its own.
    pub fn wire(&self) -> String {
        match self {
            Topic::User(_) => "user".to_string(),
            Topic::Chat(id) => format!("chat:{id}"),
            Topic::Project(id) => format!("project:{id}"),
        }
    }

    /// Topics a client may ask for. The user topic is never asked for — it is implicit.
    pub fn parse_client(s: &str) -> Option<Topic> {
        let (kind, id) = s.split_once(':')?;
        let id = Uuid::parse_str(id).ok()?;
        match kind {
            "chat" => Some(Topic::Chat(id)),
            "project" => Some(Topic::Project(id)),
            _ => None,
        }
    }

    pub fn from_mqtt(s: &str) -> Option<Topic> {
        let mut parts = s.split('/');
        let (kind, id, tail) = (parts.next()?, parts.next()?, parts.next()?);
        if tail != "stream" || parts.next().is_some() {
            return None;
        }
        let id = Uuid::parse_str(id).ok()?;
        match kind {
            "user" => Some(Topic::User(id)),
            "chat" => Some(Topic::Chat(id)),
            "project" => Some(Topic::Project(id)),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum ClientFrame {
    Subscribe { topic: String },
    Unsubscribe { topic: String },
    Ping,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerFrame {
    Ready { user_id: Uuid },
    Subscribed { topic: String },
    Unsubscribed { topic: String },
    Denied { topic: String },
    Event { topic: String, payload: serde_json::Value },
    Error { message: String },
    Pong,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chat_topic_round_trips_between_wire_and_mqtt() {
        let id = Uuid::new_v4();
        let topic = Topic::parse_client(&format!("chat:{id}")).unwrap();
        assert_eq!(topic, Topic::Chat(id));
        assert_eq!(topic.mqtt_topic(), format!("chat/{id}/stream"));
        assert_eq!(Topic::from_mqtt(&topic.mqtt_topic()), Some(topic));
        assert_eq!(topic.wire(), format!("chat:{id}"));
    }

    #[test]
    fn client_cannot_name_the_user_topic() {
        assert_eq!(Topic::parse_client("user"), None);
        assert_eq!(Topic::parse_client(&format!("user:{}", Uuid::new_v4())), None);
    }

    #[test]
    fn garbage_topics_are_rejected() {
        assert_eq!(Topic::parse_client("chat:not-a-uuid"), None);
        assert_eq!(Topic::parse_client("chat"), None);
        assert_eq!(Topic::from_mqtt("chat/x/stream"), None);
    }

    #[test]
    fn user_topic_wire_is_just_user() {
        let id = Uuid::new_v4();
        assert_eq!(Topic::User(id).wire(), "user");
        assert_eq!(Topic::from_mqtt(&format!("user/{id}/stream")), Some(Topic::User(id)));
    }

    #[test]
    fn client_frames_parse() {
        let f: ClientFrame = serde_json::from_str(r#"{"op":"subscribe","topic":"chat:00000000-0000-0000-0000-000000000001"}"#).unwrap();
        assert!(matches!(f, ClientFrame::Subscribe { .. }));
        let f: ClientFrame = serde_json::from_str(r#"{"op":"ping"}"#).unwrap();
        assert_eq!(f, ClientFrame::Ping);
    }

    #[test]
    fn server_event_frame_serializes_with_type_tag() {
        let frame = ServerFrame::Event { topic: "user".into(), payload: serde_json::json!({"kind":"X"}) };
        assert_eq!(serde_json::to_value(&frame).unwrap(), serde_json::json!({"type":"event","topic":"user","payload":{"kind":"X"}}));
    }
}
