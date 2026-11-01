use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XenditInvoiceResponse {
    pub id: String,
    pub external_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
    pub status: String,
    pub amount: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payer_email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub invoice_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiry_date: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paid_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payment_method: Option<String>,
    #[serde(default = "default_currency")]
    pub currency: String,
}

fn default_currency() -> String {
    "IDR".to_string()
}

#[derive(Debug, Clone, Deserialize)]
pub struct XenditWebhookPayload {
    pub id: String,
    pub external_id: String,
    #[serde(default)]
    pub user_id: Option<String>,
    pub status: String,
    pub amount: f64,
    #[serde(default)]
    pub paid_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub payment_method: Option<String>,
    #[serde(default)]
    pub payment_channel: Option<String>,
    #[serde(default)]
    pub currency: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct XenditErrorResponse {
    #[serde(default)]
    pub error_code: Option<String>,
    #[serde(default)]
    pub message: Option<String>,
}
