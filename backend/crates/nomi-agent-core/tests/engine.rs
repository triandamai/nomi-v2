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
    fn keeps_plans_in_drafts(&self) -> bool {
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

struct ReminderAwareTestAgent;

#[async_trait::async_trait]
impl SubAgent for ReminderAwareTestAgent {
    fn agent_type(&self) -> Cow<'static, str> {
        Cow::Borrowed("reminder-aware-test")
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
        Cow::Borrowed("reminder-aware-test")
    }
    fn intent_description(&self) -> Cow<'static, str> {
        Cow::Borrowed("test agent")
    }
    fn is_default(&self) -> bool {
        true
    }
    fn supports_reminders(&self) -> bool {
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
async fn an_answer_written_alongside_complete_task_is_the_reply(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let (user_id, agent_session_id) = seed_agent_session(&pool, session_id).await;
    let mut conn = pool.acquire().await.unwrap();

    // The answer arrives split across two text blocks, with complete_task in the same response.
    let provider = FakeLlmProvider::sequence(vec![LlmResponse {
        content: vec![
            ContentBlock::Text { text: "Here is the itinerary.\n\n".to_string() },
            ContentBlock::Text { text: "- 08:00 Arrive".to_string() },
            ContentBlock::ToolUse {
                id: "done".to_string(),
                name: COMPLETE_TASK_TOOL_NAME.to_string(),
                input: serde_json::json!({"status": "completed", "summary": "Itinerary sent"}),
                thought_signature: None,
            },
        ],
        stop_reason: StopReason::ToolUse,
        input_tokens: 1,
        output_tokens: 1,
    }]);
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
        LoopOutcome::Completed { status: "completed".to_string(), summary: "Here is the itinerary.\n\n- 08:00 Arrive".to_string() }
    );
}

async fn run_test_agent(pool: &PgPool, provider: &FakeLlmProvider) -> (Uuid, LoopOutcome) {
    let session_id = seed_session(pool).await;
    let (user_id, agent_session_id) = seed_agent_session(pool, session_id).await;
    let mut conn = pool.acquire().await.unwrap();
    let embedding_provider = FakeEmbeddingProvider::success(vec![0.0; 1536]);
    let registry = AgentRegistry::new(vec![Box::new(TestAgent)]);
    let outcome = run_agent_turn(
        &mut conn, None, None, provider, &embedding_provider, &registry, &TestAgent, session_id, agent_session_id, user_id, vec![], 100,
    )
    .await
    .unwrap();
    (session_id, outcome)
}

fn thinking_only() -> LlmResponse {
    LlmResponse {
        content: vec![ContentBlock::Thinking { text: "I'm a bit confused about the plan...".to_string(), signature: None }],
        stop_reason: StopReason::EndTurn,
        input_tokens: 1,
        output_tokens: 1,
    }
}

