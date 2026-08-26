use sqlx::PgPool;
use uuid::Uuid;

#[sqlx::test(migrations = "../../migrations")]
async fn membership_is_unique_per_org_and_user(pool: PgPool) {
    let org_id: Uuid =
        sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Acme') RETURNING id")
            .fetch_one(&pool)
            .await
            .unwrap();
    let user_id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id")
        .fetch_one(&pool)
        .await
        .unwrap();

    sqlx::query("INSERT INTO memberships (org_id, user_id, role) VALUES ($1, $2, 'owner')")
        .bind(org_id)
        .bind(user_id)
        .execute(&pool)
        .await
        .unwrap();

    let err = sqlx::query("INSERT INTO memberships (org_id, user_id, role) VALUES ($1, $2, 'member')")
        .bind(org_id)
        .bind(user_id)
        .execute(&pool)
        .await
        .unwrap_err();

    assert_eq!(
        err.as_database_error().unwrap().constraint(),
        Some("memberships_org_id_user_id_key")
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn personal_org_defaults_to_false_unless_set(pool: PgPool) {
    let org_id: Uuid =
        sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Real Team') RETURNING id")
            .fetch_one(&pool)
            .await
            .unwrap();
    let is_personal: bool =
        sqlx::query_scalar("SELECT is_personal FROM organizations WHERE id = $1")
            .bind(org_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(!is_personal);

    let personal_org_id: Uuid = sqlx::query_scalar(
        "INSERT INTO organizations (name, is_personal) VALUES ('Dinda (personal)', true) RETURNING id",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    let is_personal: bool =
        sqlx::query_scalar("SELECT is_personal FROM organizations WHERE id = $1")
            .bind(personal_org_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(is_personal);
}

#[sqlx::test(migrations = "../../migrations")]
async fn org_invite_code_is_unique_and_single_use(pool: PgPool) {
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
        "INSERT INTO org_invites (code, org_id, role, invited_by, expires_at) VALUES ('XYZ789', $1, 'member', $2, now() + interval '7 days')",
    )
    .bind(org_id)
    .bind(inviter)
    .execute(&pool)
    .await
    .unwrap();

    let dup_err = sqlx::query(
        "INSERT INTO org_invites (code, org_id, role, invited_by, expires_at) VALUES ('XYZ789', $1, 'member', $2, now() + interval '7 days')",
    )
    .bind(org_id)
    .bind(inviter)
    .execute(&pool)
    .await
    .unwrap_err();

    assert_eq!(
        dup_err.as_database_error().unwrap().constraint(),
        Some("org_invites_pkey")
    );
}
