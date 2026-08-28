use nomi_llm::gemini::GeminiProvider;
use nomi_llm::{collect_stream, ContentBlock, LlmError, LlmMessage, LlmProvider, LlmRequest, LlmRole, StopReason, ToolDefinition};
use serde_json::json;
use wiremock::matchers::{body_string_contains, method, path_regex};
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
async fn streamed_text_reply_collects_into_the_same_text_block() {
    let server = MockServer::start().await;
    let sse_body = concat!(
        "data: {\"candidates\":[{\"content\":{\"parts\":[{\"text\":\"hi \"}]}}]}\n\n",
        "data: {\"candidates\":[{\"content\":{\"parts\":[{\"text\":\"there\"}]},\"finishReason\":\"STOP\"}],\"usageMetadata\":{\"promptTokenCount\":10,\"candidatesTokenCount\":5}}\n\n",
    );
    Mock::given(method("POST"))
        .and(path_regex(r"^/v1beta/models/.*:streamGenerateContent$"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(sse_body, "text/event-stream"))
        .mount(&server)
        .await;

    let provider = GeminiProvider::new(
        reqwest::Client::new(),
        "test-key".to_string(),
        "gemini-2.0-flash".to_string(),
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
async fn streamed_function_call_arrives_whole_in_one_chunk() {
    let server = MockServer::start().await;
    let sse_body = concat!(
        "data: {\"candidates\":[{\"content\":{\"parts\":[{\"functionCall\":{\"name\":\"get_weather\",\"args\":{\"city\":\"Paris\"}}}]},\"finishReason\":\"STOP\"}],\"usageMetadata\":{\"promptTokenCount\":20,\"candidatesTokenCount\":8}}\n\n",
    );
    Mock::given(method("POST"))
        .and(path_regex(r"^/v1beta/models/.*:streamGenerateContent$"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(sse_body, "text/event-stream"))
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

    let stream = provider.complete_stream(request).await.unwrap();
    let response = collect_stream(stream).await.unwrap();

    assert_eq!(
        response.content,
        vec![ContentBlock::ToolUse {
            id: "get_weather".to_string(),
            name: "get_weather".to_string(),
            input: json!({"city": "Paris"}),
            thought_signature: None,
        }]
    );
    assert_eq!(response.stop_reason, StopReason::ToolUse);
}

// Gemini's "thinking" models attach a thoughtSignature to functionCall parts and reject any
// later turn that replays that function call without echoing the signature back verbatim
// (see the 400 "Function call is missing a thought_signature" error this reproduces). This test
// proves the signature survives the parse.
#[tokio::test]
async fn a_streamed_function_call_captures_its_thought_signature() {
    let server = MockServer::start().await;
    let sse_body = concat!(
        "data: {\"candidates\":[{\"content\":{\"parts\":[{\"functionCall\":{\"name\":\"get_weather\",\"args\":{\"city\":\"Paris\"}},\"thoughtSignature\":\"sig-abc\"}]},\"finishReason\":\"STOP\"}],\"usageMetadata\":{\"promptTokenCount\":20,\"candidatesTokenCount\":8}}\n\n",
    );
    Mock::given(method("POST"))
        .and(path_regex(r"^/v1beta/models/.*:streamGenerateContent$"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(sse_body, "text/event-stream"))
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

    let stream = provider.complete_stream(request).await.unwrap();
    let response = collect_stream(stream).await.unwrap();

    assert_eq!(
        response.content,
        vec![ContentBlock::ToolUse {
            id: "get_weather".to_string(),
            name: "get_weather".to_string(),
            input: json!({"city": "Paris"}),
            thought_signature: Some("sig-abc".to_string()),
        }]
    );
}

// The other half of the round trip: when a ToolUse block carrying a thought_signature is
// replayed into a later turn (exactly what the tool-calling loop does), the outgoing request
// must include that signature on the corresponding functionCall part, or Gemini 400s.
#[tokio::test]
async fn a_replayed_tool_use_with_a_thought_signature_includes_it_on_the_request() {
    let server = MockServer::start().await;
    let sse_body = concat!(
        "data: {\"candidates\":[{\"content\":{\"parts\":[{\"text\":\"done\"}]},\"finishReason\":\"STOP\"}],\"usageMetadata\":{\"promptTokenCount\":5,\"candidatesTokenCount\":2}}\n\n",
    );
    Mock::given(method("POST"))
        .and(path_regex(r"^/v1beta/models/.*:streamGenerateContent$"))
        .and(body_string_contains("\"thoughtSignature\":\"sig-abc\""))
        .respond_with(ResponseTemplate::new(200).set_body_raw(sse_body, "text/event-stream"))
        .mount(&server)
        .await;

    let provider = GeminiProvider::new(
        reqwest::Client::new(),
        "test-key".to_string(),
        "gemini-2.0-flash".to_string(),
        server.uri(),
    );

    let mut request = text_request();
    request.messages.push(LlmMessage {
        role: LlmRole::Assistant,
        content: vec![ContentBlock::ToolUse {
            id: "get_weather".to_string(),
            name: "get_weather".to_string(),
            input: json!({"city": "Paris"}),
            thought_signature: Some("sig-abc".to_string()),
        }],
    });
    request.messages.push(LlmMessage {
        role: LlmRole::User,
        content: vec![ContentBlock::ToolResult {
            tool_use_id: "get_weather".to_string(),
            content: "72F and sunny".to_string(),
            is_error: false,
        }],
    });

    // If the outgoing request body doesn't contain the signature, wiremock's mock above never
    // matches and this request 404s — collect_stream then returns an Err, failing this test.
    let stream = provider.complete_stream(request).await.unwrap();
    let response = collect_stream(stream).await.unwrap();

    assert_eq!(response.content, vec![ContentBlock::Text { text: "done".to_string() }]);
}

#[tokio::test]
async fn non_success_status_is_returned_before_any_stream_is_produced() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path_regex(r"^/v1beta/models/.*:streamGenerateContent$"))
        .respond_with(ResponseTemplate::new(403).set_body_string("forbidden"))
        .mount(&server)
        .await;

    let provider = GeminiProvider::new(
        reqwest::Client::new(),
        "test-key".to_string(),
        "gemini-2.0-flash".to_string(),
        server.uri(),
    );

    let result = provider.complete_stream(text_request()).await;
    assert!(matches!(result, Err(LlmError::ProviderError(_))));
}
