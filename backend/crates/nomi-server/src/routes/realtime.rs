//! `GET /api/realtime/ws`: one socket per native app. It always carries the person's own
//! `user/{id}/stream` and, on request, any chat or project they can open — each event wrapped
//! with the topic it came from, so the app can hand it to whoever is listening.

use std::collections::HashSet;
use std::time::Duration;

use axum::extract::ws::{CloseFrame, Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::http::StatusCode;
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
/// Private-range close code: the access token the socket was opened with has expired. The app
/// reconnects with a fresh one.
const CLOSE_TOKEN_EXPIRED: u16 = 4001;

pub async fn realtime_stream(
    State(state): State<AppState>,
    AuthClaims(claims): AuthClaims,
    ws: WebSocketUpgrade,
) -> axum::response::Response {
    let expires_in = token_time_left(claims.exp, chrono::Utc::now().timestamp());
    // The extractor allows a little leeway past `exp`; such a socket would be closed with 4001 straight away, so
    // answer 401 and let the client refresh first.
    if expires_in.is_zero() {
        return (StatusCode::UNAUTHORIZED, "token expired").into_response();
    }
    ws.on_upgrade(move |socket| run(socket, state, claims.sub, expires_in))
}

/// How long until a token whose `exp` is `exp` runs out, as of `now` (both unix seconds).
fn token_time_left(exp: i64, now: i64) -> Duration {
    Duration::from_secs(u64::try_from(exp - now).unwrap_or(0))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Access {
    Granted,
    /// The person may not open it (or it doesn't exist).
    Denied,
    /// The check itself failed server-side; it may well succeed later.
    Unavailable,
}

fn access_from(result: Result<(), StatusCode>) -> Access {
    match result {
        Ok(()) => Access::Granted,
        Err(status) if status.is_server_error() => Access::Unavailable,
        Err(_) => Access::Denied,
    }
}

/// The reply to a subscribe that was not granted.
fn refusal(access: Access, topic: String) -> ServerFrame {
    match access {
        Access::Unavailable => ServerFrame::Unavailable { topic },
        Access::Granted | Access::Denied => ServerFrame::Denied { topic },
    }
}

async fn access(state: &AppState, user_id: Uuid, topic: Topic) -> Access {
    let result = match topic {
        Topic::User(id) => if id == user_id { Ok(()) } else { Err(StatusCode::FORBIDDEN) },
        Topic::Chat(session_id) => crate::routes::sessions::authorize_session_access(&state.pool, user_id, session_id).await.map_err(|(status, _)| status),
        Topic::Project(project_id) => crate::routes::deploy::project_role(&state.pool, project_id, user_id).await.map(|_| ()).map_err(|(status, _)| status),
    };
    access_from(result)
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

async fn run(mut socket: WebSocket, state: AppState, user_id: Uuid, expires_in: Duration) {
    let token_expiry = tokio::time::sleep(expires_in);
    tokio::pin!(token_expiry);
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
            () = &mut token_expiry => {
                let _ = socket.send(Message::Close(Some(CloseFrame { code: CLOSE_TOKEN_EXPIRED, reason: "token expired".into() }))).await;
                break;
            }
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
                            Some(topic) if topics.len() - 1 < MAX_CLIENT_TOPICS_PER_SOCKET => match access(&state, user_id, topic).await {
                                Access::Granted => {
                                    if client.subscribe(topic.mqtt_topic(), QoS::AtMostOnce).await.is_err() { break; }
                                    topics.insert(topic);
                                    ServerFrame::Subscribed { topic: topic.wire() }
                                }
                                refused => refusal(refused, topic.wire()),
                            },
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

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;

    #[test]
    fn a_server_error_while_authorizing_is_unavailable_not_denied() {
        assert_eq!(access_from(Err(StatusCode::INTERNAL_SERVER_ERROR)), Access::Unavailable);
        assert_eq!(access_from(Err(StatusCode::SERVICE_UNAVAILABLE)), Access::Unavailable);
    }

    #[test]
    fn not_found_and_forbidden_are_denied() {
        assert_eq!(access_from(Err(StatusCode::NOT_FOUND)), Access::Denied);
        assert_eq!(access_from(Err(StatusCode::FORBIDDEN)), Access::Denied);
        assert_eq!(access_from(Ok(())), Access::Granted);
    }

    #[test]
    fn refusals_map_to_their_frames() {
        assert_eq!(refusal(Access::Unavailable, "chat:x".into()), ServerFrame::Unavailable { topic: "chat:x".into() });
        assert_eq!(refusal(Access::Denied, "chat:x".into()), ServerFrame::Denied { topic: "chat:x".into() });
    }

    #[test]
    fn time_left_on_a_token_never_goes_negative() {
        assert_eq!(token_time_left(1_000, 990), Duration::from_secs(10));
        assert_eq!(token_time_left(1_000, 1_500), Duration::ZERO);
    }
}
