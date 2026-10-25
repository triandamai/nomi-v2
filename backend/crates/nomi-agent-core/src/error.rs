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
    /// The same tool call failed the same way `attempts` times (see
    /// `SubAgent::repeated_failure_limit`); `output` is that failure.
    #[error("{tool} failed the same way {attempts} times")]
    RepeatedFailure { tool: String, attempts: u32, output: String },
    /// The turn ran past its time limit (a stalled model or tool) and was stopped.
    #[error("the turn took longer than {0:?} and was stopped")]
    TimedOut(std::time::Duration),
}

impl TurnError {
    /// The person's monthly allowance ran out (see nomi-server's quota).
    pub fn is_quota_exceeded(&self) -> bool {
        matches!(self, TurnError::LlmCallFailed(nomi_llm::LlmError::QuotaExceeded))
    }

    /// Whether running the same thing again can help: not when the allowance ran out (the card
    /// says where to go next) or when the approval it resumed was already handled.
    pub fn is_retryable(&self) -> bool {
        !self.is_quota_exceeded() && !matches!(self, TurnError::ApprovalNoLongerPending)
    }

    /// What to post with the explanation: a card when the allowance ran out, otherwise an error
    /// bubble that can rerun `retry` when trying again can help.
    pub fn notice_blocks(&self, retry: Option<crate::content_block::RetryTarget>) -> serde_json::Value {
        use crate::content_block::ContentBlock;
        let block = if self.is_quota_exceeded() {
            ContentBlock::QuotaNotice { reason: "used_up".to_string() }
        } else {
            ContentBlock::RunFailed { retry: retry.filter(|_| self.is_retryable()) }
        };
        serde_json::json!([block])
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
            TurnError::RepeatedFailure { tool, attempts, output } => {
                let output = output.trim();
                let clipped = match output.char_indices().nth(400) {
                    Some((at, _)) => format!("{}…", &output[..at]),
                    None => output.to_string(),
                };
                locale.tf("error.repeated_failure", &[("tool", tool), ("count", &attempts.to_string()), ("output", &clipped)])
            }
            TurnError::ApprovalNoLongerPending => locale.t("error.approval_handled"),
            TurnError::Db(_) => locale.t("error.internal"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content_block::RetryTarget;

    #[test]
    fn only_a_used_up_allowance_gets_the_quota_card() {
        let blocks = TurnError::LlmCallFailed(nomi_llm::LlmError::QuotaExceeded).notice_blocks(Some(RetryTarget::LastMessage));
        assert_eq!(blocks, serde_json::json!([{"kind": "quota_notice", "reason": "used_up"}]));
    }

    #[test]
    fn other_failures_get_an_error_bubble_that_can_retry() {
        let turn_job_id = uuid::Uuid::nil();
        let blocks = TurnError::ToolLoopExceeded.notice_blocks(Some(RetryTarget::Turn { turn_job_id }));
        assert_eq!(blocks, serde_json::json!([{"kind": "run_failed", "retry": {"target": "turn", "turn_job_id": turn_job_id}}]));
        let blocks = TurnError::ApprovalNoLongerPending.notice_blocks(Some(RetryTarget::LastMessage));
        assert_eq!(blocks, serde_json::json!([{"kind": "run_failed"}]));
    }
}
