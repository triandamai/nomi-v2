//! Paid plans: what a plan costs for a number of months (with the term's discount, the plan's
//! promo price, a coupon or a referral code), the payments that pay for it, and what activating
//! one does to someone's subscription. Pricing is pure ([`price`]); the rest is database access.
//! All money is whole currency units in an `i64`.

use std::fmt;

use chrono::{DateTime, Duration, Months, Utc};
use nomi_settings::config::{self, Section};
use serde::{Deserialize, Serialize};
use sqlx::{PgConnection, PgPool};
use uuid::Uuid;

use crate::plans::{self, PlanRow};

/// A discount on the first payment of someone who came with a referral code, in percent.
pub const DEFAULT_REFEREE_DISCOUNT_PERCENT: i64 = 10;
/// Days added to a referrer's plan when the person they referred pays.
pub const DEFAULT_REFERRER_BONUS_DAYS: i32 = 30;
/// Days before the paid period ends that the renewal invoice is issued.
pub const DEFAULT_RENEWAL_DAYS_BEFORE: i32 = 7;
/// Days after the paid period ends before the person falls back to the default plan.
pub const DEFAULT_GRACE_DAYS: i32 = 3;
/// How long an invoice can be paid, in hours.
pub const DEFAULT_INVOICE_HOURS: u32 = 24;

/// The most a percentage can be.
const PERCENT_MAX: i64 = 100;
/// Length of a referral code.
pub const REFERRAL_CODE_LEN: usize = 8;
/// Tries at finding a referral code nobody has.
const REFERRAL_CODE_ATTEMPTS: u32 = 10;
const REFERRAL_ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
/// Payments on a page of the admin list.
pub const PAGE_SIZE: i64 = 50;
/// Most payments the person's own list returns.
pub const USER_PAYMENTS_LIMIT: i64 = 100;
/// The longest coupon code accepted.
const COUPON_CODE_MAX: usize = 40;

pub const KIND_NEW: &str = "new";
pub const KIND_RENEWAL: &str = "renewal";
pub const STATUS_PENDING: &str = "pending";
pub const STATUS_PAID: &str = "paid";
pub const STATUS_EXPIRED: &str = "expired";
pub const STATUS_FAILED: &str = "failed";
pub const STATUS_CANCELLED: &str = "cancelled";
pub const STATUSES: [&str; 5] = [STATUS_PENDING, STATUS_PAID, STATUS_EXPIRED, STATUS_FAILED, STATUS_CANCELLED];
pub const NOTICE_REMINDER: &str = "reminder";
pub const NOTICE_ENDED: &str = "ended";

// ---------------------------------------------------------------------------------------------
// Settings

/// Admin → Configuration → Billing, with the defaults filled in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BillingConfig {
    pub referee_discount_percent: i64,
    pub referrer_bonus_days: i32,
    pub renewal_days_before: i32,
    pub grace_days: i32,
    pub invoice_hours: u32,
}

impl Default for BillingConfig {
    fn default() -> Self {
        BillingConfig {
            referee_discount_percent: DEFAULT_REFEREE_DISCOUNT_PERCENT,
            referrer_bonus_days: DEFAULT_REFERRER_BONUS_DAYS,
            renewal_days_before: DEFAULT_RENEWAL_DAYS_BEFORE,
            grace_days: DEFAULT_GRACE_DAYS,
            invoice_hours: DEFAULT_INVOICE_HOURS,
        }
    }
}

impl BillingConfig {
    pub fn from_section(section: &Section) -> BillingConfig {
        let defaults = BillingConfig::default();
        let whole = |field: &str, default: i32| section.number(field).map_or(default, |n| i32::try_from(n).unwrap_or(i32::MAX));
        BillingConfig {
            referee_discount_percent: section
                .number("referee_discount_percent")
                .map_or(defaults.referee_discount_percent, |n| i64::try_from(n).unwrap_or(PERCENT_MAX).min(PERCENT_MAX)),
            referrer_bonus_days: whole("referrer_bonus_days", defaults.referrer_bonus_days),
            renewal_days_before: whole("renewal_days_before", defaults.renewal_days_before),
            grace_days: whole("grace_days", defaults.grace_days),
            invoice_hours: section.number("invoice_hours").map_or(defaults.invoice_hours, |n| u32::try_from(n).unwrap_or(u32::MAX)),
        }
    }
}

/// The billing settings as saved now (cached for a few seconds), else the defaults.
pub async fn config() -> BillingConfig {
    BillingConfig::from_section(&config::current(&config::BILLING).await)
}

/// Whether a paid period that ended at `paid_until` is over for good at `now`: the grace is gone too.
pub fn is_lapsed(paid_until: DateTime<Utc>, now: DateTime<Utc>, grace_days: i32) -> bool {
    paid_until + Duration::days(i64::from(grace_days)) <= now
}

/// Whether a paid period ending at `paid_until` is within `days_before` days of its end (or past it).
pub fn in_renewal_window(paid_until: DateTime<Utc>, now: DateTime<Utc>, days_before: i32) -> bool {
    paid_until <= now + Duration::days(i64::from(days_before))
}

// ---------------------------------------------------------------------------------------------
// Pricing

/// A coupon's discount.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CouponDiscount {
    /// Percent of what's left, 1 to 100.
    Percent(i64),
    /// A fixed amount, never more than what's left.
    Amount(i64),
}

/// Where a price comes from, step by step. `amount` is what's paid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Breakdown {
    pub base_amount: i64,
    pub promo_discount: i64,
    pub term_discount: i64,
    pub coupon_discount: i64,
    pub referral_discount: i64,
    pub amount: i64,
}

fn percent_of(amount: i64, percent: i64) -> i64 {
    // In i128 so a big amount times a percent can't overflow; rounds down.
    (i128::from(amount) * i128::from(percent.clamp(0, PERCENT_MAX)) / i128::from(PERCENT_MAX)) as i64
}

/// The price of `months` months at `monthly_price`. In order: the plan's promo price, then the
/// term's percent off what's left, then the coupon, then the referral percent. Discounts round
/// down, and the amount never goes below 0.
pub fn price(
    monthly_price: i64,
    promo_monthly: Option<i64>,
    months: i32,
    term_discount_percent: i64,
    coupon: Option<CouponDiscount>,
    referral_percent: Option<i64>,
) -> Breakdown {
    let months = i64::from(months.max(0));
    let base_amount = monthly_price.max(0).saturating_mul(months);
    let promo_discount = promo_monthly.map_or(0, |promo| (monthly_price - promo.max(0)).max(0).saturating_mul(months)).min(base_amount);
    let after_promo = base_amount - promo_discount;
    let term_discount = percent_of(after_promo, term_discount_percent);
    let after_term = after_promo - term_discount;
    let coupon_discount = match coupon {
        Some(CouponDiscount::Percent(percent)) => percent_of(after_term, percent),
        Some(CouponDiscount::Amount(off)) => off.clamp(0, after_term),
        None => 0,
    };
    let after_coupon = after_term - coupon_discount;
    let referral_discount = referral_percent.map_or(0, |percent| percent_of(after_coupon, percent));
    Breakdown { base_amount, promo_discount, term_discount, coupon_discount, referral_discount, amount: after_coupon - referral_discount }
}

