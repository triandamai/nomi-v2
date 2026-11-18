//! DOKU Checkout (https://developers.doku.com/accept-payments/doku-checkout): a DOKU-hosted
//! payment page per invoice, its status, cancelling it, and DOKU's signed HTTP Notifications.
//!
//! Every request is signed with HMAC-SHA256 over the Client-Id, Request-Id, Request-Timestamp,
//! Request-Target (the path) and, for a body, its SHA-256 Digest; DOKU signs its notifications
//! the same way, with our notification URL's path as the target.

pub mod dto;

use async_trait::async_trait;
use base64::Engine;
use chrono::{DateTime, FixedOffset, NaiveDateTime, Utc};
use hmac::{Hmac, Mac};
use reqwest::{Client, Method, StatusCode};
use sha2::{Digest, Sha256};

use crate::error::PaymentError;
use crate::traits::PaymentGateway;
use crate::types::{CreateInvoiceRequest, Invoice, PaymentStatus, WebhookEvent};
use dto::{CancelOrder, CancelPayment, CancelRequest, CheckoutCustomer, CheckoutOrder, CheckoutPayment, CheckoutRequest, CheckoutResponse, ErrorResponse, OrderStatus};

/// Production. The sandbox is `https://api-sandbox.doku.com`.
pub const DEFAULT_DOKU_BASE_URL: &str = "https://api.doku.com";
/// Where DOKU's notifications reach the server, when the admin hasn't said otherwise.
pub const DEFAULT_NOTIFICATION_PATH: &str = "/api/payments/doku/webhook";

const CHECKOUT_PATH: &str = "/checkout/v1/payment";
const STATUS_PATH: &str = "/orders/v1/status/";
const CANCEL_PATH: &str = "/checkout/v3/cancellations";
const CANCEL_NOTE: &str = "Cancelled in Nomi";
const DEFAULT_CURRENCY: &str = "IDR";
const SIGNATURE_PREFIX: &str = "HMACSHA256=";
const TIMESTAMP_FORMAT: &str = "%Y-%m-%dT%H:%M:%SZ";
/// How DOKU writes a checkout page's expiry.
const EXPIRY_FORMAT: &str = "%Y%m%d%H%M%S";
/// DOKU's expiry is in Jakarta time, UTC+7.
const JAKARTA_OFFSET_SECS: i32 = 7 * 3600;
const SECONDS_PER_MINUTE: u32 = 60;

const HEADER_CLIENT_ID: &str = "Client-Id";
const HEADER_REQUEST_ID: &str = "Request-Id";
const HEADER_TIMESTAMP: &str = "Request-Timestamp";
const HEADER_SIGNATURE: &str = "Signature";

/// A payment id (a UUID) is 36 characters, and DOKU takes 30 at most when cards are on: ours go
/// as this prefix and the id's 128 bits in base 36, 27 characters at most.
const INVOICE_PREFIX: &str = "NM";
const INVOICE_RADIX: u32 = 36;
const UUID_HEX_LEN: usize = 32;
const UUID_GROUPS: [usize; 5] = [8, 4, 4, 4, 12];

#[derive(Debug, Clone)]
pub struct DokuConfig {
    pub client_id: String,
    pub secret_key: String,
    pub base_url: Option<String>,
    /// The path of the notification URL set in DOKU's dashboard; DOKU signs with it.
    pub notification_path: Option<String>,
}

#[derive(Clone)]
pub struct DokuClient {
    client: Client,
    config: DokuConfig,
    base_url: String,
    notification_path: String,
}

/// DOKU's request signature: `HMACSHA256=` and the base64 HMAC-SHA256, keyed with the secret
/// key, of the components one per line (`Digest` only when there is a body).
pub fn signature(secret_key: &str, client_id: &str, request_id: &str, timestamp: &str, target: &str, body: Option<&[u8]>) -> String {
    let mut components = format!("{HEADER_CLIENT_ID}:{client_id}\n{HEADER_REQUEST_ID}:{request_id}\n{HEADER_TIMESTAMP}:{timestamp}\nRequest-Target:{target}");
    if let Some(body) = body {
        let digest = base64::engine::general_purpose::STANDARD.encode(Sha256::digest(body));
        components.push_str(&format!("\nDigest:{digest}"));
    }
    let mut mac = Hmac::<Sha256>::new_from_slice(secret_key.as_bytes()).expect("HMAC takes a key of any length");
    mac.update(components.as_bytes());
    format!("{SIGNATURE_PREFIX}{}", base64::engine::general_purpose::STANDARD.encode(mac.finalize().into_bytes()))
}

