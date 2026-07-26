mod support;

use sqlx::pool::PoolConnection;
use sqlx::{PgPool, Postgres};
use uuid::Uuid;

use nomi_orchestrator::llm::{ContentBlock, LlmResponse, StopReason, ToolDefinition};
use nomi_orchestrator::turn::subagent::SubAgent;
use nomi_orchestrator::turn::tools::{run_tool_calling_loop, LoopOutcome, COMPLETE_TASK_TOOL_NAME};
use nomi_orchestrator::turn::TurnError;

use support::FakeLlmProvider;

struct TestAgent;

#[async_trait::async_trait]
impl SubAgent for TestAgent {
    fn agent_type(&self) -> &'static str {
        "test"
    }
    fn system_prompt(&self) -> &'static str {
        "test prompt"
    }
    fn tools(&self) -> Vec<ToolDefinition> {
        vec![ToolDefinition {
            name: "echo".to_string(),
            description: "Echoes back its input".to_string(),
            input_schema: serde_json::json!({"type": "object"}),
        }]
    }
    async fn execute_tool(
        &self,
        _conn: &mut PoolConnection<Postgres>,
        _user_id: Uuid,
        name: &str,
        input: serde_json::Value,
    ) -> Result<String, String> {
        match name {
            "echo" => Ok(format!("echoed: {input}")),
            other => Err(format!("unknown tool: {other}")),
        }
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

async fn seed_agent_session(pool: &PgPool, session_id: Uuid) -> (Uuid, Uuid) {
    let user_id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id")
        .fetch_one(pool)
        .await
        .unwrap();
    let identity_id: Uuid = sqlx::query_scalar(
        "INSERT INTO channel_identities (user_id, channel, channel_user_id) VALUES ($1, 'telegram', 'u1') RETURNING id",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await
    .unwrap();
    let agent_session_id: Uuid = sqlx::query_scalar(
        "INSERT INTO agent_sessions (session_id, sender_channel_identity_id, agent_type, status) VALUES ($1, $2, 'test', 'active') RETURNING id",
    )
    .bind(session_id)
    .bind(identity_id)
    .fetch_one(pool)
    .await
    .unwrap();
    (user_id, agent_session_id)
}

fn text_response(text: &str, stop_reason: StopReason) -> LlmResponse {
    LlmResponse {
        content: vec![ContentBlock::Text { text: text.to_string() }],
        stop_reason,
        input_tokens: 1,
        output_tokens: 1,
    }
}

fn tool_use_response(id: &str, name: &str, input: serde_json::Value) -> LlmResponse {
    LlmResponse {
        content: vec![ContentBlock::ToolUse { id: id.to_string(), name: name.to_string(), input }],
        stop_reason: StopReason::ToolUse,
        input_tokens: 1,
        output_tokens: 1,
    }
}

#[sqlx::test]
async fn end_turn_without_any_tool_use_returns_a_plain_reply(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let (user_id, agent_session_id) = seed_agent_session(&pool, session_id).await;
    let mut conn = pool.acquire().await.unwrap();

    let provider = FakeLlmProvider::sequence(vec![text_response("Hello!", StopReason::EndTurn)]);

    let outcome = run_tool_calling_loop(&mut conn, &provider, &TestAgent, session_id, agent_session_id, user_id, vec![], 100)
        .await
        .unwrap();

    assert_eq!(outcome, LoopOutcome::Reply("Hello!".to_string()));
}

#[sqlx::test]
async fn a_tool_use_is_executed_and_its_result_fed_back(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let (user_id, agent_session_id) = seed_agent_session(&pool, session_id).await;
    let mut conn = pool.acquire().await.unwrap();

    let provider = FakeLlmProvider::sequence(vec![
        tool_use_response("t1", "echo", serde_json::json!({"x": 1})),
        text_response("Done!", StopReason::EndTurn),
    ]);

    let outcome = run_tool_calling_loop(&mut conn, &provider, &TestAgent, session_id, agent_session_id, user_id, vec![], 100)
        .await
        .unwrap();

    assert_eq!(outcome, LoopOutcome::Reply("Done!".to_string()));
}

#[sqlx::test]
async fn complete_task_terminates_the_loop_with_a_completed_outcome(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let (user_id, agent_session_id) = seed_agent_session(&pool, session_id).await;
    let mut conn = pool.acquire().await.unwrap();

    let provider = FakeLlmProvider::sequence(vec![tool_use_response(
        COMPLETE_TASK_TOOL_NAME,
        COMPLETE_TASK_TOOL_NAME,
        serde_json::json!({"status": "completed", "summary": "All done"}),
    )]);

    let outcome = run_tool_calling_loop(&mut conn, &provider, &TestAgent, session_id, agent_session_id, user_id, vec![], 100)
        .await
        .unwrap();

    assert_eq!(
        outcome,
        LoopOutcome::Completed { status: "completed".to_string(), summary: "All done".to_string() }
    );
}

#[sqlx::test]
async fn exceeding_the_turn_cap_returns_tool_loop_exceeded(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let (user_id, agent_session_id) = seed_agent_session(&pool, session_id).await;
    let mut conn = pool.acquire().await.unwrap();

    let responses: Vec<LlmResponse> =
        (0..11).map(|i| tool_use_response(&format!("t{i}"), "echo", serde_json::json!({}))).collect();
    let provider = FakeLlmProvider::sequence(responses);

    let result = run_tool_calling_loop(&mut conn, &provider, &TestAgent, session_id, agent_session_id, user_id, vec![], 100).await;

    assert!(matches!(result, Err(TurnError::ToolLoopExceeded)));
}

#[sqlx::test]
async fn every_tool_call_is_logged_as_a_tool_called_event(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let (user_id, agent_session_id) = seed_agent_session(&pool, session_id).await;
    let mut conn = pool.acquire().await.unwrap();

    let provider = FakeLlmProvider::sequence(vec![
        tool_use_response("t1", "echo", serde_json::json!({"x": 1})),
        tool_use_response(
            COMPLETE_TASK_TOOL_NAME,
            COMPLETE_TASK_TOOL_NAME,
            serde_json::json!({"status": "completed", "summary": "done"}),
        ),
    ]);

    run_tool_calling_loop(&mut conn, &provider, &TestAgent, session_id, agent_session_id, user_id, vec![], 100)
        .await
        .unwrap();

    let event_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM agent_events WHERE session_id = $1 AND agent_session_id = $2 AND event_type = 'ToolCalled'",
    )
    .bind(session_id)
    .bind(agent_session_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(event_count, 2);
}
