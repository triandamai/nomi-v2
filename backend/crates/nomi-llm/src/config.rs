use super::anthropic::AnthropicProvider;
use super::fake::FakeLlmProvider;
use super::gemini::GeminiProvider;
use super::openai::OpenAiProvider;
use super::openrouter::OpenRouterProvider;
use super::LlmProvider;

#[derive(Debug, Clone, PartialEq)]
pub enum ProviderKind {
    Anthropic,
    OpenAi,
    OpenRouter,
    Gemini,
    Fake,
}

#[derive(Debug, Clone)]
pub struct ModelConfig {
    pub provider: ProviderKind,
    pub model_id: String,
    pub api_key: String,
    pub base_url: Option<String>,
}

pub fn build_provider(config: ModelConfig, http_client: reqwest::Client) -> Box<dyn LlmProvider> {
    match config.provider {
        ProviderKind::Anthropic => {
            let base_url = config.base_url.unwrap_or_else(AnthropicProvider::default_base_url);
            Box::new(AnthropicProvider::new(http_client, config.api_key, config.model_id, base_url))
        }
        ProviderKind::OpenAi => {
            let base_url = config.base_url.unwrap_or_else(OpenAiProvider::default_base_url);
            Box::new(OpenAiProvider::new(http_client, config.api_key, config.model_id, base_url))
        }
        ProviderKind::OpenRouter => {
            let base_url = config.base_url.unwrap_or_else(OpenRouterProvider::default_base_url);
            Box::new(OpenRouterProvider::new(http_client, config.api_key, config.model_id, base_url))
        }
        ProviderKind::Gemini => {
            let base_url = config.base_url.unwrap_or_else(GeminiProvider::default_base_url);
            Box::new(GeminiProvider::new(http_client, config.api_key, config.model_id, base_url))
        }
        ProviderKind::Fake => Box::new(FakeLlmProvider),
    }
}
