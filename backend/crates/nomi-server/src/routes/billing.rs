//! Paying for a plan: the billing terms, a quote and checkout (a Xendit invoice), the person's
//! payments and subscription, referral code, and the webhook Xendit calls when an invoice is paid.

use axum::body::Bytes;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::Json;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

use nomi_auth::extractor::AuthClaims;
use nomi_payment::PaymentStatus;
use nomi_usage::billing::{self, Payment, PaymentListItem, Quote, QuoteError, Term, STATUS_CANCELLED, STATUS_PENDING};
use nomi_usage::plans::{self, PlanRow};

use crate::app::AppState;
use crate::billing::{self as server_billing, CheckoutError, Gateway};
use crate::routes::deploy::ApiError;

const CALLBACK_TOKEN_HEADER: &str = "x-callback-token";

fn internal<E: std::fmt::Display>(e: E) -> ApiError {
    tracing::error!(error = %e, "billing request failed");
    (StatusCode::INTERNAL_SERVER_ERROR, "something went wrong".to_string())
}

fn quote_error(e: QuoteError) -> ApiError {
    match e {
        QuoteError::Db(e) => internal(e),
        other => (StatusCode::BAD_REQUEST, other.to_string()),
    }
}

fn checkout_error(e: CheckoutError) -> ApiError {
    match e {
        CheckoutError::Quote(e) => quote_error(e),
        CheckoutError::NotConfigured => (StatusCode::SERVICE_UNAVAILABLE, "Payments aren't set up yet. Please try again later.".to_string()),
        CheckoutError::Gateway(_) => (StatusCode::BAD_GATEWAY, "We couldn't create the invoice. Please try again in a moment.".to_string()),
        CheckoutError::Db(e) => internal(e),
    }
}

/// `GET /api/billing/terms`: the periods someone can pay for.
pub async fn terms(State(state): State<AppState>, AuthClaims(_): AuthClaims) -> Result<Json<Vec<Term>>, ApiError> {
    Ok(Json(billing::list_terms(&state.pool, true).await.map_err(internal)?))
}

#[derive(Deserialize)]
pub struct QuoteRequest {
    pub plan_id: Uuid,
    pub months: i32,
    #[serde(default)]
    pub code: Option<String>,
}

/// `POST /api/billing/quote`: what it would cost, step by step.
pub async fn quote(State(state): State<AppState>, AuthClaims(claims): AuthClaims, Json(req): Json<QuoteRequest>) -> Result<Json<Quote>, ApiError> {
    let cfg = billing::config().await;
    billing::quote(&state.pool, claims.sub, req.plan_id, req.months, req.code.as_deref(), &cfg).await.map(Json).map_err(quote_error)
}

#[derive(Serialize)]
pub struct CheckoutResponse {
    pub payment_id: Uuid,
    pub status: String,
    pub invoice_url: Option<String>,
}

impl From<&Payment> for CheckoutResponse {
    fn from(payment: &Payment) -> Self {
        CheckoutResponse { payment_id: payment.id, status: payment.status.clone(), invoice_url: payment.invoice_url.clone() }
    }
}

/// `POST /api/billing/checkout`: starts a payment and its invoice. Asking again for the same
/// plan, months and code while that invoice is still open gives the same invoice back.
pub async fn checkout(State(state): State<AppState>, AuthClaims(claims): AuthClaims, Json(req): Json<QuoteRequest>) -> Result<Json<CheckoutResponse>, ApiError> {
    let code = billing::normalize_code(req.code.as_deref());
    if let Some(open) = billing::reusable_payment(&state.pool, claims.sub, req.plan_id, req.months, code.as_deref()).await.map_err(internal)? {
        return Ok(Json(CheckoutResponse::from(&open)));
    }
    let cfg = billing::config().await;
    let quote = billing::quote(&state.pool, claims.sub, req.plan_id, req.months, code.as_deref(), &cfg).await.map_err(quote_error)?;
    let gateway = server_billing::gateway().await;
    let payment = server_billing::start_payment(&state.pool, claims.sub, &quote, billing::KIND_NEW, &cfg, gateway.as_ref()).await.map_err(checkout_error)?;
    Ok(Json(CheckoutResponse::from(&payment)))
}

