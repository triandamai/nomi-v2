use std::collections::VecDeque;
use std::sync::Mutex;
use std::time::Duration;

use async_trait::async_trait;
use nomi_embedding::{EmbeddingError, EmbeddingProvider};
use nomi_llm::{LlmError, LlmEventStream, LlmProvider, LlmRequest, LlmResponse};

pub const TEST_SETTINGS_KEY: [u8; 32] = [7u8; 32];
pub const TEST_MQTT_BROKER_HOST: &str = "localhost";
pub const TEST_MQTT_BROKER_PORT: u16 = 1883;

enum FakeOutcome {
    Success(LlmResponse),
    Failure(String),
    Sequence(Mutex<VecDeque<LlmResponse>>),
}

pub struct FakeLlmProvider {
    outcome: FakeOutcome,
    delay: Option<Duration>,
    pub received_requests: Mutex<Vec<LlmRequest>>,
}

impl FakeLlmProvider {
    pub fn success(response: LlmResponse) -> Self {
        Self { outcome: FakeOutcome::Success(response), delay: None, received_requests: Mutex::new(Vec::new()) }
    }

    pub fn failure(message: impl Into<String>) -> Self {
        Self { outcome: FakeOutcome::Failure(message.into()), delay: None, received_requests: Mutex::new(Vec::new()) }
    }

    pub fn sequence(responses: Vec<LlmResponse>) -> Self {
        Self {
            outcome: FakeOutcome::Sequence(Mutex::new(responses.into())),
            delay: None,
            received_requests: Mutex::new(Vec::new()),
        }
    }

    pub fn with_delay(mut self, delay: Duration) -> Self {
        self.delay = Some(delay);
        self
    }
}

#[async_trait]
impl LlmProvider for FakeLlmProvider {
    async fn complete_stream(&self, request: LlmRequest) -> Result<LlmEventStream, LlmError> {
        self.received_requests.lock().unwrap().push(request);
        if let Some(delay) = self.delay {
            tokio::time::sleep(delay).await;
        }
        let response = match &self.outcome {
            FakeOutcome::Success(response) => Ok(response.clone()),
            FakeOutcome::Failure(message) => Err(LlmError::ProviderError(message.clone())),
            FakeOutcome::Sequence(queue) => queue
                .lock()
                .unwrap()
                .pop_front()
                .ok_or_else(|| LlmError::ProviderError("sequence exhausted".to_string())),
        }?;
        Ok(nomi_llm::response_to_stream(response))
    }
}

enum FakeEmbeddingOutcome {
    Success(Vec<f32>),
    Failure(String),
}

pub struct FakeEmbeddingProvider {
    outcome: FakeEmbeddingOutcome,
}

impl FakeEmbeddingProvider {
    pub fn success(vector: Vec<f32>) -> Self {
        Self { outcome: FakeEmbeddingOutcome::Success(vector) }
    }

    pub fn failure(message: impl Into<String>) -> Self {
        Self { outcome: FakeEmbeddingOutcome::Failure(message.into()) }
    }
}

#[async_trait]
impl EmbeddingProvider for FakeEmbeddingProvider {
    async fn embed(&self, _text: &str) -> Result<Vec<f32>, EmbeddingError> {
        match &self.outcome {
            FakeEmbeddingOutcome::Success(vector) => Ok(vector.clone()),
            FakeEmbeddingOutcome::Failure(message) => Err(EmbeddingError::ProviderError(message.clone())),
        }
    }
}

pub fn dummy_embedding() -> Vec<f32> {
    vec![0.0; 1536]
}
