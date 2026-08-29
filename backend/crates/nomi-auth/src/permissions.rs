use std::collections::{HashMap, HashSet};

use sqlx::PgPool;
use uuid::Uuid;

use super::claims::permission_string;
use super::grants::list_grants_for_user;

const ACTION_ORDER: [&str; 2] = ["view", "manage"];

pub async fn compute_permissions(pool: &PgPool, user_id: Uuid) -> Result<Vec<String>, sqlx::Error> {
    let mut merged: HashMap<(String, String), HashSet<String>> = HashMap::new();

    let is_platform_admin: bool =
        sqlx::query_scalar("SELECT is_platform_admin FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_one(pool)
            .await?;
    if is_platform_admin {
        merged
            .entry(("admin".to_string(), "user".to_string()))
            .or_default()
            .extend(["view".to_string(), "manage".to_string()]);
        merged
            .entry(("admin".to_string(), "system_config".to_string()))
            .or_default()
            .extend(["view".to_string(), "manage".to_string()]);
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
        let scope = org_id.to_string();
        merged
            .entry((scope.clone(), "member".to_string()))
            .or_default()
            .extend(actions.iter().map(|s| s.to_string()));
        merged
            .entry((scope, "conversation".to_string()))
            .or_default()
            .extend(actions.iter().map(|s| s.to_string()));
    }

    for grant in list_grants_for_user(pool, user_id).await? {
        let scope = match grant.scope_type.as_str() {
            "admin" => "admin".to_string(),
            _ => match grant.org_id {
                Some(org_id) => org_id.to_string(),
                // An org-scope grant with no org_id can't exist (DB CHECK enforces it) —
                // skip defensively rather than let a malformed row panic permission computation.
                None => continue,
            },
        };
        merged.entry((scope, grant.resource)).or_default().extend(grant.actions);
    }

    let mut permissions: Vec<String> = merged
        .into_iter()
        .map(|((scope, resource), actions)| {
            let ordered: Vec<&str> = ACTION_ORDER.iter().copied().filter(|a| actions.contains(*a)).collect();
            permission_string(&scope, &resource, &ordered)
        })
        .collect();
    permissions.sort();
    Ok(permissions)
}
