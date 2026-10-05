use std::sync::Arc;

use sqlx::PgPool;
use uuid::Uuid;

use nomi_agent_chitchat::ChitchatAgent;
use nomi_agent_core::{AgentRegistry, ToolCatalog};
use nomi_agent_files::FilesAgent;
use nomi_agent_money::MoneyAgent;
use nomi_llm::{ContentBlock, LlmResponse, StopReason};
use nomi_test_support::{dummy_embedding, FakeEmbeddingProvider, FakeLlmProvider};
use nomi_turn::handle_inbound_message;

fn text_response(text: &str) -> LlmResponse {
    LlmResponse { content: vec![ContentBlock::Text { text: text.to_string() }], stop_reason: StopReason::EndTurn, input_tokens: 1, output_tokens: 1 }
}

fn tool_use_response(id: &str, name: &str, input: serde_json::Value) -> LlmResponse {
    LlmResponse {
        content: vec![ContentBlock::ToolUse { id: id.to_string(), name: name.to_string(), input, thought_signature: None }],
        stop_reason: StopReason::ToolUse,
        input_tokens: 1,
        output_tokens: 1,
    }
}

async fn seed_speaker(pool: &PgPool, channel_user_id: &str) {
    let user_id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(pool).await.unwrap();
    let org_id: Uuid = sqlx::query_scalar("INSERT INTO organizations (name, is_personal) VALUES ('Personal', true) RETURNING id").fetch_one(pool).await.unwrap();
    sqlx::query("INSERT INTO memberships (org_id, user_id, role) VALUES ($1, $2, 'owner')").bind(org_id).bind(user_id).execute(pool).await.unwrap();
    sqlx::query("INSERT INTO channel_identities (user_id, channel, channel_user_id) VALUES ($1, 'telegram', $2)")
        .bind(user_id)
        .bind(channel_user_id)
        .execute(pool)
        .await
        .unwrap();
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_message_with_an_attachment_goes_to_the_files_agent_which_hands_work_on(pool: PgPool) {
    seed_speaker(&pool, "tg-1").await;
    let registry = AgentRegistry::new(vec![Box::new(ChitchatAgent), Box::new(MoneyAgent), Box::new(FilesAgent)]);
    let catalog: Arc<ToolCatalog> = Arc::new(ToolCatalog::empty());
    let embedder = FakeEmbeddingProvider::success(dummy_embedding());
    // No intent-classification call: the Files agent answers first.
    let provider = FakeLlmProvider::sequence(vec![
        tool_use_response(
            "t1",
            "delegate_to_agent",
            serde_json::json!({"target_agent": "money", "task": "Log these expenses: lunch 45000 (food), taxi 30000 (transport)"}),
        ),
        text_response("I've passed your receipts to Money to log them."),
    ]);

    let message = "log these please\n\n<attachment name=\"receipts.csv\" kind=\"file\">\nitem,amount\nlunch,45000\ntaxi,30000\n</attachment>";
    let outcome = handle_inbound_message(&pool, None, &provider, &embedder, &registry, &catalog, "telegram", "dm", "chat-1", "tg-1", message, None)
        .await
        .unwrap();

    assert_eq!(outcome.reply, "I've passed your receipts to Money to log them.");
    let author: String = sqlx::query_scalar("SELECT agent_display_name FROM messages WHERE id = $1").bind(outcome.message_id).fetch_one(&pool).await.unwrap();
    assert_eq!(author, "Files");
    let (requester, target, task): (String, String, String) =
        sqlx::query_as("SELECT requesting_agent_type, target_agent_type, task FROM agent_delegations").fetch_one(&pool).await.unwrap();
    assert_eq!((requester.as_str(), target.as_str()), ("files", "money"));
    assert!(task.contains("lunch 45000"), "the task carries the file's content: {task}");

    // The Files agent's own tool list offers the hand-off, but never back to itself.
    let request = provider.received_requests.lock().unwrap()[0].clone();
    let delegate = request.tools.iter().find(|t| t.name == "delegate_to_agent").expect("delegation is offered");
    let targets = delegate.input_schema["properties"]["target_agent"]["enum"].to_string();
    assert!(targets.contains("money") && !targets.contains("files"), "{targets}");
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_message_without_attachments_is_classified_as_usual(pool: PgPool) {
    seed_speaker(&pool, "tg-2").await;
    let registry = AgentRegistry::new(vec![Box::new(ChitchatAgent), Box::new(MoneyAgent), Box::new(FilesAgent)]);
    let catalog: Arc<ToolCatalog> = Arc::new(ToolCatalog::empty());
    let embedder = FakeEmbeddingProvider::success(dummy_embedding());
    let provider = FakeLlmProvider::sequence(vec![text_response("chitchat"), text_response("Hi!")]);

    let outcome = handle_inbound_message(&pool, None, &provider, &embedder, &registry, &catalog, "telegram", "dm", "chat-2", "tg-2", "hello there", None)
        .await
        .unwrap();
    assert_eq!(outcome.reply, "Hi!");
}
