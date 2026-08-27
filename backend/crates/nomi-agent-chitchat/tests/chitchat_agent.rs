use sqlx::PgPool;
use uuid::Uuid;

use nomi_llm::{ContentBlock, LlmMessage, LlmResponse, LlmRole, StopReason};
use nomi_agent_core::{run_agent_turn, LoopOutcome};
use nomi_agent_chitchat::ChitchatAgent;

use nomi_test_support::{FakeEmbeddingProvider, FakeLlmProvider};

const CHITCHAT_MAX_TOKENS: u32 = 1024;

fn text_response(text: &str) -> LlmResponse {
    LlmResponse {
        content: vec![ContentBlock::Text { text: text.to_string() }],
        stop_reason: StopReason::EndTurn,
        input_tokens: 10,
        output_tokens: 5,
    }
}

fn user_message(text: &str) -> LlmMessage {
    LlmMessage { role: LlmRole::User, content: vec![ContentBlock::Text { text: text.to_string() }] }
}

fn make_embedding(first: f32) -> Vec<f32> {
    let mut v = vec![0.0f32; 1536];
    v[0] = first;
    v
}

fn to_vector_literal(embedding: &[f32]) -> String {
    format!("[{}]", embedding.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(","))
}

async fn seed_user(pool: &PgPool) -> Uuid {
    sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(pool).await.unwrap()
}

