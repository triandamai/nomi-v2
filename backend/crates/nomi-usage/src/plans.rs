//! Plans (Admin → Plans) and each person's subscription: which plan they're on, and an admin's
//! override of its monthly allowance. Someone with no subscription row is on the default plan.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgConnection, PgPool};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::FromRow)]
pub struct PlanRow {
    pub id: Uuid,
    pub slug: String,
    pub name: String,
    pub description: String,
    pub monthly_tokens: i64,
    pub price_label: String,
    pub features: Vec<String>,
    pub card_tone: String,
    pub promo_label: Option<String>,
    pub promo_price_label: Option<String>,
    pub promo_ends_at: Option<DateTime<Utc>>,
    pub is_default: bool,
    pub is_active: bool,
    pub sort_order: i32,
    /// Limits: `None` is unlimited.
    pub max_projects: Option<i32>,
    pub max_orgs: Option<i32>,
    pub max_org_members: Option<i32>,
    pub max_custom_crew: Option<i32>,
    pub max_storage_mb: Option<i64>,
    pub custom_domain: bool,
    /// `small`, `default` or `large`; `None` is the platform default.
    pub sandbox_size: Option<String>,
    /// When false, only the models in `model_ids` are included.
    pub all_models: bool,
    pub model_ids: Vec<Uuid>,
    /// What a month on it costs, in rupiah, when it can be bought; None: people join its waitlist.
    pub price_idr: Option<i64>,
    /// A cheaper monthly price in rupiah, while `promo_ends_at` is in the future (or unset).
    pub promo_price_idr: Option<i64>,
}

impl PlanRow {
    /// The promo's monthly price while it is on: set, and not past its end date.
    pub fn active_promo_price(&self, now: DateTime<Utc>) -> Option<i64> {
        self.promo_price_idr.filter(|_| self.promo_ends_at.is_none_or(|end| end > now))
    }

    /// Whether people can pay for it: listed, and with a price (no price: its waitlist instead).
    pub fn for_sale(&self) -> bool {
        self.is_active && self.price_idr.is_some_and(|p| p > 0)
    }
}

/// The currency every plan is sold in.
pub const CURRENCY: &str = "IDR";

const PLAN_COLUMNS: &str = "id, slug, name, description, monthly_tokens, price_label, features, card_tone, promo_label, \
     promo_price_label, promo_ends_at, is_default, is_active, sort_order, max_projects, max_orgs, max_org_members, \
     max_custom_crew, max_storage_mb, custom_domain, sandbox_size, all_models, \
     price_idr, promo_price_idr, \
     ARRAY(SELECT model_id FROM plan_models WHERE plan_id = plans.id ORDER BY model_id) AS model_ids";

/// What an admin fills in for a plan.
#[derive(Debug, Clone, Deserialize)]
pub struct PlanInput {
    pub slug: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub monthly_tokens: i64,
    #[serde(default)]
    pub price_label: String,
    #[serde(default)]
    pub features: Vec<String>,
    #[serde(default = "default_tone")]
    pub card_tone: String,
    pub promo_label: Option<String>,
    pub promo_price_label: Option<String>,
    pub promo_ends_at: Option<DateTime<Utc>>,
    #[serde(default = "yes")]
    pub is_active: bool,
    #[serde(default)]
    pub sort_order: i32,
    #[serde(default)]
    pub max_projects: Option<i32>,
    #[serde(default)]
    pub max_orgs: Option<i32>,
    #[serde(default)]
    pub max_org_members: Option<i32>,
    #[serde(default)]
    pub max_custom_crew: Option<i32>,
    #[serde(default)]
    pub max_storage_mb: Option<i64>,
    #[serde(default = "yes")]
    pub custom_domain: bool,
    #[serde(default)]
    pub sandbox_size: Option<String>,
    #[serde(default = "yes")]
    pub all_models: bool,
    #[serde(default)]
    pub model_ids: Vec<Uuid>,
    /// None (or left out): not sold yet.
    #[serde(default)]
    pub price_idr: Option<i64>,
    #[serde(default)]
    pub promo_price_idr: Option<i64>,
}

fn default_tone() -> String {
    "glow".to_string()
}

fn yes() -> bool {
    true
}

/// The gradient tones a plan's card can wear (the app's shapes.ts tones).
pub const CARD_TONES: [&str; 8] = ["glow", "ember", "sky", "tide", "bloom", "citrus", "dusk", "slate"];

