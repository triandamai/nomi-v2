use nomi_orchestrator::embedding::{build_embedding_provider, EmbeddingConfig};
use serde_json::json;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn build_embedding_provider_calls_the_openai_embeddings_endpoint() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/embeddings"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": [{"embedding": [0.1, 0.2], "index": 0}],
            "model": "text-embedding-3-small",
            "usage": {"prompt_tokens": 1, "total_tokens": 1}
        })))
        .mount(&server)
        .await;

    let config = EmbeddingConfig {
        model_id: "text-embedding-3-small".to_string(),
        api_key: "test-key".to_string(),
        base_url: Some(server.uri()),
    };

    let provider = build_embedding_provider(config, reqwest::Client::new());
    let embedding = provider.embed("hello").await.unwrap();
    assert_eq!(embedding, vec![0.1f32, 0.2]);
}

#[tokio::test]
async fn base_url_none_falls_through_to_the_real_default_without_panicking() {
    let config = EmbeddingConfig {
        model_id: "text-embedding-3-small".to_string(),
        api_key: "test-key".to_string(),
        base_url: None,
    };

    // Construction only — no network call is made.
    let _provider = build_embedding_provider(config, reqwest::Client::new());
}
