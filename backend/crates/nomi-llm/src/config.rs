use super::anthropic::AnthropicProvider;
use super::deepseek::DeepSeekProvider;
use super::fake::FakeLlmProvider;
use super::gemini::GeminiProvider;
use super::openai::OpenAiProvider;
use super::openrouter::OpenRouterProvider;
use super::LlmProvider;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderKind {
    Anthropic,
    OpenAi,
    OpenRouter,
    Gemini,
    DeepSeek,
    /// Any endpoint that speaks OpenAI's chat API at its own base URL (most gateways and hosts).
    OpenAiCompatible,
    Fake,
}

impl ProviderKind {
    /// The name stored in the database (provider columns and their CHECKs).
    pub fn as_str(self) -> &'static str {
        match self {
            ProviderKind::Anthropic => "anthropic",
            ProviderKind::OpenAi => "openai",
            ProviderKind::OpenRouter => "openrouter",
            ProviderKind::Gemini => "gemini",
            ProviderKind::DeepSeek => "deepseek",
            ProviderKind::OpenAiCompatible => "openai_compatible",
            ProviderKind::Fake => "fake",
        }
    }

    /// None for a name this build doesn't know (e.g. one written by a newer version).
    pub fn parse(name: &str) -> Option<Self> {
        [
            ProviderKind::Anthropic,
            ProviderKind::OpenAi,
            ProviderKind::OpenRouter,
            ProviderKind::Gemini,
            ProviderKind::DeepSeek,
            ProviderKind::OpenAiCompatible,
            ProviderKind::Fake,
        ]
        .into_iter()
        .find(|kind| kind.as_str() == name)
    }
}

#[derive(Debug, Clone)]
pub struct ModelConfig {
    pub provider: ProviderKind,
    pub model_id: String,
    pub api_key: String,
    pub base_url: Option<String>,
    /// Sent with every request to an OpenAI-compatible endpoint (a custom provider's headers).
    pub extra_headers: Vec<(String, String)>,
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
        ProviderKind::OpenAiCompatible => Box::new(OpenAiProvider::compatible(
            http_client,
            config.api_key,
            config.model_id,
            config.base_url.unwrap_or_default(),
            config.extra_headers,
        )),
        ProviderKind::OpenRouter => {
            let base_url = config.base_url.unwrap_or_else(OpenRouterProvider::default_base_url);
            Box::new(OpenRouterProvider::new(http_client, config.api_key, config.model_id, base_url))
        }
        ProviderKind::Gemini => {
            let base_url = config.base_url.unwrap_or_else(GeminiProvider::default_base_url);
            Box::new(GeminiProvider::new(http_client, config.api_key, config.model_id, base_url))
        }
        ProviderKind::DeepSeek => {
            let base_url = config.base_url.unwrap_or_else(DeepSeekProvider::default_base_url);
            Box::new(DeepSeekProvider::new(http_client, config.api_key, config.model_id, base_url))
        }
        ProviderKind::Fake => Box::new(FakeLlmProvider),
    }
}
