//! Realtime payloads decoded into [`RealtimeEvent`], as the backend sends them
//! (`backend/crates/nomi-realtime`: `StreamEnvelope`, `UserEvent`, `UserEnvelope`; project topics
//! carry `DeployChanged`). Chat and project ids come from the topic; payloads don't repeat them.

use serde::Deserialize;
use serde_json::Value;

const CHAT_PREFIX: &str = "chat:";
const PROJECT_PREFIX: &str = "project:";
const USER_TOPIC: &str = "user";
/// The stream event that appends text to a reply: `{"TextDelta":{"index":n,"text":"..."}}`.
const TEXT_DELTA: &str = "TextDelta";
const TEXT: &str = "text";
const KIND: &str = "kind";

/// One realtime event, typed. Anything this SDK doesn't know, or that is malformed, is `Unknown`.
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Enum)]
pub enum RealtimeEvent {
    // Chat topic (`chat:<id>`)
    /// A piece of the reply streaming in. `text` is what it appends, when it appends text;
    /// `event_json` is the stream event as sent.
    ChatDelta {
        session_id: String,
        turn_job_id: String,
        text: Option<String>,
        event_json: String,
    },
    TurnCompleted {
        session_id: String,
        turn_job_id: String,
        message_id: String,
    },
    TurnFailed {
        session_id: String,
        turn_job_id: String,
        error: String,
    },
    MessageCreated {
        session_id: String,
        message_id: String,
    },
    /// An existing message changed in place (a step flipped, an approval was decided).
    MessageUpdated {
        session_id: String,
        message_id: String,
    },
    SessionRenamed {
        session_id: String,
        title: String,
    },
    PersonalityChanged {
        session_id: String,
    },
    AgentDelegationUpdated {
        session_id: String,
        delegation_id: String,
    },
    /// `phase` is `thinking`, `calling_tool`, `writing` or `waiting`; `detail` names the tool.
    AgentPhaseChanged {
        session_id: String,
        agent_session_id: String,
        phase: String,
        detail: Option<String>,
    },
    AgentSessionStarted {
        session_id: String,
        agent_session_id: String,
        agent_type: String,
        agent_display_name: String,
        channel: String,
        sender_label: String,
    },
    /// `reason` is `completed`, `cancelled` or `expired`.
    AgentSessionEnded {
        session_id: String,
        agent_session_id: String,
        reason: String,
    },

    // Project topic (`project:<id>`)
    DeployChanged {
        project_id: String,
        event: String,
    },

    // User topic (`user`)
    NotificationCreated {
        notification_id: String,
        category: String,
        title: String,
        body: String,
        link: Option<String>,
    },
    /// A reply landed in one of the person's chats.
    ChatReply {
        session_id: String,
        message_id: String,
        preview: String,
    },
    /// The crew is done with a chat for now. `outcome` is `done`, `failed` or `waiting`;
    /// `waiting` is `question`, `approval` or `connect` when it waits on the person.
    ChatActivity {
        session_id: String,
        title: Option<String>,
        project_id: Option<String>,
        outcome: String,
        waiting: Option<String>,
        agent: Option<String>,
    },
    /// Something was said in the chat from one of the person's devices.
    ChatUpdated {
        session_id: String,
        project_id: Option<String>,
        title: Option<String>,
    },

    Unknown {
        kind: Option<String>,
        payload_json: String,
    },
}

#[derive(Deserialize)]
#[serde(tag = "kind")]
enum ChatWire {
    Delta {
        turn_job_id: String,
        event: Value,
    },
    TurnCompleted {
        turn_job_id: String,
        message_id: String,
    },
    TurnFailed {
        turn_job_id: String,
        error: String,
    },
    MessageCreated {
        message_id: String,
    },
    MessageUpdated {
        message_id: String,
    },
    SessionRenamed {
        title: String,
    },
    PersonalityChanged,
    AgentDelegationUpdated {
        delegation_id: String,
    },
    AgentPhaseChanged {
        agent_session_id: String,
        phase: String,
        detail: Option<String>,
    },
    AgentSessionStarted {
        agent_session_id: String,
        agent_type: String,
        agent_display_name: String,
        channel: String,
        sender_label: String,
    },
    AgentSessionEnded {
        agent_session_id: String,
        reason: String,
    },
}

#[derive(Deserialize)]
#[serde(tag = "kind")]
enum UserWire {
    NotificationCreated {
        notification_id: String,
        category: String,
        title: String,
        body: String,
        link: Option<String>,
    },
    ChatReply {
        session_id: String,
        message_id: String,
        preview: String,
    },
    ChatActivity {
        session_id: String,
        title: Option<String>,
        project_id: Option<String>,
        outcome: String,
        waiting: Option<String>,
        agent: Option<String>,
    },
    ChatUpdated {
        session_id: String,
        project_id: Option<String>,
        title: Option<String>,
    },
}

