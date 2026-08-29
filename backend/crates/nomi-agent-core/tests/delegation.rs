use async_trait::async_trait;
use serde_json::Value;
use sqlx::pool::PoolConnection;
use sqlx::{PgPool, Postgres};
use uuid::Uuid;

use nomi_llm::{ContentBlock, LlmResponse, StopReason, ToolDefinition};
use nomi_agent_core::{run_agent_turn, AgentRegistry, LoopOutcome, SubAgent};

use nomi_test_support::{FakeEmbeddingProvider, FakeLlmProvider};

struct DelegatingAgent;

#[async_trait]
impl SubAgent for DelegatingAgent {
    fn agent_type(&self) -> &'static str {
        "delegator"
    }
    fn system_prompt(&self) -> &'static str {
        "test"
    }
    fn tools(&self) -> Vec<ToolDefinition> {
        vec![]
    }
    async fn execute_tool(&self, _: &mut PoolConnection<Postgres>, _: Uuid, _: Uuid, _: Uuid, _: &str, _: Value) -> Result<String, String> {
        Err("no tools".to_string())
    }
    fn intent_label(&self) -> &'static str {
        "delegator"
    }
    fn intent_description(&self) -> &'static str {
        "test"
    }
    fn is_default(&self) -> bool {
        true
    }
    fn can_delegate(&self) -> bool {
        true
    }
}

struct TargetAgent;

#[async_trait]
impl SubAgent for TargetAgent {
    fn agent_type(&self) -> &'static str {
        "target"
    }
    fn system_prompt(&self) -> &'static str {
        "test"
    }
    fn tools(&self) -> Vec<ToolDefinition> {
        vec![]
    }
    async fn execute_tool(&self, _: &mut PoolConnection<Postgres>, _: Uuid, _: Uuid, _: Uuid, _: &str, _: Value) -> Result<String, String> {
        Err("no tools".to_string())
    }
    fn intent_label(&self) -> &'static str {
        "target"
    }
    fn intent_description(&self) -> &'static str {
        "test"
    }
}

async fn seed_session(pool: &PgPool) -> Uuid {
    let org_id: Uuid = sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Acme') RETURNING id")
        .fetch_one(pool)
        .await
        .unwrap();
    sqlx::query_scalar("INSERT INTO sessions (org_id, channel, chat_id) VALUES ($1, 'telegram', 'chat-1') RETURNING id")
        .bind(org_id)
        .fetch_one(pool)
        .await
        .unwrap()
}

async fn seed_user(pool: &PgPool) -> Uuid {
    sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(pool).await.unwrap()
}

fn delegate_tool_call_response() -> LlmResponse {
    LlmResponse {
        content: vec![ContentBlock::ToolUse {
            id: "t1".to_string(),
            name: "delegate_to_agent".to_string(),
            input: serde_json::json!({"target_agent": "target", "task": "look into it"}),
            thought_signature: None,
        }],
        stop_reason: StopReason::ToolUse,
        input_tokens: 1,
        output_tokens: 1,
    }
}

fn text_response(text: &str) -> LlmResponse {
    LlmResponse {
        content: vec![ContentBlock::Text { text: text.to_string() }],
        stop_reason: StopReason::EndTurn,
        input_tokens: 1,
        output_tokens: 1,
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn calling_delegate_to_agent_creates_a_pending_delegation_row(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let user_id = seed_user(&pool).await;
    let mut conn = pool.acquire().await.unwrap();

    let registry = AgentRegistry::new(vec![Box::new(DelegatingAgent), Box::new(TargetAgent)]);
    let provider = FakeLlmProvider::sequence(vec![
        delegate_tool_call_response(),
        text_response("I'll check and get back to you!"),
    ]);
    let embedding_provider = FakeEmbeddingProvider::success(vec![0.0; 1536]);

    let outcome = run_agent_turn(
        &mut conn,
        None,
        &provider,
        &embedding_provider,
        &registry,
        &DelegatingAgent,
        session_id,
        session_id,
        user_id,
        vec![],
        100,
    )
    .await
    .unwrap();

    assert_eq!(
        outcome,
        LoopOutcome::Reply {
            text: "I'll check and get back to you!".to_string(),
            memory_ids_used: vec![],
            input_tokens: 1,
            output_tokens: 1,
        }
    );

    let (status, target_agent_type, task): (String, String, String) = sqlx::query_as(
        "SELECT status, target_agent_type, task FROM agent_delegations WHERE session_id = $1",
    )
    .bind(session_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(status, "pending");
    assert_eq!(target_agent_type, "target");
    assert_eq!(task, "look into it");

    // The tool result fed back into the second LLM call must be a real acknowledgment, not an
    // error string — proving create_delegation's Ok path (not its Err path) was taken.
    let requests = provider.received_requests.lock().unwrap();
    let second_request = &requests[1];
    let tool_result_text = second_request
        .messages
        .iter()
        .flat_map(|m| &m.content)
        .find_map(|b| match b {
            ContentBlock::ToolResult { content, is_error, .. } => {
                assert!(!is_error);
                Some(content.clone())
            }
            _ => None,
        })
        .expect("expected a tool result in the second request");
    assert!(tool_result_text.contains("Delegated to target"));
}
