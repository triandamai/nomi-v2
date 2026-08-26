use sqlx::PgPool;
use uuid::Uuid;

async fn make_org(pool: &PgPool) -> Uuid {
    sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Acme') RETURNING id")
        .fetch_one(pool)
        .await
        .unwrap()
}

async fn make_user_with_channel_identity(pool: &PgPool, channel_user_id: &str) -> (Uuid, Uuid) {
    let user_id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id")
        .fetch_one(pool)
        .await
        .unwrap();
    let identity_id: Uuid = sqlx::query_scalar(
        "INSERT INTO channel_identities (user_id, channel, channel_user_id) VALUES ($1, 'telegram', $2) RETURNING id",
    )
    .bind(user_id)
    .bind(channel_user_id)
    .fetch_one(pool)
    .await
    .unwrap();
    (user_id, identity_id)
}

#[sqlx::test(migrations = "../../migrations")]
async fn session_requires_an_org(pool: PgPool) {
    let err = sqlx::query(
        "INSERT INTO sessions (org_id, channel, chat_id) VALUES (gen_random_uuid(), 'telegram', 'chat-1')",
    )
    .execute(&pool)
    .await
    .unwrap_err();

    // gen_random_uuid() here is a random, non-existent org — this must fail the FK, not just NOT NULL.
    assert!(err.as_database_error().unwrap().message().contains("violates foreign key constraint"));
}

#[sqlx::test(migrations = "../../migrations")]
async fn session_is_unique_per_channel_and_chat_id(pool: PgPool) {
    let org_id = make_org(&pool).await;
    sqlx::query("INSERT INTO sessions (org_id, channel, chat_id) VALUES ($1, 'telegram', 'chat-1')")
        .bind(org_id)
        .execute(&pool)
        .await
        .unwrap();

    let err = sqlx::query("INSERT INTO sessions (org_id, channel, chat_id) VALUES ($1, 'telegram', 'chat-1')")
        .bind(org_id)
        .execute(&pool)
        .await
        .unwrap_err();

    assert_eq!(
        err.as_database_error().unwrap().constraint(),
        Some("sessions_channel_chat_id_key")
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn message_sender_is_nullable_for_assistant_replies(pool: PgPool) {
    let org_id = make_org(&pool).await;
    let session_id: Uuid = sqlx::query_scalar(
        "INSERT INTO sessions (org_id, channel, chat_id) VALUES ($1, 'telegram', 'chat-1') RETURNING id",
    )
    .bind(org_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    let (_user_id, identity_id) = make_user_with_channel_identity(&pool, "111").await;

    sqlx::query(
        "INSERT INTO messages (session_id, sender_channel_identity_id, content) VALUES ($1, $2, 'hi')",
    )
    .bind(session_id)
    .bind(identity_id)
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query("INSERT INTO messages (session_id, sender_channel_identity_id, content) VALUES ($1, NULL, 'hello back')")
        .bind(session_id)
        .execute(&pool)
        .await
        .unwrap();

    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM messages WHERE session_id = $1")
        .bind(session_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 2);
}

#[sqlx::test(migrations = "../../migrations")]
async fn session_participant_cannot_be_added_twice(pool: PgPool) {
    let org_id = make_org(&pool).await;
    let session_id: Uuid = sqlx::query_scalar(
        "INSERT INTO sessions (org_id, channel, chat_type, chat_id) VALUES ($1, 'web', 'group', 'group-1') RETURNING id",
    )
    .bind(org_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    let (user_id, _identity_id) = make_user_with_channel_identity(&pool, "222").await;

    sqlx::query("INSERT INTO session_participants (session_id, user_id) VALUES ($1, $2)")
        .bind(session_id)
        .bind(user_id)
        .execute(&pool)
        .await
        .unwrap();

    let err = sqlx::query("INSERT INTO session_participants (session_id, user_id) VALUES ($1, $2)")
        .bind(session_id)
        .bind(user_id)
        .execute(&pool)
        .await
        .unwrap_err();

    assert_eq!(
        err.as_database_error().unwrap().constraint(),
        Some("session_participants_pkey")
    );
}