/// The invoice number a payment goes to DOKU under: a UUID shortened (see `INVOICE_PREFIX`),
/// anything else as it is.
pub fn invoice_number_for(external_id: &str) -> String {
    let hex: String = external_id.chars().filter(|c| *c != '-').collect();
    let is_uuid = external_id.len() == UUID_HEX_LEN + UUID_GROUPS.len() - 1 && hex.len() == UUID_HEX_LEN;
    match u128::from_str_radix(&hex, 16) {
        Ok(mut n) if is_uuid => {
            let mut digits = Vec::new();
            while n > 0 {
                digits.push(char::from_digit((n % u128::from(INVOICE_RADIX)) as u32, INVOICE_RADIX).unwrap_or('0').to_ascii_uppercase());
                n /= u128::from(INVOICE_RADIX);
            }
            if digits.is_empty() {
                digits.push('0');
            }
            format!("{INVOICE_PREFIX}{}", digits.into_iter().rev().collect::<String>())
        }
        _ => external_id.to_string(),
    }
}

/// The payment id back from `invoice_number_for`'s invoice number.
pub fn external_id_from(invoice_number: &str) -> String {
    let decoded = invoice_number.strip_prefix(INVOICE_PREFIX).and_then(|rest| u128::from_str_radix(rest, INVOICE_RADIX).ok());
    let Some(n) = decoded else { return invoice_number.to_string() };
    let hex = format!("{n:0width$x}", width = UUID_HEX_LEN);
    let mut groups = Vec::with_capacity(UUID_GROUPS.len());
    let mut at = 0;
    for len in UUID_GROUPS {
        groups.push(&hex[at..at + len]);
        at += len;
    }
    groups.join("-")
}

/// Whether two strings are equal, taking as long whatever they share.
fn constant_time_eq(a: &str, b: &str) -> bool {
    let (a, b) = (a.as_bytes(), b.as_bytes());
    let mut diff = a.len() ^ b.len();
    for i in 0..a.len().max(b.len()) {
        diff |= usize::from(a.get(i).copied().unwrap_or(0) ^ b.get(i).copied().unwrap_or(0));
    }
    diff == 0
}

fn utc(date: Option<&str>) -> Option<DateTime<Utc>> {
    date.and_then(|d| DateTime::parse_from_rfc3339(d).ok()).map(|d| d.with_timezone(&Utc))
}

fn jakarta_expiry(expired_date: Option<&str>) -> Option<DateTime<Utc>> {
    let naive = NaiveDateTime::parse_from_str(expired_date?, EXPIRY_FORMAT).ok()?;
    let jakarta = FixedOffset::east_opt(JAKARTA_OFFSET_SECS)?;
    naive.and_local_timezone(jakarta).single().map(|d| d.with_timezone(&Utc))
}

/// What an order's status means here. A failed try isn't the end: on DOKU's page the person can
/// pay another way, so DOKU says to ignore `FAILED` for checkout.
fn map_status(status: &OrderStatus) -> PaymentStatus {
    let transaction = status.transaction.status.as_deref().unwrap_or_default().to_uppercase();
    let order = status.order.status.as_deref().unwrap_or_default().to_uppercase();
    match transaction.as_str() {
        "SUCCESS" => PaymentStatus::Paid,
        "EXPIRED" => PaymentStatus::Expired,
        _ if order == "ORDER_EXPIRED" => PaymentStatus::Expired,
        _ => PaymentStatus::Pending,
    }
}

