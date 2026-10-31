//! Sync against a fake backend (wiremock). Realtime events are put on the SDK's event stream
//! directly, as the socket would.

use std::future::Future;
use std::sync::Arc;
use std::time::Duration;

use serde_json::{json, Value};
use wiremock::matchers::{body_json, method, path, query_param};
use wiremock::{Mock, MockServer, Request, ResponseTemplate};

use super::api::parse_time;
use super::SyncSettings;
use crate::client::EngineSettings;
use crate::error::NomiError;
use crate::event::SdkEvent;
use crate::network::events::RealtimeEvent;
use crate::network::realtime::RealtimeSettings;
use crate::storage::models::{
    Draft, Message, NotificationItem, OutboxEntry, OutboxOperation, OutboxState, Session,
    SessionFilter, STATUS_FAILED, STATUS_SENDING, STATUS_SENT,
};
use crate::storage::AccountStore;
use crate::test_support::{credentials, MemoryCredentials};
use crate::{default_sdk_config, NomiSdk};

const USER: &str = "7a0c5d2e-user";
const OTHER_USER: &str = "93b1e4f0-other";
const CHAT: &str = "s1";
const CHAT_TOPIC: &str = "chat:s1";
const MESSAGES_PATH: &str = "/api/sessions/s1/messages";
const WAIT: Duration = Duration::from_secs(5);
const POLL: Duration = Duration::from_millis(10);
const LIMIT: u32 = 100;
/// The epoch-ms of `at(0)`.
const EPOCH_2026: &str = "2026-01-01T00:00:00Z";
/// Retries far enough apart that a test sees exactly one attempt before it acts.
const SLOW: Duration = Duration::from_secs(60);
/// Wide enough that any server timestamp in these tests counts as "just now".
const WIDE_ECHO_WINDOW: Duration = Duration::from_secs(20 * 365 * 24 * 60 * 60);

fn settings() -> SyncSettings {
    SyncSettings {
        retry_base: Duration::from_millis(10),
        retry_max: Duration::from_millis(20),
        max_attempts: 3,
        echo_window: WIDE_ECHO_WINDOW,
        done_retention: Duration::from_secs(60),
    }
}

/// An RFC 3339 time `seconds` (under an hour) into 2026.
fn at(seconds: u32) -> String {
    format!("2026-01-01T00:{:02}:{:02}Z", seconds / 60, seconds % 60)
}

fn at_ms(seconds: u32) -> i64 {
    parse_time(EPOCH_2026).unwrap() + i64::from(seconds) * 1_000
}

fn message_json(id: &str, sender: &str, content: &str, seconds: u32) -> Value {
    json!({
        "id": id, "sender": sender, "content": content, "content_blocks": null,
        "created_at": at(seconds), "my_feedback": null, "agent_display_name": null,
        "memory_count": 0, "steps": [], "sources": [], "edited_at": null, "versions": 0
    })
}

fn page(messages: Vec<Value>, has_more: bool) -> Value {
    json!({ "messages": messages, "has_more": has_more })
}

fn sent_message(id: &str, content: &str) -> Value {
    json!({ "user_message": message_json(id, "user", content, 1) })
}

fn stored_message(id: &str, role: &str, content: &str, seconds: u32) -> Message {
    Message {
        id: id.into(),
        session_id: CHAT.into(),
        role: role.into(),
        agent_type: None,
        agent_display_name: None,
        content: content.into(),
        content_json: None,
        created_at: at_ms(seconds),
        status: STATUS_SENT.into(),
    }
}

fn session_json(id: &str, title: Option<&str>, preview: &str, seconds: u32) -> Value {
    json!({
        "id": id, "channel": "web", "chat_type": "dm", "chat_id": id, "title": title,
        "last_message": { "content": preview, "created_at": at(seconds) },
        "agent_active": false, "updated_at": at(seconds), "project_id": null,
        "pinned": false, "pinned_at": null, "in_project_id": null
    })
}

fn notification_json(id: &str, read: bool, seconds: u32) -> Value {
    json!({
        "id": id, "kind": "deploy", "title": format!("T {id}"), "body": "B", "link": null,
        "read": read, "created_at": at(seconds)
    })
}

struct Harness {
    _dir: tempfile::TempDir,
    server: MockServer,
    sdk: Arc<NomiSdk>,
    store: Arc<AccountStore>,
}

impl Harness {
    async fn new() -> Self {
        Self::with(settings()).await
    }

    async fn with(sync: SyncSettings) -> Self {
        let server = MockServer::start().await;
        let dir = tempfile::tempdir().unwrap();
        let (sdk, store) = open(
            &server,
            &dir,
            MemoryCredentials::holding(credentials(USER, "a")),
            sync,
        )
        .await;
        Self {
            _dir: dir,
            server,
            sdk,
            store,
        }
    }

    async fn mock(&self, mock: Mock) {
        mock.mount(&self.server).await;
    }

