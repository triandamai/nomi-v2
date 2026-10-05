#[derive(Debug, thiserror::Error)]
pub enum TurnError {
    #[error(transparent)]
    Db(#[from] sqlx::Error),
    #[error("llm call failed: {0}")]
    LlmCallFailed(#[from] nomi_llm::LlmError),
    #[error("tool-calling loop exceeded its turn limit without completing")]
    ToolLoopExceeded,
    #[error("this action is no longer pending approval")]
    ApprovalNoLongerPending,
}

impl TurnError {
    /// What went wrong, for the person in the chat: plain language and what they can do. The
    /// raw error stays in the logs and `agent_events`.
    pub fn user_message(&self) -> String {
        match self {
            TurnError::LlmCallFailed(e) => e.user_message(),
            TurnError::ToolLoopExceeded => "I went around in circles on that one and stopped. Try asking in a different way.".to_string(),
            TurnError::ApprovalNoLongerPending => "That approval was already handled.".to_string(),
            TurnError::Db(_) => "Something went wrong on my side. Send your message again.".to_string(),
        }
    }
}
