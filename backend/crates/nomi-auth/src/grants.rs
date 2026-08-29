use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct PermissionGrant {
    pub id: Uuid,
    pub scope_type: String,
    pub org_id: Option<Uuid>,
    pub org_name: Option<String>,
    pub resource: String,
    pub actions: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

pub struct NewGrant<'a> {
    pub user_id: Uuid,
    pub scope_type: &'a str,
    pub org_id: Option<Uuid>,
    pub resource: &'a str,
    pub actions: &'a [String],
    pub granted_by: Uuid,
}

#[derive(Debug, thiserror::Error)]
pub enum GrantError {
    #[error("resource must start with a lowercase letter and contain only lowercase letters, digits, and underscores")]
    InvalidResource,
    #[error("actions must be a non-empty subset of view, manage")]
    InvalidActions,
    #[error("org_id is required when scope_type is org")]
    OrgRequiredForOrgScope,
    #[error("org_id must not be set when scope_type is admin")]
    OrgNotAllowedForAdminScope,
    #[error(transparent)]
    Db(#[from] sqlx::Error),
}

const ALLOWED_ACTIONS: [&str; 2] = ["view", "manage"];

fn validate_resource(resource: &str) -> Result<(), GrantError> {
    let mut chars = resource.chars();
    let first_ok = chars.next().map(|c| c.is_ascii_lowercase()).unwrap_or(false);
    let rest_ok = chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_');
    if first_ok && rest_ok {
        Ok(())
    } else {
        Err(GrantError::InvalidResource)
    }
}

fn validate_actions(actions: &[String]) -> Result<(), GrantError> {
    if actions.is_empty() || !actions.iter().all(|a| ALLOWED_ACTIONS.contains(&a.as_str())) {
        Err(GrantError::InvalidActions)
    } else {
        Ok(())
    }
}

pub async fn list_grants_for_user(pool: &PgPool, user_id: Uuid) -> Result<Vec<PermissionGrant>, sqlx::Error> {
    sqlx::query_as(
        "SELECT up.id, up.scope_type, up.org_id, o.name AS org_name, up.resource, up.actions, up.created_at \
         FROM user_permissions up \
         LEFT JOIN organizations o ON o.id = up.org_id \
         WHERE up.user_id = $1 \
         ORDER BY up.created_at ASC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}

pub async fn grant_permission(pool: &PgPool, input: NewGrant<'_>) -> Result<Vec<PermissionGrant>, GrantError> {
    validate_resource(input.resource)?;
    validate_actions(input.actions)?;
    match (input.scope_type, input.org_id) {
        ("admin", Some(_)) => return Err(GrantError::OrgNotAllowedForAdminScope),
        ("org", None) => return Err(GrantError::OrgRequiredForOrgScope),
        _ => {}
    }

    let mut tx = pool.begin().await?;

    let existing: Option<(Uuid, Vec<String>)> = sqlx::query_as(
        "SELECT id, actions FROM user_permissions \
         WHERE user_id = $1 AND scope_type = $2 AND org_id IS NOT DISTINCT FROM $3 AND resource = $4 \
         FOR UPDATE",
    )
    .bind(input.user_id)
    .bind(input.scope_type)
    .bind(input.org_id)
    .bind(input.resource)
    .fetch_optional(&mut *tx)
    .await?;

    match existing {
        Some((id, mut current_actions)) => {
            for action in input.actions {
                if !current_actions.contains(action) {
                    current_actions.push(action.clone());
                }
            }
            sqlx::query("UPDATE user_permissions SET actions = $1 WHERE id = $2")
                .bind(&current_actions)
                .bind(id)
                .execute(&mut *tx)
                .await?;
        }
        None => {
            sqlx::query(
                "INSERT INTO user_permissions (user_id, scope_type, org_id, resource, actions, granted_by) \
                 VALUES ($1, $2, $3, $4, $5, $6)",
            )
            .bind(input.user_id)
            .bind(input.scope_type)
            .bind(input.org_id)
            .bind(input.resource)
            .bind(input.actions)
            .bind(input.granted_by)
            .execute(&mut *tx)
            .await?;
        }
    }

    tx.commit().await?;
    list_grants_for_user(pool, input.user_id).await.map_err(GrantError::Db)
}

pub async fn revoke_permission(pool: &PgPool, user_id: Uuid, grant_id: Uuid) -> Result<Vec<PermissionGrant>, sqlx::Error> {
    sqlx::query("DELETE FROM user_permissions WHERE id = $1 AND user_id = $2")
        .bind(grant_id)
        .bind(user_id)
        .execute(pool)
        .await?;
    list_grants_for_user(pool, user_id).await
}