impl PlanInput {
    /// Trims it, and says what's wrong with it, if anything.
    pub fn validate(mut self) -> Result<PlanInput, &'static str> {
        self.slug = self.slug.trim().to_ascii_lowercase();
        self.name = self.name.trim().to_string();
        self.features = self.features.into_iter().map(|f| f.trim().to_string()).filter(|f| !f.is_empty()).collect();
        let blank_to_none = |v: Option<String>| v.map(|s| s.trim().to_string()).filter(|s| !s.is_empty());
        self.promo_label = blank_to_none(self.promo_label);
        self.promo_price_label = blank_to_none(self.promo_price_label);
        if self.slug.is_empty() || !self.slug.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
            return Err("slug may only use a-z, 0-9 and -");
        }
        if self.name.is_empty() {
            return Err("name is required");
        }
        if self.monthly_tokens <= 0 {
            return Err("monthly tokens must be more than zero");
        }
        if !CARD_TONES.contains(&self.card_tone.as_str()) {
            return Err("unknown card tone");
        }
        let negative = |v: Option<i64>| v.is_some_and(|n| n < 0);
        if [self.max_projects, self.max_orgs, self.max_org_members, self.max_custom_crew].into_iter().any(|v| negative(v.map(i64::from)))
            || negative(self.max_storage_mb)
        {
            return Err("limits can't be negative");
        }
        self.sandbox_size = self.sandbox_size.map(|s| s.trim().to_ascii_lowercase()).filter(|s| !s.is_empty());
        if self.sandbox_size.as_deref().is_some_and(|s| nomi_sandbox::Size::parse(s).is_none()) {
            return Err("unknown sandbox size");
        }
        if negative(self.promo_price_idr) {
            return Err("prices can't be negative");
        }
        self.model_ids.sort();
        self.model_ids.dedup();
        if self.price_idr.is_some_and(|p| p <= 0) {
            return Err("the price must be more than zero");
        }
        Ok(self)
    }
}

pub async fn list_plans(pool: &PgPool, active_only: bool) -> Result<Vec<PlanRow>, sqlx::Error> {
    sqlx::query_as(&format!(
        "SELECT {PLAN_COLUMNS} FROM plans WHERE ($1 = false OR is_active) ORDER BY sort_order, monthly_tokens"
    ))
    .bind(active_only)
    .fetch_all(pool)
    .await
}

pub async fn get_plan(pool: &PgPool, id: Uuid) -> Result<Option<PlanRow>, sqlx::Error> {
    sqlx::query_as(&format!("SELECT {PLAN_COLUMNS} FROM plans WHERE id = $1")).bind(id).fetch_optional(pool).await
}

pub async fn default_plan(pool: &PgPool) -> Result<PlanRow, sqlx::Error> {
    sqlx::query_as(&format!("SELECT {PLAN_COLUMNS} FROM plans ORDER BY is_default DESC, sort_order LIMIT 1")).fetch_one(pool).await
}

async fn get_plan_in(conn: &mut PgConnection, id: Uuid) -> Result<Option<PlanRow>, sqlx::Error> {
    sqlx::query_as(&format!("SELECT {PLAN_COLUMNS} FROM plans WHERE id = $1")).bind(id).fetch_optional(conn).await
}

pub(crate) async fn default_plan_in(conn: &mut PgConnection) -> Result<PlanRow, sqlx::Error> {
    sqlx::query_as(&format!("SELECT {PLAN_COLUMNS} FROM plans ORDER BY is_default DESC, sort_order LIMIT 1")).fetch_one(conn).await
}

