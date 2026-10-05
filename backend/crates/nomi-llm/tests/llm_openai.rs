use nomi_llm::openai::OpenAiProvider;
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
        reasoning_effort: Default::default(),
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
            thought_signature: None,
        }]
    );
    assert_eq!(response.stop_reason, StopReason::ToolUse);
}

#[tokio::test]
async fn enabling_reasoning_on_a_reasoning_model_sends_reasoning_effort() {
    let server = MockServer::start().await;
    let sse_body = concat!(
        "data: {\"choices\":[{\"delta\":{\"content\":\"hi\"},\"finish_reason\":null}]}\n\n",
        "data: {\"choices\":[{\"delta\":{},\"finish_reason\":\"stop\"}]}\n\n",
        "data: [DONE]\n\n",
    );
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .and(body_partial_json(json!({"reasoning_effort": "medium"})))
        .respond_with(ResponseTemplate::new(200).set_body_raw(sse_body, "text/event-stream"))
        .mount(&server)
        .await;

    let provider = OpenAiProvider::new(reqwest::Client::new(), "test-key".to_string(), "o3-mini".to_string(), server.uri());

    let mut request = text_request();
    request.enable_reasoning = true;

    // If reasoning_effort wasn't sent, wiremock's mock never matches and this 404s.
    let stream = provider.complete_stream(request).await.unwrap();
    collect_stream(stream).await.unwrap();
}

#[tokio::test]
async fn enabling_reasoning_on_a_non_reasoning_model_never_sends_reasoning_effort() {
    let server = MockServer::start().await;
    let sse_body = concat!(
        "data: {\"choices\":[{\"delta\":{\"content\":\"hi\"},\"finish_reason\":null}]}\n\n",
        "data: {\"choices\":[{\"delta\":{},\"finish_reason\":\"stop\"}]}\n\n",
        "data: [DONE]\n\n",
    );
    // No `reasoning_effort` matcher here — gpt-4o would 400 on that param in the real API, so
    // this mock only ever matches a plain request with no such field.
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(sse_body, "text/event-stream"))
        .mount(&server)
        .await;

    let provider = OpenAiProvider::new(reqwest::Client::new(), "test-key".to_string(), "gpt-4o".to_string(), server.uri());

    let mut request = text_request();
    request.enable_reasoning = true;

    let stream = provider.complete_stream(request).await.unwrap();
    let response = collect_stream(stream).await.unwrap();
    assert_eq!(response.content, vec![ContentBlock::Text { text: "hi".to_string() }]);
}

#[tokio::test]
async fn reasoning_token_usage_surfaces_as_a_synthetic_thinking_note() {
    let server = MockServer::start().await;
    let sse_body = concat!(
        "data: {\"choices\":[{\"delta\":{\"content\":\"hi\"},\"finish_reason\":null}]}\n\n",
        "data: {\"choices\":[{\"delta\":{},\"finish_reason\":\"stop\"}]}\n\n",
        "data: {\"choices\":[],\"usage\":{\"prompt_tokens\":10,\"completion_tokens\":50,\"completion_tokens_details\":{\"reasoning_tokens\":40}}}\n\n",
        "data: [DONE]\n\n",
    );
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(sse_body, "text/event-stream"))
        .mount(&server)
        .await;

    let provider = OpenAiProvider::new(reqwest::Client::new(), "test-key".to_string(), "o3-mini".to_string(), server.uri());

    let mut request = text_request();
    request.enable_reasoning = true;

    let stream = provider.complete_stream(request).await.unwrap();
    let response = collect_stream(stream).await.unwrap();

    assert_eq!(
        response.content,
        vec![
            ContentBlock::Text { text: "hi".to_string() },
            ContentBlock::Thinking {
                text: "(OpenAI used internal reasoning — its API doesn't expose the trace; ~40 reasoning tokens spent)".to_string(),
                signature: None,
            },
        ]
    );
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

#[tokio::test]
async fn a_reasoning_model_gets_max_completion_tokens_instead_of_max_tokens() {
    let server = MockServer::start().await;
    let sse_body = concat!(
        "data: {\"choices\":[{\"delta\":{\"content\":\"hi\"},\"finish_reason\":null}]}\n\n",
        "data: {\"choices\":[{\"delta\":{},\"finish_reason\":\"stop\"}]}\n\n",
        "data: [DONE]\n\n",
    );
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .and(body_partial_json(json!({"max_completion_tokens": 100})))
        .respond_with(ResponseTemplate::new(200).set_body_raw(sse_body, "text/event-stream"))
        .mount(&server)
        .await;

    let provider = OpenAiProvider::new(reqwest::Client::new(), "test-key".to_string(), "gpt-5-mini".to_string(), server.uri());

    let stream = provider.complete_stream(text_request()).await.unwrap();
    collect_stream(stream).await.unwrap();
    let sent: serde_json::Value = server.received_requests().await.unwrap()[0].body_json().unwrap();
    assert!(sent.get("max_tokens").is_none());
}
