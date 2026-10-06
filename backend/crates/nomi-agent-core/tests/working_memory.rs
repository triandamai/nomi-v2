//! The memory worker's jobs: folding long chats into a running summary, and tidying each
//! person's memories.

use sqlx::PgPool;
use uuid::Uuid;

use nomi_agent_core::working_memory::{consolidate, people_to_consolidate, sessions_to_summarize, summarize_session, Tidied};
use nomi_llm::{ContentBlock, LlmResponse, StopReason};
use nomi_test_support::FakeLlmProvider;

async fn person_with_chat(pool: &PgPool) -> (Uuid, Uuid, Uuid) {
    let user_id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(pool).await.unwrap();
    let org: Uuid = sqlx::query_scalar("INSERT INTO organizations (name, is_personal) VALUES ('Home', true) RETURNING id").fetch_one(pool).await.unwrap();
    let session_id: Uuid = sqlx::query_scalar("INSERT INTO sessions (org_id, channel, chat_id, user_id) VALUES ($1, 'web', 'c', $2) RETURNING id")
        .bind(org)
        .bind(user_id)
        .fetch_one(pool)
        .await
        .unwrap();
    let identity: Uuid = sqlx::query_scalar("INSERT INTO channel_identities (user_id, channel, channel_user_id) VALUES ($1, 'web', $2) RETURNING id")
        .bind(user_id)
        .bind(user_id.to_string())
        .fetch_one(pool)
        .await
        .unwrap();
    (user_id, session_id, identity)
}

/// `count` messages, a minute apart, ending a minute ago, alternating person and Nomi, with a
/// shown-thinking note after every Nomi reply (which never counts as conversation).
async fn chat(pool: &PgPool, session_id: Uuid, identity: Uuid, count: i64) {
    for i in 0..count {
        let at = format!("{} minutes", count - i);
        let from_person = i % 2 == 0;
        sqlx::query(
            "INSERT INTO messages (session_id, sender_channel_identity_id, content, created_at) \
             VALUES ($1, $2, $3, now() - $4::interval)",
        )
        .bind(session_id)
        .bind(if from_person { Some(identity) } else { None })
        .bind(format!("message {i}"))
        .bind(&at)
        .execute(pool)
        .await
        .unwrap();
        if !from_person {
            sqlx::query("INSERT INTO messages (session_id, content, created_at) VALUES ($1, '🧠 thinking', now() - $2::interval)")
                .bind(session_id)
                .bind(&at)
                .execute(pool)
                .await
                .unwrap();
        }
    }
}

