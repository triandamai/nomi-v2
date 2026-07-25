use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, thiserror::Error)]
pub enum AuthorizeError {
    #[error("not authorized for this organization")]
    Forbidden,
    #[error(transparent)]
    Db(#[from] sqlx::Error),
}

pub async fn authorize_org_action(
    pool: &PgPool,
    user_id: Uuid,
    org_id: Uuid,
    allowed_roles: &[&str],
) -> Result<(), AuthorizeError> {
    let role: Option<String> = sqlx::query_scalar(
        "SELECT role FROM memberships WHERE org_id = $1 AND user_id = $2 AND status = 'active'",
    )
    .bind(org_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    match role {
        Some(r) if allowed_roles.contains(&r.as_str()) => Ok(()),
        _ => Err(AuthorizeError::Forbidden),
    }
}
