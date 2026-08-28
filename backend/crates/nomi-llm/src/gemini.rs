use async_stream::try_stream;
use async_trait::async_trait;
use eventsource_stream::Eventsource;
use futures_util::StreamExt;
use serde_json::json;

use super::types::{ContentBlock, LlmError, LlmEventStream, LlmRequest, LlmRole, PartialBlock, StopReason, StreamEvent};
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

    fn build_body(&self, request: &LlmRequest) -> serde_json::Value {
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
        body
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
        ContentBlock::ToolUse { name, input, thought_signature, .. } => {
            let mut part = json!({ "functionCall": { "name": name, "args": input } });
            if let Some(signature) = thought_signature {
                part["thoughtSignature"] = json!(signature);
            }
            part
        }
        ContentBlock::ToolResult { tool_use_id, content, .. } => {
            json!({ "functionResponse": { "name": tool_use_id, "response": { "content": content } } })
        }
    }
}

#[async_trait]
impl LlmProvider for GeminiProvider {
    async fn complete_stream(&self, request: LlmRequest) -> Result<LlmEventStream, LlmError> {
        let body = self.build_body(&request);
        let url = format!(
            "{}/v1beta/models/{}:streamGenerateContent?alt=sse&key={}",
            self.base_url, self.model, self.api_key
        );

        let response = self.client.post(url).json(&body).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(LlmError::ProviderError(format!("gemini returned {status}: {text}")));
        }

        let mut events = response.bytes_stream().eventsource();

        let stream = try_stream! {
            let mut next_index: usize = 0;
            let mut text_index: Option<usize> = None;
            let mut saw_function_call = false;
            let mut input_tokens: u32 = 0;
            let mut output_tokens: u32 = 0;
            let mut finish_reason: Option<String> = None;

            loop {
                let event = match events.next().await {
                    Some(event) => event.map_err(|e| LlmError::ParseError(format!("sse stream error: {e}")))?,
                    None => break,
                };

                let data: serde_json::Value = serde_json::from_str(&event.data)
                    .map_err(|e| LlmError::ParseError(format!("invalid stream chunk json: {e}")))?;

                if let Some(usage) = data.get("usageMetadata") {
                    input_tokens = usage.get("promptTokenCount").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                    output_tokens = usage.get("candidatesTokenCount").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                }

                let candidate = data.get("candidates").and_then(|c| c.as_array()).and_then(|c| c.first());
                let Some(candidate) = candidate else {
                    continue;
                };

                if let Some(fr) = candidate.get("finishReason").and_then(|f| f.as_str()) {
                    finish_reason = Some(fr.to_string());
                }

                let parts = candidate.get("content").and_then(|c| c.get("parts")).and_then(|p| p.as_array());
                if let Some(parts) = parts {
                    for part in parts {
                        if let Some(text) = part.get("text").and_then(|t| t.as_str()) {
                            let is_first = text_index.is_none();
                            let index = *text_index.get_or_insert_with(|| {
                                let idx = next_index;
                                next_index += 1;
                                idx
                            });
                            if is_first {
                                yield StreamEvent::ContentBlockStart { index, block: PartialBlock::Text };
                            }
                            yield StreamEvent::TextDelta { index, text: text.to_string() };
                        } else if let Some(fc) = part.get("functionCall") {
                            saw_function_call = true;
                            let name = fc.get("name").and_then(|v| v.as_str()).unwrap_or_default().to_string();
                            let args = fc.get("args").cloned().unwrap_or(serde_json::Value::Null);
                            // thoughtSignature is a sibling of functionCall on the part itself,
                            // not nested inside it — required by Gemini's "thinking" models so a
                            // later turn can replay this exact function call without a 400.
                            let thought_signature =
                                part.get("thoughtSignature").and_then(|v| v.as_str()).map(|s| s.to_string());
                            let index = next_index;
                            next_index += 1;
                            // Gemini has no tool-call id; synthesize one from the function name,
                            // matching complete()'s convention.
                            yield StreamEvent::ContentBlockStart {
                                index,
                                block: PartialBlock::ToolUse { id: name.clone(), name, thought_signature },
                            };
                            yield StreamEvent::ToolInputDelta { index, partial_json: args.to_string() };
                            yield StreamEvent::ContentBlockDone { index };
                        }
                    }
                }
            }

            if let Some(index) = text_index {
                yield StreamEvent::ContentBlockDone { index };
            }

            let stop_reason = if saw_function_call {
                StopReason::ToolUse
            } else {
                match finish_reason.as_deref() {
                    Some("STOP") => StopReason::EndTurn,
                    Some("MAX_TOKENS") => StopReason::MaxTokens,
                    Some(other) => StopReason::Other(other.to_string()),
                    None => StopReason::Other("unknown".to_string()),
                }
            };

            yield StreamEvent::Done { stop_reason, input_tokens, output_tokens };
        };

        Ok(Box::pin(stream))
    }
}
