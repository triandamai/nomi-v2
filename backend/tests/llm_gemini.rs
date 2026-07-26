use nomi_orchestrator::llm::gemini::GeminiProvider;
use nomi_orchestrator::llm::{ContentBlock, LlmError, LlmMessage, LlmProvider, LlmRequest, LlmRole, StopReason, ToolDefinition};
use serde_json::json;
use wiremock::matchers::{body_partial_json, method, path_regex};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn text_request() -> LlmRequest {
    LlmRequest {
        system: Some("be helpful".to_string()),
        messages: vec![LlmMessage {
            role: LlmRole::User,
            content: vec![ContentBlock::Text { text: "hello".to_string() }],
        }],
        tools: vec![],
        max_tokens: 100,
    }
}

#[tokio::test]
async fn text_only_reply_parses_into_a_text_block() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path_regex(r"^/v1beta/models/.*:generateContent$"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "candidates": [{
                "content": {"parts": [{"text": "hi there"}]},
                "finishReason": "STOP"
            }],
            "usageMetadata": {"promptTokenCount": 10, "candidatesTokenCount": 5}
        })))
        .mount(&server)
        .await;

    let provider = GeminiProvider::new(
        reqwest::Client::new(),
        "test-key".to_string(),
        "gemini-2.0-flash".to_string(),
        server.uri(),
    );

    let response = provider.complete(text_request()).await.unwrap();

    assert_eq!(response.content, vec![ContentBlock::Text { text: "hi there".to_string() }]);
    assert_eq!(response.stop_reason, StopReason::EndTurn);
    assert_eq!(response.input_tokens, 10);
    assert_eq!(response.output_tokens, 5);
}

#[tokio::test]
async fn function_call_reply_parses_into_a_tool_use_block() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path_regex(r"^/v1beta/models/.*:generateContent$"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "candidates": [{
                "content": {"parts": [{"functionCall": {"name": "get_weather", "args": {"city": "Paris"}}}]},
                "finishReason": "STOP"
            }],
            "usageMetadata": {"promptTokenCount": 20, "candidatesTokenCount": 8}
        })))
        .mount(&server)
        .await;

    let provider = GeminiProvider::new(
        reqwest::Client::new(),
        "test-key".to_string(),
        "gemini-2.0-flash".to_string(),
        server.uri(),
    );

    let mut request = text_request();
    request.tools = vec![ToolDefinition {
        name: "get_weather".to_string(),
        description: "Get the weather".to_string(),
        input_schema: json!({"type": "object", "properties": {"city": {"type": "string"}}}),
    }];

    let response = provider.complete(request).await.unwrap();

    assert_eq!(
        response.content,
        vec![ContentBlock::ToolUse {
            id: "get_weather".to_string(),
            name: "get_weather".to_string(),
            input: json!({"city": "Paris"}),
        }]
    );
    assert_eq!(response.stop_reason, StopReason::ToolUse);
}

#[tokio::test]
async fn tool_result_in_request_is_mapped_into_a_function_response_part() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path_regex(r"^/v1beta/models/.*:generateContent$"))
        .and(body_partial_json(json!({
            "contents": [
                {"role": "user", "parts": [{"functionResponse": {"name": "get_weather", "response": {"content": "sunny"}}}]}
            ]
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "candidates": [{
                "content": {"parts": [{"text": "It's sunny in Paris."}]},
                "finishReason": "STOP"
            }],
            "usageMetadata": {"promptTokenCount": 5, "candidatesTokenCount": 5}
        })))
        .mount(&server)
        .await;

    let provider = GeminiProvider::new(
        reqwest::Client::new(),
        "test-key".to_string(),
        "gemini-2.0-flash".to_string(),
        server.uri(),
    );

    let request = LlmRequest {
        system: None,
        messages: vec![LlmMessage {
            role: LlmRole::User,
            content: vec![ContentBlock::ToolResult {
                tool_use_id: "get_weather".to_string(),
                content: "sunny".to_string(),
                is_error: false,
            }],
        }],
        tools: vec![],
        max_tokens: 100,
    };

    let response = provider.complete(request).await.unwrap();
    assert_eq!(response.content, vec![ContentBlock::Text { text: "It's sunny in Paris.".to_string() }]);
}

#[tokio::test]
async fn non_success_status_becomes_a_provider_error() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path_regex(r"^/v1beta/models/.*:generateContent$"))
        .respond_with(ResponseTemplate::new(403).set_body_string("forbidden"))
        .mount(&server)
        .await;

    let provider = GeminiProvider::new(
        reqwest::Client::new(),
        "test-key".to_string(),
        "gemini-2.0-flash".to_string(),
        server.uri(),
    );

    let result = provider.complete(text_request()).await;
    assert!(matches!(result, Err(LlmError::ProviderError(_))));
}
