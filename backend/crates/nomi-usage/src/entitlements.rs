//! What someone's plan includes beyond its token allowance: how many projects and organizations
//! they may own, how many members their organizations may hold, how much file storage, whether
//! they may pick their own address, which sandbox size and which of Nomi's models they get.
//! A limit left empty on the plan is unlimited, so nothing is refused until an admin sets one.

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use serde::Serialize;
use sqlx::{PgConnection, PgPool};
use uuid::Uuid;

use crate::plans;

/// Bytes in a megabyte, for comparing stored bytes with a plan's storage limit.
pub const BYTES_PER_MB: i64 = 1024 * 1024;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Entitlements {
    pub max_projects: Option<i32>,
    pub max_orgs: Option<i32>,
    pub max_org_members: Option<i32>,
    pub max_custom_crew: Option<i32>,
    pub max_storage_mb: Option<i64>,
    pub custom_domain: bool,
    pub sandbox_size: Option<String>,
    pub all_models: bool,
    pub model_ids: Vec<Uuid>,
    pub plan_name: String,
}

impl Entitlements {
    pub fn from_plan(plan: &plans::PlanRow) -> Entitlements {
        Entitlements {
            max_projects: plan.max_projects,
            max_orgs: plan.max_orgs,
            max_org_members: plan.max_org_members,
            max_custom_crew: plan.max_custom_crew,
            max_storage_mb: plan.max_storage_mb,
            custom_domain: plan.custom_domain,
            sandbox_size: plan.sandbox_size.clone(),
            all_models: plan.all_models,
            model_ids: plan.model_ids.clone(),
            plan_name: plan.name.clone(),
        }
    }
}

/// The entitlements of the plan `user_id` is on now.
pub async fn for_user(pool: &PgPool, user_id: Uuid) -> Result<Entitlements, sqlx::Error> {
    Ok(Entitlements::from_plan(&plans::subscription_for(pool, user_id).await?.plan))
}

/// The entitlements of whoever owns `project_id`, or `None` when there's no such project.
pub async fn for_project_owner(pool: &PgPool, project_id: Uuid) -> Result<Option<Entitlements>, sqlx::Error> {
    let owner: Option<Uuid> = sqlx::query_scalar("SELECT user_id FROM projects WHERE id = $1").bind(project_id).fetch_optional(pool).await?;
    match owner {
        Some(owner) => Ok(Some(for_user(pool, owner).await?)),
        None => Ok(None),
    }
}

/// The limit that stopped something.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LimitHit {
    Projects { limit: i32 },
    Orgs { limit: i32 },
    OrgMembers { limit: i32 },
    Storage { limit_mb: i64 },
    CustomDomain,
    Model,
    CustomCrew { limit: i32 },
}

fn plural(n: i32, one: &str, many: &str) -> String {
    format!("{n} {}", if n == 1 { one } else { many })
}

impl LimitHit {
    /// A short sentence for the person: which limit, and that upgrading lifts it.
    pub fn message(&self) -> String {
        match self {
            LimitHit::Projects { limit } => {
                format!("Your plan allows {}. Upgrade your plan to create more.", plural(*limit, "project", "projects"))
            }
            LimitHit::Orgs { limit } => {
                format!("Your plan allows {}. Upgrade your plan to create more.", plural(*limit, "organization", "organizations"))
            }
            LimitHit::OrgMembers { limit } => {
                format!("Your plan allows {} in an organization. Upgrade your plan to add more people.", plural(*limit, "member", "members"))
            }
            LimitHit::Storage { limit_mb } => {
                format!("Your plan includes {limit_mb} MB of project storage and it's full. Upgrade your plan to store more.")
            }
            LimitHit::CustomDomain => "Your plan doesn't include choosing your own address. Upgrade your plan to pick one.".to_string(),
            LimitHit::Model => "That model isn't included in your plan. Upgrade your plan to use it.".to_string(),
            LimitHit::CustomCrew { limit } => {
                format!("Your plan allows {} of your own. Upgrade your plan to add more.", plural(*limit, "custom crew member", "custom crew members"))
            }
        }
    }
}

impl LimitHit {
    /// The same sentence as `&'static str`, for error types that carry one. Each distinct sentence
    /// is kept once; admins choose the limits, so there are only ever a handful.
    pub fn static_message(&self) -> &'static str {
        static KNOWN: OnceLock<Mutex<HashMap<String, &'static str>>> = OnceLock::new();
        let message = self.message();
        let mut known = KNOWN.get_or_init(Default::default).lock().unwrap_or_else(|e| e.into_inner());
        known.entry(message).or_insert_with_key(|m| Box::leak(m.clone().into_boxed_str()))
    }
}

