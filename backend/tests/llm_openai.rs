use nomi_orchestrator::llm::openai::OpenAiProvider;
use nomi_orchestrator::llm::{collect_stream, ContentBlock, LlmError, LlmMessage, LlmProvider, LlmRequest, LlmRole, PartialBlock, StopReason, StreamEvent, ToolDefinition};
use serde_json::json;
use wiremock::matchers::{body_partial_json, method, path};
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
        .and(path("/v1/chat/completions"))
        .and(body_partial_json(json!({
            "messages": [
                {"role": "system", "content": "be helpful"},
                {"role": "user", "content": "hello"}
            ]
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "choices": [{"message": {"content": "hi there"}, "finish_reason": "stop"}],
            "usage": {"prompt_tokens": 10, "completion_tokens": 5}
        })))
        .mount(&server)
        .await;

    let provider = OpenAiProvider::new(
        reqwest::Client::new(),
        "test-key".to_string(),
        "gpt-4o".to_string(),
        server.uri(),
    );

    let response = provider.complete(text_request()).await.unwrap();

    assert_eq!(response.content, vec![ContentBlock::Text { text: "hi there".to_string() }]);
    assert_eq!(response.stop_reason, StopReason::EndTurn);
    assert_eq!(response.input_tokens, 10);
    assert_eq!(response.output_tokens, 5);
}

#[tokio::test]
async fn tool_call_reply_parses_into_a_tool_use_block() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "choices": [{
                "message": {
                    "content": null,
                    "tool_calls": [{
                        "id": "call_1",
                        "type": "function",
                        "function": {"name": "get_weather", "arguments": "{\"city\":\"Paris\"}"}
                    }]
                },
                "finish_reason": "tool_calls"
            }],
            "usage": {"prompt_tokens": 20, "completion_tokens": 8}
        })))
        .mount(&server)
        .await;

    let provider = OpenAiProvider::new(
        reqwest::Client::new(),
        "test-key".to_string(),
        "gpt-4o".to_string(),
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
            id: "call_1".to_string(),
            name: "get_weather".to_string(),
            input: json!({"city": "Paris"}),
        }]
    );
    assert_eq!(response.stop_reason, StopReason::ToolUse);
}

#[tokio::test]
async fn tool_result_in_request_becomes_a_separate_tool_role_message() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .and(body_partial_json(json!({
            "messages": [
                {"role": "tool", "tool_call_id": "call_1", "content": "sunny"}
            ]
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "choices": [{"message": {"content": "It's sunny in Paris."}, "finish_reason": "stop"}],
            "usage": {"prompt_tokens": 5, "completion_tokens": 5}
        })))
        .mount(&server)
        .await;

    let provider = OpenAiProvider::new(
        reqwest::Client::new(),
        "test-key".to_string(),
        "gpt-4o".to_string(),
        server.uri(),
    );

    let request = LlmRequest {
        system: None,
        messages: vec![LlmMessage {
            role: LlmRole::User,
            content: vec![ContentBlock::ToolResult {
                tool_use_id: "call_1".to_string(),
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
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(401).set_body_string("unauthorized"))
        .mount(&server)
        .await;

    let provider = OpenAiProvider::new(
        reqwest::Client::new(),
        "test-key".to_string(),
        "gpt-4o".to_string(),
        server.uri(),
    );

    let result = provider.complete(text_request()).await;
    assert!(matches!(result, Err(LlmError::ProviderError(_))));
}

#[tokio::test]
async fn streamed_text_reply_collects_into_the_same_text_block() {
    let server = MockServer::start().await;
    let sse_body = concat!(
        "data: {\"choices\":[{\"delta\":{\"content\":\"hi \"},\"finish_reason\":null}]}\n\n",
        "data: {\"choices\":[{\"delta\":{\"content\":\"there\"},\"finish_reason\":null}]}\n\n",
        "data: {\"choices\":[{\"delta\":{},\"finish_reason\":\"stop\"}]}\n\n",
        "data: {\"choices\":[],\"usage\":{\"prompt_tokens\":10,\"completion_tokens\":5}}\n\n",
        "data: [DONE]\n\n",
    );
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .and(body_partial_json(json!({"stream": true, "stream_options": {"include_usage": true}})))
        .respond_with(ResponseTemplate::new(200).set_body_raw(sse_body, "text/event-stream"))
        .mount(&server)
        .await;

    let provider = OpenAiProvider::new(
        reqwest::Client::new(),
        "test-key".to_string(),
        "gpt-4o".to_string(),
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
async fn streamed_tool_call_reassembles_fragmented_arguments_by_index() {
    let server = MockServer::start().await;
    let sse_body = concat!(
        "data: {\"choices\":[{\"delta\":{\"tool_calls\":[{\"index\":0,\"id\":\"call_1\",\"function\":{\"name\":\"get_weather\",\"arguments\":\"\"}}]},\"finish_reason\":null}]}\n\n",
        "data: {\"choices\":[{\"delta\":{\"tool_calls\":[{\"index\":0,\"function\":{\"arguments\":\"{\\\"city\\\":\"}}]},\"finish_reason\":null}]}\n\n",
        "data: {\"choices\":[{\"delta\":{\"tool_calls\":[{\"index\":0,\"function\":{\"arguments\":\"\\\"Paris\\\"}\"}}]},\"finish_reason\":null}]}\n\n",
        "data: {\"choices\":[{\"delta\":{},\"finish_reason\":\"tool_calls\"}]}\n\n",
        "data: {\"choices\":[],\"usage\":{\"prompt_tokens\":20,\"completion_tokens\":8}}\n\n",
        "data: [DONE]\n\n",
    );
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(sse_body, "text/event-stream"))
        .mount(&server)
        .await;

    let provider = OpenAiProvider::new(
        reqwest::Client::new(),
        "test-key".to_string(),
        "gpt-4o".to_string(),
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
            id: "call_1".to_string(),
            name: "get_weather".to_string(),
            input: json!({"city": "Paris"}),
        }]
    );
    assert_eq!(response.stop_reason, StopReason::ToolUse);
}

#[tokio::test]
async fn non_success_status_is_returned_before_any_stream_is_produced() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(401).set_body_string("unauthorized"))
        .mount(&server)
        .await;

    let provider = OpenAiProvider::new(
        reqwest::Client::new(),
        "test-key".to_string(),
        "gpt-4o".to_string(),
        server.uri(),
    );

    let result = provider.complete_stream(text_request()).await;
    assert!(matches!(result, Err(LlmError::ProviderError(_))));
}
