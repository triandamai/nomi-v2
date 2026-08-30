use async_stream::try_stream;
use async_trait::async_trait;
use eventsource_stream::Eventsource;
use futures_util::StreamExt;
use serde_json::json;
use std::collections::BTreeMap;

use super::types::{ContentBlock, LlmError, LlmEventStream, LlmRequest, LlmRole, PartialBlock, StopReason, StreamEvent};
use super::LlmProvider;

pub struct OpenAiProvider {
    client: reqwest::Client,
    api_key: String,
    model: String,
    base_url: String,
}

impl OpenAiProvider {
    pub fn new(client: reqwest::Client, api_key: String, model: String, base_url: String) -> Self {
        Self { client, api_key, model, base_url }
    }

    pub fn default_base_url() -> String {
        "https://api.openai.com".to_string()
    }

    fn build_body(&self, request: &LlmRequest, stream: bool) -> serde_json::Value {
        let mut messages: Vec<serde_json::Value> = Vec::new();
        if let Some(system) = &request.system {
            messages.push(json!({ "role": "system", "content": system }));
        }

        for m in &request.messages {
            let mut text_parts = Vec::new();
            let mut tool_calls = Vec::new();
            let mut tool_result_messages = Vec::new();

            for block in &m.content {
                match block {
                    ContentBlock::Text { text } => text_parts.push(text.clone()),
                    ContentBlock::ToolUse { id, name, input, .. } => {
                        tool_calls.push(json!({
                            "id": id,
                            "type": "function",
                            "function": { "name": name, "arguments": input.to_string() }
                        }));
                    }
                    ContentBlock::ToolResult { tool_use_id, content, .. } => {
                        tool_result_messages.push(json!({
                            "role": "tool",
                            "tool_call_id": tool_use_id,
                            "content": content,
                        }));
                    }
                }
            }

            if !text_parts.is_empty() || !tool_calls.is_empty() {
                let mut msg = json!({ "role": role_to_str(&m.role) });
                msg["content"] = if !text_parts.is_empty() {
                    json!(text_parts.join(""))
                } else {
                    serde_json::Value::Null
                };
                if !tool_calls.is_empty() {
                    msg["tool_calls"] = json!(tool_calls);
                }
                messages.push(msg);
            }
            messages.extend(tool_result_messages);
        }

        let tools: Vec<serde_json::Value> = request
            .tools
            .iter()
            .map(|t| json!({
                "type": "function",
                "function": { "name": t.name, "description": t.description, "parameters": t.input_schema }
            }))
            .collect();

        let mut body = json!({
            "model": self.model,
            "max_tokens": request.max_tokens,
            "messages": messages,
        });
        if !tools.is_empty() {
            body["tools"] = json!(tools);
        }
        if stream {
            body["stream"] = json!(true);
            body["stream_options"] = json!({ "include_usage": true });
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
    let response = client.get(format!("{base_url}/v1/models")).bearer_auth(api_key).send().await?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        return Err(LlmError::ProviderError(format!("openai returned {status}: {text}")));
    }

    let body: serde_json::Value = response.json().await.map_err(|e| LlmError::ParseError(e.to_string()))?;
    let data = body.get("data").and_then(|d| d.as_array()).ok_or_else(|| LlmError::ParseError("missing data".to_string()))?;

    Ok(data
        .iter()
        .filter_map(|m| m.get("id").and_then(|v| v.as_str()))
        .map(|id| crate::ModelSummary { id: id.to_string(), label: None })
        .collect())
}

#[async_trait]
impl LlmProvider for OpenAiProvider {
    async fn complete_stream(&self, request: LlmRequest) -> Result<LlmEventStream, LlmError> {
        let body = self.build_body(&request, true);

        let response = self
            .client
            .post(format!("{}/v1/chat/completions", self.base_url))
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(LlmError::ProviderError(format!("openai returned {status}: {text}")));
        }

        let mut events = response.bytes_stream().eventsource();

        let stream = try_stream! {
            let mut next_index: usize = 0;
            let mut text_index: Option<usize> = None;
            let mut tool_index_map: BTreeMap<u64, usize> = BTreeMap::new();
            let mut open_indices: Vec<usize> = Vec::new();
            let mut stop_reason = StopReason::Other("unknown".to_string());
            let mut input_tokens: u32 = 0;
            let mut output_tokens: u32 = 0;

            loop {
                let event = match events.next().await {
                    Some(event) => event.map_err(|e| LlmError::ParseError(format!("sse stream error: {e}")))?,
                    None => Err(LlmError::ParseError("stream ended without a Done event".to_string()))?,
                };

                if event.data == "[DONE]" {
                    yield StreamEvent::Done { stop_reason, input_tokens, output_tokens };
                    return;
                }

                let data: serde_json::Value = serde_json::from_str(&event.data)
                    .map_err(|e| LlmError::ParseError(format!("invalid stream chunk json: {e}")))?;

                if let Some(usage) = data.get("usage") {
                    if !usage.is_null() {
                        input_tokens = usage.get("prompt_tokens").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                        output_tokens = usage.get("completion_tokens").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                    }
                }

                let choice = data.get("choices").and_then(|c| c.as_array()).and_then(|c| c.first());
                let Some(choice) = choice else {
                    continue;
                };

                if let Some(text) = choice.get("delta").and_then(|d| d.get("content")).and_then(|c| c.as_str()) {
                    let is_first = text_index.is_none();
                    let index = *text_index.get_or_insert_with(|| {
                        let idx = next_index;
                        next_index += 1;
                        open_indices.push(idx);
                        idx
                    });
                    if is_first {
                        yield StreamEvent::ContentBlockStart { index, block: PartialBlock::Text };
                    }
                    yield StreamEvent::TextDelta { index, text: text.to_string() };
                }

                if let Some(tool_calls) = choice.get("delta").and_then(|d| d.get("tool_calls")).and_then(|t| t.as_array()) {
                    for tc in tool_calls {
                        let raw_index = tc.get("index").and_then(|v| v.as_u64()).unwrap_or(0);
                        let is_new = !tool_index_map.contains_key(&raw_index);
                        let index = *tool_index_map.entry(raw_index).or_insert_with(|| {
                            let idx = next_index;
                            next_index += 1;
                            idx
                        });
                        if is_new {
                            let id = tc.get("id").and_then(|v| v.as_str()).unwrap_or_default().to_string();
                            let name = tc
                                .get("function")
                                .and_then(|f| f.get("name"))
                                .and_then(|v| v.as_str())
                                .unwrap_or_default()
                                .to_string();
                            yield StreamEvent::ContentBlockStart {
                                index,
                                block: PartialBlock::ToolUse { id, name, thought_signature: None },
                            };
                            open_indices.push(index);
                        }
                        if let Some(args) = tc.get("function").and_then(|f| f.get("arguments")).and_then(|v| v.as_str()) {
                            yield StreamEvent::ToolInputDelta { index, partial_json: args.to_string() };
                        }
                    }
                }

                if let Some(finish_reason) = choice.get("finish_reason").and_then(|f| f.as_str()) {
                    stop_reason = match finish_reason {
                        "stop" => StopReason::EndTurn,
                        "tool_calls" => StopReason::ToolUse,
                        "length" => StopReason::MaxTokens,
                        other => StopReason::Other(other.to_string()),
                    };
                    for index in open_indices.drain(..) {
                        yield StreamEvent::ContentBlockDone { index };
                    }
                }
            }
        };

        Ok(Box::pin(stream))
    }
}
