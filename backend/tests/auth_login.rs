use nomi_orchestrator::auth::{claims::Claims, login::{login, LoginError}, registration::{register_user, OrgMode}};
use sqlx::PgPool;

const SECRET: &str = "test-secret-do-not-use-in-prod";

#[sqlx::test]
async fn login_succeeds_with_correct_credentials(pool: PgPool) {
    let user_id = register_user(
        &pool,
        "alice@example.com",
        "correct-password",
        OrgMode::Create { name: "Acme".to_string() },
    )
    .await
    .unwrap();

    let (token, returned_user_id) = login(&pool, "alice@example.com", "correct-password", SECRET)
        .await
        .unwrap();
    assert_eq!(returned_user_id, user_id);

    let claims = Claims::decode(&token, SECRET).unwrap();
    assert_eq!(claims.sub, user_id);
    assert!(claims
        .permissions
        .iter()
        .any(|p| p.contains("member") && p.contains("manage")));
}

#[sqlx::test]
async fn login_rejects_wrong_password(pool: PgPool) {
    register_user(
        &pool,
        "bob@example.com",
        "correct-password",
        OrgMode::Create { name: "Acme".to_string() },
    )
    .await
    .unwrap();

    let result = login(&pool, "bob@example.com", "wrong-password", SECRET).await;
    assert!(matches!(result, Err(LoginError::InvalidCredentials)));
}

#[sqlx::test]
async fn login_rejects_unknown_email(pool: PgPool) {
    let result = login(&pool, "nobody@example.com", "whatever", SECRET).await;
    assert!(matches!(result, Err(LoginError::InvalidCredentials)));
}
