use nomi_embedding::openai::OpenAiEmbeddingProvider;
use nomi_embedding::{EmbeddingError, EmbeddingProvider};
use serde_json::json;
use wiremock::matchers::{body_partial_json, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn embed_returns_the_vector_from_the_response() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/embeddings"))
        .and(body_partial_json(json!({
            "model": "text-embedding-3-small",
            "input": "hello world"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": [{"embedding": [0.1, 0.2, 0.3], "index": 0}],
            "model": "text-embedding-3-small",
            "usage": {"prompt_tokens": 2, "total_tokens": 2}
        })))
        .mount(&server)
        .await;

    let provider = OpenAiEmbeddingProvider::new(
        reqwest::Client::new(),
        "test-key".to_string(),
        "text-embedding-3-small".to_string(),
        server.uri(),
    );

    let embedding = provider.embed("hello world").await.unwrap();
    assert_eq!(embedding, vec![0.1f32, 0.2, 0.3]);
}

#[tokio::test]
async fn non_success_status_becomes_a_provider_error() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/embeddings"))
        .respond_with(ResponseTemplate::new(401).set_body_string("unauthorized"))
        .mount(&server)
        .await;

    let provider = OpenAiEmbeddingProvider::new(
        reqwest::Client::new(),
        "test-key".to_string(),
        "text-embedding-3-small".to_string(),
        server.uri(),
    );

    let result = provider.embed("hello").await;
    assert!(matches!(result, Err(EmbeddingError::ProviderError(_))));
}

#[tokio::test]
async fn malformed_response_becomes_a_parse_error() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/embeddings"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"data": []})))
        .mount(&server)
        .await;

    let provider = OpenAiEmbeddingProvider::new(
        reqwest::Client::new(),
        "test-key".to_string(),
        "text-embedding-3-small".to_string(),
        server.uri(),
    );

    let result = provider.embed("hello").await;
    assert!(matches!(result, Err(EmbeddingError::ParseError(_))));
}

#[tokio::test]
async fn provider_name_and_model_id_are_exposed() {
    let provider = OpenAiEmbeddingProvider::new(
        reqwest::Client::new(),
        "test-key".to_string(),
        "text-embedding-3-small".to_string(),
        "http://example.invalid".to_string(),
    );
    assert_eq!(provider.provider_name(), "openai");
    assert_eq!(provider.model_id(), "text-embedding-3-small");
}

#[tokio::test]
async fn embed_for_query_defaults_to_calling_embed() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/embeddings"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": [{"embedding": [0.5, 0.6], "index": 0}],
            "model": "text-embedding-3-small",
            "usage": {"prompt_tokens": 1, "total_tokens": 1}
        })))
        .mount(&server)
        .await;

    let provider = OpenAiEmbeddingProvider::new(
        reqwest::Client::new(),
        "test-key".to_string(),
        "text-embedding-3-small".to_string(),
        server.uri(),
    );

    let embedding = provider.embed_for_query("hello").await.unwrap();
    assert_eq!(embedding, vec![0.5f32, 0.6]);
}