#[derive(Deserialize)]
#[serde(tag = "kind")]
enum ProjectWire {
    DeployChanged { event: String },
}

/// The chat id of a `chat:<id>` topic.
pub(crate) fn chat_of(topic: &str) -> Option<&str> {
    topic.strip_prefix(CHAT_PREFIX).filter(|id| !id.is_empty())
}

/// Decodes the payload of an `event` frame on `topic`. Never fails: whatever doesn't decode is
/// [`RealtimeEvent::Unknown`].
pub(crate) fn decode(topic: &str, payload: &Value) -> RealtimeEvent {
    decode_known(topic, payload).unwrap_or_else(|| RealtimeEvent::Unknown {
        kind: payload.get(KIND).and_then(Value::as_str).map(str::to_owned),
        payload_json: payload.to_string(),
    })
}

fn decode_known(topic: &str, payload: &Value) -> Option<RealtimeEvent> {
    if let Some(session_id) = chat_of(topic) {
        let session_id = session_id.to_owned();
        return Some(match ChatWire::deserialize(payload).ok()? {
            ChatWire::Delta { turn_job_id, event } => RealtimeEvent::ChatDelta {
                session_id,
                turn_job_id,
                text: event
                    .get(TEXT_DELTA)
                    .and_then(|delta| delta.get(TEXT))
                    .and_then(Value::as_str)
                    .map(str::to_owned),
                event_json: event.to_string(),
            },
            ChatWire::TurnCompleted {
                turn_job_id,
                message_id,
            } => RealtimeEvent::TurnCompleted {
                session_id,
                turn_job_id,
                message_id,
            },
            ChatWire::TurnFailed { turn_job_id, error } => RealtimeEvent::TurnFailed {
                session_id,
                turn_job_id,
                error,
            },
            ChatWire::MessageCreated { message_id } => RealtimeEvent::MessageCreated {
                session_id,
                message_id,
            },
            ChatWire::MessageUpdated { message_id } => RealtimeEvent::MessageUpdated {
                session_id,
                message_id,
            },
            ChatWire::SessionRenamed { title } => {
                RealtimeEvent::SessionRenamed { session_id, title }
            }
            ChatWire::PersonalityChanged => RealtimeEvent::PersonalityChanged { session_id },
            ChatWire::AgentDelegationUpdated { delegation_id } => {
                RealtimeEvent::AgentDelegationUpdated {
                    session_id,
                    delegation_id,
                }
            }
            ChatWire::AgentPhaseChanged {
                agent_session_id,
                phase,
                detail,
            } => RealtimeEvent::AgentPhaseChanged {
                session_id,
                agent_session_id,
                phase,
                detail,
            },
            // The payload repeats the chat id; the topic's is the one subscribed to.
            ChatWire::AgentSessionStarted {
                agent_session_id,
                agent_type,
                agent_display_name,
                channel,
                sender_label,
            } => RealtimeEvent::AgentSessionStarted {
                session_id,
                agent_session_id,
                agent_type,
                agent_display_name,
                channel,
                sender_label,
            },
            ChatWire::AgentSessionEnded {
                agent_session_id,
                reason,
            } => RealtimeEvent::AgentSessionEnded {
                session_id,
                agent_session_id,
                reason,
            },
        });
    }
    if let Some(project_id) = topic
        .strip_prefix(PROJECT_PREFIX)
        .filter(|id| !id.is_empty())
    {
        let ProjectWire::DeployChanged { event } = ProjectWire::deserialize(payload).ok()?;
        return Some(RealtimeEvent::DeployChanged {
            project_id: project_id.to_owned(),
            event,
        });
    }
    if topic != USER_TOPIC {
        return None;
    }
    Some(match UserWire::deserialize(payload).ok()? {
        UserWire::NotificationCreated {
            notification_id,
            category,
            title,
            body,
            link,
        } => RealtimeEvent::NotificationCreated {
            notification_id,
            category,
            title,
            body,
            link,
        },
        UserWire::ChatReply {
            session_id,
            message_id,
            preview,
        } => RealtimeEvent::ChatReply {
            session_id: session_id.to_lowercase(),
            message_id,
            preview,
        },
        UserWire::ChatActivity {
            session_id,
            title,
            project_id,
            outcome,
            waiting,
            agent,
        } => RealtimeEvent::ChatActivity {
            session_id: session_id.to_lowercase(),
            title,
            project_id,
            outcome,
            waiting,
            agent,
        },
        UserWire::ChatUpdated {
            session_id,
            project_id,
            title,
        } => RealtimeEvent::ChatUpdated {
            session_id: session_id.to_lowercase(),
            project_id,
            title,
        },
    })
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    const CHAT: &str = "chat:5e1f";
    const SESSION: &str = "5e1f";

    #[test]
    fn decodes_a_text_delta_and_keeps_the_raw_event() {
        let event = decode(
            CHAT,
            &json!({ "kind": "Delta", "turn_job_id": "t1", "event": { "TextDelta": { "index": 0, "text": "Hi" } } }),
        );
        assert_eq!(
            event,
            RealtimeEvent::ChatDelta {
                session_id: SESSION.into(),
                turn_job_id: "t1".into(),
                text: Some("Hi".into()),
                event_json: r#"{"TextDelta":{"index":0,"text":"Hi"}}"#.into(),
            }
        );
    }

    #[test]
    fn a_delta_without_text_has_none() {
        let event = decode(
            CHAT,
            &json!({ "kind": "Delta", "turn_job_id": "t1", "event": { "ToolStarted": {} } }),
        );
        assert!(matches!(event, RealtimeEvent::ChatDelta { text: None, .. }));
    }

    #[test]
    fn takes_chat_ids_from_the_topic() {
        assert_eq!(
            decode(
                CHAT,
                &json!({ "kind": "TurnCompleted", "turn_job_id": "t1", "message_id": "m1" })
            ),
            RealtimeEvent::TurnCompleted {
                session_id: SESSION.into(),
                turn_job_id: "t1".into(),
                message_id: "m1".into(),
            }
        );
        assert_eq!(
            decode(CHAT, &json!({ "kind": "PersonalityChanged" })),
            RealtimeEvent::PersonalityChanged {
                session_id: SESSION.into()
            }
        );
        assert_eq!(
            decode(
                CHAT,
                &json!({ "kind": "AgentPhaseChanged", "agent_session_id": "a1", "phase": "writing", "detail": null })
            ),
            RealtimeEvent::AgentPhaseChanged {
                session_id: SESSION.into(),
                agent_session_id: "a1".into(),
                phase: "writing".into(),
                detail: None,
            }
        );
    }

    #[test]
    fn decodes_user_events() {
        assert_eq!(
            decode(
                USER_TOPIC,
                &json!({ "kind": "NotificationCreated", "notification_id": "n1", "category": "deploy", "title": "T", "body": "B", "link": null })
            ),
            RealtimeEvent::NotificationCreated {
                notification_id: "n1".into(),
                category: "deploy".into(),
                title: "T".into(),
                body: "B".into(),
                link: None,
            }
        );
        assert_eq!(
            decode(
                USER_TOPIC,
                &json!({ "kind": "ChatReply", "session_id": "5E1F", "message_id": "m1", "preview": "Done" })
            ),
            RealtimeEvent::ChatReply {
                session_id: SESSION.into(),
                message_id: "m1".into(),
                preview: "Done".into(),
            }
        );
        assert_eq!(
            decode(
                USER_TOPIC,
                &json!({ "kind": "ChatActivity", "session_id": SESSION, "title": null, "project_id": null, "outcome": "waiting", "waiting": "approval", "agent": "coder" })
            ),
            RealtimeEvent::ChatActivity {
                session_id: SESSION.into(),
                title: None,
                project_id: None,
                outcome: "waiting".into(),
                waiting: Some("approval".into()),
                agent: Some("coder".into()),
            }
        );
    }

    #[test]
    fn decodes_project_events() {
        assert_eq!(
            decode(
                "project:p1",
                &json!({ "kind": "DeployChanged", "event": "live" })
            ),
            RealtimeEvent::DeployChanged {
                project_id: "p1".into(),
                event: "live".into(),
            }
        );
    }

    #[test]
    fn unknown_malformed_or_misplaced_payloads_are_unknown() {
        let unknown = json!({ "kind": "SomethingNew", "x": 1 });
        assert_eq!(
            decode(CHAT, &unknown),
            RealtimeEvent::Unknown {
                kind: Some("SomethingNew".into()),
                payload_json: unknown.to_string(),
            }
        );
        // A required field is missing.
        assert!(matches!(
            decode(
                CHAT,
                &json!({ "kind": "TurnCompleted", "turn_job_id": "t1" })
            ),
            RealtimeEvent::Unknown { .. }
        ));
        // A chat event on the user topic.
        assert!(matches!(
            decode(
                USER_TOPIC,
                &json!({ "kind": "MessageCreated", "message_id": "m1" })
            ),
            RealtimeEvent::Unknown { .. }
        ));
        assert!(matches!(
            decode(CHAT, &json!("not an object")),
            RealtimeEvent::Unknown { kind: None, .. }
        ));
    }
}
