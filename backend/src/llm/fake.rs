use async_trait::async_trait;

use super::{ContentBlock, LlmError, LlmProvider, LlmRequest, LlmResponse, StopReason};

pub struct FakeLlmProvider;

#[async_trait]
impl LlmProvider for FakeLlmProvider {
    async fn complete(&self, _request: LlmRequest) -> Result<LlmResponse, LlmError> {
        Ok(LlmResponse {
            content: vec![ContentBlock::Text {
                text: "This is a fake response for local development and testing.".to_string(),
            }],
            stop_reason: StopReason::EndTurn,
            input_tokens: 0,
            output_tokens: 0,
        })
    }
}
