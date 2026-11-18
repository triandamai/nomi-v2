//! Paid plans on the server: the payment gateway the checkout, the webhook and the renewal worker
//! share, starting a payment (and its Xendit or DOKU invoice), settling one from what the gateway says,
//! and telling the person about it. The pricing and the database work are in `nomi_usage::billing`.

use std::sync::{Arc, RwLock};

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use nomi_i18n::Locale;
use nomi_payment::doku::DokuClient;
use nomi_payment::xendit::XenditClient;
use nomi_payment::{CreateInvoiceRequest, PaymentGateway, PaymentProviderKind, PaymentStatus};
use nomi_usage::billing::{self, Activation, BillingConfig, Payment, Quote, QuoteError, STATUS_EXPIRED, STATUS_FAILED, STATUS_PAID};
use nomi_usage::plans;

use crate::notifications::{self, Kind, Notice};
use crate::quota::format_tokens;

/// Seconds in an hour: an invoice's lifetime goes to the gateway in seconds.
const SECONDS_PER_HOUR: u32 = 3600;
/// Where the person lands after paying, or failing to.
const BILLING_PAGE: &str = "/billing";
/// A paid amount may differ from the invoice by less than this (it travels as a float).
const AMOUNT_TOLERANCE: f64 = 0.5;

/// A payment gateway, which provider it is, and (Xendit) the token it calls the webhook with.
#[derive(Clone)]
pub struct Gateway {
    pub provider: PaymentProviderKind,
    pub client: Arc<dyn PaymentGateway>,
    pub webhook_token: Option<String>,
}

static OVERRIDE: RwLock<Option<Gateway>> = RwLock::new(None);

/// Replaces the gateways built from Admin → Configuration, for tests (`None` goes back to them).
pub fn set_gateway_override(gateway: Option<Gateway>) {
    *OVERRIDE.write().unwrap_or_else(|e| e.into_inner()) = gateway;
}

fn overridden() -> Option<Gateway> {
    OVERRIDE.read().unwrap_or_else(|e| e.into_inner()).clone()
}

/// `provider`'s gateway as configured now, or `None` when it isn't set up.
pub async fn gateway_for(provider: PaymentProviderKind) -> Option<Gateway> {
    if let Some(gateway) = overridden() {
        return Some(gateway);
    }
    match provider {
        PaymentProviderKind::Xendit => {
            let config = crate::configuration::xendit_config().await?;
            let webhook_token = config.webhook_token.clone();
            let client = XenditClient::new(config).ok()?;
            Some(Gateway { provider, client: Arc::new(client), webhook_token })
        }
        PaymentProviderKind::Doku => {
            let client = DokuClient::new(crate::configuration::doku_config().await?).ok()?;
            Some(Gateway { provider, client: Arc::new(client), webhook_token: None })
        }
    }
}

/// The gateway new payments go through: the one chosen in Admin → Configuration → Payment, or,
/// with none chosen, the first that is set up. `None` when that one isn't set up.
pub async fn gateway() -> Option<Gateway> {
    if let Some(gateway) = overridden() {
        return Some(gateway);
    }
    if let Some(chosen) = crate::configuration::chosen_payment_provider().await {
        return gateway_for(chosen).await;
    }
    for provider in PaymentProviderKind::ALL {
        if let Some(gateway) = gateway_for(provider).await {
            return Some(gateway);
        }
    }
    None
}

/// The gateway of the provider `payment` was made with, so it can be checked or cancelled after
/// the admin moved new payments to another provider.
pub async fn gateway_of(payment: &Payment) -> Option<Gateway> {
    gateway_for(PaymentProviderKind::from_name(&payment.provider).unwrap_or(PaymentProviderKind::Xendit)).await
}

/// How a payment was made, as Billing shows it: "BANK_TRANSFER · BCA".
pub fn method_label(method: Option<&str>, channel: Option<&str>) -> Option<String> {
    let parts: Vec<&str> = [method, channel].into_iter().flatten().map(str::trim).filter(|p| !p.is_empty()).collect();
    (!parts.is_empty()).then(|| parts.join(" · "))
}

