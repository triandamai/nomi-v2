use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LlmRole {
    User,
    Assistant,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ContentBlock {
    Text { text: String },
    ToolUse { id: String, name: String, input: Value },
    ToolResult { tool_use_id: String, content: String, is_error: bool },
}

#[derive(Debug, Clone)]
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
}

#[derive(Debug, Clone, PartialEq)]
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
    #[error("http request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("provider returned an error response: {0}")]
    ProviderError(String),
    #[error("failed to parse provider response: {0}")]
    ParseError(String),
}

#[cfg(test)]
mod tests {
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
        };
        match tool_use {
            ContentBlock::ToolUse { id, name, input } => {
                assert_eq!(id, "toolu_1");
                assert_eq!(name, "get_weather");
                assert_eq!(input, serde_json::json!({"city": "Paris"}));
            }
            _ => panic!("expected ToolUse variant"),
        }
    }
}
