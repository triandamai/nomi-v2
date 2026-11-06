//! Listing each provider's embedding models, building providers from config, the fake provider,
//! and the parse errors shared by every provider.

use nomi_embedding::{build_embedding_provider, estimate_tokens, list_embedding_provider_models, EmbeddingConfig, EmbeddingError, EmbeddingProviderKind};
use serde_json::json;
use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn config(provider: EmbeddingProviderKind, base_url: Option<String>) -> EmbeddingConfig {
    EmbeddingConfig { provider, model_id: "m-embed".to_string(), api_key: "key-1".to_string(), base_url }
}

async fn reply(server: &MockServer, verb: &str, at: &str, status: u16, body: serde_json::Value) {
    Mock::given(method(verb)).and(path(at)).respond_with(ResponseTemplate::new(status).set_body_json(body)).mount(server).await;
}

#[tokio::test]
async fn openai_lists_only_its_embedding_models() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/models"))
        .and(header("authorization", "Bearer key-1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"data": [{"id": "gpt-5"}, {"id": "text-embedding-3-small"}, {"id": "text-embedding-3-large"}, {"object": "no id"}]})))
        .mount(&server)
        .await;
    let models = list_embedding_provider_models(config(EmbeddingProviderKind::OpenAi, Some(server.uri())), reqwest::Client::new()).await.unwrap();
    assert_eq!(models.iter().map(|m| m.id.as_str()).collect::<Vec<_>>(), ["text-embedding-3-small", "text-embedding-3-large"]);
    assert!(models.iter().all(|m| m.label.is_none()));
}

#[tokio::test]
async fn gemini_lists_models_that_can_embed_with_their_names() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1beta/models"))
        .and(header("x-goog-api-key", "key-1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"models": [
            {"name": "models/gemini-embedding-001", "displayName": "Gemini Embedding", "supportedGenerationMethods": ["embedContent"]},
            {"name": "models/gemini-3-pro", "supportedGenerationMethods": ["generateContent"]},
            {"name": "bare-embed", "supportedGenerationMethods": ["embedContent"]},
            {"displayName": "no name", "supportedGenerationMethods": ["embedContent"]},
            {"name": "models/no-methods"}
        ]})))
        .mount(&server)
        .await;
    let models = list_embedding_provider_models(config(EmbeddingProviderKind::Gemini, Some(server.uri())), reqwest::Client::new()).await.unwrap();
    assert_eq!(models.len(), 2);
    assert_eq!(models[0].id, "gemini-embedding-001");
    assert_eq!(models[0].label.as_deref(), Some("Gemini Embedding"));
    assert_eq!(models[1].id, "bare-embed");
}

#[tokio::test]
async fn cohere_lists_its_embed_endpoint_models() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/models"))
        .and(query_param("endpoint", "embed"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"models": [{"name": "embed-v4.0"}, {"no": "name"}]})))
        .mount(&server)
        .await;
    let models = list_embedding_provider_models(config(EmbeddingProviderKind::Cohere, Some(server.uri())), reqwest::Client::new()).await.unwrap();
    assert_eq!(models.iter().map(|m| m.id.as_str()).collect::<Vec<_>>(), ["embed-v4.0"]);
}

#[tokio::test]
async fn the_fake_provider_lists_its_one_model_without_a_server() {
    let models = list_embedding_provider_models(config(EmbeddingProviderKind::Fake, None), reqwest::Client::new()).await.unwrap();
    assert_eq!(models.len(), 1);
    assert_eq!(models[0].id, "fake-embedding-model");
}

#[tokio::test]
async fn listing_reports_a_rejected_key_and_a_body_it_cannot_read() {
    for (kind, list_path, name, field) in [
        (EmbeddingProviderKind::OpenAi, "/v1/models", "openai", "data"),
        (EmbeddingProviderKind::Gemini, "/v1beta/models", "gemini", "models"),
        (EmbeddingProviderKind::Cohere, "/v1/models", "cohere", "models"),
    ] {
        let rejected = MockServer::start().await;
        reply(&rejected, "GET", list_path, 401, json!({"error": "bad key"})).await;
        match list_embedding_provider_models(config(kind.clone(), Some(rejected.uri())), reqwest::Client::new()).await {
            Err(EmbeddingError::ProviderError(message)) => assert!(message.starts_with(&format!("{name} returned 401")) && message.contains("bad key"), "{message}"),
            other => panic!("{name}: expected a provider error, got {other:?}"),
        }

        let empty = MockServer::start().await;
        reply(&empty, "GET", list_path, 200, json!({"other": []})).await;
        match list_embedding_provider_models(config(kind.clone(), Some(empty.uri())), reqwest::Client::new()).await {
            Err(EmbeddingError::ParseError(message)) => assert_eq!(message, format!("missing {field}")),
            other => panic!("{name}: expected a parse error, got {other:?}"),
        }

        let garbage = MockServer::start().await;
        Mock::given(method("GET")).and(path(list_path)).respond_with(ResponseTemplate::new(200).set_body_string("not json")).mount(&garbage).await;
        assert!(matches!(
            list_embedding_provider_models(config(kind, Some(garbage.uri())), reqwest::Client::new()).await,
            Err(EmbeddingError::ParseError(_))
        ));
    }
}

