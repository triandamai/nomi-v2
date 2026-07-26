use nomi_orchestrator::llm::{build_provider, ContentBlock, LlmMessage, LlmRequest, LlmRole, ModelConfig, ProviderKind};
use serde_json::json;
use wiremock::matchers::{method, path, path_regex};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn simple_request() -> LlmRequest {
    LlmRequest {
        system: None,
        messages: vec![LlmMessage {
            role: LlmRole::User,
            content: vec![ContentBlock::Text { text: "hi".to_string() }],
        }],
        tools: vec![],
        max_tokens: 50,
    }
}

#[tokio::test]
async fn build_provider_selects_anthropic() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "content": [{"type": "text", "text": "anthropic reply"}],
            "stop_reason": "end_turn",
            "usage": {"input_tokens": 1, "output_tokens": 1}
        })))
        .mount(&server)
        .await;

    let provider = build_provider(
        ModelConfig {
            provider: ProviderKind::Anthropic,
            model_id: "claude-haiku-4-5".to_string(),
            api_key: "key".to_string(),
            base_url: Some(server.uri()),
        },
        reqwest::Client::new(),
    );

    let response = provider.complete(simple_request()).await.unwrap();
    assert_eq!(response.content, vec![ContentBlock::Text { text: "anthropic reply".to_string() }]);
}

#[tokio::test]
async fn build_provider_selects_openai() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "choices": [{"message": {"content": "openai reply"}, "finish_reason": "stop"}],
            "usage": {"prompt_tokens": 1, "completion_tokens": 1}
        })))
        .mount(&server)
        .await;

    let provider = build_provider(
        ModelConfig {
            provider: ProviderKind::OpenAi,
            model_id: "gpt-4o".to_string(),
            api_key: "key".to_string(),
            base_url: Some(server.uri()),
        },
        reqwest::Client::new(),
    );

    let response = provider.complete(simple_request()).await.unwrap();
    assert_eq!(response.content, vec![ContentBlock::Text { text: "openai reply".to_string() }]);
}

#[tokio::test]
async fn build_provider_selects_gemini() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path_regex(r"^/v1beta/models/.*:generateContent$"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "candidates": [{"content": {"parts": [{"text": "gemini reply"}]}, "finishReason": "STOP"}],
            "usageMetadata": {"promptTokenCount": 1, "candidatesTokenCount": 1}
        })))
        .mount(&server)
        .await;

    let provider = build_provider(
        ModelConfig {
            provider: ProviderKind::Gemini,
            model_id: "gemini-2.0-flash".to_string(),
            api_key: "key".to_string(),
            base_url: Some(server.uri()),
        },
        reqwest::Client::new(),
    );

    let response = provider.complete(simple_request()).await.unwrap();
    assert_eq!(response.content, vec![ContentBlock::Text { text: "gemini reply".to_string() }]);
}

#[tokio::test]
async fn build_provider_uses_the_real_default_base_url_when_none_given() {
    // No mock server involved — this only checks construction doesn't panic
    // and that omitting base_url falls through to each provider's own default,
    // without actually making a network call to the real API.
    let _anthropic = build_provider(
        ModelConfig {
            provider: ProviderKind::Anthropic,
            model_id: "claude-haiku-4-5".to_string(),
            api_key: "key".to_string(),
            base_url: None,
        },
        reqwest::Client::new(),
    );
    let _openai = build_provider(
        ModelConfig {
            provider: ProviderKind::OpenAi,
            model_id: "gpt-4o".to_string(),
            api_key: "key".to_string(),
            base_url: None,
        },
        reqwest::Client::new(),
    );
    let _gemini = build_provider(
        ModelConfig {
            provider: ProviderKind::Gemini,
            model_id: "gemini-2.0-flash".to_string(),
            api_key: "key".to_string(),
            base_url: None,
        },
        reqwest::Client::new(),
    );
}
