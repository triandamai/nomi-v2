use async_trait::async_trait;
use crate::error::PaymentError;
use crate::types::{CreateInvoiceRequest, Invoice, WebhookEvent};

#[async_trait]
pub trait PaymentGateway: Send + Sync {
    /// Create a payment invoice / checkout session.
    async fn create_invoice(&self, request: CreateInvoiceRequest) -> Result<Invoice, PaymentError>;

    /// Retrieve invoice details by invoice ID.
    async fn get_invoice(&self, invoice_id: &str) -> Result<Invoice, PaymentError>;

    /// Manually expire an open invoice.
    async fn expire_invoice(&self, invoice_id: &str) -> Result<Invoice, PaymentError>;

    /// Verify incoming webhook callback token/signature and parse payload.
    fn verify_webhook(
        &self,
        callback_token: Option<&str>,
        payload: &[u8],
    ) -> Result<WebhookEvent, PaymentError>;
}