/// `GET /api/billing/payments`: theirs, newest first.
pub async fn list_payments(State(state): State<AppState>, AuthClaims(claims): AuthClaims) -> Result<Json<Vec<PaymentListItem>>, ApiError> {
    Ok(Json(billing::list_payments_for_user(&state.pool, claims.sub).await.map_err(internal)?))
}

/// Asks the gateway about a pending payment and applies the answer; any trouble leaves it as it was.
async fn refresh(state: &AppState, payment: &Payment, gateway: &Gateway) {
    let Some(invoice_id) = payment.provider_invoice_id.as_deref() else { return };
    match gateway.client.get_invoice(invoice_id).await {
        Ok(invoice) => {
            if let Err(e) = server_billing::apply_status(&state.pool, payment, &invoice.status, invoice.amount, invoice.paid_at, invoice.payment_method.as_deref()).await {
                tracing::error!(error = %e, payment_id = %payment.id, "couldn't apply an invoice's status");
            }
        }
        Err(e) => tracing::warn!(error = %e, payment_id = %payment.id, "couldn't read an invoice"),
    }
}

async fn own_payment(state: &AppState, user_id: Uuid, id: Uuid) -> Result<Payment, ApiError> {
    billing::get_payment(&state.pool, id)
        .await
        .map_err(internal)?
        .filter(|p| p.user_id == user_id)
        .ok_or((StatusCode::NOT_FOUND, "payment not found".to_string()))
}

async fn item(state: &AppState, id: Uuid) -> Result<Json<PaymentListItem>, ApiError> {
    billing::payment_item(&state.pool, id).await.map_err(internal)?.map(Json).ok_or((StatusCode::NOT_FOUND, "payment not found".to_string()))
}

/// `GET /api/billing/payments/:id`: a payment of theirs. While it's pending, the gateway is
/// asked first, so the page they return to is right even if the webhook is late.
pub async fn get_payment(State(state): State<AppState>, AuthClaims(claims): AuthClaims, Path(id): Path<Uuid>) -> Result<Json<PaymentListItem>, ApiError> {
    let payment = own_payment(&state, claims.sub, id).await?;
    if payment.status == STATUS_PENDING {
        if let Some(gateway) = server_billing::gateway().await {
            refresh(&state, &payment, &gateway).await;
        }
    }
    item(&state, id).await
}

/// `POST /api/billing/payments/:id/cancel`: closes the invoice and cancels the payment.
pub async fn cancel_payment(State(state): State<AppState>, AuthClaims(claims): AuthClaims, Path(id): Path<Uuid>) -> Result<Json<PaymentListItem>, ApiError> {
    let payment = own_payment(&state, claims.sub, id).await?;
    if payment.status != STATUS_PENDING {
        return Err((StatusCode::CONFLICT, "only a payment that is waiting can be cancelled".to_string()));
    }
    if let Some(invoice_id) = payment.provider_invoice_id.as_deref() {
        let gateway = server_billing::gateway().await.ok_or((StatusCode::SERVICE_UNAVAILABLE, "Payments aren't set up yet. Please try again later.".to_string()))?;
        if let Err(e) = gateway.client.expire_invoice(invoice_id).await {
            tracing::warn!(error = %e, payment_id = %payment.id, "couldn't expire an invoice");
            // It may have been paid in the meantime: look before giving up.
            refresh(&state, &payment, &gateway).await;
            let current = own_payment(&state, claims.sub, id).await?;
            return if current.status == STATUS_PENDING {
                Err((StatusCode::BAD_GATEWAY, "We couldn't cancel the invoice. Please try again in a moment.".to_string()))
            } else {
                item(&state, id).await
            };
        }
    }
    billing::release(&state.pool, id, STATUS_CANCELLED).await.map_err(internal)?;
    item(&state, id).await
}

#[derive(Serialize)]
pub struct SubscriptionView {
    pub plan: PlanRow,
    pub monthly_tokens: i64,
    pub paid_until: Option<DateTime<Utc>>,
    pub term_months: Option<i32>,
    pub auto_renew: bool,
    /// The paid period ran out and they're back on the default plan.
    pub lapsed: bool,
    /// Referral days waiting for their next paid period.
    pub bonus_days: i32,
    pub pending_renewal: Option<Payment>,
}