/// What a person would pay, and for what.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Quote {
    pub plan_id: Uuid,
    pub term_months: i32,
    pub currency: String,
    /// The plan's list price per month.
    pub monthly_price: i64,
    pub base_amount: i64,
    pub promo_discount: i64,
    pub term_discount: i64,
    pub coupon_discount: i64,
    pub referral_discount: i64,
    pub amount: i64,
    pub coupon_code: Option<String>,
    pub referral_code: Option<String>,
    #[serde(skip)]
    pub coupon_id: Option<Uuid>,
}

/// Why a quote can't be made, in words fit for the person.
#[derive(Debug)]
pub enum QuoteError {
    PlanUnavailable,
    UnknownTerm,
    UnknownCode,
    CouponNotStarted,
    CouponEnded,
    CouponInactive,
    CouponUsedUp,
    CouponAlreadyUsed,
    CouponWrongPlan,
    CouponMinMonths(i32),
    CouponFirstPaymentOnly,
    OwnReferralCode,
    ReferralFirstPaymentOnly,
    Db(sqlx::Error),
}

impl fmt::Display for QuoteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QuoteError::PlanUnavailable => f.write_str("That plan isn't available to buy."),
            QuoteError::UnknownTerm => f.write_str("That billing period isn't available."),
            QuoteError::UnknownCode => f.write_str("We don't recognise that code."),
            QuoteError::CouponNotStarted => f.write_str("That coupon isn't valid yet."),
            QuoteError::CouponEnded => f.write_str("That coupon has expired."),
            QuoteError::CouponInactive => f.write_str("That coupon isn't active."),
            QuoteError::CouponUsedUp => f.write_str("That coupon has been used up."),
            QuoteError::CouponAlreadyUsed => f.write_str("You've already used that coupon."),
            QuoteError::CouponWrongPlan => f.write_str("That coupon isn't valid for this plan."),
            QuoteError::CouponMinMonths(months) => write!(f, "That coupon needs at least {months} months."),
            QuoteError::CouponFirstPaymentOnly => f.write_str("That coupon is only for a first payment."),
            QuoteError::OwnReferralCode => f.write_str("You can't use your own referral code."),
            QuoteError::ReferralFirstPaymentOnly => f.write_str("A referral code only works on a first payment."),
            QuoteError::Db(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for QuoteError {}

impl From<sqlx::Error> for QuoteError {
    fn from(e: sqlx::Error) -> Self {
        QuoteError::Db(e)
    }
}

// ---------------------------------------------------------------------------------------------
// Terms

/// A number of months someone can pay for at once, and what that earns them off.
#[derive(Debug, Clone, Serialize, PartialEq, Eq, sqlx::FromRow)]
pub struct Term {
    pub id: Uuid,
    pub months: i32,
    pub discount_percent: i32,
    pub label: String,
    pub is_active: bool,
    pub sort_order: i32,
}

const TERM_COLUMNS: &str = "id, months, discount_percent, label, is_active, sort_order";

#[derive(Debug, Clone, Deserialize)]
pub struct TermInput {
    pub months: i32,
    #[serde(default)]
    pub discount_percent: i32,
    #[serde(default)]
    pub label: String,
    #[serde(default = "yes")]
    pub is_active: bool,
    #[serde(default)]
    pub sort_order: i32,
}

fn yes() -> bool {
    true
}

impl TermInput {
    pub fn validate(mut self) -> Result<TermInput, &'static str> {
        self.label = self.label.trim().to_string();
        if self.months <= 0 {
            return Err("months must be more than zero");
        }
        if !(0..=PERCENT_MAX as i32).contains(&self.discount_percent) {
            return Err("the discount must be between 0 and 100 percent");
        }
        Ok(self)
    }
}

pub async fn list_terms(pool: &PgPool, active_only: bool) -> Result<Vec<Term>, sqlx::Error> {
    sqlx::query_as(&format!("SELECT {TERM_COLUMNS} FROM billing_terms WHERE ($1 = false OR is_active) ORDER BY sort_order, months"))
        .bind(active_only)
        .fetch_all(pool)
        .await
}

pub async fn create_term(pool: &PgPool, input: &TermInput) -> Result<Term, sqlx::Error> {
    sqlx::query_as(&format!(
        "INSERT INTO billing_terms (months, discount_percent, label, is_active, sort_order) VALUES ($1, $2, $3, $4, $5) RETURNING {TERM_COLUMNS}"
    ))
    .bind(input.months)
    .bind(input.discount_percent)
    .bind(&input.label)
    .bind(input.is_active)
    .bind(input.sort_order)
    .fetch_one(pool)
    .await
}

pub async fn update_term(pool: &PgPool, id: Uuid, input: &TermInput) -> Result<Option<Term>, sqlx::Error> {
    sqlx::query_as(&format!(
        "UPDATE billing_terms SET months = $2, discount_percent = $3, label = $4, is_active = $5, sort_order = $6, updated_at = now() \
         WHERE id = $1 RETURNING {TERM_COLUMNS}"
    ))
    .bind(id)
    .bind(input.months)
    .bind(input.discount_percent)
    .bind(&input.label)
    .bind(input.is_active)
    .bind(input.sort_order)
    .fetch_optional(pool)
    .await
}

/// `false` when there's no such term.
pub async fn delete_term(pool: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
    Ok(sqlx::query("DELETE FROM billing_terms WHERE id = $1").bind(id).execute(pool).await?.rows_affected() > 0)
}

// ---------------------------------------------------------------------------------------------
// Coupons

pub const DISCOUNT_PERCENT: &str = "percent";
pub const DISCOUNT_AMOUNT: &str = "amount";

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Coupon {
    pub id: Uuid,
    pub code: String,
    pub description: Option<String>,
    pub discount_kind: String,
    pub discount_value: i64,
    pub max_redemptions: Option<i32>,
    pub per_user_limit: i32,
    pub starts_at: Option<DateTime<Utc>>,
    pub ends_at: Option<DateTime<Utc>>,
    /// Empty: any plan.
    pub plan_ids: Vec<Uuid>,
    pub min_months: i32,
    pub first_payment_only: bool,
    pub is_active: bool,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    /// Pending and paid payments that used it.
    pub redemptions: i64,
}

const COUPON_COLUMNS: &str = "id, code, description, discount_kind, discount_value, max_redemptions, per_user_limit, starts_at, ends_at, \
     plan_ids, min_months, first_payment_only, is_active, created_by, created_at, updated_at, \
     (SELECT count(*) FROM coupon_redemptions r WHERE r.coupon_id = coupons.id) AS redemptions";

