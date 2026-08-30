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

pub async fn list_models(client: &reqwest::Client, api_key: &str, base_url: &str) -> Result<Vec<crate::EmbeddingModelSummary>, EmbeddingError> {
    let response = client.get(format!("{base_url}/v1beta/models?key={api_key}")).send().await?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        return Err(EmbeddingError::ProviderError(format!("gemini returned {status}: {text}")));
    }

    let body: serde_json::Value = response.json().await.map_err(|e| EmbeddingError::ParseError(e.to_string()))?;
    let models = body.get("models").and_then(|m| m.as_array()).ok_or_else(|| EmbeddingError::ParseError("missing models".to_string()))?;

    // Only keep models whose supportedGenerationMethods includes embedContent — the list
    // endpoint returns every Gemini model (chat and embedding) mixed together.
    Ok(models
        .iter()
        .filter(|m| {
            m.get("supportedGenerationMethods")
                .and_then(|methods| methods.as_array())
                .map(|methods| methods.iter().any(|method| method.as_str() == Some("embedContent")))
                .unwrap_or(false)
        })
        .filter_map(|m| {
            let name = m.get("name").and_then(|v| v.as_str())?;
            let id = name.strip_prefix("models/").unwrap_or(name).to_string();
            let label = m.get("displayName").and_then(|v| v.as_str()).map(|s| s.to_string());
            Some(crate::EmbeddingModelSummary { id, label })
        })
        .collect())
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