/// `GET /api/billing/subscription`.
pub async fn subscription(State(state): State<AppState>, AuthClaims(claims): AuthClaims) -> Result<Json<SubscriptionView>, ApiError> {
    let sub = plans::subscription_for(&state.pool, claims.sub).await.map_err(internal)?;
    let pending_renewal = billing::pending_renewal(&state.pool, claims.sub).await.map_err(internal)?;
    let bonus_days = billing::bonus_days(&state.pool, claims.sub).await.map_err(internal)?;
    Ok(Json(SubscriptionView {
        plan: sub.plan,
        monthly_tokens: sub.monthly_tokens,
        paid_until: sub.paid_until,
        term_months: sub.term_months,
        auto_renew: sub.auto_renew,
        lapsed: sub.lapsed,
        bonus_days,
        pending_renewal,
    }))
}

#[derive(Deserialize)]
pub struct AutoRenewRequest {
    pub auto_renew: bool,
}

/// `PUT /api/billing/auto-renew`.
pub async fn set_auto_renew(State(state): State<AppState>, AuthClaims(claims): AuthClaims, Json(req): Json<AutoRenewRequest>) -> Result<Json<Value>, ApiError> {
    billing::set_auto_renew(&state.pool, claims.sub, req.auto_renew).await.map_err(internal)?;
    Ok(Json(json!({ "auto_renew": req.auto_renew })))
}

#[derive(Serialize)]
pub struct ReferralView {
    pub code: String,
    pub referred_count: i64,
    pub rewarded_count: i64,
    pub referee_discount_percent: i64,
    pub referrer_bonus_days: i32,
}

/// `GET /api/billing/referral`: their code (made on first use), and what it earns.
pub async fn referral(State(state): State<AppState>, AuthClaims(claims): AuthClaims) -> Result<Json<ReferralView>, ApiError> {
    let code = billing::referral_code_for(&state.pool, claims.sub).await.map_err(internal)?;
    let (referred_count, rewarded_count) = billing::referral_counts(&state.pool, claims.sub).await.map_err(internal)?;
    let cfg = billing::config().await;
    Ok(Json(ReferralView {
        code,
        referred_count,
        rewarded_count,
        referee_discount_percent: cfg.referee_discount_percent,
        referrer_bonus_days: cfg.referrer_bonus_days,
    }))
}

/// `POST /api/payments/xendit/webhook`: Xendit says an invoice was paid, expired or failed. No
/// sign-in: it is trusted by its callback token, which must be configured (otherwise 503) and
/// match. Unknown invoices are acknowledged and ignored, and replays change nothing.
pub async fn xendit_webhook(State(state): State<AppState>, headers: HeaderMap, body: Bytes) -> Result<Json<Value>, ApiError> {
    let gateway = server_billing::gateway().await.ok_or((StatusCode::SERVICE_UNAVAILABLE, "payments aren't set up".to_string()))?;
    let expected = gateway
        .webhook_token
        .as_deref()
        .filter(|token| !token.is_empty())
        .ok_or((StatusCode::SERVICE_UNAVAILABLE, "the webhook token isn't set up".to_string()))?;
    let provided = headers.get(CALLBACK_TOKEN_HEADER).and_then(|v| v.to_str().ok());
    if !provided.is_some_and(|provided| server_billing::tokens_match(provided, expected)) {
        return Err((StatusCode::UNAUTHORIZED, "invalid callback token".to_string()));
    }
    let event = gateway.client.verify_webhook(provided, &body).map_err(|e| {
        tracing::warn!(error = %e, "an unreadable webhook");
        (StatusCode::BAD_REQUEST, "unreadable webhook".to_string())
    })?;
    let Some(payment) = billing::payment_by_external_id(&state.pool, &event.external_id).await.map_err(internal)? else {
        tracing::info!(external_id = %event.external_id, "webhook for an unknown payment");
        return Ok(Json(json!({ "ok": true })));
    };
    // A webhook that names another invoice than the one we made isn't for this payment.
    if payment.provider_invoice_id.as_deref().is_some_and(|invoice| invoice != event.invoice_id) {
        tracing::warn!(payment_id = %payment.id, "webhook for a different invoice than the payment's");
        return Ok(Json(json!({ "ok": true })));
    }
    let status: PaymentStatus = event.status;
    let method = server_billing::method_label(event.payment_method.as_deref(), event.payment_channel.as_deref());
    server_billing::apply_status(&state.pool, &payment, &status, event.amount, event.paid_at, method.as_deref()).await.map_err(internal)?;
    Ok(Json(json!({ "ok": true })))
}
