use std::collections::VecDeque;
use std::sync::Mutex;
use std::time::Duration;

use async_trait::async_trait;
use nomi_llm::{LlmError, LlmEventStream, LlmProvider, LlmRequest, LlmResponse};

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
