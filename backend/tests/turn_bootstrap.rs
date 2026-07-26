use sqlx::PgPool;
use uuid::Uuid;

use nomi_orchestrator::turn::bootstrap::bootstrap_identity_and_session;

#[sqlx::test]
async fn new_sender_creates_user_org_membership_identity_and_session(pool: PgPool) {
    let result = bootstrap_identity_and_session(&pool, "telegram", "dm", "chat-1", "tg-user-1")
        .await
        .unwrap();

    let user_count: i64 = sqlx::query_scalar("SELECT count(*) FROM users WHERE id = $1")
        .bind(result.user_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(user_count, 1);

    let (is_personal, name): (bool, String) =
        sqlx::query_as("SELECT is_personal, name FROM organizations WHERE id = $1")
            .bind(result.org_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(is_personal);
    assert_eq!(name, "Personal");

    let role: String = sqlx::query_scalar(
        "SELECT role FROM memberships WHERE org_id = $1 AND user_id = $2",
    )
    .bind(result.org_id)
    .bind(result.user_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(role, "owner");

    let (channel, channel_user_id): (String, String) = sqlx::query_as(
        "SELECT channel, channel_user_id FROM channel_identities WHERE id = $1",
    )
    .bind(result.sender_channel_identity_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(channel, "telegram");
    assert_eq!(channel_user_id, "tg-user-1");

    let (org_id, channel, chat_type, chat_id): (Uuid, String, String, String) = sqlx::query_as(
        "SELECT org_id, channel, chat_type, chat_id FROM sessions WHERE id = $1",
    )
    .bind(result.session_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(org_id, result.org_id);
    assert_eq!(channel, "telegram");
    assert_eq!(chat_type, "dm");
    assert_eq!(chat_id, "chat-1");
}

#[sqlx::test]
async fn existing_sender_reuses_identity_org_and_session(pool: PgPool) {
    let first = bootstrap_identity_and_session(&pool, "telegram", "dm", "chat-1", "tg-user-1")
        .await
        .unwrap();
    let second = bootstrap_identity_and_session(&pool, "telegram", "dm", "chat-1", "tg-user-1")
        .await
        .unwrap();

    assert_eq!(first, second);

    let user_count: i64 = sqlx::query_scalar("SELECT count(*) FROM users").fetch_one(&pool).await.unwrap();
    assert_eq!(user_count, 1);
    let session_count: i64 = sqlx::query_scalar("SELECT count(*) FROM sessions").fetch_one(&pool).await.unwrap();
    assert_eq!(session_count, 1);
}

#[sqlx::test]
async fn existing_sender_new_chat_creates_a_new_session_under_the_same_personal_org(pool: PgPool) {
    let first = bootstrap_identity_and_session(&pool, "telegram", "dm", "chat-1", "tg-user-1")
        .await
        .unwrap();
    let second = bootstrap_identity_and_session(&pool, "telegram", "group", "chat-2", "tg-user-1")
        .await
        .unwrap();

    assert_eq!(first.user_id, second.user_id);
    assert_eq!(first.org_id, second.org_id);
    assert_eq!(first.sender_channel_identity_id, second.sender_channel_identity_id);
    assert_ne!(first.session_id, second.session_id);

    let session_count: i64 = sqlx::query_scalar("SELECT count(*) FROM sessions").fetch_one(&pool).await.unwrap();
    assert_eq!(session_count, 2);
}

#[sqlx::test]
async fn resolves_the_personal_org_even_when_the_user_also_belongs_to_a_named_org(pool: PgPool) {
    let bootstrapped = bootstrap_identity_and_session(&pool, "telegram", "dm", "chat-1", "tg-user-1")
        .await
        .unwrap();

    let acme_org_id: Uuid = sqlx::query_scalar(
        "INSERT INTO organizations (name, is_personal) VALUES ('Acme', false) RETURNING id",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    sqlx::query("INSERT INTO memberships (org_id, user_id, role) VALUES ($1, $2, 'member')")
        .bind(acme_org_id)
        .bind(bootstrapped.user_id)
        .execute(&pool)
        .await
        .unwrap();

    let second = bootstrap_identity_and_session(&pool, "telegram", "group", "chat-2", "tg-user-1")
        .await
        .unwrap();

    assert_eq!(second.org_id, bootstrapped.org_id);
    assert_ne!(second.org_id, acme_org_id);
}