impl Coupon {
    fn discount(&self) -> CouponDiscount {
        if self.discount_kind == DISCOUNT_PERCENT {
            CouponDiscount::Percent(self.discount_value)
        } else {
            CouponDiscount::Amount(self.discount_value)
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct CouponInput {
    pub code: String,
    #[serde(default)]
    pub description: Option<String>,
    pub discount_kind: String,
    pub discount_value: i64,
    #[serde(default)]
    pub max_redemptions: Option<i32>,
    #[serde(default = "one")]
    pub per_user_limit: i32,
    #[serde(default)]
    pub starts_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub ends_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub plan_ids: Vec<Uuid>,
    #[serde(default = "one")]
    pub min_months: i32,
    #[serde(default)]
    pub first_payment_only: bool,
    #[serde(default = "yes")]
    pub is_active: bool,
}

fn one() -> i32 {
    1
}

impl CouponInput {
    /// Trims and upper-cases it, and says what's wrong with it, if anything.
    pub fn validate(mut self) -> Result<CouponInput, &'static str> {
        self.code = self.code.trim().to_ascii_uppercase();
        self.description = self.description.map(|d| d.trim().to_string()).filter(|d| !d.is_empty());
        if self.code.is_empty() || self.code.len() > COUPON_CODE_MAX || !self.code.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '-' || c == '_') {
            return Err("the code may only use A-Z, 0-9, - and _");
        }
        if self.discount_kind != DISCOUNT_PERCENT && self.discount_kind != DISCOUNT_AMOUNT {
            return Err("the discount must be a percent or an amount");
        }
        if self.discount_value <= 0 || (self.discount_kind == DISCOUNT_PERCENT && self.discount_value > PERCENT_MAX) {
            return Err("the discount must be more than zero (and at most 100 percent)");
        }
        if self.max_redemptions.is_some_and(|n| n <= 0) || self.per_user_limit <= 0 || self.min_months <= 0 {
            return Err("limits must be more than zero");
        }
        if matches!((self.starts_at, self.ends_at), (Some(start), Some(end)) if end <= start) {
            return Err("the coupon must end after it starts");
        }
        self.plan_ids.sort();
        self.plan_ids.dedup();
        Ok(self)
    }
}

pub async fn list_coupons(pool: &PgPool) -> Result<Vec<Coupon>, sqlx::Error> {
    sqlx::query_as(&format!("SELECT {COUPON_COLUMNS} FROM coupons ORDER BY created_at DESC")).fetch_all(pool).await
}

async fn coupon_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Coupon>, sqlx::Error> {
    sqlx::query_as(&format!("SELECT {COUPON_COLUMNS} FROM coupons WHERE id = $1")).bind(id).fetch_optional(pool).await
}

pub async fn create_coupon(pool: &PgPool, input: &CouponInput, created_by: Uuid) -> Result<Coupon, sqlx::Error> {
    let id: Uuid = sqlx::query_scalar(
        "INSERT INTO coupons (code, description, discount_kind, discount_value, max_redemptions, per_user_limit, starts_at, ends_at, \
                              plan_ids, min_months, first_payment_only, is_active, created_by) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13) RETURNING id",
    )
    .bind(&input.code)
    .bind(&input.description)
    .bind(&input.discount_kind)
    .bind(input.discount_value)
    .bind(input.max_redemptions)
    .bind(input.per_user_limit)
    .bind(input.starts_at)
    .bind(input.ends_at)
    .bind(&input.plan_ids)
    .bind(input.min_months)
    .bind(input.first_payment_only)
    .bind(input.is_active)
    .bind(created_by)
    .fetch_one(pool)
    .await?;
    coupon_by_id(pool, id).await?.ok_or(sqlx::Error::RowNotFound)
}

pub async fn update_coupon(pool: &PgPool, id: Uuid, input: &CouponInput) -> Result<Option<Coupon>, sqlx::Error> {
    let updated = sqlx::query(
        "UPDATE coupons SET code = $2, description = $3, discount_kind = $4, discount_value = $5, max_redemptions = $6, \
             per_user_limit = $7, starts_at = $8, ends_at = $9, plan_ids = $10, min_months = $11, first_payment_only = $12, \
             is_active = $13, updated_at = now() \
         WHERE id = $1",
    )
    .bind(id)
    .bind(&input.code)
    .bind(&input.description)
    .bind(&input.discount_kind)
    .bind(input.discount_value)
    .bind(input.max_redemptions)
    .bind(input.per_user_limit)
    .bind(input.starts_at)
    .bind(input.ends_at)
    .bind(&input.plan_ids)
    .bind(input.min_months)
    .bind(input.first_payment_only)
    .bind(input.is_active)
    .execute(pool)
    .await?
    .rows_affected();
    if updated == 0 {
        return Ok(None);
    }
    coupon_by_id(pool, id).await
}

#[derive(Debug, PartialEq, Eq)]
pub enum DeleteCoupon {
    Deleted,
    NotFound,
    /// Someone used it (or is paying with it); deactivate it instead.
    Redeemed,
}

pub async fn delete_coupon(pool: &PgPool, id: Uuid) -> Result<DeleteCoupon, sqlx::Error> {
    let Some(coupon) = coupon_by_id(pool, id).await? else { return Ok(DeleteCoupon::NotFound) };
    if coupon.redemptions > 0 {
        return Ok(DeleteCoupon::Redeemed);
    }
    sqlx::query("DELETE FROM coupons WHERE id = $1").bind(id).execute(pool).await?;
    Ok(DeleteCoupon::Deleted)
}

// ---------------------------------------------------------------------------------------------
// Referral codes

fn random_referral_code() -> String {
    let bytes = Uuid::new_v4().into_bytes();
    bytes.iter().take(REFERRAL_CODE_LEN).map(|b| REFERRAL_ALPHABET[usize::from(*b) % REFERRAL_ALPHABET.len()] as char).collect()
}

/// Their referral code, made on the first call.
pub async fn referral_code_for(pool: &PgPool, user_id: Uuid) -> Result<String, sqlx::Error> {
    let existing = || sqlx::query_scalar::<_, String>("SELECT code FROM referral_codes WHERE user_id = $1").bind(user_id).fetch_optional(pool);
    if let Some(code) = existing().await? {
        return Ok(code);
    }
    for _ in 0..REFERRAL_CODE_ATTEMPTS {
        let code = random_referral_code();
        let inserted = sqlx::query("INSERT INTO referral_codes (user_id, code) VALUES ($1, $2) ON CONFLICT DO NOTHING")
            .bind(user_id)
            .bind(&code)
            .execute(pool)
            .await?
            .rows_affected();
        if inserted == 1 {
            return Ok(code);
        }
        // Either someone else made theirs in the meantime, or the code is taken: look, then retry.
        if let Some(code) = existing().await? {
            return Ok(code);
        }
    }
    Err(sqlx::Error::Protocol("couldn't find a free referral code".into()))
}

/// How many people they referred, and how many of those they were rewarded for.
pub async fn referral_counts(pool: &PgPool, user_id: Uuid) -> Result<(i64, i64), sqlx::Error> {
    sqlx::query_as("SELECT count(*), count(rewarded_at) FROM referrals WHERE referrer_user_id = $1").bind(user_id).fetch_one(pool).await
}

// ---------------------------------------------------------------------------------------------
// Quotes

/// Whether `user_id` has paid for anything yet.
pub async fn has_paid(pool: &PgPool, user_id: Uuid) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM payments WHERE user_id = $1 AND status = 'paid')").bind(user_id).fetch_one(pool).await
}