    async fn requests(&self, verb: &str, url_path: &str) -> Vec<Request> {
        self.server
            .received_requests()
            .await
            .unwrap_or_default()
            .into_iter()
            .filter(|request| request.method.as_str() == verb && request.url.path() == url_path)
            .collect()
    }

    fn event(&self, topic: &str, payload: Value) {
        self.sdk.emit(SdkEvent::Realtime {
            topic: topic.into(),
            event: crate::network::events::decode(topic, &payload),
            payload_json: payload.to_string(),
        });
    }

    async fn messages(&self) -> Vec<Message> {
        self.store
            .messages_page(CHAT.into(), None, LIMIT)
            .await
            .unwrap()
            .messages
    }

    async fn sessions(&self) -> Vec<Session> {
        self.store
            .observe_sessions(SessionFilter {
                search: None,
                unread_only: false,
                limit: LIMIT,
            })
            .next()
            .await
            .unwrap()
            .unwrap()
    }

    async fn draft(&self) -> Option<Draft> {
        self.store
            .observe_draft(CHAT.into())
            .next()
            .await
            .unwrap()
            .unwrap()
            .draft
    }

    async fn notifications(&self) -> Vec<NotificationItem> {
        self.store
            .observe_notifications(LIMIT)
            .next()
            .await
            .unwrap()
            .unwrap()
    }

    async fn outbox(&self) -> Vec<OutboxEntry> {
        self.store.outbox_entries().await.unwrap()
    }
}

async fn open(
    server: &MockServer,
    dir: &tempfile::TempDir,
    store: MemoryCredentials,
    sync: SyncSettings,
) -> (Arc<NomiSdk>, Arc<AccountStore>) {
    let config = default_sdk_config(server.uri(), dir.path().display().to_string());
    let settings = EngineSettings {
        realtime: RealtimeSettings::default(),
        sync,
    };
    let sdk = NomiSdk::with_settings(config, store.boxed(), settings).unwrap();
    let account = sdk.open_account(USER.into()).await.unwrap();
    (sdk, account)
}

/// Polls `check` until it returns `Some`, or fails the test.
async fn eventually<T, F: Future<Output = Option<T>>>(
    what: &str,
    mut check: impl FnMut() -> F,
) -> T {
    tokio::time::timeout(WAIT, async {
        loop {
            if let Some(value) = check().await {
                return value;
            }
            tokio::time::sleep(POLL).await;
        }
    })
    .await
    .unwrap_or_else(|_| panic!("timed out waiting for {what}"))
}

fn get(url_path: &str) -> wiremock::MockBuilder {
    Mock::given(method("GET")).and(path(url_path))
}

fn post(url_path: &str) -> wiremock::MockBuilder {
    Mock::given(method("POST")).and(path(url_path))
}

fn ok(body: Value) -> ResponseTemplate {
    ResponseTemplate::new(200).set_body_json(body)
}

// Chats and messages

#[tokio::test]
async fn refreshing_sessions_mirrors_the_server_list_and_keeps_unread_flags() {
    let h = Harness::new().await;
    let stale = |id: &str, unread| Session {
        id: id.into(),
        title: "old".into(),
        last_message_preview: "old".into(),
        updated_at: 1,
        unread,
    };
    h.store
        .upsert_sessions(vec![stale("s1", true), stale("gone", false)])
        .await
        .unwrap();
    h.mock(get("/api/sessions").respond_with(ok(json!({ "sessions": [
        session_json("s1", Some("Plans"), "See you", 20),
        session_json("s2", None, "Hello", 10),
    ] }))))
    .await;

    h.sdk.refresh_sessions().await.unwrap();

    let sessions = h.sessions().await;
    assert_eq!(
        sessions,
        vec![
            Session {
                id: "s1".into(),
                title: "Plans".into(),
                last_message_preview: "See you".into(),
                updated_at: at_ms(20),
                unread: true,
            },
            Session {
                id: "s2".into(),
                title: "".into(),
                last_message_preview: "Hello".into(),
                updated_at: at_ms(10),
                unread: false,
            },
        ]
    );
}

#[tokio::test]
async fn a_chat_the_outbox_still_owes_a_message_survives_a_refresh() {
    let h = Harness::with(SyncSettings {
        retry_base: SLOW,
        retry_max: SLOW,
        ..settings()
    })
    .await;
    h.mock(post(MESSAGES_PATH).respond_with(ResponseTemplate::new(503)))
        .await;
    h.mock(get("/api/sessions").respond_with(ok(json!({ "sessions": [] }))))
        .await;
    h.sdk
        .send_message(CHAT.into(), "first!".into())
        .await
        .unwrap();

    h.sdk.refresh_sessions().await.unwrap();

    let sessions = h.sessions().await;
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].last_message_preview, "first!");
}