impl DokuClient {
    pub fn new(config: DokuConfig) -> Result<Self, PaymentError> {
        if config.client_id.trim().is_empty() {
            return Err(PaymentError::Configuration("DOKU client ID cannot be empty".to_string()));
        }
        if config.secret_key.trim().is_empty() {
            return Err(PaymentError::Configuration("DOKU secret key cannot be empty".to_string()));
        }
        let base_url = config.base_url.clone().filter(|u| !u.trim().is_empty()).unwrap_or_else(|| DEFAULT_DOKU_BASE_URL.to_string()).trim_end_matches('/').to_string();
        let notification_path = config.notification_path.clone().filter(|p| !p.trim().is_empty()).unwrap_or_else(|| DEFAULT_NOTIFICATION_PATH.to_string());
        let client = Client::builder().build()?;
        Ok(Self { client, config, base_url, notification_path })
    }

    /// Sends a request signed as DOKU asks, and turns a refusal into a `PaymentError`.
    async fn send(&self, method: Method, path: &str, request_id: &str, body: Option<Vec<u8>>) -> Result<reqwest::Response, PaymentError> {
        let timestamp = Utc::now().format(TIMESTAMP_FORMAT).to_string();
        let signature = signature(&self.config.secret_key, &self.config.client_id, request_id, &timestamp, path, body.as_deref());
        let mut request = self
            .client
            .request(method, format!("{}{path}", self.base_url))
            .header(HEADER_CLIENT_ID, &self.config.client_id)
            .header(HEADER_REQUEST_ID, request_id)
            .header(HEADER_TIMESTAMP, timestamp)
            .header(HEADER_SIGNATURE, signature);
        if let Some(body) = body {
            request = request.header("Content-Type", "application/json").body(body);
        }
        let response = request.send().await?;
        if response.status().is_success() {
            Ok(response)
        } else {
            Err(Self::error(response).await)
        }
    }

    async fn error(response: reqwest::Response) -> PaymentError {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        let message = serde_json::from_str::<ErrorResponse>(&text)
            .ok()
            .map(|e| if e.error_messages.is_empty() { e.message } else { e.error_messages })
            .filter(|parts| !parts.is_empty())
            .map(|parts| parts.join("; "))
            .unwrap_or(text);
        match status {
            StatusCode::BAD_REQUEST => PaymentError::BadRequest { status: status.as_u16(), message },
            StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => PaymentError::Unauthorized(message),
            StatusCode::NOT_FOUND => PaymentError::NotFound(message),
            _ => PaymentError::ApiError { status: status.as_u16(), message },
        }
    }

    fn invoice_from(invoice_number: &str, status: &OrderStatus) -> Invoice {
        let paid = map_status(status) == PaymentStatus::Paid;
        Invoice {
            id: invoice_number.to_string(),
            external_id: external_id_from(invoice_number),
            user_id: None,
            status: map_status(status),
            amount: status.order.amount,
            invoice_url: String::new(),
            expiry_date: None,
            paid_at: if paid { utc(status.transaction.date.as_deref()) } else { None },
            payment_method: status.channel.id.clone().or_else(|| status.service.id.clone()),
            currency: DEFAULT_CURRENCY.to_string(),
        }
    }
}

#[async_trait]
impl PaymentGateway for DokuClient {
    async fn create_invoice(&self, request: CreateInvoiceRequest) -> Result<Invoice, PaymentError> {
        let invoice_number = invoice_number_for(&request.external_id);
        let currency = request.currency.clone().unwrap_or_else(|| DEFAULT_CURRENCY.to_string());
        let body = CheckoutRequest {
            order: CheckoutOrder {
                // DOKU takes whole rupiah.
                amount: request.amount.round() as i64,
                invoice_number: invoice_number.clone(),
                currency: currency.clone(),
                callback_url: request.success_redirect_url.clone(),
                callback_url_result: request.success_redirect_url.clone(),
                auto_redirect: request.success_redirect_url.is_some(),
            },
            payment: CheckoutPayment { payment_due_date: request.invoice_duration.map(|secs| (secs / SECONDS_PER_MINUTE).max(1)) },
            customer: request.payer_email.clone().map(|email| CheckoutCustomer { email }),
        };
        // The invoice number is the request id too, so cancelling can name it.
        let response = self.send(Method::POST, CHECKOUT_PATH, &invoice_number, Some(serde_json::to_vec(&body)?)).await?;
        let made: CheckoutResponse = response.json().await?;
        Ok(Invoice {
            id: invoice_number,
            external_id: request.external_id,
            user_id: None,
            status: PaymentStatus::Pending,
            amount: if made.response.order.amount > 0.0 { made.response.order.amount } else { request.amount },
            invoice_url: made.response.payment.url,
            expiry_date: jakarta_expiry(made.response.payment.expired_date.as_deref()),
            paid_at: None,
            payment_method: None,
            currency,
        })
    }