/// `code` as it's stored and compared: trimmed and upper-cased; `None` when blank.
pub fn normalize_code(code: Option<&str>) -> Option<String> {
    code.map(|c| c.trim().to_ascii_uppercase()).filter(|c| !c.is_empty())
}

async fn check_coupon(pool: &PgPool, coupon: &Coupon, user_id: Uuid, plan_id: Uuid, months: i32, now: DateTime<Utc>) -> Result<(), QuoteError> {
    if !coupon.is_active {
        return Err(QuoteError::CouponInactive);
    }
    if coupon.starts_at.is_some_and(|start| start > now) {
        return Err(QuoteError::CouponNotStarted);
    }
    if coupon.ends_at.is_some_and(|end| end <= now) {
        return Err(QuoteError::CouponEnded);
    }
    if !coupon.plan_ids.is_empty() && !coupon.plan_ids.contains(&plan_id) {
        return Err(QuoteError::CouponWrongPlan);
    }
    if months < coupon.min_months {
        return Err(QuoteError::CouponMinMonths(coupon.min_months));
    }
    if coupon.first_payment_only && has_paid(pool, user_id).await? {
        return Err(QuoteError::CouponFirstPaymentOnly);
    }
    if coupon.max_redemptions.is_some_and(|max| coupon.redemptions >= i64::from(max)) {
        return Err(QuoteError::CouponUsedUp);
    }
    let by_user: i64 = sqlx::query_scalar("SELECT count(*) FROM coupon_redemptions WHERE coupon_id = $1 AND user_id = $2")
        .bind(coupon.id)
        .bind(user_id)
        .fetch_one(pool)
        .await?;
    if by_user >= i64::from(coupon.per_user_limit) {
        return Err(QuoteError::CouponAlreadyUsed);
    }
    Ok(())
}

/// What `months` months of `plan_id` cost `user_id`, with `code` (a coupon or someone's referral
/// code, matched case-insensitively) if there is one.
pub async fn quote(pool: &PgPool, user_id: Uuid, plan_id: Uuid, months: i32, code: Option<&str>, cfg: &BillingConfig) -> Result<Quote, QuoteError> {
    let now = Utc::now();
    let plan: PlanRow = plans::get_plan(pool, plan_id).await?.filter(PlanRow::for_sale).ok_or(QuoteError::PlanUnavailable)?;
    let term: Term = sqlx::query_as(&format!("SELECT {TERM_COLUMNS} FROM billing_terms WHERE months = $1 AND is_active"))
        .bind(months)
        .fetch_optional(pool)
        .await?
        .ok_or(QuoteError::UnknownTerm)?;

    let mut coupon_discount = None;
    let mut referral_percent = None;
    let mut coupon_code = None;
    let mut coupon_id = None;
    let mut referral_code = None;
    if let Some(code) = normalize_code(code) {
        let coupon: Option<Coupon> = sqlx::query_as(&format!("SELECT {COUPON_COLUMNS} FROM coupons WHERE code = $1")).bind(&code).fetch_optional(pool).await?;
        if let Some(coupon) = coupon {
            check_coupon(pool, &coupon, user_id, plan_id, months, now).await?;
            coupon_discount = Some(coupon.discount());
            coupon_code = Some(coupon.code.clone());
            coupon_id = Some(coupon.id);
        } else {
            let referrer: Uuid =
                sqlx::query_scalar("SELECT user_id FROM referral_codes WHERE code = $1").bind(&code).fetch_optional(pool).await?.ok_or(QuoteError::UnknownCode)?;
            if referrer == user_id {
                return Err(QuoteError::OwnReferralCode);
            }
            let referred: bool = sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM referrals WHERE referee_user_id = $1)").bind(user_id).fetch_one(pool).await?;
            if referred || has_paid(pool, user_id).await? {
                return Err(QuoteError::ReferralFirstPaymentOnly);
            }
            referral_percent = Some(cfg.referee_discount_percent);
            referral_code = Some(code);
        }
    }

    let b = price(plan.price_amount, plan.active_promo_price(now), months, i64::from(term.discount_percent), coupon_discount, referral_percent);
    Ok(Quote {
        plan_id,
        term_months: months,
        currency: plan.currency,
        monthly_price: plan.price_amount,
        base_amount: b.base_amount,
        promo_discount: b.promo_discount,
        term_discount: b.term_discount,
        coupon_discount: b.coupon_discount,
        referral_discount: b.referral_discount,
        amount: b.amount,
        coupon_code,
        referral_code,
        coupon_id,
    })
}

// ---------------------------------------------------------------------------------------------
// Payments

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Payment {
    pub id: Uuid,
    pub user_id: Uuid,
    pub plan_id: Uuid,
    pub term_months: i32,
    pub kind: String,
    pub base_amount: i64,
    pub term_discount: i64,
    pub promo_discount: i64,
    pub coupon_discount: i64,
    pub referral_discount: i64,
    pub amount: i64,
    pub currency: String,
    pub coupon_id: Option<Uuid>,
    pub referral_code: Option<String>,
    pub status: String,
    pub provider: String,
    pub provider_invoice_id: Option<String>,
    pub invoice_url: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub paid_at: Option<DateTime<Utc>>,
    pub period_start: Option<DateTime<Utc>>,
    pub period_end: Option<DateTime<Utc>>,
    pub payment_method: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Always selected `FROM payments p`.
const PAYMENT_COLUMNS: &str = "p.id, p.user_id, p.plan_id, p.term_months, p.kind, p.base_amount, p.term_discount, p.promo_discount, \
     p.coupon_discount, p.referral_discount, p.amount, p.currency, p.coupon_id, p.referral_code, p.status, p.provider, \
     p.provider_invoice_id, p.invoice_url, p.expires_at, p.paid_at, p.period_start, p.period_end, p.payment_method, \
     p.created_at, p.updated_at";

/// A payment with the plan's name and who paid, for lists.
#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct PaymentListItem {
    #[sqlx(flatten)]
    #[serde(flatten)]
    pub payment: Payment,
    pub plan_name: String,
    pub user_email: Option<String>,
}

const LIST_FROM: &str = "FROM payments p JOIN plans pl ON pl.id = p.plan_id \
     LEFT JOIN web_credentials w ON w.user_id = p.user_id LEFT JOIN google_identities g ON g.user_id = p.user_id";

#[derive(Debug, Serialize)]
pub struct PaymentPage {
    pub items: Vec<PaymentListItem>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
}

