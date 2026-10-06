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
        self.user_message_in(crate::Locale::En)
    }

    /// [`TurnError::user_message`] in the person's language.
    pub fn user_message_in(&self, locale: crate::Locale) -> String {
        match self {
            TurnError::LlmCallFailed(e) => e.user_message_in(locale),
            TurnError::ToolLoopExceeded => locale.t("error.tool_loop"),
            TurnError::ApprovalNoLongerPending => locale.t("error.approval_handled"),
            TurnError::Db(_) => locale.t("error.internal"),
        }
    }
}
