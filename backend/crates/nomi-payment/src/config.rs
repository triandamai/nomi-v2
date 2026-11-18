use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::error::PaymentError;
use crate::traits::PaymentGateway;
use crate::doku::{DokuClient, DokuConfig};
use crate::xendit::{XenditClient, XenditConfig};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaymentProviderKind {
    Xendit,
    Doku,
}

impl PaymentProviderKind {
    /// Every provider, as `as_str` names them.
    pub const ALL: [PaymentProviderKind; 2] = [PaymentProviderKind::Xendit, PaymentProviderKind::Doku];

    pub fn as_str(&self) -> &'static str {
        match self {
            PaymentProviderKind::Xendit => "xendit",
            PaymentProviderKind::Doku => "doku",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|kind| kind.as_str().eq_ignore_ascii_case(name.trim()))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentConfig {
    pub provider: PaymentProviderKind,
    /// Xendit's secret API key, or DOKU's secret key.
    pub api_key: String,
    /// DOKU's client id.
    #[serde(default)]
    pub client_id: Option<String>,
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
        PaymentProviderKind::Doku => {
            let client = DokuClient::new(DokuConfig {
                client_id: config.client_id.unwrap_or_default(),
                secret_key: config.api_key,
                base_url: config.base_url,
                notification_path: None,
            })?;
            Ok(Arc::new(client))
        }
    }
}
