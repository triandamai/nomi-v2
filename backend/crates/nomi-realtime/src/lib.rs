pub mod mqtt;

pub use mqtt::{MqttError, MqttPublisher};

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
