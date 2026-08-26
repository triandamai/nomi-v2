use nomi_auth::permissions::compute_permissions;
use sqlx::PgPool;
use uuid::Uuid;

async fn make_user(pool: &PgPool) -> Uuid {
    sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id")
        .fetch_one(pool)
        .await
        .unwrap()
}

async fn make_org(pool: &PgPool, name: &str) -> Uuid {
    sqlx::query_scalar("INSERT INTO organizations (name) VALUES ($1) RETURNING id")
        .bind(name)
        .fetch_one(pool)
        .await
        .unwrap()
}

async fn add_membership(pool: &PgPool, org_id: Uuid, user_id: Uuid, role: &str) {
    sqlx::query("INSERT INTO memberships (org_id, user_id, role) VALUES ($1, $2, $3)")
        .bind(org_id)
        .bind(user_id)
        .bind(role)
        .execute(pool)
        .await
        .unwrap();
}

#[sqlx::test(migrations = "../../migrations")]
async fn platform_admin_gets_the_admin_permission_string(pool: PgPool) {
    let user_id = make_user(&pool).await;
    sqlx::query("UPDATE users SET is_platform_admin = true WHERE id = $1")
        .bind(user_id)
        .execute(&pool)
        .await
        .unwrap();

    let permissions = compute_permissions(&pool, user_id).await.unwrap();
    assert!(permissions.contains(&"nomi:admin:user:[view,manage]".to_string()));
}

#[sqlx::test(migrations = "../../migrations")]
async fn platform_admins_get_the_system_config_permission(pool: PgPool) {
    let user_id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id")
        .fetch_one(&pool)
        .await
        .unwrap();
    sqlx::query("UPDATE users SET is_platform_admin = true WHERE id = $1")
        .bind(user_id)
        .execute(&pool)
        .await
        .unwrap();

    let permissions = compute_permissions(&pool, user_id).await.unwrap();
    assert!(permissions.contains(&"nomi:admin:system_config:[view,manage]".to_string()));
}

#[sqlx::test(migrations = "../../migrations")]
async fn non_platform_admins_do_not_get_the_system_config_permission(pool: PgPool) {
    let user_id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id")
        .fetch_one(&pool)
        .await
        .unwrap();

    let permissions = compute_permissions(&pool, user_id).await.unwrap();
    assert!(!permissions.iter().any(|p| p.contains("system_config")));
}

#[sqlx::test(migrations = "../../migrations")]
async fn owner_gets_manage_permissions_on_member_and_conversation(pool: PgPool) {
    let user_id = make_user(&pool).await;
    let org_id = make_org(&pool, "Acme").await;
    add_membership(&pool, org_id, user_id, "owner").await;

    let permissions = compute_permissions(&pool, user_id).await.unwrap();
    assert!(permissions.contains(&format!("nomi:{org_id}:member:[view,manage]")));
    assert!(permissions.contains(&format!("nomi:{org_id}:conversation:[view,manage]")));
}

#[sqlx::test(migrations = "../../migrations")]
async fn member_role_gets_view_only_permissions(pool: PgPool) {
    let user_id = make_user(&pool).await;
    let org_id = make_org(&pool, "Acme").await;
    add_membership(&pool, org_id, user_id, "member").await;

    let permissions = compute_permissions(&pool, user_id).await.unwrap();
    assert!(permissions.contains(&format!("nomi:{org_id}:member:[view]")));
    assert!(permissions.contains(&format!("nomi:{org_id}:conversation:[view]")));
    assert!(!permissions.contains(&format!("nomi:{org_id}:member:[view,manage]")));
}

#[sqlx::test(migrations = "../../migrations")]
async fn permissions_span_every_org_the_user_belongs_to(pool: PgPool) {
    let user_id = make_user(&pool).await;
    let org_a = make_org(&pool, "Org A").await;
    let org_b = make_org(&pool, "Org B").await;
    add_membership(&pool, org_a, user_id, "owner").await;
    add_membership(&pool, org_b, user_id, "member").await;

    let permissions = compute_permissions(&pool, user_id).await.unwrap();
    assert!(permissions.contains(&format!("nomi:{org_a}:member:[view,manage]")));
    assert!(permissions.contains(&format!("nomi:{org_b}:member:[view]")));
}

#[sqlx::test(migrations = "../../migrations")]
async fn removed_membership_is_excluded(pool: PgPool) {
    let user_id = make_user(&pool).await;
    let org_id = make_org(&pool, "Acme").await;
    add_membership(&pool, org_id, user_id, "member").await;
    sqlx::query("UPDATE memberships SET status = 'removed' WHERE org_id = $1 AND user_id = $2")
        .bind(org_id)
        .bind(user_id)
        .execute(&pool)
        .await
        .unwrap();

    let permissions = compute_permissions(&pool, user_id).await.unwrap();
    assert!(!permissions.contains(&format!("nomi:{org_id}:member:[view]")));
}
