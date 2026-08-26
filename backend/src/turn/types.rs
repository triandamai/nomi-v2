use uuid::Uuid;

#[derive(Debug, Clone, PartialEq)]
pub struct TurnOutcome {
    pub session_id: Uuid,
    pub reply: String,
}