pub async fn get_payment(pool: &PgPool, id: Uuid) -> Result<Option<Payment>, sqlx::Error> {
    sqlx::query_as(&format!("SELECT {PAYMENT_COLUMNS} FROM payments p WHERE p.id = $1")).bind(id).fetch_optional(pool).await
}

/// One payment as the lists show it.
pub async fn payment_item(pool: &PgPool, id: Uuid) -> Result<Option<PaymentListItem>, sqlx::Error> {
    sqlx::query_as(&format!(
        "SELECT {PAYMENT_COLUMNS}, pl.name AS plan_name, COALESCE(w.email, g.email) AS user_email {LIST_FROM} WHERE p.id = $1"
    ))
    .bind(id)
    .fetch_optional(pool)
    .await
}

/// The payment `external_id` (a payment id as text) names, if there is one.
pub async fn payment_by_external_id(pool: &PgPool, external_id: &str) -> Result<Option<Payment>, sqlx::Error> {
    match Uuid::parse_str(external_id) {
        Ok(id) => get_payment(pool, id).await,
        Err(_) => Ok(None),
    }
}

pub async fn list_payments_for_user(pool: &PgPool, user_id: Uuid) -> Result<Vec<PaymentListItem>, sqlx::Error> {
    sqlx::query_as(&format!(
        "SELECT {PAYMENT_COLUMNS}, pl.name AS plan_name, COALESCE(w.email, g.email) AS user_email {LIST_FROM} \
         WHERE p.user_id = $1 ORDER BY p.created_at DESC LIMIT $2"
    ))
    .bind(user_id)
    .bind(USER_PAYMENTS_LIMIT)
    .fetch_all(pool)
    .await
}

/// Everyone's payments, newest first, `page` counted from 1.
pub async fn list_payments_for_admin(pool: &PgPool, status: Option<&str>, page: i64) -> Result<PaymentPage, sqlx::Error> {
    let page = page.max(1);
    let items = sqlx::query_as(&format!(
        "SELECT {PAYMENT_COLUMNS}, pl.name AS plan_name, COALESCE(w.email, g.email) AS user_email {LIST_FROM} \
         WHERE ($1::text IS NULL OR p.status = $1) ORDER BY p.created_at DESC LIMIT $2 OFFSET $3"
    ))
    .bind(status)
    .bind(PAGE_SIZE)
    .bind((page - 1) * PAGE_SIZE)
    .fetch_all(pool)
    .await?;
    let total: i64 = sqlx::query_scalar("SELECT count(*) FROM payments p WHERE ($1::text IS NULL OR p.status = $1)").bind(status).fetch_one(pool).await?;
    Ok(PaymentPage { items, total, page, page_size: PAGE_SIZE })
}

/// A payment of theirs that is still waiting to be paid for the same plan, term and code.
pub async fn reusable_payment(pool: &PgPool, user_id: Uuid, plan_id: Uuid, months: i32, code: Option<&str>) -> Result<Option<Payment>, sqlx::Error> {
    sqlx::query_as(&format!(
        "SELECT {PAYMENT_COLUMNS} FROM payments p LEFT JOIN coupons c ON c.id = p.coupon_id \
         WHERE p.user_id = $1 AND p.plan_id = $2 AND p.term_months = $3 AND p.status = 'pending' \
           AND p.invoice_url IS NOT NULL AND (p.expires_at IS NULL OR p.expires_at > now()) \
           AND COALESCE(c.code, p.referral_code, '') = $4 \
         ORDER BY p.created_at DESC LIMIT 1"
    ))
    .bind(user_id)
    .bind(plan_id)
    .bind(months)
    .bind(code.unwrap_or_default())
    .fetch_optional(pool)
    .await
}

/// Their renewal invoice that is still waiting to be paid, if any.
pub async fn pending_renewal(pool: &PgPool, user_id: Uuid) -> Result<Option<Payment>, sqlx::Error> {
    sqlx::query_as(&format!(
        "SELECT {PAYMENT_COLUMNS} FROM payments p WHERE p.user_id = $1 AND p.kind = 'renewal' AND p.status = 'pending' \
           AND (p.expires_at IS NULL OR p.expires_at > now()) ORDER BY p.created_at DESC LIMIT 1"
    ))
    .bind(user_id)
    .fetch_optional(pool)
    .await
}

/// Referral rewards (days) waiting for their next paid period.
pub async fn bonus_days(pool: &PgPool, user_id: Uuid) -> Result<i32, sqlx::Error> {
    Ok(sqlx::query_scalar("SELECT bonus_days FROM user_subscriptions WHERE user_id = $1").bind(user_id).fetch_optional(pool).await?.unwrap_or(0))
}

/// Saves their choice of renewing by themselves; a person with no subscription row gets one on the default plan.
pub async fn set_auto_renew(pool: &PgPool, user_id: Uuid, auto_renew: bool) -> Result<(), sqlx::Error> {
    let default = plans::default_plan(pool).await?;
    sqlx::query(
        "INSERT INTO user_subscriptions (user_id, plan_id, auto_renew) VALUES ($1, $2, $3) \
         ON CONFLICT (user_id) DO UPDATE SET auto_renew = $3, updated_at = now()",
    )
    .bind(user_id)
    .bind(default.id)
    .bind(auto_renew)
    .execute(pool)
    .await?;
    Ok(())
}

