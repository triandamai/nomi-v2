pub mod types;
pub mod anthropic;
pub mod openai;
pub mod gemini;
pub mod config;

pub use types::*;
pub use config::{build_provider, ModelConfig, ProviderKind};

use async_trait::async_trait;

#[async_trait]
pub trait LlmProvider: Send + Sync {
    async fn complete(&self, request: LlmRequest) -> Result<LlmResponse, LlmError>;
}
