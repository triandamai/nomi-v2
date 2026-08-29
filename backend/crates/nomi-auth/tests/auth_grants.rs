use nomi_auth::grants::{grant_permission, list_grants_for_user, revoke_permission, GrantError, NewGrant};
use nomi_auth::permissions::compute_permissions;
use sqlx::PgPool;
use uuid::Uuid;

async fn make_user(pool: &PgPool) -> Uuid {
    sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(pool).await.unwrap()
}

#[sqlx::test(migrations = "../../migrations")]
async fn granting_an_admin_scope_permission_appears_in_compute_permissions(pool: PgPool) {
    let user_id = make_user(&pool).await;
    let granter = make_user(&pool).await;

    grant_permission(
        &pool,
        NewGrant {
            user_id,
            scope_type: "admin",
            org_id: None,
            resource: "user",
            actions: &["view".to_string()],
            granted_by: granter,
        },
    )
    .await
    .unwrap();

    let permissions = compute_permissions(&pool, user_id).await.unwrap();
    assert!(permissions.contains(&"nomi:admin:user:[view]".to_string()));
}

#[sqlx::test(migrations = "../../migrations")]
async fn granting_the_same_resource_twice_merges_actions_into_one_string(pool: PgPool) {
    let user_id = make_user(&pool).await;
    let granter = make_user(&pool).await;

    grant_permission(
        &pool,
        NewGrant { user_id, scope_type: "admin", org_id: None, resource: "user", actions: &["view".to_string()], granted_by: granter },
    )
    .await
    .unwrap();
    let grants = grant_permission(
        &pool,
        NewGrant { user_id, scope_type: "admin", org_id: None, resource: "user", actions: &["manage".to_string()], granted_by: granter },
    )
    .await
    .unwrap();

    // One row, not two — the second grant merged into the first.
    assert_eq!(grants.len(), 1);
    assert_eq!(grants[0].actions.len(), 2);

    let permissions = compute_permissions(&pool, user_id).await.unwrap();
    assert!(permissions.contains(&"nomi:admin:user:[view,manage]".to_string()));
}

#[sqlx::test(migrations = "../../migrations")]
async fn org_scope_grant_merges_with_role_derived_permission_on_the_same_resource(pool: PgPool) {
    let user_id = make_user(&pool).await;
    let granter = make_user(&pool).await;
    let org_id: Uuid = sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Acme') RETURNING id")
        .fetch_one(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO memberships (org_id, user_id, role) VALUES ($1, $2, 'member')")
        .bind(org_id)
        .bind(user_id)
        .execute(&pool)
        .await
        .unwrap();

    // 'member' role alone yields view-only on `member`. An explicit grant adds manage.
    grant_permission(
        &pool,
        NewGrant {
            user_id,
            scope_type: "org",
            org_id: Some(org_id),
            resource: "member",
            actions: &["manage".to_string()],
            granted_by: granter,
        },
    )
    .await
    .unwrap();

    let permissions = compute_permissions(&pool, user_id).await.unwrap();
    assert!(permissions.contains(&format!("nomi:{org_id}:member:[view,manage]")));
}

#[sqlx::test(migrations = "../../migrations")]
async fn revoking_a_grant_removes_it_from_compute_permissions(pool: PgPool) {
    let user_id = make_user(&pool).await;
    let granter = make_user(&pool).await;

    let grants = grant_permission(
        &pool,
        NewGrant { user_id, scope_type: "admin", org_id: None, resource: "billing", actions: &["view".to_string()], granted_by: granter },
    )
    .await
    .unwrap();
    let grant_id = grants[0].id;

    let remaining = revoke_permission(&pool, user_id, grant_id).await.unwrap();
    assert!(remaining.is_empty());

    let permissions = compute_permissions(&pool, user_id).await.unwrap();
    assert!(!permissions.iter().any(|p| p.contains("billing")));
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_user_with_zero_grants_and_no_role_gets_an_empty_permissions_array(pool: PgPool) {
    let user_id = make_user(&pool).await;
    let permissions = compute_permissions(&pool, user_id).await.unwrap();
    assert!(permissions.is_empty());
}

#[sqlx::test(migrations = "../../migrations")]
async fn invalid_resource_format_is_rejected(pool: PgPool) {
    let user_id = make_user(&pool).await;
    let granter = make_user(&pool).await;
    let result = grant_permission(
        &pool,
        NewGrant { user_id, scope_type: "admin", org_id: None, resource: "Not-Valid!", actions: &["view".to_string()], granted_by: granter },
    )
    .await;
    assert!(matches!(result, Err(GrantError::InvalidResource)));
}

#[sqlx::test(migrations = "../../migrations")]
async fn empty_actions_is_rejected(pool: PgPool) {
    let user_id = make_user(&pool).await;
    let granter = make_user(&pool).await;
    let result = grant_permission(
        &pool,
        NewGrant { user_id, scope_type: "admin", org_id: None, resource: "user", actions: &[], granted_by: granter },
    )
    .await;
    assert!(matches!(result, Err(GrantError::InvalidActions)));
}

#[sqlx::test(migrations = "../../migrations")]
async fn org_scope_without_org_id_is_rejected(pool: PgPool) {
    let user_id = make_user(&pool).await;
    let granter = make_user(&pool).await;
    let result = grant_permission(
        &pool,
        NewGrant { user_id, scope_type: "org", org_id: None, resource: "member", actions: &["view".to_string()], granted_by: granter },
    )
    .await;
    assert!(matches!(result, Err(GrantError::OrgRequiredForOrgScope)));
}

#[sqlx::test(migrations = "../../migrations")]
async fn list_grants_for_user_only_returns_that_users_rows(pool: PgPool) {
    let user_a = make_user(&pool).await;
    let user_b = make_user(&pool).await;
    let granter = make_user(&pool).await;
    grant_permission(
        &pool,
        NewGrant { user_id: user_a, scope_type: "admin", org_id: None, resource: "user", actions: &["view".to_string()], granted_by: granter },
    )
    .await
    .unwrap();

    let b_grants = list_grants_for_user(&pool, user_b).await.unwrap();
    assert!(b_grants.is_empty());
}
