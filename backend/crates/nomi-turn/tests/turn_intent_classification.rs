use nomi_agent_chitchat::ChitchatAgent;
use nomi_agent_core::AgentRegistry;
use nomi_agent_money::MoneyAgent;
use nomi_llm::{ContentBlock, LlmResponse, StopReason};
use nomi_turn::routing::classify_intent;

use nomi_test_support::FakeLlmProvider;

fn text_response(text: &str) -> LlmResponse {
    LlmResponse {
        content: vec![ContentBlock::Text { text: text.to_string() }],
        stop_reason: StopReason::EndTurn,
        input_tokens: 1,
        output_tokens: 1,
    }
}

fn registry() -> AgentRegistry {
    AgentRegistry::new(vec![Box::new(MoneyAgent), Box::new(ChitchatAgent)])
}

#[tokio::test]
async fn classifies_money_intent() {
    let provider = FakeLlmProvider::success(text_response("money"));
    let registry = registry();
    let agent = classify_intent(&provider, &registry, "how much did I spend on food this month?").await;
    assert_eq!(agent.agent_type(), "money");
}

#[tokio::test]
async fn classifies_chitchat_intent() {
    let provider = FakeLlmProvider::success(text_response("chitchat"));
    let registry = registry();
    let agent = classify_intent(&provider, &registry, "how's the weather today?").await;
    assert_eq!(agent.agent_type(), "chitchat");
}

#[tokio::test]
async fn is_case_and_whitespace_insensitive() {
    let provider = FakeLlmProvider::success(text_response("  Money  \n"));
    let registry = registry();
    let agent = classify_intent(&provider, &registry, "budget please").await;
    assert_eq!(agent.agent_type(), "money");
}

#[tokio::test]
async fn unrecognized_text_falls_back_to_chitchat() {
    let provider = FakeLlmProvider::success(text_response("something else entirely"));
    let registry = registry();
    let agent = classify_intent(&provider, &registry, "anything").await;
    assert_eq!(agent.agent_type(), "chitchat");
}

#[tokio::test]
async fn provider_failure_falls_back_to_chitchat() {
    let provider = FakeLlmProvider::failure("provider down");
    let registry = registry();
    let agent = classify_intent(&provider, &registry, "anything").await;
    assert_eq!(agent.agent_type(), "chitchat");
}
