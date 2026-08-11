use nomi_orchestrator::llm::anthropic::AnthropicProvider;
use nomi_orchestrator::llm::{collect_stream, ContentBlock, LlmError, LlmMessage, LlmProvider, LlmRequest, LlmRole, PartialBlock, StopReason, StreamEvent, ToolDefinition};
use serde_json::json;
use wiremock::matchers::{body_partial_json, header, method, path};
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
        .and(path("/v1/messages"))
        .and(header("x-api-key", "test-key"))
        .and(header("anthropic-version", "2023-06-01"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "content": [{"type": "text", "text": "hi there"}],
            "stop_reason": "end_turn",
            "usage": {"input_tokens": 10, "output_tokens": 5}
        })))
        .mount(&server)
        .await;

    let provider = AnthropicProvider::new(
        reqwest::Client::new(),
        "test-key".to_string(),
        "claude-haiku-4-5".to_string(),
        server.uri(),
    );

    let response = provider.complete(text_request()).await.unwrap();

    assert_eq!(response.content, vec![ContentBlock::Text { text: "hi there".to_string() }]);
    assert_eq!(response.stop_reason, StopReason::EndTurn);
    assert_eq!(response.input_tokens, 10);
    assert_eq!(response.output_tokens, 5);
}

#[tokio::test]
async fn tool_use_reply_parses_into_a_tool_use_block() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "content": [{"type": "tool_use", "id": "toolu_1", "name": "get_weather", "input": {"city": "Paris"}}],
            "stop_reason": "tool_use",
            "usage": {"input_tokens": 20, "output_tokens": 8}
        })))
        .mount(&server)
        .await;

    let provider = AnthropicProvider::new(
        reqwest::Client::new(),
        "test-key".to_string(),
        "claude-haiku-4-5".to_string(),
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
            id: "toolu_1".to_string(),
            name: "get_weather".to_string(),
            input: json!({"city": "Paris"}),
        }]
    );
    assert_eq!(response.stop_reason, StopReason::ToolUse);
}

