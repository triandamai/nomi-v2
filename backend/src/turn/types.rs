use uuid::Uuid;

use crate::llm::LlmError;

#[derive(Debug, Clone, PartialEq)]
pub struct TurnOutcome {
    pub session_id: Uuid,
    pub reply: String,
}

#[derive(Debug, thiserror::Error)]
pub enum TurnError {
    #[error(transparent)]
    Db(#[from] sqlx::Error),
    #[error("llm call failed: {0}")]
    LlmCallFailed(#[from] LlmError),
}
