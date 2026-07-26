use sqlx::PgPool;
use uuid::Uuid;

use nomi_orchestrator::web_identity::ensure_web_channel_identity;

async fn seed_user(pool: &PgPool) -> Uuid {
    sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(pool).await.unwrap()
}

#[sqlx::test]
async fn creates_a_new_identity_for_a_first_time_user(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let identity_id = ensure_web_channel_identity(&pool, user_id).await.unwrap();

    let (channel, channel_user_id, row_user_id): (String, String, Uuid) = sqlx::query_as(
        "SELECT channel, channel_user_id, user_id FROM channel_identities WHERE id = $1",
    )
    .bind(identity_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(channel, "web");
    assert_eq!(channel_user_id, user_id.to_string());
    assert_eq!(row_user_id, user_id);
}

#[sqlx::test]
async fn is_idempotent_across_repeated_calls(pool: PgPool) {
    let user_id = seed_user(&pool).await;
    let first = ensure_web_channel_identity(&pool, user_id).await.unwrap();
    let second = ensure_web_channel_identity(&pool, user_id).await.unwrap();
    assert_eq!(first, second);

    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM channel_identities WHERE user_id = $1")
        .bind(user_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 1);
}

#[sqlx::test]
async fn different_users_get_different_identities(pool: PgPool) {
    let user_a = seed_user(&pool).await;
    let user_b = seed_user(&pool).await;
    let identity_a = ensure_web_channel_identity(&pool, user_a).await.unwrap();
    let identity_b = ensure_web_channel_identity(&pool, user_b).await.unwrap();
    assert_ne!(identity_a, identity_b);
}

#[sqlx::test]
async fn concurrent_calls_for_the_same_user_do_not_duplicate_and_resolve_to_the_same_identity(pool: PgPool) {
    let user_id = seed_user(&pool).await;

    let call1 = ensure_web_channel_identity(&pool, user_id);
    let call2 = ensure_web_channel_identity(&pool, user_id);
    let (result1, result2) = tokio::join!(call1, call2);

    let identity1 = result1.unwrap();
    let identity2 = result2.unwrap();
    assert_eq!(identity1, identity2);

    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM channel_identities WHERE user_id = $1")
        .bind(user_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 1);
}