#[tokio::test]
async fn refreshing_a_chat_stores_its_newest_page_and_drops_what_the_server_deleted() {
    let h = Harness::new().await;
    h.store
        .upsert_messages(vec![
            stored_message("m0", "user", "older, outside the page", 5),
            stored_message("m2", "assistant", "deleted on the server", 25),
        ])
        .await
        .unwrap();
    h.mock(
        get(MESSAGES_PATH)
            .and(query_param("limit", "50"))
            .respond_with(ok(page(
                vec![
                    message_json("m1", "user", "Hi", 20),
                    message_json("m3", "assistant", "Hello!", 30),
                ],
                true,
            ))),
    )
    .await;

    h.sdk.refresh_messages(CHAT.into()).await.unwrap();

    let ids: Vec<String> = h.messages().await.into_iter().map(|m| m.id).collect();
    assert_eq!(ids, ["m0", "m1", "m3"]);
    assert!(h.sdk.has_older_messages(CHAT.into()).await.unwrap());
}

#[tokio::test]
async fn loads_older_pages_before_the_oldest_stored_message() {
    let h = Harness::new().await;
    h.mock(
        get(MESSAGES_PATH)
            .and(query_param("before", "m1"))
            .respond_with(ok(page(
                vec![message_json("m0", "user", "The start", 10)],
                false,
            ))),
    )
    .await;
    h.mock(
        get(MESSAGES_PATH).respond_with(ok(page(vec![message_json("m1", "user", "Hi", 20)], true))),
    )
    .await;

    // Nothing stored yet: the first load is the newest page.
    assert!(h.sdk.load_older_messages(CHAT.into()).await.unwrap());
    assert!(!h.sdk.load_older_messages(CHAT.into()).await.unwrap());

    let ids: Vec<String> = h.messages().await.into_iter().map(|m| m.id).collect();
    assert_eq!(ids, ["m0", "m1"]);
    assert!(!h.sdk.has_older_messages(CHAT.into()).await.unwrap());
}

#[tokio::test]
async fn binding_a_chat_resyncs_it_and_watches_its_topic() {
    let h = Harness::new().await;
    h.mock(
        get(MESSAGES_PATH)
            .respond_with(ok(page(vec![message_json("m1", "user", "Hi", 20)], false))),
    )
    .await;
    h.store
        .append_draft(CHAT.into(), "stale-turn".into(), "half a reply".into(), 1)
        .await
        .unwrap();

    let binding = h.sdk.bind_chat("S1".into()).unwrap();
    assert_eq!(binding.session_id(), CHAT);

    eventually("the resync", || async {
        (h.draft().await.is_none() && h.messages().await.len() == 1).then_some(())
    })
    .await;
    binding.release();
}

// Sending

#[tokio::test]
async fn a_sent_message_shows_at_once_and_is_replaced_by_the_servers_copy() {
    let h = Harness::new().await;
    h.mock(
        post(MESSAGES_PATH)
            .and(body_json(json!({ "text": "Hello" })))
            .respond_with(ResponseTemplate::new(201).set_body_json(sent_message("m9", "Hello"))),
    )
    .await;

    let local_id = h
        .sdk
        .send_message(CHAT.into(), "Hello".into())
        .await
        .unwrap();
    assert!(local_id.starts_with("local-"));

    let delivered = eventually("delivery", || async {
        let messages = h.messages().await;
        (messages.len() == 1 && messages[0].id == "m9").then_some(messages)
    })
    .await;
    assert_eq!(delivered[0].status, STATUS_SENT);
    assert!(h.outbox().await.is_empty());
    assert_eq!(h.sessions().await[0].last_message_preview, "Hello");
}

