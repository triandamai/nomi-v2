use sqlx::PgPool;
use uuid::Uuid;

#[sqlx::test]
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

#[sqlx::test]
async fn link_code_is_single_use_via_used_at(pool: PgPool) {
    let user_id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id")
        .fetch_one(&pool)
        .await
        .unwrap();

    sqlx::query(
        "INSERT INTO link_codes (code, user_id, expires_at) VALUES ('ABC234', $1, now() + interval '10 minutes')",
    )
    .bind(user_id)
    .execute(&pool)
    .await
    .unwrap();

    // Marking it used is a normal update, not a constraint — this proves the column round-trips.
    sqlx::query("UPDATE link_codes SET used_at = now() WHERE code = 'ABC234'")
        .execute(&pool)
        .await
        .unwrap();

    let used_at: Option<chrono::DateTime<chrono::Utc>> =
        sqlx::query_scalar("SELECT used_at FROM link_codes WHERE code = 'ABC234'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(used_at.is_some());

    // A second link_codes row can't reuse the same primary key.
    let dup_err = sqlx::query(
        "INSERT INTO link_codes (code, user_id, expires_at) VALUES ('ABC234', $1, now() + interval '10 minutes')",
    )
    .bind(user_id)
    .execute(&pool)
    .await
    .unwrap_err();
    assert_eq!(
        dup_err.as_database_error().unwrap().constraint(),
        Some("link_codes_pkey")
    );
}