/// Whether two tokens are equal, taking as long whatever they share.
pub fn tokens_match(a: &str, b: &str) -> bool {
    let (a, b) = (a.as_bytes(), b.as_bytes());
    let mut diff = a.len() ^ b.len();
    for i in 0..a.len().max(b.len()) {
        diff |= usize::from(a.get(i).copied().unwrap_or(0) ^ b.get(i).copied().unwrap_or(0));
    }
    diff == 0
}

#[derive(Debug)]
pub enum CheckoutError {
    Quote(QuoteError),
    /// Something has to be paid, and no payment gateway is set up.
    NotConfigured,
    /// The gateway refused to make the invoice.
    Gateway(String),
    Db(sqlx::Error),
}

impl From<QuoteError> for CheckoutError {
    fn from(e: QuoteError) -> Self {
        CheckoutError::Quote(e)
    }
}

impl From<sqlx::Error> for CheckoutError {
    fn from(e: sqlx::Error) -> Self {
        CheckoutError::Db(e)
    }
}

async fn user_email(pool: &PgPool, user_id: Uuid) -> Option<String> {
    sqlx::query_scalar("SELECT COALESCE(w.email, g.email) FROM users u LEFT JOIN web_credentials w ON w.user_id = u.id LEFT JOIN google_identities g ON g.user_id = u.id WHERE u.id = $1")
        .bind(user_id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()
        .flatten()
}

pub async fn user_locale(pool: &PgPool, user_id: Uuid) -> Locale {
    let language: Option<String> = sqlx::query_scalar("SELECT language FROM user_preferences WHERE user_id = $1").bind(user_id).fetch_optional(pool).await.ok().flatten();
    Locale::from_code_or_default(language.as_deref())
}

/// Makes the payment for `quote` and, when something has to be paid, its invoice. A quote worth
/// nothing activates the plan straight away.
pub async fn start_payment(pool: &PgPool, user_id: Uuid, quote: &Quote, kind: &str, cfg: &BillingConfig, gateway: Option<&Gateway>) -> Result<Payment, CheckoutError> {
    if quote.amount > 0 && gateway.is_none() {
        return Err(CheckoutError::NotConfigured);
    }
    let payment = billing::create_payment(pool, user_id, quote, kind, cfg.invoice_hours).await?;
    let Some(gateway) = gateway.filter(|_| quote.amount > 0) else {
        let mut tx = pool.begin().await?;
        let activation = billing::activate(&mut tx, payment.id, None, None, cfg).await?;
        tx.commit().await?;
        if let Some(activation) = activation {
            notify_activated(pool, &activation).await;
        }
        return Ok(billing::get_payment(pool, payment.id).await?.unwrap_or(payment));
    };

    let plan_name = plans::get_plan(pool, quote.plan_id).await?.map(|p| p.name).unwrap_or_default();
    let return_url = crate::public_url::link(&format!("{BILLING_PAGE}?payment={}", payment.id));
    let request = CreateInvoiceRequest {
        external_id: payment.id.to_string(),
        // The one place money becomes a float.
        amount: quote.amount as f64,
        description: Some(format!("{plan_name} \u{2014} {}", months_words(Locale::En, quote.term_months))),
        payer_email: user_email(pool, user_id).await,
        invoice_duration: Some(cfg.invoice_hours.saturating_mul(SECONDS_PER_HOUR)),
        customer: None,
        currency: Some(quote.currency.clone()),
        success_redirect_url: return_url.clone(),
        failure_redirect_url: return_url,
    };
    match gateway.client.create_invoice(request).await {
        Ok(invoice) => {
            billing::attach_invoice(pool, payment.id, gateway.provider.as_str(), &invoice.id, &invoice.invoice_url, invoice.expiry_date).await?;
            Ok(billing::get_payment(pool, payment.id).await?.unwrap_or(payment))
        }
        Err(e) => {
            tracing::error!(error = %e, payment_id = %payment.id, "couldn't create the invoice");
            billing::release(pool, payment.id, STATUS_FAILED).await?;
            Err(CheckoutError::Gateway(e.to_string()))
        }
    }
}

/// What applying a gateway's word about a payment did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Settled {
    /// It was paid and the plan is active now.
    Activated,
    /// It had been paid already; nothing changed.
    AlreadyPaid,
    /// It expired or failed, and its coupon is free again.
    Released,
    /// The gateway says it was paid, but not what we asked for; nothing was activated.
    AmountMismatch,
    /// Still waiting, or nothing to do.
    Ignored,
}

