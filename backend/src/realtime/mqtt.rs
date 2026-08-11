use std::time::Duration;

use rumqttc::{AsyncClient, MqttOptions, QoS};
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, thiserror::Error)]
pub enum MqttError {
    #[error("mqtt publish failed: {0}")]
    Publish(#[from] rumqttc::ClientError),
    #[error("failed to serialize stream envelope: {0}")]
    Serialize(#[from] serde_json::Error),
}

#[derive(Clone)]
pub struct MqttPublisher {
    client: AsyncClient,
}

impl MqttPublisher {
    pub fn connect(broker_host: &str, broker_port: u16, client_id: &str) -> Self {
        let mut options = MqttOptions::new(client_id, broker_host, broker_port);
        options.set_keep_alive(Duration::from_secs(30));
        let (client, mut eventloop) = AsyncClient::new(options, 64);

        // rumqttc requires the EventLoop to be polled continuously to make progress (send
        // outgoing publishes, receive acks) — this task drives it for the process's lifetime.
        // A connection error just gets retried on the next poll; it never surfaces to publish()
        // callers directly (matching this project's fail-open policy for MQTT — see Task 5).
        tokio::spawn(async move {
            loop {
                if let Err(e) = eventloop.poll().await {
                    tracing::warn!(error = %e, "mqtt eventloop error, retrying");
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            }
        });

        Self { client }
    }

    pub async fn publish<T: Serialize>(&self, session_id: Uuid, envelope: &T) -> Result<(), MqttError> {
        let payload = serde_json::to_vec(envelope)?;
        let topic = format!("chat/{session_id}/stream");
        self.client.publish(topic, QoS::AtMostOnce, false, payload).await?;
        Ok(())
    }
}