#[tokio::test]
async fn tool_result_in_request_is_mapped_into_anthropic_wire_format() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .and(body_partial_json(json!({
            "messages": [
                {"role": "user", "content": [{"type": "tool_result", "tool_use_id": "toolu_1", "content": "sunny", "is_error": false}]}
            ]
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "content": [{"type": "text", "text": "It's sunny in Paris."}],
            "stop_reason": "end_turn",
            "usage": {"input_tokens": 5, "output_tokens": 5}
        })))
        .mount(&server)
        .await;

    let provider = AnthropicProvider::new(
        reqwest::Client::new(),
        "test-key".to_string(),
        "claude-haiku-4-5".to_string(),
        server.uri(),
    );

    let request = LlmRequest {
        system: None,
        messages: vec![LlmMessage {
            role: LlmRole::User,
            content: vec![ContentBlock::ToolResult {
                tool_use_id: "toolu_1".to_string(),
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
        .and(path("/v1/messages"))
        .respond_with(ResponseTemplate::new(400).set_body_string("bad request"))
        .mount(&server)
        .await;

    let provider = AnthropicProvider::new(
        reqwest::Client::new(),
        "test-key".to_string(),
        "claude-haiku-4-5".to_string(),
        server.uri(),
    );

    let result = provider.complete(text_request()).await;
    assert!(matches!(result, Err(LlmError::ProviderError(_))));
}

#[tokio::test]
async fn streamed_text_reply_collects_into_the_same_text_block() {
    let server = MockServer::start().await;
    let sse_body = concat!(
        "event: message_start\n",
        "data: {\"type\":\"message_start\",\"message\":{\"usage\":{\"input_tokens\":10}}}\n\n",
        "event: content_block_start\n",
        "data: {\"type\":\"content_block_start\",\"index\":0,\"content_block\":{\"type\":\"text\",\"text\":\"\"}}\n\n",
        "event: content_block_delta\n",
        "data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\"hi \"}}\n\n",
        "event: content_block_delta\n",
        "data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\"there\"}}\n\n",
        "event: content_block_stop\n",
        "data: {\"type\":\"content_block_stop\",\"index\":0}\n\n",
        "event: message_delta\n",
        "data: {\"type\":\"message_delta\",\"delta\":{\"stop_reason\":\"end_turn\"},\"usage\":{\"output_tokens\":5}}\n\n",
        "event: message_stop\n",
        "data: {\"type\":\"message_stop\"}\n\n",
    );
    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .and(body_partial_json(json!({"stream": true})))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_raw(sse_body, "text/event-stream"),
        )
        .mount(&server)
        .await;

    let provider = AnthropicProvider::new(
        reqwest::Client::new(),
        "test-key".to_string(),
        "claude-haiku-4-5".to_string(),
        server.uri(),
    );

    let stream = provider.complete_stream(text_request()).await.unwrap();
    let response = collect_stream(stream).await.unwrap();

    assert_eq!(response.content, vec![ContentBlock::Text { text: "hi there".to_string() }]);
    assert_eq!(response.stop_reason, StopReason::EndTurn);
    assert_eq!(response.input_tokens, 10);
    assert_eq!(response.output_tokens, 5);
}

#[tokio::test]
async fn streamed_tool_use_reply_reassembles_the_fragmented_input_json() {
    let server = MockServer::start().await;
    let sse_body = concat!(
        "event: message_start\n",
        "data: {\"type\":\"message_start\",\"message\":{\"usage\":{\"input_tokens\":20}}}\n\n",
        "event: content_block_start\n",
        "data: {\"type\":\"content_block_start\",\"index\":0,\"content_block\":{\"type\":\"tool_use\",\"id\":\"toolu_1\",\"name\":\"get_weather\"}}\n\n",
        "event: content_block_delta\n",
        "data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"input_json_delta\",\"partial_json\":\"{\\\"city\\\":\"}}\n\n",
        "event: content_block_delta\n",
        "data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"input_json_delta\",\"partial_json\":\"\\\"Paris\\\"}\"}}\n\n",
        "event: content_block_stop\n",
        "data: {\"type\":\"content_block_stop\",\"index\":0}\n\n",
        "event: message_delta\n",
        "data: {\"type\":\"message_delta\",\"delta\":{\"stop_reason\":\"tool_use\"},\"usage\":{\"output_tokens\":8}}\n\n",
        "event: message_stop\n",
        "data: {\"type\":\"message_stop\"}\n\n",
    );
    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(sse_body, "text/event-stream"))
        .mount(&server)
        .await;

    let provider = AnthropicProvider::new(
        reqwest::Client::new(),
        "test-key".to_string(),
        "claude-haiku-4-5".to_string(),
        server.uri(),
    );

    let mut request = text_request();
    request.tools = vec![ToolDefinition {
        name: "get_weather".to_string(),
        description: "Get the weather".to_string(),
        input_schema: json!({"type": "object", "properties": {"city": {"type": "string"}}}),
    }];

    let stream = provider.complete_stream(request).await.unwrap();
    let response = collect_stream(stream).await.unwrap();

    assert_eq!(
        response.content,
        vec![ContentBlock::ToolUse {
            id: "toolu_1".to_string(),
            name: "get_weather".to_string(),
            input: json!({"city": "Paris"}),
        }]
    );
    assert_eq!(response.stop_reason, StopReason::ToolUse);
}

#[tokio::test]
async fn a_mid_stream_error_event_surfaces_as_an_err() {
    let server = MockServer::start().await;
    let sse_body = concat!(
        "event: content_block_start\n",
        "data: {\"type\":\"content_block_start\",\"index\":0,\"content_block\":{\"type\":\"text\",\"text\":\"\"}}\n\n",
        "event: error\n",
        "data: {\"type\":\"error\",\"error\":{\"type\":\"overloaded_error\",\"message\":\"overloaded\"}}\n\n",
    );
    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(sse_body, "text/event-stream"))
        .mount(&server)
        .await;

    let provider = AnthropicProvider::new(
        reqwest::Client::new(),
        "test-key".to_string(),
        "claude-haiku-4-5".to_string(),
        server.uri(),
    );

    let stream = provider.complete_stream(text_request()).await.unwrap();
    let result = collect_stream(stream).await;
    assert!(matches!(result, Err(LlmError::ProviderError(_))));
}

#[tokio::test]
async fn non_success_status_is_returned_before_any_stream_is_produced() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .respond_with(ResponseTemplate::new(400).set_body_string("bad request"))
        .mount(&server)
        .await;

    let provider = AnthropicProvider::new(
        reqwest::Client::new(),
        "test-key".to_string(),
        "claude-haiku-4-5".to_string(),
        server.uri(),
    );

    let result = provider.complete_stream(text_request()).await;
    assert!(matches!(result, Err(LlmError::ProviderError(_))));
}
