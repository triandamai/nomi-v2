use sqlx::PgPool;
use uuid::Uuid;

#[sqlx::test(migrations = "../../migrations")]
async fn channel_identity_is_unique_per_channel_and_channel_user_id(pool: PgPool) {
    let user_a: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id")
        .fetch_one(&pool)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO channel_identities (user_id, channel, channel_user_id) VALUES ($1, 'telegram', '12345')",
    )
    .bind(user_a)
    .execute(&pool)
    .await
    .unwrap();

    let user_b: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id")
        .fetch_one(&pool)
        .await
        .unwrap();
    let err = sqlx::query(
        "INSERT INTO channel_identities (user_id, channel, channel_user_id) VALUES ($1, 'telegram', '12345')",
    )
    .bind(user_b)
    .execute(&pool)
    .await
    .unwrap_err();

    let db_err = err.as_database_error().unwrap();
    assert_eq!(
        db_err.constraint(),
        Some("channel_identities_channel_channel_user_id_key")
    );
}
