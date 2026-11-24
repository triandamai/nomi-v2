//! What a tool call runs in, and what a tool is told when it is offered.

use nomi_realtime::MqttPublisher;
use uuid::Uuid;

/// The call a tool runs for: which chat, which agent's turn, whose request. Tools ignore what
/// they don't need.
#[derive(Clone, Copy)]
pub struct ToolContext<'a> {
    pub session_id: Uuid,
    pub agent_session_id: Uuid,
    pub user_id: Uuid,
    /// The calling agent's `agent_type`.
    pub agent_type: &'a str,
    /// What the person calls the calling agent.
    pub agent_name: &'a str,
    /// Where live updates go, with the reply message they belong to, when there is a live chat.
    pub mqtt: Option<(&'a MqttPublisher, Uuid)>,
    /// The skills compiled into the calling agent (`SubAgent::builtin_skills`).
    pub builtin_skills: &'a [nomi_skills::BuiltinSkill],
}

/// What a tool may need to know to describe itself for this turn.
#[derive(Debug, Clone, Copy)]
pub struct OfferContext<'a> {
    /// The crew members a reminder can be addressed to.
    pub reminder_targets: &'a [String],
    /// The conversation holds an attached file.
    pub mentions_files: bool,
    /// The calling member's skills for this person, theirs first (`read_skill`'s index).
    pub skills: &'a [nomi_skills::SkillEntry],
}

/// Whether a call goes through the person's permission levels and the approval card.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Approval {
    /// Checked like an agent's own tool: allow / ask first / never.
    Checked,
    /// Runs straight away: it changes nothing the person would want to approve.
    Never,
}

/// Which agents get a tool without it being granted to them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutoOffer {
    /// Only its own agent, and custom agents it is granted to.
    GrantOnly,
    /// Every agent.
    Everyone,
    /// Agents that keep records (`SubAgent::uses_records`).
    IfUsesRecords,
    /// Agents that take reminders (`SubAgent::supports_reminders`).
    IfSupportsReminders,
}

impl<'a> ToolContext<'a> {
    /// A call with no live chat to publish to.
    pub fn new(session_id: Uuid, agent_session_id: Uuid, user_id: Uuid, agent_type: &'a str, agent_name: &'a str) -> Self {
        Self { session_id, agent_session_id, user_id, agent_type, agent_name, mqtt: None, builtin_skills: &[] }
    }
}
