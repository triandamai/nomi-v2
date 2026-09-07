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
}
