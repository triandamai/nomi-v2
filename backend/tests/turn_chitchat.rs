mod support;

use sqlx::PgPool;
use uuid::Uuid;

use nomi_orchestrator::llm::{ContentBlock, LlmResponse, LlmRole, StopReason};
use nomi_orchestrator::turn::chitchat::run_chitchat_turn;

use support::{dummy_embedding, FakeEmbeddingProvider, FakeLlmProvider};

async fn seed_session_and_user(pool: &PgPool) -> (Uuid, Uuid) {
    let org_id: Uuid = sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Acme') RETURNING id")
        .fetch_one(pool)
        .await
        .unwrap();
    let session_id: Uuid = sqlx::query_scalar(
        "INSERT INTO sessions (org_id, channel, chat_id) VALUES ($1, 'telegram', 'chat-1') RETURNING id",
    )
    .bind(org_id)
    .fetch_one(pool)
    .await
    .unwrap();
    let user_id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id")
        .fetch_one(pool)
        .await
        .unwrap();
    (session_id, user_id)
}

fn canned_response(text: &str) -> LlmResponse {
    LlmResponse {
        content: vec![ContentBlock::Text { text: text.to_string() }],
        stop_reason: StopReason::EndTurn,
        input_tokens: 10,
        output_tokens: 5,
    }
}

fn make_embedding(first: f32) -> Vec<f32> {
    let mut v = vec![0.0f32; 1536];
    v[0] = first;
    v
}

fn make_embedding_literal(first: f32) -> String {
    make_embedding(first).iter().map(|x| x.to_string()).collect::<Vec<_>>().join(",")
}