/// Whether one more fits when `used` are already there: always without a limit.
pub fn within(limit: Option<i32>, used: i64) -> bool {
    limit.is_none_or(|limit| used < i64::from(limit))
}

/// Whether `used_bytes` plus `adding_bytes` still fits in the storage limit (megabytes).
pub fn storage_fits(limit_mb: Option<i64>, used_bytes: i64, adding_bytes: i64) -> bool {
    limit_mb.is_none_or(|mb| used_bytes.saturating_add(adding_bytes) <= mb.saturating_mul(BYTES_PER_MB))
}

pub type Check = Result<Result<(), LimitHit>, sqlx::Error>;

fn verdict(ok: bool, hit: LimitHit) -> Result<(), LimitHit> {
    if ok {
        Ok(())
    } else {
        Err(hit)
    }
}

/// How many projects `user_id` owns.
pub async fn project_count(pool: &PgPool, user_id: Uuid) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar("SELECT count(*) FROM projects WHERE user_id = $1").bind(user_id).fetch_one(pool).await
}

/// How many organizations (other than their personal one) `user_id` owns.
pub async fn org_count(pool: &PgPool, user_id: Uuid) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT count(*) FROM memberships m JOIN organizations o ON o.id = m.org_id \
         WHERE m.user_id = $1 AND m.role = 'owner' AND m.status = 'active' AND NOT o.is_personal",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await
}

/// Bytes of files across the projects `user_id` owns.
pub async fn storage_bytes(pool: &PgPool, user_id: Uuid) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT COALESCE(sum(f.size_bytes), 0)::bigint FROM project_files f JOIN projects p ON p.id = f.project_id WHERE p.user_id = $1",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await
}

pub async fn check_new_project(pool: &PgPool, user_id: Uuid) -> Check {
    let Some(limit) = for_user(pool, user_id).await?.max_projects else { return Ok(Ok(())) };
    Ok(verdict(within(Some(limit), project_count(pool, user_id).await?), LimitHit::Projects { limit }))
}

pub async fn check_new_org(pool: &PgPool, user_id: Uuid) -> Check {
    let Some(limit) = for_user(pool, user_id).await?.max_orgs else { return Ok(Ok(())) };
    Ok(verdict(within(Some(limit), org_count(pool, user_id).await?), LimitHit::Orgs { limit }))
}

/// One more member (or invite) in `org_id`, on the plan of the organization's owner.
pub async fn check_new_org_member(pool: &PgPool, org_id: Uuid) -> Check {
    org_member_check(pool, org_id, None).await
}

/// Same as [`check_new_org_member`] for someone accepting the invite `code`, which already holds
/// a seat and so doesn't count against them.
pub async fn check_accept_invite(pool: &PgPool, org_id: Uuid, code: &str) -> Check {
    org_member_check(pool, org_id, Some(code)).await
}

async fn org_member_check(pool: &PgPool, org_id: Uuid, except_invite: Option<&str>) -> Check {
    let owner: Option<Uuid> = sqlx::query_scalar(
        "SELECT user_id FROM memberships WHERE org_id = $1 AND role = 'owner' AND status = 'active' ORDER BY created_at LIMIT 1",
    )
    .bind(org_id)
    .fetch_optional(pool)
    .await?;
    let Some(owner) = owner else { return Ok(Ok(())) };
    let Some(limit) = for_user(pool, owner).await?.max_org_members else { return Ok(Ok(())) };
    let used: i64 = sqlx::query_scalar(
        "SELECT (SELECT count(*) FROM memberships WHERE org_id = $1 AND status IN ('active', 'invited')) \
              + (SELECT count(*) FROM org_invites WHERE org_id = $1 AND used_at IS NULL AND expires_at > now() AND code IS DISTINCT FROM $2)",
    )
    .bind(org_id)
    .bind(except_invite)
    .fetch_one(pool)
    .await?;
    Ok(verdict(within(Some(limit), used), LimitHit::OrgMembers { limit }))
}

