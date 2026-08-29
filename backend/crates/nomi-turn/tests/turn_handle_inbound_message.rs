
use std::time::Duration;

use sqlx::PgPool;
use uuid::Uuid;

use nomi_llm::{ContentBlock, LlmResponse, LlmRole, StopReason};
use nomi_turn::handle_inbound_message;

use nomi_agent_chitchat::ChitchatAgent;
use nomi_agent_core::AgentRegistry;
use nomi_agent_money::MoneyAgent;
use nomi_agent_personality::PersonalityAgent;
use nomi_test_support::{dummy_embedding, FakeEmbeddingProvider, FakeLlmProvider};

fn canned_response(text: &str) -> LlmResponse {
    LlmResponse {
        content: vec![ContentBlock::Text { text: text.to_string() }],
        stop_reason: StopReason::EndTurn,
        input_tokens: 10,
        output_tokens: 5,
    }
}

fn tool_use_response(id: &str, name: &str, input: serde_json::Value) -> LlmResponse {
    LlmResponse {
        content: vec![ContentBlock::ToolUse { id: id.to_string(), name: name.to_string(), input, thought_signature: None }],
        stop_reason: StopReason::ToolUse,
        input_tokens: 10,
        output_tokens: 5,
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn new_sender_gets_bootstrapped_and_receives_a_chitchat_reply(pool: PgPool) {
    let provider = FakeLlmProvider::success(canned_response("Hi! How can I help?"));
    let embedder = FakeEmbeddingProvider::success(dummy_embedding());
    let registry = AgentRegistry::new(vec![Box::new(MoneyAgent), Box::new(ChitchatAgent)]);

    let outcome = handle_inbound_message(&pool, &provider, &embedder, &registry, "telegram", "dm", "chat-1", "tg-1", "hello", None)
        .await
        .unwrap();

    assert_eq!(outcome.reply, "Hi! How can I help?");

    let message_count: i64 = sqlx::query_scalar("SELECT count(*) FROM messages WHERE session_id = $1")
        .bind(outcome.session_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(message_count, 2); // inbound + reply

    let user_count: i64 = sqlx::query_scalar("SELECT count(*) FROM users").fetch_one(&pool).await.unwrap();
    assert_eq!(user_count, 1);
    let org_count: i64 = sqlx::query_scalar("SELECT count(*) FROM organizations").fetch_one(&pool).await.unwrap();
    assert_eq!(org_count, 1);
}

#[sqlx::test(migrations = "../../migrations")]
async fn existing_sender_reuses_identity_and_session_across_two_calls(pool: PgPool) {
    let provider = FakeLlmProvider::success(canned_response("ok"));
    let embedder = FakeEmbeddingProvider::success(dummy_embedding());
    let registry = AgentRegistry::new(vec![Box::new(MoneyAgent), Box::new(ChitchatAgent)]);

    let first = handle_inbound_message(&pool, &provider, &embedder, &registry, "telegram", "dm", "chat-1", "tg-1", "first", None)
        .await
        .unwrap();
    let second = handle_inbound_message(&pool, &provider, &embedder, &registry, "telegram", "dm", "chat-1", "tg-1", "second", None)
        .await
        .unwrap();

    assert_eq!(first.session_id, second.session_id);

    let message_count: i64 = sqlx::query_scalar("SELECT count(*) FROM messages WHERE session_id = $1")
        .bind(first.session_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(message_count, 4); // 2 inbound + 2 replies

    let user_count: i64 = sqlx::query_scalar("SELECT count(*) FROM users").fetch_one(&pool).await.unwrap();
    assert_eq!(user_count, 1);
}

#[sqlx::test(migrations = "../../migrations")]
async fn inbound_message_is_durable_even_when_the_provider_call_fails(pool: PgPool) {
    let provider = FakeLlmProvider::failure("provider unavailable");
    let embedder = FakeEmbeddingProvider::success(dummy_embedding());
    let registry = AgentRegistry::new(vec![Box::new(MoneyAgent), Box::new(ChitchatAgent)]);

    let result = handle_inbound_message(&pool, &provider, &embedder, &registry, "telegram", "dm", "chat-1", "tg-1", "hello", None)
        .await;
    assert!(result.is_err());

    let (content,): (String,) = sqlx::query_as(
        "SELECT content FROM messages m JOIN sessions s ON m.session_id = s.id WHERE s.channel = 'telegram' AND s.chat_id = 'chat-1'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(content, "hello");

    let event_type: String = sqlx::query_scalar(
        "SELECT event_type FROM agent_events e JOIN sessions s ON e.session_id = s.id WHERE s.channel = 'telegram' AND s.chat_id = 'chat-1'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(event_type, "TurnFailed");
}

#[sqlx::test(migrations = "../../migrations")]
async fn an_active_agent_session_does_not_block_the_chitchat_fallback_in_this_slice(pool: PgPool) {
    let provider = FakeLlmProvider::success(canned_response("still chatting"));
    let embedder = FakeEmbeddingProvider::success(dummy_embedding());
    let registry = AgentRegistry::new(vec![Box::new(MoneyAgent), Box::new(ChitchatAgent)]);

    let first = handle_inbound_message(&pool, &provider, &embedder, &registry, "telegram", "dm", "chat-1", "tg-1", "hi", None)
        .await
        .unwrap();

    let identity_id: Uuid = sqlx::query_scalar(
        "SELECT ci.id FROM channel_identities ci WHERE ci.channel = 'telegram' AND ci.channel_user_id = 'tg-1'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO agent_sessions (session_id, sender_channel_identity_id, agent_type, status) VALUES ($1, $2, 'booking', 'active')",
    )
    .bind(first.session_id)
    .bind(identity_id)
    .execute(&pool)
    .await
    .unwrap();

    let second = handle_inbound_message(&pool, &provider, &embedder, &registry, "telegram", "dm", "chat-1", "tg-1", "still there?", None)
        .await
        .unwrap();

    assert_eq!(second.reply, "still chatting");
}

#[sqlx::test(migrations = "../../migrations")]
async fn concurrent_messages_for_the_same_session_are_serialized(pool: PgPool) {
    let bootstrap_provider = FakeLlmProvider::success(canned_response("bootstrapped"));
    let embedder = FakeEmbeddingProvider::success(dummy_embedding());
    let registry = AgentRegistry::new(vec![Box::new(MoneyAgent), Box::new(ChitchatAgent)]);
    handle_inbound_message(&pool, &bootstrap_provider, &embedder, &registry, "telegram", "dm", "chat-1", "tg-1", "bootstrap", None)
        .await
        .unwrap();

    let provider = FakeLlmProvider::success(canned_response("ok")).with_delay(Duration::from_millis(200));

    let call1 = handle_inbound_message(&pool, &provider, &embedder, &registry, "telegram", "dm", "chat-1", "tg-1", "first", None);
    let call2 = handle_inbound_message(&pool, &provider, &embedder, &registry, "telegram", "dm", "chat-1", "tg-1", "second", None);
    let (result1, result2) = tokio::join!(call1, call2);
    result1.unwrap();
    result2.unwrap();

    let rows: Vec<(Option<Uuid>, String)> = sqlx::query_as(
        "SELECT m.sender_channel_identity_id, m.content FROM messages m \
         JOIN sessions s ON m.session_id = s.id \
         WHERE s.channel = 'telegram' AND s.chat_id = 'chat-1' \
         ORDER BY m.created_at ASC",
    )
    .fetch_all(&pool)
    .await
    .unwrap();

    // bootstrap turn (2 rows) + two serialized turns (2 rows each) = 6, never interleaved.
    assert_eq!(rows.len(), 6);
    let (first_inbound, first_reply, second_inbound, second_reply) =
        (&rows[2], &rows[3], &rows[4], &rows[5]);
    assert!(first_inbound.0.is_some());
    assert!(first_inbound.1 == "first" || first_inbound.1 == "second");
    assert!(first_reply.0.is_none());
    assert_ne!(first_inbound.1, second_inbound.1);
    assert!(second_inbound.0.is_some());
    assert!(second_reply.0.is_none());
}

// The chitchat path's history fetch (`fetch_recent_messages` in `backend/src/turn/mod.rs`) is
// shared with the money-agent path and is no longer chitchat's own code (it used to be
// duplicated inline in the now-deleted `turn/chitchat.rs`, ported to `nomi-agent-chitchat` in
// Task 8). Since `nomi_agent_core::run_agent_turn` takes an already-built message list rather
// than fetching it itself, this behavior can no longer be exercised by calling into
// `ChitchatAgent` directly — it only shows up by driving the real `handle_inbound_message`
// entrypoint, which is what these two tests (ported from the old `turn_chitchat.rs`'s
// `keeps_only_the_last_20_messages_ordered_oldest_first` and
// `maps_sender_presence_to_role_correctly`) do.
#[sqlx::test(migrations = "../../migrations")]
async fn chitchat_turn_keeps_only_the_last_20_messages_ordered_oldest_first(pool: PgPool) {
    let embedder = FakeEmbeddingProvider::success(dummy_embedding());
    let registry = AgentRegistry::new(vec![Box::new(MoneyAgent), Box::new(ChitchatAgent)]);

    let bootstrap_provider = FakeLlmProvider::success(canned_response("bootstrapped"));
    let bootstrap = handle_inbound_message(&pool, &bootstrap_provider, &embedder, &registry, "telegram", "dm", "chat-1", "tg-1", "bootstrap", None)
        .await
        .unwrap();

    let identity_id: Uuid = sqlx::query_scalar(
        "SELECT id FROM channel_identities WHERE channel = 'telegram' AND channel_user_id = 'tg-1'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    let base = chrono::Utc::now();
    for i in 0..25 {
        sqlx::query(
            "INSERT INTO messages (session_id, sender_channel_identity_id, content, created_at) VALUES ($1, $2, $3, $4)",
        )
        .bind(bootstrap.session_id)
        .bind(identity_id)
        .bind(format!("seq-{i}"))
        .bind(base + chrono::Duration::milliseconds(i))
        .execute(&pool)
        .await
        .unwrap();
    }

    // The 25 rows above were seeded with explicit, synthetic created_at values up to 24ms after
    // `base`. "latest" below is inserted through the real handle_inbound_message path and gets a
    // genuine `now()` timestamp from Postgres — on a fast local DB, all 25 inserts can finish in
    // under 24ms of wall-clock time, letting "latest"'s real timestamp land *before* some of the
    // tail-end synthetic ones. Sleeping past the synthetic span guarantees real "now" has moved
    // beyond it before "latest" is inserted, regardless of how fast the loop above ran.
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let provider = FakeLlmProvider::success(canned_response("ok"));
    handle_inbound_message(&pool, &provider, &embedder, &registry, "telegram", "dm", "chat-1", "tg-1", "latest", None)
        .await
        .unwrap();

    let requests = provider.received_requests.lock().unwrap();
    // requests[0] is the intent classifier's own single-message request (every inbound message
    // is reclassified since chitchat never spawns an agent_session to "continue"); requests[1]
    // is the actual chitchat turn, built from `fetch_recent_messages`.
    let sent = &requests[1];
    assert_eq!(sent.messages.len(), 20);
    // The 25 seeded messages (seq-0..seq-24) plus this call's own "latest" inbound message (which
    // is persisted before this LLM call happens) make the most-recent-20 window seq-6..seq-24
    // followed by "latest".
    for (offset, message) in sent.messages.iter().take(19).enumerate() {
        let expected_seq = 6 + offset;
        match &message.content[0] {
            ContentBlock::Text { text } => assert_eq!(text, &format!("seq-{expected_seq}")),
            _ => panic!("expected a text block"),
        }
    }
    match &sent.messages[19].content[0] {
        ContentBlock::Text { text } => assert_eq!(text, "latest"),
        _ => panic!("expected a text block"),
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn chitchat_turn_maps_sender_presence_to_role_correctly(pool: PgPool) {
    let embedder = FakeEmbeddingProvider::success(dummy_embedding());
    let registry = AgentRegistry::new(vec![Box::new(MoneyAgent), Box::new(ChitchatAgent)]);

    let bootstrap_provider = FakeLlmProvider::success(canned_response("hello back"));
    handle_inbound_message(&pool, &bootstrap_provider, &embedder, &registry, "telegram", "dm", "chat-1", "tg-1", "hi", None)
        .await
        .unwrap();

    let provider = FakeLlmProvider::success(canned_response("ok"));
    handle_inbound_message(&pool, &provider, &embedder, &registry, "telegram", "dm", "chat-1", "tg-1", "latest", None)
        .await
        .unwrap();

    let requests = provider.received_requests.lock().unwrap();
    // requests[0] is the intent classifier's own single-message request; requests[1] is the
    // actual chitchat turn, built from `fetch_recent_messages`.
    let sent = &requests[1];
    // sender present (the user's own messages) maps to User; sender absent (the assistant's own
    // replies, sender_channel_identity_id = NULL) maps to Assistant.
    assert_eq!(sent.messages[0].role, LlmRole::User); // "hi"
    assert_eq!(sent.messages[1].role, LlmRole::Assistant); // "hello back"
    assert_eq!(sent.messages[2].role, LlmRole::User); // "latest"
}

// End-to-end proof that a chitchat reply's used memories get linked via message_memory_usage,
// and that the reply itself is recorded as an AgentReplied event with token counts — the
// bookkeeping the old bespoke turn/chitchat.rs used to do inline, restored here as the caller's
// responsibility now that nomi_agent_core::run_agent_turn reports which memories it used via
// LoopOutcome::Reply::memory_ids_used instead of writing the link itself.
#[sqlx::test(migrations = "../../migrations")]
async fn a_chitchat_replys_used_memory_is_linked_and_recorded_as_an_agent_replied_event(pool: PgPool) {
    let embedder = FakeEmbeddingProvider::success(dummy_embedding());
    let registry = AgentRegistry::new(vec![Box::new(MoneyAgent), Box::new(ChitchatAgent)]);

    // Bootstrap the user/identity first so a memory can be seeded for the real user_id. Each
    // handle_inbound_message call makes 3 provider calls in order (intent classification, the
    // chitchat reply itself, then memory extraction since ChitchatAgent::uses_memory() is
    // true) — queue an explicit "NONE" for extraction so a FakeLlmProvider::success() reusing
    // the reply text doesn't get treated as an extracted fact and plant a second, competing
    // memory with the same embedding as the one this test seeds below.
    let bootstrap_provider =
        FakeLlmProvider::sequence(vec![canned_response("chitchat"), canned_response("hi there"), canned_response("NONE")]);
    handle_inbound_message(&pool, &bootstrap_provider, &embedder, &registry, "telegram", "dm", "chat-1", "tg-1", "hello", None)
        .await
        .unwrap();
    let user_id: Uuid = sqlx::query_scalar(
        "SELECT user_id FROM channel_identities WHERE channel = 'telegram' AND channel_user_id = 'tg-1'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    let literal = format!("[{}]", vec!["0"; 1536].join(","));
    let memory_id: Uuid = sqlx::query_scalar(
        "INSERT INTO memory_items (user_id, content, embedding, embedding_provider, embedding_model) \
         VALUES ($1, $2, $3::vector, 'fake', 'fake-model') RETURNING id",
    )
    .bind(user_id)
    .bind("User is vegetarian")
    .bind(&literal)
    .fetch_one(&pool)
    .await
    .unwrap();

    let provider = FakeLlmProvider::sequence(vec![
        canned_response("chitchat"),
        canned_response("Noted, no meat!"),
        canned_response("NONE"),
    ]);
    let outcome = handle_inbound_message(&pool, &provider, &embedder, &registry, "telegram", "dm", "chat-1", "tg-1", "what should I eat?", None)
        .await
        .unwrap();

    let linked_memory_id: Uuid = sqlx::query_scalar(
        "SELECT memory_id FROM message_memory_usage mmu \
         JOIN messages m ON m.id = mmu.message_id \
         WHERE m.session_id = $1 AND m.content = 'Noted, no meat!'",
    )
    .bind(outcome.session_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(linked_memory_id, memory_id);

    let (event_type, payload): (String, serde_json::Value) = sqlx::query_as(
        "SELECT event_type, payload FROM agent_events WHERE session_id = $1 AND event_type = 'AgentReplied'",
    )
    .bind(outcome.session_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(event_type, "AgentReplied");
    assert_eq!(payload["input_tokens"], 10);
    assert_eq!(payload["output_tokens"], 5);
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_personality_change_is_recorded_and_folded_into_the_next_chitchat_reply(pool: PgPool) {
    let embedder = FakeEmbeddingProvider::success(dummy_embedding());
    let registry =
        AgentRegistry::new(vec![Box::new(ChitchatAgent), Box::new(MoneyAgent), Box::new(PersonalityAgent)]);

    // Turn 1: classified as "personality"; the agent calls set_personality, then complete_task.
    let personality_provider = FakeLlmProvider::sequence(vec![
        canned_response("personality"),
        tool_use_response("t1", "set_personality", serde_json::json!({"description": "Be sarcastic and blunt."})),
        tool_use_response(
            "t2",
            "complete_task",
            serde_json::json!({"status": "completed", "summary": "Done, I'll be blunt now."}),
        ),
    ]);
    let outcome = handle_inbound_message(
        &pool,
        &personality_provider,
        &embedder,
        &registry,
        "telegram",
        "dm",
        "chat-1",
        "tg-1",
        "be more sarcastic and blunt",
        None,
    )
    .await
    .unwrap();
    assert_eq!(outcome.reply, "Done, I'll be blunt now.");

    let event_count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM agent_events WHERE event_type = 'PersonalityChanged'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(event_count, 1);

    // Turn 2: a plain chitchat message. The personality agent_session from Turn 1 already
    // completed, so this reclassifies fresh and falls through to chitchat.
    let chitchat_provider =
        FakeLlmProvider::sequence(vec![canned_response("chitchat"), canned_response("Sure thing."), canned_response("NONE")]);
    handle_inbound_message(
        &pool,
        &chitchat_provider,
        &embedder,
        &registry,
        "telegram",
        "dm",
        "chat-1",
        "tg-1",
        "what's up?",
        None,
    )
    .await
    .unwrap();

    let requests = chitchat_provider.received_requests.lock().unwrap();
    let system = requests[1].system.as_ref().unwrap();
    assert!(system.contains("Adopt this personality in your replies: Be sarcastic and blunt."));
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_personality_set_in_one_chat_context_is_visible_in_a_different_chat_context_for_the_same_person(pool: PgPool) {
    let embedder = FakeEmbeddingProvider::success(dummy_embedding());
    let registry =
        AgentRegistry::new(vec![Box::new(ChitchatAgent), Box::new(MoneyAgent), Box::new(PersonalityAgent)]);

    // Seed a personality change through a personal DM.
    let personality_provider = FakeLlmProvider::sequence(vec![
        canned_response("personality"),
        tool_use_response("t1", "set_personality", serde_json::json!({"description": "Talk like a 1920s detective."})),
        tool_use_response(
            "t2",
            "complete_task",
            serde_json::json!({"status": "completed", "summary": "Right-o, consider it done, see."}),
        ),
    ]);
    handle_inbound_message(
        &pool,
        &personality_provider,
        &embedder,
        &registry,
        "telegram",
        "dm",
        "dm-chat",
        "tg-user-1",
        "talk like a detective",
        None,
    )
    .await
    .unwrap();

    // Same sender (same channel_user_id), but a DIFFERENT chat_id and chat_type ("group") — a
    // distinct session, same person per channel_identities' (channel, channel_user_id)
    // resolution (proven by turn_bootstrap.rs's existing
    // existing_sender_new_chat_creates_a_new_session_under_the_same_personal_org).
    let chitchat_provider = FakeLlmProvider::sequence(vec![
        canned_response("chitchat"),
        canned_response("Say, what can I do for ya?"),
        canned_response("NONE"),
    ]);
    let group_outcome = handle_inbound_message(
        &pool,
        &chitchat_provider,
        &embedder,
        &registry,
        "telegram",
        "group",
        "group-chat",
        "tg-user-1",
        "hello there",
        None,
    )
    .await
    .unwrap();

    let requests = chitchat_provider.received_requests.lock().unwrap();
    let system = requests[1].system.as_ref().unwrap();
    assert!(system.contains("Adopt this personality in your replies: Talk like a 1920s detective."));

    let dm_session_id: Uuid =
        sqlx::query_scalar("SELECT id FROM sessions WHERE channel = 'telegram' AND chat_id = 'dm-chat'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_ne!(dm_session_id, group_outcome.session_id);
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_chat_driven_rollback_is_folded_into_the_next_chitchat_reply(pool: PgPool) {
    let embedder = FakeEmbeddingProvider::success(dummy_embedding());
    let registry =
        AgentRegistry::new(vec![Box::new(ChitchatAgent), Box::new(MoneyAgent), Box::new(PersonalityAgent)]);

    // Turn 1: set an initial personality (v1).
    let set_v1_provider = FakeLlmProvider::sequence(vec![
        canned_response("personality"),
        tool_use_response("t1", "set_personality", serde_json::json!({"description": "Be sarcastic and blunt."})),
        tool_use_response("t2", "complete_task", serde_json::json!({"status": "completed", "summary": "Done, sarcastic now."})),
    ]);
    handle_inbound_message(&pool, &set_v1_provider, &embedder, &registry, "telegram", "dm", "chat-1", "tg-1", "be sarcastic", None)
        .await
        .unwrap();

    // Turn 2: change it again (v2), so there's something to roll back from.
    let set_v2_provider = FakeLlmProvider::sequence(vec![
        canned_response("personality"),
        tool_use_response("t3", "set_personality", serde_json::json!({"description": "Be warm and encouraging."})),
        tool_use_response("t4", "complete_task", serde_json::json!({"status": "completed", "summary": "Done, warm now."})),
    ]);
    handle_inbound_message(&pool, &set_v2_provider, &embedder, &registry, "telegram", "dm", "chat-1", "tg-1", "actually be warm", None)
        .await
        .unwrap();

    // Turn 3: roll back to v1 via chat — the agent lists versions first, then rolls back.
    let rollback_provider = FakeLlmProvider::sequence(vec![
        canned_response("personality"),
        tool_use_response("t5", "list_personality_versions", serde_json::json!({})),
        tool_use_response("t6", "rollback_personality", serde_json::json!({"version": 1})),
        tool_use_response("t7", "complete_task", serde_json::json!({"status": "completed", "summary": "Rolled back to sarcastic."})),
    ]);
    let outcome = handle_inbound_message(
        &pool,
        &rollback_provider,
        &embedder,
        &registry,
        "telegram",
        "dm",
        "chat-1",
        "tg-1",
        "go back to how you were before",
        None,
    )
    .await
    .unwrap();
    assert_eq!(outcome.reply, "Rolled back to sarcastic.");

    // Turn 4: a plain chitchat message should now carry v1's personality again.
    let chitchat_provider =
        FakeLlmProvider::sequence(vec![canned_response("chitchat"), canned_response("Yeah, whatever."), canned_response("NONE")]);
    handle_inbound_message(&pool, &chitchat_provider, &embedder, &registry, "telegram", "dm", "chat-1", "tg-1", "hey", None)
        .await
        .unwrap();

    let requests = chitchat_provider.received_requests.lock().unwrap();
    let system = requests[1].system.as_ref().unwrap();
    assert!(system.contains("Adopt this personality in your replies: Be sarcastic and blunt."));
}
