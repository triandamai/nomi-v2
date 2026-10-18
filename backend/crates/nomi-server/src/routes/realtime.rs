//! `GET /api/realtime/ws`: one socket per native app. It always carries the person's own
//! `user/{id}/stream` and, on request, any chat or project they can open — each event wrapped
//! with the topic it came from, so the app can hand it to whoever is listening.

use std::collections::HashSet;
use std::time::Duration;

use axum::extract::ws::{CloseFrame, Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::response::IntoResponse;
use rumqttc::{AsyncClient, Event, MqttOptions, Packet, QoS};
use tokio::sync::mpsc;
use uuid::Uuid;

use nomi_auth::extractor::AuthClaims;
use nomi_realtime::{ClientFrame, ServerFrame, Topic};

use crate::app::AppState;

/// Topics a client may ask for, not counting the implicit user topic every socket has.
const MAX_CLIENT_TOPICS_PER_SOCKET: usize = 64;
const BROKER_EVENT_CAPACITY: usize = 256;
const MQTT_KEEP_ALIVE: Duration = Duration::from_secs(30);
const MQTT_CHANNEL_CAPACITY: usize = 64;
const CLOSE_BROKER_LOST: u16 = 1011;

pub async fn realtime_stream(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| run(socket, state, claims.sub))
}

async fn may_open(state: &AppState, user_id: Uuid, topic: Topic) -> bool {
    match topic {
        Topic::User(id) => id == user_id,
        Topic::Chat(session_id) => crate::routes::sessions::authorize_session_access(&state.pool, user_id, session_id).await.is_ok(),
        Topic::Project(project_id) => crate::routes::deploy::project_role(&state.pool, project_id, user_id).await.is_ok(),
    }
}

async fn send(socket: &mut WebSocket, frame: &ServerFrame) -> bool {
    let text = serde_json::to_string(frame).expect("server frames always serialize");
    socket.send(Message::Text(text)).await.is_ok()
}

/// What the broker driver task reports to the socket task.
enum BrokerEvent {
    Publish { topic: String, payload: Vec<u8> },
    Lost(String),
}

/// Aborts the MQTT driver task when the socket task ends, so no broker connection leaks.
struct AbortOnDrop(tokio::task::JoinHandle<()>);
impl Drop for AbortOnDrop {
    fn drop(&mut self) {
        self.0.abort();
    }
}

/// The canonical wire name when the client's string parses to a topic, otherwise the raw string.
fn echo(parsed: Option<Topic>, raw: String) -> String {
    parsed.map(|t| t.wire()).unwrap_or(raw)
}

async fn run(mut socket: WebSocket, state: AppState, user_id: Uuid) {
    let mut options = MqttOptions::new(format!("realtime-gw-{}", Uuid::new_v4()), state.mqtt_broker_host.clone(), state.mqtt_broker_port);
    options.set_keep_alive(MQTT_KEEP_ALIVE);
    let (client, mut eventloop) = AsyncClient::new(options, MQTT_CHANNEL_CAPACITY);

    // The event loop is polled only here, in its own task: it is not cancel-safe, and the
    // socket task must be free to await subscribe/unsubscribe (which the loop has to drain).
    let (broker_tx, mut broker_rx) = mpsc::channel::<BrokerEvent>(BROKER_EVENT_CAPACITY);
    let _driver = AbortOnDrop(tokio::spawn(async move {
        loop {
            match eventloop.poll().await {
                Ok(Event::Incoming(Packet::Publish(p))) => {
                    if broker_tx.send(BrokerEvent::Publish { topic: p.topic, payload: p.payload.to_vec() }).await.is_err() {
                        return;
                    }
                }
                Ok(_) => {}
                Err(e) => {
                    let _ = broker_tx.send(BrokerEvent::Lost(e.to_string())).await;
                    return;
                }
            }
        }
    }));

    let user_topic = Topic::User(user_id);
    if client.subscribe(user_topic.mqtt_topic(), QoS::AtMostOnce).await.is_err() {
        return;
    }
    let mut topics: HashSet<Topic> = HashSet::from([user_topic]); // always holds the user topic
    if !send(&mut socket, &ServerFrame::Ready { user_id }).await {
        return;
    }

    loop {
        tokio::select! {
            event = broker_rx.recv() => match event {
                Some(BrokerEvent::Publish { topic, payload }) => {
                    let Some(topic) = Topic::from_mqtt(&topic) else { continue };
                    if !topics.contains(&topic) { continue; } // a late message after unsubscribe
                    let Ok(payload) = serde_json::from_slice(&payload) else { continue };
                    if !send(&mut socket, &ServerFrame::Event { topic: topic.wire(), payload }).await { break; }
                }
                Some(BrokerEvent::Lost(error)) => {
                    tracing::warn!(%error, %user_id, "realtime gateway lost the broker");
                    let _ = socket.send(Message::Close(Some(CloseFrame { code: CLOSE_BROKER_LOST, reason: "broker unavailable".into() }))).await;
                    break;
                }
                None => break,
            },
            incoming = socket.recv() => {
                let Some(Ok(msg)) = incoming else { break };
                let Message::Text(text) = msg else { continue };
                let reply = match serde_json::from_str::<ClientFrame>(&text) {
                    Err(_) => ServerFrame::Error { message: "unreadable frame".into() },
                    Ok(ClientFrame::Ping) => ServerFrame::Pong,
                    Ok(ClientFrame::Subscribe { topic: name }) => {
                        let parsed = Topic::parse_client(&name);
                        match parsed {
                            Some(topic) if topics.contains(&topic) => ServerFrame::Subscribed { topic: topic.wire() },
                            Some(topic) if topics.len() - 1 < MAX_CLIENT_TOPICS_PER_SOCKET && may_open(&state, user_id, topic).await => {
                                if client.subscribe(topic.mqtt_topic(), QoS::AtMostOnce).await.is_err() { break; }
                                topics.insert(topic);
                                ServerFrame::Subscribed { topic: topic.wire() }
                            }
                            _ => ServerFrame::Denied { topic: echo(parsed, name) },
                        }
                    }
                    Ok(ClientFrame::Unsubscribe { topic: name }) => {
                        let parsed = Topic::parse_client(&name);
                        if let Some(topic) = parsed.filter(|t| topics.remove(t)) {
                            let _ = client.unsubscribe(topic.mqtt_topic()).await;
                        }
                        ServerFrame::Unsubscribed { topic: echo(parsed, name) }
                    }
                };
                if !send(&mut socket, &reply).await { break; }
            }
        }
    }
}