// Ported from the old backend/tests/turn_chitchat.rs `persists_reply_and_chitchat_reply_event_
// on_success`, minus the message/agent_events persistence assertions: those DB writes were
// never part of chitchat's own logic conceptually (they're the caller's job, same as for
// money_agent's `run_subagent_turn`) and are now exercised end-to-end through
// `backend/tests/turn_handle_inbound_message.rs` via the temporary bridge in
// `backend/src/turn/mod.rs`. What *is* chitchat/`run_agent_turn`-specific — the reply text
// coming back, and memory extraction firing afterward because `ChitchatAgent::uses_memory()`
// is `true` — is asserted here directly against the engine.
#[sqlx::test(migrations = "../../migrations")]
async fn end_turn_returns_the_reply_and_triggers_memory_extraction_afterward(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let mut conn = pool.acquire().await.unwrap();

    // First response answers the user; the second is the memory-extraction call that
    // `run_agent_turn` fires after a `Reply` outcome because `ChitchatAgent::uses_memory()`
    // returns `true`.
    let provider = FakeLlmProvider::sequence(vec![text_response("Hello there!"), text_response("User said hi")]);
    let embedder = FakeEmbeddingProvider::success(make_embedding(0.5));

    let outcome = run_agent_turn(
        &mut conn,
        None,
        &provider,
        &embedder,
        &ChitchatAgent,
        Uuid::new_v4(),
        Uuid::new_v4(),
        user_id,
        vec![user_message("hi")],
        CHITCHAT_MAX_TOKENS,
    )
    .await
    .unwrap();

    // No memories exist for this user yet, so uses_memory()=true retrieves none.
    assert_eq!(
        outcome,
        LoopOutcome::Reply { text: "Hello there!".to_string(), memory_ids_used: vec![], input_tokens: 10, output_tokens: 5 }
    );

    let stored: String = sqlx::query_scalar("SELECT content FROM memory_items WHERE user_id = $1")
        .bind(user_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(stored, "User said hi");
}

// Ported from `persists_nothing_when_the_provider_call_fails`: back then this checked that a
// failed provider call left `messages`/`agent_events` untouched. `run_agent_turn` itself never
// writes to either of those tables (that's the caller's job now), so the equivalent DB write
// in scope here is memory extraction, which must not run when the main call never produced a
// reply to extract from.
#[sqlx::test(migrations = "../../migrations")]
async fn provider_failure_returns_an_error_and_extracts_no_memory(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let mut conn = pool.acquire().await.unwrap();
    let provider = FakeLlmProvider::failure("provider unavailable");
    let embedder = FakeEmbeddingProvider::success(make_embedding(0.5));

    let result = run_agent_turn(
        &mut conn,
        None,
        &provider,
        &embedder,
        &ChitchatAgent,
        Uuid::new_v4(),
        Uuid::new_v4(),
        user_id,
        vec![user_message("hi")],
        CHITCHAT_MAX_TOKENS,
    )
    .await;

    assert!(result.is_err());

    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM memory_items WHERE user_id = $1")
        .bind(user_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 0);
}

// Ported from `retrieved_memories_are_folded_into_the_system_prompt_and_linked_to_the_reply`.
// The "linked to the reply" half now asserts against `LoopOutcome::Reply::memory_ids_used`
// instead of a `message_memory_usage` row directly: `run_agent_turn` doesn't write that table
// itself (persisting the reply message, and therefore the link, is still the caller's job —
// same division of responsibility as everything else in `messages`/`agent_events`), but it now
// reports which memories were actually used so a caller (`nomi-turn::run_subagent_turn`) can
// write that row. See `nomi-turn`'s own tests for the end-to-end persistence of that row.
#[sqlx::test(migrations = "../../migrations")]
async fn retrieved_memories_are_folded_into_the_system_prompt_and_reported_as_used(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let memory_id: Uuid = sqlx::query_scalar(
        "INSERT INTO memory_items (user_id, content, embedding) VALUES ($1, $2, $3::vector) RETURNING id",
    )
    .bind(user_id)
    .bind("User is vegetarian")
    .bind(to_vector_literal(&make_embedding(1.0)))
    .fetch_one(&pool)
    .await
    .unwrap();

    let mut conn = pool.acquire().await.unwrap();
    let provider = FakeLlmProvider::sequence(vec![text_response("Got it, no meat!"), text_response("NONE")]);
    let embedder = FakeEmbeddingProvider::success(make_embedding(1.0));

    let outcome = run_agent_turn(
        &mut conn,
        None,
        &provider,
        &embedder,
        &ChitchatAgent,
        Uuid::new_v4(),
        Uuid::new_v4(),
        user_id,
        vec![user_message("what should I eat?")],
        CHITCHAT_MAX_TOKENS,
    )
    .await
    .unwrap();

    match outcome {
        LoopOutcome::Reply { memory_ids_used, .. } => assert_eq!(memory_ids_used, vec![memory_id]),
        LoopOutcome::Completed { .. } => panic!("expected a Reply outcome"),
    }

    let requests = provider.received_requests.lock().unwrap();
    let system = requests[0].system.as_ref().unwrap();
    assert!(system.contains("Relevant things you know about this user"));
    assert!(system.contains("User is vegetarian"));
}

// Direct port of `a_failing_embedding_provider_does_not_prevent_a_normal_reply`: an embedding
// failure must fail memory retrieval open (empty memories, plain system prompt) rather than
// failing the whole turn.
#[sqlx::test(migrations = "../../migrations")]
async fn a_failing_embedding_provider_does_not_prevent_a_normal_reply(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let mut conn = pool.acquire().await.unwrap();
    let provider = FakeLlmProvider::sequence(vec![text_response("Still here!"), text_response("NONE")]);
    let embedder = FakeEmbeddingProvider::failure("embeddings unavailable");

    let outcome = run_agent_turn(
        &mut conn,
        None,
        &provider,
        &embedder,
        &ChitchatAgent,
        Uuid::new_v4(),
        Uuid::new_v4(),
        user_id,
        vec![user_message("hi")],
        CHITCHAT_MAX_TOKENS,
    )
    .await
    .unwrap();

    // The embedding failure makes memory retrieval fail open (empty), not the whole turn.
    assert_eq!(
        outcome,
        LoopOutcome::Reply { text: "Still here!".to_string(), memory_ids_used: vec![], input_tokens: 10, output_tokens: 5 }
    );

    let requests = provider.received_requests.lock().unwrap();
    assert_eq!(
        requests[0].system.as_ref().unwrap(),
        "You are a helpful, friendly assistant chatting with the user. Keep replies concise."
    );
}
