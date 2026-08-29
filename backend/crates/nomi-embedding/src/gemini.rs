use async_trait::async_trait;
use serde_json::json;

use super::types::EmbeddingError;
use super::EmbeddingProvider;

const OUTPUT_DIMENSIONALITY: u32 = 1536;

pub struct GeminiEmbeddingProvider {
    client: reqwest::Client,
    api_key: String,
    model: String,
    base_url: String,
}

impl GeminiEmbeddingProvider {
    pub fn new(client: reqwest::Client, api_key: String, model: String, base_url: String) -> Self {
        Self { client, api_key, model, base_url }
    }

    pub fn default_base_url() -> String {
        "https://generativelanguage.googleapis.com".to_string()
    }
}

#[async_trait]
impl EmbeddingProvider for GeminiEmbeddingProvider {
    async fn embed(&self, text: &str) -> Result<Vec<f32>, EmbeddingError> {
        let body = json!({
            "content": { "parts": [{ "text": text }] },
            "output_dimensionality": OUTPUT_DIMENSIONALITY,
        });

        let url = format!("{}/v1beta/models/{}:embedContent?key={}", self.base_url, self.model, self.api_key);

        let response = self.client.post(url).json(&body).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(EmbeddingError::ProviderError(format!("gemini returned {status}: {text}")));
        }

        let body: serde_json::Value = response
            .json()
            .await
            .map_err(|e| EmbeddingError::ParseError(e.to_string()))?;

        let embedding = body
            .get("embedding")
            .and_then(|e| e.get("values"))
            .and_then(|v| v.as_array())
            .ok_or_else(|| EmbeddingError::ParseError("missing embedding.values".to_string()))?;

        embedding
            .iter()
            .map(|v| {
                v.as_f64()
                    .map(|f| f as f32)
                    .ok_or_else(|| EmbeddingError::ParseError("embedding element is not a number".to_string()))
            })
            .collect()
    }

    fn provider_name(&self) -> &'static str {
        "gemini"
    }

    fn model_id(&self) -> &str {
        &self.model
    }
}
