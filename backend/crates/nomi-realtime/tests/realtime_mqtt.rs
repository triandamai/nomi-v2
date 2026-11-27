use std::time::Duration;

use rumqttc::{AsyncClient, Event, MqttOptions, Packet, QoS};
use uuid::Uuid;

use nomi_realtime::MqttPublisher;

// Assumes the EMQX service from backend/docker-compose.yml is running on localhost:1883,
// matching how sqlx::test assumes a running local Postgres.
const BROKER_HOST: &str = "localhost";
const BROKER_PORT: u16 = 1883;

#[tokio::test]
async fn a_published_envelope_is_received_by_a_subscriber_on_the_session_topic() {
    let session_id = Uuid::new_v4();
    let topic = format!("chat/{session_id}/stream");

    let mut sub_options = MqttOptions::new(format!("test-sub-{session_id}"), BROKER_HOST, BROKER_PORT);
    sub_options.set_keep_alive(Duration::from_secs(5));
    let (subscriber, mut sub_eventloop) = AsyncClient::new(sub_options, 16);
    subscriber.subscribe(&topic, QoS::AtMostOnce).await.unwrap();

    // Drain the SubAck before publishing, so we don't race the subscribe confirmation.
    loop {
        match sub_eventloop.poll().await.unwrap() {
            Event::Incoming(Packet::SubAck(_)) => break,
            _ => continue,
        }
    }

    let publisher = MqttPublisher::connect(BROKER_HOST, BROKER_PORT, &format!("test-pub-{session_id}"));
    #[derive(serde::Serialize)]
    struct Payload {
        value: String,
    }
    publisher.publish(session_id, &Payload { value: "hello".to_string() }).await.unwrap();

    let received = tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            if let Event::Incoming(Packet::Publish(publish)) = sub_eventloop.poll().await.unwrap() {
                return publish.payload;
            }
        }
    })
    .await
    .expect("timed out waiting for the published message");

    let payload: serde_json::Value = serde_json::from_slice(&received).unwrap();
    assert_eq!(payload["value"], "hello");
}

/// A stream envelope well past rumqttc's own 10 KB default (a long tool result, a big view)
/// still reaches the chat: the clients allow what the broker allows.
#[tokio::test]
async fn an_envelope_larger_than_ten_kilobytes_is_still_delivered() {
    const LARGE_VALUE_BYTES: usize = 20 * 1024;
    let session_id = Uuid::new_v4();
    let topic = format!("chat/{session_id}/stream");

    let mut sub_options = nomi_realtime::broker_options(format!("test-sub-large-{session_id}"), BROKER_HOST, BROKER_PORT);
    sub_options.set_keep_alive(Duration::from_secs(5));
    let (subscriber, mut sub_eventloop) = AsyncClient::new(sub_options, 16);
    subscriber.subscribe(&topic, QoS::AtMostOnce).await.unwrap();
    loop {
        if let Event::Incoming(Packet::SubAck(_)) = sub_eventloop.poll().await.unwrap() {
            break;
        }
    }

    let publisher = MqttPublisher::connect(BROKER_HOST, BROKER_PORT, &format!("test-pub-large-{session_id}"));
    #[derive(serde::Serialize)]
    struct Payload {
        value: String,
    }
    let value = "x".repeat(LARGE_VALUE_BYTES);
    publisher.publish(session_id, &Payload { value: value.clone() }).await.unwrap();

    let received = tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            if let Event::Incoming(Packet::Publish(publish)) = sub_eventloop.poll().await.unwrap() {
                return publish.payload;
            }
        }
    })
    .await
    .expect("timed out waiting for the large message");
    let payload: serde_json::Value = serde_json::from_slice(&received).unwrap();
    assert_eq!(payload["value"].as_str().map(str::len), Some(LARGE_VALUE_BYTES));
}
