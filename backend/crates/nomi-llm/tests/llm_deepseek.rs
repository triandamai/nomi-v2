use nomi_llm::deepseek::DeepSeekProvider;
use nomi_llm::{collect_stream, ContentBlock, LlmError, LlmMessage, LlmProvider, LlmRequest, LlmRole, StopReason, ToolDefinition};
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
        enable_reasoning: false,
    }
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
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(sse_body, "text/event-stream"))
        .mount(&server)
        .await;

    let provider = DeepSeekProvider::new(reqwest::Client::new(), "test-key".to_string(), "deepseek-flash".to_string(), server.uri());

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
        "data: {\"choices\":[{\"delta\":{\"tool_calls\":[{\"index\":0,\"function\":{\"arguments\":\"{\\\"city\\\":\\\"Paris\\\"}\"}}]},\"finish_reason\":null}]}\n\n",
        "data: {\"choices\":[{\"delta\":{},\"finish_reason\":\"tool_calls\"}]}\n\n",
        "data: [DONE]\n\n",
    );
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(sse_body, "text/event-stream"))
        .mount(&server)
        .await;

    let provider = DeepSeekProvider::new(reqwest::Client::new(), "test-key".to_string(), "deepseek-flash".to_string(), server.uri());

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
            thought_signature: None,
        }]
    );
    assert_eq!(response.stop_reason, StopReason::ToolUse);
}

#[tokio::test]
async fn enabling_reasoning_sends_thinking_enabled() {
    let server = MockServer::start().await;
    let sse_body = concat!(
        "data: {\"choices\":[{\"delta\":{\"content\":\"hi\"},\"finish_reason\":null}]}\n\n",
        "data: {\"choices\":[{\"delta\":{},\"finish_reason\":\"stop\"}]}\n\n",
        "data: [DONE]\n\n",
    );
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .and(body_partial_json(json!({"thinking": {"type": "enabled"}})))
        .respond_with(ResponseTemplate::new(200).set_body_raw(sse_body, "text/event-stream"))
        .mount(&server)
        .await;

    let provider = DeepSeekProvider::new(reqwest::Client::new(), "test-key".to_string(), "deepseek-flash".to_string(), server.uri());

    let mut request = text_request();
    request.enable_reasoning = true;

    // If "thinking": {"type": "enabled"} wasn't sent, wiremock's mock never matches and this 404s.
    let stream = provider.complete_stream(request).await.unwrap();
    collect_stream(stream).await.unwrap();
}

#[tokio::test]
async fn disabling_reasoning_sends_thinking_disabled() {
    let server = MockServer::start().await;
    let sse_body = concat!(
        "data: {\"choices\":[{\"delta\":{\"content\":\"hi\"},\"finish_reason\":null}]}\n\n",
        "data: {\"choices\":[{\"delta\":{},\"finish_reason\":\"stop\"}]}\n\n",
        "data: [DONE]\n\n",
    );
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .and(body_partial_json(json!({"thinking": {"type": "disabled"}})))
        .respond_with(ResponseTemplate::new(200).set_body_raw(sse_body, "text/event-stream"))
        .mount(&server)
        .await;

    let provider = DeepSeekProvider::new(reqwest::Client::new(), "test-key".to_string(), "deepseek-flash".to_string(), server.uri());

    // enable_reasoning is false by default in text_request() — must still explicitly disable,
    // since DeepSeek defaults to reasoning-on otherwise.
    let stream = provider.complete_stream(text_request()).await.unwrap();
    collect_stream(stream).await.unwrap();
}

#[tokio::test]
async fn a_reasoning_content_delta_is_captured_as_a_thinking_block_before_the_reply_text() {
    let server = MockServer::start().await;
    let sse_body = concat!(
        "data: {\"choices\":[{\"delta\":{\"reasoning_content\":\"working it out\"},\"finish_reason\":null}]}\n\n",
        "data: {\"choices\":[{\"delta\":{\"content\":\"hi\"},\"finish_reason\":null}]}\n\n",
        "data: {\"choices\":[{\"delta\":{},\"finish_reason\":\"stop\"}]}\n\n",
        "data: [DONE]\n\n",
    );
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(sse_body, "text/event-stream"))
        .mount(&server)
        .await;

    let provider = DeepSeekProvider::new(reqwest::Client::new(), "test-key".to_string(), "deepseek-flash".to_string(), server.uri());

    let mut request = text_request();
    request.enable_reasoning = true;

    let stream = provider.complete_stream(request).await.unwrap();
    let response = collect_stream(stream).await.unwrap();

    assert_eq!(
        response.content,
        vec![
            ContentBlock::Thinking { text: "working it out".to_string(), signature: None },
            ContentBlock::Text { text: "hi".to_string() },
        ]
    );
}

#[tokio::test]
async fn non_success_status_is_returned_before_any_stream_is_produced() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(401).set_body_string("unauthorized"))
        .mount(&server)
        .await;

    let provider = DeepSeekProvider::new(reqwest::Client::new(), "test-key".to_string(), "deepseek-flash".to_string(), server.uri());

    let result = provider.complete_stream(text_request()).await;
    assert!(matches!(result, Err(LlmError::ProviderError(_))));
}

#[tokio::test]
async fn list_models_parses_the_data_array() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/models"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "object": "list",
            "data": [
                {"id": "deepseek-flash", "object": "model", "owned_by": "deepseek"},
                {"id": "deepseek-v4-pro", "object": "model", "owned_by": "deepseek"},
            ]
        })))
        .mount(&server)
        .await;

    let models = nomi_llm::deepseek::list_models(&reqwest::Client::new(), "test-key", &server.uri()).await.unwrap();

    assert_eq!(models.len(), 2);
    assert_eq!(models[0].id, "deepseek-flash");
    assert_eq!(models[1].id, "deepseek-v4-pro");
}