fn reply_of(outcome: &LoopOutcome) -> &str {
    match outcome {
        LoopOutcome::Reply { text, .. } => text,
        other => panic!("expected a reply, got {other:?}"),
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_turn_that_only_thought_is_asked_once_for_the_answer(pool: PgPool) {
    let provider = FakeLlmProvider::sequence(vec![thinking_only(), text_response("Here's the answer.", StopReason::EndTurn)]);
    let (_, outcome) = run_test_agent(&pool, &provider).await;

    assert_eq!(reply_of(&outcome), "Here's the answer.");
    let requests = provider.received_requests.lock().unwrap();
    let last = requests[1].messages.last().unwrap();
    assert!(matches!(&last.content[0], ContentBlock::Text { text } if text.contains("Write your answer")), "{last:?}");
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_turn_never_ends_silent_even_when_the_model_only_thinks(pool: PgPool) {
    let provider = FakeLlmProvider::sequence(vec![thinking_only(), thinking_only()]);
    let (_, outcome) = run_test_agent(&pool, &provider).await;
    assert_eq!(reply_of(&outcome), nomi_agent_core::Locale::En.t("engine.no_answer"));
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_crew_answers_in_the_persons_language(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let (user_id, agent_session_id) = seed_agent_session(&pool, session_id).await;
    sqlx::query("INSERT INTO user_preferences (user_id, language) VALUES ($1, 'id')").bind(user_id).execute(&pool).await.unwrap();
    let provider = FakeLlmProvider::sequence(vec![thinking_only(), thinking_only()]);
    let mut conn = pool.acquire().await.unwrap();
    let embedding_provider = FakeEmbeddingProvider::success(vec![0.0; 1536]);
    let registry = AgentRegistry::new(vec![Box::new(TestAgent)]);
    let outcome = run_agent_turn(
        &mut conn, None, None, &provider, &embedding_provider, &registry, &TestAgent, session_id, agent_session_id, user_id, vec![], 100,
    )
    .await
    .unwrap();

    // The model is told to answer in Indonesian, and Nomi's own fallback reply is Indonesian too.
    let system = provider.received_requests.lock().unwrap()[0].system.clone().unwrap();
    assert!(system.contains("Indonesian (Bahasa Indonesia)"), "{system}");
    assert!(system.contains("polite, warm and calm"), "the crew is told the Indonesian tone: {system}");
    assert_eq!(reply_of(&outcome), "Maaf, aku belum berhasil menyusun jawabannya. Boleh ditanyakan sekali lagi?");
}

#[sqlx::test(migrations = "../../migrations")]
async fn agents_see_the_chats_running_summary(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let (user_id, agent_session_id) = seed_agent_session(&pool, session_id).await;
    sqlx::query("UPDATE sessions SET summary = 'Planning a Bali trip in May.' WHERE id = $1").bind(session_id).execute(&pool).await.unwrap();
    let provider = FakeLlmProvider::sequence(vec![text_response("Sure!", StopReason::EndTurn)]);
    let mut conn = pool.acquire().await.unwrap();
    let embedding_provider = FakeEmbeddingProvider::success(vec![0.0; 1536]);
    let registry = AgentRegistry::new(vec![Box::new(TestAgent)]);
    run_agent_turn(&mut conn, None, None, &provider, &embedding_provider, &registry, &TestAgent, session_id, agent_session_id, user_id, vec![], 100)
        .await
        .unwrap();

    let system = provider.received_requests.lock().unwrap()[0].system.clone().unwrap();
    assert!(system.contains("Earlier in this chat (summary):\nPlanning a Bali trip in May."), "{system}");
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_plan_written_out_in_chat_is_saved_as_a_plan_draft(pool: PgPool) {
    let itinerary = "## Bogor day trip\n\n### Morning\n- 08:00 Arrive in Bogor\n- 08:30 Botanical Gardens\n- 11:00 Coffee nearby\n\n### Afternoon\n- 12:30 Lunch: Nasi Timbel\n- 14:00 Zoology Museum\n- 16:00 Market walk or a tea house\n\n### Evening\n- 18:00 Dinner on Suryakencana Street\n- 20:00 Head home";
    let provider = FakeLlmProvider::sequence(vec![text_response(itinerary, StopReason::EndTurn)]);
    let (session_id, outcome) = run_test_agent(&pool, &provider).await;

    assert_eq!(reply_of(&outcome), nomi_agent_core::Locale::En.t("engine.plan_draft"));
    let (title, kind): (String, String) = sqlx::query_as(
        "SELECT p.title, m.content_blocks->0->>'kind' FROM agent_plans p JOIN messages m ON m.session_id = p.session_id WHERE p.session_id = $1",
    )
    .bind(session_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!((title.as_str(), kind.as_str()), ("Bogor day trip", "plan"));
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_short_answer_stays_a_chat_reply(pool: PgPool) {
    let provider = FakeLlmProvider::sequence(vec![text_response("Bogor is about an hour from Jakarta by train.", StopReason::EndTurn)]);
    let (session_id, outcome) = run_test_agent(&pool, &provider).await;
    assert_eq!(reply_of(&outcome), "Bogor is about an hour from Jakarta by train.");
    let plans: i64 = sqlx::query_scalar("SELECT count(*) FROM agent_plans WHERE session_id = $1").bind(session_id).fetch_one(&pool).await.unwrap();
    assert_eq!(plans, 0);
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

    let (content, blocks): (String, Option<serde_json::Value>) = sqlx::query_as(
        "SELECT content, content_blocks FROM messages WHERE session_id = $1 AND sender_channel_identity_id IS NULL",
    )
    .bind(session_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(content, "🧠 working through it");
    // The chat renders this block as a collapsed "Thought process" on the agent's reply.
    assert_eq!(blocks, Some(serde_json::json!([{"kind": "reasoning", "text": "working through it"}])));
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
    let system = requests[0].system.as_ref().unwrap();
    assert!(system.starts_with("test prompt"));
    assert!(!system.contains("Be sarcastic and blunt."));
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
async fn a_pause_with_no_agent_session_row_parks_the_state_in_a_row_of_its_own(pool: PgPool) {
    // The default agent and delegated turns pass the chat's session_id as a sentinel
    // agent_session_id with no agent_sessions row behind it. Pausing must still leave a paused
    // state the approval endpoint can find — otherwise every click on the card answers
    // "this action is no longer pending".
    let session_id = seed_session(&pool).await;
    let (user_id, live_agent_session_id) = seed_agent_session(&pool, session_id).await;
    let mut conn = pool.acquire().await.unwrap();

    let provider = FakeLlmProvider::sequence(vec![tool_use_response("t1", "echo", serde_json::json!({"x": 1}))]);
    let embedding_provider = FakeEmbeddingProvider::success(vec![0.0; 1536]);
    let registry = AgentRegistry::new(vec![Box::new(TestAgent)]);

    let outcome = run_agent_turn(&mut conn, None, None, &provider, &embedding_provider, &registry, &TestAgent, session_id, session_id, user_id, vec![], 100)
        .await
        .unwrap();
    let message_id = match outcome {
        LoopOutcome::AwaitingApproval { message_id } => message_id,
        other => panic!("expected AwaitingApproval, got {other:?}"),
    };

    let (id, status, agent_type, state): (Uuid, String, String, serde_json::Value) = sqlx::query_as(
        "SELECT id, status, agent_type, state FROM agent_sessions WHERE state->>'pending_approval_message_id' = $1 AND (state->>'paused_for_approval')::boolean = true",
    )
    .bind(message_id.to_string())
    .fetch_one(&pool)
    .await
    .expect("the paused state must be findable by its approval message id");
    assert_ne!(id, live_agent_session_id, "the chat's live agent session must not be hijacked");
    assert_eq!(status, "awaiting_approval");
    assert_eq!(agent_type, "test");
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
    assert!(second_request_text.contains("The user denied this action."));
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

#[sqlx::test(migrations = "../../migrations")]
async fn create_reminder_is_only_available_to_agents_that_opt_in(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let (user_id, agent_session_id) = seed_agent_session(&pool, session_id).await;
    let mut conn = pool.acquire().await.unwrap();

    let provider = FakeLlmProvider::sequence(vec![
        tool_use_response("t1", "create_reminder", serde_json::json!({"run_at": "2026-09-21T11:00:00Z", "label": "x", "prompt": "y", "target_agent": "test"})),
        text_response("ok", StopReason::EndTurn),
    ]);
    let embedding_provider = FakeEmbeddingProvider::success(vec![0.0; 1536]);
    let registry = AgentRegistry::new(vec![Box::new(PersonalityAwareTestAgent)]);

    let outcome = run_agent_turn(&mut conn, None, None, &provider, &embedding_provider, &registry, &PersonalityAwareTestAgent, session_id, agent_session_id, user_id, vec![], 100)
        .await
        .unwrap();

    assert!(matches!(outcome, LoopOutcome::Reply { .. }));
    let job_count: i64 = sqlx::query_scalar("SELECT count(*) FROM scheduled_jobs WHERE session_id = $1")
        .bind(session_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(job_count, 0, "PersonalityAwareTestAgent never opted into supports_reminders()");
}

#[sqlx::test(migrations = "../../migrations")]
async fn create_reminder_inserts_a_job_and_returns_a_confirmation(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let (user_id, agent_session_id) = seed_agent_session(&pool, session_id).await;
    let mut conn = pool.acquire().await.unwrap();

    let provider = FakeLlmProvider::sequence(vec![
        tool_use_response("t1", "create_reminder", serde_json::json!({"run_at": "2026-09-21T11:00:00Z", "label": "take a bath", "prompt": "Remind the user to take a bath.", "target_agent": "reminder-aware-test"})),
        text_response("I'll remind you tomorrow at 11!", StopReason::EndTurn),
    ]);
    let embedding_provider = FakeEmbeddingProvider::success(vec![0.0; 1536]);
    let registry = AgentRegistry::new(vec![Box::new(ReminderAwareTestAgent)]);

    let outcome = run_agent_turn(&mut conn, None, None, &provider, &embedding_provider, &registry, &ReminderAwareTestAgent, session_id, agent_session_id, user_id, vec![], 100)
        .await
        .unwrap();

    assert_eq!(
        outcome,
        LoopOutcome::Reply { text: "I'll remind you tomorrow at 11!".to_string(), memory_ids_used: vec![], input_tokens: 1, output_tokens: 1 }
    );
    let label: String = sqlx::query_scalar("SELECT label FROM scheduled_jobs WHERE session_id = $1")
        .bind(session_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(label, "take a bath");
}

#[sqlx::test(migrations = "../../migrations")]
async fn cancel_reminder_updates_status(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let (user_id, agent_session_id) = seed_agent_session(&pool, session_id).await;
    let mut conn = pool.acquire().await.unwrap();

    let reminder_id: Uuid = sqlx::query_scalar(
        "INSERT INTO scheduled_jobs (session_id, user_id, created_by_agent_type, target_agent_type, label, prompt, run_at) \
         VALUES ($1, $2, 'reminder-aware-test', 'reminder-aware-test', 'x', 'y', now() + interval '1 day') RETURNING id",
    )
    .bind(session_id)
    .bind(user_id)
    .fetch_one(&pool)
    .await
    .unwrap();

    let provider = FakeLlmProvider::sequence(vec![
        tool_use_response("t1", "cancel_reminder", serde_json::json!({"reminder_id": reminder_id.to_string()})),
        text_response("Cancelled.", StopReason::EndTurn),
    ]);
    let embedding_provider = FakeEmbeddingProvider::success(vec![0.0; 1536]);
    let registry = AgentRegistry::new(vec![Box::new(ReminderAwareTestAgent)]);

    run_agent_turn(&mut conn, None, None, &provider, &embedding_provider, &registry, &ReminderAwareTestAgent, session_id, agent_session_id, user_id, vec![], 100)
        .await
        .unwrap();

    let status: String = sqlx::query_scalar("SELECT status FROM scheduled_jobs WHERE id = $1")
        .bind(reminder_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(status, "cancelled");
}

#[sqlx::test(migrations = "../../migrations")]
async fn system_prompt_carries_current_time_and_timezone_when_reminders_are_supported(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let (user_id, agent_session_id) = seed_agent_session(&pool, session_id).await;
    sqlx::query("INSERT INTO user_preferences (user_id, timezone) VALUES ($1, 'America/New_York')")
        .bind(user_id)
        .execute(&pool)
        .await
        .unwrap();
    let mut conn = pool.acquire().await.unwrap();

    let provider = FakeLlmProvider::sequence(vec![text_response("ok", StopReason::EndTurn)]);
    let embedding_provider = FakeEmbeddingProvider::success(vec![0.0; 1536]);
    let registry = AgentRegistry::new(vec![Box::new(ReminderAwareTestAgent)]);

    run_agent_turn(&mut conn, None, None, &provider, &embedding_provider, &registry, &ReminderAwareTestAgent, session_id, agent_session_id, user_id, vec![], 100)
        .await
        .unwrap();

    let received = provider.received_requests.lock().unwrap();
    let sent_system_prompt = received[0].system.as_deref().unwrap_or_default();
    assert!(sent_system_prompt.contains("America/New_York"));
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_stop_requested_while_the_model_is_answering_ends_the_turn_without_a_reply(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let (user_id, agent_session_id) = seed_agent_session(&pool, session_id).await;

    // The model "thinks" for 500ms; the stop lands 150ms in, mid-call.
    let provider = FakeLlmProvider::sequence(vec![text_response("Too late", StopReason::EndTurn)])
        .with_delay(std::time::Duration::from_millis(500));
    let embedding_provider = FakeEmbeddingProvider::success(vec![0.0; 1536]);
    let registry = AgentRegistry::new(vec![Box::new(TestAgent)]);

    let stop_pool = pool.clone();
    let stopper = tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(150)).await;
        sqlx::query("INSERT INTO agent_stop_requests (user_id, session_id) VALUES ($1, $2)")
            .bind(user_id)
            .bind(session_id)
            .execute(&stop_pool)
            .await
            .unwrap();
    });

    let mut conn = pool.acquire().await.unwrap();
    let outcome = run_agent_turn(
        &mut conn, None, None, &provider, &embedding_provider, &registry, &TestAgent, session_id, agent_session_id, user_id, vec![], 100,
    )
    .await
    .unwrap();
    stopper.await.unwrap();

    assert_eq!(outcome, LoopOutcome::Cancelled);
}

#[sqlx::test(migrations = "../../migrations")]
async fn an_old_stop_or_one_for_another_agent_does_not_stop_a_new_turn(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let (user_id, agent_session_id) = seed_agent_session(&pool, session_id).await;
    // Both requests predate the turn or target someone else, so neither applies.
    sqlx::query("INSERT INTO agent_stop_requests (user_id, session_id, requested_at) VALUES ($1, $2, now() - interval '1 minute')")
        .bind(user_id)
        .bind(session_id)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO agent_stop_requests (user_id, agent_type, requested_at) VALUES ($1, 'money', now() + interval '1 hour')")
        .bind(user_id)
        .execute(&pool)
        .await
        .unwrap();
    // The supervisor exempting itself: a stop it asked for never stops its own turn.
    sqlx::query("INSERT INTO agent_stop_requests (user_id, exempt_agent_type, requested_at) VALUES ($1, 'test', now() + interval '1 hour')")
        .bind(user_id)
        .execute(&pool)
        .await
        .unwrap();

    let provider = FakeLlmProvider::sequence(vec![text_response("Hello!", StopReason::EndTurn)]);
    let embedding_provider = FakeEmbeddingProvider::success(vec![0.0; 1536]);
    let registry = AgentRegistry::new(vec![Box::new(TestAgent)]);
    let mut conn = pool.acquire().await.unwrap();
    let outcome = run_agent_turn(
        &mut conn, None, None, &provider, &embedding_provider, &registry, &TestAgent, session_id, agent_session_id, user_id, vec![], 100,
    )
    .await
    .unwrap();

    assert!(matches!(outcome, LoopOutcome::Reply { .. }), "got {outcome:?}");
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_chats_thinking_level_reaches_the_model_request(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let (user_id, agent_session_id) = seed_agent_session(&pool, session_id).await;
    let embedding_provider = FakeEmbeddingProvider::success(vec![0.0; 1536]);
    let registry = AgentRegistry::new(vec![Box::new(TestAgent)]);

    for (level, enabled, effort) in [
        ("off", false, nomi_llm::ReasoningEffort::Medium),
        ("low", true, nomi_llm::ReasoningEffort::Low),
        ("high", true, nomi_llm::ReasoningEffort::High),
    ] {
        sqlx::query("UPDATE sessions SET thinking_level = $1 WHERE id = $2").bind(level).bind(session_id).execute(&pool).await.unwrap();
        let provider = FakeLlmProvider::sequence(vec![text_response("ok", StopReason::EndTurn)]);
        let mut conn = pool.acquire().await.unwrap();
        run_agent_turn(&mut conn, None, None, &provider, &embedding_provider, &registry, &TestAgent, session_id, agent_session_id, user_id, vec![], 100)
            .await
            .unwrap();
        let request = provider.received_requests.lock().unwrap()[0].clone();
        assert_eq!(request.enable_reasoning, enabled, "{level}");
        if enabled {
            assert_eq!(request.reasoning_effort, effort, "{level}");
        }
    }
}

struct RecordsAgent(&'static str);

#[async_trait::async_trait]
impl SubAgent for RecordsAgent {
    fn agent_type(&self) -> Cow<'static, str> {
        Cow::Borrowed(self.0)
    }
    fn system_prompt(&self) -> Cow<'static, str> {
        Cow::Borrowed("keeps records")
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
        Cow::Borrowed(self.0)
    }
    fn intent_description(&self) -> Cow<'static, str> {
        Cow::Borrowed("records")
    }
    fn is_default(&self) -> bool {
        true
    }
    fn uses_records(&self) -> bool {
        true
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn an_agent_keeps_private_records_that_other_agents_cannot_touch(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let (user_id, agent_session_id) = seed_agent_session(&pool, session_id).await;
    let embedding_provider = FakeEmbeddingProvider::success(vec![0.0; 1536]);

    let scout = RecordsAgent("travel_scout");
    let registry = AgentRegistry::new(vec![Box::new(RecordsAgent("travel_scout"))]);
    let provider = FakeLlmProvider::sequence(vec![
        tool_use_response("t1", "save_record", serde_json::json!({"collection": "Trips", "data": {"place": "Bali", "nights": 5}})),
        text_response("Saved", StopReason::EndTurn),
    ]);
    let mut conn = pool.acquire().await.unwrap();
    run_agent_turn(&mut conn, None, None, &provider, &embedding_provider, &registry, &scout, session_id, agent_session_id, user_id, vec![], 100)
        .await
        .unwrap();
    // Offered the tools, and never asked for approval to use its own storage.
    assert!(provider.received_requests.lock().unwrap()[0].tools.iter().any(|t| t.name == "list_records"));
    let (owner, collection, place): (String, String, String) =
        sqlx::query_as("SELECT agent_type, collection, data->>'place' FROM agent_records WHERE user_id = $1").bind(user_id).fetch_one(&pool).await.unwrap();
    assert_eq!((owner.as_str(), collection.as_str(), place.as_str()), ("travel_scout", "trips", "Bali"));
    let record_id: Uuid = sqlx::query_scalar("SELECT id FROM agent_records").fetch_one(&pool).await.unwrap();

    // Another agent can neither list nor delete it.
    let other = RecordsAgent("gift_finder");
    let registry = AgentRegistry::new(vec![Box::new(RecordsAgent("gift_finder"))]);
    let provider = FakeLlmProvider::sequence(vec![
        tool_use_response("t2", "list_records", serde_json::json!({"collection": "trips"})),
        tool_use_response("t3", "delete_record", serde_json::json!({"id": record_id.to_string()})),
        text_response("done", StopReason::EndTurn),
    ]);
    run_agent_turn(&mut conn, None, None, &provider, &embedding_provider, &registry, &other, session_id, agent_session_id, user_id, vec![], 100)
        .await
        .unwrap();
    let requests = provider.received_requests.lock().unwrap();
    let results = format!("{:?}", requests[2].messages);
    assert!(results.contains("No records in trips yet."), "{results}");
    assert!(results.contains("no record of yours with that id"), "{results}");
    let still_there: i64 = sqlx::query_scalar("SELECT count(*) FROM agent_records").fetch_one(&pool).await.unwrap();
    assert_eq!(still_there, 1);
}

#[sqlx::test(migrations = "../../migrations")]
async fn thinking_is_asked_to_stay_short_unless_it_is_off(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let (user_id, agent_session_id) = seed_agent_session(&pool, session_id).await;
    for (level, expect_style) in [("medium", true), ("off", false)] {
        sqlx::query("UPDATE sessions SET thinking_level = $1 WHERE id = $2").bind(level).bind(session_id).execute(&pool).await.unwrap();
        let mut conn = pool.acquire().await.unwrap();
        let provider = FakeLlmProvider::sequence(vec![text_response("Fine.", StopReason::EndTurn)]);
        let embedding_provider = FakeEmbeddingProvider::success(vec![0.0; 1536]);
        let registry = AgentRegistry::new(vec![Box::new(TestAgent)]);
        run_agent_turn(&mut conn, None, None, &provider, &embedding_provider, &registry, &TestAgent, session_id, agent_session_id, user_id, vec![], 100)
            .await
            .unwrap();
        let requests = provider.received_requests.lock().unwrap();
        let system = requests[0].system.as_ref().unwrap();
        assert_eq!(system.contains(nomi_agent_core::engine::REASONING_STYLE), expect_style, "level {level}");
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_reply_cut_off_after_thinking_says_so_instead_of_coming_back_empty(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let (user_id, agent_session_id) = seed_agent_session(&pool, session_id).await;
    let mut conn = pool.acquire().await.unwrap();
    let provider = FakeLlmProvider::sequence(vec![LlmResponse {
        content: vec![ContentBlock::Thinking { text: "Plan the trip day by day.".to_string(), signature: None }],
        stop_reason: StopReason::MaxTokens,
        input_tokens: 1,
        output_tokens: 1,
    }]);
    let embedding_provider = FakeEmbeddingProvider::success(vec![0.0; 1536]);
    let registry = AgentRegistry::new(vec![Box::new(TestAgent)]);

    let outcome = run_agent_turn(&mut conn, None, None, &provider, &embedding_provider, &registry, &TestAgent, session_id, agent_session_id, user_id, vec![], 100)
        .await
        .unwrap();

    assert_eq!(
        outcome,
        LoopOutcome::Reply { text: nomi_agent_core::Locale::En.t("engine.cut_off"), memory_ids_used: vec![], input_tokens: 1, output_tokens: 1 }
    );
}

/// TestAgent, but it remembers things about the person.
struct RememberingTestAgent;

#[async_trait::async_trait]
impl SubAgent for RememberingTestAgent {
    fn agent_type(&self) -> Cow<'static, str> {
        Cow::Borrowed("test")
    }
    fn system_prompt(&self) -> Cow<'static, str> {
        Cow::Borrowed("test prompt")
    }
    fn tools(&self) -> Vec<ToolDefinition> {
        TestAgent.tools()
    }
    async fn execute_tool(
        &self,
        conn: &mut PoolConnection<Postgres>,
        session_id: Uuid,
        agent_session_id: Uuid,
        user_id: Uuid,
        name: &str,
        input: serde_json::Value,
    ) -> Result<ToolOutcome, String> {
        TestAgent.execute_tool(conn, session_id, agent_session_id, user_id, name, input).await
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
    fn uses_memory(&self) -> bool {
        true
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_reply_after_a_tool_call_still_learns_from_what_the_person_said(pool: PgPool) {
    let session_id = seed_session(&pool).await;
    let (user_id, agent_session_id) = seed_agent_session(&pool, session_id).await;
    sqlx::query("INSERT INTO tool_permission_rules (user_id, tool_name, decision) VALUES ($1, 'echo', 'allow')")
        .bind(user_id)
        .execute(&pool)
        .await
        .unwrap();
    let identity: Uuid = sqlx::query_scalar("SELECT id FROM channel_identities WHERE user_id = $1").bind(user_id).fetch_one(&pool).await.unwrap();
    sqlx::query("INSERT INTO messages (session_id, sender_channel_identity_id, content) VALUES ($1, $2, 'Remind me to call my sister Rina on Sunday')")
        .bind(session_id)
        .bind(identity)
        .execute(&pool)
        .await
        .unwrap();
    let mut conn = pool.acquire().await.unwrap();

    // The agent calls a tool before replying, so its last user turn is a tool result, not text.
    let provider = FakeLlmProvider::sequence(vec![
        tool_use_response("t1", "echo", serde_json::json!({"x": 1})),
        text_response("Done, I'll remind you Sunday.", StopReason::EndTurn),
        text_response(r#"{"action":"add","kind":"person","text":"Sister Rina"}"#, StopReason::EndTurn),
    ]);
    let embedding_provider = FakeEmbeddingProvider::success(vec![0.5; 1536]);
    let registry = AgentRegistry::new(vec![Box::new(RememberingTestAgent)]);
    let history = vec![nomi_llm::LlmMessage {
        role: nomi_llm::LlmRole::User,
        content: vec![ContentBlock::Text { text: "Remind me to call my sister Rina on Sunday".to_string() }],
    }];

    run_agent_turn(
        &mut conn, None, None, &provider, &embedding_provider, &registry, &RememberingTestAgent, session_id, agent_session_id, user_id,
        history, 100,
    )
    .await
    .unwrap();

    let stored: Vec<String> = sqlx::query_scalar("SELECT content FROM memory_items WHERE user_id = $1").bind(user_id).fetch_all(&pool).await.unwrap();
    assert_eq!(stored, vec!["Sister Rina".to_string()]);
    let requests = provider.received_requests.lock().unwrap();
    let extraction = requests.last().unwrap();
    let ContentBlock::Text { text } = &extraction.messages[0].content[0] else { panic!("extraction prompt is text") };
    assert!(text.contains("Remind me to call my sister Rina on Sunday"), "{text}");
}
