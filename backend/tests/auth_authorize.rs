use nomi_orchestrator::auth::authorize::{authorize_org_action, AuthorizeError};
use sqlx::PgPool;
use uuid::Uuid;

async fn make_membership(pool: &PgPool, role: &str) -> (Uuid, Uuid) {
    let org_id: Uuid = sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Acme') RETURNING id")
        .fetch_one(pool)
        .await
        .unwrap();
    let user_id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id")
        .fetch_one(pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO memberships (org_id, user_id, role) VALUES ($1, $2, $3)")
        .bind(org_id)
        .bind(user_id)
        .bind(role)
        .execute(pool)
        .await
        .unwrap();
    (org_id, user_id)
}

#[sqlx::test]
async fn allows_a_matching_role(pool: PgPool) {
    let (org_id, user_id) = make_membership(&pool, "owner").await;
    let result = authorize_org_action(&pool, user_id, org_id, &["owner", "admin"]).await;
    assert!(result.is_ok());
}

#[sqlx::test]
async fn rejects_a_role_not_in_the_allow_list(pool: PgPool) {
    let (org_id, user_id) = make_membership(&pool, "member").await;
    let result = authorize_org_action(&pool, user_id, org_id, &["owner", "admin"]).await;
    assert!(matches!(result, Err(AuthorizeError::Forbidden)));
}

#[sqlx::test]
async fn rejects_a_different_org(pool: PgPool) {
    let (_org_a, user_id) = make_membership(&pool, "owner").await;
    let org_b: Uuid = sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Other Org') RETURNING id")
        .fetch_one(&pool)
        .await
        .unwrap();

    let result = authorize_org_action(&pool, user_id, org_b, &["owner", "admin"]).await;
    assert!(matches!(result, Err(AuthorizeError::Forbidden)));
}

#[sqlx::test]
async fn rejects_a_membership_removed_after_it_was_granted(pool: PgPool) {
    let (org_id, user_id) = make_membership(&pool, "owner").await;

    // Simulate the membership being revoked after some earlier access token was already issued.
    sqlx::query("UPDATE memberships SET status = 'removed' WHERE org_id = $1 AND user_id = $2")
        .bind(org_id)
        .bind(user_id)
        .execute(&pool)
        .await
        .unwrap();

    let result = authorize_org_action(&pool, user_id, org_id, &["owner", "admin"]).await;
    assert!(matches!(result, Err(AuthorizeError::Forbidden)));
}
