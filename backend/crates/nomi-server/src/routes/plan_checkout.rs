//! Moving to a plan: paying for one through the payment gateway (Xendit), joining the waitlist
//! of one that isn't sold yet, and the gateway's word that a payment went through, which moves
//! the person onto the plan they paid for.

use axum::body::Bytes;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::Json;
use serde_json::{json, Value};
use uuid::Uuid;

use nomi_auth::extractor::AuthClaims;
use nomi_payment::xendit::XenditClient;
use nomi_payment::{CreateInvoiceRequest, PaymentGateway, PaymentStatus};
use nomi_usage::plans::{self, SubscriptionChange};

use crate::app::AppState;

type RouteError = (StatusCode, Json<Value>);

fn error(status: StatusCode, code: &str) -> RouteError {
    (status, Json(json!({ "error": code })))
}

fn internal(e: impl std::fmt::Display) -> RouteError {
    tracing::error!(error = %e, "plan checkout failed");
    error(StatusCode::INTERNAL_SERVER_ERROR, "server_error")
}

/// How long the gateway keeps a checkout open, in seconds: a day.
const INVOICE_DURATION_SECS: u32 = 24 * 60 * 60;
/// The currency plans are sold in.
const CURRENCY: &str = "IDR";
/// Where the gateway sends the person back to.
const RETURN_PATH: &str = "/billing";
/// The header Xendit signs its callbacks with.
const CALLBACK_TOKEN_HEADER: &str = "x-callback-token";
/// What a payment's own id starts with, so it's clear in the gateway's dashboard what it was for.
const EXTERNAL_ID_PREFIX: &str = "nomi-plan-";

/// A plan that's on offer, or 404.
async fn offered_plan(state: &AppState, id: Uuid) -> Result<plans::PlanRow, RouteError> {
    plans::get_plan(&state.pool, id)
        .await
        .map_err(internal)?
        .filter(|p| p.is_active)
        .ok_or_else(|| error(StatusCode::NOT_FOUND, "not_found"))
}

/// `POST /api/plans/:id/checkout` → `{ url }`: the gateway's page to pay a month of the plan on.
/// `409 already_on_plan`, `409 not_for_sale` (no price yet: join its waitlist), `503
/// payments_not_configured` while no gateway is set up, `502 gateway_error` when it refuses.
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
    let Some(price) = plan.price_idr else {
        return Err(error(StatusCode::CONFLICT, "not_for_sale"));
    };
    let Some(config) = crate::configuration::xendit_config().await else {
        return Err(error(
            StatusCode::SERVICE_UNAVAILABLE,
            "payments_not_configured",
        ));
    };
    let gateway = XenditClient::new(config)
        .map_err(|_| error(StatusCode::SERVICE_UNAVAILABLE, "payments_not_configured"))?;

    let email: Option<String> =
        sqlx::query_scalar("SELECT email FROM web_credentials WHERE user_id = $1")
            .bind(claims.sub)
            .fetch_optional(&state.pool)
            .await
            .map_err(internal)?;
    let payment_id = Uuid::new_v4();
    let external_id = format!("{EXTERNAL_ID_PREFIX}{payment_id}");
    sqlx::query("INSERT INTO plan_payments (id, user_id, plan_id, external_id, amount_idr) VALUES ($1, $2, $3, $4, $5)")
        .bind(payment_id)
        .bind(claims.sub)
        .bind(plan.id)
        .bind(&external_id)
        .bind(price)
        .execute(&state.pool)
        .await
        .map_err(internal)?;

    let back = crate::public_url::link(RETURN_PATH);
    let request = CreateInvoiceRequest {
        external_id,
        amount: price as f64,
        description: Some(format!("Nomi {}", plan.name)),
        payer_email: email,
        invoice_duration: Some(INVOICE_DURATION_SECS),
        customer: None,
        currency: Some(CURRENCY.to_string()),
        success_redirect_url: back.clone(),
        failure_redirect_url: back,
    };
    match gateway.create_invoice(request).await {
        Ok(invoice) => {
            sqlx::query("UPDATE plan_payments SET invoice_id = $2, invoice_url = $3, updated_at = now() WHERE id = $1")
                .bind(payment_id)
                .bind(&invoice.id)
                .bind(&invoice.invoice_url)
                .execute(&state.pool)
                .await
                .map_err(internal)?;
            Ok(Json(json!({ "url": invoice.invoice_url })))
        }
        Err(e) => {
            tracing::warn!(error = %e, user = %claims.sub, "the payment gateway refused a checkout");
            sqlx::query(
                "UPDATE plan_payments SET status = 'failed', updated_at = now() WHERE id = $1",
            )
            .bind(payment_id)
            .execute(&state.pool)
            .await
            .map_err(internal)?;
            Err(error(StatusCode::BAD_GATEWAY, "gateway_error"))
        }
    }
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