/// Makes the pending payment for `quote`, counting its coupon (a cap on a coupon is checked
/// again here, under a lock, so two people can't both take the last use).
pub async fn create_payment(pool: &PgPool, user_id: Uuid, quote: &Quote, kind: &str, invoice_hours: u32) -> Result<Payment, QuoteError> {
    let mut tx = pool.begin().await?;
    if let Some(coupon_id) = quote.coupon_id {
        let (max, per_user): (Option<i32>, i32) =
            sqlx::query_as("SELECT max_redemptions, per_user_limit FROM coupons WHERE id = $1 FOR UPDATE").bind(coupon_id).fetch_one(&mut *tx).await?;
        let total: i64 = sqlx::query_scalar("SELECT count(*) FROM coupon_redemptions WHERE coupon_id = $1").bind(coupon_id).fetch_one(&mut *tx).await?;
        if max.is_some_and(|max| total >= i64::from(max)) {
            return Err(QuoteError::CouponUsedUp);
        }
        let by_user: i64 =
            sqlx::query_scalar("SELECT count(*) FROM coupon_redemptions WHERE coupon_id = $1 AND user_id = $2").bind(coupon_id).bind(user_id).fetch_one(&mut *tx).await?;
        if by_user >= i64::from(per_user) {
            return Err(QuoteError::CouponAlreadyUsed);
        }
    }
    let expires_at = Utc::now() + Duration::hours(i64::from(invoice_hours));
    let id: Uuid = sqlx::query_scalar(
        "INSERT INTO payments (user_id, plan_id, term_months, kind, base_amount, term_discount, promo_discount, coupon_discount, \
                               referral_discount, amount, currency, coupon_id, referral_code, status, expires_at) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, 'pending', $14) RETURNING id",
    )
    .bind(user_id)
    .bind(quote.plan_id)
    .bind(quote.term_months)
    .bind(kind)
    .bind(quote.base_amount)
    .bind(quote.term_discount)
    .bind(quote.promo_discount)
    .bind(quote.coupon_discount)
    .bind(quote.referral_discount)
    .bind(quote.amount)
    .bind(&quote.currency)
    .bind(quote.coupon_id)
    .bind(&quote.referral_code)
    .bind(expires_at)
    .fetch_one(&mut *tx)
    .await?;
    if let Some(coupon_id) = quote.coupon_id {
        sqlx::query("INSERT INTO coupon_redemptions (coupon_id, user_id, payment_id) VALUES ($1, $2, $3)")
            .bind(coupon_id)
            .bind(user_id)
            .bind(id)
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await?;
    Ok(get_payment(pool, id).await?.ok_or(sqlx::Error::RowNotFound)?)
}

/// Saves the invoice the provider made for a payment.
pub async fn attach_invoice(pool: &PgPool, payment_id: Uuid, invoice_id: &str, invoice_url: &str, expires_at: Option<DateTime<Utc>>) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE payments SET provider_invoice_id = $2, invoice_url = $3, expires_at = COALESCE($4, expires_at), updated_at = now() WHERE id = $1",
    )
    .bind(payment_id)
    .bind(invoice_id)
    .bind(invoice_url)
    .bind(expires_at)
    .execute(pool)
    .await?;
    Ok(())
}

/// Ends a pending payment as expired, failed or cancelled, and gives its coupon back. `false`
/// when it wasn't pending (so nothing changed).
pub async fn release(pool: &PgPool, payment_id: Uuid, status: &str) -> Result<bool, sqlx::Error> {
    let mut tx = pool.begin().await?;
    let changed = sqlx::query("UPDATE payments SET status = $2, updated_at = now() WHERE id = $1 AND status = 'pending'")
        .bind(payment_id)
        .bind(status)
        .execute(&mut *tx)
        .await?
        .rows_affected();
    if changed > 0 {
        sqlx::query("DELETE FROM coupon_redemptions WHERE payment_id = $1").bind(payment_id).execute(&mut *tx).await?;
    }
    tx.commit().await?;
    Ok(changed > 0)
}

/// What activating a payment did.
#[derive(Debug, Clone)]
pub struct Activation {
    pub payment: Payment,
    pub paid_until: DateTime<Utc>,
    /// Who was rewarded for referring them, if this payment made that happen.
    pub referrer_id: Option<Uuid>,
}

fn add_months(from: DateTime<Utc>, months: i32) -> DateTime<Utc> {
    let months = u32::try_from(months).unwrap_or(0);
    from.checked_add_months(Months::new(months)).unwrap_or(from)
}

/// The words for what was paid, in the change history: "Paid: 6 months".
fn paid_note(months: i32) -> String {
    if months == 1 {
        "Paid: 1 month".to_string()
    } else {
        format!("Paid: {months} months")
    }
}

/// Puts the person on the payment's plan for the months paid (on top of what's left of the same
/// plan, plus any referral bonus days waiting), and marks the payment paid. Does nothing, and
/// returns `None`, for a payment that is already paid. If it came with a referral code and they
/// weren't referred yet, the referrer is rewarded.
pub async fn activate(
    conn: &mut PgConnection,
    payment_id: Uuid,
    paid_at: Option<DateTime<Utc>>,
    payment_method: Option<&str>,
    cfg: &BillingConfig,
) -> Result<Option<Activation>, sqlx::Error> {
    let payment: Payment =
        sqlx::query_as(&format!("SELECT {PAYMENT_COLUMNS} FROM payments p WHERE p.id = $1 FOR UPDATE OF p")).bind(payment_id).fetch_one(&mut *conn).await?;
    if payment.status == STATUS_PAID {
        return Ok(None);
    }
    let now = Utc::now();
    let current: Option<(Uuid, Option<DateTime<Utc>>, i32)> =
        sqlx::query_as("SELECT plan_id, paid_until, bonus_days FROM user_subscriptions WHERE user_id = $1 FOR UPDATE").bind(payment.user_id).fetch_optional(&mut *conn).await?;
    let (still_running, bonus_days) = match current {
        Some((plan_id, until, bonus)) => (until.filter(|until| plan_id == payment.plan_id && *until > now), bonus),
        None => (None, 0),
    };
    let period_start = still_running.unwrap_or(now);
    let period_end = add_months(period_start, payment.term_months) + Duration::days(i64::from(bonus_days));

    sqlx::query(
        "INSERT INTO user_subscriptions (user_id, plan_id, term_months, paid_until, bonus_days) VALUES ($1, $2, $3, $4, 0) \
         ON CONFLICT (user_id) DO UPDATE SET plan_id = $2, term_months = $3, paid_until = $4, bonus_days = 0, updated_at = now()",
    )
    .bind(payment.user_id)
    .bind(payment.plan_id)
    .bind(payment.term_months)
    .bind(period_end)
    .execute(&mut *conn)
    .await?;
    sqlx::query("INSERT INTO subscription_changes (user_id, plan_id, note) VALUES ($1, $2, $3)")
        .bind(payment.user_id)
        .bind(payment.plan_id)
        .bind(paid_note(payment.term_months))
        .execute(&mut *conn)
        .await?;
    sqlx::query(
        "UPDATE payments SET status = 'paid', paid_at = COALESCE($2, now()), period_start = $3, period_end = $4, \
             payment_method = COALESCE($5, payment_method), updated_at = now() WHERE id = $1",
    )
    .bind(payment_id)
    .bind(paid_at)
    .bind(period_start)
    .bind(period_end)
    .bind(payment_method)
    .execute(&mut *conn)
    .await?;
    // A payment that was released (expired) and then paid anyway still counts against its coupon.
    if let Some(coupon_id) = payment.coupon_id {
        sqlx::query("INSERT INTO coupon_redemptions (coupon_id, user_id, payment_id) VALUES ($1, $2, $3) ON CONFLICT (payment_id) DO NOTHING")
            .bind(coupon_id)
            .bind(payment.user_id)
            .bind(payment_id)
            .execute(&mut *conn)
            .await?;
    }
    let referrer_id = reward_referrer(&mut *conn, &payment, cfg, now).await?;

    let payment: Payment = sqlx::query_as(&format!("SELECT {PAYMENT_COLUMNS} FROM payments p WHERE p.id = $1")).bind(payment_id).fetch_one(&mut *conn).await?;
    Ok(Some(Activation { payment, paid_until: period_end, referrer_id }))
}

