use serde_json::Value;
use std::pin::Pin;
use futures_core::Stream;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum LlmRole {
    User,
    Assistant,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ContentBlock {
    Text { text: String },
    /// `thought_signature` is Gemini-specific: its "thinking" models attach an opaque signature
    /// to a functionCall part, which must be echoed back verbatim when that tool call is
    /// replayed into a later turn, or Gemini rejects the request. Always `None` for other
    /// providers.
    ToolUse { id: String, name: String, input: Value, thought_signature: Option<String> },
    ToolResult { tool_use_id: String, content: String, is_error: bool },
    /// A provider's own reasoning/thinking trace, kept separate from `Text` so callers can show
    /// it distinctly (and so it's never mistaken for the actual reply). `signature` is
    /// Anthropic-specific: extended thinking blocks carry a signature that must be echoed back
    /// verbatim when replayed into a later turn in the same tool loop, or the API rejects the
    /// request. `None` for providers with no such requirement (Gemini's thought summaries,
    /// OpenRouter's unified `reasoning` field) or when a provider can only report that reasoning
    /// happened without exposing its text (see `openai.rs`).
    Thinking { text: String, signature: Option<String> },
    /// A file the model looks at itself: an image, a PDF, audio or video, base64-encoded.
    /// Only ever sent in a user message. Providers that can't take `media_type` get a short
    /// text note instead (see `media::MediaRouter`, which picks a model that can).
    Media { media_type: String, data: String, name: String },
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct LlmMessage {
    pub role: LlmRole,
    pub content: Vec<ContentBlock>,
}

#[derive(Debug, Clone)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

#[derive(Debug, Clone)]
pub struct LlmRequest {
    pub system: Option<String>,
    pub messages: Vec<LlmMessage>,
    pub tools: Vec<ToolDefinition>,
    pub max_tokens: u32,
    /// Asks the provider to produce a reasoning/thinking trace alongside its reply, when it
    /// supports one (see each provider's `build_body`). Left `false` for one-shot utility calls
    /// (title generation, memory extraction, BYOK validation, supervisor phrasing) where the
    /// extra latency/cost isn't worth it — only `run_agent_turn`'s main reply request sets it.
    pub enable_reasoning: bool,
    /// How hard to think when `enable_reasoning` is on (the chat's thinking level). Providers map
    /// it to their own knob: a thinking budget, or a reasoning-effort setting.
    pub reasoning_effort: ReasoningEffort,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReasoningEffort {
    Low,
    #[default]
    Medium,
    High,
}

impl ReasoningEffort {
    pub fn as_str(self) -> &'static str {
        match self {
            ReasoningEffort::Low => "low",
            ReasoningEffort::Medium => "medium",
            ReasoningEffort::High => "high",
        }
    }

    /// Tokens thinking may use at this level, given on top of the reply's own `max_tokens` so
    /// thinking never eats the room the answer needs. Kept small: thinking is shown in chat
    /// and should stay short and on point.
    pub fn budget_tokens(self) -> u32 {
        match self {
            ReasoningEffort::Low => 1024,
            ReasoningEffort::Medium => 1024,
            ReasoningEffort::High => 2048,
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum StopReason {
    EndTurn,
    ToolUse,
    MaxTokens,
    Other(String),
}

#[derive(Debug, Clone)]
pub struct LlmResponse {
    pub content: Vec<ContentBlock>,
    pub stop_reason: StopReason,
    pub input_tokens: u32,
    pub output_tokens: u32,
}

#[derive(Debug, thiserror::Error)]
pub enum LlmError {
    // Request errors quote the URL and providers echo request details, so secrets are stripped
    // from everything shown or logged (see crate::redact).
    #[error("http request failed: {}", crate::redact::redacted(.0.to_string()))]
    Http(#[from] reqwest::Error),
    #[error("provider returned an error response: {}", crate::redact::redacted(.0))]
    ProviderError(String),
    #[error("failed to parse provider response: {}", crate::redact::redacted(.0))]
    ParseError(String),
    /// The person has used their plan's monthly allowance on Nomi's models and has no key of
    /// their own to fall back on.
    #[error("monthly token quota used up")]
    QuotaExceeded,
}

impl LlmError {
    /// Whether trying the same request again later may work: the provider couldn't be reached,
    /// timed out, was rate limited or overloaded, or failed on its side (5xx). Bad requests,
    /// rejected keys and a used-up allowance are final.
    pub fn is_retryable(&self) -> bool {
        match self {
            LlmError::Http(e) => {
                e.is_connect() || e.is_timeout() || e.is_request() || e.is_body() || e.status().is_some_and(|s| retryable_status(s.as_u16()))
            }
            LlmError::ProviderError(text) => {
                let lower = text.to_lowercase();
                if lower.contains("overloaded") || lower.contains("rate limit") || lower.contains("unavailable") || lower.contains("try again") {
                    return true;
                }
                // "<provider> returned 503 Service Unavailable: ..."
                lower
                    .split("returned ")
                    .nth(1)
                    .and_then(|rest| rest.get(..3))
                    .and_then(|code| code.parse::<u16>().ok())
                    .is_some_and(retryable_status)
            }
            LlmError::ParseError(_) | LlmError::QuotaExceeded => false,
        }
    }
}

fn retryable_status(status: u16) -> bool {
    matches!(status, 408 | 409 | 425 | 429) || (500..=599).contains(&status)
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum PartialBlock {
    Text,
    Thinking,
    ToolUse { id: String, name: String, thought_signature: Option<String> },
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum StreamEvent {
    ContentBlockStart { index: usize, block: PartialBlock },
    TextDelta { index: usize, text: String },
    ThinkingDelta { index: usize, text: String },
    /// Anthropic-only: arrives after a thinking block's text is complete, carrying the
    /// signature that must be echoed back verbatim if this block is replayed into a later turn.
    ThinkingSignature { index: usize, signature: String },
    ToolInputDelta { index: usize, partial_json: String },
    ContentBlockDone { index: usize },
    /// How much of the request's input the provider served from its prompt cache (billed at a
    /// fraction of the input price). Sent just before `Done` by providers that report it;
    /// `Done.input_tokens` always counts every input token, cached or not.
    CacheUsage { cached_input_tokens: u32 },
    Done { stop_reason: StopReason, input_tokens: u32, output_tokens: u32 },
}

pub type LlmEventStream = Pin<Box<dyn Stream<Item = Result<StreamEvent, LlmError>> + Send>>;

#[cfg(test)]
mod tests {
    #[test]
    fn retryable_errors_are_the_temporary_ones() {
        use super::LlmError;
        assert!(LlmError::ProviderError("gemini returned 503 Service Unavailable: busy".into()).is_retryable());
        assert!(LlmError::ProviderError("anthropic returned 429 Too Many Requests: slow down".into()).is_retryable());
        assert!(LlmError::ProviderError("anthropic stream error: {\"type\":\"overloaded_error\"}".into()).is_retryable());
        assert!(!LlmError::ProviderError("openai returned 401 Unauthorized: bad key".into()).is_retryable());
        assert!(!LlmError::ProviderError("openai returned 400 Bad Request: context too long".into()).is_retryable());
        assert!(!LlmError::QuotaExceeded.is_retryable());
    }

    use super::*;

    #[test]
    fn llm_request_holds_the_fields_it_was_constructed_with() {
        let request = LlmRequest {
            system: Some("be helpful".to_string()),
            messages: vec![LlmMessage {
                role: LlmRole::User,
                content: vec![ContentBlock::Text { text: "hi".to_string() }],
            }],
            tools: vec![],
            max_tokens: 1024,
            enable_reasoning: false,
            reasoning_effort: Default::default(),
        };

        assert_eq!(request.system, Some("be helpful".to_string()));
        assert_eq!(request.messages.len(), 1);
        assert_eq!(request.messages[0].role, LlmRole::User);
        assert_eq!(
            request.messages[0].content[0],
            ContentBlock::Text { text: "hi".to_string() }
        );
        assert_eq!(request.max_tokens, 1024);
    }

    #[test]
    fn stop_reason_variants_are_distinguishable() {
        assert_ne!(StopReason::EndTurn, StopReason::ToolUse);
        assert_eq!(
            StopReason::Other("weird".to_string()),
            StopReason::Other("weird".to_string())
        );
    }

    #[test]
    fn content_block_variants_carry_their_fields() {
        let tool_use = ContentBlock::ToolUse {
            id: "toolu_1".to_string(),
            name: "get_weather".to_string(),
            input: serde_json::json!({"city": "Paris"}),
            thought_signature: None,
        };
        match tool_use {
            ContentBlock::ToolUse { id, name, input, .. } => {
                assert_eq!(id, "toolu_1");
                assert_eq!(name, "get_weather");
                assert_eq!(input, serde_json::json!({"city": "Paris"}));
            }
            _ => panic!("expected ToolUse variant"),
        }
    }
}