#[tokio::test]
async fn messages_to_one_chat_go_out_in_order() {
    let h = Harness::new().await;
    for (index, text) in ["one", "two", "three"].into_iter().enumerate() {
        h.mock(
            post(MESSAGES_PATH)
                .and(body_json(json!({ "text": text })))
                .respond_with(
                    ResponseTemplate::new(201)
                        .set_body_json(sent_message(&format!("m{index}"), text)),
                ),
        )
        .await;
    }
    for text in ["one", "two", "three"] {
        h.sdk.send_message(CHAT.into(), text.into()).await.unwrap();
    }

    eventually("delivery", || async {
        h.outbox().await.is_empty().then_some(())
    })
    .await;
    let sent: Vec<String> = h
        .requests("POST", MESSAGES_PATH)
        .await
        .iter()
        .map(|request| request.body_json::<Value>().unwrap()["text"].to_string())
        .collect();
    assert_eq!(sent, [r#""one""#, r#""two""#, r#""three""#]);
}

#[tokio::test]
async fn a_failing_backend_is_retried_with_backoff() {
    let h = Harness::new().await;
    h.mock(
        post(MESSAGES_PATH)
            .respond_with(ResponseTemplate::new(503))
            .up_to_n_times(1),
    )
    .await;
    h.mock(
        post(MESSAGES_PATH)
            .respond_with(ResponseTemplate::new(201).set_body_json(sent_message("m9", "Hello"))),
    )
    .await;

    h.sdk
        .send_message(CHAT.into(), "Hello".into())
        .await
        .unwrap();

    eventually("delivery", || async {
        h.outbox().await.is_empty().then_some(())
    })
    .await;
    assert_eq!(h.requests("POST", MESSAGES_PATH).await.len(), 2);
}

#[tokio::test]
async fn a_refused_message_fails_and_can_be_retried_or_discarded() {
    let h = Harness::new().await;
    h.mock(
        post(MESSAGES_PATH)
            .respond_with(ResponseTemplate::new(403))
            .up_to_n_times(1),
    )
    .await;
    h.mock(
        post(MESSAGES_PATH)
            .respond_with(ResponseTemplate::new(201).set_body_json(sent_message("m9", "Hello"))),
    )
    .await;

    let local_id = h
        .sdk
        .send_message(CHAT.into(), "Hello".into())
        .await
        .unwrap();
    let failed = eventually("the failure", || async {
        h.outbox()
            .await
            .into_iter()
            .find(|entry| entry.state == OutboxState::Failed)
    })
    .await;
    assert_eq!(failed.attempts, 1);
    assert!(failed.last_error.unwrap().contains("403"));
    assert_eq!(h.messages().await[0].status, STATUS_FAILED);
    // A refusal isn't retried by itself.
    assert_eq!(h.requests("POST", MESSAGES_PATH).await.len(), 1);

    assert!(h.sdk.retry_outbox(failed.key.clone()).await.unwrap());
    assert!(!h.sdk.retry_outbox(failed.key).await.unwrap());
    eventually("the retried delivery", || async {
        let messages = h.messages().await;
        (messages.len() == 1 && messages[0].id == "m9").then_some(())
    })
    .await;
    assert!(store_lacks(&h.store, &local_id).await);
}

#[tokio::test]
async fn discarding_a_failed_message_removes_it() {
    let h = Harness::new().await;
    h.mock(post(MESSAGES_PATH).respond_with(ResponseTemplate::new(400)))
        .await;

    let local_id = h
        .sdk
        .send_message(CHAT.into(), "Hello".into())
        .await
        .unwrap();
    let failed = eventually("the failure", || async {
        h.outbox()
            .await
            .into_iter()
            .find(|e| e.state == OutboxState::Failed)
    })
    .await;

    assert!(h.sdk.discard_outbox(failed.key.clone()).await.unwrap());
    assert!(!h.sdk.discard_outbox(failed.key).await.unwrap());
    assert!(h.outbox().await.is_empty());
    assert!(store_lacks(&h.store, &local_id).await);
}

async fn store_lacks(store: &AccountStore, message_id: &str) -> bool {
    let id = message_id.to_owned();
    store
        .read(move |connection| crate::storage::repositories::messages::get(connection, &id))
        .await
        .unwrap()
        .is_none()
}

#[tokio::test]
async fn a_message_that_keeps_failing_gives_up_after_its_attempts() {
    let h = Harness::new().await;
    h.mock(post(MESSAGES_PATH).respond_with(ResponseTemplate::new(500)))
        .await;

    h.sdk
        .send_message(CHAT.into(), "Hello".into())
        .await
        .unwrap();

    let failed = eventually("giving up", || async {
        h.outbox()
            .await
            .into_iter()
            .find(|e| e.state == OutboxState::Failed)
    })
    .await;
    assert_eq!(failed.attempts, settings().max_attempts);
    assert_eq!(
        h.requests("POST", MESSAGES_PATH).await.len(),
        settings().max_attempts as usize
    );
}

#[tokio::test]
async fn a_message_that_may_have_landed_is_claimed_instead_of_sent_twice() {
    let h = Harness::new().await;
    // The backend stored it, but the answer was unreadable: the outcome is unknown.
    h.mock(post(MESSAGES_PATH).respond_with(ResponseTemplate::new(201).set_body_string("{ cut")))
        .await;
    h.mock(get(MESSAGES_PATH).respond_with(ok(page(
        vec![
            message_json("m1", "user", "Hello", 20),
            message_json("m2", "assistant", "Hi there", 21),
        ],
        false,
    ))))
    .await;

    h.sdk
        .send_message(CHAT.into(), "Hello".into())
        .await
        .unwrap();

    let messages = eventually("the claim", || async {
        let messages = h.messages().await;
        (h.outbox().await.is_empty() && !messages.is_empty()).then_some(messages)
    })
    .await;
    assert_eq!(messages, vec![stored_message("m1", "user", "Hello", 20)]);
    assert_eq!(h.requests("POST", MESSAGES_PATH).await.len(), 1);
}

#[tokio::test]
async fn an_old_copy_of_the_same_text_is_not_claimed() {
    let h = Harness::with(SyncSettings {
        echo_window: Duration::from_secs(5 * 60),
        ..settings()
    })
    .await;
    h.mock(
        post(MESSAGES_PATH)
            .respond_with(ResponseTemplate::new(201).set_body_string("{ cut"))
            .up_to_n_times(1),
    )
    .await;
    h.mock(
        post(MESSAGES_PATH)
            .respond_with(ResponseTemplate::new(201).set_body_json(sent_message("m9", "ok"))),
    )
    .await;
    // "ok" was said months ago; that isn't this message.
    h.mock(
        get(MESSAGES_PATH)
            .respond_with(ok(page(vec![message_json("m1", "user", "ok", 20)], false))),
    )
    .await;

    h.sdk.send_message(CHAT.into(), "ok".into()).await.unwrap();

    eventually("the resend", || async {
        h.outbox().await.is_empty().then_some(())
    })
    .await;
    assert_eq!(h.requests("POST", MESSAGES_PATH).await.len(), 2);
    let ids: Vec<String> = h.messages().await.into_iter().map(|m| m.id).collect();
    assert_eq!(ids, ["m9"]);
}

#[tokio::test]
async fn queued_writes_are_replayed_after_a_restart() {
    let server = MockServer::start().await;
    let dir = tempfile::tempdir().unwrap();
    // Signed out: writes queue, nothing goes out.
    let (offline, store) = open(&server, &dir, MemoryCredentials::default(), settings()).await;
    let local_id = offline
        .send_message(CHAT.into(), "Written offline".into())
        .await
        .unwrap();
    assert_eq!(
        store.outbox_entries().await.unwrap()[0].operation,
        OutboxOperation::SendMessage {
            session_id: CHAT.into(),
            text: "Written offline".into(),
            local_message_id: local_id,
        }
    );
    offline.shutdown().await;

    post(MESSAGES_PATH)
        .respond_with(
            ResponseTemplate::new(201).set_body_json(sent_message("m9", "Written offline")),
        )
        .mount(&server)
        .await;
    let (online, store) = open(
        &server,
        &dir,
        MemoryCredentials::holding(credentials(USER, "a")),
        settings(),
    )
    .await;

    eventually("the replay", || async {
        store
            .outbox_entries()
            .await
            .unwrap()
            .is_empty()
            .then_some(())
    })
    .await;
    let page = store.messages_page(CHAT.into(), None, LIMIT).await.unwrap();
    assert_eq!(page.messages[0].id, "m9");
    online.shutdown().await;
}

#[tokio::test]
async fn signing_in_wakes_a_waiting_outbox() {
    let server = MockServer::start().await;
    let dir = tempfile::tempdir().unwrap();
    let (sdk, store) = open(&server, &dir, MemoryCredentials::default(), settings()).await;
    post(MESSAGES_PATH)
        .respond_with(ResponseTemplate::new(201).set_body_json(sent_message("m9", "Hi")))
        .mount(&server)
        .await;
    post("/api/auth/login")
        .respond_with(ok(json!({
            "access_token": crate::test_support::token(USER, "a"),
            "refresh_token": "refresh-a"
        })))
        .mount(&server)
        .await;
    sdk.send_message(CHAT.into(), "Hi".into()).await.unwrap();

    sdk.sign_in("me@nomi.test".into(), "secret".into())
        .await
        .unwrap();
    // Signing in doesn't close the account, so the same engine delivers.
    eventually("delivery", || async {
        store
            .outbox_entries()
            .await
            .unwrap()
            .is_empty()
            .then_some(())
    })
    .await;
}

#[tokio::test]
async fn a_message_needs_text_within_the_backends_limit() {
    let h = Harness::new().await;
    assert!(matches!(
        h.sdk.send_message(CHAT.into(), "  \n".into()).await,
        Err(NomiError::InvalidArgument { .. })
    ));
    assert!(matches!(
        h.sdk
            .send_message(CHAT.into(), "x".repeat(super::MAX_MESSAGE_CHARS + 1))
            .await,
        Err(NomiError::InvalidArgument { .. })
    ));
    assert!(h.outbox().await.is_empty());
}

#[tokio::test]
async fn another_users_account_is_never_synced() {
    let server = MockServer::start().await;
    let dir = tempfile::tempdir().unwrap();
    let config = default_sdk_config(server.uri(), dir.path().display().to_string());
    let sdk = NomiSdk::with_settings(
        config,
        MemoryCredentials::holding(credentials(OTHER_USER, "a")).boxed(),
        EngineSettings {
            realtime: RealtimeSettings::default(),
            sync: settings(),
        },
    )
    .unwrap();
    sdk.open_account(USER.into()).await.unwrap();

    assert!(matches!(
        sdk.refresh_sessions().await,
        Err(NomiError::InvalidArgument { .. })
    ));
    sdk.send_message(CHAT.into(), "Hi".into()).await.unwrap();
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert!(server.received_requests().await.unwrap().is_empty());
}

#[tokio::test]
async fn sync_stops_with_its_account() {
    let h = Harness::new().await;
    h.sdk.close_account().await.unwrap();
    assert!(matches!(
        h.sdk.refresh_sessions().await,
        Err(NomiError::AccountClosed)
    ));
    assert!(matches!(
        h.sdk.bind_chat(CHAT.into()),
        Err(NomiError::AccountClosed)
    ));
}

// Notifications

#[tokio::test]
async fn pages_through_the_inbox() {
    let h = Harness::new().await;
    h.mock(
        get("/api/notifications")
            .and(query_param("page", "2"))
            .respond_with(ok(json!({
                "items": [notification_json("n3", false, 1)],
                "unread": 2, "page": 2, "pages": 2, "page_size": 20,
                "total": 3, "counts": {}
            }))),
    )
    .await;
    h.mock(
        get("/api/notifications")
            .and(query_param("page", "1"))
            .respond_with(ok(json!({
                "items": [notification_json("n1", false, 3), notification_json("n2", true, 2)],
                "unread": 2, "page": 1, "pages": 2, "page_size": 20,
                "total": 3, "counts": {}
            }))),
    )
    .await;

    h.sdk.refresh_notifications().await.unwrap();
    assert!(!h.sdk.load_more_notifications().await.unwrap());
    assert!(!h.sdk.load_more_notifications().await.unwrap());

    let notifications = h.notifications().await;
    let ids: Vec<&str> = notifications.iter().map(|n| n.id.as_str()).collect();
    assert_eq!(ids, ["n1", "n2", "n3"]);
    assert!(notifications[0].read_at.is_none());
    assert!(notifications[1].read_at.is_some());
    assert_eq!(h.requests("GET", "/api/notifications").await.len(), 2);
}

#[tokio::test]
async fn marking_read_is_local_at_once_and_reaches_the_backend() {
    let h = Harness::new().await;
    h.mock(post("/api/notifications/n1/read").respond_with(ResponseTemplate::new(204)))
        .await;
    h.store
        .upsert_notifications(vec![NotificationItem {
            id: "n1".into(),
            kind: "deploy".into(),
            title: "T".into(),
            body: "B".into(),
            link: None,
            created_at: 1,
            read_at: None,
        }])
        .await
        .unwrap();

    h.sdk.mark_notification_read("n1".into()).await.unwrap();
    assert!(h.notifications().await[0].read_at.is_some());

    eventually("delivery", || async {
        h.outbox().await.is_empty().then_some(())
    })
    .await;
    assert_eq!(
        h.requests("POST", "/api/notifications/n1/read").await.len(),
        1
    );
}

#[tokio::test]
async fn mark_all_read_folds_in_waiting_single_reads() {
    let server = MockServer::start().await;
    let dir = tempfile::tempdir().unwrap();
    let (sdk, store) = open(&server, &dir, MemoryCredentials::default(), settings()).await;

    sdk.mark_notification_read("n1".into()).await.unwrap();
    sdk.mark_notification_read("n2".into()).await.unwrap();
    sdk.mark_all_notifications_read().await.unwrap();
    sdk.mark_all_notifications_read().await.unwrap();

    let operations: Vec<OutboxOperation> = store
        .outbox_entries()
        .await
        .unwrap()
        .into_iter()
        .map(|entry| entry.operation)
        .collect();
    assert_eq!(operations, [OutboxOperation::MarkAllNotificationsRead]);
}

// Realtime

#[tokio::test]
async fn streams_a_reply_into_the_draft_then_stores_the_finished_message() {
    let h = Harness::new().await;
    h.mock(
        get("/api/sessions/s1/messages/m2").respond_with(ok(message_json(
            "m2",
            "assistant",
            "Hello there",
            30,
        ))),
    )
    .await;
    let delta = |text: &str| json!({ "kind": "Delta", "turn_job_id": "t1", "event": { "TextDelta": { "index": 0, "text": text } } });

    h.event(CHAT_TOPIC, delta("Hello"));
    h.event(CHAT_TOPIC, delta(" there"));
    let draft = eventually("the draft", || async {
        h.draft().await.filter(|draft| draft.text == "Hello there")
    })
    .await;
    assert_eq!(draft.turn_job_id, "t1");

    h.event(
        CHAT_TOPIC,
        json!({ "kind": "TurnCompleted", "turn_job_id": "t1", "message_id": "m2" }),
    );
    // A delta arriving after its turn finished is dropped.
    h.event(CHAT_TOPIC, delta(" again"));
    eventually("the finished reply", || async {
        let messages = h.messages().await;
        (messages.len() == 1 && h.draft().await.is_none()).then_some(())
    })
    .await;
    assert_eq!(
        h.messages().await,
        vec![stored_message("m2", "assistant", "Hello there", 30)]
    );
}

#[tokio::test]
async fn a_failed_turn_clears_its_draft_and_says_why() {
    let h = Harness::new().await;
    let events = h.sdk.subscribe();
    h.event(
        CHAT_TOPIC,
        json!({ "kind": "Delta", "turn_job_id": "t1", "event": { "TextDelta": { "index": 0, "text": "Hm" } } }),
    );
    h.event(
        CHAT_TOPIC,
        json!({ "kind": "TurnFailed", "turn_job_id": "t1", "error": "model unavailable" }),
    );

    let failure = tokio::time::timeout(WAIT, async {
        loop {
            if let Some(SdkEvent::ChatTurnFailed {
                session_id, error, ..
            }) = events.next_event().await
            {
                return (session_id, error);
            }
        }
    })
    .await
    .unwrap();
    assert_eq!(failure, (CHAT.to_owned(), "model unavailable".to_owned()));
    assert_eq!(h.draft().await, None);
}

#[tokio::test]
async fn only_an_assistant_message_ends_the_streaming_draft() {
    let h = Harness::new().await;
    h.mock(
        get("/api/sessions/s1/messages/u1").respond_with(ok(message_json(
            "u1",
            "user",
            "From my phone",
            10,
        ))),
    )
    .await;
    h.mock(
        get("/api/sessions/s1/messages/a1").respond_with(ok(message_json(
            "a1",
            "assistant",
            "Done",
            11,
        ))),
    )
    .await;
    h.store
        .append_draft(CHAT.into(), "t1".into(), "Working".into(), 1)
        .await
        .unwrap();

    h.event(
        CHAT_TOPIC,
        json!({ "kind": "MessageCreated", "message_id": "u1" }),
    );
    eventually("the user message", || async {
        (h.messages().await.len() == 1).then_some(())
    })
    .await;
    assert!(h.draft().await.is_some());

    h.event(
        CHAT_TOPIC,
        json!({ "kind": "MessageCreated", "message_id": "a1" }),
    );
    eventually("the draft to end", || async {
        h.draft().await.is_none().then_some(())
    })
    .await;
    assert_eq!(h.messages().await.len(), 2);
}

#[tokio::test]
async fn a_message_the_server_no_longer_has_is_removed() {
    let h = Harness::new().await;
    h.store
        .upsert_messages(vec![stored_message("m1", "assistant", "Old", 10)])
        .await
        .unwrap();
    h.mock(get("/api/sessions/s1/messages/m1").respond_with(ResponseTemplate::new(404)))
        .await;

    h.event(
        CHAT_TOPIC,
        json!({ "kind": "MessageUpdated", "message_id": "m1" }),
    );

    eventually("the removal", || async {
        h.messages().await.is_empty().then_some(())
    })
    .await;
}

#[tokio::test]
async fn tracks_agents_through_their_lifecycle() {
    let h = Harness::new().await;
    let agents = || async {
        h.store
            .observe_active_agents(None)
            .next()
            .await
            .unwrap()
            .unwrap()
    };

    h.event(
        CHAT_TOPIC,
        json!({ "kind": "AgentPhaseChanged", "agent_session_id": "early", "phase": "writing", "detail": null }),
    );
    h.event(
        CHAT_TOPIC,
        json!({ "kind": "AgentSessionStarted", "agent_session_id": "a1", "session_id": CHAT,
                "agent_type": "coder", "agent_display_name": "Coder", "channel": "web", "sender_label": "x" }),
    );
    h.event(
        CHAT_TOPIC,
        json!({ "kind": "AgentPhaseChanged", "agent_session_id": "a1", "phase": "calling_tool", "detail": "search" }),
    );
    let active = eventually("two agents", || async {
        let active = agents().await;
        (active.len() == 2 && active.iter().any(|a| a.phase == "calling_tool")).then_some(active)
    })
    .await;
    let coder = active.iter().find(|a| a.agent_session_id == "a1").unwrap();
    assert_eq!(
        (coder.agent_type.as_str(), coder.detail.as_deref()),
        ("coder", Some("search"))
    );
    let early = active
        .iter()
        .find(|a| a.agent_session_id == "early")
        .unwrap();
    assert_eq!(early.agent_type, "unknown");

    h.event(
        CHAT_TOPIC,
        json!({ "kind": "AgentSessionEnded", "agent_session_id": "a1", "session_id": CHAT, "reason": "completed" }),
    );
    eventually("the end", || async {
        (agents().await.len() == 1).then_some(())
    })
    .await;

    h.event(
        "user",
        json!({ "kind": "ChatActivity", "session_id": CHAT, "title": null, "project_id": null,
                "outcome": "done", "waiting": null, "agent": null }),
    );
    eventually("the chat to go quiet", || async {
        agents().await.is_empty().then_some(())
    })
    .await;
}

#[tokio::test]
async fn renames_a_stored_chat() {
    let h = Harness::new().await;
    h.store
        .upsert_sessions(vec![Session {
            id: CHAT.into(),
            title: "".into(),
            last_message_preview: "".into(),
            updated_at: 1,
            unread: false,
        }])
        .await
        .unwrap();

    h.event(
        CHAT_TOPIC,
        json!({ "kind": "SessionRenamed", "title": "Trip" }),
    );

    eventually("the rename", || async {
        (h.sessions().await[0].title == "Trip").then_some(())
    })
    .await;
}

#[tokio::test]
async fn user_events_update_the_inbox_and_the_chat_list() {
    let h = Harness::new().await;
    h.sdk.set_foreground_chat(Some("S2".into()));

    h.event(
        "user",
        json!({ "kind": "NotificationCreated", "notification_id": "n1", "category": "quota",
                "title": "Almost out", "body": "B", "link": "/usage" }),
    );
    h.event(
        "user",
        json!({ "kind": "ChatReply", "session_id": "s1", "message_id": "m1", "preview": "Away" }),
    );
    h.event(
        "user",
        json!({ "kind": "ChatReply", "session_id": "s2", "message_id": "m2", "preview": "On screen" }),
    );

    let sessions = eventually("both replies", || async {
        let sessions = h.sessions().await;
        (sessions.len() == 2).then_some(sessions)
    })
    .await;
    let unread = |id: &str| sessions.iter().find(|s| s.id == id).unwrap().unread;
    assert!(unread("s1"));
    assert!(!unread("s2"));
    let notification = eventually("the notification", || async {
        h.notifications().await.into_iter().next()
    })
    .await;
    assert_eq!(
        (notification.kind.as_str(), notification.link.as_deref()),
        ("quota", Some("/usage"))
    );
}

#[tokio::test]
async fn an_update_to_an_unknown_chat_refetches_the_list() {
    let h = Harness::new().await;
    h.mock(get("/api/sessions").respond_with(ok(json!({ "sessions": [
        session_json("s9", Some("New elsewhere"), "Hi", 10),
    ] }))))
    .await;

    h.event(
        "user",
        json!({ "kind": "ChatUpdated", "session_id": "s9", "project_id": null, "title": null }),
    );

    eventually("the refetch", || async {
        h.sessions()
            .await
            .into_iter()
            .find(|s| s.title == "New elsewhere")
    })
    .await;
}

#[tokio::test]
async fn a_resync_refetches_the_chat_and_drops_stale_live_state() {
    let h = Harness::new().await;
    h.mock(get(MESSAGES_PATH).respond_with(ok(page(
        vec![message_json("m1", "assistant", "Missed while away", 10)],
        false,
    ))))
    .await;
    h.store
        .append_draft(CHAT.into(), "t1".into(), "half".into(), 1)
        .await
        .unwrap();

    h.sdk.emit(SdkEvent::TopicResynced {
        topic: CHAT_TOPIC.into(),
    });

    eventually("the resync", || async {
        (h.draft().await.is_none() && h.messages().await.len() == 1).then_some(())
    })
    .await;
}

#[tokio::test]
async fn dropped_events_resync_the_bound_chats_and_the_user_topic() {
    let h = Harness::new().await;
    h.mock(get(MESSAGES_PATH).respond_with(ok(page(vec![], false))))
        .await;
    h.mock(get("/api/sessions").respond_with(ok(json!({ "sessions": [] }))))
        .await;
    h.mock(get("/api/notifications").respond_with(ok(json!({
        "items": [], "unread": 0, "page": 1, "pages": 1, "page_size": 20, "total": 0, "counts": {}
    }))))
    .await;
    let _binding = h.sdk.bind_chat(CHAT.into()).unwrap();
    eventually("the first resync", || async {
        (h.requests("GET", MESSAGES_PATH).await.len() == 1).then_some(())
    })
    .await;

    h.sdk.emit(SdkEvent::EventsDropped { count: 3 });

    eventually("the resyncs", || async {
        let chat = h.requests("GET", MESSAGES_PATH).await.len() == 2;
        let list = h.requests("GET", "/api/sessions").await.len() == 1;
        let inbox = h.requests("GET", "/api/notifications").await.len() == 1;
        (chat && list && inbox).then_some(())
    })
    .await;
}

#[tokio::test]
async fn resolving_an_approval_reports_a_card_no_longer_pending() {
    let h = Harness::new().await;
    h.mock(
        Mock::given(method("PUT"))
            .and(path("/api/sessions/s1/messages/m1/approval"))
            .and(body_json(
                json!({ "decision": "approve", "remember": true }),
            ))
            .respond_with(ResponseTemplate::new(409)),
    )
    .await;

    assert!(matches!(
        h.sdk
            .resolve_approval(CHAT.into(), "m1".into(), true, true)
            .await,
        Err(NomiError::Http { status: 409, .. })
    ));
}

#[tokio::test]
async fn a_message_queued_while_signed_out_waits_as_sending() {
    let server = MockServer::start().await;
    let dir = tempfile::tempdir().unwrap();
    let (sdk, store) = open(&server, &dir, MemoryCredentials::default(), settings()).await;

    sdk.send_message(CHAT.into(), "Later".into()).await.unwrap();
    tokio::time::sleep(Duration::from_millis(100)).await;

    let page = store.messages_page(CHAT.into(), None, LIMIT).await.unwrap();
    assert_eq!(page.messages[0].status, STATUS_SENDING);
    assert_eq!(store.outbox_entries().await.unwrap()[0].attempts, 0);
}

#[test]
fn realtime_events_are_carried_typed() {
    // The same decoder serves the socket and the exported `decode_realtime_event`.
    assert_eq!(
        crate::decode_realtime_event(
            CHAT_TOPIC.into(),
            r#"{"kind":"MessageUpdated","message_id":"m1"}"#.into()
        ),
        RealtimeEvent::MessageUpdated {
            session_id: CHAT.into(),
            message_id: "m1".into(),
        }
    );
    assert!(matches!(
        crate::decode_realtime_event(CHAT_TOPIC.into(), "not json".into()),
        RealtimeEvent::Unknown { kind: None, .. }
    ));
}
