use async_trait::async_trait;
use uuid::Uuid;

/// Seam for pushing a fired reminder's result somewhere beyond the in-app chat message that
/// `scheduler_worker.rs` always posts unconditionally. `LogOnlyDelivery` is the only concrete
/// implementation today — real Telegram/WhatsApp/web-push integrations are future work that
/// implement this same trait with no changes needed to the scheduler itself.
#[async_trait]
pub trait NotificationDelivery: Send + Sync {
    async fn deliver(&self, user_id: Uuid, message: &str) -> Result<(), String>;
}

pub struct LogOnlyDelivery;

#[async_trait]
impl NotificationDelivery for LogOnlyDelivery {
    async fn deliver(&self, user_id: Uuid, message: &str) -> Result<(), String> {
        tracing::info!(%user_id, %message, "notification delivery not yet configured — logging only");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn log_only_delivery_always_succeeds() {
        let delivery = LogOnlyDelivery;
        let result = delivery.deliver(Uuid::new_v4(), "test message").await;
        assert!(result.is_ok());
    }
}
