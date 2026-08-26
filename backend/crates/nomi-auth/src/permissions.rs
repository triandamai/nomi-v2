use sqlx::PgPool;
use uuid::Uuid;

use super::claims::permission_string;

pub async fn compute_permissions(pool: &PgPool, user_id: Uuid) -> Result<Vec<String>, sqlx::Error> {
    let mut permissions = Vec::new();

    let is_platform_admin: bool =
        sqlx::query_scalar("SELECT is_platform_admin FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_one(pool)
            .await?;
    if is_platform_admin {
        permissions.push(permission_string("admin", "user", &["view", "manage"]));
        permissions.push(permission_string("admin", "system_config", &["view", "manage"]));
    }

    let memberships: Vec<(Uuid, String)> = sqlx::query_as(
        "SELECT org_id, role FROM memberships WHERE user_id = $1 AND status = 'active'",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    for (org_id, role) in memberships {
        let actions: &[&str] = if role == "owner" || role == "admin" {
            &["view", "manage"]
        } else {
            &["view"]
        };
        let org_scope = org_id.to_string();
        permissions.push(permission_string(&org_scope, "member", actions));
        permissions.push(permission_string(&org_scope, "conversation", actions));
    }

    Ok(permissions)
}
