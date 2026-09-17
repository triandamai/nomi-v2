use std::borrow::Cow;

use sqlx::pool::PoolConnection;
use sqlx::{PgPool, Postgres};
use uuid::Uuid;

use nomi_llm::{ContentBlock, LlmResponse, StopReason, ToolDefinition};
use nomi_agent_core::{run_agent_turn, LoopOutcome, SubAgent, ToolOutcome, COMPLETE_TASK_TOOL_NAME};
use nomi_agent_core::AgentRegistry;
use nomi_agent_core::TurnError;

use nomi_test_support::{FakeEmbeddingProvider, FakeLlmProvider};

struct TestAgent;

#[async_trait::async_trait]
impl SubAgent for TestAgent {
    fn agent_type(&self) -> Cow<'static, str> {
        Cow::Borrowed("test")
    }
    fn system_prompt(&self) -> Cow<'static, str> {
        Cow::Borrowed("test prompt")
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
        _session_id: Uuid,
        _agent_session_id: Uuid,
        _user_id: Uuid,
        name: &str,
        input: serde_json::Value,
    ) -> Result<ToolOutcome, String> {
        match name {
            "echo" => Ok(ToolOutcome::text(format!("echoed: {input}"))),
            other => Err(format!("unknown tool: {other}")),
        }
    }
    fn intent_label(&self) -> Cow<'static, str> {
        Cow::Borrowed("test")
    }
    fn intent_description(&self) -> Cow<'static, str> {
        Cow::Borrowed("test agent")
    }
    fn is_default(&self) -> bool {
        true
    }
    fn supports_todos(&self) -> bool {
        true
    }
    fn supports_plans(&self) -> bool {
        true
    }
}

struct PersonalityAwareTestAgent;

