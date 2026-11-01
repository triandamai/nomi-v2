use async_trait::async_trait;
use base64::Engine;
use reqwest::{Client, StatusCode};

use crate::error::PaymentError;
use crate::traits::PaymentGateway;
use crate::types::{CreateInvoiceRequest, Invoice, PaymentStatus, WebhookEvent};
use crate::xendit::dto::{XenditErrorResponse, XenditInvoiceResponse, XenditWebhookPayload};

const DEFAULT_XENDIT_BASE_URL: &str = "https://api.xendit.co";

#[derive(Debug, Clone)]
pub struct XenditConfig {
    pub api_key: String,
    pub webhook_token: Option<String>,
    pub base_url: Option<String>,
}

#[derive(Clone)]
pub struct XenditClient {
    client: Client,
    config: XenditConfig,
    base_url: String,
}

impl XenditClient {
    pub fn new(config: XenditConfig) -> Result<Self, PaymentError> {
        if config.api_key.trim().is_empty() {
            return Err(PaymentError::Configuration("Xendit API key cannot be empty".to_string()));
        }

        let base_url = config
            .base_url
            .clone()
            .unwrap_or_else(|| DEFAULT_XENDIT_BASE_URL.to_string())
            .trim_end_matches('/')
            .to_string();

        let client = Client::builder().build()?;

        Ok(Self {
            client,
            config,
            base_url,
        })
    }

    fn auth_header_value(&self) -> String {
        let credentials = format!("{}:", self.config.api_key);
        let encoded = base64::engine::general_purpose::STANDARD.encode(credentials.as_bytes());
        format!("Basic {}", encoded)
    }

    fn map_status(status_str: &str) -> PaymentStatus {
        match status_str.to_uppercase().as_str() {
            "PENDING" => PaymentStatus::Pending,
            "PAID" | "SETTLED" => PaymentStatus::Paid,
            "EXPIRED" => PaymentStatus::Expired,
            _ => PaymentStatus::Failed,
        }
    }

    fn to_domain_invoice(dto: XenditInvoiceResponse) -> Invoice {
        Invoice {
            id: dto.id,
            external_id: dto.external_id,
            user_id: dto.user_id,
            status: Self::map_status(&dto.status),
            amount: dto.amount,
            invoice_url: dto.invoice_url,
            expiry_date: dto.expiry_date,
            paid_at: dto.paid_at,
            payment_method: dto.payment_method,
            currency: dto.currency,
        }
    }

    async fn handle_error_response(response: reqwest::Response) -> PaymentError {
        let status = response.status();
        let status_code = status.as_u16();
        let text = response.text().await.unwrap_or_default();

        let message = if let Ok(err_obj) = serde_json::from_str::<XenditErrorResponse>(&text) {
            let mut parts = Vec::new();
            if let Some(code) = err_obj.error_code {
                parts.push(code);
            }
            if let Some(msg) = err_obj.message {
                parts.push(msg);
            }
            if parts.is_empty() {
                text
            } else {
                parts.join(": ")
            }
        } else {
            text
        };

        match status {
            StatusCode::BAD_REQUEST => PaymentError::BadRequest {
                status: status_code,
                message,
            },
            StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => {
                PaymentError::Unauthorized(message)
            }
            StatusCode::NOT_FOUND => PaymentError::NotFound(message),
            _ => PaymentError::ApiError {
                status: status_code,
                message,
            },
        }
    }
}

#[async_trait]
impl PaymentGateway for XenditClient {
    async fn create_invoice(&self, request: CreateInvoiceRequest) -> Result<Invoice, PaymentError> {
        let url = format!("{}/v2/invoices", self.base_url);

        let response = self
            .client
            .post(&url)
            .header("Authorization", self.auth_header_value())
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(Self::handle_error_response(response).await);
        }

        let dto: XenditInvoiceResponse = response.json().await?;
        Ok(Self::to_domain_invoice(dto))
    }

    async fn get_invoice(&self, invoice_id: &str) -> Result<Invoice, PaymentError> {
        let url = format!("{}/v2/invoices/{}", self.base_url, invoice_id);

        let response = self
            .client
            .get(&url)
            .header("Authorization", self.auth_header_value())
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(Self::handle_error_response(response).await);
        }

        let dto: XenditInvoiceResponse = response.json().await?;
        Ok(Self::to_domain_invoice(dto))
    }

    async fn expire_invoice(&self, invoice_id: &str) -> Result<Invoice, PaymentError> {
        let url = format!("{}/invoices/{}/expire!", self.base_url, invoice_id);

        let response = self
            .client
            .post(&url)
            .header("Authorization", self.auth_header_value())
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(Self::handle_error_response(response).await);
        }

        let dto: XenditInvoiceResponse = response.json().await?;
        Ok(Self::to_domain_invoice(dto))
    }

    fn verify_webhook(
        &self,
        callback_token: Option<&str>,
        payload: &[u8],
    ) -> Result<WebhookEvent, PaymentError> {
        // If a webhook verification token is configured, enforce that callback_token matches it
        if let Some(expected_token) = &self.config.webhook_token {
            match callback_token {
                Some(token) if token == expected_token => {}
                Some(_) => {
                    return Err(PaymentError::WebhookVerificationFailed(
                        "Invalid callback token".to_string(),
                    ))
                }
                None => {
                    return Err(PaymentError::WebhookVerificationFailed(
                        "Missing x-callback-token header".to_string(),
                    ))
                }
            }
        }

        let raw_value: serde_json::Value = serde_json::from_slice(payload)?;
        let parsed: XenditWebhookPayload = serde_json::from_value(raw_value.clone())?;

        let event_type = match parsed.status.to_uppercase().as_str() {
            "PAID" | "SETTLED" => "invoice.paid".to_string(),
            "EXPIRED" => "invoice.expired".to_string(),
            _ => format!("invoice.{}", parsed.status.to_lowercase()),
        };

        Ok(WebhookEvent {
            event_type,
            invoice_id: parsed.id,
            external_id: parsed.external_id,
            status: Self::map_status(&parsed.status),
            amount: parsed.amount,
            paid_at: parsed.paid_at,
            payment_method: parsed.payment_method,
            payment_channel: parsed.payment_channel,
            raw_payload: raw_value,
        })
    }
}
