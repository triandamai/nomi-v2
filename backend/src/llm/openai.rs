use async_trait::async_trait;
use serde_json::json;

use super::types::{ContentBlock, LlmError, LlmRequest, LlmResponse, LlmRole, StopReason};
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
}

fn role_to_str(role: &LlmRole) -> &'static str {
    match role {
        LlmRole::User => "user",
        LlmRole::Assistant => "assistant",
    }
}

#[async_trait]
impl LlmProvider for OpenAiProvider {
    async fn complete(&self, request: LlmRequest) -> Result<LlmResponse, LlmError> {
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
                    ContentBlock::ToolUse { id, name, input } => {
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

        let body: serde_json::Value = response
            .json()
            .await
            .map_err(|e| LlmError::ParseError(e.to_string()))?;

        let choice = body
            .get("choices")
            .and_then(|c| c.as_array())
            .and_then(|c| c.first())
            .ok_or_else(|| LlmError::ParseError("missing choices[0]".to_string()))?;

        let message = choice
            .get("message")
            .ok_or_else(|| LlmError::ParseError("missing message".to_string()))?;

        let mut content = Vec::new();
        if let Some(text) = message.get("content").and_then(|c| c.as_str()) {
            content.push(ContentBlock::Text { text: text.to_string() });
        }
        if let Some(tool_calls) = message.get("tool_calls").and_then(|t| t.as_array()) {
            for tc in tool_calls {
                let id = tc.get("id").and_then(|v| v.as_str()).unwrap_or_default().to_string();
                let function = tc
                    .get("function")
                    .ok_or_else(|| LlmError::ParseError("missing function".to_string()))?;
                let name = function.get("name").and_then(|v| v.as_str()).unwrap_or_default().to_string();
                let arguments_str = function.get("arguments").and_then(|v| v.as_str()).unwrap_or("{}");
                let input: serde_json::Value = serde_json::from_str(arguments_str)
                    .map_err(|e| LlmError::ParseError(format!("invalid tool_calls arguments JSON: {e}")))?;
                content.push(ContentBlock::ToolUse { id, name, input });
            }
        }

        let stop_reason = match choice.get("finish_reason").and_then(|s| s.as_str()) {
            Some("stop") => StopReason::EndTurn,
            Some("tool_calls") => StopReason::ToolUse,
            Some("length") => StopReason::MaxTokens,
            Some(other) => StopReason::Other(other.to_string()),
            None => StopReason::Other("unknown".to_string()),
        };

        let input_tokens = body.get("usage").and_then(|u| u.get("prompt_tokens")).and_then(|v| v.as_u64()).unwrap_or(0) as u32;
        let output_tokens = body.get("usage").and_then(|u| u.get("completion_tokens")).and_then(|v| v.as_u64()).unwrap_or(0) as u32;

        Ok(LlmResponse { content, stop_reason, input_tokens, output_tokens })
    }
}
