//! Listing each provider's models, replaying a finished response as a stream, Gemini's schema
//! cleaning, and Gemini's File API for files too big to send inline.

use base64::Engine;
use nomi_llm::gemini::{clean_gemini_schema, GeminiProvider};
use nomi_llm::{collect_stream, list_provider_models, response_to_stream, ContentBlock, LlmError, LlmMessage, LlmProvider, LlmRequest, LlmResponse, LlmRole, ModelConfig, ProviderKind, StopReason};
use serde_json::json;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn config(provider: ProviderKind, base_url: Option<String>) -> ModelConfig {
    ModelConfig { provider, model_id: "m".into(), api_key: "key".into(), base_url, extra_headers: vec![("x-team".into(), "nomi".into())] }
}

#[tokio::test]
async fn every_provider_lists_its_models() {
    let server = MockServer::start().await;
    let list = |at: &str, body: serde_json::Value| Mock::given(method("GET")).and(path(at.to_string())).respond_with(ResponseTemplate::new(200).set_body_json(body));
    // OpenAI's own list and Anthropic's share /v1/models; tell them apart by header.
    Mock::given(method("GET")).and(path("/v1/models")).and(header("x-api-key", "key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"data": [{"id": "claude-x", "display_name": "Claude X"}, {"no": "id"}]})))
        .mount(&server).await;
    list("/v1/models", json!({"data": [{"id": "gpt-x"}]})).mount(&server).await;
    list("/models", json!({"data": [{"id": "deepseek-chat", "name": "DeepSeek Chat"}]})).mount(&server).await;
    list("/v1beta/models", json!({"models": [{"name": "models/gemini-x", "displayName": "Gemini X"}, {"name": "bare"}, {}]})).mount(&server).await;

    let ids = |models: Vec<nomi_llm::ModelSummary>| models.into_iter().map(|m| (m.id, m.label)).collect::<Vec<_>>();
    let http = reqwest::Client::new();
    assert_eq!(ids(list_provider_models(config(ProviderKind::Anthropic, Some(server.uri())), http.clone()).await.unwrap()), [("claude-x".into(), Some("Claude X".into()))]);
    assert_eq!(ids(list_provider_models(config(ProviderKind::OpenAi, Some(server.uri())), http.clone()).await.unwrap())[0].0, "gpt-x");
    assert_eq!(ids(list_provider_models(config(ProviderKind::DeepSeek, Some(server.uri())), http.clone()).await.unwrap())[0].0, "deepseek-chat");
    assert_eq!(ids(list_provider_models(config(ProviderKind::OpenRouter, Some(server.uri())), http.clone()).await.unwrap()), [("deepseek-chat".into(), Some("DeepSeek Chat".into()))]);
    assert_eq!(ids(list_provider_models(config(ProviderKind::OpenAiCompatible, Some(server.uri())), http.clone()).await.unwrap())[0].0, "deepseek-chat");
    assert_eq!(
        ids(list_provider_models(config(ProviderKind::Gemini, Some(server.uri())), http.clone()).await.unwrap()),
        [("gemini-x".into(), Some("Gemini X".into())), ("bare".into(), None)]
    );
    assert_eq!(ids(list_provider_models(config(ProviderKind::Fake, None), http).await.unwrap())[0].0, "fake-model");
    let sent = server.received_requests().await.unwrap();
    assert!(sent.iter().any(|r| r.headers.get("x-team").is_some()), "a custom provider's headers go along");
}

#[tokio::test]
async fn listing_reports_rejections_and_unreadable_answers() {
    for (kind, at, name) in [
        (ProviderKind::Anthropic, "/v1/models", "anthropic"),
        (ProviderKind::DeepSeek, "/models", "deepseek"),
        (ProviderKind::OpenRouter, "/models", "openrouter"),
        (ProviderKind::Gemini, "/v1beta/models", "gemini"),
        (ProviderKind::OpenAi, "/v1/models", "openai"),
    ] {
        let rejected = MockServer::start().await;
        Mock::given(method("GET")).and(path(at)).respond_with(ResponseTemplate::new(401).set_body_string("bad key")).mount(&rejected).await;
        match list_provider_models(config(kind, Some(rejected.uri())), reqwest::Client::new()).await {
            Err(LlmError::ProviderError(message)) => assert!(message.contains("401") && message.contains("bad key"), "{name}: {message}"),
            other => panic!("{name}: {other:?}"),
        }
        let empty = MockServer::start().await;
        Mock::given(method("GET")).and(path(at)).respond_with(ResponseTemplate::new(200).set_body_json(json!({"nothing": true}))).mount(&empty).await;
        assert!(list_provider_models(config(kind, Some(empty.uri())), reqwest::Client::new()).await.is_err(), "{name}");
        let garbage = MockServer::start().await;
        Mock::given(method("GET")).and(path(at)).respond_with(ResponseTemplate::new(200).set_body_string("<html>")).mount(&garbage).await;
        assert!(matches!(list_provider_models(config(kind, Some(garbage.uri())), reqwest::Client::new()).await, Err(LlmError::ParseError(_))), "{name}");
    }
}

#[tokio::test]
async fn a_finished_response_streams_back_as_the_same_response() {
    let response = LlmResponse {
        content: vec![
            ContentBlock::Thinking { text: "hmm".into(), signature: Some("sig".into()) },
            ContentBlock::Thinking { text: "more".into(), signature: None },
            ContentBlock::Text { text: "Let me check.".into() },
            ContentBlock::ToolUse { id: "t1".into(), name: "lookup".into(), input: json!({"q": "rent"}), thought_signature: Some("ts".into()) },
            ContentBlock::ToolResult { tool_use_id: "t1".into(), content: "ignored".into(), is_error: false },
        ],
        stop_reason: StopReason::ToolUse,
        input_tokens: 12,
        output_tokens: 7,
    };
    let collected = collect_stream(response_to_stream(response.clone())).await.unwrap();
    assert_eq!(collected.content, response.content[..4].to_vec());
    assert_eq!((collected.stop_reason, collected.input_tokens, collected.output_tokens), (StopReason::ToolUse, 12, 7));
}

#[test]
fn gemini_schemas_lose_what_gemini_refuses() {
    let schema = json!({
        "type": "object",
        "additionalProperties": false,
        "properties": {
            "name": {"type": ["string", "null"], "description": "Who"},
            "kind": {"type": "string", "enum": ["a", 1, true, null]},
            "level": {"type": "integer", "enum": [1, 2, null], "description": "How much"},
            "flag": {"type": "boolean", "enum": [true]},
            "only_nulls": {"type": "string", "enum": [null]},
            "maybe": {"type": "string", "nullable": true},
            "tags": {"type": "array", "items": {"type": ["null"]}},
            "weird": {"properties": "not an object"}
        },
        "anyOf": [{"type": "string"}]
    });
    let cleaned = clean_gemini_schema(&schema);
    let props = &cleaned["properties"];
    assert_eq!(props["name"]["type"], "string");
    assert_eq!(props["kind"]["enum"], json!(["a", "1", "true"]));
    assert!(props["level"].get("enum").is_none());
    assert_eq!(props["level"]["description"], "How much (Allowed values: 1, 2.)");
    assert_eq!(props["flag"]["description"], "Allowed values: true.");
    assert!(props["only_nulls"].get("enum").is_none());
    assert_eq!(props["maybe"]["nullable"], true);
    assert_eq!(props["tags"]["items"]["type"], "string");
    assert_eq!(clean_gemini_schema(&json!([1, "x"])), json!([1, "x"]));
}

fn media_request(bytes: usize) -> LlmRequest {
    LlmRequest {
        system: None,
        messages: vec![LlmMessage {
            role: LlmRole::User,
            content: vec![ContentBlock::Media { media_type: "video/mp4".into(), data: base64::engine::general_purpose::STANDARD.encode(vec![7u8; bytes]), name: "clip.mp4".into() }],
        }],
        tools: vec![],
        max_tokens: 10,
        enable_reasoning: false,
        reasoning_effort: Default::default(),
    }
}

const BIG: usize = 11 * 1024 * 1024;

#[tokio::test]
async fn gemini_uploads_a_big_file_and_refers_to_it() {
    let server = MockServer::start().await;
    Mock::given(method("POST")).and(path("/upload/v1beta/files"))
        .respond_with(ResponseTemplate::new(200).insert_header("x-goog-upload-url", format!("{}/resumable/1", server.uri()).as_str()))
        .mount(&server).await;
    Mock::given(method("POST")).and(path("/resumable/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"file": {"uri": "https://files/abc", "name": "files/abc", "state": "ACTIVE"}})))
        .mount(&server).await;
    Mock::given(method("POST")).and(wiremock::matchers::path_regex(":streamGenerateContent$"))
        .and(wiremock::matchers::body_string_contains("https://files/abc"))
        .respond_with(ResponseTemplate::new(200).set_body_raw("data: {\"candidates\":[{\"content\":{\"parts\":[{\"text\":\"a clip\"}]},\"finishReason\":\"STOP\"}]}\n\n", "text/event-stream"))
        .mount(&server).await;
    let provider = GeminiProvider::new(reqwest::Client::new(), "k".into(), "gemini-x".into(), server.uri());
    let response = collect_stream(provider.complete_stream(media_request(BIG)).await.unwrap()).await.unwrap();
    assert_eq!(response.content, vec![ContentBlock::Text { text: "a clip".into() }]);
}

#[tokio::test]
async fn gemini_upload_failures_are_errors() {
    let provider_at = |server: &MockServer| GeminiProvider::new(reqwest::Client::new(), "k".into(), "gemini-x".into(), server.uri());

    let refused = MockServer::start().await;
    Mock::given(method("POST")).and(path("/upload/v1beta/files")).respond_with(ResponseTemplate::new(403).set_body_string("no")).mount(&refused).await;
    assert!(matches!(provider_at(&refused).complete_stream(media_request(BIG)).await, Err(LlmError::ProviderError(m)) if m.contains("403")));

    let no_url = MockServer::start().await;
    Mock::given(method("POST")).and(path("/upload/v1beta/files")).respond_with(ResponseTemplate::new(200)).mount(&no_url).await;
    assert!(matches!(provider_at(&no_url).complete_stream(media_request(BIG)).await, Err(LlmError::ParseError(m)) if m.contains("no upload url")));

    let failed = MockServer::start().await;
    Mock::given(method("POST")).and(path("/upload/v1beta/files"))
        .respond_with(ResponseTemplate::new(200).insert_header("x-goog-upload-url", format!("{}/resumable/2", failed.uri()).as_str()))
        .mount(&failed).await;
    Mock::given(method("POST")).and(path("/resumable/2"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"file": {"uri": "https://files/x", "name": "files/x", "state": "FAILED"}})))
        .mount(&failed).await;
    assert!(matches!(provider_at(&failed).complete_stream(media_request(BIG)).await, Err(LlmError::ProviderError(m)) if m.contains("422")));

    let no_uri = MockServer::start().await;
    Mock::given(method("POST")).and(path("/upload/v1beta/files"))
        .respond_with(ResponseTemplate::new(200).insert_header("x-goog-upload-url", format!("{}/resumable/3", no_uri.uri()).as_str()))
        .mount(&no_uri).await;
    Mock::given(method("POST")).and(path("/resumable/3")).respond_with(ResponseTemplate::new(500).set_body_string("disk full")).mount(&no_uri).await;
    assert!(matches!(provider_at(&no_uri).complete_stream(media_request(BIG)).await, Err(LlmError::ProviderError(m)) if m.contains("disk full")));

    // Small files go inline: no upload at all.
    let inline = MockServer::start().await;
    Mock::given(method("POST")).and(wiremock::matchers::path_regex(":streamGenerateContent$")).and(wiremock::matchers::body_string_contains("inline_data"))
        .respond_with(ResponseTemplate::new(400).set_body_string("model says no"))
        .mount(&inline).await;
    assert!(matches!(provider_at(&inline).complete_stream(media_request(10)).await, Err(LlmError::ProviderError(m)) if m.contains("model says no")));
}
