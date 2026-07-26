use async_trait::async_trait;
use serde_json::json;

use super::types::{ContentBlock, LlmError, LlmRequest, LlmResponse, LlmRole, StopReason};
use super::LlmProvider;

pub struct GeminiProvider {
    client: reqwest::Client,
    api_key: String,
    model: String,
    base_url: String,
}

impl GeminiProvider {
    pub fn new(client: reqwest::Client, api_key: String, model: String, base_url: String) -> Self {
        Self { client, api_key, model, base_url }
    }

    pub fn default_base_url() -> String {
        "https://generativelanguage.googleapis.com".to_string()
    }
}

fn role_to_str(role: &LlmRole) -> &'static str {
    match role {
        LlmRole::User => "user",
        LlmRole::Assistant => "model",
    }
}

fn content_block_to_part(block: &ContentBlock) -> serde_json::Value {
    match block {
        ContentBlock::Text { text } => json!({ "text": text }),
        ContentBlock::ToolUse { name, input, .. } => {
            json!({ "functionCall": { "name": name, "args": input } })
        }
        // Gemini's functionResponse part is keyed by the function's name, not a
        // tool-call id (Gemini has no id concept for function calls). On the
        // inbound/parse side of this file, ToolUse.id is synthesized directly
        // from the function's name, so tool_use_id here already holds that
        // name - reuse it as-is for the outbound functionResponse.name.
        ContentBlock::ToolResult { tool_use_id, content, .. } => {
            json!({ "functionResponse": { "name": tool_use_id, "response": { "content": content } } })
        }
    }
}

#[async_trait]
impl LlmProvider for GeminiProvider {
    async fn complete(&self, request: LlmRequest) -> Result<LlmResponse, LlmError> {
        let contents: Vec<serde_json::Value> = request
            .messages
            .iter()
            .map(|m| {
                json!({
                    "role": role_to_str(&m.role),
                    "parts": m.content.iter().map(content_block_to_part).collect::<Vec<_>>(),
                })
            })
            .collect();

        let mut body = json!({ "contents": contents });
        if let Some(system) = &request.system {
            body["systemInstruction"] = json!({ "parts": [{ "text": system }] });
        }
        if !request.tools.is_empty() {
            let declarations: Vec<serde_json::Value> = request
                .tools
                .iter()
                .map(|t| json!({ "name": t.name, "description": t.description, "parameters": t.input_schema }))
                .collect();
            body["tools"] = json!([{ "functionDeclarations": declarations }]);
        }
        body["generationConfig"] = json!({ "maxOutputTokens": request.max_tokens });

        let url = format!(
            "{}/v1beta/models/{}:generateContent?key={}",
            self.base_url, self.model, self.api_key
        );

        let response = self.client.post(url).json(&body).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(LlmError::ProviderError(format!("gemini returned {status}: {text}")));
        }

        let body: serde_json::Value = response
            .json()
            .await
            .map_err(|e| LlmError::ParseError(e.to_string()))?;

        let candidate = body
            .get("candidates")
            .and_then(|c| c.as_array())
            .and_then(|c| c.first())
            .ok_or_else(|| LlmError::ParseError("missing candidates[0]".to_string()))?;

        let parts = candidate
            .get("content")
            .and_then(|c| c.get("parts"))
            .and_then(|p| p.as_array())
            .ok_or_else(|| LlmError::ParseError("missing content.parts".to_string()))?;

        let mut content = Vec::new();
        let mut saw_function_call = false;
        for part in parts {
            if let Some(text) = part.get("text").and_then(|t| t.as_str()) {
                content.push(ContentBlock::Text { text: text.to_string() });
            } else if let Some(fc) = part.get("functionCall") {
                saw_function_call = true;
                let name = fc.get("name").and_then(|v| v.as_str()).unwrap_or_default().to_string();
                let args = fc.get("args").cloned().unwrap_or(serde_json::Value::Null);
                // Gemini has no tool-call id; synthesize one from the function name.
                content.push(ContentBlock::ToolUse { id: name.clone(), name, input: args });
            }
        }

        let finish_reason = candidate.get("finishReason").and_then(|s| s.as_str());
        let stop_reason = if saw_function_call {
            StopReason::ToolUse
        } else {
            match finish_reason {
                Some("STOP") => StopReason::EndTurn,
                Some("MAX_TOKENS") => StopReason::MaxTokens,
                Some(other) => StopReason::Other(other.to_string()),
                None => StopReason::Other("unknown".to_string()),
            }
        };

        let input_tokens = body.get("usageMetadata").and_then(|u| u.get("promptTokenCount")).and_then(|v| v.as_u64()).unwrap_or(0) as u32;
        let output_tokens = body.get("usageMetadata").and_then(|u| u.get("candidatesTokenCount")).and_then(|v| v.as_u64()).unwrap_or(0) as u32;

        Ok(LlmResponse { content, stop_reason, input_tokens, output_tokens })
    }
}
