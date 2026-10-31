//! The network engine against a fake backend: wiremock for REST, a tokio-tungstenite gateway for
//! the realtime socket.

use std::sync::Arc;
use std::time::Duration;

use futures_util::{SinkExt as _, StreamExt as _};
use serde_json::{json, Value};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::handshake::server::{ErrorResponse, Request, Response};
use tokio_tungstenite::tungstenite::http::StatusCode;
use tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode;
use tokio_tungstenite::tungstenite::protocol::CloseFrame;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::WebSocketStream;
use wiremock::matchers::{body_json, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use super::realtime::{backoff_delay, normalize_topic, ConnectionState, RealtimeSettings};
use crate::error::{EmailCodeError, NomiError};
use crate::event::{EventSubscription, SdkEvent};
use crate::network::auth::{SignInResult, SignOutReason};
use crate::network::events::RealtimeEvent;
use crate::test_support::{credentials, token, MemoryCredentials};
use crate::{default_sdk_config, NomiSdk};

const USER: &str = "6f1c2b9e-user";
const EVENT_TIMEOUT: Duration = Duration::from_secs(5);
/// Tokens containing this are refused at the socket upgrade with 401.
const REJECTED_MARK: &str = "rejected";

fn settings() -> RealtimeSettings {
    RealtimeSettings {
        ping_every: Duration::from_secs(60),
        stale_after: Duration::from_secs(60),
        stable_after: Duration::from_secs(60),
        connect_timeout: Duration::from_secs(2),
        close_timeout: Duration::from_secs(1),
        backoff_base: Duration::from_millis(10),
        backoff_max: Duration::from_millis(50),
    }
}

struct Harness {
    _dir: tempfile::TempDir,
    server: MockServer,
    store: MemoryCredentials,
    sdk: Arc<NomiSdk>,
    events: Arc<EventSubscription>,
}

impl Harness {
    async fn new(store: MemoryCredentials) -> Self {
        Self::with(store, settings(), None).await
    }

    /// `base_url` defaults to the wiremock server; realtime tests pass the fake gateway's.
    async fn with(
        store: MemoryCredentials,
        settings: RealtimeSettings,
        base_url: Option<String>,
    ) -> Self {
        let server = MockServer::start().await;
        let dir = tempfile::tempdir().unwrap();
        let base = base_url.unwrap_or_else(|| server.uri());
        let config = default_sdk_config(base, dir.path().display().to_string());
        let sdk = NomiSdk::with_settings(config, store.boxed(), settings).unwrap();
        let events = sdk.subscribe();
        Self {
            _dir: dir,
            server,
            store,
            sdk,
            events,
        }
    }

    async fn next_matching<T>(&self, mut pick: impl FnMut(SdkEvent) -> Option<T>) -> T {
        tokio::time::timeout(EVENT_TIMEOUT, async {
            loop {
                let event = self
                    .events
                    .next_event()
                    .await
                    .expect("the event stream ended");
                if let Some(found) = pick(event) {
                    return found;
                }
            }
        })
        .await
        .expect("timed out waiting for an event")
    }

    async fn state(&self, wanted: impl Fn(&ConnectionState) -> bool) -> ConnectionState {
        self.next_matching(|event| match event {
            SdkEvent::ConnectionChanged { state } if wanted(&state) => Some(state),
            _ => None,
        })
        .await
    }
}

fn signed_in() -> MemoryCredentials {
    MemoryCredentials::holding(credentials(USER, "a"))
}

fn token_pair(user: &str, tag: &str) -> Value {
    json!({ "access_token": token(user, tag), "refresh_token": format!("refresh-{tag}") })
}

// Pure helpers

#[test]
fn normalizes_topics_as_the_gateway_spells_them() {
    assert_eq!(normalize_topic("user").unwrap(), "user");
    assert_eq!(normalize_topic("chat:ABC-1").unwrap(), "chat:abc-1");
    assert_eq!(normalize_topic(" project:P ").unwrap(), "project:p");
    for bad in ["", "chat:", "user:1", "session:1"] {
        assert!(
            matches!(normalize_topic(bad), Err(NomiError::InvalidArgument { .. })),
            "{bad}"
        );
    }
}

#[test]
fn backoff_grows_and_is_capped() {
    let (base, max) = (Duration::from_millis(500), Duration::from_secs(30));
    for _ in 0..100 {
        assert!(backoff_delay(0, base, max) <= base);
        assert!(backoff_delay(3, base, max) <= base * 8);
        assert!(backoff_delay(40, base, max) <= max);
    }
}

#[test]
fn reads_the_user_from_an_access_token() {
    assert_eq!(
        super::auth::user_from_token(&token(USER, "a"))
            .unwrap()
            .user_id,
        USER
    );
    assert_eq!(super::auth::user_from_token("not-a-jwt"), None);
    assert_eq!(super::auth::user_from_token("a.!!!.c"), None);
}

// Sign-in

#[tokio::test]
async fn signs_in_and_stores_the_session() {
    let harness = Harness::new(MemoryCredentials::default()).await;
    Mock::given(method("POST"))
        .and(path("/api/auth/login"))
        .and(body_json(
            json!({ "email": "a@nomi.test", "password": "pw" }),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(token_pair(USER, "a")))
        .mount(&harness.server)
        .await;

    let result = harness
        .sdk
        .sign_in("a@nomi.test".into(), "pw".into())
        .await
        .unwrap();

    assert!(matches!(result, SignInResult::SignedIn { user } if user.user_id == USER));
    assert_eq!(harness.store.get(), Some(credentials(USER, "a")));
    assert_eq!(harness.sdk.signed_in_user().unwrap().user_id, USER);
    harness
        .next_matching(|event| {
            matches!(event, SdkEvent::SignedIn { user_id } if user_id == USER).then_some(())
        })
        .await;
}

#[tokio::test]
async fn an_email_code_challenge_stores_nothing_until_verified() {
    let harness = Harness::new(MemoryCredentials::default()).await;
    let verification = json!({
        "challenge_id": "c1", "purpose": "sign_in", "email": "a@nomi.test",
        "expires_in": 600, "resend_in": 30, "sends_left": 4
    });
    Mock::given(method("POST"))
        .and(path("/api/auth/login"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({ "verification": verification })),
        )
        .mount(&harness.server)
        .await;
    Mock::given(method("POST"))
        .and(path("/api/auth/verify"))
        .and(body_json(json!({ "challenge_id": "c1", "code": "111111" })))
        .respond_with(
            ResponseTemplate::new(400)
                .set_body_json(json!({ "error": "wrong_code", "attempts_left": 2 })),
        )
        .mount(&harness.server)
        .await;
    Mock::given(method("POST"))
        .and(path("/api/auth/verify"))
        .and(body_json(json!({ "challenge_id": "c1", "code": "424242" })))
        .respond_with(ResponseTemplate::new(200).set_body_json(token_pair(USER, "a")))
        .mount(&harness.server)
        .await;
    Mock::given(method("POST"))
        .and(path("/api/auth/resend"))
        .respond_with(
            ResponseTemplate::new(429).set_body_json(json!({ "error": "wait", "seconds": 30 })),
        )
        .mount(&harness.server)
        .await;

    let SignInResult::EmailCodeRequired { verification } = harness
        .sdk
        .sign_in("a@nomi.test".into(), "pw".into())
        .await
        .unwrap()
    else {
        panic!("expected an email code challenge");
    };
    assert_eq!(
        (verification.challenge_id.as_str(), verification.sends_left),
        ("c1", 4)
    );
    assert_eq!(harness.store.get(), None);

    let wrong = harness
        .sdk
        .verify_email_code("c1".into(), "111111".into())
        .await;
    assert_eq!(
        wrong,
        Err(NomiError::EmailCode {
            error: EmailCodeError::WrongCode { attempts_left: 2 }
        })
    );
    let wait = harness.sdk.resend_email_code("c1".into()).await;
    assert_eq!(
        wait,
        Err(NomiError::EmailCode {
            error: EmailCodeError::Wait { seconds: 30 }
        })
    );

    let user = harness
        .sdk
        .verify_email_code("c1".into(), "424242".into())
        .await
        .unwrap();
    assert_eq!(user.user_id, USER);
    assert!(harness.store.get().is_some());
}

#[tokio::test]
async fn rejected_credentials_are_unauthorized() {
    let harness = Harness::new(MemoryCredentials::default()).await;
    Mock::given(method("POST"))
        .and(path("/api/auth/login"))
        .respond_with(ResponseTemplate::new(401))
        .mount(&harness.server)
        .await;
    assert_eq!(
        harness
            .sdk
            .sign_in("a@nomi.test".into(), "bad".into())
            .await,
        Err(NomiError::Unauthorized)
    );
}

#[tokio::test]
async fn an_unreachable_backend_is_a_network_error() {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let closed = format!("http://{}", listener.local_addr().unwrap());
    drop(listener);
    let harness = Harness::with(MemoryCredentials::default(), settings(), Some(closed)).await;
    assert!(matches!(
        harness.sdk.sign_in("a".into(), "b".into()).await,
        Err(NomiError::Network { .. })
    ));
}

#[tokio::test]
async fn a_stored_session_is_signed_in_from_the_start() {
    let harness = Harness::new(signed_in()).await;
    assert_eq!(harness.sdk.signed_in_user().unwrap().user_id, USER);
}

// Authorized requests

fn profile_body() -> Value {
    json!({ "display_name": "Ada", "username": "ada", "email": "a@nomi.test", "avatar_url": null })
}

#[tokio::test]
async fn sends_the_bearer_token() {
    let harness = Harness::new(signed_in()).await;
    Mock::given(method("GET"))
        .and(path("/api/profile"))
        .and(header(
            "authorization",
            format!("Bearer {}", token(USER, "a")).as_str(),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(profile_body()))
        .mount(&harness.server)
        .await;
    assert_eq!(
        harness
            .sdk
            .fetch_profile()
            .await
            .unwrap()
            .display_name
            .as_deref(),
        Some("Ada")
    );
}

#[tokio::test]
async fn an_expired_token_is_refreshed_once_and_the_request_retried() {
    let harness = Harness::new(signed_in()).await;
    Mock::given(method("GET"))
        .and(path("/api/profile"))
        .and(header(
            "authorization",
            format!("Bearer {}", token(USER, "a")).as_str(),
        ))
        .respond_with(ResponseTemplate::new(401))
        .mount(&harness.server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/profile"))
        .and(header(
            "authorization",
            format!("Bearer {}", token(USER, "b")).as_str(),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(profile_body()))
        .mount(&harness.server)
        .await;
    Mock::given(method("POST"))
        .and(path("/api/auth/refresh"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({ "access_token": token(USER, "b") })),
        )
        .expect(1)
        .mount(&harness.server)
        .await;

    // Concurrent requests that all hit the expired token share one refresh.
    let calls = (0..4).map(|_| harness.sdk.fetch_profile());
    for result in futures_util::future::join_all(calls).await {
        assert_eq!(result.unwrap().email, "a@nomi.test");
    }
    let stored = harness.store.get().unwrap();
    assert_eq!(
        (stored.access_token, stored.refresh_token),
        (token(USER, "b"), "refresh-a".to_owned())
    );
}

#[tokio::test]
async fn a_refused_refresh_ends_the_session() {
    let harness = Harness::new(signed_in()).await;
    Mock::given(method("GET"))
        .and(path("/api/profile"))
        .respond_with(ResponseTemplate::new(401))
        .mount(&harness.server)
        .await;
    Mock::given(method("POST"))
        .and(path("/api/auth/refresh"))
        .respond_with(ResponseTemplate::new(401))
        .mount(&harness.server)
        .await;

    assert_eq!(
        harness.sdk.fetch_profile().await,
        Err(NomiError::Unauthorized)
    );
    assert_eq!(harness.store.get(), None);
    assert_eq!(harness.sdk.signed_in_user(), None);
    let reason = harness.next_matching(|event| match event {
        SdkEvent::SignedOut { reason } => Some(reason),
        _ => None,
    });
    assert_eq!(reason.await, SignOutReason::SessionExpired);
}

#[tokio::test]
async fn a_failed_refresh_keeps_the_session() {
    let harness = Harness::new(signed_in()).await;
    Mock::given(method("POST"))
        .and(path("/api/auth/refresh"))
        .respond_with(ResponseTemplate::new(503))
        .mount(&harness.server)
        .await;
    assert!(matches!(
        harness.sdk.refresh_session().await,
        Err(NomiError::Http { status: 503, .. })
    ));
    assert!(harness.store.get().is_some());
}

#[tokio::test]
async fn calls_that_need_a_session_say_so_without_one() {
    let harness = Harness::new(MemoryCredentials::default()).await;
    assert_eq!(
        harness.sdk.fetch_profile().await,
        Err(NomiError::NotSignedIn)
    );
    assert_eq!(harness.sdk.refresh_session().await, Ok(false));
}

#[tokio::test]
async fn sign_out_forgets_locally_and_revokes_on_the_backend() {
    let harness = Harness::new(signed_in()).await;
    Mock::given(method("POST"))
        .and(path("/api/auth/logout"))
        .and(body_json(json!({ "refresh_token": "refresh-a" })))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&harness.server)
        .await;
    harness.sdk.open_account(USER.into()).await.unwrap();

    harness.sdk.sign_out().await.unwrap();

    assert_eq!(harness.store.get(), None);
    assert!(harness.sdk.current_account().is_none());
    let reason = harness.next_matching(|event| match event {
        SdkEvent::SignedOut { reason } => Some(reason),
        _ => None,
    });
    assert_eq!(reason.await, SignOutReason::Requested);
}

// Realtime

type Responder = Arc<dyn Fn(&str, Value) -> (u16, Value) + Send + Sync>;

/// A fake backend on one port: `/api/realtime/ws` upgrades (refusing tokens marked
/// [`REJECTED_MARK`] with 401) and hands each socket to the test; any other request is answered
/// by `responder` with `(status, json)`.
struct Gateway {
    url: String,
    sockets: mpsc::UnboundedReceiver<Socket>,
}

struct Socket {
    token: String,
    ws: WebSocketStream<TcpStream>,
}

const SOCKET_REQUEST: &[u8] = b"GET /api/realtime/ws";
const HEADER_END: &[u8] = b"\r\n\r\n";

impl Gateway {
    async fn start(responder: Responder) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        let (tx, sockets) = mpsc::unbounded_channel();
        tokio::spawn(async move {
            while let Ok((stream, _)) = listener.accept().await {
                let (tx, responder) = (tx.clone(), responder.clone());
                tokio::spawn(async move {
                    let mut head = [0u8; SOCKET_REQUEST.len()];
                    let _ = stream.peek(&mut head).await;
                    if head == SOCKET_REQUEST {
                        upgrade(stream, tx).await;
                    } else {
                        answer(stream, responder).await;
                    }
                });
            }
        });
        Self { url, sockets }
    }

    async fn accept(&mut self) -> Socket {
        tokio::time::timeout(EVENT_TIMEOUT, self.sockets.recv())
            .await
            .expect("no socket arrived")
            .unwrap()
    }
}

// tungstenite fixes the callback's signature, large error type included.
#[allow(clippy::result_large_err)]
async fn upgrade(stream: TcpStream, tx: mpsc::UnboundedSender<Socket>) {
    let mut token = String::new();
    let callback = |request: &Request, response: Response| -> Result<Response, ErrorResponse> {
        let bearer = request
            .headers()
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        token = bearer.strip_prefix("Bearer ").unwrap_or("").to_owned();
        if token.contains(REJECTED_MARK) {
            let mut refusal = ErrorResponse::new(None);
            *refusal.status_mut() = StatusCode::UNAUTHORIZED;
            return Err(refusal);
        }
        Ok(response)
    };
    if let Ok(ws) = tokio_tungstenite::accept_hdr_async(stream, callback).await {
        let _ = tx.send(Socket { token, ws });
    }
}

/// Just enough HTTP/1.1 for the SDK's JSON POSTs: read the head and a `content-length` body,
/// answer, close.
async fn answer(mut stream: TcpStream, responder: Responder) {
    use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};
    let mut buffer = Vec::new();
    let mut chunk = [0u8; 1024];
    let head_end = loop {
        let read = stream.read(&mut chunk).await.unwrap_or(0);
        if read == 0 {
            return;
        }
        buffer.extend_from_slice(&chunk[..read]);
        if let Some(index) = buffer
            .windows(HEADER_END.len())
            .position(|window| window == HEADER_END)
        {
            break index + HEADER_END.len();
        }
    };
    let head = String::from_utf8_lossy(&buffer[..head_end]).to_string();
    let length = head
        .lines()
        .find_map(|line| {
            line.to_ascii_lowercase()
                .strip_prefix("content-length:")
                .map(|v| v.trim().parse().unwrap_or(0))
        })
        .unwrap_or(0usize);
    while buffer.len() < head_end + length {
        let read = stream.read(&mut chunk).await.unwrap_or(0);
        if read == 0 {
            break;
        }
        buffer.extend_from_slice(&chunk[..read]);
    }
    let path = head.split_whitespace().nth(1).unwrap_or("").to_owned();
    let body = serde_json::from_slice(&buffer[head_end..]).unwrap_or(Value::Null);
    let (status, reply) = responder(&path, body);
    let reply = reply.to_string();
    let response = format!(
        "HTTP/1.1 {status} X\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{reply}",
        reply.len()
    );
    let _ = stream.write_all(response.as_bytes()).await;
    let _ = stream.shutdown().await;
}

/// Refreshes `refresh-<from>` into an access token tagged `to`; logs out happily; 404 otherwise.
fn refreshing(from: &'static str, to: &'static str) -> Responder {
    Arc::new(move |path, body| match path {
        "/api/auth/refresh" if body["refresh_token"] == format!("refresh-{from}") => {
            (200, json!({ "access_token": token(USER, to) }))
        }
        "/api/auth/logout" => (200, json!({})),
        "/api/auth/login" => (200, token_pair(USER, "a")),
        _ => (404, json!({})),
    })
}

impl Socket {
    async fn send(&mut self, frame: Value) {
        self.ws
            .send(Message::Text(frame.to_string().into()))
            .await
            .unwrap();
    }

    async fn ready(&mut self) {
        self.send(json!({ "type": "ready", "user_id": USER })).await;
    }

    /// The next frame the client sent, skipping pings.
    async fn receive(&mut self) -> Value {
        tokio::time::timeout(EVENT_TIMEOUT, async {
            loop {
                match self.ws.next().await {
                    Some(Ok(Message::Text(text))) => {
                        let frame: Value = serde_json::from_str(text.as_str()).unwrap();
                        if frame["op"] != "ping" {
                            return frame;
                        }
                    }
                    Some(Ok(_)) => continue,
                    other => panic!("the client went away: {other:?}"),
                }
            }
        })
        .await
        .expect("the client sent nothing")
    }

    /// Whether the client closes this socket (or it ends) within the timeout.
    async fn closed_by_client(&mut self) -> bool {
        tokio::time::timeout(EVENT_TIMEOUT, async {
            loop {
                match self.ws.next().await {
                    Some(Ok(Message::Close(_))) | None | Some(Err(_)) => return true,
                    Some(Ok(_)) => continue,
                }
            }
        })
        .await
        .unwrap_or(false)
    }

    async fn close(&mut self, code: u16) {
        let frame = CloseFrame {
            code: CloseCode::from(code),
            reason: "".into(),
        };
        let _ = self.ws.close(Some(frame)).await;
    }
}

async fn realtime_harness(
    store: MemoryCredentials,
    settings: RealtimeSettings,
    responder: Responder,
) -> (Harness, Gateway) {
    let gateway = Gateway::start(responder).await;
    let harness = Harness::with(store, settings, Some(gateway.url.clone())).await;
    (harness, gateway)
}

fn no_rest() -> Responder {
    Arc::new(|_, _| (404, json!({})))
}

async fn connected(harness: &Harness, gateway: &mut Gateway) -> Socket {
    harness.sdk.start().await.unwrap();
    let mut socket = gateway.accept().await;
    socket.ready().await;
    harness
        .state(|state| matches!(state, ConnectionState::Connected { .. }))
        .await;
    socket
}

#[tokio::test]
async fn connects_once_running_and_signed_in_then_subscribes_watched_topics() {
    let (harness, mut gateway) = realtime_harness(signed_in(), settings(), no_rest()).await;
    let _chat = harness.sdk.watch_topic("chat:C1".into()).unwrap();
    let _user = harness.sdk.watch_topic("user".into()).unwrap();
    assert_eq!(harness.sdk.connection_state(), ConnectionState::Idle);

    harness.sdk.start().await.unwrap();
    let mut socket = gateway.accept().await;
    assert_eq!(socket.token, token(USER, "a"));
    socket.ready().await;

    harness
        .state(|state| {
            *state
                == ConnectionState::Connected {
                    user_id: USER.into(),
                }
        })
        .await;
    // `user` is implicit on the gateway; only the chat is subscribed.
    assert_eq!(
        socket.receive().await,
        json!({ "op": "subscribe", "topic": "chat:c1" })
    );
    let mut resynced = vec![];
    while resynced.len() < 2 {
        resynced.push(
            harness
                .next_matching(|event| match event {
                    SdkEvent::TopicResynced { topic } => Some(topic),
                    _ => None,
                })
                .await,
        );
    }
    resynced.sort();
    assert_eq!(resynced, ["chat:c1", "user"]);
}

#[tokio::test]
async fn delivers_events_and_follows_watch_and_release() {
    let (harness, mut gateway) = realtime_harness(signed_in(), settings(), no_rest()).await;
    let mut socket = connected(&harness, &mut gateway).await;

    let watch = harness.sdk.watch_topic("chat:c1".into()).unwrap();
    assert_eq!(
        socket.receive().await,
        json!({ "op": "subscribe", "topic": "chat:c1" })
    );
    let second = harness.sdk.watch_topic("chat:c1".into()).unwrap();

    socket.send(json!({ "type": "event", "topic": "chat:c1", "payload": { "kind": "MessageCreated", "message_id": "m1" } })).await;
    let (topic, event, payload) = harness
        .next_matching(|event| match event {
            SdkEvent::Realtime {
                topic,
                event,
                payload_json,
            } => Some((topic, event, payload_json)),
            _ => None,
        })
        .await;
    assert_eq!(topic, "chat:c1");
    assert_eq!(
        event,
        RealtimeEvent::MessageCreated {
            session_id: "c1".into(),
            message_id: "m1".into()
        }
    );
    assert_eq!(
        serde_json::from_str::<Value>(&payload).unwrap(),
        json!({ "kind": "MessageCreated", "message_id": "m1" })
    );

    // Only the last release unsubscribes.
    watch.release();
    drop(second);
    assert_eq!(
        socket.receive().await,
        json!({ "op": "unsubscribe", "topic": "chat:c1" })
    );
}

#[tokio::test]
async fn a_denied_topic_is_reported_and_retried_when_watched_again() {
    let (harness, mut gateway) = realtime_harness(signed_in(), settings(), no_rest()).await;
    let mut socket = connected(&harness, &mut gateway).await;
    let _first = harness.sdk.watch_topic("chat:gone".into()).unwrap();
    assert_eq!(socket.receive().await["op"], "subscribe");

    socket
        .send(json!({ "type": "denied", "topic": "chat:gone" }))
        .await;
    let lost = harness.next_matching(|event| match event {
        SdkEvent::TopicAccessLost { topic } => Some(topic),
        _ => None,
    });
    assert_eq!(lost.await, "chat:gone");

    let _again = harness.sdk.watch_topic("chat:gone".into()).unwrap();
    assert_eq!(
        socket.receive().await,
        json!({ "op": "subscribe", "topic": "chat:gone" })
    );
}

#[tokio::test]
async fn an_unavailable_topic_is_asked_for_again() {
    let (harness, mut gateway) = realtime_harness(signed_in(), settings(), no_rest()).await;
    let mut socket = connected(&harness, &mut gateway).await;
    let _watch = harness.sdk.watch_topic("chat:busy".into()).unwrap();
    assert_eq!(socket.receive().await["op"], "subscribe");
    socket
        .send(json!({ "type": "unavailable", "topic": "chat:busy" }))
        .await;
    assert_eq!(
        socket.receive().await,
        json!({ "op": "subscribe", "topic": "chat:busy" })
    );
}

#[tokio::test]
async fn reconnects_after_the_server_drops_the_socket_and_resubscribes() {
    let (harness, mut gateway) = realtime_harness(signed_in(), settings(), no_rest()).await;
    let _chat = harness.sdk.watch_topic("chat:c1".into()).unwrap();
    let mut first = connected(&harness, &mut gateway).await;
    assert_eq!(first.receive().await["op"], "subscribe");

    first.close(1011).await;
    harness
        .state(|state| matches!(state, ConnectionState::Waiting { attempt: 1, .. }))
        .await;

    let mut second = gateway.accept().await;
    second.ready().await;
    assert_eq!(
        second.receive().await,
        json!({ "op": "subscribe", "topic": "chat:c1" })
    );
}

#[tokio::test]
async fn a_refused_upgrade_refreshes_the_token_and_reconnects() {
    let store = MemoryCredentials::holding(credentials(USER, REJECTED_MARK));
    let (harness, mut gateway) =
        realtime_harness(store, settings(), refreshing(REJECTED_MARK, "fresh")).await;

    harness.sdk.start().await.unwrap();

    let mut socket = gateway.accept().await;
    assert_eq!(socket.token, token(USER, "fresh"));
    socket.ready().await;
    harness
        .state(|state| matches!(state, ConnectionState::Connected { .. }))
        .await;
    assert_eq!(
        harness.store.get().unwrap().access_token,
        token(USER, "fresh")
    );
}

#[tokio::test]
async fn an_expired_socket_refreshes_and_reconnects() {
    let (harness, mut gateway) =
        realtime_harness(signed_in(), settings(), refreshing("a", "fresh")).await;
    let mut first = connected(&harness, &mut gateway).await;

    first.close(4001).await;

    let second = gateway.accept().await;
    assert_eq!(second.token, token(USER, "fresh"));
}

#[tokio::test]
async fn still_refused_after_a_refresh_ends_the_session() {
    let store = MemoryCredentials::holding(credentials(USER, REJECTED_MARK));
    // The refreshed token is still refused.
    let (harness, _gateway) = realtime_harness(
        store,
        settings(),
        refreshing(REJECTED_MARK, "rejected-again"),
    )
    .await;

    harness.sdk.start().await.unwrap();

    let reason = harness.next_matching(|event| match event {
        SdkEvent::SignedOut { reason } => Some(reason),
        _ => None,
    });
    assert_eq!(reason.await, SignOutReason::AccessRevoked);
    assert_eq!(harness.store.get(), None);
    assert_eq!(harness.sdk.connection_state(), ConnectionState::Idle);
}

#[tokio::test]
async fn pings_and_recycles_a_silent_socket() {
    let settings = RealtimeSettings {
        ping_every: Duration::from_millis(50),
        stale_after: Duration::from_millis(400),
        ..settings()
    };
    let (harness, mut gateway) = realtime_harness(signed_in(), settings, no_rest()).await;
    harness.sdk.start().await.unwrap();
    let mut first = gateway.accept().await;

    let ping = tokio::time::timeout(EVENT_TIMEOUT, first.ws.next())
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    assert_eq!(
        serde_json::from_str::<Value>(ping.to_text().unwrap()).unwrap(),
        json!({ "op": "ping" })
    );

    // The gateway reads but never answers: the client gives up on the socket and opens another.
    let drain = tokio::spawn(async move { while first.ws.next().await.is_some() {} });
    let _second = gateway.accept().await;
    drain.abort();
}

#[tokio::test]
async fn shutting_down_closes_the_socket() {
    let (harness, mut gateway) = realtime_harness(signed_in(), settings(), no_rest()).await;
    let mut socket = connected(&harness, &mut gateway).await;

    harness.sdk.shutdown().await;

    assert!(socket.closed_by_client().await);
    assert_eq!(harness.sdk.connection_state(), ConnectionState::Idle);
}

#[tokio::test]
async fn signing_out_closes_the_socket_and_signing_in_opens_one() {
    let (harness, mut gateway) =
        realtime_harness(signed_in(), settings(), refreshing("a", "unused")).await;
    let mut socket = connected(&harness, &mut gateway).await;

    harness.sdk.sign_out().await.unwrap();
    assert!(socket.closed_by_client().await);
    harness.state(|state| *state == ConnectionState::Idle).await;

    harness
        .sdk
        .sign_in("a@nomi.test".into(), "pw".into())
        .await
        .unwrap();
    let socket = gateway.accept().await;
    assert_eq!(socket.token, token(USER, "a"));
}
