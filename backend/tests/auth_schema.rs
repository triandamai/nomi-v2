use sqlx::PgPool;
use uuid::Uuid;

#[sqlx::test]
async fn users_gains_is_platform_admin_defaulting_false(pool: PgPool) {
    let user_id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id")
        .fetch_one(&pool)
        .await
        .unwrap();
    let is_platform_admin: bool =
        sqlx::query_scalar("SELECT is_platform_admin FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(!is_platform_admin);
}

#[sqlx::test]
async fn web_credentials_email_is_unique(pool: PgPool) {
    let user_a: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id")
        .fetch_one(&pool)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO web_credentials (user_id, email, password_hash) VALUES ($1, 'a@example.com', 'hash1')",
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
        "INSERT INTO web_credentials (user_id, email, password_hash) VALUES ($1, 'a@example.com', 'hash2')",
    )
    .bind(user_b)
    .execute(&pool)
    .await
    .unwrap_err();

    assert_eq!(
        err.as_database_error().unwrap().constraint(),
        Some("web_credentials_email_key")
    );
}

#[sqlx::test]
async fn refresh_tokens_token_hash_is_unique(pool: PgPool) {
    let user_id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id")
        .fetch_one(&pool)
        .await
        .unwrap();

    sqlx::query(
        "INSERT INTO refresh_tokens (user_id, token_hash, expires_at) VALUES ($1, 'hash-a', now() + interval '30 days')",
    )
    .bind(user_id)
    .execute(&pool)
    .await
    .unwrap();

    let err = sqlx::query(
        "INSERT INTO refresh_tokens (user_id, token_hash, expires_at) VALUES ($1, 'hash-a', now() + interval '30 days')",
    )
    .bind(user_id)
    .execute(&pool)
    .await
    .unwrap_err();

    assert_eq!(
        err.as_database_error().unwrap().constraint(),
        Some("refresh_tokens_token_hash_key")
    );
}

#[sqlx::test]
async fn membership_role_check_rejects_invalid_value(pool: PgPool) {
    let org_id: Uuid =
        sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Acme') RETURNING id")
            .fetch_one(&pool)
            .await
            .unwrap();
    let user_id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id")
        .fetch_one(&pool)
        .await
        .unwrap();

    let err = sqlx::query("INSERT INTO memberships (org_id, user_id, role) VALUES ($1, $2, 'superowner')")
        .bind(org_id)
        .bind(user_id)
        .execute(&pool)
        .await
        .unwrap_err();

    assert_eq!(
        err.as_database_error().unwrap().constraint(),
        Some("memberships_role_check")
    );
}

#[sqlx::test]
async fn membership_status_check_rejects_invalid_value(pool: PgPool) {
    let org_id: Uuid =
        sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Acme') RETURNING id")
            .fetch_one(&pool)
            .await
            .unwrap();
    let user_id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id")
        .fetch_one(&pool)
        .await
        .unwrap();

    let err = sqlx::query(
        "INSERT INTO memberships (org_id, user_id, role, status) VALUES ($1, $2, 'member', 'banned')",
    )
    .bind(org_id)
    .bind(user_id)
    .execute(&pool)
    .await
    .unwrap_err();

    assert_eq!(
        err.as_database_error().unwrap().constraint(),
        Some("memberships_status_check")
    );
}

#[sqlx::test]
async fn org_invites_role_check_rejects_invalid_value(pool: PgPool) {
    let org_id: Uuid =
        sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Acme') RETURNING id")
            .fetch_one(&pool)
            .await
            .unwrap();
    let inviter: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id")
        .fetch_one(&pool)
        .await
        .unwrap();

    let err = sqlx::query(
        "INSERT INTO org_invites (code, org_id, role, invited_by, expires_at) VALUES ('BADROLE', $1, 'superowner', $2, now() + interval '7 days')",
    )
    .bind(org_id)
    .bind(inviter)
    .execute(&pool)
    .await
    .unwrap_err();

    assert_eq!(
        err.as_database_error().unwrap().constraint(),
        Some("org_invites_role_check")
    );
}

#[sqlx::test]
async fn agent_sessions_status_check_rejects_invalid_value(pool: PgPool) {
    let org_id: Uuid =
        sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Acme') RETURNING id")
            .fetch_one(&pool)
            .await
            .unwrap();
    let session_id: Uuid = sqlx::query_scalar(
        "INSERT INTO sessions (org_id, channel, chat_id) VALUES ($1, 'telegram', 'chat-1') RETURNING id",
    )
    .bind(org_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    let user_id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id")
        .fetch_one(&pool)
        .await
        .unwrap();
    let identity_id: Uuid = sqlx::query_scalar(
        "INSERT INTO channel_identities (user_id, channel, channel_user_id) VALUES ($1, 'telegram', '999') RETURNING id",
    )
    .bind(user_id)
    .fetch_one(&pool)
    .await
    .unwrap();

    let err = sqlx::query(
        "INSERT INTO agent_sessions (session_id, sender_channel_identity_id, agent_type, status) VALUES ($1, $2, 'booking', 'Active')",
    )
    .bind(session_id)
    .bind(identity_id)
    .execute(&pool)
    .await
    .unwrap_err();

    assert_eq!(
        err.as_database_error().unwrap().constraint(),
        Some("agent_sessions_status_check")
    );
}