pub async fn create_plan(pool: &PgPool, input: &PlanInput) -> Result<PlanRow, sqlx::Error> {
    let mut tx = pool.begin().await?;
    let id: Uuid = sqlx::query_scalar(
        "INSERT INTO plans (slug, name, description, monthly_tokens, price_label, features, card_tone, promo_label, \
                            promo_price_label, promo_ends_at, is_active, sort_order, max_projects, max_orgs, max_org_members, \
                            max_custom_crew, max_storage_mb, custom_domain, sandbox_size, all_models, \
                            price_idr, promo_price_idr) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22) RETURNING id",
    )
    .bind(&input.slug)
    .bind(&input.name)
    .bind(&input.description)
    .bind(input.monthly_tokens)
    .bind(&input.price_label)
    .bind(&input.features)
    .bind(&input.card_tone)
    .bind(&input.promo_label)
    .bind(&input.promo_price_label)
    .bind(input.promo_ends_at)
    .bind(input.is_active)
    .bind(input.sort_order)
    .bind(input.max_projects)
    .bind(input.max_orgs)
    .bind(input.max_org_members)
    .bind(input.max_custom_crew)
    .bind(input.max_storage_mb)
    .bind(input.custom_domain)
    .bind(&input.sandbox_size)
    .bind(input.all_models)
    .bind(input.price_idr)
    .bind(input.promo_price_idr)
    .fetch_one(&mut *tx)
    .await?;
    save_models(&mut tx, id, &input.model_ids).await?;
    tx.commit().await?;
    get_plan(pool, id).await?.ok_or(sqlx::Error::RowNotFound)
}

pub async fn update_plan(pool: &PgPool, id: Uuid, input: &PlanInput) -> Result<Option<PlanRow>, sqlx::Error> {
    let mut tx = pool.begin().await?;
    let updated = sqlx::query(
        "UPDATE plans SET slug = $2, name = $3, description = $4, monthly_tokens = $5, price_label = $6, features = $7, \
             card_tone = $8, promo_label = $9, promo_price_label = $10, promo_ends_at = $11, \
             is_active = $12 OR is_default, sort_order = $13, max_projects = $14, max_orgs = $15, max_org_members = $16, \
             max_custom_crew = $17, max_storage_mb = $18, custom_domain = $19, sandbox_size = $20, all_models = $21, \
             price_idr = $22, promo_price_idr = $23, updated_at = now() \
         WHERE id = $1",
    )
    .bind(id)
    .bind(&input.slug)
    .bind(&input.name)
    .bind(&input.description)
    .bind(input.monthly_tokens)
    .bind(&input.price_label)
    .bind(&input.features)
    .bind(&input.card_tone)
    .bind(&input.promo_label)
    .bind(&input.promo_price_label)
    .bind(input.promo_ends_at)
    .bind(input.is_active)
    .bind(input.sort_order)
    .bind(input.max_projects)
    .bind(input.max_orgs)
    .bind(input.max_org_members)
    .bind(input.max_custom_crew)
    .bind(input.max_storage_mb)
    .bind(input.custom_domain)
    .bind(&input.sandbox_size)
    .bind(input.all_models)
    .bind(input.price_idr)
    .bind(input.promo_price_idr)
    .execute(&mut *tx)
    .await?
    .rows_affected();
    if updated == 0 {
        return Ok(None);
    }
    save_models(&mut tx, id, &input.model_ids).await?;
    tx.commit().await?;
    get_plan(pool, id).await
}

/// Replaces the models a plan includes.
async fn save_models(tx: &mut sqlx::Transaction<'_, sqlx::Postgres>, plan_id: Uuid, model_ids: &[Uuid]) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM plan_models WHERE plan_id = $1").bind(plan_id).execute(&mut **tx).await?;
    sqlx::query("INSERT INTO plan_models (plan_id, model_id) SELECT $1, UNNEST($2::uuid[])")
        .bind(plan_id)
        .bind(model_ids)
        .execute(&mut **tx)
        .await?;
    Ok(())
}

/// Makes `id` the plan new people start on. `false` when it doesn't exist.
pub async fn set_default_plan(pool: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
    let mut tx = pool.begin().await?;
    let exists: bool = sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM plans WHERE id = $1)").bind(id).fetch_one(&mut *tx).await?;
    if !exists {
        return Ok(false);
    }
    sqlx::query("UPDATE plans SET is_default = false WHERE is_default").execute(&mut *tx).await?;
    sqlx::query("UPDATE plans SET is_default = true, is_active = true WHERE id = $1").bind(id).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(true)
}

#[derive(Debug, PartialEq, Eq)]
pub enum DeletePlan {
    Deleted,
    NotFound,
    IsDefault,
    /// People are on it (or were, in the change history); hide it instead.
    InUse,
}