/// How a payment was made, as Billing shows it: "BANK_TRANSFER · BCA".
fn method_label(method: Option<&str>, channel: Option<&str>) -> Option<String> {
    let parts: Vec<&str> = [method, channel]
        .into_iter()
        .flatten()
        .map(str::trim)
        .filter(|p| !p.is_empty())
        .collect();
    (!parts.is_empty()).then(|| parts.join(" · "))
}

/// `POST /api/payments/xendit/webhook`: Xendit's callback for an invoice. Only taken with the
/// verification token set in Admin → Payment (anything else could claim a payment), and only for
/// the amount asked. A paid invoice moves the person onto the plan and tells them; asking again
/// about the same invoice changes nothing.
pub async fn xendit_webhook(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<Value>, RouteError> {
    let Some(config) = crate::configuration::xendit_config()
        .await
        .filter(|c| c.webhook_token.is_some())
    else {
        return Err(error(
            StatusCode::SERVICE_UNAVAILABLE,
            "payments_not_configured",
        ));
    };
    let gateway = XenditClient::new(config)
        .map_err(|_| error(StatusCode::SERVICE_UNAVAILABLE, "payments_not_configured"))?;
    let token = headers
        .get(CALLBACK_TOKEN_HEADER)
        .and_then(|v| v.to_str().ok());
    let event = gateway.verify_webhook(token, &body).map_err(|e| {
        tracing::warn!(error = %e, "refused a payment callback");
        error(StatusCode::UNAUTHORIZED, "invalid_callback")
    })?;

    let payment: Option<(Uuid, Uuid, Uuid, i64)> =
        sqlx::query_as("SELECT id, user_id, plan_id, amount_idr FROM plan_payments WHERE external_id = $1 AND status = 'pending'")
            .bind(&event.external_id)
            .fetch_optional(&state.pool)
            .await
            .map_err(internal)?;
    // Not ours, or already settled: answered so the gateway stops asking.
    let Some((payment_id, user_id, plan_id, amount)) = payment else {
        return Ok(Json(json!({ "handled": false })));
    };

    let status = match event.status {
        PaymentStatus::Paid if event.amount.round() as i64 == amount => "paid",
        PaymentStatus::Paid => {
            tracing::warn!(%payment_id, paid = event.amount, asked = amount, "a payment came in for another amount; left pending");
            return Ok(Json(json!({ "handled": false })));
        }
        PaymentStatus::Expired => "expired",
        PaymentStatus::Failed => "failed",
        PaymentStatus::Pending | PaymentStatus::Unknown => {
            return Ok(Json(json!({ "handled": false })))
        }
    };
    let updated = sqlx::query(
        "UPDATE plan_payments SET status = $2, payment_method = $3, paid_at = CASE WHEN $2 = 'paid' THEN COALESCE($4, now()) END, \
             invoice_id = COALESCE(invoice_id, $5), updated_at = now() \
         WHERE id = $1 AND status = 'pending'",
    )
    .bind(payment_id)
    .bind(status)
    .bind(method_label(event.payment_method.as_deref(), event.payment_channel.as_deref()))
    .bind(event.paid_at)
    .bind(&event.invoice_id)
    .execute(&state.pool)
    .await
    .map_err(internal)?;
    if updated.rows_affected() == 1 && status == "paid" {
        let before = plans::subscription_for(&state.pool, user_id)
            .await
            .map_err(internal)?;
        let change = SubscriptionChange {
            plan_id,
            quota_override: None,
            override_until: None,
            note: None,
        };
        plans::change_subscription(&state.pool, user_id, &change, user_id)
            .await
            .map_err(internal)?;
        let after = plans::subscription_for(&state.pool, user_id)
            .await
            .map_err(internal)?;
        crate::routes::plans::notify_change(&state.pool, user_id, &before, &after, None)
            .await
            .map_err(internal)?;
        sqlx::query("DELETE FROM plan_waitlist WHERE user_id = $1 AND plan_id = $2")
            .bind(user_id)
            .bind(plan_id)
            .execute(&state.pool)
            .await
            .map_err(internal)?;
    }
    Ok(Json(json!({ "handled": true })))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_how_it_was_paid() {
        assert_eq!(
            method_label(Some("BANK_TRANSFER"), Some("BCA")).as_deref(),
            Some("BANK_TRANSFER · BCA")
        );
        assert_eq!(
            method_label(Some("EWALLET"), None).as_deref(),
            Some("EWALLET")
        );
        assert_eq!(method_label(None, Some(" ")), None);
    }
}
