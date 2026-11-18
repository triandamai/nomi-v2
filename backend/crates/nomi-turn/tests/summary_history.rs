//! What a turn sends once a chat has been summarised: the summary and the newest messages, not
//! the messages the summary replaced.

use sqlx::PgPool;
use uuid::Uuid;

use nomi_llm::ContentBlock;

#[sqlx::test(migrations = "../../migrations")]
async fn a_summarised_chat_is_sent_as_its_summary_and_the_newest_messages(pool: PgPool) {
    let user_id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(&pool).await.unwrap();
    let org: Uuid = sqlx::query_scalar("INSERT INTO organizations (name, is_personal) VALUES ('Home', true) RETURNING id").fetch_one(&pool).await.unwrap();
    let session_id: Uuid = sqlx::query_scalar("INSERT INTO sessions (org_id, channel, chat_id, user_id) VALUES ($1, 'web', 'c', $2) RETURNING id")
        .bind(org)
        .bind(user_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    let identity: Uuid = sqlx::query_scalar("INSERT INTO channel_identities (user_id, channel, channel_user_id) VALUES ($1, 'web', $2) RETURNING id")
        .bind(user_id)
        .bind(user_id.to_string())
        .fetch_one(&pool)
        .await
        .unwrap();
    let mut ids = Vec::new();
    for i in 0..6i32 {
        let id: Uuid = sqlx::query_scalar("INSERT INTO messages (session_id, sender_channel_identity_id, content, created_at) VALUES ($1, $2, $3, now() - make_interval(mins => $4)) RETURNING id")
            .bind(session_id)
            .bind(if i % 2 == 0 { Some(identity) } else { None })
            .bind(format!("message {i}"))
            .bind(10 - i)
            .fetch_one(&pool)
            .await
            .unwrap();
        ids.push(id);
    }
    // A summary of the first two, placed before the rest.
    let summary: Uuid = sqlx::query_scalar(
        "INSERT INTO messages (session_id, content, content_blocks, created_at) VALUES ($1, 'Planning a Bali trip.', '[{\"kind\":\"summary\",\"replaced\":2}]', now() - interval '8 minutes 30 seconds') RETURNING id",
    )
    .bind(session_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    sqlx::query("UPDATE messages SET summarized_by = $2 WHERE id = ANY($1)").bind(&ids[..2]).bind(summary).execute(&pool).await.unwrap();

    let mut conn = pool.acquire().await.unwrap();
    let history = nomi_turn::fetch_recent_messages(&mut conn, session_id).await.unwrap();
    let text: String = history
        .iter()
        .flat_map(|m| m.content.iter())
        .map(|b| match b {
            ContentBlock::Text { text } => text.clone(),
            _ => String::new(),
        })
        .collect::<Vec<_>>()
        .join("\n");
    assert!(text.contains("Planning a Bali trip."), "{text}");
    assert!(!text.contains("message 0") && !text.contains("message 1"), "{text}");
    assert!(text.contains("message 2") && text.contains("message 5"), "{text}");
}
