//! "Summarise and continue": the older part of a chat becomes one summary message, and only that
//! and the newest messages are sent to the model afterwards.

use sqlx::PgPool;
use uuid::Uuid;

use nomi_agent_core::chat_summary::{has_older_messages, summarize_chat, Summarized, KEEP_RECENT_MESSAGES};
use nomi_llm::{ContentBlock, LlmResponse, StopReason};
use nomi_test_support::FakeLlmProvider;

async fn chat_of(pool: &PgPool, count: i64) -> (Uuid, Vec<Uuid>) {
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
    let mut ids = Vec::new();
    for i in 0..count {
        let id: Uuid = sqlx::query_scalar(
            "INSERT INTO messages (session_id, sender_channel_identity_id, content, created_at) VALUES ($1, $2, $3, now() - make_interval(mins => $4)) RETURNING id",
        )
        .bind(session_id)
        .bind(if i % 2 == 0 { Some(identity) } else { None })
        .bind(format!("message {i}"))
        .bind((count - i) as i32)
        .fetch_one(pool)
        .await
        .unwrap();
        ids.push(id);
    }
    (session_id, ids)
}

fn written(text: &str) -> LlmResponse {
    LlmResponse { content: vec![ContentBlock::Text { text: text.into() }], stop_reason: StopReason::EndTurn, input_tokens: 1, output_tokens: 1 }
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_older_messages_are_replaced_by_a_summary_before_the_newest(pool: PgPool) {
    let total = KEEP_RECENT_MESSAGES as i64 + 4;
    let (session_id, ids) = chat_of(&pool, total).await;
    assert!(has_older_messages(&pool, session_id).await.unwrap());

    let provider = FakeLlmProvider::success(written("Planning a Bali trip."));
    let outcome = summarize_chat(&pool, &provider, session_id).await.unwrap();
    let Summarized::Done { message_id, replaced } = outcome else { panic!("{outcome:?}") };
    assert_eq!(replaced, 4);

    let (content, blocks, summary_at): (String, serde_json::Value, chrono::DateTime<chrono::Utc>) =
        sqlx::query_as("SELECT content, content_blocks, created_at FROM messages WHERE id = $1").bind(message_id).fetch_one(&pool).await.unwrap();
    assert_eq!(content, "Planning a Bali trip.");
    assert_eq!(blocks[0]["kind"], "summary");
    let first_kept_at: chrono::DateTime<chrono::Utc> = sqlx::query_scalar("SELECT created_at FROM messages WHERE id = $1").bind(ids[4]).fetch_one(&pool).await.unwrap();
    assert!(summary_at < first_kept_at);

    let replaced_ids: Vec<Uuid> = sqlx::query_scalar("SELECT id FROM messages WHERE summarized_by = $1 ORDER BY created_at").bind(message_id).fetch_all(&pool).await.unwrap();
    assert_eq!(replaced_ids, ids[..4].to_vec());
    assert!(!has_older_messages(&pool, session_id).await.unwrap());
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_second_summary_folds_the_first_into_itself(pool: PgPool) {
    let (session_id, _) = chat_of(&pool, KEEP_RECENT_MESSAGES as i64 + 4).await;
    let Summarized::Done { message_id: first, .. } = summarize_chat(&pool, &FakeLlmProvider::success(written("one")), session_id).await.unwrap() else { panic!() };
    // Nothing new behind the kept messages: the summary alone isn't worth another.
    assert_eq!(summarize_chat(&pool, &FakeLlmProvider::success(written("two")), session_id).await.unwrap(), Summarized::NothingToSummarise);
    sqlx::query("INSERT INTO messages (session_id, content) SELECT $1, 'later ' || g FROM generate_series(1, 3) g").bind(session_id).execute(&pool).await.unwrap();
    let provider = FakeLlmProvider::success(written("one and two"));
    let Summarized::Done { message_id: second, .. } = summarize_chat(&pool, &provider, session_id).await.unwrap() else { panic!() };
    let folded: Option<Uuid> = sqlx::query_scalar("SELECT summarized_by FROM messages WHERE id = $1").bind(first).fetch_one(&pool).await.unwrap();
    assert_eq!(folded, Some(second));
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_failed_model_call_changes_nothing(pool: PgPool) {
    let (session_id, _) = chat_of(&pool, KEEP_RECENT_MESSAGES as i64 + 4).await;
    assert!(summarize_chat(&pool, &FakeLlmProvider::failure("down"), session_id).await.is_err());
    let marked: i64 = sqlx::query_scalar("SELECT count(*) FROM messages WHERE session_id = $1 AND (summarized_by IS NOT NULL OR content_blocks IS NOT NULL)").bind(session_id).fetch_one(&pool).await.unwrap();
    assert_eq!(marked, 0);
}
