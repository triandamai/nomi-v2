mod support;

use nomi_orchestrator::app::AppState;
use nomi_orchestrator::llm::{ContentBlock, LlmResponse, StopReason};
use sqlx::PgPool;
use std::sync::Arc;
use support::FakeLlmProvider;
use tokio::sync::RwLock;

#[sqlx::test]
async fn swapping_the_provider_lock_is_visible_to_the_next_reader(pool: PgPool) {
    let provider = Arc::new(FakeLlmProvider::success(LlmResponse {
        content: vec![ContentBlock::Text { text: "first".to_string() }],
        stop_reason: StopReason::EndTurn,
        input_tokens: 0,
        output_tokens: 0,
    }));
    let lock: Arc<RwLock<Arc<dyn nomi_orchestrator::llm::LlmProvider>>> = Arc::new(RwLock::new(provider));

    let new_provider = Arc::new(FakeLlmProvider::success(LlmResponse {
        content: vec![ContentBlock::Text { text: "second".to_string() }],
        stop_reason: StopReason::EndTurn,
        input_tokens: 0,
        output_tokens: 0,
    }));
    *lock.write().await = new_provider;

    let current = lock.read().await.clone();
    let response = nomi_orchestrator::llm::complete(
        current.as_ref(),
        nomi_orchestrator::llm::LlmRequest {
            system: None,
            messages: vec![],
            tools: vec![],
            max_tokens: 10,
        },
    )
    .await
    .unwrap();
    assert_eq!(response.content, vec![ContentBlock::Text { text: "second".to_string() }]);

    // pool is unused directly but #[sqlx::test] requires the parameter to provision a test DB
    let _ = &pool;
}
