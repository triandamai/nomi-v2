use async_trait::async_trait;
use serde_json::json;

use super::types::EmbeddingError;
use super::EmbeddingProvider;

pub struct OpenAiEmbeddingProvider {
    client: reqwest::Client,
    api_key: String,
    model: String,
    base_url: String,
}

impl OpenAiEmbeddingProvider {
    pub fn new(client: reqwest::Client, api_key: String, model: String, base_url: String) -> Self {
        Self { client, api_key, model, base_url }
    }

    pub fn default_base_url() -> String {
        "https://api.openai.com".to_string()
    }
}

pub async fn list_models(client: &reqwest::Client, api_key: &str, base_url: &str) -> Result<Vec<crate::EmbeddingModelSummary>, EmbeddingError> {
    let response = client.get(format!("{base_url}/v1/models")).bearer_auth(api_key).send().await?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        return Err(EmbeddingError::ProviderError(format!("openai returned {status}: {text}")));
    }

    let body: serde_json::Value = response.json().await.map_err(|e| EmbeddingError::ParseError(e.to_string()))?;
    let data = body.get("data").and_then(|d| d.as_array()).ok_or_else(|| EmbeddingError::ParseError("missing data".to_string()))?;

    // /v1/models returns every model on the account (chat, image, embedding, ...) mixed
    // together — OpenAI's embedding model ids all contain "embed" (text-embedding-3-small,
    // text-embedding-ada-002, ...), which is the only reliable filter available since the
    // list endpoint doesn't expose a capability/type field to filter on directly.
    Ok(data
        .iter()
        .filter_map(|m| m.get("id").and_then(|v| v.as_str()))
        .filter(|id| id.contains("embed"))
        .map(|id| crate::EmbeddingModelSummary { id: id.to_string(), label: None })
        .collect())
}

#[async_trait]
impl EmbeddingProvider for OpenAiEmbeddingProvider {
    async fn embed(&self, text: &str) -> Result<Vec<f32>, EmbeddingError> {
        let body = json!({
            "model": self.model,
            "input": text,
        });

        let response = self
            .client
            .post(format!("{}/v1/embeddings", self.base_url))
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(EmbeddingError::ProviderError(format!("openai returned {status}: {text}")));
        }

        let body: serde_json::Value = response
            .json()
            .await
            .map_err(|e| EmbeddingError::ParseError(e.to_string()))?;

        let embedding = body
            .get("data")
            .and_then(|d| d.as_array())
            .and_then(|d| d.first())
            .and_then(|e| e.get("embedding"))
            .and_then(|e| e.as_array())
            .ok_or_else(|| EmbeddingError::ParseError("missing data[0].embedding".to_string()))?;

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
        "openai"
    }

    fn model_id(&self) -> &str {
        &self.model
    }
}
