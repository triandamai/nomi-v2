use async_trait::async_trait;
use serde_json::json;

use super::types::EmbeddingError;
use super::EmbeddingProvider;

const OUTPUT_DIMENSION: u32 = 1536;

pub struct CohereEmbeddingProvider {
    client: reqwest::Client,
    api_key: String,
    model: String,
    base_url: String,
}

impl CohereEmbeddingProvider {
    pub fn new(client: reqwest::Client, api_key: String, model: String, base_url: String) -> Self {
        Self { client, api_key, model, base_url }
    }

    pub fn default_base_url() -> String {
        "https://api.cohere.com".to_string()
    }

    async fn embed_with_input_type(&self, text: &str, input_type: &str) -> Result<Vec<f32>, EmbeddingError> {
        let body = json!({
            "model": self.model,
            "texts": [text],
            "input_type": input_type,
            "output_dimension": OUTPUT_DIMENSION,
            "embedding_types": ["float"],
        });

        let response = self
            .client
            .post(format!("{}/v2/embed", self.base_url))
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(EmbeddingError::ProviderError(format!("cohere returned {status}: {text}")));
        }

        let body: serde_json::Value = response
            .json()
            .await
            .map_err(|e| EmbeddingError::ParseError(e.to_string()))?;

        let embedding = body
            .get("embeddings")
            .and_then(|e| e.get("float"))
            .and_then(|f| f.as_array())
            .and_then(|f| f.first())
            .and_then(|e| e.as_array())
            .ok_or_else(|| EmbeddingError::ParseError("missing embeddings.float[0]".to_string()))?;

        embedding
            .iter()
            .map(|v| {
                v.as_f64()
                    .map(|f| f as f32)
                    .ok_or_else(|| EmbeddingError::ParseError("embedding element is not a number".to_string()))
            })
            .collect()
    }
}

#[async_trait]
impl EmbeddingProvider for CohereEmbeddingProvider {
    async fn embed(&self, text: &str) -> Result<Vec<f32>, EmbeddingError> {
        self.embed_with_input_type(text, "search_document").await
    }

    async fn embed_for_query(&self, text: &str) -> Result<Vec<f32>, EmbeddingError> {
        self.embed_with_input_type(text, "search_query").await
    }

    fn provider_name(&self) -> &'static str {
        "cohere"
    }

    fn model_id(&self) -> &str {
        &self.model
    }
}
