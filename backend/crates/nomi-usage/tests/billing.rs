use chrono::{DateTime, Duration, Months, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use nomi_usage::billing::{self, BillingConfig, CouponInput, Quote, QuoteError, KIND_NEW, STATUS_EXPIRED, STATUS_PAID};
use nomi_usage::plans::{self, PlanInput, PlanRow, SubscriptionChange};

const PRICE: i64 = 50_000;
const BONUS_DAYS: i32 = 30;
const SLACK_SECONDS: i64 = 60;
const INVOICE_HOURS: u32 = 24;

async fn person(pool: &PgPool) -> Uuid {
    sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(pool).await.unwrap()
}

fn plan_input(slug: &str, extra: serde_json::Value) -> PlanInput {
    let mut base = serde_json::json!({ "slug": slug, "name": slug, "monthly_tokens": 100, "price_idr": PRICE });
    base.as_object_mut().unwrap().extend(extra.as_object().unwrap().clone());
    serde_json::from_value::<PlanInput>(base).unwrap().validate().unwrap()
}

async fn paid_plan(pool: &PgPool, slug: &str) -> PlanRow {
    plans::create_plan(pool, &plan_input(slug, serde_json::json!({}))).await.unwrap()
}

async fn coupon(pool: &PgPool, code: &str, extra: serde_json::Value) -> billing::Coupon {
    let mut base = serde_json::json!({ "code": code, "discount_kind": "percent", "discount_value": 10 });
    base.as_object_mut().unwrap().extend(extra.as_object().unwrap().clone());
    let input: CouponInput = serde_json::from_value::<CouponInput>(base).unwrap().validate().unwrap();
    billing::create_coupon(pool, &input, person(pool).await).await.unwrap()
}

fn cfg() -> BillingConfig {
    BillingConfig::default()
}

async fn quote(pool: &PgPool, user: Uuid, plan: &PlanRow, months: i32, code: Option<&str>) -> Result<Quote, QuoteError> {
    billing::quote(pool, user, plan.id, months, code, &cfg()).await
}

/// A pending payment for `months` of `plan`, then activated.
async fn pay(pool: &PgPool, user: Uuid, plan: &PlanRow, months: i32, code: Option<&str>) -> billing::Activation {
    let q = quote(pool, user, plan, months, code).await.unwrap();
    let payment = billing::create_payment(pool, user, &q, KIND_NEW, INVOICE_HOURS).await.unwrap();
    let mut tx = pool.begin().await.unwrap();
    let activation = billing::activate(&mut tx, payment.id, None, Some("QRIS"), &cfg()).await.unwrap().unwrap();
    tx.commit().await.unwrap();
    activation
}

fn near(a: DateTime<Utc>, b: DateTime<Utc>) -> bool {
    (a - b).num_seconds().abs() < SLACK_SECONDS
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_quote_applies_the_term_and_the_promo(pool: PgPool) {
    let user = person(&pool).await;
    let plan = paid_plan(&pool, "plus").await;
    let q = quote(&pool, user, &plan, 6, None).await.unwrap();
    assert_eq!((q.base_amount, q.term_discount, q.amount, q.currency.as_str()), (300_000, 30_000, 270_000, "IDR"));

    let promo = plans::create_plan(&pool, &plan_input("promo", serde_json::json!({ "promo_price_idr": 40_000, "promo_ends_at": Utc::now() + Duration::days(1) }))).await.unwrap();
    let q = quote(&pool, user, &promo, 12, None).await.unwrap();
    assert_eq!((q.promo_discount, q.term_discount, q.amount), (120_000, 96_000, 384_000));

    let over = plans::create_plan(&pool, &plan_input("over", serde_json::json!({ "promo_price_idr": 40_000, "promo_ends_at": Utc::now() - Duration::days(1) }))).await.unwrap();
    assert_eq!(quote(&pool, user, &over, 1, None).await.unwrap().promo_discount, 0);
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_quote_refuses_what_cant_be_bought(pool: PgPool) {
    let user = person(&pool).await;
    let free = plans::default_plan(&pool).await.unwrap();
    assert!(matches!(quote(&pool, user, &free, 1, None).await, Err(QuoteError::PlanUnavailable)));
    let hidden = plans::create_plan(&pool, &plan_input("hidden", serde_json::json!({ "is_active": false }))).await.unwrap();
    assert!(matches!(quote(&pool, user, &hidden, 1, None).await, Err(QuoteError::PlanUnavailable)));
    let plan = paid_plan(&pool, "plus").await;
    assert!(matches!(quote(&pool, user, &plan, 5, None).await, Err(QuoteError::UnknownTerm)));
    sqlx::query("UPDATE billing_terms SET is_active = false WHERE months = 6").execute(&pool).await.unwrap();
    assert!(matches!(quote(&pool, user, &plan, 6, None).await, Err(QuoteError::UnknownTerm)));
    assert!(matches!(billing::quote(&pool, user, Uuid::new_v4(), 1, None, &cfg()).await, Err(QuoteError::PlanUnavailable)));
    assert!(matches!(quote(&pool, user, &plan, 1, Some("NOPE")).await, Err(QuoteError::UnknownCode)));
}

#[sqlx::test(migrations = "../../migrations")]
async fn coupons_are_checked_against_the_plan_the_months_and_the_dates(pool: PgPool) {
    let user = person(&pool).await;
    let plan = paid_plan(&pool, "plus").await;
    let other = paid_plan(&pool, "team").await;
    coupon(&pool, "OFF", serde_json::json!({ "discount_value": 20 })).await;
    // Matched without regard to case or spaces.
    let q = quote(&pool, user, &plan, 1, Some("  off ")).await.unwrap();
    assert_eq!((q.coupon_code.as_deref(), q.coupon_discount, q.amount), (Some("OFF"), 10_000, 40_000));

    coupon(&pool, "OFFLINE", serde_json::json!({ "is_active": false })).await;
    assert!(matches!(quote(&pool, user, &plan, 1, Some("OFFLINE")).await, Err(QuoteError::CouponInactive)));
    coupon(&pool, "SOON", serde_json::json!({ "starts_at": Utc::now() + Duration::days(1) })).await;
    assert!(matches!(quote(&pool, user, &plan, 1, Some("SOON")).await, Err(QuoteError::CouponNotStarted)));
    coupon(&pool, "OLD", serde_json::json!({ "starts_at": Utc::now() - Duration::days(3), "ends_at": Utc::now() - Duration::days(1) })).await;
    assert!(matches!(quote(&pool, user, &plan, 1, Some("OLD")).await, Err(QuoteError::CouponEnded)));
    coupon(&pool, "ONLYPRO", serde_json::json!({ "plan_ids": [plan.id] })).await;
    assert!(quote(&pool, user, &plan, 1, Some("ONLYPRO")).await.is_ok());
    assert!(matches!(quote(&pool, user, &other, 1, Some("ONLYPRO")).await, Err(QuoteError::CouponWrongPlan)));
    coupon(&pool, "LONG", serde_json::json!({ "min_months": 6 })).await;
    assert!(matches!(quote(&pool, user, &plan, 1, Some("LONG")).await, Err(QuoteError::CouponMinMonths(6))));
    assert!(quote(&pool, user, &plan, 6, Some("LONG")).await.is_ok());
    let e = quote(&pool, user, &plan, 1, Some("LONG")).await.unwrap_err();
    assert_eq!(e.to_string(), "That coupon needs at least 6 months.");

    coupon(&pool, "FIRST", serde_json::json!({ "first_payment_only": true })).await;
    assert!(quote(&pool, user, &plan, 1, Some("FIRST")).await.is_ok());
    pay(&pool, user, &plan, 1, None).await;
    assert!(matches!(quote(&pool, user, &plan, 1, Some("FIRST")).await, Err(QuoteError::CouponFirstPaymentOnly)));
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_coupons_cap_counts_pending_payments_and_frees_up_when_one_expires(pool: PgPool) {
    let (a, b) = (person(&pool).await, person(&pool).await);
    let plan = paid_plan(&pool, "plus").await;
    coupon(&pool, "ONCE", serde_json::json!({ "max_redemptions": 1 })).await;

    let q = quote(&pool, a, &plan, 1, Some("ONCE")).await.unwrap();
    let payment = billing::create_payment(&pool, a, &q, KIND_NEW, INVOICE_HOURS).await.unwrap();
    // A pending payment holds the use.
    assert!(matches!(quote(&pool, b, &plan, 1, Some("ONCE")).await, Err(QuoteError::CouponUsedUp)));
    assert!(matches!(billing::create_payment(&pool, b, &q, KIND_NEW, INVOICE_HOURS).await, Err(QuoteError::CouponUsedUp)));
    // The one who holds it has used their own share too.
    assert!(matches!(quote(&pool, a, &plan, 1, Some("ONCE")).await, Err(QuoteError::CouponUsedUp)));

    assert!(billing::release(&pool, payment.id, STATUS_EXPIRED).await.unwrap());
    assert!(!billing::release(&pool, payment.id, STATUS_EXPIRED).await.unwrap(), "releasing twice changes nothing");
    assert!(quote(&pool, b, &plan, 1, Some("ONCE")).await.is_ok());
    let uses: i64 = sqlx::query_scalar("SELECT count(*) FROM coupon_redemptions").fetch_one(&pool).await.unwrap();
    assert_eq!(uses, 0);

    // A paid payment keeps it.
    pay(&pool, b, &plan, 1, Some("ONCE")).await;
    assert!(matches!(quote(&pool, a, &plan, 1, Some("ONCE")).await, Err(QuoteError::CouponUsedUp)));
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_coupon_has_a_limit_per_person(pool: PgPool) {
    let (a, b) = (person(&pool).await, person(&pool).await);
    let plan = paid_plan(&pool, "plus").await;
    coupon(&pool, "TWICE", serde_json::json!({ "per_user_limit": 2 })).await;
    for _ in 0..2 {
        pay(&pool, a, &plan, 1, Some("TWICE")).await;
    }
    assert!(matches!(quote(&pool, a, &plan, 1, Some("TWICE")).await, Err(QuoteError::CouponAlreadyUsed)));
    assert!(quote(&pool, b, &plan, 1, Some("TWICE")).await.is_ok());
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_redeemed_coupon_cant_be_deleted_but_an_unused_one_can(pool: PgPool) {
    let user = person(&pool).await;
    let plan = paid_plan(&pool, "plus").await;
    let used = coupon(&pool, "USED", serde_json::json!({})).await;
    let fresh = coupon(&pool, "FRESH", serde_json::json!({})).await;
    pay(&pool, user, &plan, 1, Some("USED")).await;
    assert_eq!(billing::delete_coupon(&pool, used.id).await.unwrap(), billing::DeleteCoupon::Redeemed);
    assert_eq!(billing::delete_coupon(&pool, fresh.id).await.unwrap(), billing::DeleteCoupon::Deleted);
    assert_eq!(billing::delete_coupon(&pool, fresh.id).await.unwrap(), billing::DeleteCoupon::NotFound);
    let listed = billing::list_coupons(&pool).await.unwrap();
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].redemptions, 1);
}

#[sqlx::test(migrations = "../../migrations")]
async fn activating_puts_them_on_the_plan_for_the_months_paid(pool: PgPool) {
    let user = person(&pool).await;
    let plan = paid_plan(&pool, "plus").await;
    let activation = pay(&pool, user, &plan, 6, None).await;
    let expected = Utc::now().checked_add_months(Months::new(6)).unwrap();
    assert!(near(activation.paid_until, expected), "{}", activation.paid_until);

    let sub = plans::subscription_for(&pool, user).await.unwrap();
    assert_eq!((sub.plan.id, sub.term_months, sub.lapsed, sub.auto_renew), (plan.id, Some(6), false, true));
    assert!(near(sub.paid_until.unwrap(), expected));
    let payment = billing::get_payment(&pool, activation.payment.id).await.unwrap().unwrap();
    assert_eq!((payment.status.as_str(), payment.payment_method.as_deref()), (STATUS_PAID, Some("QRIS")));
    assert!(payment.paid_at.is_some() && payment.period_start.is_some() && near(payment.period_end.unwrap(), expected));
    let history = plans::change_history(&pool, user).await.unwrap();
    assert_eq!(history[0].note.as_deref(), Some("Paid: 6 months"));
}

#[sqlx::test(migrations = "../../migrations")]
async fn activating_twice_changes_nothing(pool: PgPool) {
    let user = person(&pool).await;
    let plan = paid_plan(&pool, "plus").await;
    let first = pay(&pool, user, &plan, 1, None).await;
    let mut tx = pool.begin().await.unwrap();
    assert!(billing::activate(&mut tx, first.payment.id, None, None, &cfg()).await.unwrap().is_none());
    tx.commit().await.unwrap();
    let sub = plans::subscription_for(&pool, user).await.unwrap();
    assert_eq!(sub.paid_until, Some(first.paid_until));
    let changes: i64 = sqlx::query_scalar("SELECT count(*) FROM subscription_changes WHERE user_id = $1").bind(user).fetch_one(&pool).await.unwrap();
    assert_eq!(changes, 1);
}

#[sqlx::test(migrations = "../../migrations")]
async fn paying_again_extends_the_same_plan_but_a_new_plan_starts_now(pool: PgPool) {
    let user = person(&pool).await;
    let plan = paid_plan(&pool, "plus").await;
    let other = paid_plan(&pool, "team").await;
    let first = pay(&pool, user, &plan, 1, None).await;
    let second = pay(&pool, user, &plan, 6, None).await;
    assert_eq!(second.payment.period_start, Some(first.paid_until));
    assert_eq!(second.paid_until, first.paid_until.checked_add_months(Months::new(6)).unwrap());

    let switched = pay(&pool, user, &other, 1, None).await;
    assert!(near(switched.payment.period_start.unwrap(), Utc::now()));
    assert_eq!(plans::subscription_for(&pool, user).await.unwrap().plan.id, other.id);
}

#[sqlx::test(migrations = "../../migrations")]
async fn waiting_bonus_days_are_added_once(pool: PgPool) {
    let user = person(&pool).await;
    let plan = paid_plan(&pool, "plus").await;
    let free = plans::default_plan(&pool).await.unwrap();
    sqlx::query("INSERT INTO user_subscriptions (user_id, plan_id, bonus_days) VALUES ($1, $2, 10)").bind(user).bind(free.id).execute(&pool).await.unwrap();
    let activation = pay(&pool, user, &plan, 1, None).await;
    let expected = Utc::now().checked_add_months(Months::new(1)).unwrap() + Duration::days(10);
    assert!(near(activation.paid_until, expected));
    assert_eq!(billing::bonus_days(&pool, user).await.unwrap(), 0);
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_referral_rewards_the_referrer_once(pool: PgPool) {
    let (referrer, referee) = (person(&pool).await, person(&pool).await);
    let plan = paid_plan(&pool, "plus").await;
    let code = billing::referral_code_for(&pool, referrer).await.unwrap();
    assert_eq!(code.len(), 8);
    assert_eq!(billing::referral_code_for(&pool, referrer).await.unwrap(), code, "the same code every time");

    // Their own code, and a code that doesn't exist.
    assert!(matches!(quote(&pool, referrer, &plan, 1, Some(&code)).await, Err(QuoteError::OwnReferralCode)));
    // The referee gets the percent off and the code is matched in any case.
    let q = quote(&pool, referee, &plan, 1, Some(&code.to_lowercase())).await.unwrap();
    assert_eq!((q.referral_discount, q.amount, q.referral_code.as_deref()), (5_000, 45_000, Some(code.as_str())));

    // A referrer with nothing paid yet has the days wait.
    let first = pay(&pool, referee, &plan, 1, Some(&code)).await;
    assert_eq!(first.referrer_id, Some(referrer));
    assert_eq!(billing::bonus_days(&pool, referrer).await.unwrap(), BONUS_DAYS);
    assert_eq!(billing::referral_counts(&pool, referrer).await.unwrap(), (1, 1));

    // Their second payment can't use a referral code, and a forced one rewards nobody again.
    assert!(matches!(quote(&pool, referee, &plan, 1, Some(&code)).await, Err(QuoteError::ReferralFirstPaymentOnly)));
    let q = quote(&pool, referee, &plan, 1, None).await.unwrap();
    let payment = billing::create_payment(&pool, referee, &Quote { referral_code: Some(code.clone()), ..q }, KIND_NEW, INVOICE_HOURS).await.unwrap();
    let mut tx = pool.begin().await.unwrap();
    let again = billing::activate(&mut tx, payment.id, None, None, &cfg()).await.unwrap().unwrap();
    tx.commit().await.unwrap();
    assert_eq!(again.referrer_id, None);
    assert_eq!(billing::bonus_days(&pool, referrer).await.unwrap(), BONUS_DAYS);
    assert_eq!(first.payment.user_id, referee);
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_referrer_on_a_paid_period_has_it_extended(pool: PgPool) {
    let (referrer, referee) = (person(&pool).await, person(&pool).await);
    let plan = paid_plan(&pool, "plus").await;
    let own = pay(&pool, referrer, &plan, 1, None).await;
    let code = billing::referral_code_for(&pool, referrer).await.unwrap();
    pay(&pool, referee, &plan, 1, Some(&code)).await;
    let sub = plans::subscription_for(&pool, referrer).await.unwrap();
    assert!(near(sub.paid_until.unwrap(), own.paid_until + Duration::days(i64::from(BONUS_DAYS))));
    assert_eq!(billing::bonus_days(&pool, referrer).await.unwrap(), 0);
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_lapsed_paid_period_falls_back_to_the_default_plan_after_the_grace(pool: PgPool) {
    let user = person(&pool).await;
    let plan = paid_plan(&pool, "plus").await;
    let free = plans::default_plan(&pool).await.unwrap();
    pay(&pool, user, &plan, 1, None).await;
    sqlx::query("UPDATE user_subscriptions SET quota_override = 777 WHERE user_id = $1").bind(user).execute(&pool).await.unwrap();

    // Two days past the end is still inside the 3-day grace.
    sqlx::query("UPDATE user_subscriptions SET paid_until = now() - interval '2 days' WHERE user_id = $1").bind(user).execute(&pool).await.unwrap();
    let sub = plans::subscription_for(&pool, user).await.unwrap();
    assert_eq!((sub.plan.id, sub.lapsed), (plan.id, false));

    sqlx::query("UPDATE user_subscriptions SET paid_until = now() - interval '4 days' WHERE user_id = $1").bind(user).execute(&pool).await.unwrap();
    let sub = plans::subscription_for(&pool, user).await.unwrap();
    assert_eq!((sub.plan.id, sub.lapsed), (free.id, true));
    // The admin's override isn't part of what was paid for.
    assert_eq!((sub.quota_override, sub.monthly_tokens), (Some(777), 777));

    // A plan with no end (an admin's) never lapses.
    sqlx::query("UPDATE user_subscriptions SET paid_until = NULL WHERE user_id = $1").bind(user).execute(&pool).await.unwrap();
    assert_eq!(plans::subscription_for(&pool, user).await.unwrap().plan.id, plan.id);
}

#[sqlx::test(migrations = "../../migrations")]
async fn an_admin_assigning_another_plan_ends_the_paid_period(pool: PgPool) {
    let (admin, user) = (person(&pool).await, person(&pool).await);
    let plan = paid_plan(&pool, "plus").await;
    let other = paid_plan(&pool, "team").await;
    pay(&pool, user, &plan, 1, None).await;
    let same = SubscriptionChange { plan_id: plan.id, quota_override: Some(5), override_until: None, note: None };
    plans::change_subscription(&pool, user, &same, admin).await.unwrap();
    assert!(plans::subscription_for(&pool, user).await.unwrap().paid_until.is_some(), "the same plan keeps its paid period");
    let moved = SubscriptionChange { plan_id: other.id, quota_override: None, override_until: None, note: None };
    plans::change_subscription(&pool, user, &moved, admin).await.unwrap();
    let sub = plans::subscription_for(&pool, user).await.unwrap();
    assert_eq!((sub.plan.id, sub.paid_until, sub.term_months), (other.id, None, None));
}

#[sqlx::test(migrations = "../../migrations")]
async fn renewals_are_selected_by_the_window_and_notices_are_claimed_once(pool: PgPool) {
    let (soon, later, past, gone, manual) = (person(&pool).await, person(&pool).await, person(&pool).await, person(&pool).await, person(&pool).await);
    let plan = paid_plan(&pool, "plus").await;
    for user in [soon, later, past, gone, manual] {
        pay(&pool, user, &plan, 1, None).await;
    }
    for (user, interval) in [(soon, "3 days"), (later, "20 days"), (manual, "5 days")] {
        sqlx::query(&format!("UPDATE user_subscriptions SET paid_until = now() + interval '{interval}' WHERE user_id = $1")).bind(user).execute(&pool).await.unwrap();
    }
    sqlx::query("UPDATE user_subscriptions SET paid_until = now() - interval '1 day' WHERE user_id = $1").bind(past).execute(&pool).await.unwrap();
    sqlx::query("UPDATE user_subscriptions SET paid_until = now() - interval '10 days' WHERE user_id = $1").bind(gone).execute(&pool).await.unwrap();
    sqlx::query("UPDATE user_subscriptions SET auto_renew = false WHERE user_id = $1").bind(manual).execute(&pool).await.unwrap();

    let now = Utc::now();
    let due = billing::renewal_candidates(&pool, now, &cfg()).await.unwrap();
    let mut users: Vec<Uuid> = due.iter().map(|c| c.user_id).collect();
    users.sort();
    let mut expected = vec![soon, past, manual];
    expected.sort();
    assert_eq!(users, expected, "ends within 7 days, or ended inside the grace");
    assert!(due.iter().find(|c| c.user_id == manual).is_some_and(|c| !c.auto_renew));
    assert_eq!(due[0].term_months, 1);

    // Only the one past the grace is announced as ended, once.
    let lapsed = billing::lapsed_unannounced(&pool, now, &cfg()).await.unwrap();
    assert_eq!(lapsed.iter().map(|l| l.user_id).collect::<Vec<_>>(), vec![gone]);
    assert!(billing::claim_notice(&pool, gone, billing::NOTICE_ENDED, lapsed[0].paid_until).await.unwrap());
    assert!(!billing::claim_notice(&pool, gone, billing::NOTICE_ENDED, lapsed[0].paid_until).await.unwrap());
    assert!(billing::lapsed_unannounced(&pool, now, &cfg()).await.unwrap().is_empty());

    // A plan taken off sale isn't renewed.
    sqlx::query("UPDATE plans SET is_active = false WHERE id = $1").bind(plan.id).execute(&pool).await.unwrap();
    assert!(billing::renewal_candidates(&pool, now, &cfg()).await.unwrap().is_empty());
}

#[sqlx::test(migrations = "../../migrations")]
async fn payments_are_listed_for_the_person_and_for_admins(pool: PgPool) {
    let (a, b) = (person(&pool).await, person(&pool).await);
    let plan = paid_plan(&pool, "plus").await;
    pay(&pool, a, &plan, 1, None).await;
    let q = quote(&pool, b, &plan, 1, None).await.unwrap();
    let pending = billing::create_payment(&pool, b, &q, KIND_NEW, INVOICE_HOURS).await.unwrap();
    assert_eq!(billing::list_payments_for_user(&pool, a).await.unwrap().len(), 1);
    let page = billing::list_payments_for_admin(&pool, None, 1).await.unwrap();
    assert_eq!((page.total, page.items.len()), (2, 2));
    assert_eq!(page.items[0].plan_name, "plus");
    let only_pending = billing::list_payments_for_admin(&pool, Some("pending"), 1).await.unwrap();
    assert_eq!(only_pending.items.iter().map(|i| i.payment.id).collect::<Vec<_>>(), vec![pending.id]);
    assert!(billing::list_payments_for_admin(&pool, None, 2).await.unwrap().items.is_empty());
    // An open payment for the same plan, term and code is found again; another term isn't.
    billing::attach_invoice(&pool, pending.id, "doku", "inv-1", "https://pay.example/inv-1", None).await.unwrap();
    assert_eq!(billing::get_payment(&pool, pending.id).await.unwrap().unwrap().provider, "doku", "the provider that made the invoice is kept");
    assert!(billing::reusable_payment(&pool, b, plan.id, 1, None).await.unwrap().is_some());
    assert!(billing::reusable_payment(&pool, b, plan.id, 6, None).await.unwrap().is_none());
    assert!(billing::reusable_payment(&pool, b, plan.id, 1, Some("X")).await.unwrap().is_none());
}
