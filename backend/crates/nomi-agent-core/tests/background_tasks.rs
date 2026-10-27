use std::borrow::Cow;

use async_trait::async_trait;
use serde_json::Value;
use sqlx::pool::PoolConnection;
use sqlx::{PgPool, Postgres};
use uuid::Uuid;

use nomi_agent_core::{run_agent_turn, AgentRegistry, LoopOutcome, SubAgent, ToolOutcome};
use nomi_llm::{ContentBlock, LlmResponse, StopReason, ToolDefinition};
use nomi_test_support::{FakeEmbeddingProvider, FakeLlmProvider};

struct CrewSpecialistAgent;

#[async_trait]
impl SubAgent for CrewSpecialistAgent {
    fn agent_type(&self) -> Cow<'static, str> {
        Cow::Borrowed("money")
    }
    fn system_prompt(&self) -> Cow<'static, str> {
        Cow::Borrowed("You are the Money agent.")
    }
    fn tools(&self) -> Vec<ToolDefinition> {
        vec![ToolDefinition {
            name: "quick_note".to_string(),
            description: "Quick synchronous work while background task runs".to_string(),
            input_schema: serde_json::json!({ "type": "object", "properties": {} }),
        }]
    }
    async fn execute_tool(
        &self,
        _: &mut PoolConnection<Postgres>,
        _: Uuid,
        _: Uuid,
        _: Uuid,
        name: &str,
        _: Value,
    ) -> Result<ToolOutcome, String> {
        if name == "quick_note" {
            Ok(ToolOutcome::text("finished quick_note while background task runs"))
        } else {
            Err("unknown tool".to_string())
        }
    }
    fn intent_label(&self) -> Cow<'static, str> {
        Cow::Borrowed("money")
    }
    fn intent_description(&self) -> Cow<'static, str> {
        Cow::Borrowed("money specialist")
    }
    fn is_default(&self) -> bool {
        true
    }
}

async fn seed_session(pool: &PgPool) -> Uuid {
    let org_id: Uuid = sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Acme') RETURNING id")
        .fetch_one(pool)
        .await
        .unwrap();
    sqlx::query_scalar("INSERT INTO sessions (org_id, channel, chat_id) VALUES ($1, 'web', 'chat-bg') RETURNING id")
        .bind(org_id)
        .fetch_one(pool)
        .await
        .unwrap()
}

async fn seed_user(pool: &PgPool) -> Uuid {
    sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id")
        .fetch_one(pool)
        .await
        .unwrap()
}

#[sqlx::test(migrations = "../../migrations")]
async fn crew_agent_can_start_background_task_and_work_on_another_task_while_waiting(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let user_id = seed_user(&pool).await;
    let mut conn = pool.acquire().await.unwrap();

    let registry = AgentRegistry::new(vec![Box::new(CrewSpecialistAgent)]);
    let provider = FakeLlmProvider::sequence(vec![
        LlmResponse {
            content: vec![ContentBlock::ToolUse {
                id: "bg1".to_string(),
                name: "run_background_task".to_string(),
                input: serde_json::json!({
                    "label": "audit-export",
                    "command": "echo exporting && sleep 1"
                }),
                thought_signature: None,
            }],
            stop_reason: StopReason::ToolUse,
            input_tokens: 1,
            output_tokens: 1,
        },
        LlmResponse {
            content: vec![ContentBlock::ToolUse {
                id: "q1".to_string(),
                name: "quick_note".to_string(),
                input: serde_json::json!({}),
                thought_signature: None,
            }],
            stop_reason: StopReason::ToolUse,
            input_tokens: 1,
            output_tokens: 1,
        },
        LlmResponse {
            content: vec![ContentBlock::Text {
                text: "I started audit-export in the background and finished quick_note meanwhile. Nomi will follow up when audit-export completes!".to_string(),
            }],
            stop_reason: StopReason::EndTurn,
            input_tokens: 1,
            output_tokens: 1,
        },
    ]);
    let embedding_provider = FakeEmbeddingProvider::success(vec![0.0; 1536]);

    let outcome = run_agent_turn(
        &mut conn,
        None,
        None,
        &provider,
        &embedding_provider,
        &registry,
        &CrewSpecialistAgent,
        session_id,
        session_id,
        user_id,
        vec![],
        200,
    )
    .await
    .unwrap();

    assert!(matches!(outcome, LoopOutcome::Reply { .. }));

    let (status, agent_type, label, command): (String, String, String, String) = sqlx::query_as(
        "SELECT status, agent_type, label, command FROM background_tasks WHERE session_id = $1",
    )
    .bind(session_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(status, "pending");
    assert_eq!(agent_type, "money");
    assert_eq!(label, "audit-export");
    assert_eq!(command, "echo exporting && sleep 1");

    let requests = provider.received_requests.lock().unwrap();
    assert!(
        requests[0].system.as_deref().unwrap_or_default().contains("run_background_task"),
        "every crew agent system prompt must include BACKGROUND_EXECUTION_GUIDE"
    );
}
