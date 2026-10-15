use base64::Engine;
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

    #[cfg(test)]
    pub(crate) fn body_for_test(&self, request: &LlmRequest) -> serde_json::Value {
        self.build_body(request)
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
                .map(|t| json!({ "name": t.name, "description": t.description, "parameters": clean_gemini_schema(&t.input_schema) }))
                .collect();
            body["tools"] = json!([{ "functionDeclarations": declarations }]);
        }
        // Thinking tokens count against maxOutputTokens same as the final reply — leave headroom
        // beyond what the caller asked for so enabling reasoning doesn't starve the reply itself.
        // Thinking counts against maxOutputTokens, so its budget is added on top of the reply's.
        let thinking_budget = request.reasoning_effort.budget_tokens();
        let max_output_tokens = if request.enable_reasoning { request.max_tokens + thinking_budget } else { request.max_tokens };
        let mut generation_config = json!({ "maxOutputTokens": max_output_tokens });
        if request.enable_reasoning {
            generation_config["thinkingConfig"] = json!({ "includeThoughts": true, "thinkingBudget": thinking_budget });
        }
        body["generationConfig"] = generation_config;
        body
    }
}

fn role_to_str(role: &LlmRole) -> &'static str {
    match role {
        LlmRole::User => "user",
        LlmRole::Assistant => "model",
    }
}

/// Gemini uses a strict subset of OpenAPI 3.0 via Protocol Buffers.
/// Its `Schema.type` field is a single enum (not repeated), so array types like
/// `["number", "null"]` cause proto deserialization failure.
/// Its `Schema.enum` field is `repeated string enum = 9`, so non-string enum values
/// (or enums on non-string types) also fail validation.
/// This normalizes tool schemas so they conform strictly to Gemini's schema specification.
pub fn clean_gemini_schema(value: &serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Object(map) => {
            let mut out = serde_json::Map::new();
            let mut is_nullable = false;
            let mut detected_type: Option<String> = None;

            for (k, v) in map {
                match k.as_str() {
                    "type" => {
                        if let serde_json::Value::Array(types) = v {
                            if types.iter().any(|t| t.as_str() == Some("null")) {
                                is_nullable = true;
                            }
                            let non_null: Vec<&str> = types
                                .iter()
                                .filter_map(|t| t.as_str())
                                .filter(|t| *t != "null")
                                .collect();
                            let primary = non_null.first().copied().unwrap_or("string");
                            detected_type = Some(primary.to_string());
                            out.insert("type".to_string(), serde_json::json!(primary));
                        } else if let Some(s) = v.as_str() {
                            detected_type = Some(s.to_string());
                            out.insert(k.clone(), serde_json::json!(s));
                        } else {
                            out.insert(k.clone(), clean_gemini_schema(v));
                        }
                    }
                    "nullable" => {
                        if v.as_bool() == Some(true) {
                            is_nullable = true;
                        }
                    }
                    "properties" => {
                        if let serde_json::Value::Object(props) = v {
                            let mut cleaned_props = serde_json::Map::new();
                            for (prop_name, prop_val) in props {
                                cleaned_props.insert(prop_name.clone(), clean_gemini_schema(prop_val));
                            }
                            out.insert(k.clone(), serde_json::Value::Object(cleaned_props));
                        } else {
                            out.insert(k.clone(), clean_gemini_schema(v));
                        }
                    }
                    "items" => {
                        out.insert(k.clone(), clean_gemini_schema(v));
                    }
                    "enum" => {
                        out.insert(k.clone(), v.clone());
                    }
                    _ => {
                        out.insert(k.clone(), clean_gemini_schema(v));
                    }
                }
            }

            if is_nullable {
                out.insert("nullable".to_string(), serde_json::json!(true));
            }

            if let Some(enum_val) = out.get("enum").cloned() {
                if let serde_json::Value::Array(items) = enum_val {
                    let schema_type = detected_type.as_deref().unwrap_or("");
                    if schema_type == "string" {
                        let string_enums: Vec<serde_json::Value> = items
                            .into_iter()
                            .filter_map(|item| match item {
                                serde_json::Value::String(s) => Some(serde_json::Value::String(s)),
                                serde_json::Value::Number(n) => Some(serde_json::Value::String(n.to_string())),
                                serde_json::Value::Bool(b) => Some(serde_json::Value::String(b.to_string())),
                                _ => None,
                            })
                            .collect();
                        if string_enums.is_empty() {
                            out.remove("enum");
                        } else {
                            out.insert("enum".to_string(), serde_json::Value::Array(string_enums));
                        }
                    } else {
                        out.remove("enum");
                        let allowed_str = items
                            .iter()
                            .filter_map(|it| match it {
                                serde_json::Value::Null => None,
                                serde_json::Value::String(s) => Some(s.clone()),
                                serde_json::Value::Number(n) => Some(n.to_string()),
                                serde_json::Value::Bool(b) => Some(b.to_string()),
                                _ => None,
                            })
                            .collect::<Vec<_>>()
                            .join(", ");
                        if !allowed_str.is_empty() {
                            let note = format!("Allowed values: {allowed_str}.");
                            let desc = match out.get("description").and_then(|d| d.as_str()) {
                                Some(existing) => format!("{existing} ({note})"),
                                None => note,
                            };
                            out.insert("description".to_string(), serde_json::json!(desc));
                        }
                    }
                }
            }

            serde_json::Value::Object(out)
        }
        serde_json::Value::Array(arr) => {
            serde_json::Value::Array(arr.iter().map(clean_gemini_schema).collect())
        }
        other => other.clone(),
    }
}

