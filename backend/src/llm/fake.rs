use async_trait::async_trait;

use super::{response_to_stream, ContentBlock, LlmError, LlmEventStream, LlmProvider, LlmRequest, LlmResponse, StopReason};

/// Sending a message whose text contains this exact string makes FakeLlmProvider return an
/// error instead of its canned success reply — used by the frontend's e2e suite to exercise a
/// real backend turn failure (a 502 from POST /api/sessions/:id/messages) without needing
/// browser-level network mocking, which can't intercept this app's server-side backend calls.
/// Shared contract with frontend/e2e/conversation.e2e.ts — keep both in sync if this changes.
const SIMULATE_FAILURE_SENTINEL: &str = "__SIMULATE_TURN_FAILURE__";

pub struct FakeLlmProvider;

#[async_trait]
impl LlmProvider for FakeLlmProvider {
    async fn complete_stream(&self, request: LlmRequest) -> Result<LlmEventStream, LlmError> {
        let should_fail = request.messages.iter().any(|m| {
            m.content.iter().any(|block| match block {
                ContentBlock::Text { text } => text.contains(SIMULATE_FAILURE_SENTINEL),
                _ => false,
            })
        });

        if should_fail {
            return Err(LlmError::ProviderError("simulated failure for e2e testing".to_string()));
        }

        Ok(response_to_stream(LlmResponse {
            content: vec![ContentBlock::Text {
                text: "This is a fake response for local development and testing.".to_string(),
            }],
            stop_reason: StopReason::EndTurn,
            input_tokens: 0,
            output_tokens: 0,
        }))
    }
}
