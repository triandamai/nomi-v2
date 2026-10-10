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
    /// The turn ran past its time limit (a stalled model or tool) and was stopped.
    #[error("the turn took longer than {0:?} and was stopped")]
    TimedOut(std::time::Duration),
}

impl TurnError {
    /// The person's monthly allowance ran out (see nomi-server's quota).
    pub fn is_quota_exceeded(&self) -> bool {
        matches!(self, TurnError::LlmCallFailed(nomi_llm::LlmError::QuotaExceeded))
    }

    /// What to post with the explanation: a card when the allowance ran out, nothing otherwise.
    pub fn notice_blocks(&self) -> Option<serde_json::Value> {
        self.is_quota_exceeded().then(|| serde_json::json!([crate::content_block::ContentBlock::QuotaNotice { reason: "used_up".to_string() }]))
    }

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
            TurnError::TimedOut(_) => locale.t("error.timed_out"),
            TurnError::ApprovalNoLongerPending => locale.t("error.approval_handled"),
            TurnError::Db(_) => locale.t("error.internal"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_a_used_up_allowance_gets_the_quota_card() {
        let blocks = TurnError::LlmCallFailed(nomi_llm::LlmError::QuotaExceeded).notice_blocks().unwrap();
        assert_eq!(blocks, serde_json::json!([{"kind": "quota_notice", "reason": "used_up"}]));
        assert!(TurnError::ToolLoopExceeded.notice_blocks().is_none());
    }
}