pub async fn list_models(client: &reqwest::Client, api_key: &str, base_url: &str) -> Result<Vec<crate::ModelSummary>, LlmError> {
    let response = client
        .get(format!("{base_url}/v1beta/models"))
        .header("x-goog-api-key", api_key)
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        return Err(LlmError::ProviderError(format!("gemini returned {status}: {text}")));
    }

    let body: serde_json::Value = response.json().await.map_err(|e| LlmError::ParseError(e.to_string()))?;
    let models = body.get("models").and_then(|m| m.as_array()).ok_or_else(|| LlmError::ParseError("missing models".to_string()))?;

    Ok(models
        .iter()
        .filter_map(|m| {
            let name = m.get("name").and_then(|v| v.as_str())?;
            let id = name.strip_prefix("models/").unwrap_or(name).to_string();
            let label = m.get("displayName").and_then(|v| v.as_str()).map(|s| s.to_string());
            Some(crate::ModelSummary { id, label })
        })
        .collect())
}

fn content_block_to_part(block: &ContentBlock) -> serde_json::Value {
    match block {
        ContentBlock::Text { text } => json!({ "text": text }),
        // Gemini's thought summaries don't require a signature to replay (unlike its function-call
        // thoughtSignature, or Anthropic's thinking signature) — just the `thought: true` marker.
        ContentBlock::Thinking { text, .. } => json!({ "text": text, "thought": true }),
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
        // Gemini takes images, PDFs, audio and video. Big files were uploaded first (see
        // `upload_large_media`) and are referenced by their File API uri.
        ContentBlock::Media { media_type, data, .. } => match data.strip_prefix(UPLOADED_PREFIX) {
            Some(uri) => json!({ "file_data": { "mime_type": media_type, "file_uri": uri } }),
            None => json!({ "inline_data": { "mime_type": media_type, "data": data } }),
        },
    }
}

/// Marks a Media block's `data` as a File API uri rather than base64 bytes.
const UPLOADED_PREFIX: &str = "gemini-file:";

/// Inline files share a 20 MB cap with the rest of the request; anything bigger (in base64)
/// goes through the File API.
const MAX_INLINE_BASE64: usize = 14 * 1024 * 1024;

impl GeminiProvider {
    /// Uploads each file too big to send inline and points its block at the upload instead.
    async fn upload_large_media(&self, mut request: LlmRequest) -> Result<LlmRequest, LlmError> {
        for message in &mut request.messages {
            for block in &mut message.content {
                if let ContentBlock::Media { media_type, data, name } = block {
                    if data.len() > MAX_INLINE_BASE64 && !data.starts_with(UPLOADED_PREFIX) {
                        let bytes = base64::engine::general_purpose::STANDARD
                            .decode(data.as_bytes())
                            .map_err(|e| LlmError::ParseError(format!("invalid base64 file data: {e}")))?;
                        let uri = self.upload_file(bytes, media_type, name).await?;
                        *data = format!("{UPLOADED_PREFIX}{uri}");
                    }
                }
            }
        }
        Ok(request)
    }

