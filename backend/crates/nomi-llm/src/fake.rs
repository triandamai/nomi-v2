use async_trait::async_trait;

use super::{response_to_stream, ContentBlock, LlmError, LlmEventStream, LlmProvider, LlmRequest, LlmResponse, StopReason};

/// Sending a message whose text contains this exact string makes FakeLlmProvider return an
/// error instead of its canned success reply. Used to exercise turn-failure handling directly
/// in `turn::process_turn` / `handle_inbound_message` (see
/// `process_turn_records_a_turn_failed_event_on_llm_failure` in `turn_process.rs`). Note that
/// `POST /api/sessions/:id/messages` is ingest-only and always returns `202` regardless of this
/// sentinel — a turn failure now happens asynchronously in the worker, well after the HTTP
/// response has already gone out, so this no longer produces an observable HTTP-level failure.
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

        let mut content = Vec::new();
        if request.enable_reasoning {
            content.push(ContentBlock::Thinking {
                text: "Thinking through a fake response for local development and testing.".to_string(),
                signature: None,
            });
        }
        content.push(ContentBlock::Text {
            text: "This is a fake response for local development and testing.".to_string(),
        });

        Ok(response_to_stream(LlmResponse {
            content,
            stop_reason: StopReason::EndTurn,
            input_tokens: 0,
            output_tokens: 0,
        }))
    }
}
