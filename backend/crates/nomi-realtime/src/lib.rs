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
    /// A generic "something new landed in this session's messages, refetch" signal — used for
    /// per-tool-call/thought activity messages (see SubAgent::surfaces_activity), which have no
    /// delegation record of their own to key off of the way AgentDelegationUpdated does.
    SessionActivity { session_id: Uuid },
}