    /// The File API's resumable upload, then a wait until the file is ready to use (videos are
    /// processed for a while after upload).
    async fn upload_file(&self, bytes: Vec<u8>, media_type: &str, name: &str) -> Result<String, LlmError> {
        let start = self
            .client
            .post(format!("{}/upload/v1beta/files", self.base_url))
            .header("x-goog-api-key", &self.api_key)
            .header("X-Goog-Upload-Protocol", "resumable")
            .header("X-Goog-Upload-Command", "start")
            .header("X-Goog-Upload-Header-Content-Length", bytes.len().to_string())
            .header("X-Goog-Upload-Header-Content-Type", media_type)
            .json(&json!({ "file": { "display_name": name } }))
            .send()
            .await?;
        if !start.status().is_success() {
            let status = start.status();
            let text = start.text().await.unwrap_or_default();
            return Err(LlmError::ProviderError(format!("gemini returned {status}: {text}")));
        }
        let upload_url = start
            .headers()
            .get("x-goog-upload-url")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| LlmError::ParseError("gemini upload gave no upload url".to_string()))?
            .to_string();

        let length = bytes.len();
        let uploaded = self
            .client
            .post(upload_url)
            .header("Content-Length", length.to_string())
            .header("X-Goog-Upload-Offset", "0")
            .header("X-Goog-Upload-Command", "upload, finalize")
            .body(bytes)
            .send()
            .await?;
        if !uploaded.status().is_success() {
            let status = uploaded.status();
            let text = uploaded.text().await.unwrap_or_default();
            return Err(LlmError::ProviderError(format!("gemini returned {status}: {text}")));
        }
        let body: serde_json::Value = uploaded.json().await.map_err(|e| LlmError::ParseError(e.to_string()))?;
        let file = body.get("file").cloned().unwrap_or_default();
        let uri = file.get("uri").and_then(|v| v.as_str()).ok_or_else(|| LlmError::ParseError("gemini upload gave no file uri".to_string()))?.to_string();
        let file_name = file.get("name").and_then(|v| v.as_str()).unwrap_or_default().to_string();
        let mut state = file.get("state").and_then(|v| v.as_str()).unwrap_or("ACTIVE").to_string();

        // Up to about five minutes for a long video.
        for _ in 0..150 {
            match state.as_str() {
                "ACTIVE" => return Ok(uri),
                "FAILED" => return Err(LlmError::ProviderError("gemini returned 422: the uploaded file could not be processed".to_string())),
                _ => {}
            }
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            let status: serde_json::Value = self
                .client
                .get(format!("{}/v1beta/{file_name}", self.base_url))
                .header("x-goog-api-key", &self.api_key)
                .send()
                .await?
                .json()
                .await
                .map_err(|e| LlmError::ParseError(e.to_string()))?;
            state = status.get("state").and_then(|v| v.as_str()).unwrap_or("ACTIVE").to_string();
        }
        Err(LlmError::ProviderError("gemini returned 504: the uploaded file took too long to process".to_string()))
    }
}

#[async_trait]
impl LlmProvider for GeminiProvider {
    async fn complete_stream(&self, request: LlmRequest) -> Result<LlmEventStream, LlmError> {
        let request = self.upload_large_media(request).await?;
        let body = self.build_body(&request);
        let url = format!(
            "{}/v1beta/models/{}:streamGenerateContent?alt=sse",
            self.base_url, self.model
        );

        let response = self
            .client
            .post(url)
            .header("x-goog-api-key", &self.api_key)
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(LlmError::ProviderError(format!("gemini returned {status}: {text}")));
        }

        let mut events = response.bytes_stream().eventsource();

        let stream = try_stream! {
            let mut next_index: usize = 0;
            let mut text_index: Option<usize> = None;
            let mut thinking_index: Option<usize> = None;
            let mut saw_function_call = false;
            let mut input_tokens: u32 = 0;
            let mut cached_input_tokens: u32 = 0;
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
                    cached_input_tokens = usage.get("cachedContentTokenCount").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
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
                        let is_thought = part.get("thought").and_then(|v| v.as_bool()).unwrap_or(false);
                        if is_thought {
                            if let Some(text) = part.get("text").and_then(|t| t.as_str()) {
                                let is_first = thinking_index.is_none();
                                let index = *thinking_index.get_or_insert_with(|| {
                                    let idx = next_index;
                                    next_index += 1;
                                    idx
                                });
                                if is_first {
                                    yield StreamEvent::ContentBlockStart { index, block: PartialBlock::Thinking };
                                }
                                yield StreamEvent::ThinkingDelta { index, text: text.to_string() };
                            }
                        } else if let Some(text) = part.get("text").and_then(|t| t.as_str()) {
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

            if let Some(index) = thinking_index {
                yield StreamEvent::ContentBlockDone { index };
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

            if cached_input_tokens > 0 {
                yield StreamEvent::CacheUsage { cached_input_tokens };
            }
            yield StreamEvent::Done { stop_reason, input_tokens, output_tokens };
        };

        Ok(Box::pin(stream))
    }
}