    async fn get_invoice(&self, invoice_id: &str) -> Result<Invoice, PaymentError> {
        let request_id = format!("{invoice_id}-{}", Utc::now().timestamp_millis());
        let response = self.send(Method::GET, &format!("{STATUS_PATH}{invoice_id}"), &request_id, None).await?;
        let status: OrderStatus = response.json().await?;
        Ok(Self::invoice_from(invoice_id, &status))
    }

    /// Cancels an unpaid order (only bank transfer and QRIS orders can be, and only once "Order
    /// Cancellation" is on in DOKU's dashboard; otherwise DOKU refuses and this says so).
    async fn expire_invoice(&self, invoice_id: &str) -> Result<Invoice, PaymentError> {
        let body = CancelRequest {
            order: CancelOrder { invoice_number: invoice_id.to_string() },
            payment: CancelPayment { original_request_id: invoice_id.to_string() },
            note: CANCEL_NOTE.to_string(),
        };
        let request_id = format!("{invoice_id}-cancel-{}", Utc::now().timestamp_millis());
        self.send(Method::POST, CANCEL_PATH, &request_id, Some(serde_json::to_vec(&body)?)).await?;
        let mut invoice = Self::invoice_from(invoice_id, &OrderStatus::default());
        invoice.status = PaymentStatus::Expired;
        Ok(invoice)
    }

    fn verify_webhook(&self, headers: &http::HeaderMap, payload: &[u8]) -> Result<WebhookEvent, PaymentError> {
        let header = |name: &str| headers.get(name).and_then(|v| v.to_str().ok()).map(str::trim);
        let refuse = |why: &str| Err(PaymentError::WebhookVerificationFailed(why.to_string()));
        let (Some(client_id), Some(request_id), Some(timestamp), Some(given)) = (header(HEADER_CLIENT_ID), header(HEADER_REQUEST_ID), header(HEADER_TIMESTAMP), header(HEADER_SIGNATURE)) else {
            return refuse("missing DOKU signature headers");
        };
        if !constant_time_eq(client_id, &self.config.client_id) {
            return refuse("a notification for another client id");
        }
        let expected = signature(&self.config.secret_key, client_id, request_id, timestamp, &self.notification_path, Some(payload));
        if !constant_time_eq(given, &expected) {
            return refuse("invalid signature");
        }

        let raw_payload: serde_json::Value = serde_json::from_slice(payload)?;
        let status: OrderStatus = serde_json::from_value(raw_payload.clone())?;
        let invoice_id = status.order.invoice_number.clone().unwrap_or_default();
        let mapped = map_status(&status);
        let event_type = match mapped {
            PaymentStatus::Paid => "invoice.paid".to_string(),
            PaymentStatus::Expired => "invoice.expired".to_string(),
            _ => format!("invoice.{}", status.transaction.status.as_deref().unwrap_or("pending").to_lowercase()),
        };
        Ok(WebhookEvent {
            event_type,
            external_id: external_id_from(&invoice_id),
            invoice_id,
            amount: status.order.amount,
            paid_at: if mapped == PaymentStatus::Paid { utc(status.transaction.date.as_deref()) } else { None },
            status: mapped,
            payment_method: status.service.id.clone(),
            payment_channel: status.channel.id.clone(),
            raw_payload,
        })
    }
}
