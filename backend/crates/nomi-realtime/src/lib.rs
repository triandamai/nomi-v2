pub mod gateway;
pub mod mqtt;

pub use gateway::{ClientFrame, ServerFrame, Topic};
pub use mqtt::{user_topic, MqttError, MqttPublisher};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use nomi_llm::StreamEvent;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum StreamEnvelope {
    Delta { turn_job_id: Uuid, event: StreamEvent },
    TurnCompleted { turn_job_id: Uuid, message_id: Uuid },
    TurnFailed { turn_job_id: Uuid, error: String },
    AgentDelegationUpdated { delegation_id: Uuid },
    /// A brand-new message landed — the frontend fetches it and appends it to the conversation
    /// instead of refetching the whole thing.
    MessageCreated { message_id: Uuid },
    /// The chat got a new name (the person renamed it, or asked the crew to).
    SessionRenamed { title: String },
    /// The personality this chat replies in changed (set, reset or rolled back): the page reloads it.
    PersonalityChanged,
    /// An existing message's content_blocks changed in place (a todo list step flipped, an
    /// approval was decided) — the frontend fetches it and replaces its existing entry.
    MessageUpdated { message_id: Uuid },
    /// An agent's live status changed — thinking, calling a tool, writing a reply, or back to
    /// waiting. `detail` carries the tool name for `calling_tool`, `None` otherwise. Never sent
    /// for the default (chitchat) agent's turns, which have no real `agent_sessions` row.
    AgentPhaseChanged { agent_session_id: Uuid, phase: String, detail: Option<String> },
    /// An `agent_sessions` row was created — an agent started working. Every field the admin
    /// command center needs to render a new table row is resolved once here (at spawn time,
    /// mirroring how `messages.agent_display_name` is resolved at insert time) so the dashboard
    /// never needs a follow-up lookup per event.
    AgentSessionStarted {
        agent_session_id: Uuid,
        session_id: Uuid,
        agent_type: String,
        agent_display_name: String,
        channel: String,
        sender_label: String,
    },
    /// An `agent_sessions` row was closed. `reason` is one of "completed" | "cancelled" |
    /// "expired", matching `agent_sessions.status`'s possible non-active values exactly.
    AgentSessionEnded { agent_session_id: Uuid, session_id: Uuid, reason: String },
}

/// Events on `user/{id}/stream`: what a person's apps hear about that isn't tied to a chat they
/// have open — the inbox, and replies that landed in any of their chats.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum UserEvent {
    NotificationCreated {
        notification_id: Uuid,
        /// The inbox category (`quota`, `deploy`, ...). On the wire it is `category`, because `kind` is the event tag.
        #[serde(rename = "category")]
        kind: String,
        title: String,
        body: String,
        link: Option<String>,
    },
    ChatReply { session_id: Uuid, message_id: Uuid, preview: String },
}

/// What goes to everything a person has open (`user/{user_id}/activity`), whichever chat they're
/// looking at: the app subscribes once while it's open (see `user_topic`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum UserEnvelope {
    /// The crew is done with a chat for now: it finished, failed, or is waiting on the person
    /// (`waiting`: "question", "approval" or "connect"). `project_id` is set for a project's chat.
    ChatActivity {
        session_id: Uuid,
        title: Option<String>,
        project_id: Option<Uuid>,
        /// "done", "failed" or "waiting".
        outcome: String,
        waiting: Option<String>,
        /// Who answered last.
        agent: Option<String>,
    },
}