/// Records that the payer was referred and rewards the referrer, once per payer.
async fn reward_referrer(conn: &mut PgConnection, payment: &Payment, cfg: &BillingConfig, now: DateTime<Utc>) -> Result<Option<Uuid>, sqlx::Error> {
    let Some(code) = payment.referral_code.as_deref() else { return Ok(None) };
    let referrer: Option<Uuid> = sqlx::query_scalar("SELECT user_id FROM referral_codes WHERE code = $1").bind(code).fetch_optional(&mut *conn).await?;
    let Some(referrer) = referrer.filter(|referrer| *referrer != payment.user_id) else { return Ok(None) };
    let recorded = sqlx::query(
        "INSERT INTO referrals (referee_user_id, referrer_user_id, payment_id, rewarded_at) VALUES ($1, $2, $3, now()) ON CONFLICT (referee_user_id) DO NOTHING",
    )
    .bind(payment.user_id)
    .bind(referrer)
    .bind(payment.id)
    .execute(&mut *conn)
    .await?
    .rows_affected();
    if recorded == 0 {
        return Ok(None);
    }
    // An active paid period is extended; otherwise the days wait for their next one.
    let extended = sqlx::query("UPDATE user_subscriptions SET paid_until = paid_until + make_interval(days => $2), updated_at = now() WHERE user_id = $1 AND paid_until > $3")
        .bind(referrer)
        .bind(cfg.referrer_bonus_days)
        .bind(now)
        .execute(&mut *conn)
        .await?
        .rows_affected();
    if extended == 0 {
        let default = plans::default_plan_in(&mut *conn).await?;
        sqlx::query(
            "INSERT INTO user_subscriptions (user_id, plan_id, bonus_days) VALUES ($1, $2, $3) \
             ON CONFLICT (user_id) DO UPDATE SET bonus_days = user_subscriptions.bonus_days + $3, updated_at = now()",
        )
        .bind(referrer)
        .bind(default.id)
        .bind(cfg.referrer_bonus_days)
        .execute(&mut *conn)
        .await?;
    }
    Ok(Some(referrer))
}

// ---------------------------------------------------------------------------------------------
// Renewals

/// A paid period close to its end (or just past it, within the grace).
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct RenewalCandidate {
    pub user_id: Uuid,
    pub plan_id: Uuid,
    pub plan_name: String,
    pub term_months: i32,
    pub paid_until: DateTime<Utc>,
    pub auto_renew: bool,
}

/// Paid periods that end within `cfg.renewal_days_before` days (or ended less than the grace ago)
/// on a plan that is still for sale.
pub async fn renewal_candidates(pool: &PgPool, now: DateTime<Utc>, cfg: &BillingConfig) -> Result<Vec<RenewalCandidate>, sqlx::Error> {
    sqlx::query_as(
        "SELECT s.user_id, s.plan_id, p.name AS plan_name, s.term_months, s.paid_until, s.auto_renew \
         FROM user_subscriptions s JOIN plans p ON p.id = s.plan_id \
         WHERE s.paid_until IS NOT NULL AND s.term_months IS NOT NULL AND NOT p.is_default AND p.is_active AND p.price_amount > 0 \
           AND s.paid_until <= $1 + make_interval(days => $2) AND s.paid_until + make_interval(days => $3) > $1 \
         ORDER BY s.paid_until",
    )
    .bind(now)
    .bind(cfg.renewal_days_before)
    .bind(cfg.grace_days)
    .fetch_all(pool)
    .await
}

/// A paid period (and its grace) that ran out.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Lapsed {
    pub user_id: Uuid,
    pub plan_name: String,
    pub paid_until: DateTime<Utc>,
}

/// Lapsed paid periods nobody was told about yet.
pub async fn lapsed_unannounced(pool: &PgPool, now: DateTime<Utc>, cfg: &BillingConfig) -> Result<Vec<Lapsed>, sqlx::Error> {
    sqlx::query_as(
        "SELECT s.user_id, p.name AS plan_name, s.paid_until \
         FROM user_subscriptions s JOIN plans p ON p.id = s.plan_id \
         WHERE s.paid_until IS NOT NULL AND s.paid_until + make_interval(days => $2) <= $1 \
           AND NOT EXISTS (SELECT 1 FROM billing_notices n WHERE n.user_id = s.user_id AND n.kind = 'ended' AND n.paid_until = s.paid_until) \
         ORDER BY s.paid_until",
    )
    .bind(now)
    .bind(cfg.grace_days)
    .fetch_all(pool)
    .await
}

