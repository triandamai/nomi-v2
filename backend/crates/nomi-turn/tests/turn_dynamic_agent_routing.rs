use std::sync::Arc;

use sqlx::PgPool;
use uuid::Uuid;

use nomi_agent_chitchat::ChitchatAgent;
use nomi_agent_core::AgentRegistry;
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

async fn seed_user(pool: &PgPool) -> Uuid {
    sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(pool).await.unwrap()
}

#[sqlx::test(migrations = "../../migrations")]
async fn classifies_into_a_dynamic_agent_by_intent_label(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let mut conn = pool.acquire().await.unwrap();

    sqlx::query(
        "INSERT INTO dynamic_agents (name, system_prompt, intent_label, intent_description, granted_tools, is_active, created_by) \
         VALUES ('Weather Bot', 'You report the weather.', 'weather', 'the user is asking about the weather', '{}', true, $1)",
    )
    .bind(user_id)
    .execute(&mut *conn)
    .await
    .unwrap();

    let registry = AgentRegistry::new(vec![Box::new(ChitchatAgent)]);
    let catalog = Arc::new(nomi_agent_core::ToolCatalog::empty());
    let provider = FakeLlmProvider::success(text_response("weather"));

    let agent = classify_intent(&mut conn, &provider, &registry, &catalog, "what's the weather like?").await;
    assert_eq!(agent.intent_label(), "weather");
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_soft_disabled_dynamic_agent_is_excluded_from_classification(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let mut conn = pool.acquire().await.unwrap();

    sqlx::query(
        "INSERT INTO dynamic_agents (name, system_prompt, intent_label, intent_description, granted_tools, is_active, created_by) \
         VALUES ('Disabled Bot', 'p', 'disabled_intent', 'd', '{}', false, $1)",
    )
    .bind(user_id)
    .execute(&mut *conn)
    .await
    .unwrap();

    let registry = AgentRegistry::new(vec![Box::new(ChitchatAgent)]);
    let catalog = Arc::new(nomi_agent_core::ToolCatalog::empty());
    let provider = FakeLlmProvider::success(text_response("disabled_intent"));

    let agent = classify_intent(&mut conn, &provider, &registry, &catalog, "anything").await;
    // Falls back to the default agent — the classifier prompt never even offered
    // "disabled_intent" as an option, and the post-classification fallback match against
    // dynamic_rows only searches ACTIVE rows (fetch_active_dynamic_agents), so a model that
    // somehow still emits "disabled_intent" also can't match it.
    assert_eq!(agent.agent_type(), registry.default_agent().agent_type());
}
