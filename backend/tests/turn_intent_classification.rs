mod support;

use nomi_orchestrator::llm::{ContentBlock, LlmResponse, StopReason};
use nomi_orchestrator::turn::routing::{classify_intent, Intent};

use support::FakeLlmProvider;

fn text_response(text: &str) -> LlmResponse {
    LlmResponse {
        content: vec![ContentBlock::Text { text: text.to_string() }],
        stop_reason: StopReason::EndTurn,
        input_tokens: 1,
        output_tokens: 1,
    }
}

#[tokio::test]
async fn classifies_money_intent() {
    let provider = FakeLlmProvider::success(text_response("money"));
    let intent = classify_intent(&provider, "how much did I spend on food this month?").await;
    assert_eq!(intent, Intent::Money);
}

#[tokio::test]
async fn classifies_chitchat_intent() {
    let provider = FakeLlmProvider::success(text_response("chitchat"));
    let intent = classify_intent(&provider, "how's the weather today?").await;
    assert_eq!(intent, Intent::Chitchat);
}

#[tokio::test]
async fn is_case_and_whitespace_insensitive() {
    let provider = FakeLlmProvider::success(text_response("  Money  \n"));
    let intent = classify_intent(&provider, "budget please").await;
    assert_eq!(intent, Intent::Money);
}

#[tokio::test]
async fn unrecognized_text_falls_back_to_chitchat() {
    let provider = FakeLlmProvider::success(text_response("something else entirely"));
    let intent = classify_intent(&provider, "anything").await;
    assert_eq!(intent, Intent::Chitchat);
}

#[tokio::test]
async fn provider_failure_falls_back_to_chitchat() {
    let provider = FakeLlmProvider::failure("provider down");
    let intent = classify_intent(&provider, "anything").await;
    assert_eq!(intent, Intent::Chitchat);
}
