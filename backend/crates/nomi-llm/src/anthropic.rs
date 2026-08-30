use async_stream::try_stream;
use async_trait::async_trait;
use eventsource_stream::Eventsource;
use futures_util::StreamExt;
use serde_json::json;

use super::types::{ContentBlock, LlmError, LlmEventStream, LlmRequest, LlmRole, PartialBlock, StopReason, StreamEvent};
use super::LlmProvider;

pub struct AnthropicProvider {
    client: reqwest::Client,
    api_key: String,
    model: String,
    base_url: String,
}

impl AnthropicProvider {
    pub fn new(client: reqwest::Client, api_key: String, model: String, base_url: String) -> Self {
        Self { client, api_key, model, base_url }
    }

    pub fn default_base_url() -> String {
        "https://api.anthropic.com".to_string()
    }

    fn build_body(&self, request: &LlmRequest, stream: bool) -> serde_json::Value {
        let messages: Vec<serde_json::Value> = request
            .messages
            .iter()
            .map(|m| {
                json!({
                    "role": role_to_str(&m.role),
                    "content": m.content.iter().map(content_block_to_json).collect::<Vec<_>>(),
                })
            })
            .collect();

        let tools: Vec<serde_json::Value> = request
            .tools
            .iter()
            .map(|t| json!({ "name": t.name, "description": t.description, "input_schema": t.input_schema }))
            .collect();

        let mut body = json!({
            "model": self.model,
            "max_tokens": request.max_tokens,
            "messages": messages,
            "stream": stream,
        });
        if let Some(system) = &request.system {
            body["system"] = json!(system);
        }
        if !tools.is_empty() {
            body["tools"] = json!(tools);
        }
        body
    }
}

fn role_to_str(role: &LlmRole) -> &'static str {
    match role {
        LlmRole::User => "user",
        LlmRole::Assistant => "assistant",
    }
}

pub async fn list_models(client: &reqwest::Client, api_key: &str, base_url: &str) -> Result<Vec<crate::ModelSummary>, LlmError> {
    let response = client
        .get(format!("{base_url}/v1/models"))
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        return Err(LlmError::ProviderError(format!("anthropic returned {status}: {text}")));
    }

    let body: serde_json::Value = response.json().await.map_err(|e| LlmError::ParseError(e.to_string()))?;
    let data = body.get("data").and_then(|d| d.as_array()).ok_or_else(|| LlmError::ParseError("missing data".to_string()))?;

    Ok(data
        .iter()
        .filter_map(|m| {
            let id = m.get("id").and_then(|v| v.as_str())?.to_string();
            let label = m.get("display_name").and_then(|v| v.as_str()).map(|s| s.to_string());
            Some(crate::ModelSummary { id, label })
        })
        .collect())
}

fn content_block_to_json(block: &ContentBlock) -> serde_json::Value {
    match block {
        ContentBlock::Text { text } => json!({ "type": "text", "text": text }),
        ContentBlock::ToolUse { id, name, input, .. } => {
            json!({ "type": "tool_use", "id": id, "name": name, "input": input })
        }
        ContentBlock::ToolResult { tool_use_id, content, is_error } => {
            json!({ "type": "tool_result", "tool_use_id": tool_use_id, "content": content, "is_error": is_error })
        }
    }
}

#[async_trait]
impl LlmProvider for AnthropicProvider {
    async fn complete_stream(&self, request: LlmRequest) -> Result<LlmEventStream, LlmError> {
        let body = self.build_body(&request, true);

        let response = self
            .client
            .post(format!("{}/v1/messages", self.base_url))
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(LlmError::ProviderError(format!("anthropic returned {status}: {text}")));
        }

        let mut events = response.bytes_stream().eventsource();

        let stream = try_stream! {
            let mut input_tokens: u32 = 0;

            loop {
                let event = match events.next().await {
                    Some(event) => event.map_err(|e| LlmError::ParseError(format!("sse stream error: {e}")))?,
                    None => Err(LlmError::ParseError("stream ended without a Done event".to_string()))?,
                };

                match event.event.as_str() {
                    "message_start" => {
                        let data: serde_json::Value = serde_json::from_str(&event.data)
                            .map_err(|e| LlmError::ParseError(format!("invalid message_start json: {e}")))?;
                        input_tokens = data
                            .get("message")
                            .and_then(|m| m.get("usage"))
                            .and_then(|u| u.get("input_tokens"))
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0) as u32;
                    }
                    "content_block_start" => {
                        let data: serde_json::Value = serde_json::from_str(&event.data)
                            .map_err(|e| LlmError::ParseError(format!("invalid content_block_start json: {e}")))?;
                        let index = data.get("index").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                        let block = data
                            .get("content_block")
                            .ok_or_else(|| LlmError::ParseError("missing content_block".to_string()))?;
                        let partial = match block.get("type").and_then(|t| t.as_str()) {
                            Some("text") => PartialBlock::Text,
                            Some("tool_use") => PartialBlock::ToolUse {
                                id: block.get("id").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
                                name: block.get("name").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
                                thought_signature: None,
                            },
                            other => Err(LlmError::ParseError(format!("unknown content_block type: {other:?}")))?,
                        };
                        yield StreamEvent::ContentBlockStart { index, block: partial };
                    }
                    "content_block_delta" => {
                        let data: serde_json::Value = serde_json::from_str(&event.data)
                            .map_err(|e| LlmError::ParseError(format!("invalid content_block_delta json: {e}")))?;
                        let index = data.get("index").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                        let delta = data
                            .get("delta")
                            .ok_or_else(|| LlmError::ParseError("missing delta".to_string()))?;
                        match delta.get("type").and_then(|t| t.as_str()) {
                            Some("text_delta") => {
                                let text = delta.get("text").and_then(|v| v.as_str()).unwrap_or_default().to_string();
                                yield StreamEvent::TextDelta { index, text };
                            }
                            Some("input_json_delta") => {
                                let partial_json =
                                    delta.get("partial_json").and_then(|v| v.as_str()).unwrap_or_default().to_string();
                                yield StreamEvent::ToolInputDelta { index, partial_json };
                            }
                            _ => {}
                        }
                    }
                    "content_block_stop" => {
                        let data: serde_json::Value = serde_json::from_str(&event.data)
                            .map_err(|e| LlmError::ParseError(format!("invalid content_block_stop json: {e}")))?;
                        let index = data.get("index").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                        yield StreamEvent::ContentBlockDone { index };
                    }
                    "message_delta" => {
                        let data: serde_json::Value = serde_json::from_str(&event.data)
                            .map_err(|e| LlmError::ParseError(format!("invalid message_delta json: {e}")))?;
                        let stop_reason = match data.get("delta").and_then(|d| d.get("stop_reason")).and_then(|s| s.as_str()) {
                            Some("end_turn") => StopReason::EndTurn,
                            Some("tool_use") => StopReason::ToolUse,
                            Some("max_tokens") => StopReason::MaxTokens,
                            Some(other) => StopReason::Other(other.to_string()),
                            None => StopReason::Other("unknown".to_string()),
                        };
                        let output_tokens =
                            data.get("usage").and_then(|u| u.get("output_tokens")).and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                        yield StreamEvent::Done { stop_reason, input_tokens, output_tokens };
                        return;
                    }
                    "error" => {
                        let data: serde_json::Value =
                            serde_json::from_str(&event.data).unwrap_or(serde_json::Value::Null);
                        Err(LlmError::ProviderError(format!("anthropic stream error: {data}")))?;
                    }
                    _ => {}
                }
            }
        };

        Ok(Box::pin(stream))
    }
}
