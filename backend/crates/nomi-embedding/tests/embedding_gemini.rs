use nomi_embedding::gemini::GeminiEmbeddingProvider;
use nomi_embedding::{EmbeddingError, EmbeddingProvider};
use serde_json::json;
use wiremock::matchers::{body_partial_json, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn embed_returns_the_vector_from_the_response() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1beta/models/gemini-embedding-001:embedContent"))
        .and(body_partial_json(json!({
            "content": { "parts": [{ "text": "hello world" }] },
            "output_dimensionality": 1536
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "embedding": { "values": [0.1, 0.2, 0.3] }
        })))
        .mount(&server)
        .await;

    let provider = GeminiEmbeddingProvider::new(
        reqwest::Client::new(),
        "test-key".to_string(),
        "gemini-embedding-001".to_string(),
        server.uri(),
    );

    let embedding = provider.embed("hello world").await.unwrap();
    assert_eq!(embedding, vec![0.1f32, 0.2, 0.3]);
}

#[tokio::test]
async fn api_key_is_sent_as_a_query_parameter() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1beta/models/gemini-embedding-001:embedContent"))
        .and(query_param("key", "test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "embedding": { "values": [0.1] }
        })))
        .mount(&server)
        .await;

    let provider = GeminiEmbeddingProvider::new(
        reqwest::Client::new(),
        "test-key".to_string(),
        "gemini-embedding-001".to_string(),
        server.uri(),
    );

    provider.embed("hello").await.unwrap();
}

#[tokio::test]
async fn non_success_status_becomes_a_provider_error() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1beta/models/gemini-embedding-001:embedContent"))
        .respond_with(ResponseTemplate::new(401).set_body_string("unauthorized"))
        .mount(&server)
        .await;

    let provider = GeminiEmbeddingProvider::new(
        reqwest::Client::new(),
        "test-key".to_string(),
        "gemini-embedding-001".to_string(),
        server.uri(),
    );

    let result = provider.embed("hello").await;
    assert!(matches!(result, Err(EmbeddingError::ProviderError(_))));
}

#[tokio::test]
async fn malformed_response_becomes_a_parse_error() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1beta/models/gemini-embedding-001:embedContent"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"embedding": {}})))
        .mount(&server)
        .await;

    let provider = GeminiEmbeddingProvider::new(
        reqwest::Client::new(),
        "test-key".to_string(),
        "gemini-embedding-001".to_string(),
        server.uri(),
    );

    let result = provider.embed("hello").await;
    assert!(matches!(result, Err(EmbeddingError::ParseError(_))));
}

#[tokio::test]
async fn provider_name_and_model_id_are_exposed() {
    let provider = GeminiEmbeddingProvider::new(
        reqwest::Client::new(),
        "test-key".to_string(),
        "gemini-embedding-001".to_string(),
        "http://example.invalid".to_string(),
    );
    assert_eq!(provider.provider_name(), "gemini");
    assert_eq!(provider.model_id(), "gemini-embedding-001");
}