fn summary(text: &str) -> LlmResponse {
    LlmResponse { content: vec![ContentBlock::Text { text: text.into() }], stop_reason: StopReason::EndTurn, input_tokens: 1, output_tokens: 1 }
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_long_chat_folds_its_older_messages_into_a_summary(pool: PgPool) {
    let (_, session_id, identity) = person_with_chat(&pool).await;
    // 35 messages: the latest 20 stay in full, 15 older aren't enough to fold yet.
    chat(&pool, session_id, identity, 35).await;
    assert!(sessions_to_summarize(&pool, 10).await.unwrap().is_empty());

    // 10 more: 25 older messages now wait behind the latest 20.
    sqlx::query("DELETE FROM messages WHERE session_id = $1").bind(session_id).execute(&pool).await.unwrap();
    chat(&pool, session_id, identity, 45).await;
    let due = sessions_to_summarize(&pool, 10).await.unwrap();
    assert_eq!(due.iter().map(|d| d.0).collect::<Vec<_>>(), vec![session_id]);

    let provider = FakeLlmProvider::success(summary("Planning a Bali trip in May.\nBudget about Rp10 jt."));
    assert!(summarize_session(&pool, &provider, session_id).await.unwrap());

    let request = &provider.received_requests.lock().unwrap()[0];
    let input = match &request.messages[0].content[0] {
        ContentBlock::Text { text } => text.clone(),
        _ => panic!("text"),
    };
    assert!(input.contains("Person: message 0") && input.contains("message 24") && !input.contains("message 25"), "{input}");
    assert!(!input.contains("thinking"), "shown thinking isn't conversation");

    let (text, through_is_25th): (Option<String>, bool) = sqlx::query_as(
        "SELECT summary, summary_through = (SELECT created_at FROM messages WHERE session_id = $1 AND content = 'message 24') FROM sessions WHERE id = $1",
    )
    .bind(session_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(text.as_deref(), Some("Planning a Bali trip in May.\nBudget about Rp10 jt."));
    assert!(through_is_25th);
    assert!(sessions_to_summarize(&pool, 10).await.unwrap().is_empty(), "nothing left to fold");
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_failed_summary_changes_nothing(pool: PgPool) {
    let (_, session_id, identity) = person_with_chat(&pool).await;
    chat(&pool, session_id, identity, 45).await;
    let provider = FakeLlmProvider::failure("down");
    assert!(!summarize_session(&pool, &provider, session_id).await.unwrap());
    let summary: Option<String> = sqlx::query_scalar("SELECT summary FROM sessions WHERE id = $1").bind(session_id).fetch_one(&pool).await.unwrap();
    assert_eq!(summary, None);
}

fn vector(first: f32, second: f32) -> String {
    let mut v = vec![0.0f32; 1536];
    v[0] = first;
    v[1] = second;
    format!("[{}]", v.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(","))
}

async fn memory(pool: &PgPool, user_id: Uuid, content: &str, embedding: &str, weight: f64, last_used_days_ago: Option<i32>) -> Uuid {
    sqlx::query_scalar(
        "INSERT INTO memory_items (user_id, content, embedding, weight, embedding_provider, embedding_model, created_at, last_used_at) \
         VALUES ($1, $2, $3::vector, $4, 'fake', 'fake-model', now() - interval '200 days', now() - make_interval(days => $5)) RETURNING id",
    )
    .bind(user_id)
    .bind(content)
    .bind(embedding)
    .bind(weight)
    .bind(last_used_days_ago.unwrap_or(0))
    .fetch_one(pool)
    .await
    .unwrap()
}

#[sqlx::test(migrations = "../../migrations")]
async fn consolidation_merges_fades_and_archives(pool: PgPool) {
    let (user_id, _, _) = person_with_chat(&pool).await;
    // The same fact twice: merged into the stronger one.
    let strong = memory(&pool, user_id, "Vegetarian", &vector(1.0, 0.0), 2.0, Some(1)).await;
    let echo = memory(&pool, user_id, "Doesn't eat meat", &vector(1.0, 0.05), 1.0, Some(1)).await;
    // Unused for 40 days: fades. Faint and unused for 90: archived.
    let stale = memory(&pool, user_id, "Into chess", &vector(0.0, 1.0), 1.0, Some(40)).await;
    let faint = memory(&pool, user_id, "Liked a podcast once", &vector(0.5, -1.0), 0.12, Some(90)).await;
    // In use: left alone.
    let fresh = memory(&pool, user_id, "Partner Rina", &vector(-1.0, 0.0), 1.0, Some(2)).await;

    assert_eq!(people_to_consolidate(&pool, 10).await.unwrap(), vec![user_id]);
    let tidied = consolidate(&pool, user_id).await.unwrap();
    assert_eq!(tidied, Tidied { merged: 1, faded: 2, archived: 1 });

    let state = |id: Uuid| {
        let pool = pool.clone();
        async move {
            sqlx::query_as::<_, (f64, bool, Option<Uuid>)>("SELECT weight, archived_at IS NOT NULL, superseded_by FROM memory_items WHERE id = $1")
                .bind(id)
                .fetch_one(&pool)
                .await
                .unwrap()
        }
    };
    assert_eq!(state(echo).await, (1.0, true, Some(strong)));
    assert!((state(strong).await.0 - 2.2).abs() < 1e-9);
    assert!((state(stale).await.0 - 0.95).abs() < 1e-9);
    assert!(state(faint).await.1);
    assert_eq!(state(fresh).await, (1.0, false, None));

    // Done for today.
    assert!(people_to_consolidate(&pool, 10).await.unwrap().is_empty());
}
