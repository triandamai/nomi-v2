use nomi_orchestrator::auth::registration::{register_user, OrgMode, RegistrationError};
use sqlx::PgPool;
use uuid::Uuid;

#[sqlx::test]
async fn create_mode_creates_org_and_owner_membership(pool: PgPool) {
    let user_id = register_user(
        &pool,
        "founder@example.com",
        "hunter2-hunter2",
        OrgMode::Create { name: "Acme".to_string() },
    )
    .await
    .unwrap();

    let (org_id, role): (Uuid, String) =
        sqlx::query_as("SELECT org_id, role FROM memberships WHERE user_id = $1")
            .bind(user_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(role, "owner");

    let org_name: String = sqlx::query_scalar("SELECT name FROM organizations WHERE id = $1")
        .bind(org_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(org_name, "Acme");
}

#[sqlx::test]
async fn join_mode_uses_invite_role_and_marks_invite_used(pool: PgPool) {
    let org_id: Uuid =
        sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Acme') RETURNING id")
            .fetch_one(&pool)
            .await
            .unwrap();
    let inviter: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id")
        .fetch_one(&pool)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO org_invites (code, org_id, role, invited_by, expires_at) VALUES ('JOINME', $1, 'admin', $2, now() + interval '7 days')",
    )
    .bind(org_id)
    .bind(inviter)
    .execute(&pool)
    .await
    .unwrap();

    let user_id = register_user(
        &pool,
        "newmember@example.com",
        "hunter2-hunter2",
        OrgMode::Join { invite_code: "JOINME".to_string() },
    )
    .await
    .unwrap();

    let role: String = sqlx::query_scalar("SELECT role FROM memberships WHERE org_id = $1 AND user_id = $2")
        .bind(org_id)
        .bind(user_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(role, "admin");

    let used_at: Option<chrono::DateTime<chrono::Utc>> =
        sqlx::query_scalar("SELECT used_at FROM org_invites WHERE code = 'JOINME'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(used_at.is_some());
}

#[sqlx::test]
async fn join_mode_rejects_a_reused_invite_code(pool: PgPool) {
    let org_id: Uuid =
        sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Acme') RETURNING id")
            .fetch_one(&pool)
            .await
            .unwrap();
    let inviter: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id")
        .fetch_one(&pool)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO org_invites (code, org_id, role, invited_by, expires_at, used_at) VALUES ('USEDUP', $1, 'member', $2, now() + interval '7 days', now())",
    )
    .bind(org_id)
    .bind(inviter)
    .execute(&pool)
    .await
    .unwrap();

    let result = register_user(
        &pool,
        "latecomer@example.com",
        "hunter2-hunter2",
        OrgMode::Join { invite_code: "USEDUP".to_string() },
    )
    .await;

    assert!(matches!(result, Err(RegistrationError::InvalidInvite)));

    let orphan_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM web_credentials WHERE email = 'latecomer@example.com')",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(
        !orphan_exists,
        "rejected registration must not leave an orphan web_credentials row"
    );
}

#[sqlx::test]
async fn rejects_duplicate_email(pool: PgPool) {
    register_user(
        &pool,
        "dup@example.com",
        "hunter2-hunter2",
        OrgMode::Create { name: "First Co".to_string() },
    )
    .await
    .unwrap();

    let result = register_user(
        &pool,
        "dup@example.com",
        "different-password",
        OrgMode::Create { name: "Second Co".to_string() },
    )
    .await;

    assert!(matches!(result, Err(RegistrationError::EmailTaken)));
}
