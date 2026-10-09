//! Plans (Admin → Plans) and each person's subscription: which plan they're on, and an admin's
//! override of its monthly allowance. Someone with no subscription row is on the default plan.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
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
}

const PLAN_COLUMNS: &str = "id, slug, name, description, monthly_tokens, price_label, features, card_tone, promo_label, \
     promo_price_label, promo_ends_at, is_default, is_active, sort_order";

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

pub async fn create_plan(pool: &PgPool, input: &PlanInput) -> Result<PlanRow, sqlx::Error> {
    sqlx::query_as(&format!(
        "INSERT INTO plans (slug, name, description, monthly_tokens, price_label, features, card_tone, promo_label, \
                            promo_price_label, promo_ends_at, is_active, sort_order) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12) RETURNING {PLAN_COLUMNS}"
    ))
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
    .fetch_one(pool)
    .await
}

pub async fn update_plan(pool: &PgPool, id: Uuid, input: &PlanInput) -> Result<Option<PlanRow>, sqlx::Error> {
    sqlx::query_as(&format!(
        "UPDATE plans SET slug = $2, name = $3, description = $4, monthly_tokens = $5, price_label = $6, features = $7, \
             card_tone = $8, promo_label = $9, promo_price_label = $10, promo_ends_at = $11, \
             is_active = $12 OR is_default, sort_order = $13, updated_at = now() \
         WHERE id = $1 RETURNING {PLAN_COLUMNS}"
    ))
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
    .fetch_optional(pool)
    .await
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
        "SELECT EXISTS (SELECT 1 FROM user_subscriptions WHERE plan_id = $1) OR EXISTS (SELECT 1 FROM subscription_changes WHERE plan_id = $1)",
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
}

pub async fn subscription_for(pool: &PgPool, user_id: Uuid) -> Result<Subscription, sqlx::Error> {
    let row: Option<(Uuid, Option<i64>, Option<DateTime<Utc>>, Option<String>)> =
        sqlx::query_as("SELECT plan_id, quota_override, override_until, note FROM user_subscriptions WHERE user_id = $1")
            .bind(user_id)
            .fetch_optional(pool)
            .await?;
    let (plan, quota_override, override_until, note) = match row {
        Some((plan_id, quota_override, override_until, note)) => {
            let plan = match get_plan(pool, plan_id).await? {
                Some(plan) => plan,
                None => default_plan(pool).await?,
            };
            (plan, quota_override, override_until, note)
        }
        None => (default_plan(pool).await?, None, None, None),
    };
    // An override past its date no longer counts.
    let active_override = quota_override.filter(|_| override_until.is_none_or(|until| until > Utc::now()));
    let monthly_tokens = active_override.unwrap_or(plan.monthly_tokens);
    Ok(Subscription { plan, quota_override: active_override, override_until: active_override.and(override_until), note, monthly_tokens })
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
    let mut tx = pool.begin().await?;
    sqlx::query(
        "INSERT INTO user_subscriptions (user_id, plan_id, quota_override, override_until, note, updated_by) \
         VALUES ($1, $2, $3, $4, $5, $6) \
         ON CONFLICT (user_id) DO UPDATE SET plan_id = $2, quota_override = $3, override_until = $4, note = $5, \
             updated_by = $6, updated_at = now()",
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
    }
}
