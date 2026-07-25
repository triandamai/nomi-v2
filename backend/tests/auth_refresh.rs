use nomi_orchestrator::auth::{
    login::login,
    refresh_token::{issue_refresh_token, refresh_access_token, revoke_refresh_token, RefreshError},
    claims::Claims,
    registration::{register_user, OrgMode},
};
use sqlx::PgPool;

const SECRET: &str = "test-secret-do-not-use-in-prod";

#[sqlx::test]
async fn issue_and_refresh_roundtrip(pool: PgPool) {
    let user_id = register_user(
        &pool,
        "carol@example.com",
        "correct-password",
        OrgMode::Create { name: "Acme".to_string() },
    )
    .await
    .unwrap();

    let raw_refresh = issue_refresh_token(&pool, user_id).await.unwrap();
    let new_access_token = refresh_access_token(&pool, &raw_refresh, SECRET).await.unwrap();

    let claims = Claims::decode(&new_access_token, SECRET).unwrap();
    assert_eq!(claims.sub, user_id);
}

#[sqlx::test]
async fn refresh_rejects_after_revocation(pool: PgPool) {
    let user_id = register_user(
        &pool,
        "dave@example.com",
        "correct-password",
        OrgMode::Create { name: "Acme".to_string() },
    )
    .await
    .unwrap();

    let raw_refresh = issue_refresh_token(&pool, user_id).await.unwrap();
    revoke_refresh_token(&pool, &raw_refresh).await.unwrap();

    let result = refresh_access_token(&pool, &raw_refresh, SECRET).await;
    assert!(matches!(result, Err(RefreshError::Invalid)));
}

#[sqlx::test]
async fn refresh_rejects_after_expiry(pool: PgPool) {
    let user_id = register_user(
        &pool,
        "erin@example.com",
        "correct-password",
        OrgMode::Create { name: "Acme".to_string() },
    )
    .await
    .unwrap();

    let raw_refresh = "manually-inserted-expired-token";
    let token_hash = nomi_orchestrator::auth::refresh_token::hash_token(raw_refresh);
    sqlx::query(
        "INSERT INTO refresh_tokens (user_id, token_hash, expires_at) VALUES ($1, $2, now() - interval '1 day')",
    )
    .bind(user_id)
    .bind(&token_hash)
    .execute(&pool)
    .await
    .unwrap();

    let result = refresh_access_token(&pool, raw_refresh, SECRET).await;
    assert!(matches!(result, Err(RefreshError::Invalid)));
}

#[sqlx::test]
async fn refresh_recomputes_permissions_after_membership_change(pool: PgPool) {
    let user_id = register_user(
        &pool,
        "frank@example.com",
        "correct-password",
        OrgMode::Create { name: "First Org".to_string() },
    )
    .await
    .unwrap();

    let raw_refresh = issue_refresh_token(&pool, user_id).await.unwrap();

    let new_org_id: uuid::Uuid =
        sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Second Org') RETURNING id")
            .fetch_one(&pool)
            .await
            .unwrap();
    sqlx::query("INSERT INTO memberships (org_id, user_id, role) VALUES ($1, $2, 'member')")
        .bind(new_org_id)
        .bind(user_id)
        .execute(&pool)
        .await
        .unwrap();

    let new_access_token = refresh_access_token(&pool, &raw_refresh, SECRET).await.unwrap();
    let claims = Claims::decode(&new_access_token, SECRET).unwrap();
    assert!(claims
        .permissions
        .contains(&format!("nomi:{new_org_id}:member:[view]")));
}
