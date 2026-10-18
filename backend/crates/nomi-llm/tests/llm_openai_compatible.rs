use nomi_llm::{build_provider, collect_stream, list_provider_models, ContentBlock, LlmMessage, LlmRequest, LlmRole, ModelConfig, ProviderKind};
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn hello() -> LlmRequest {
    LlmRequest {
        system: None,
        messages: vec![LlmMessage { role: LlmRole::User, content: vec![ContentBlock::Text { text: "hello".to_string() }] }],
        tools: vec![],
        max_tokens: 100,
        enable_reasoning: false,
        reasoning_effort: Default::default(),
    }
}

const REPLY: &str = concat!(
    "data: {\"choices\":[{\"delta\":{\"content\":\"hi\"},\"finish_reason\":null}]}\n\n",
    "data: {\"choices\":[{\"delta\":{},\"finish_reason\":\"stop\"}]}\n\n",
    "data: [DONE]\n\n",
);

fn compatible(base_url: String, headers: Vec<(String, String)>) -> ModelConfig {
    ModelConfig {
        provider: ProviderKind::OpenAiCompatible,
        model_id: "some/model".to_string(),
        api_key: "gw-key".to_string(),
        base_url: Some(base_url),
        extra_headers: headers,
    }
}

#[tokio::test]
async fn an_openai_compatible_endpoint_is_called_at_its_own_base_url() {
    // Gateways document their base URL with the version in it (https://…/v1); nothing is added.
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .and(header("authorization", "Bearer gw-key"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(REPLY, "text/event-stream"))
        .expect(1)
        .mount(&server)
        .await;

    let provider = build_provider(compatible(format!("{}/v1", server.uri()), vec![]), reqwest::Client::new());
    let response = collect_stream(provider.complete_stream(hello()).await.unwrap()).await.unwrap();

    assert_eq!(response.content, vec![ContentBlock::Text { text: "hi".to_string() }]);
}

#[tokio::test]
async fn custom_headers_go_with_every_request() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/gateway/chat/completions"))
        .and(header("x-team", "nomi"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(REPLY, "text/event-stream"))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/gateway/models"))
        .and(header("x-team", "nomi"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({ "data": [{ "id": "some/model" }] })))
        .expect(1)
        .mount(&server)
        .await;

    let config = compatible(format!("{}/gateway", server.uri()), vec![("X-Team".to_string(), "nomi".to_string())]);
    let provider = build_provider(config.clone(), reqwest::Client::new());
    collect_stream(provider.complete_stream(hello()).await.unwrap()).await.unwrap();
    let models = list_provider_models(config, reqwest::Client::new()).await.unwrap();

    assert_eq!(models.iter().map(|m| m.id.as_str()).collect::<Vec<_>>(), ["some/model"]);
}

#[tokio::test]
async fn native_openai_still_adds_its_version_path() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(REPLY, "text/event-stream"))
        .expect(1)
        .mount(&server)
        .await;

    let config = ModelConfig { provider: ProviderKind::OpenAi, base_url: Some(server.uri()), ..compatible(String::new(), vec![]) };
    let provider = build_provider(config, reqwest::Client::new());
    collect_stream(provider.complete_stream(hello()).await.unwrap()).await.unwrap();
}
