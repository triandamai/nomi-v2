pub mod types;
pub mod anthropic;
pub mod openai;
pub mod openrouter;
pub mod gemini;
pub mod deepseek;
pub mod config;
pub mod fake;
pub mod media;
mod user_message;

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
            ContentBlock::ToolResult { .. } | ContentBlock::Media { .. } => {
                // A provider's own response never contains a ToolResult or Media block (those are
                // only ever something we send as part of a request) — nothing to emit.
            }
            ContentBlock::Thinking { text, signature } => {
                events.push(Ok(StreamEvent::ContentBlockStart { index, block: PartialBlock::Thinking }));
                events.push(Ok(StreamEvent::ThinkingDelta { index, text }));
                if let Some(signature) = signature {
                    events.push(Ok(StreamEvent::ThinkingSignature { index, signature }));
                }
                events.push(Ok(StreamEvent::ContentBlockDone { index }));
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
        enable_reasoning: false,
        reasoning_effort: Default::default(),
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
        ProviderKind::OpenRouter => openrouter::OpenRouterProvider::default_base_url(),
        ProviderKind::Gemini => gemini::GeminiProvider::default_base_url(),
        ProviderKind::DeepSeek => deepseek::DeepSeekProvider::default_base_url(),
        ProviderKind::Fake => String::new(),
    });
    match config.provider {
        ProviderKind::Anthropic => anthropic::list_models(&http_client, &config.api_key, &base_url).await,
        ProviderKind::OpenAi => openai::list_models(&http_client, &config.api_key, &base_url).await,
        ProviderKind::OpenRouter => openrouter::list_models(&http_client, &config.api_key, &base_url).await,
        ProviderKind::Gemini => gemini::list_models(&http_client, &config.api_key, &base_url).await,
        ProviderKind::DeepSeek => deepseek::list_models(&http_client, &config.api_key, &base_url).await,
        ProviderKind::Fake => Ok(vec![ModelSummary {
            id: "fake-model".to_string(),
            label: Some("Fake Model (dev/testing)".to_string()),
        }]),
    }
}

enum PendingBlock {
    Text(String),
    Thinking { text: String, signature: Option<String> },
    ToolUse { id: String, name: String, input_json: String, thought_signature: Option<String> },
}

pub async fn collect_stream(mut stream: LlmEventStream) -> Result<LlmResponse, LlmError> {
    let mut pending: BTreeMap<usize, PendingBlock> = BTreeMap::new();
    let mut finished: BTreeMap<usize, ContentBlock> = BTreeMap::new();
    // A tool call whose input isn't valid JSON. Expected when the response hit max_tokens mid
    // call (the call is dropped and the response reported as cut off); an error otherwise.
    let mut broken_tool_input: Option<String> = None;

    while let Some(event) = stream.next().await {
        match event? {
            StreamEvent::ContentBlockStart { index, block } => {
                let pending_block = match block {
                    PartialBlock::Text => PendingBlock::Text(String::new()),
                    PartialBlock::Thinking => PendingBlock::Thinking { text: String::new(), signature: None },
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
            StreamEvent::ThinkingDelta { index, text } => {
                if let Some(PendingBlock::Thinking { text: buffer, .. }) = pending.get_mut(&index) {
                    buffer.push_str(&text);
                }
            }
            StreamEvent::ThinkingSignature { index, signature } => {
                if let Some(PendingBlock::Thinking { signature: slot, .. }) = pending.get_mut(&index) {
                    *slot = Some(signature);
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
                        PendingBlock::Thinking { text, signature } => ContentBlock::Thinking { text, signature },
                        PendingBlock::ToolUse { id, name, input_json, thought_signature } => {
                            let input = if input_json.is_empty() {
                                serde_json::json!({})
                            } else {
                                match serde_json::from_str(&input_json) {
                                    Ok(input) => input,
                                    Err(e) => {
                                        broken_tool_input = Some(format!("invalid tool input json: {e}"));
                                        continue;
                                    }
                                }
                            };
                            ContentBlock::ToolUse { id, name, input, thought_signature }
                        }
                    };
                    finished.insert(index, content_block);
                }
            }
            StreamEvent::Done { stop_reason, input_tokens, output_tokens } => {
                if let Some(error) = broken_tool_input {
                    if stop_reason != StopReason::MaxTokens {
                        return Err(LlmError::ParseError(error));
                    }
                }
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

    fn tool_call_stream(partial_json: &str, stop_reason: StopReason) -> LlmEventStream {
        let events = vec![
            Ok(StreamEvent::ContentBlockStart { index: 0, block: PartialBlock::ToolUse { id: "t1".into(), name: "write_files".into(), thought_signature: None } }),
            Ok(StreamEvent::ToolInputDelta { index: 0, partial_json: partial_json.to_string() }),
            Ok(StreamEvent::ContentBlockDone { index: 0 }),
            Ok(StreamEvent::Done { stop_reason, input_tokens: 1, output_tokens: 1 }),
        ];
        Box::pin(futures_util::stream::iter(events))
    }

    #[tokio::test]
    async fn a_tool_call_cut_off_by_max_tokens_is_dropped_not_an_error() {
        let response = collect_stream(tool_call_stream(r#"{"files": [{"path": "a.ts", "cont"#, StopReason::MaxTokens)).await.unwrap();
        assert_eq!(response.stop_reason, StopReason::MaxTokens);
        assert!(response.content.is_empty());
        assert!(collect_stream(tool_call_stream(r#"{"files": "#, StopReason::ToolUse)).await.is_err());
    }

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