#[sqlx::test]
async fn persists_reply_and_chitchat_reply_event_on_success(pool: PgPool) {
    let (session_id, user_id) = seed_session_and_user(&pool).await;
    let mut conn = pool.acquire().await.unwrap();
    let provider = FakeLlmProvider::success(canned_response("Hello there!"));
    let embedder = FakeEmbeddingProvider::success(dummy_embedding());

    let reply = run_chitchat_turn(&mut conn, &provider, &embedder, session_id, user_id, "hi")
        .await
        .unwrap();
    assert_eq!(reply, "Hello there!");

    let (sender, content): (Option<Uuid>, String) = sqlx::query_as(
        "SELECT sender_channel_identity_id, content FROM messages WHERE session_id = $1 ORDER BY created_at DESC LIMIT 1",
    )
    .bind(session_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(sender, None);
    assert_eq!(content, "Hello there!");

    let payload: serde_json::Value = sqlx::query_scalar(
        "SELECT payload FROM agent_events WHERE session_id = $1 AND event_type = 'ChitchatReply'",
    )
    .bind(session_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(payload, serde_json::json!({"input_tokens": 10, "output_tokens": 5}));
}

#[sqlx::test]
async fn persists_nothing_when_the_provider_call_fails(pool: PgPool) {
    let (session_id, user_id) = seed_session_and_user(&pool).await;
    let mut conn = pool.acquire().await.unwrap();
    let provider = FakeLlmProvider::failure("provider unavailable");
    let embedder = FakeEmbeddingProvider::success(dummy_embedding());

    let result = run_chitchat_turn(&mut conn, &provider, &embedder, session_id, user_id, "hi").await;
    assert!(result.is_err());

    let message_count: i64 = sqlx::query_scalar("SELECT count(*) FROM messages WHERE session_id = $1")
        .bind(session_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(message_count, 0);

    let event_count: i64 = sqlx::query_scalar("SELECT count(*) FROM agent_events WHERE session_id = $1")
        .bind(session_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(event_count, 0);
}

#[sqlx::test]
async fn keeps_only_the_last_20_messages_ordered_oldest_first(pool: PgPool) {
    let (session_id, user_id) = seed_session_and_user(&pool).await;
    let identity_id: Uuid = sqlx::query_scalar(
        "INSERT INTO channel_identities (user_id, channel, channel_user_id) VALUES ($1, 'telegram', 'u1') RETURNING id",
    )
    .bind(user_id)
    .fetch_one(&pool)
    .await
    .unwrap();

    let base = chrono::Utc::now();
    for i in 0..25 {
        sqlx::query(
            "INSERT INTO messages (session_id, sender_channel_identity_id, content, created_at) VALUES ($1, $2, $3, $4)",
        )
        .bind(session_id)
        .bind(identity_id)
        .bind(format!("seq-{i}"))
        .bind(base + chrono::Duration::milliseconds(i))
        .execute(&pool)
        .await
        .unwrap();
    }

    let mut conn = pool.acquire().await.unwrap();
    let provider = FakeLlmProvider::success(canned_response("ok"));
    let embedder = FakeEmbeddingProvider::success(dummy_embedding());

    run_chitchat_turn(&mut conn, &provider, &embedder, session_id, user_id, "latest").await.unwrap();

    let requests = provider.received_requests.lock().unwrap();
    let sent = &requests[0];
    assert_eq!(sent.messages.len(), 20);
    for (offset, message) in sent.messages.iter().enumerate() {
        let expected_seq = 5 + offset;
        match &message.content[0] {
            ContentBlock::Text { text } => assert_eq!(text, &format!("seq-{expected_seq}")),
            _ => panic!("expected a text block"),
        }
    }
}

#[sqlx::test]
async fn maps_sender_presence_to_role_correctly(pool: PgPool) {
    let (session_id, user_id) = seed_session_and_user(&pool).await;
    let identity_id: Uuid = sqlx::query_scalar(
        "INSERT INTO channel_identities (user_id, channel, channel_user_id) VALUES ($1, 'telegram', 'u1') RETURNING id",
    )
    .bind(user_id)
    .fetch_one(&pool)
    .await
    .unwrap();

    let base = chrono::Utc::now();
    sqlx::query(
        "INSERT INTO messages (session_id, sender_channel_identity_id, content, created_at) VALUES ($1, $2, 'hi', $3)",
    )
    .bind(session_id)
    .bind(identity_id)
    .bind(base)
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO messages (session_id, sender_channel_identity_id, content, created_at) VALUES ($1, NULL, 'hello back', $2)",
    )
    .bind(session_id)
    .bind(base + chrono::Duration::milliseconds(1))
    .execute(&pool)
    .await
    .unwrap();

    let mut conn = pool.acquire().await.unwrap();
    let provider = FakeLlmProvider::success(canned_response("ok"));
    let embedder = FakeEmbeddingProvider::success(dummy_embedding());

    run_chitchat_turn(&mut conn, &provider, &embedder, session_id, user_id, "latest").await.unwrap();

    let requests = provider.received_requests.lock().unwrap();
    let sent = &requests[0];
    assert_eq!(sent.messages[0].role, LlmRole::User);
    assert_eq!(sent.messages[1].role, LlmRole::Assistant);
}

#[sqlx::test]
async fn retrieved_memories_are_folded_into_the_system_prompt_and_linked_to_the_reply(pool: PgPool) {
    let (session_id, user_id) = seed_session_and_user(&pool).await;
    let memory_id: Uuid = sqlx::query_scalar(
        "INSERT INTO memory_items (user_id, content, embedding) VALUES ($1, $2, $3::vector) RETURNING id",
    )
    .bind(user_id)
    .bind("User is vegetarian")
    .bind(format!("[{}]", make_embedding_literal(1.0)))
    .fetch_one(&pool)
    .await
    .unwrap();

    let mut conn = pool.acquire().await.unwrap();
    let provider = FakeLlmProvider::success(canned_response("Got it, no meat!"));
    let embedder = FakeEmbeddingProvider::success(make_embedding(1.0));

    run_chitchat_turn(&mut conn, &provider, &embedder, session_id, user_id, "what should I eat?")
        .await
        .unwrap();

    let requests = provider.received_requests.lock().unwrap();
    let system = requests[0].system.as_ref().unwrap();
    assert!(system.contains("Relevant things you know about this user"));
    assert!(system.contains("User is vegetarian"));

    let linked_memory_id: Uuid = sqlx::query_scalar(
        "SELECT memory_id FROM message_memory_usage mu \
         JOIN messages m ON mu.message_id = m.id \
         WHERE m.session_id = $1 AND m.sender_channel_identity_id IS NULL",
    )
    .bind(session_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(linked_memory_id, memory_id);
}

#[sqlx::test]
async fn a_failing_embedding_provider_does_not_prevent_a_normal_reply(pool: PgPool) {
    let (session_id, user_id) = seed_session_and_user(&pool).await;
    let mut conn = pool.acquire().await.unwrap();
    let provider = FakeLlmProvider::success(canned_response("Still here!"));
    let embedder = FakeEmbeddingProvider::failure("embeddings unavailable");

    let reply = run_chitchat_turn(&mut conn, &provider, &embedder, session_id, user_id, "hi")
        .await
        .unwrap();
    assert_eq!(reply, "Still here!");

    let requests = provider.received_requests.lock().unwrap();
    assert_eq!(
        requests[0].system.as_ref().unwrap(),
        "You are a helpful, friendly assistant chatting with the user. Keep replies concise."
    );
}
