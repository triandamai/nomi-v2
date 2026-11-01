use thiserror::Error;

#[derive(Error, Debug)]
pub enum PaymentError {
    #[error("HTTP request failed: {0}")]
    Network(#[from] reqwest::Error),

    #[error("JSON serialization/deserialization failed: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Authentication failed: {0}")]
    Unauthorized(String),

    #[error("Bad request ({status}): {message}")]
    BadRequest { status: u16, message: String },

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Gateway API error ({status}): {message}")]
    ApiError { status: u16, message: String },

    #[error("Webhook verification failed: {0}")]
    WebhookVerificationFailed(String),

    #[error("Invalid configuration: {0}")]
    Configuration(String),
}