pub async fn delete_plan(pool: &PgPool, id: Uuid) -> Result<DeletePlan, sqlx::Error> {
    let Some(plan) = get_plan(pool, id).await? else { return Ok(DeletePlan::NotFound) };
    if plan.is_default {
        return Ok(DeletePlan::IsDefault);
    }
    let used: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM user_subscriptions WHERE plan_id = $1) OR EXISTS (SELECT 1 FROM subscription_changes WHERE plan_id = $1) \
         OR EXISTS (SELECT 1 FROM payments WHERE plan_id = $1)",
    )
    .bind(id)
    .fetch_one(pool)
    .await?;
    if used {
        return Ok(DeletePlan::InUse);
    }
    sqlx::query("DELETE FROM plans WHERE id = $1").bind(id).execute(pool).await?;
    Ok(DeletePlan::Deleted)
}

/// Someone's plan and allowance, as it stands now.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Subscription {
    pub plan: PlanRow,
    /// The admin's override of the plan's allowance, while it lasts.
    pub quota_override: Option<i64>,
    pub override_until: Option<DateTime<Utc>>,
    pub note: Option<String>,
    /// Tokens a month they may use on Nomi's models: the override while it lasts, else the plan's.
    pub monthly_tokens: i64,
    /// When the paid period ends; `None` for a plan nobody paid for (the default, or an admin's).
    pub paid_until: Option<DateTime<Utc>>,
    pub term_months: Option<i32>,
    pub auto_renew: bool,
    /// The paid period and its grace have passed: they're on the default plan again.
    pub lapsed: bool,
}

/// What `user_subscriptions` holds for one person.
#[derive(sqlx::FromRow)]
struct SubscriptionRow {
    plan_id: Uuid,
    quota_override: Option<i64>,
    override_until: Option<DateTime<Utc>>,
    note: Option<String>,
    paid_until: Option<DateTime<Utc>>,
    term_months: Option<i32>,
    auto_renew: bool,
}

pub async fn subscription_for(pool: &PgPool, user_id: Uuid) -> Result<Subscription, sqlx::Error> {
    subscription_in(&mut *pool.acquire().await?, user_id).await
}

