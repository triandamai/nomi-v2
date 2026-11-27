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

/// The largest packet a Nomi client sends or accepts: EMQX's own default `max_packet_size`.
/// rumqttc's default is 10 KB, which a long tool result or a big view goes past.
pub const MAX_PACKET_BYTES: usize = 1024 * 1024;

/// Options for a connection to the broker, allowing packets up to `MAX_PACKET_BYTES` both ways.
pub fn broker_options(client_id: impl Into<String>, broker_host: impl Into<String>, broker_port: u16) -> MqttOptions {
    let mut options = MqttOptions::new(client_id, broker_host, broker_port);
    options.set_max_packet_size(MAX_PACKET_BYTES, MAX_PACKET_BYTES);
    options
}

impl MqttPublisher {
    pub fn connect(broker_host: &str, broker_port: u16, client_id: &str) -> Self {
        let mut options = broker_options(client_id, broker_host, broker_port);
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

    /// Sends to any topic without waiting: when the broker is unreachable and the queue is full,
    /// the message is dropped rather than holding up the caller (a request handler, say).
    pub fn try_publish_to<T: Serialize>(&self, topic: &str, envelope: &T) -> Result<(), MqttError> {
        let payload = serde_json::to_vec(envelope)?;
        self.client.try_publish(topic, QoS::AtMostOnce, false, payload)?;
        Ok(())
    }

    /// Sends to everything the person has open (see `crate::UserEnvelope`).
    pub async fn publish_to_user<T: Serialize>(&self, user_id: Uuid, envelope: &T) -> Result<(), MqttError> {
        let payload = serde_json::to_vec(envelope)?;
        self.client.publish(user_topic(user_id), QoS::AtMostOnce, false, payload).await?;
        Ok(())
    }

    pub async fn publish<T: Serialize>(&self, session_id: Uuid, envelope: &T) -> Result<(), MqttError> {
        let payload = serde_json::to_vec(envelope)?;
        let topic = format!("chat/{session_id}/stream");
        self.client.publish(topic, QoS::AtMostOnce, false, payload).await?;
        Ok(())
    }
}

/// A person's own topic: what concerns them whichever chat they have open.
pub fn user_topic(user_id: Uuid) -> String {
    format!("user/{user_id}/activity")
}
