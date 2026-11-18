//! What DOKU's Checkout API sends and returns, read leniently: DOKU adds fields over time, and
//! sends an amount as a number in one answer and as a string in another.

use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Serialize)]
pub struct CheckoutRequest {
    pub order: CheckoutOrder,
    pub payment: CheckoutPayment,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer: Option<CheckoutCustomer>,
}

#[derive(Debug, Serialize)]
pub struct CheckoutOrder {
    pub amount: i64,
    pub invoice_number: String,
    pub currency: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub callback_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub callback_url_result: Option<String>,
    pub auto_redirect: bool,
}

#[derive(Debug, Serialize)]
pub struct CheckoutPayment {
    /// Minutes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payment_due_date: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct CheckoutCustomer {
    pub email: String,
}

#[derive(Debug, Serialize)]
pub struct CancelRequest {
    pub order: CancelOrder,
    pub payment: CancelPayment,
    pub note: String,
}

#[derive(Debug, Serialize)]
pub struct CancelOrder {
    pub invoice_number: String,
}

#[derive(Debug, Serialize)]
pub struct CancelPayment {
    pub original_request_id: String,
}

#[derive(Debug, Deserialize)]
pub struct CheckoutResponse {
    pub response: CheckoutResponseBody,
}

#[derive(Debug, Deserialize)]
pub struct CheckoutResponseBody {
    pub order: OrderInfo,
    pub payment: CheckoutPaymentInfo,
}

#[derive(Debug, Deserialize)]
pub struct CheckoutPaymentInfo {
    pub url: String,
    /// `yyyyMMddHHmmss`, in Jakarta time.
    #[serde(default)]
    pub expired_date: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
pub struct OrderInfo {
    #[serde(default)]
    pub invoice_number: Option<String>,
    #[serde(default, deserialize_with = "lenient_amount")]
    pub amount: f64,
    /// Check Status only: `ORDER_GENERATED`, `ORDER_EXPIRED` or `ORDER_RECOVERED`.
    #[serde(default)]
    pub status: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
pub struct TransactionInfo {
    /// `PENDING`, `SUCCESS`, `FAILED` or `EXPIRED`.
    #[serde(default)]
    pub status: Option<String>,
    /// When it happened, in UTC (`2021-01-27T03:24:23Z`).
    #[serde(default)]
    pub date: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
pub struct Id {
    #[serde(default)]
    pub id: Option<String>,
}

/// A Check Status answer and an HTTP Notification have the same shape.
#[derive(Debug, Default, Deserialize)]
pub struct OrderStatus {
    #[serde(default)]
    pub order: OrderInfo,
    #[serde(default)]
    pub transaction: TransactionInfo,
    #[serde(default)]
    pub service: Id,
    #[serde(default)]
    pub channel: Id,
}

#[derive(Debug, Default, Deserialize)]
pub struct ErrorResponse {
    #[serde(default)]
    pub error_messages: Vec<String>,
    #[serde(default)]
    pub message: Vec<String>,
}

fn lenient_amount<'de, D: Deserializer<'de>>(deserializer: D) -> Result<f64, D::Error> {
    Ok(match serde_json::Value::deserialize(deserializer)? {
        serde_json::Value::Number(n) => n.as_f64().unwrap_or_default(),
        serde_json::Value::String(s) => s.trim().parse().unwrap_or_default(),
        _ => 0.0,
    })
}