fn amount_matches(paid: f64, expected: i64) -> bool {
    (paid - expected as f64).abs() < AMOUNT_TOLERANCE
}

/// Applies `status` (from a webhook, or from asking the gateway) to `payment`. Safe to repeat.
pub async fn apply_status(
    pool: &PgPool,
    payment: &Payment,
    status: &PaymentStatus,
    amount: f64,
    paid_at: Option<DateTime<Utc>>,
    payment_method: Option<&str>,
) -> Result<Settled, sqlx::Error> {
    match status {
        PaymentStatus::Paid => {
            if payment.status == STATUS_PAID {
                return Ok(Settled::AlreadyPaid);
            }
            if !amount_matches(amount, payment.amount) {
                tracing::error!(payment_id = %payment.id, paid = amount, expected = payment.amount, "a payment came in for the wrong amount");
                return Ok(Settled::AmountMismatch);
            }
            let cfg = billing::config().await;
            let mut tx = pool.begin().await?;
            let activation = billing::activate(&mut tx, payment.id, paid_at, payment_method, &cfg).await?;
            tx.commit().await?;
            match activation {
                Some(activation) => {
                    notify_activated(pool, &activation).await;
                    Ok(Settled::Activated)
                }
                None => Ok(Settled::AlreadyPaid),
            }
        }
        PaymentStatus::Expired => Ok(released(billing::release(pool, payment.id, STATUS_EXPIRED).await?)),
        PaymentStatus::Failed => Ok(released(billing::release(pool, payment.id, STATUS_FAILED).await?)),
        PaymentStatus::Pending | PaymentStatus::Unknown => Ok(Settled::Ignored),
    }
}

fn released(changed: bool) -> Settled {
    if changed {
        Settled::Released
    } else {
        Settled::Ignored
    }
}

// ---------------------------------------------------------------------------------------------
// What people are told

fn months_words(locale: Locale, months: i32) -> String {
    if months == 1 {
        locale.t("subscription.months_one")
    } else {
        locale.tf("subscription.months_other", &[("count", &months.to_string())])
    }
}

fn date(at: DateTime<Utc>) -> String {
    at.format("%Y-%m-%d").to_string()
}

fn money(locale: Locale, amount: i64, currency: &str) -> String {
    format!("{currency} {}", format_tokens(amount, locale))
}

fn notice(title: String, body: String, link: &str) -> Notice {
    Notice { kind: Kind::Subscription, title, body, link: Some(link.to_string()) }
}

pub fn paid_notice(locale: Locale, plan: &str, months: i32, until: DateTime<Utc>) -> Notice {
    notice(
        locale.tf("subscription.paid_title", &[("plan", plan)]),
        locale.tf("subscription.paid_body", &[("plan", plan), ("months", &months_words(locale, months)), ("date", &date(until))]),
        BILLING_PAGE,
    )
}

pub fn renewal_notice(locale: Locale, plan: &str, payment: &Payment, until: DateTime<Utc>) -> Notice {
    notice(
        locale.tf("subscription.renewal_title", &[("plan", plan)]),
        locale.tf(
            "subscription.renewal_body",
            &[
                ("plan", plan),
                ("date", &date(until)),
                ("amount", &money(locale, payment.amount, &payment.currency)),
                ("months", &months_words(locale, payment.term_months)),
            ],
        ),
        payment.invoice_url.as_deref().unwrap_or(BILLING_PAGE),
    )
}

pub fn reminder_notice(locale: Locale, plan: &str, until: DateTime<Utc>) -> Notice {
    notice(
        locale.tf("subscription.reminder_title", &[("plan", plan)]),
        locale.tf("subscription.reminder_body", &[("plan", plan), ("date", &date(until))]),
        BILLING_PAGE,
    )
}

