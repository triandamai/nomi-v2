use std::sync::Arc;

use sqlx::PgPool;
use uuid::Uuid;

use nomi_agent_chitchat::ChitchatAgent;
use nomi_agent_core::{AgentRegistry, ToolCatalog};
use nomi_agent_money::MoneyAgent;
use nomi_llm::{ContentBlock, LlmResponse, StopReason};
use nomi_test_support::{dummy_embedding, FakeEmbeddingProvider, FakeLlmProvider};
use nomi_turn::handle_inbound_message;

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
async fn a_chitchat_reply_is_tagged_nomi(pool: PgPool) {
    let provider = FakeLlmProvider::success(text_response("Hi there!"));
    let embedder = FakeEmbeddingProvider::success(dummy_embedding());
    let registry = AgentRegistry::new(vec![Box::new(MoneyAgent), Box::new(ChitchatAgent)]);
    let catalog: Arc<ToolCatalog> = Arc::new(ToolCatalog::empty());

    let outcome =
        handle_inbound_message(&pool, None, &provider, &embedder, &registry, &catalog, "telegram", "dm", "chat-1", "tg-1", "hello", None)
            .await
            .unwrap();

    let agent_display_name: Option<String> =
        sqlx::query_scalar("SELECT agent_display_name FROM messages WHERE id = $1")
            .bind(outcome.message_id.unwrap())
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(agent_display_name, Some("Nomi".to_string()));
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_specialist_agent_reply_is_tagged_with_its_title_cased_type(pool: PgPool) {
    // First LLM call classifies intent ("money"), second is the money agent's own reply.
    let provider = FakeLlmProvider::sequence(vec![text_response("money"), text_response("You spent $12 on coffee.")]);
    let embedder = FakeEmbeddingProvider::success(dummy_embedding());
    let registry = AgentRegistry::new(vec![Box::new(MoneyAgent), Box::new(ChitchatAgent)]);
    let catalog: Arc<ToolCatalog> = Arc::new(ToolCatalog::empty());

    let outcome = handle_inbound_message(
        &pool, None, &provider, &embedder, &registry, &catalog, "telegram", "dm", "chat-1", "tg-1", "how much did I spend?", None,
    )
    .await
    .unwrap();

    let agent_display_name: Option<String> =
        sqlx::query_scalar("SELECT agent_display_name FROM messages WHERE id = $1")
            .bind(outcome.message_id.unwrap())
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(agent_display_name, Some("Money".to_string()));
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_dynamic_agent_reply_is_tagged_with_its_configured_name(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    sqlx::query(
        "INSERT INTO dynamic_agents (name, system_prompt, intent_label, intent_description, granted_tools, is_active, created_by) \
         VALUES ('Weather Bot', 'You report the weather.', 'weather', 'the user is asking about the weather', '{}', true, $1)",
    )
    .bind(user_id)
    .execute(&pool)
    .await
    .unwrap();

    // First LLM call classifies intent ("weather"), second is the dynamic agent's own reply.
    let provider = FakeLlmProvider::sequence(vec![text_response("weather"), text_response("It's sunny today.")]);
    let embedder = FakeEmbeddingProvider::success(dummy_embedding());
    let registry = AgentRegistry::new(vec![Box::new(ChitchatAgent)]);
    let catalog: Arc<ToolCatalog> = Arc::new(ToolCatalog::empty());

    let outcome = handle_inbound_message(
        &pool, None, &provider, &embedder, &registry, &catalog, "telegram", "dm", "chat-1", "tg-1", "what's the weather?", None,
    )
    .await
    .unwrap();

    let agent_display_name: Option<String> =
        sqlx::query_scalar("SELECT agent_display_name FROM messages WHERE id = $1")
            .bind(outcome.message_id.unwrap())
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(agent_display_name, Some("Weather Bot".to_string()));
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_user_message_has_no_agent_display_name(pool: PgPool) {
    let provider = FakeLlmProvider::success(text_response("Hi there!"));
    let embedder = FakeEmbeddingProvider::success(dummy_embedding());
    let registry = AgentRegistry::new(vec![Box::new(MoneyAgent), Box::new(ChitchatAgent)]);
    let catalog: Arc<ToolCatalog> = Arc::new(ToolCatalog::empty());

    let outcome =
        handle_inbound_message(&pool, None, &provider, &embedder, &registry, &catalog, "telegram", "dm", "chat-1", "tg-1", "hello", None)
            .await
            .unwrap();

    let agent_display_name: Option<String> = sqlx::query_scalar(
        "SELECT agent_display_name FROM messages WHERE session_id = $1 AND sender_channel_identity_id IS NOT NULL",
    )
    .bind(outcome.session_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(agent_display_name, None);
}
