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

fn registry() -> AgentRegistry {
    AgentRegistry::new(vec![Box::new(MoneyAgent), Box::new(ChitchatAgent)])
}

async fn reply_from(pool: &PgPool, message_id: Option<Uuid>) -> Option<String> {
    sqlx::query_scalar("SELECT agent_display_name FROM messages WHERE id = $1").bind(message_id.unwrap()).fetch_one(pool).await.unwrap()
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_named_member_answers_without_classifying(pool: PgPool) {
    // One reply only: had the turn classified first, "Logged." would have been read as the label.
    let provider = FakeLlmProvider::sequence(vec![text_response("Logged.")]);
    let embedder = FakeEmbeddingProvider::success(dummy_embedding());
    let catalog: Arc<ToolCatalog> = Arc::new(ToolCatalog::empty());

    let outcome = handle_inbound_message(&pool, None, &provider, &embedder, &registry(), &catalog, "telegram", "dm", "chat-1", "tg-1", "@dana coffee 25k", None)
        .await
        .unwrap();

    assert_eq!(reply_from(&pool, outcome.message_id).await, Some("Dana".to_string()));
    let sessions: i64 = sqlx::query_scalar("SELECT count(*) FROM agent_sessions WHERE agent_type = 'money' AND status = 'active'").fetch_one(&pool).await.unwrap();
    assert_eq!(sessions, 1);
}

#[sqlx::test(migrations = "../../migrations")]
async fn naming_another_member_ends_the_current_ones_session(pool: PgPool) {
    let provider = FakeLlmProvider::sequence(vec![text_response("Logged."), text_response("Hi!")]);
    let embedder = FakeEmbeddingProvider::success(dummy_embedding());
    let catalog: Arc<ToolCatalog> = Arc::new(ToolCatalog::empty());

    handle_inbound_message(&pool, None, &provider, &embedder, &registry(), &catalog, "telegram", "dm", "chat-1", "tg-1", "@Dana coffee 25k", None).await.unwrap();
    let outcome = handle_inbound_message(&pool, None, &provider, &embedder, &registry(), &catalog, "telegram", "dm", "chat-1", "tg-1", "@Nomi hello", None)
        .await
        .unwrap();

    assert_eq!(reply_from(&pool, outcome.message_id).await, Some("Nomi".to_string()));
    let active: i64 = sqlx::query_scalar("SELECT count(*) FROM agent_sessions WHERE status = 'active'").fetch_one(&pool).await.unwrap();
    assert_eq!(active, 0);
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_dynamic_member_is_named_by_its_configured_name(pool: PgPool) {
    let user_id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(&pool).await.unwrap();
    sqlx::query(
        "INSERT INTO dynamic_agents (name, system_prompt, intent_label, intent_description, granted_tools, is_active, created_by) \
         VALUES ('Sol', 'You report the weather.', 'weather', 'the user is asking about the weather', '{}', true, $1)",
    )
    .bind(user_id)
    .execute(&pool)
    .await
    .unwrap();
    let provider = FakeLlmProvider::sequence(vec![text_response("Sunny.")]);
    let embedder = FakeEmbeddingProvider::success(dummy_embedding());
    let catalog: Arc<ToolCatalog> = Arc::new(ToolCatalog::empty());

    let outcome = handle_inbound_message(&pool, None, &provider, &embedder, &registry(), &catalog, "telegram", "dm", "chat-1", "tg-1", "@sol tomorrow?", None)
        .await
        .unwrap();

    assert_eq!(reply_from(&pool, outcome.message_id).await, Some("Sol".to_string()));
}

#[sqlx::test(migrations = "../../migrations")]
async fn an_email_address_is_not_a_mention(pool: PgPool) {
    // Classified ("chitchat" isn't a specialist's label, so Nomi), then Nomi's reply.
    let provider = FakeLlmProvider::sequence(vec![text_response("chitchat"), text_response("Sure.")]);
    let embedder = FakeEmbeddingProvider::success(dummy_embedding());
    let catalog: Arc<ToolCatalog> = Arc::new(ToolCatalog::empty());

    let outcome = handle_inbound_message(&pool, None, &provider, &embedder, &registry(), &catalog, "telegram", "dm", "chat-1", "tg-1", "write to dana@example.com", None)
        .await
        .unwrap();

    assert_eq!(reply_from(&pool, outcome.message_id).await, Some("Nomi".to_string()));
}