pub fn ended_notice(locale: Locale, plan: &str, default_plan: &str, until: DateTime<Utc>) -> Notice {
    notice(
        locale.tf("subscription.ended_title", &[("plan", plan)]),
        locale.tf("subscription.ended_body", &[("plan", plan), ("default", default_plan), ("date", &date(until))]),
        BILLING_PAGE,
    )
}

fn referral_notice(locale: Locale, days: i32) -> Notice {
    notice(locale.t("subscription.referral_title"), locale.tf("subscription.referral_body", &[("days", &days.to_string())]), BILLING_PAGE)
}

/// Sends `notice` to `user_id`; a failure is logged, never raised: what it reports already happened.
pub async fn tell(pool: &PgPool, user_id: Uuid, notice: Notice) {
    if let Err(e) = notifications::send(pool, user_id, notice).await {
        tracing::warn!(error = %e, %user_id, "couldn't send a billing notification");
    }
}

/// Tells the payer their plan is active, and the referrer (if there was one) about their reward.
pub async fn notify_activated(pool: &PgPool, activation: &Activation) {
    let payment = &activation.payment;
    let plan = match plans::get_plan(pool, payment.plan_id).await {
        Ok(Some(plan)) => plan.name,
        _ => String::new(),
    };
    let locale = user_locale(pool, payment.user_id).await;
    tell(pool, payment.user_id, paid_notice(locale, &plan, payment.term_months, activation.paid_until)).await;
    if let Some(referrer) = activation.referrer_id {
        let days = billing::config().await.referrer_bonus_days;
        tell(pool, referrer, referral_notice(user_locale(pool, referrer).await, days)).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokens_are_compared_whole() {
        assert!(tokens_match("secret", "secret"));
        assert!(!tokens_match("secret", "secreT"));
        assert!(!tokens_match("secret", "secre"));
        assert!(!tokens_match("secret", "secret2"));
        assert!(!tokens_match("", "x"));
        assert!(tokens_match("", ""));
    }

    #[test]
    fn names_how_it_was_paid() {
        assert_eq!(method_label(Some("BANK_TRANSFER"), Some("BCA")).as_deref(), Some("BANK_TRANSFER · BCA"));
        assert_eq!(method_label(Some("EWALLET"), None).as_deref(), Some("EWALLET"));
        assert_eq!(method_label(None, Some(" ")), None);
    }

    #[test]
    fn a_paid_amount_must_be_the_invoiced_one() {
        assert!(amount_matches(50_000.0, 50_000));
        assert!(amount_matches(50_000.2, 50_000));
        assert!(!amount_matches(49_999.0, 50_000));
        assert!(!amount_matches(0.0, 50_000));
    }

    #[test]
    fn notices_are_in_the_persons_language() {
        let until = DateTime::parse_from_rfc3339("2026-12-01T00:00:00Z").unwrap().with_timezone(&Utc);
        let en = paid_notice(Locale::En, "Pro", 6, until);
        assert_eq!(en.title, "Payment received: you're on Pro");
        assert!(en.body.contains("6 months") && en.body.contains("2026-12-01"), "{}", en.body);
        assert!(paid_notice(Locale::Id, "Pro", 1, until).body.contains("1 bulan"));
        assert!(ended_notice(Locale::En, "Pro", "Free", until).body.contains("Free"));
        assert!(reminder_notice(Locale::En, "Pro", until).body.contains("2026-12-01"));
        assert_eq!(money(Locale::En, 49_000, "IDR"), "IDR 49,000");
        assert_eq!(money(Locale::Id, 49_000, "IDR"), "IDR 49.000");
    }

    #[test]
    fn every_provider_admin_can_choose_has_a_gateway() {
        let choices: Vec<&str> = nomi_settings::config::PAYMENT_PROVIDERS.to_vec();
        let gateways: Vec<&str> = PaymentProviderKind::ALL.iter().map(PaymentProviderKind::as_str).collect();
        assert_eq!(choices, gateways);
        assert!(choices.iter().all(|name| PaymentProviderKind::from_name(name).is_some()));
    }
}
