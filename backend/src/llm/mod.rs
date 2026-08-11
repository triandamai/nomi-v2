pub mod types;
pub mod anthropic;
pub mod openai;
pub mod gemini;
pub mod config;
pub mod fake;

pub use types::*;
pub use config::{build_provider, ModelConfig, ProviderKind};

use async_trait::async_trait;
use futures_util::StreamExt;
use std::collections::BTreeMap;

#[async_trait]
pub trait LlmProvider: Send + Sync {
    async fn complete(&self, request: LlmRequest) -> Result<LlmResponse, LlmError>;

    async fn complete_stream(&self, request: LlmRequest) -> Result<LlmEventStream, LlmError> {
        let response = self.complete(request).await?;
        Ok(response_to_stream(response))
    }
}

/// Wraps a complete (non-streaming) LlmResponse as a single-shot LlmEventStream — the trait's
/// default complete_stream body, and reused directly by fake providers that have no real
/// network round trip to chunk.
pub fn response_to_stream(response: LlmResponse) -> LlmEventStream {
    let mut events: Vec<Result<StreamEvent, LlmError>> = Vec::new();
    for (index, block) in response.content.into_iter().enumerate() {
        match block {
            ContentBlock::Text { text } => {
                events.push(Ok(StreamEvent::ContentBlockStart { index, block: PartialBlock::Text }));
                events.push(Ok(StreamEvent::TextDelta { index, text }));
                events.push(Ok(StreamEvent::ContentBlockDone { index }));
            }
            ContentBlock::ToolUse { id, name, input } => {
                events.push(Ok(StreamEvent::ContentBlockStart { index, block: PartialBlock::ToolUse { id, name } }));
                events.push(Ok(StreamEvent::ToolInputDelta { index, partial_json: input.to_string() }));
                events.push(Ok(StreamEvent::ContentBlockDone { index }));
            }
            ContentBlock::ToolResult { .. } => {
                // A provider's own response never contains a ToolResult block (that's only ever
                // something we send as part of a request) — nothing to emit.
            }
        }
    }
    events.push(Ok(StreamEvent::Done {
        stop_reason: response.stop_reason,
        input_tokens: response.input_tokens,
        output_tokens: response.output_tokens,
    }));
    Box::pin(futures_util::stream::iter(events))
}

pub async fn complete(provider: &dyn LlmProvider, request: LlmRequest) -> Result<LlmResponse, LlmError> {
    let stream = provider.complete_stream(request).await?;
    collect_stream(stream).await
}

enum PendingBlock {
    Text(String),
    ToolUse { id: String, name: String, input_json: String },
}

pub async fn collect_stream(mut stream: LlmEventStream) -> Result<LlmResponse, LlmError> {
    let mut pending: BTreeMap<usize, PendingBlock> = BTreeMap::new();
    let mut finished: BTreeMap<usize, ContentBlock> = BTreeMap::new();

    while let Some(event) = stream.next().await {
        match event? {
            StreamEvent::ContentBlockStart { index, block } => {
                let pending_block = match block {
                    PartialBlock::Text => PendingBlock::Text(String::new()),
                    PartialBlock::ToolUse { id, name } => {
                        PendingBlock::ToolUse { id, name, input_json: String::new() }
                    }
                };
                pending.insert(index, pending_block);
            }
            StreamEvent::TextDelta { index, text } => {
                if let Some(PendingBlock::Text(buffer)) = pending.get_mut(&index) {
                    buffer.push_str(&text);
                }
            }
            StreamEvent::ToolInputDelta { index, partial_json } => {
                if let Some(PendingBlock::ToolUse { input_json, .. }) = pending.get_mut(&index) {
                    input_json.push_str(&partial_json);
                }
            }
            StreamEvent::ContentBlockDone { index } => {
                if let Some(block) = pending.remove(&index) {
                    let content_block = match block {
                        PendingBlock::Text(text) => ContentBlock::Text { text },
                        PendingBlock::ToolUse { id, name, input_json } => {
                            let input = if input_json.is_empty() {
                                serde_json::json!({})
                            } else {
                                serde_json::from_str(&input_json)
                                    .map_err(|e| LlmError::ParseError(format!("invalid tool input json: {e}")))?
                            };
                            ContentBlock::ToolUse { id, name, input }
                        }
                    };
                    finished.insert(index, content_block);
                }
            }
            StreamEvent::Done { stop_reason, input_tokens, output_tokens } => {
                let content = finished.into_values().collect();
                return Ok(LlmResponse { content, stop_reason, input_tokens, output_tokens });
            }
        }
    }

    Err(LlmError::ParseError("stream ended without a Done event".to_string()))
}