/// [`subscription_for`] on a connection already in hand.
pub async fn subscription_in(conn: &mut PgConnection, user_id: Uuid) -> Result<Subscription, sqlx::Error> {
    let row: Option<SubscriptionRow> = sqlx::query_as(
        "SELECT plan_id, quota_override, override_until, note, paid_until, term_months, auto_renew \
         FROM user_subscriptions WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_optional(&mut *conn)
    .await?;
    let Some(row) = row else {
        let plan = default_plan_in(&mut *conn).await?;
        let monthly_tokens = plan.monthly_tokens;
        return Ok(Subscription {
            plan,
            quota_override: None,
            override_until: None,
            note: None,
            monthly_tokens,
            paid_until: None,
            term_months: None,
            auto_renew: true,
            lapsed: false,
        });
    };
    // A paid period that ran out (and its grace) puts them back on the default plan; an admin's
    // override of the allowance is theirs to end, so it stays.
    let grace_days = crate::billing::config().await.grace_days;
    let lapsed = row.paid_until.is_some_and(|until| crate::billing::is_lapsed(until, Utc::now(), grace_days));
    let plan = if lapsed {
        default_plan_in(&mut *conn).await?
    } else {
        match get_plan_in(&mut *conn, row.plan_id).await? {
            Some(plan) => plan,
            None => default_plan_in(&mut *conn).await?,
        }
    };
    // An override past its date no longer counts.
    let active_override = row.quota_override.filter(|_| row.override_until.is_none_or(|until| until > Utc::now()));
    let monthly_tokens = active_override.unwrap_or(plan.monthly_tokens);
    Ok(Subscription {
        plan,
        quota_override: active_override,
        override_until: active_override.and(row.override_until),
        note: row.note,
        monthly_tokens,
        paid_until: row.paid_until,
        term_months: row.term_months,
        auto_renew: row.auto_renew,
        lapsed,
    })
}

/// An admin's change to someone's subscription.
#[derive(Debug, Clone, Deserialize)]
pub struct SubscriptionChange {
    pub plan_id: Uuid,
    /// `None` uses the plan's own allowance.
    pub quota_override: Option<i64>,
    /// When the override ends; `None` keeps it until changed.
    pub override_until: Option<DateTime<Utc>>,
    pub note: Option<String>,
}

/// Saves `change` for `user_id`, and records it in the change history.
pub async fn change_subscription(pool: &PgPool, user_id: Uuid, change: &SubscriptionChange, changed_by: Uuid) -> Result<(), sqlx::Error> {
    let override_until = change.quota_override.and(change.override_until);
    let note = change.note.as_deref().map(str::trim).filter(|n| !n.is_empty());
    // A plan an admin assigns has no end: only the same plan keeps the paid period it already has.
    let mut tx = pool.begin().await?;
    sqlx::query(
        "INSERT INTO user_subscriptions (user_id, plan_id, quota_override, override_until, note, updated_by) \
         VALUES ($1, $2, $3, $4, $5, $6) \
         ON CONFLICT (user_id) DO UPDATE SET \
             paid_until = CASE WHEN user_subscriptions.plan_id = $2 THEN user_subscriptions.paid_until END, \
             term_months = CASE WHEN user_subscriptions.plan_id = $2 THEN user_subscriptions.term_months END, \
             plan_id = $2, quota_override = $3, override_until = $4, note = $5, updated_by = $6, updated_at = now()",
    )
    .bind(user_id)
    .bind(change.plan_id)
    .bind(change.quota_override)
    .bind(override_until)
    .bind(note)
    .bind(changed_by)
    .execute(&mut *tx)
    .await?;
    sqlx::query(
        "INSERT INTO subscription_changes (user_id, plan_id, quota_override, override_until, note, changed_by) VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(user_id)
    .bind(change.plan_id)
    .bind(change.quota_override)
    .bind(override_until)
    .bind(note)
    .bind(changed_by)
    .execute(&mut *tx)
    .await?;
    tx.commit().await
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct ChangeRow {
    pub plan_name: String,
    pub quota_override: Option<i64>,
    pub override_until: Option<DateTime<Utc>>,
    pub note: Option<String>,
    pub changed_by_email: Option<String>,
    pub created_at: DateTime<Utc>,
}

pub async fn change_history(pool: &PgPool, user_id: Uuid) -> Result<Vec<ChangeRow>, sqlx::Error> {
    sqlx::query_as(
        "SELECT p.name AS plan_name, c.quota_override, c.override_until, c.note, w.email AS changed_by_email, c.created_at \
         FROM subscription_changes c JOIN plans p ON p.id = c.plan_id \
         LEFT JOIN web_credentials w ON w.user_id = c.changed_by \
         WHERE c.user_id = $1 ORDER BY c.created_at DESC LIMIT 20",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input() -> PlanInput {
        PlanInput {
            slug: " Team ".into(),
            name: " Team ".into(),
            description: String::new(),
            monthly_tokens: 5,
            price_label: String::new(),
            features: vec![" a ".into(), " ".into()],
            card_tone: "sky".into(),
            promo_label: Some(" ".into()),
            promo_price_label: None,
            promo_ends_at: None,
            is_active: true,
            sort_order: 0,
            max_projects: None,
            max_orgs: None,
            max_org_members: None,
            max_custom_crew: None,
            max_storage_mb: None,
            custom_domain: true,
            sandbox_size: None,
            all_models: true,
            model_ids: vec![],
            price_idr: None,
            promo_price_idr: None,
        }
    }

    #[test]
    fn plan_input_is_trimmed_and_checked() {
        let ok = input().validate().unwrap();
        assert_eq!((ok.slug.as_str(), ok.name.as_str()), ("team", "Team"));
        assert_eq!(ok.features, vec!["a".to_string()]);
        assert_eq!(ok.promo_label, None);
        assert!(PlanInput { monthly_tokens: 0, ..input() }.validate().is_err());
        assert!(PlanInput { slug: "pro plan".into(), ..input() }.validate().is_err());
        assert!(PlanInput { card_tone: "neon".into(), ..input() }.validate().is_err());
        assert!(PlanInput { max_projects: Some(-1), ..input() }.validate().is_err());
        assert!(PlanInput { max_storage_mb: Some(-1), ..input() }.validate().is_err());
        assert!(PlanInput { sandbox_size: Some("huge".into()), ..input() }.validate().is_err());
        assert_eq!(PlanInput { sandbox_size: Some(" Small ".into()), ..input() }.validate().unwrap().sandbox_size.as_deref(), Some("small"));
        assert_eq!(PlanInput { sandbox_size: Some(" ".into()), ..input() }.validate().unwrap().sandbox_size, None);
        assert!(PlanInput { promo_price_idr: Some(-1), ..input() }.validate().is_err());
        assert!(PlanInput { price_idr: Some(0), ..input() }.validate().is_err());
    }
}
