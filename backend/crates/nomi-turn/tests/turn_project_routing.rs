use std::sync::Arc;

use sqlx::PgPool;
use uuid::Uuid;

use nomi_agent_chitchat::ChitchatAgent;
use nomi_agent_core::{AgentRegistry, ToolCatalog};
use nomi_agent_money::MoneyAgent;
use nomi_agent_planning::PlanningAgent;
use nomi_llm::{ContentBlock, LlmResponse, StopReason};
use nomi_test_support::{dummy_embedding, FakeEmbeddingProvider, FakeLlmProvider};
use nomi_turn::handle_inbound_message;

fn reply(text: &str) -> LlmResponse {
    LlmResponse { content: vec![ContentBlock::Text { text: text.to_string() }], stop_reason: StopReason::EndTurn, input_tokens: 1, output_tokens: 1 }
}

fn registry() -> AgentRegistry {
    AgentRegistry::new(vec![Box::new(MoneyAgent), Box::new(PlanningAgent::new()), Box::new(ChitchatAgent)])
}

#[sqlx::test(migrations = "../../migrations")]
async fn asking_to_build_an_app_in_a_web_chat_starts_a_project_chat(pool: PgPool) {
    // The model would answer "money" if asked: the build check must win without asking it.
    let provider = FakeLlmProvider::success(reply("money"));
    let embedder = FakeEmbeddingProvider::success(dummy_embedding());
    let catalog: Arc<ToolCatalog> = Arc::new(ToolCatalog::empty());

    let outcome = handle_inbound_message(
        &pool, None, &provider, &embedder, &registry(), &catalog, "web", "dm", "chat-a", "web-user-1",
        "Create a web app to track my habits", None,
    )
    .await
    .unwrap();
    assert!(outcome.reply.contains("Create a web app to track my habits"), "{}", outcome.reply);

    // A new project chat, holding the request and a queued turn for Rena.
    let (project_id, project_session, name): (Uuid, Uuid, String) =
        sqlx::query_as("SELECT id, session_id, name FROM projects").fetch_one(&pool).await.unwrap();
    assert_ne!(project_session, outcome.session_id);
    assert_eq!(name, "Create a web app to track my habits");
    let moved: String = sqlx::query_scalar("SELECT content FROM messages WHERE session_id = $1").bind(project_session).fetch_one(&pool).await.unwrap();
    assert_eq!(moved, "Create a web app to track my habits");
    let queued: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM turn_jobs WHERE session_id = $1 AND status = 'pending'").bind(project_session).fetch_one(&pool).await.unwrap();
    assert_eq!(queued, 1);

    // The original chat gets a card linking to it, and no agent was started there.
    let blocks: serde_json::Value = sqlx::query_scalar("SELECT content_blocks FROM messages WHERE id = $1").bind(outcome.message_id.unwrap()).fetch_one(&pool).await.unwrap();
    assert_eq!(blocks[0]["kind"], "project_link");
    assert_eq!(blocks[0]["project_id"], project_id.to_string());
    assert_eq!(blocks[0]["session_id"], project_session.to_string());
    let agents: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM agent_sessions WHERE session_id = $1").bind(outcome.session_id).fetch_one(&pool).await.unwrap();
    assert_eq!(agents, 0);
}

#[sqlx::test(migrations = "../../migrations")]
async fn in_other_chats_a_build_request_goes_to_rena_even_mid_conversation(pool: PgPool) {
    let provider = FakeLlmProvider::sequence(vec![reply("money"), reply("Noted."), reply("Let's plan it.")]);
    let embedder = FakeEmbeddingProvider::success(dummy_embedding());
    let catalog: Arc<ToolCatalog> = Arc::new(ToolCatalog::empty());
    let registry = registry();

    // Talking to Finley first...
    let first = handle_inbound_message(&pool, None, &provider, &embedder, &registry, &catalog, "telegram", "dm", "chat-b", "tg-1", "I spent 20k on coffee", None)
        .await
        .unwrap();
    let money: String = sqlx::query_scalar("SELECT agent_type FROM agent_sessions WHERE session_id = $1 AND status = 'active'").bind(first.session_id).fetch_one(&pool).await.unwrap();
    assert_eq!(money, "money");

    // ...then asking for an app: Finley steps aside and Rena takes it, in this same chat.
    handle_inbound_message(&pool, None, &provider, &embedder, &registry, &catalog, "telegram", "dm", "chat-b", "tg-1", "bikinin aplikasi kasir dong", None)
        .await
        .unwrap();
    let active: Vec<String> = sqlx::query_scalar("SELECT agent_type FROM agent_sessions WHERE session_id = $1 AND status = 'active'").bind(first.session_id).fetch_all(&pool).await.unwrap();
    assert_eq!(active, ["planning"]);
    let projects: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM projects").fetch_one(&pool).await.unwrap();
    assert_eq!(projects, 0, "only the web has project chats");
}