/// Whether `user_id` may store `adding_bytes` more (the change in size; zero or less always fits).
pub async fn check_storage(pool: &PgPool, user_id: Uuid, adding_bytes: i64) -> Check {
    if adding_bytes <= 0 {
        return Ok(Ok(()));
    }
    let Some(limit_mb) = for_user(pool, user_id).await?.max_storage_mb else { return Ok(Ok(())) };
    let used = storage_bytes(pool, user_id).await?;
    Ok(verdict(storage_fits(Some(limit_mb), used, adding_bytes), LimitHit::Storage { limit_mb }))
}

/// Storage check for a write into `project_id`, charged to the project's owner.
pub async fn check_project_storage(pool: &PgPool, project_id: Uuid, adding_bytes: i64) -> Check {
    check_project_storage_in(&mut *pool.acquire().await?, project_id, adding_bytes).await
}

/// [`check_project_storage`] on a connection already in hand.
pub async fn check_project_storage_in(conn: &mut PgConnection, project_id: Uuid, adding_bytes: i64) -> Check {
    if adding_bytes <= 0 {
        return Ok(Ok(()));
    }
    let owner: Option<Uuid> = sqlx::query_scalar("SELECT user_id FROM projects WHERE id = $1").bind(project_id).fetch_optional(&mut *conn).await?;
    let Some(owner) = owner else { return Ok(Ok(())) };
    let Some(limit_mb) = plans::subscription_in(&mut *conn, owner).await?.plan.max_storage_mb else { return Ok(Ok(())) };
    let used: i64 = sqlx::query_scalar(
        "SELECT COALESCE(sum(f.size_bytes), 0)::bigint FROM project_files f JOIN projects p ON p.id = f.project_id WHERE p.user_id = $1",
    )
    .bind(owner)
    .fetch_one(&mut *conn)
    .await?;
    Ok(verdict(storage_fits(Some(limit_mb), used, adding_bytes), LimitHit::Storage { limit_mb }))
}

/// Storage check for writing a file of `new_len` bytes at `path` in `project_id`: only what it
/// adds over the file it replaces counts, so a write that shrinks or keeps the size always fits.
pub async fn check_file_write(conn: &mut PgConnection, project_id: Uuid, path: &str, new_len: i64) -> Check {
    let existing: Option<i32> =
        sqlx::query_scalar("SELECT size_bytes FROM project_files WHERE project_id = $1 AND path = $2").bind(project_id).bind(path).fetch_optional(&mut *conn).await?;
    check_project_storage_in(conn, project_id, new_len - i64::from(existing.unwrap_or(0))).await
}

pub async fn check_custom_domain(pool: &PgPool, user_id: Uuid) -> Check {
    Ok(verdict(for_user(pool, user_id).await?.custom_domain, LimitHit::CustomDomain))
}

/// Whether one of Nomi's admin-listed models is in the plan. Models run with the person's own
/// key never come here.
pub fn model_allowed(entitlements: &Entitlements, model_id: Uuid) -> bool {
    entitlements.all_models || entitlements.model_ids.contains(&model_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    const LIMIT: i32 = 3;
    const FEW: i64 = 2;

    #[test]
    fn no_limit_is_always_within() {
        assert!(within(None, i64::MAX));
    }

    #[test]
    fn refuses_at_the_limit_not_before() {
        assert!(within(Some(LIMIT), FEW));
        assert!(!within(Some(LIMIT), i64::from(LIMIT)));
        assert!(!within(Some(0), 0));
    }

    #[test]
    fn storage_fits_up_to_the_limit() {
        assert!(storage_fits(None, i64::MAX / 2, i64::MAX / 2));
        assert!(storage_fits(Some(1), BYTES_PER_MB - 1, 1));
        assert!(!storage_fits(Some(1), BYTES_PER_MB, 1));
    }

    #[test]
    fn messages_name_the_limit_and_the_upgrade() {
        let m = LimitHit::Projects { limit: LIMIT }.message();
        assert_eq!(m, "Your plan allows 3 projects. Upgrade your plan to create more.");
        assert_eq!(LimitHit::Projects { limit: LIMIT }.static_message(), m);
        assert!(LimitHit::Projects { limit: 1 }.message().contains("1 project."));
        for hit in [
            LimitHit::Orgs { limit: LIMIT },
            LimitHit::OrgMembers { limit: LIMIT },
            LimitHit::Storage { limit_mb: 10 },
            LimitHit::CustomDomain,
            LimitHit::Model,
            LimitHit::CustomCrew { limit: LIMIT },
        ] {
            assert!(hit.message().contains("Upgrade your plan"), "{hit:?}");
        }
    }
}
