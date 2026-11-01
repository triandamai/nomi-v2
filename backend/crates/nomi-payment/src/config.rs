use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::error::PaymentError;
use crate::traits::PaymentGateway;
use crate::xendit::{XenditClient, XenditConfig};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaymentProviderKind {
    Xendit,
    // Prepared for future providers:
    // Midtrans,
    // Stripe,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentConfig {
    pub provider: PaymentProviderKind,
    pub api_key: String,
    #[serde(default)]
    pub webhook_token: Option<String>,
    #[serde(default)]
    pub base_url: Option<String>,
}

pub fn build_payment_gateway(config: PaymentConfig) -> Result<Arc<dyn PaymentGateway>, PaymentError> {
    match config.provider {
        PaymentProviderKind::Xendit => {
            let xendit_config = XenditConfig {
                api_key: config.api_key,
                webhook_token: config.webhook_token,
                base_url: config.base_url,
            };
            let client = XenditClient::new(xendit_config)?;
            Ok(Arc::new(client))
        }
    }
}
