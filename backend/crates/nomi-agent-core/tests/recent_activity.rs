//! The memory worker's "recently" note: what a person did lately across their chats, rewritten
//! once a chat goes quiet.

use sqlx::PgPool;
use uuid::Uuid;

use nomi_agent_core::recent_activity::{chats_gone_quiet, for_person, note_chat, Noted};
use nomi_llm::{ContentBlock, LlmResponse, StopReason};
use nomi_test_support::FakeLlmProvider;

async fn person_with_chat(pool: &PgPool) -> (Uuid, Uuid, Uuid) {
    let user_id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(pool).await.unwrap();
    let org: Uuid = sqlx::query_scalar("INSERT INTO organizations (name, is_personal) VALUES ('Home', true) RETURNING id").fetch_one(pool).await.unwrap();
    let session_id: Uuid = sqlx::query_scalar("INSERT INTO sessions (org_id, channel, chat_id, user_id, title) VALUES ($1, 'web', 'c', $2, 'Lunch money') RETURNING id")
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

/// A message `ago` (an interval) from the person, or from `agent` when there's no identity.
async fn said(pool: &PgPool, session_id: Uuid, identity: Option<Uuid>, agent: Option<&str>, content: &str, ago: &str) {
    sqlx::query("INSERT INTO messages (session_id, sender_channel_identity_id, agent_display_name, content, created_at) VALUES ($1, $2, $3, $4, now() - $5::interval)")
        .bind(session_id)
        .bind(identity)
        .bind(agent)
        .bind(content)
        .bind(ago)
        .execute(pool)
        .await
        .unwrap();
}

fn note(text: &str) -> LlmResponse {
    LlmResponse { content: vec![ContentBlock::Text { text: text.into() }], stop_reason: StopReason::EndTurn, input_tokens: 1, output_tokens: 1 }
}

fn input_of(provider: &FakeLlmProvider) -> String {
    match &provider.received_requests.lock().unwrap()[0].messages[0].content[0] {
        ContentBlock::Text { text } => text.clone(),
        _ => panic!("text"),
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_chat_is_noted_once_it_goes_quiet_naming_the_agent_who_worked_on_it(pool: PgPool) {
    let (user_id, session_id, identity) = person_with_chat(&pool).await;
    said(&pool, session_id, Some(identity), None, "Save lunch 50k", "2 minutes").await;
    said(&pool, session_id, None, Some("Dana"), "Logged lunch, Rp 50.000.", "1 minute").await;
    assert!(chats_gone_quiet(&pool, 10).await.unwrap().is_empty(), "still going: not noted yet");

    sqlx::query("UPDATE messages SET created_at = created_at - interval '10 minutes' WHERE session_id = $1").bind(session_id).execute(&pool).await.unwrap();
    assert_eq!(chats_gone_quiet(&pool, 10).await.unwrap(), vec![(session_id, user_id)]);

    let provider = FakeLlmProvider::success(note("Today: Dana logged lunch, Rp 50k"));
    assert_eq!(note_chat(&pool, &provider, session_id, user_id).await.unwrap(), Noted::Updated { messages: 2, chars: 32 });
    let input = input_of(&provider);
    assert!(input.contains("Note so far:\n(none yet)") && input.contains("\"Lunch money\""), "{input}");
    assert!(input.contains("Person: Save lunch 50k") && input.contains("Dana: Logged lunch, Rp 50.000."), "{input}");

    assert_eq!(for_person(&pool, user_id).await.unwrap().as_deref(), Some("Today: Dana logged lunch, Rp 50k"));
    assert!(chats_gone_quiet(&pool, 10).await.unwrap().is_empty(), "noted through its last message");
}

#[sqlx::test(migrations = "../../migrations")]
async fn only_whats_new_goes_in_and_the_note_so_far_is_carried_on(pool: PgPool) {
    let (user_id, session_id, identity) = person_with_chat(&pool).await;
    said(&pool, session_id, Some(identity), None, "Save lunch 50k", "30 minutes").await;
    let first = FakeLlmProvider::success(note("Today: Dana logged lunch, Rp 50k"));
    note_chat(&pool, &first, session_id, user_id).await.unwrap();

    said(&pool, session_id, Some(identity), None, "And coffee 20k", "10 minutes").await;
    let second = FakeLlmProvider::success(note("Today: Dana logged lunch Rp 50k and coffee Rp 20k"));
    assert_eq!(note_chat(&pool, &second, session_id, user_id).await.unwrap(), Noted::Updated { messages: 1, chars: 49 });
    let input = input_of(&second);
    assert!(input.contains("Note so far:\nToday: Dana logged lunch, Rp 50k"), "{input}");
    assert!(input.contains("And coffee 20k") && !input.contains("Save lunch 50k"), "{input}");
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_failed_model_call_leaves_the_chat_to_try_again(pool: PgPool) {
    let (user_id, session_id, identity) = person_with_chat(&pool).await;
    said(&pool, session_id, Some(identity), None, "Save lunch 50k", "10 minutes").await;
    let provider = FakeLlmProvider::success(note("   "));
    assert!(matches!(note_chat(&pool, &provider, session_id, user_id).await.unwrap(), Noted::ModelFailed(_)));
    assert_eq!(for_person(&pool, user_id).await.unwrap(), None);
    assert_eq!(chats_gone_quiet(&pool, 10).await.unwrap(), vec![(session_id, user_id)]);
}

#[sqlx::test(migrations = "../../migrations")]
async fn old_chats_and_shown_thinking_are_left_out(pool: PgPool) {
    let (_, session_id, identity) = person_with_chat(&pool).await;
    said(&pool, session_id, Some(identity), None, "Save lunch 50k", "3 days").await;
    assert!(chats_gone_quiet(&pool, 10).await.unwrap().is_empty(), "quiet for days: before the look-back");

    sqlx::query("DELETE FROM messages").execute(&pool).await.unwrap();
    said(&pool, session_id, None, None, "🧠 thinking", "10 minutes").await;
    assert!(chats_gone_quiet(&pool, 10).await.unwrap().is_empty(), "shown thinking isn't something said");
}