#[tokio::test]
async fn an_unreachable_provider_is_an_http_error() {
    let error = list_embedding_provider_models(config(EmbeddingProviderKind::OpenAi, Some("http://127.0.0.1:9".to_string())), reqwest::Client::new()).await.unwrap_err();
    assert!(matches!(error, EmbeddingError::Http(_)));
    assert!(error.to_string().starts_with("http request failed"));
}

#[tokio::test]
async fn the_factory_builds_each_provider_with_its_model() {
    let client = reqwest::Client::new();
    for (kind, name) in [
        (EmbeddingProviderKind::OpenAi, "openai"),
        (EmbeddingProviderKind::Gemini, "gemini"),
        (EmbeddingProviderKind::Cohere, "cohere"),
    ] {
        // Without a base URL each one uses its provider's own.
        let provider = build_embedding_provider(config(kind, None), client.clone());
        assert_eq!(provider.provider_name(), name);
        assert_eq!(provider.model_id(), "m-embed");
    }
    let fake = build_embedding_provider(config(EmbeddingProviderKind::Fake, None), client);
    assert_eq!(fake.provider_name(), "fake");
    assert_eq!(fake.model_id(), "");
    assert_eq!(fake.embed("anything").await.unwrap().len(), 1536);
    // The fake has no query side or token count of its own: the defaults apply.
    assert_eq!(fake.embed_for_query("q").await.unwrap().len(), 1536);
    let counted = fake.embed_counted("q", true).await.unwrap();
    assert_eq!((counted.vector.len(), counted.tokens), (1536, None));
    assert_eq!(fake.embed_counted("d", false).await.unwrap().vector.len(), 1536);
}

#[tokio::test]
async fn every_provider_refuses_a_vector_with_something_other_than_numbers() {
    let server = MockServer::start().await;
    reply(&server, "POST", "/v1/embeddings", 200, json!({"data": [{"embedding": [0.1, "x"]}]})).await;
    reply(&server, "POST", "/v1beta/models/m-embed:embedContent", 200, json!({"embedding": {"values": [0.1, null]}})).await;
    reply(&server, "POST", "/v2/embed", 200, json!({"embeddings": {"float": [[0.1, true]]}})).await;
    let client = reqwest::Client::new();
    for kind in [EmbeddingProviderKind::OpenAi, EmbeddingProviderKind::Gemini, EmbeddingProviderKind::Cohere] {
        let provider = build_embedding_provider(config(kind, Some(server.uri())), client.clone());
        match provider.embed("t").await {
            Err(EmbeddingError::ParseError(message)) => assert_eq!(message, "embedding element is not a number"),
            other => panic!("{}: {other:?}", provider.provider_name()),
        }
    }
}

#[tokio::test]
async fn openai_asks_for_stored_dimensions_only_on_v3_models() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/embeddings"))
        .and(wiremock::matchers::body_json(json!({"model": "text-embedding-ada-002", "input": "t"})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"data": [{"embedding": [1.0]}], "usage": {"total_tokens": 7}})))
        .mount(&server)
        .await;
    let provider = nomi_embedding::openai::OpenAiEmbeddingProvider::new(reqwest::Client::new(), "k".into(), "text-embedding-ada-002".into(), server.uri());
    use nomi_embedding::EmbeddingProvider;
    let counted = provider.embed_counted("t", false).await.unwrap();
    assert_eq!(counted.tokens, Some(7), "total_tokens when prompt_tokens is missing");
}

#[test]
fn tokens_are_estimated_at_about_four_characters_each() {
    assert_eq!(estimate_tokens(""), 1);
    assert_eq!(estimate_tokens("abcd"), 1);
    assert_eq!(estimate_tokens("abcde"), 2);
    assert_eq!(estimate_tokens("ééééééééé"), 3, "characters, not bytes");
}