#[async_trait::async_trait]
impl SubAgent for PersonalityAwareTestAgent {
    fn agent_type(&self) -> Cow<'static, str> {
        Cow::Borrowed("personality-aware-test")
    }
    fn system_prompt(&self) -> Cow<'static, str> {
        Cow::Borrowed("test prompt")
    }
    fn tools(&self) -> Vec<ToolDefinition> {
        vec![]
    }
    async fn execute_tool(
        &self,
        _conn: &mut PoolConnection<Postgres>,
        _session_id: Uuid,
        _agent_session_id: Uuid,
        _user_id: Uuid,
        name: &str,
        _input: serde_json::Value,
    ) -> Result<ToolOutcome, String> {
        Err(format!("unknown tool: {name}"))
    }
    fn intent_label(&self) -> Cow<'static, str> {
        Cow::Borrowed("personality-aware-test")
    }
    fn intent_description(&self) -> Cow<'static, str> {
        Cow::Borrowed("test agent")
    }
    fn uses_personality(&self) -> bool {
        true
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
        content: vec![ContentBlock::ToolUse { id: id.to_string(), name: name.to_string(), input, thought_signature: None }],
        stop_reason: StopReason::ToolUse,
        input_tokens: 1,
        output_tokens: 1,
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn end_turn_without_any_tool_use_returns_a_plain_reply(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let (user_id, agent_session_id) = seed_agent_session(&pool, session_id).await;
    let mut conn = pool.acquire().await.unwrap();

    let provider = FakeLlmProvider::sequence(vec![text_response("Hello!", StopReason::EndTurn)]);
    let embedding_provider = FakeEmbeddingProvider::success(vec![0.0; 1536]);
    let registry = AgentRegistry::new(vec![Box::new(TestAgent)]);

    let outcome = run_agent_turn(
        &mut conn,
        None,
        None,
        &provider,
        &embedding_provider,
        &registry,
        &TestAgent,
        session_id,
        agent_session_id,
        user_id,
        vec![],
        100,
    )
    .await
    .unwrap();

    assert_eq!(
        outcome,
        LoopOutcome::Reply { text: "Hello!".to_string(), memory_ids_used: vec![], input_tokens: 1, output_tokens: 1 }
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_tool_use_is_executed_and_its_result_fed_back(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let (user_id, agent_session_id) = seed_agent_session(&pool, session_id).await;
    let mut conn = pool.acquire().await.unwrap();

    // Permission-gated by default (Task 5) — allow `echo` so this test still exercises normal
    // tool execution rather than the approval pause.
    sqlx::query("INSERT INTO tool_permission_rules (user_id, tool_name, decision) VALUES ($1, 'echo', 'allow')")
        .bind(user_id)
        .execute(&pool)
        .await
        .unwrap();

    let provider = FakeLlmProvider::sequence(vec![
        tool_use_response("t1", "echo", serde_json::json!({"x": 1})),
        text_response("Done!", StopReason::EndTurn),
    ]);
    let embedding_provider = FakeEmbeddingProvider::success(vec![0.0; 1536]);
    let registry = AgentRegistry::new(vec![Box::new(TestAgent)]);

    let outcome = run_agent_turn(
        &mut conn,
        None,
        None,
        &provider,
        &embedding_provider,
        &registry,
        &TestAgent,
        session_id,
        agent_session_id,
        user_id,
        vec![],
        100,
    )
    .await
    .unwrap();

    assert_eq!(
        outcome,
        LoopOutcome::Reply { text: "Done!".to_string(), memory_ids_used: vec![], input_tokens: 1, output_tokens: 1 }
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn complete_task_terminates_the_loop_with_a_completed_outcome(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let (user_id, agent_session_id) = seed_agent_session(&pool, session_id).await;
    let mut conn = pool.acquire().await.unwrap();

    let provider = FakeLlmProvider::sequence(vec![tool_use_response(
        COMPLETE_TASK_TOOL_NAME,
        COMPLETE_TASK_TOOL_NAME,
        serde_json::json!({"status": "completed", "summary": "All done"}),
    )]);
    let embedding_provider = FakeEmbeddingProvider::success(vec![0.0; 1536]);
    let registry = AgentRegistry::new(vec![Box::new(TestAgent)]);

    let outcome = run_agent_turn(
        &mut conn,
        None,
        None,
        &provider,
        &embedding_provider,
        &registry,
        &TestAgent,
        session_id,
        agent_session_id,
        user_id,
        vec![],
        100,
    )
    .await
    .unwrap();

    assert_eq!(
        outcome,
        LoopOutcome::Completed { status: "completed".to_string(), summary: "All done".to_string() }
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn exceeding_the_turn_cap_returns_tool_loop_exceeded(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let (user_id, agent_session_id) = seed_agent_session(&pool, session_id).await;
    let mut conn = pool.acquire().await.unwrap();

    // Permission-gated by default (Task 5) — allow `echo` so this test still exercises the
    // tool-turn cap rather than the approval pause.
    sqlx::query("INSERT INTO tool_permission_rules (user_id, tool_name, decision) VALUES ($1, 'echo', 'allow')")
        .bind(user_id)
        .execute(&pool)
        .await
        .unwrap();

    let responses: Vec<LlmResponse> =
        (0..11).map(|i| tool_use_response(&format!("t{i}"), "echo", serde_json::json!({}))).collect();
    let provider = FakeLlmProvider::sequence(responses);
    let embedding_provider = FakeEmbeddingProvider::success(vec![0.0; 1536]);
    let registry = AgentRegistry::new(vec![Box::new(TestAgent)]);

    let result = run_agent_turn(
        &mut conn,
        None,
        None,
        &provider,
        &embedding_provider,
        &registry,
        &TestAgent,
        session_id,
        agent_session_id,
        user_id,
        vec![],
        100,
    )
    .await;

    assert!(matches!(result, Err(TurnError::ToolLoopExceeded)));
}

#[sqlx::test(migrations = "../../migrations")]
async fn every_tool_call_is_logged_as_a_tool_called_event(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let (user_id, agent_session_id) = seed_agent_session(&pool, session_id).await;
    let mut conn = pool.acquire().await.unwrap();

    // Permission-gated by default (Task 5) — allow `echo` so this test still exercises normal
    // tool execution/logging rather than the approval pause.
    sqlx::query("INSERT INTO tool_permission_rules (user_id, tool_name, decision) VALUES ($1, 'echo', 'allow')")
        .bind(user_id)
        .execute(&pool)
        .await
        .unwrap();

    let provider = FakeLlmProvider::sequence(vec![
        tool_use_response("t1", "echo", serde_json::json!({"x": 1})),
        tool_use_response(
            COMPLETE_TASK_TOOL_NAME,
            COMPLETE_TASK_TOOL_NAME,
            serde_json::json!({"status": "completed", "summary": "done"}),
        ),
    ]);
    let embedding_provider = FakeEmbeddingProvider::success(vec![0.0; 1536]);
    let registry = AgentRegistry::new(vec![Box::new(TestAgent)]);

    run_agent_turn(
        &mut conn,
        None,
        None,
        &provider,
        &embedding_provider,
        &registry,
        &TestAgent,
        session_id,
        agent_session_id,
        user_id,
        vec![],
        100,
    )
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

#[sqlx::test(migrations = "../../migrations")]
async fn every_llm_request_asks_for_reasoning(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let (user_id, agent_session_id) = seed_agent_session(&pool, session_id).await;
    let mut conn = pool.acquire().await.unwrap();

    let provider = FakeLlmProvider::sequence(vec![text_response("Hello!", StopReason::EndTurn)]);
    let embedding_provider = FakeEmbeddingProvider::success(vec![0.0; 1536]);
    let registry = AgentRegistry::new(vec![Box::new(TestAgent)]);

    run_agent_turn(
        &mut conn,
        None,
        None,
        &provider,
        &embedding_provider,
        &registry,
        &TestAgent,
        session_id,
        agent_session_id,
        user_id,
        vec![],
        100,
    )
    .await
    .unwrap();

    let requests = provider.received_requests.lock().unwrap();
    assert!(requests[0].enable_reasoning);
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_thinking_block_is_posted_as_activity_even_for_an_agent_that_does_not_surface_activity(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let (user_id, agent_session_id) = seed_agent_session(&pool, session_id).await;
    let mut conn = pool.acquire().await.unwrap();

    // TestAgent doesn't override `surfaces_activity` (defaults to false) — reasoning must show
    // up regardless, unlike the tool-call "💭" commentary which stays gated behind that flag.
    let provider = FakeLlmProvider::sequence(vec![LlmResponse {
        content: vec![
            ContentBlock::Thinking { text: "working through it".to_string(), signature: None },
            ContentBlock::Text { text: "Hello!".to_string() },
        ],
        stop_reason: StopReason::EndTurn,
        input_tokens: 1,
        output_tokens: 1,
    }]);
    let embedding_provider = FakeEmbeddingProvider::success(vec![0.0; 1536]);
    let registry = AgentRegistry::new(vec![Box::new(TestAgent)]);

    run_agent_turn(
        &mut conn,
        None,
        None,
        &provider,
        &embedding_provider,
        &registry,
        &TestAgent,
        session_id,
        agent_session_id,
        user_id,
        vec![],
        100,
    )
    .await
    .unwrap();

    let content: String = sqlx::query_scalar(
        "SELECT content FROM messages WHERE session_id = $1 AND sender_channel_identity_id IS NULL",
    )
    .bind(session_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(content, "🧠 working through it");
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_stored_personality_is_folded_into_the_system_prompt_when_uses_personality_is_true(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let (user_id, agent_session_id) = seed_agent_session(&pool, session_id).await;
    sqlx::query("INSERT INTO user_personality (user_id, description) VALUES ($1, $2)")
        .bind(user_id)
        .bind("Be sarcastic and blunt.")
        .execute(&pool)
        .await
        .unwrap();
    let mut conn = pool.acquire().await.unwrap();

    let provider = FakeLlmProvider::sequence(vec![text_response("Fine.", StopReason::EndTurn)]);
    let embedding_provider = FakeEmbeddingProvider::success(vec![0.0; 1536]);
    let registry = AgentRegistry::new(vec![Box::new(PersonalityAwareTestAgent)]);

    run_agent_turn(
        &mut conn,
        None,
        None,
        &provider,
        &embedding_provider,
        &registry,
        &PersonalityAwareTestAgent,
        session_id,
        agent_session_id,
        user_id,
        vec![],
        100,
    )
    .await
    .unwrap();

    let requests = provider.received_requests.lock().unwrap();
    let system = requests[0].system.as_ref().unwrap();
    assert!(system.contains("Adopt this personality in your replies: Be sarcastic and blunt."));
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_stored_personality_is_not_folded_in_for_an_agent_that_does_not_opt_in(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let (user_id, agent_session_id) = seed_agent_session(&pool, session_id).await;
    sqlx::query("INSERT INTO user_personality (user_id, description) VALUES ($1, $2)")
        .bind(user_id)
        .bind("Be sarcastic and blunt.")
        .execute(&pool)
        .await
        .unwrap();
    let mut conn = pool.acquire().await.unwrap();

    let provider = FakeLlmProvider::sequence(vec![text_response("Fine.", StopReason::EndTurn)]);
    let embedding_provider = FakeEmbeddingProvider::success(vec![0.0; 1536]);
    let registry = AgentRegistry::new(vec![Box::new(TestAgent)]);

    run_agent_turn(
        &mut conn,
        None,
        None,
        &provider,
        &embedding_provider,
        &registry,
        &TestAgent,
        session_id,
        agent_session_id,
        user_id,
        vec![],
        100,
    )
    .await
    .unwrap();

    let requests = provider.received_requests.lock().unwrap();
    assert_eq!(requests[0].system.as_ref().unwrap(), "test prompt");
}

#[sqlx::test(migrations = "../../migrations")]
async fn show_table_is_available_to_every_agent_and_posts_a_table_block(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let (user_id, agent_session_id) = seed_agent_session(&pool, session_id).await;
    let mut conn = pool.acquire().await.unwrap();

    let provider = FakeLlmProvider::sequence(vec![
        tool_use_response(
            "t1",
            "show_table",
            serde_json::json!({
                "variant": "data",
                "columns": [{"key": "name", "label": "Name"}],
                "rows": [{"name": "Alice"}]
            }),
        ),
        text_response("Done!", StopReason::EndTurn),
    ]);
    let embedding_provider = FakeEmbeddingProvider::success(vec![0.0; 1536]);
    let registry = AgentRegistry::new(vec![Box::new(TestAgent)]);

    run_agent_turn(&mut conn, None, None, &provider, &embedding_provider, &registry, &TestAgent, session_id, agent_session_id, user_id, vec![], 100)
        .await
        .unwrap();

    let content_blocks: Option<serde_json::Value> = sqlx::query_scalar(
        "SELECT content_blocks FROM messages WHERE session_id = $1 AND sender_channel_identity_id IS NULL ORDER BY created_at DESC LIMIT 1",
    )
    .bind(session_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    let blocks = content_blocks.unwrap();
    assert_eq!(blocks[0]["kind"], "table");
    assert_eq!(blocks[0]["rows"][0]["name"], "Alice");
}

#[sqlx::test(migrations = "../../migrations")]
async fn update_todos_upserts_the_same_message_instead_of_creating_a_new_one_each_time(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let (user_id, agent_session_id) = seed_agent_session(&pool, session_id).await;
    let mut conn = pool.acquire().await.unwrap();

    let todo_input = |status: &str| {
        serde_json::json!({"items": [{"id": "1", "text": "Write index.html", "status": status}]})
    };

    let provider = FakeLlmProvider::sequence(vec![
        tool_use_response("t1", "update_todos", todo_input("pending")),
        text_response("ok", StopReason::EndTurn),
    ]);
    let embedding_provider = FakeEmbeddingProvider::success(vec![0.0; 1536]);
    let registry = AgentRegistry::new(vec![Box::new(TestAgent)]);

    run_agent_turn(&mut conn, None, None, &provider, &embedding_provider, &registry, &TestAgent, session_id, agent_session_id, user_id, vec![], 100)
        .await
        .unwrap();

    let provider2 = FakeLlmProvider::sequence(vec![
        tool_use_response("t2", "update_todos", todo_input("done")),
        text_response("ok again", StopReason::EndTurn),
    ]);
    run_agent_turn(&mut conn, None, None, &provider2, &embedding_provider, &registry, &TestAgent, session_id, agent_session_id, user_id, vec![], 100)
        .await
        .unwrap();

    let todo_message_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM messages WHERE session_id = $1 AND content_blocks IS NOT NULL AND content_blocks @> '[{\"kind\": \"todo_list\"}]'",
    )
    .bind(session_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(todo_message_count, 1, "the second update_todos call should update the same message, not create a second one");

    let status: String = sqlx::query_scalar(
        "SELECT content_blocks->0->'items'->0->>'status' FROM messages WHERE session_id = $1 AND content_blocks @> '[{\"kind\": \"todo_list\"}]'",
    )
    .bind(session_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(status, "done");
}

#[sqlx::test(migrations = "../../migrations")]
async fn an_unmatched_tool_call_pauses_the_turn_and_never_executes(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let (user_id, agent_session_id) = seed_agent_session(&pool, session_id).await;
    let mut conn = pool.acquire().await.unwrap();

    let provider = FakeLlmProvider::sequence(vec![tool_use_response("t1", "echo", serde_json::json!({"x": 1}))]);
    let embedding_provider = FakeEmbeddingProvider::success(vec![0.0; 1536]);
    let registry = AgentRegistry::new(vec![Box::new(TestAgent)]);

    let outcome = run_agent_turn(&mut conn, None, None, &provider, &embedding_provider, &registry, &TestAgent, session_id, agent_session_id, user_id, vec![], 100)
        .await
        .unwrap();

    let message_id = match outcome {
        LoopOutcome::AwaitingApproval { message_id } => message_id,
        other => panic!("expected AwaitingApproval, got {other:?}"),
    };

    let content_blocks: serde_json::Value = sqlx::query_scalar("SELECT content_blocks FROM messages WHERE id = $1")
        .bind(message_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(content_blocks[0]["kind"], "approval_request");
    assert_eq!(content_blocks[0]["status"], "pending");
    assert_eq!(content_blocks[0]["tool_name"], "echo");

    let state: serde_json::Value = sqlx::query_scalar("SELECT state FROM agent_sessions WHERE id = $1")
        .bind(agent_session_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(state["paused_for_approval"], true);
    assert_eq!(state["pending_tool_use_id"], "t1");
}

#[sqlx::test(migrations = "../../migrations")]
async fn an_allow_rule_lets_the_tool_execute_without_pausing(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let (user_id, agent_session_id) = seed_agent_session(&pool, session_id).await;
    let mut conn = pool.acquire().await.unwrap();

    sqlx::query("INSERT INTO tool_permission_rules (user_id, tool_name, decision) VALUES ($1, 'echo', 'allow')")
        .bind(user_id)
        .execute(&pool)
        .await
        .unwrap();

    let provider = FakeLlmProvider::sequence(vec![
        tool_use_response("t1", "echo", serde_json::json!({"x": 1})),
        text_response("Done!", StopReason::EndTurn),
    ]);
    let embedding_provider = FakeEmbeddingProvider::success(vec![0.0; 1536]);
    let registry = AgentRegistry::new(vec![Box::new(TestAgent)]);

    let outcome = run_agent_turn(&mut conn, None, None, &provider, &embedding_provider, &registry, &TestAgent, session_id, agent_session_id, user_id, vec![], 100)
        .await
        .unwrap();

    assert_eq!(outcome, LoopOutcome::Reply { text: "Done!".to_string(), memory_ids_used: vec![], input_tokens: 1, output_tokens: 1 });
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_deny_rule_blocks_the_tool_but_lets_the_turn_continue(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let (user_id, agent_session_id) = seed_agent_session(&pool, session_id).await;
    let mut conn = pool.acquire().await.unwrap();

    sqlx::query("INSERT INTO tool_permission_rules (user_id, tool_name, decision) VALUES ($1, 'echo', 'deny')")
        .bind(user_id)
        .execute(&pool)
        .await
        .unwrap();

    let provider = FakeLlmProvider::sequence(vec![
        tool_use_response("t1", "echo", serde_json::json!({"x": 1})),
        text_response("Okay, I won't.", StopReason::EndTurn),
    ]);
    let embedding_provider = FakeEmbeddingProvider::success(vec![0.0; 1536]);
    let registry = AgentRegistry::new(vec![Box::new(TestAgent)]);

    let outcome = run_agent_turn(&mut conn, None, None, &provider, &embedding_provider, &registry, &TestAgent, session_id, agent_session_id, user_id, vec![], 100)
        .await
        .unwrap();

    assert_eq!(outcome, LoopOutcome::Reply { text: "Okay, I won't.".to_string(), memory_ids_used: vec![], input_tokens: 1, output_tokens: 1 });

    let requests = provider.received_requests.lock().unwrap();
    let second_request_text = format!("{:?}", requests[1].messages);
    assert!(second_request_text.contains("Denied by your permission rules."));
}

#[sqlx::test(migrations = "../../migrations")]
async fn show_table_and_update_todos_are_never_gated(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let (user_id, agent_session_id) = seed_agent_session(&pool, session_id).await;
    let mut conn = pool.acquire().await.unwrap();

    // No permission rules at all — if these were gated the same way `echo` is, this would pause.
    let provider = FakeLlmProvider::sequence(vec![
        tool_use_response("t1", "show_table", serde_json::json!({"variant": "data", "columns": [], "rows": []})),
        text_response("Done!", StopReason::EndTurn),
    ]);
    let embedding_provider = FakeEmbeddingProvider::success(vec![0.0; 1536]);
    let registry = AgentRegistry::new(vec![Box::new(TestAgent)]);

    let outcome = run_agent_turn(&mut conn, None, None, &provider, &embedding_provider, &registry, &TestAgent, session_id, agent_session_id, user_id, vec![], 100)
        .await
        .unwrap();

    assert!(matches!(outcome, LoopOutcome::Reply { .. }));
}

#[sqlx::test(migrations = "../../migrations")]
async fn write_plan_is_only_available_to_agents_that_opt_in(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let (user_id, agent_session_id) = seed_agent_session(&pool, session_id).await;
    let mut conn = pool.acquire().await.unwrap();

    // PersonalityAwareTestAgent does not override supports_plans(), so it stays false —
    // calling write_plan should come back as an unrecognized tool, same as calling any tool an
    // agent never registered.
    let provider = FakeLlmProvider::sequence(vec![
        tool_use_response("t1", "write_plan", serde_json::json!({"title": "x", "content": "y"})),
        text_response("ok", StopReason::EndTurn),
    ]);
    let embedding_provider = FakeEmbeddingProvider::success(vec![0.0; 1536]);
    let registry = AgentRegistry::new(vec![Box::new(PersonalityAwareTestAgent)]);

    let outcome = run_agent_turn(&mut conn, None, None, &provider, &embedding_provider, &registry, &PersonalityAwareTestAgent, session_id, agent_session_id, user_id, vec![], 100)
        .await
        .unwrap();

    // The fake provider only queues 2 responses and PersonalityAwareTestAgent never actually
    // calls write_plan for real (the tool was never in its `tools` list, so a well-behaved LLM
    // wouldn't call it) — this test instead asserts no `write_plan` row appears in agent_plans
    // for an agent that never opted in, proving the tool registration itself is gated correctly.
    assert!(matches!(outcome, LoopOutcome::Reply { .. }));
    let plan_count: i64 = sqlx::query_scalar("SELECT count(*) FROM agent_plans WHERE agent_session_id = $1")
        .bind(agent_session_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(plan_count, 0);
}

#[sqlx::test(migrations = "../../migrations")]
async fn write_plan_creates_a_new_version_each_call_and_posts_a_message_with_the_full_body(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let (user_id, agent_session_id) = seed_agent_session(&pool, session_id).await;
    let mut conn = pool.acquire().await.unwrap();

    let provider = FakeLlmProvider::sequence(vec![
        tool_use_response("t1", "write_plan", serde_json::json!({"title": "Build the login page", "content": "1. Add form\n2. Wire auth"})),
        text_response("Wrote the plan.", StopReason::EndTurn),
    ]);
    let embedding_provider = FakeEmbeddingProvider::success(vec![0.0; 1536]);
    let registry = AgentRegistry::new(vec![Box::new(TestAgent)]);

    run_agent_turn(&mut conn, None, None, &provider, &embedding_provider, &registry, &TestAgent, session_id, agent_session_id, user_id, vec![], 100)
        .await
        .unwrap();

    let (title, version, content, content_s3_key): (String, i32, Option<String>, Option<String>) = sqlx::query_as(
        "SELECT title, version, content, content_s3_key FROM agent_plans WHERE agent_session_id = $1",
    )
    .bind(agent_session_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(title, "Build the login page");
    assert_eq!(version, 1);
    assert_eq!(content.as_deref(), Some("1. Add form\n2. Wire auth"));
    assert!(content_s3_key.is_none(), "no S3 configured in this test, so content must be stored inline");

    let (message_content, content_blocks): (String, serde_json::Value) = sqlx::query_as(
        "SELECT content, content_blocks FROM messages WHERE session_id = $1 AND content_blocks @> '[{\"kind\": \"plan\"}]'",
    )
    .bind(session_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(message_content.contains("Build the login page"));
    assert!(message_content.contains("1. Add form\n2. Wire auth"), "the posted message's plain-text content must carry the FULL plan body, so it stays in the LLM's own context on later turns");
    assert_eq!(content_blocks[0]["kind"], "plan");
    assert_eq!(content_blocks[0]["version"], 1);
    assert_eq!(content_blocks[0]["agent_session_id"], agent_session_id.to_string());
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_second_write_plan_call_creates_version_two_not_a_second_version_one(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let (user_id, agent_session_id) = seed_agent_session(&pool, session_id).await;
    let mut conn = pool.acquire().await.unwrap();

    let provider = FakeLlmProvider::sequence(vec![
        tool_use_response("t1", "write_plan", serde_json::json!({"title": "v1", "content": "first draft"})),
        text_response("ok", StopReason::EndTurn),
    ]);
    let embedding_provider = FakeEmbeddingProvider::success(vec![0.0; 1536]);
    let registry = AgentRegistry::new(vec![Box::new(TestAgent)]);
    run_agent_turn(&mut conn, None, None, &provider, &embedding_provider, &registry, &TestAgent, session_id, agent_session_id, user_id, vec![], 100)
        .await
        .unwrap();

    let provider2 = FakeLlmProvider::sequence(vec![
        tool_use_response("t2", "write_plan", serde_json::json!({"title": "v2", "content": "revised"})),
        text_response("ok", StopReason::EndTurn),
    ]);
    run_agent_turn(&mut conn, None, None, &provider2, &embedding_provider, &registry, &TestAgent, session_id, agent_session_id, user_id, vec![], 100)
        .await
        .unwrap();

    let versions: Vec<i32> = sqlx::query_scalar("SELECT version FROM agent_plans WHERE agent_session_id = $1 ORDER BY version")
        .bind(agent_session_id)
        .fetch_all(&pool)
        .await
        .unwrap();
    assert_eq!(versions, vec![1, 2]);

    let plan_message_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM messages WHERE session_id = $1 AND content_blocks @> '[{\"kind\": \"plan\"}]'",
    )
    .bind(session_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(plan_message_count, 2, "unlike update_todos, each write_plan call posts a NEW message — history, not an upsert");
}

#[sqlx::test(migrations = "../../migrations")]
async fn run_agent_turn_sets_phase_to_thinking_then_waiting_on_a_plain_reply(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let (user_id, agent_session_id) = seed_agent_session(&pool, session_id).await;
    let mut conn = pool.acquire().await.unwrap();

    let provider = FakeLlmProvider::sequence(vec![text_response("Hello!", StopReason::EndTurn)]);
    let embedding_provider = FakeEmbeddingProvider::success(vec![0.0; 1536]);
    let registry = AgentRegistry::new(vec![Box::new(TestAgent)]);

    run_agent_turn(&mut conn, None, None, &provider, &embedding_provider, &registry, &TestAgent, session_id, agent_session_id, user_id, vec![], 100)
        .await
        .unwrap();

    let (phase, detail): (String, Option<String>) =
        sqlx::query_as("SELECT current_phase, current_phase_detail FROM agent_sessions WHERE id = $1")
            .bind(agent_session_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(phase, "waiting");
    assert!(detail.is_none());
}

#[sqlx::test(migrations = "../../migrations")]
async fn resolve_tool_batch_sets_phase_to_calling_tool_with_the_tool_name_as_detail(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let (user_id, agent_session_id) = seed_agent_session(&pool, session_id).await;
    let mut conn = pool.acquire().await.unwrap();

    sqlx::query("INSERT INTO tool_permission_rules (user_id, tool_name, decision) VALUES ($1, 'echo', 'allow')")
        .bind(user_id)
        .execute(&pool)
        .await
        .unwrap();

    let registry = AgentRegistry::new(vec![Box::new(TestAgent)]);
    let tool_use_blocks =
        vec![ContentBlock::ToolUse { id: "t1".to_string(), name: "echo".to_string(), input: serde_json::json!({"x": 1}), thought_signature: None }];

    // Called directly (not via run_agent_turn) so the phase this write leaves behind isn't
    // immediately overwritten by the next loop iteration's "thinking" update.
    nomi_agent_core::resolve_tool_batch(
        &mut conn, None, None, &registry, &TestAgent, session_id, agent_session_id, user_id, &tool_use_blocks, &[], &[],
    )
    .await
    .unwrap();

    let (phase, detail): (String, Option<String>) =
        sqlx::query_as("SELECT current_phase, current_phase_detail FROM agent_sessions WHERE id = $1")
            .bind(agent_session_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(phase, "calling_tool");
    assert_eq!(detail.as_deref(), Some("echo"));
}
