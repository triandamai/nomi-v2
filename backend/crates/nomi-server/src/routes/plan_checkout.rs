//! Moving to a plan the simple way: a month of it through the payment gateway (Xendit), and the
//! waitlist of one that isn't sold yet. The checkout runs on the billing engine (`nomi_usage::billing`,
//! `crate::billing`) like the full checkout at `/api/billing/checkout`, as one month with no code;
//! the gateway's callback (`POST /api/payments/xendit/webhook`) is `routes::billing::xendit_webhook`.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde_json::{json, Value};
use uuid::Uuid;

use nomi_auth::extractor::AuthClaims;
use nomi_usage::billing::{self, QuoteError};
use nomi_usage::plans;

use crate::app::AppState;
use crate::billing::{self as server_billing, CheckoutError};

type RouteError = (StatusCode, Json<Value>);

fn error(status: StatusCode, code: &str) -> RouteError {
    (status, Json(json!({ "error": code })))
}

fn internal(e: impl std::fmt::Display) -> RouteError {
    tracing::error!(error = %e, "plan checkout failed");
    error(StatusCode::INTERNAL_SERVER_ERROR, "server_error")
}

/// What this checkout pays for: one month.
const CHECKOUT_MONTHS: i32 = 1;
/// Where someone lands when nothing was left to pay (the plan is theirs already).
const BILLING_PAGE: &str = "/billing";

/// A plan that's on offer, or 404.
async fn offered_plan(state: &AppState, id: Uuid) -> Result<plans::PlanRow, RouteError> {
    plans::get_plan(&state.pool, id)
        .await
        .map_err(internal)?
        .filter(|p| p.is_active)
        .ok_or_else(|| error(StatusCode::NOT_FOUND, "not_found"))
}

fn checkout_error(e: CheckoutError) -> RouteError {
    match e {
        CheckoutError::NotConfigured => error(StatusCode::SERVICE_UNAVAILABLE, "payments_not_configured"),
        CheckoutError::Gateway(_) => error(StatusCode::BAD_GATEWAY, "gateway_error"),
        CheckoutError::Quote(QuoteError::Db(e)) | CheckoutError::Db(e) => internal(e),
        // No price, or no one-month term on offer.
        CheckoutError::Quote(_) => error(StatusCode::CONFLICT, "not_for_sale"),
    }
}

/// `POST /api/plans/:id/checkout` → `{ url }`: the gateway's page to pay a month of the plan on.
/// `409 already_on_plan`, `409 not_for_sale` (no price yet: join its waitlist), `503
/// payments_not_configured` while no gateway is set up, `502 gateway_error` when it refuses.
/// Asking again while that invoice is open gives the same page back.
pub async fn checkout(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, RouteError> {
    let plan = offered_plan(&state, id).await?;
    let current = plans::subscription_for(&state.pool, claims.sub)
        .await
        .map_err(internal)?;
    if current.plan.id == plan.id {
        return Err(error(StatusCode::CONFLICT, "already_on_plan"));
    }
    if !plan.for_sale() {
        return Err(error(StatusCode::CONFLICT, "not_for_sale"));
    }
    let Some(gateway) = server_billing::gateway().await else {
        return Err(error(
            StatusCode::SERVICE_UNAVAILABLE,
            "payments_not_configured",
        ));
    };

    let open = billing::reusable_payment(&state.pool, claims.sub, plan.id, CHECKOUT_MONTHS, None)
        .await
        .map_err(internal)?;
    let payment = match open {
        Some(open) => open,
        None => {
            let cfg = billing::config().await;
            let quote = billing::quote(&state.pool, claims.sub, plan.id, CHECKOUT_MONTHS, None, &cfg)
                .await
                .map_err(|e| checkout_error(CheckoutError::Quote(e)))?;
            server_billing::start_payment(&state.pool, claims.sub, &quote, billing::KIND_NEW, &cfg, Some(&gateway))
                .await
                .map_err(|e| {
                    if let CheckoutError::Gateway(reason) = &e {
                        tracing::warn!(error = %reason, user = %claims.sub, "the payment gateway refused a checkout");
                    }
                    checkout_error(e)
                })?
        }
    };
    // Nothing to pay (a free promo): the plan is theirs already, so back to Billing.
    let url = payment
        .invoice_url
        .or_else(|| crate::public_url::link(&format!("{BILLING_PAGE}?payment={}", payment.id)));
    Ok(Json(json!({ "url": url })))
}

/// `POST /api/plans/:id/waitlist` → `{ on_waitlist: true }`: tell me when this plan opens. Asking
/// again changes nothing.
pub async fn join_waitlist(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, RouteError> {
    let plan = offered_plan(&state, id).await?;
    sqlx::query(
        "INSERT INTO plan_waitlist (user_id, plan_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
    )
    .bind(claims.sub)
    .bind(plan.id)
    .execute(&state.pool)
    .await
    .map_err(internal)?;
    Ok(Json(json!({ "on_waitlist": true })))
}
