use nomi_embedding::cohere::CohereEmbeddingProvider;
use nomi_embedding::{EmbeddingError, EmbeddingProvider};
use serde_json::json;
use wiremock::matchers::{body_partial_json, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn embed_sends_search_document_input_type_and_returns_the_vector() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v2/embed"))
        .and(body_partial_json(json!({
            "model": "embed-v4.0",
            "texts": ["hello world"],
            "input_type": "search_document",
            "output_dimension": 1536
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "test",
            "embeddings": { "float": [[0.1, 0.2, 0.3]] }
        })))
        .mount(&server)
        .await;

    let provider = CohereEmbeddingProvider::new(
        reqwest::Client::new(),
        "test-key".to_string(),
        "embed-v4.0".to_string(),
        server.uri(),
    );

    let embedding = provider.embed("hello world").await.unwrap();
    assert_eq!(embedding, vec![0.1f32, 0.2, 0.3]);
}

#[tokio::test]
async fn embed_for_query_sends_search_query_input_type() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v2/embed"))
        .and(body_partial_json(json!({ "input_type": "search_query" })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "test",
            "embeddings": { "float": [[0.4, 0.5]] }
        })))
        .mount(&server)
        .await;

    let provider = CohereEmbeddingProvider::new(
        reqwest::Client::new(),
        "test-key".to_string(),
        "embed-v4.0".to_string(),
        server.uri(),
    );

    let embedding = provider.embed_for_query("what did I eat").await.unwrap();
    assert_eq!(embedding, vec![0.4f32, 0.5]);
}

#[tokio::test]
async fn non_success_status_becomes_a_provider_error() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v2/embed"))
        .respond_with(ResponseTemplate::new(401).set_body_string("unauthorized"))
        .mount(&server)
        .await;

    let provider = CohereEmbeddingProvider::new(
        reqwest::Client::new(),
        "test-key".to_string(),
        "embed-v4.0".to_string(),
        server.uri(),
    );

    let result = provider.embed("hello").await;
    assert!(matches!(result, Err(EmbeddingError::ProviderError(_))));
}

#[tokio::test]
async fn malformed_response_becomes_a_parse_error() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v2/embed"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"embeddings": {}})))
        .mount(&server)
        .await;

    let provider = CohereEmbeddingProvider::new(
        reqwest::Client::new(),
        "test-key".to_string(),
        "embed-v4.0".to_string(),
        server.uri(),
    );

    let result = provider.embed("hello").await;
    assert!(matches!(result, Err(EmbeddingError::ParseError(_))));
}

#[tokio::test]
async fn provider_name_and_model_id_are_exposed() {
    let provider = CohereEmbeddingProvider::new(
        reqwest::Client::new(),
        "test-key".to_string(),
        "embed-v4.0".to_string(),
        "http://example.invalid".to_string(),
    );
    assert_eq!(provider.provider_name(), "cohere");
    assert_eq!(provider.model_id(), "embed-v4.0");
}