/// Takes the right to send `kind` for this paid period: `true` the first time, `false` after.
pub async fn claim_notice(pool: &PgPool, user_id: Uuid, kind: &str, paid_until: DateTime<Utc>) -> Result<bool, sqlx::Error> {
    Ok(sqlx::query("INSERT INTO billing_notices (user_id, kind, paid_until) VALUES ($1, $2, $3) ON CONFLICT DO NOTHING")
        .bind(user_id)
        .bind(kind)
        .bind(paid_until)
        .execute(pool)
        .await?
        .rows_affected()
        == 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    const MONTHLY: i64 = 50_000;

    #[test]
    fn one_month_at_list_price() {
        let b = price(MONTHLY, None, 1, 0, None, None);
        assert_eq!(b, Breakdown { base_amount: 50_000, promo_discount: 0, term_discount: 0, coupon_discount: 0, referral_discount: 0, amount: 50_000 });
    }

    #[test]
    fn a_term_takes_its_percent_off() {
        let b = price(MONTHLY, None, 6, 10, None, None);
        assert_eq!((b.base_amount, b.term_discount, b.amount), (300_000, 30_000, 270_000));
        let b = price(MONTHLY, None, 12, 20, None, None);
        assert_eq!((b.base_amount, b.term_discount, b.amount), (600_000, 120_000, 480_000));
    }

    #[test]
    fn the_promo_price_comes_off_before_the_term() {
        // 12 months at 40,000 instead of 50,000: 120,000 off, then 20% of the remaining 480,000.
        let b = price(MONTHLY, Some(40_000), 12, 20, None, None);
        assert_eq!((b.base_amount, b.promo_discount, b.term_discount, b.amount), (600_000, 120_000, 96_000, 384_000));
    }

    #[test]
    fn a_promo_above_the_list_price_does_nothing() {
        let b = price(MONTHLY, Some(60_000), 3, 0, None, None);
        assert_eq!((b.promo_discount, b.amount), (0, 150_000));
    }

    #[test]
    fn a_free_promo_makes_it_free() {
        let b = price(MONTHLY, Some(0), 2, 10, Some(CouponDiscount::Amount(5_000)), Some(10));
        assert_eq!((b.promo_discount, b.term_discount, b.coupon_discount, b.referral_discount, b.amount), (100_000, 0, 0, 0, 0));
    }

    #[test]
    fn a_percent_coupon_is_taken_from_what_is_left() {
        // 6 months: 300,000 - 10% = 270,000; 25% of that is 67,500.
        let b = price(MONTHLY, None, 6, 10, Some(CouponDiscount::Percent(25)), None);
        assert_eq!((b.coupon_discount, b.amount), (67_500, 202_500));
    }

    #[test]
    fn a_fixed_coupon_is_capped_at_what_is_left() {
        let b = price(MONTHLY, None, 1, 0, Some(CouponDiscount::Amount(20_000)), None);
        assert_eq!((b.coupon_discount, b.amount), (20_000, 30_000));
        let b = price(MONTHLY, None, 1, 0, Some(CouponDiscount::Amount(80_000)), None);
        assert_eq!((b.coupon_discount, b.amount), (50_000, 0));
    }

    #[test]
    fn the_referral_percent_comes_last() {
        // 12 months: 600,000 - 20% = 480,000; coupon 10% = 48,000 -> 432,000; referral 10% = 43,200.
        let b = price(MONTHLY, None, 12, 20, Some(CouponDiscount::Percent(10)), Some(10));
        assert_eq!(b, Breakdown { base_amount: 600_000, promo_discount: 0, term_discount: 120_000, coupon_discount: 48_000, referral_discount: 43_200, amount: 388_800 });
    }

    #[test]
    fn everything_at_once() {
        // 6 months at 50,000 with a 45,000 promo: 300,000 - 30,000 = 270,000; term 10% = 27,000 -> 243,000;
        // coupon 5,000 -> 238,000; referral 10% = 23,800 -> 214,200.
        let b = price(MONTHLY, Some(45_000), 6, 10, Some(CouponDiscount::Amount(5_000)), Some(10));
        assert_eq!(b, Breakdown { base_amount: 300_000, promo_discount: 30_000, term_discount: 27_000, coupon_discount: 5_000, referral_discount: 23_800, amount: 214_200 });
        assert_eq!(b.base_amount - b.promo_discount - b.term_discount - b.coupon_discount - b.referral_discount, b.amount);
    }

    #[test]
    fn discounts_round_down() {
        // 10% of 333 is 33.3 -> 33.
        let b = price(111, None, 3, 10, None, None);
        assert_eq!((b.base_amount, b.term_discount, b.amount), (333, 33, 300));
        // 33% of 100 is 33; 33% of 67 is 22.11 -> 22.
        let b = price(100, None, 1, 33, None, Some(33));
        assert_eq!((b.term_discount, b.referral_discount, b.amount), (33, 22, 45));
    }

    #[test]
    fn the_amount_never_goes_below_zero() {
        let b = price(MONTHLY, None, 1, 100, Some(CouponDiscount::Percent(100)), Some(100));
        assert_eq!(b.amount, 0);
        let b = price(MONTHLY, None, 1, 150, Some(CouponDiscount::Amount(-5)), Some(-5));
        assert_eq!((b.term_discount, b.coupon_discount, b.referral_discount, b.amount), (50_000, 0, 0, 0));
        assert_eq!(price(0, None, 12, 0, None, None).amount, 0);
    }

    #[test]
    fn a_huge_price_does_not_overflow() {
        let b = price(i64::MAX / 2, None, 12, 50, None, None);
        assert!(b.amount >= 0);
        assert!(b.term_discount > 0);
    }

    #[test]
    fn lapsing_waits_for_the_grace() {
        let until = Utc::now();
        assert!(!is_lapsed(until, until + Duration::days(2), 3));
        assert!(is_lapsed(until, until + Duration::days(3), 3));
        assert!(!is_lapsed(until, until - Duration::days(1), 0));
        assert!(is_lapsed(until, until, 0));
    }

    #[test]
    fn the_renewal_window_opens_days_before_the_end() {
        let now = Utc::now();
        assert!(!in_renewal_window(now + Duration::days(8), now, 7));
        assert!(in_renewal_window(now + Duration::days(7), now, 7));
        assert!(in_renewal_window(now - Duration::days(1), now, 7));
    }

    #[test]
    fn coupon_input_is_trimmed_upper_cased_and_checked() {
        let input = || CouponInput {
            code: " spring-25 ".into(),
            description: Some(" ".into()),
            discount_kind: DISCOUNT_PERCENT.into(),
            discount_value: 20,
            max_redemptions: None,
            per_user_limit: 1,
            starts_at: None,
            ends_at: None,
            plan_ids: vec![],
            min_months: 1,
            first_payment_only: false,
            is_active: true,
        };
        let ok = input().validate().unwrap();
        assert_eq!((ok.code.as_str(), ok.description), ("SPRING-25", None));
        assert!(CouponInput { code: "no spaces".into(), ..input() }.validate().is_err());
        assert!(CouponInput { discount_value: 101, ..input() }.validate().is_err());
        assert!(CouponInput { discount_value: 0, ..input() }.validate().is_err());
        assert!(CouponInput { discount_kind: DISCOUNT_AMOUNT.into(), discount_value: 5_000, ..input() }.validate().is_ok());
        assert!(CouponInput { discount_kind: "free".into(), ..input() }.validate().is_err());
        assert!(CouponInput { max_redemptions: Some(0), ..input() }.validate().is_err());
        assert!(CouponInput { min_months: 0, ..input() }.validate().is_err());
        let now = Utc::now();
        assert!(CouponInput { starts_at: Some(now), ends_at: Some(now), ..input() }.validate().is_err());
    }

    #[test]
    fn term_input_is_checked() {
        let term = |months, percent| TermInput { months, discount_percent: percent, label: " x ".into(), is_active: true, sort_order: 0 };
        assert_eq!(term(6, 10).validate().unwrap().label, "x");
        assert!(term(0, 10).validate().is_err());
        assert!(term(6, 101).validate().is_err());
        assert!(term(6, -1).validate().is_err());
    }

    #[test]
    fn referral_codes_are_eight_upper_case_characters() {
        for _ in 0..50 {
            let code = random_referral_code();
            assert_eq!(code.len(), REFERRAL_CODE_LEN);
            assert!(code.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit()), "{code}");
        }
    }

    #[test]
    fn codes_are_matched_trimmed_and_upper_case() {
        assert_eq!(normalize_code(Some(" abc12 ")).as_deref(), Some("ABC12"));
        assert_eq!(normalize_code(Some("  ")), None);
        assert_eq!(normalize_code(None), None);
    }

    #[test]
    fn the_settings_fall_back_to_their_defaults() {
        assert_eq!(BillingConfig::from_section(&Section::default()), BillingConfig::default());
    }

    #[test]
    fn what_was_paid_is_worded_for_the_history() {
        assert_eq!(paid_note(1), "Paid: 1 month");
        assert_eq!(paid_note(6), "Paid: 6 months");
    }

    #[test]
    fn months_are_calendar_months() {
        let jan_31 = DateTime::parse_from_rfc3339("2026-01-31T10:00:00Z").unwrap().with_timezone(&Utc);
        assert_eq!(add_months(jan_31, 1).to_rfc3339(), "2026-02-28T10:00:00+00:00");
        assert_eq!(add_months(jan_31, 12).to_rfc3339(), "2027-01-31T10:00:00+00:00");
    }
}
