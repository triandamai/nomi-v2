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
    async fn complete_stream(&self, request: LlmRequest) -> Result<LlmEventStream, LlmError>;
}

/// Wraps a complete (non-streaming) LlmResponse as a single-shot LlmEventStream.
/// Used directly by fake providers to produce single-shot streams without a real network round trip.
pub fn response_to_stream(response: LlmResponse) -> LlmEventStream {
    let mut events: Vec<Result<StreamEvent, LlmError>> = Vec::new();
    for (index, block) in response.content.into_iter().enumerate() {
        match block {
            ContentBlock::Text { text } => {
                events.push(Ok(StreamEvent::ContentBlockStart { index, block: PartialBlock::Text }));
                events.push(Ok(StreamEvent::TextDelta { index, text }));
                events.push(Ok(StreamEvent::ContentBlockDone { index }));
            }
            ContentBlock::ToolUse { id, name, input, thought_signature } => {
                events.push(Ok(StreamEvent::ContentBlockStart {
                    index,
                    block: PartialBlock::ToolUse { id, name, thought_signature },
                }));
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

/// Runs a minimal real call through the given config to prove it actually works, without
/// persisting anything — used to validate a bring-your-own-key submission before saving it.
pub async fn validate_model_config(config: ModelConfig, http_client: reqwest::Client) -> Result<(), LlmError> {
    let provider = build_provider(config, http_client);
    let request = LlmRequest {
        system: None,
        messages: vec![LlmMessage { role: LlmRole::User, content: vec![ContentBlock::Text { text: "Hi".to_string() }] }],
        tools: vec![],
        max_tokens: 8,
    };
    complete(provider.as_ref(), request).await?;
    Ok(())
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ModelSummary {
    pub id: String,
    pub label: Option<String>,
}

/// Calls the provider's real model-listing API using the given key, without persisting
/// anything — used by the admin UI to populate a model picker instead of a free-text field.
pub async fn list_provider_models(config: ModelConfig, http_client: reqwest::Client) -> Result<Vec<ModelSummary>, LlmError> {
    let base_url = config.base_url.clone().unwrap_or_else(|| match config.provider {
        ProviderKind::Anthropic => anthropic::AnthropicProvider::default_base_url(),
        ProviderKind::OpenAi => openai::OpenAiProvider::default_base_url(),
        ProviderKind::Gemini => gemini::GeminiProvider::default_base_url(),
        ProviderKind::Fake => String::new(),
    });
    match config.provider {
        ProviderKind::Anthropic => anthropic::list_models(&http_client, &config.api_key, &base_url).await,
        ProviderKind::OpenAi => openai::list_models(&http_client, &config.api_key, &base_url).await,
        ProviderKind::Gemini => gemini::list_models(&http_client, &config.api_key, &base_url).await,
        ProviderKind::Fake => Ok(vec![ModelSummary {
            id: "fake-model".to_string(),
            label: Some("Fake Model (dev/testing)".to_string()),
        }]),
    }
}

enum PendingBlock {
    Text(String),
    ToolUse { id: String, name: String, input_json: String, thought_signature: Option<String> },
}

pub async fn collect_stream(mut stream: LlmEventStream) -> Result<LlmResponse, LlmError> {
    let mut pending: BTreeMap<usize, PendingBlock> = BTreeMap::new();
    let mut finished: BTreeMap<usize, ContentBlock> = BTreeMap::new();

    while let Some(event) = stream.next().await {
        match event? {
            StreamEvent::ContentBlockStart { index, block } => {
                let pending_block = match block {
                    PartialBlock::Text => PendingBlock::Text(String::new()),
                    PartialBlock::ToolUse { id, name, thought_signature } => {
                        PendingBlock::ToolUse { id, name, input_json: String::new(), thought_signature }
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
                        PendingBlock::ToolUse { id, name, input_json, thought_signature } => {
                            let input = if input_json.is_empty() {
                                serde_json::json!({})
                            } else {
                                serde_json::from_str(&input_json)
                                    .map_err(|e| LlmError::ParseError(format!("invalid tool input json: {e}")))?
                            };
                            ContentBlock::ToolUse { id, name, input, thought_signature }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn validate_model_config_succeeds_against_the_fake_provider() {
        let config = ModelConfig {
            provider: ProviderKind::Fake,
            model_id: String::new(),
            api_key: String::new(),
            base_url: None,
        };
        let result = validate_model_config(config, reqwest::Client::new()).await;
        assert!(result.is_ok());
    }
}
