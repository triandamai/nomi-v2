//! Storage through the public API: `NomiSdk` accounts and `AccountStore`.

use std::sync::Arc;
use std::time::Duration;

use crate::{
    default_sdk_config, AccountStore, AgentActivity, Message, MessageCursor, NomiError, NomiSdk,
    NotificationItem, SdkEvent, Session, SessionFilter,
};

use crate::test_support::MemoryCredentials;

const BASE_URL: &str = "https://api-nomi.trian.space";
const ACCOUNT: &str = "0b9c6f0e-account-a";
const OTHER_ACCOUNT: &str = "5d1e2a7c-account-b";
/// How long a stream must stay silent to count as "didn't emit".
const QUIET: Duration = Duration::from_millis(200);
const EMIT_TIMEOUT: Duration = Duration::from_secs(5);
const ALL: u32 = 100;

struct Fixture {
    dir: tempfile::TempDir,
    sdk: Arc<NomiSdk>,
}

impl Fixture {
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let sdk = NomiSdk::new(
            default_sdk_config(BASE_URL.to_owned(), dir.path().display().to_string()),
            MemoryCredentials::default().boxed(),
        )
        .unwrap();
        Self { dir, sdk }
    }

    async fn store(&self) -> Arc<AccountStore> {
        self.sdk.open_account(ACCOUNT.to_owned()).await.unwrap()
    }
}

fn session(id: &str, updated_at: i64, unread: bool) -> Session {
    Session {
        id: id.into(),
        title: format!("Chat {id}"),
        last_message_preview: String::new(),
        updated_at,
        unread,
    }
}

fn message(id: &str, session_id: &str, created_at: i64) -> Message {
    Message {
        id: id.into(),
        session_id: session_id.into(),
        role: "user".into(),
        agent_type: None,
        agent_display_name: None,
        content: format!("text {id}"),
        content_json: None,
        my_feedback: None,
        steps_json: None,
        created_at,
        status: "sent".into(),
    }
}

fn notification(id: &str, created_at: i64, read_at: Option<i64>) -> NotificationItem {
    NotificationItem {
        id: id.into(),
        kind: "reminder".into(),
        title: id.into(),
        body: String::new(),
        link: None,
        created_at,
        read_at,
    }
}

fn activity(
    agent_session_id: &str,
    session_id: &str,
    phase: &str,
    updated_at: i64,
) -> AgentActivity {
    AgentActivity {
        agent_session_id: agent_session_id.into(),
        session_id: session_id.into(),
        agent_type: "planning".into(),
        phase: phase.into(),
        detail: None,
        updated_at,
    }
}

fn filter() -> SessionFilter {
    SessionFilter {
        search: None,
        unread_only: false,
        limit: ALL,
    }
}

fn ids<T>(items: &[T], id: impl Fn(&T) -> &str) -> Vec<String> {
    items.iter().map(|item| id(item).to_owned()).collect()
}

async fn emitted<T>(
    next: impl std::future::Future<Output = Result<Option<T>, NomiError>>,
) -> Option<T> {
    tokio::time::timeout(EMIT_TIMEOUT, next)
        .await
        .expect("the observation never emitted")
        .unwrap()
}

async fn stays_quiet<T>(
    next: impl std::future::Future<Output = Result<Option<T>, NomiError>>,
) -> bool {
    tokio::time::timeout(QUIET, next).await.is_err()
}

// Sessions

#[tokio::test]
async fn lists_sessions_newest_first_and_updates_in_place() {
    let fixture = Fixture::new();
    let store = fixture.store().await;
    store
        .upsert_sessions(vec![
            session("a", 1, false),
            session("b", 3, false),
            session("c", 2, true),
        ])
        .await
        .unwrap();
    store
        .upsert_sessions(vec![Session {
            title: "Renamed".into(),
            ..session("a", 4, false)
        }])
        .await
        .unwrap();

    let observed = emitted(store.observe_sessions(filter()).next())
        .await
        .unwrap();
    assert_eq!(ids(&observed, |s| &s.id), ["a", "b", "c"]);
    assert_eq!(
        store.get_session("a".into()).await.unwrap().unwrap().title,
        "Renamed"
    );
    assert_eq!(store.get_session("missing".into()).await.unwrap(), None);
}

#[tokio::test]
async fn filters_sessions_by_unread_text_and_limit() {
    let fixture = Fixture::new();
    let store = fixture.store().await;
    let mut discount = session("d", 5, false);
    discount.last_message_preview = "50% off".into();
    store
        .upsert_sessions(vec![
            session("a", 1, true),
            session("b", 2, false),
            session("c", 3, true),
            discount,
        ])
        .await
        .unwrap();

    let unread = SessionFilter {
        unread_only: true,
        ..filter()
    };
    assert_eq!(
        ids(
            &emitted(store.observe_sessions(unread).next())
                .await
                .unwrap(),
            |s| &s.id
        ),
        ["c", "a"]
    );

    let search = SessionFilter {
        search: Some("CHAT b".into()),
        ..filter()
    };
    assert_eq!(
        ids(
            &emitted(store.observe_sessions(search).next())
                .await
                .unwrap(),
            |s| &s.id
        ),
        ["b"]
    );

    // LIKE wildcards in the search text are literal.
    let percent = SessionFilter {
        search: Some("0%".into()),
        ..filter()
    };
    assert_eq!(
        ids(
            &emitted(store.observe_sessions(percent).next())
                .await
                .unwrap(),
            |s| &s.id
        ),
        ["d"]
    );

    let limited = SessionFilter {
        limit: 2,
        ..filter()
    };
    assert_eq!(
        emitted(store.observe_sessions(limited).next())
            .await
            .unwrap()
            .len(),
        2
    );
}

#[tokio::test]
async fn marks_a_session_read() {
    let fixture = Fixture::new();
    let store = fixture.store().await;
    store
        .upsert_sessions(vec![session("a", 1, true)])
        .await
        .unwrap();
    store.mark_session_read("a".into()).await.unwrap();
    assert!(!store.get_session("a".into()).await.unwrap().unwrap().unread);
}

#[tokio::test]
async fn deleting_a_session_removes_everything_stored_for_it() {
    let fixture = Fixture::new();
    let store = fixture.store().await;
    store
        .upsert_sessions(vec![session("a", 1, false), session("b", 1, false)])
        .await
        .unwrap();
    store
        .upsert_messages(vec![message("m1", "a", 1), message("m2", "b", 1)])
        .await
        .unwrap();
    store
        .append_draft("a".into(), "turn".into(), "hi".into(), 1)
        .await
        .unwrap();
    store
        .upsert_agent_activity(activity("x", "a", "thinking", 1))
        .await
        .unwrap();

    store.delete_session("a".into()).await.unwrap();

    assert_eq!(store.get_session("a".into()).await.unwrap(), None);
    assert!(store
        .messages_page("a".into(), None, ALL)
        .await
        .unwrap()
        .messages
        .is_empty());
    assert_eq!(
        store
            .messages_page("b".into(), None, ALL)
            .await
            .unwrap()
            .messages
            .len(),
        1
    );
    assert_eq!(
        emitted(store.observe_draft("a".into()).next())
            .await
            .unwrap()
            .draft,
        None
    );
    assert!(emitted(store.observe_active_agents(None).next())
        .await
        .unwrap()
        .is_empty());
}

// Messages

#[tokio::test]
async fn pages_through_messages_oldest_first_with_ties_broken_by_id() {
    let fixture = Fixture::new();
    let store = fixture.store().await;
    let messages = vec![
        message("m1", "s", 10),
        message("m3", "s", 20),
        message("m2", "s", 20),
        message("m4", "s", 30),
    ];
    store.upsert_messages(messages).await.unwrap();
    store
        .upsert_messages(vec![message("other", "elsewhere", 25)])
        .await
        .unwrap();

    let newest = store.messages_page("s".into(), None, 2).await.unwrap();
    assert_eq!(ids(&newest.messages, |m| &m.id), ["m3", "m4"]);
    let cursor = newest.older.clone().unwrap();
    assert_eq!(
        cursor,
        MessageCursor {
            created_at: 20,
            id: "m3".into()
        }
    );

    let older = store
        .messages_page("s".into(), Some(cursor), 2)
        .await
        .unwrap();
    assert_eq!(ids(&older.messages, |m| &m.id), ["m1", "m2"]);
    assert_eq!(older.older, None);

    let observed = emitted(store.observe_messages("s".into(), 3).next())
        .await
        .unwrap();
    assert_eq!(ids(&observed, |m| &m.id), ["m2", "m3", "m4"]);
}

#[tokio::test]
async fn upserting_a_message_replaces_it() {
    let fixture = Fixture::new();
    let store = fixture.store().await;
    store
        .upsert_messages(vec![message("m1", "s", 1)])
        .await
        .unwrap();
    store
        .upsert_messages(vec![Message {
            status: "failed".into(),
            ..message("m1", "s", 1)
        }])
        .await
        .unwrap();
    let page = store.messages_page("s".into(), None, ALL).await.unwrap();
    assert_eq!(page.messages.len(), 1);
    assert_eq!(page.messages[0].status, "failed");

    store.delete_message("m1".into()).await.unwrap();
    assert!(store
        .messages_page("s".into(), None, ALL)
        .await
        .unwrap()
        .messages
        .is_empty());
}

// Drafts

#[tokio::test]
async fn a_draft_grows_within_a_turn_and_restarts_for_a_new_one() {
    let fixture = Fixture::new();
    let store = fixture.store().await;
    let draft = store.observe_draft("s".into());
    assert_eq!(emitted(draft.next()).await.unwrap().draft, None);

    store
        .append_draft("s".into(), "t1".into(), "Hel".into(), 1)
        .await
        .unwrap();
    store
        .set_draft_phase("s".into(), "t1".into(), Some("writing".into()), 2)
        .await
        .unwrap();
    store
        .append_draft("s".into(), "t1".into(), "lo".into(), 3)
        .await
        .unwrap();
    let current = emitted(draft.next()).await.unwrap().draft.unwrap();
    assert_eq!(
        (current.text.as_str(), current.phase.as_deref()),
        ("Hello", Some("writing"))
    );

    store
        .append_draft("s".into(), "t2".into(), "New".into(), 4)
        .await
        .unwrap();
    let current = emitted(draft.next()).await.unwrap().draft.unwrap();
    assert_eq!(
        (
            current.turn_job_id.as_str(),
            current.text.as_str(),
            current.phase
        ),
        ("t2", "New", None)
    );
}

#[tokio::test]
async fn clearing_a_draft_for_another_turn_keeps_it() {
    let fixture = Fixture::new();
    let store = fixture.store().await;
    store
        .append_draft("s".into(), "t2".into(), "kept".into(), 1)
        .await
        .unwrap();
    store
        .clear_draft("s".into(), Some("t1".into()))
        .await
        .unwrap();
    assert!(emitted(store.observe_draft("s".into()).next())
        .await
        .unwrap()
        .draft
        .is_some());
    store.clear_draft("s".into(), None).await.unwrap();
    assert!(emitted(store.observe_draft("s".into()).next())
        .await
        .unwrap()
        .draft
        .is_none());
}

// Notifications and activity

#[tokio::test]
async fn a_notification_read_here_stays_read_after_a_refresh() {
    let fixture = Fixture::new();
    let store = fixture.store().await;
    store
        .upsert_notifications(vec![
            notification("n1", 1, None),
            notification("n2", 2, None),
        ])
        .await
        .unwrap();
    let unread = store.observe_unread_notification_count();
    assert_eq!(emitted(unread.next()).await, Some(2));

    store.mark_all_notifications_read(10).await.unwrap();
    assert_eq!(emitted(unread.next()).await, Some(0));

    store
        .upsert_notifications(vec![
            notification("n1", 1, None),
            notification("n3", 3, None),
        ])
        .await
        .unwrap();
    assert_eq!(emitted(unread.next()).await, Some(1));
    let listed = emitted(store.observe_notifications(ALL).next())
        .await
        .unwrap();
    assert_eq!(ids(&listed, |n| &n.id), ["n3", "n2", "n1"]);
    assert_eq!(listed[2].read_at, Some(10));
}

#[tokio::test]
async fn active_agents_leave_out_idle_ones() {
    let fixture = Fixture::new();
    let store = fixture.store().await;
    store
        .upsert_agent_activity(activity("x", "a", "thinking", 1))
        .await
        .unwrap();
    store
        .upsert_agent_activity(activity("y", "a", "waiting", 2))
        .await
        .unwrap();
    store
        .upsert_agent_activity(activity("z", "b", "writing", 3))
        .await
        .unwrap();

    assert_eq!(
        ids(
            &emitted(store.observe_active_agents(None).next())
                .await
                .unwrap(),
            |a| &a.agent_session_id
        ),
        ["z", "x"]
    );
    let in_a = emitted(store.observe_active_agents(Some("a".into())).next())
        .await
        .unwrap();
    assert_eq!(ids(&in_a, |a| &a.agent_session_id), ["x"]);

    store.delete_agent_activity("x".into()).await.unwrap();
    store.clear_agent_activity("b".into()).await.unwrap();
    assert!(emitted(store.observe_active_agents(None).next())
        .await
        .unwrap()
        .is_empty());
}

#[tokio::test]
async fn clear_all_empties_every_table_and_keeps_the_account_open() {
    let fixture = Fixture::new();
    let store = fixture.store().await;
    store
        .upsert_sessions(vec![session("a", 1, false)])
        .await
        .unwrap();
    store
        .upsert_messages(vec![message("m", "a", 1)])
        .await
        .unwrap();
    store
        .upsert_notifications(vec![notification("n", 1, None)])
        .await
        .unwrap();

    store.clear_all().await.unwrap();

    assert!(emitted(store.observe_sessions(filter()).next())
        .await
        .unwrap()
        .is_empty());
    assert!(store
        .messages_page("a".into(), None, ALL)
        .await
        .unwrap()
        .messages
        .is_empty());
    assert_eq!(
        emitted(store.observe_unread_notification_count().next()).await,
        Some(0)
    );
    store
        .upsert_sessions(vec![session("b", 1, false)])
        .await
        .unwrap();
}

// Observations

#[tokio::test]
async fn an_observation_emits_after_each_relevant_commit_only() {
    let fixture = Fixture::new();
    let store = fixture.store().await;
    let sessions = store.observe_sessions(filter());
    assert_eq!(emitted(sessions.next()).await, Some(vec![]));

    // Another table's write doesn't wake it.
    store
        .upsert_messages(vec![message("m", "a", 1)])
        .await
        .unwrap();
    assert!(stays_quiet(sessions.next()).await);

    store
        .upsert_sessions(vec![session("a", 1, false)])
        .await
        .unwrap();
    assert_eq!(
        ids(&emitted(sessions.next()).await.unwrap(), |s| &s.id),
        ["a"]
    );

    // A write that leaves its answer unchanged doesn't emit either.
    store
        .upsert_sessions(vec![session("a", 1, false)])
        .await
        .unwrap();
    assert!(stays_quiet(sessions.next()).await);
}

#[tokio::test]
async fn a_slow_observer_gets_the_latest_state_not_a_backlog() {
    let fixture = Fixture::new();
    let store = fixture.store().await;
    let sessions = store.observe_sessions(filter());
    emitted(sessions.next()).await;
    for (index, id) in ["a", "b", "c"].into_iter().enumerate() {
        store
            .upsert_sessions(vec![session(id, index as i64, false)])
            .await
            .unwrap();
    }
    assert_eq!(emitted(sessions.next()).await.unwrap().len(), 3);
    assert!(stays_quiet(sessions.next()).await);
}

#[tokio::test]
async fn cancel_ends_a_pending_observation() {
    let fixture = Fixture::new();
    let store = fixture.store().await;
    let sessions = store.observe_sessions(filter());
    emitted(sessions.next()).await;
    let pending = tokio::spawn({
        let sessions = sessions.clone();
        async move { sessions.next().await }
    });
    tokio::task::yield_now().await;
    sessions.cancel();
    assert_eq!(pending.await.unwrap(), Ok(None));
    assert_eq!(sessions.next().await, Ok(None));
}

// Accounts

#[tokio::test]
async fn each_account_has_its_own_database() {
    let fixture = Fixture::new();
    let first = fixture.store().await;
    first
        .upsert_sessions(vec![session("a", 1, false)])
        .await
        .unwrap();
    let status = first.database_status().await.unwrap();
    assert!(status
        .path
        .ends_with(&format!("accounts/{ACCOUNT}/nomi.db")));
    assert!(status.size_bytes > 0);

    let second = fixture
        .sdk
        .open_account(OTHER_ACCOUNT.into())
        .await
        .unwrap();
    assert!(emitted(second.observe_sessions(filter()).next())
        .await
        .unwrap()
        .is_empty());

    let reopened = fixture.store().await;
    assert_eq!(
        emitted(reopened.observe_sessions(filter()).next())
            .await
            .unwrap()
            .len(),
        1
    );
}

#[tokio::test]
async fn switching_accounts_closes_the_previous_store() {
    let fixture = Fixture::new();
    let first = fixture.store().await;
    let observation = first.observe_sessions(filter());
    emitted(observation.next()).await;

    fixture
        .sdk
        .open_account(OTHER_ACCOUNT.into())
        .await
        .unwrap();

    assert_eq!(
        first.upsert_sessions(vec![session("a", 1, false)]).await,
        Err(NomiError::AccountClosed)
    );
    assert_eq!(emitted(observation.next()).await, None);
    assert_eq!(
        fixture.sdk.current_account().unwrap().account_id(),
        OTHER_ACCOUNT
    );
}

#[tokio::test]
async fn opening_the_open_account_again_returns_the_same_store() {
    let fixture = Fixture::new();
    let first = fixture.store().await;
    let again = fixture.store().await;
    assert!(Arc::ptr_eq(&first, &again));
    assert_eq!(
        fixture.sdk.diagnostics().account_id.as_deref(),
        Some(ACCOUNT)
    );
}

#[tokio::test]
async fn account_changes_are_announced_once_each() {
    let fixture = Fixture::new();
    let events = fixture.sdk.subscribe();
    fixture.store().await;
    fixture.store().await;
    fixture.sdk.close_account().await.unwrap();
    fixture.sdk.close_account().await.unwrap();

    let mut changes = Vec::new();
    while let Ok(Some(event)) = tokio::time::timeout(QUIET, events.next_event()).await {
        if let SdkEvent::AccountChanged { account_id } = event {
            changes.push(account_id);
        }
    }
    assert_eq!(changes, [Some(ACCOUNT.to_owned()), None]);
}

#[tokio::test]
async fn removing_an_account_deletes_its_data() {
    let fixture = Fixture::new();
    let store = fixture.store().await;
    store
        .upsert_sessions(vec![session("a", 1, false)])
        .await
        .unwrap();
    let directory = fixture.dir.path().join("accounts").join(ACCOUNT);
    assert!(directory.is_dir());

    fixture.sdk.remove_account(ACCOUNT.into()).await.unwrap();

    assert!(!directory.exists());
    assert!(fixture.sdk.current_account().is_none());
    assert_eq!(
        store.get_session("a".into()).await,
        Err(NomiError::AccountClosed)
    );
    let fresh = fixture.store().await;
    assert_eq!(fresh.get_session("a".into()).await.unwrap(), None);
}

#[tokio::test]
async fn removing_another_account_leaves_the_open_one_alone() {
    let fixture = Fixture::new();
    fixture
        .sdk
        .open_account(OTHER_ACCOUNT.into())
        .await
        .unwrap();
    let store = fixture.store().await;
    fixture
        .sdk
        .remove_account(OTHER_ACCOUNT.into())
        .await
        .unwrap();
    store
        .upsert_sessions(vec![session("a", 1, false)])
        .await
        .unwrap();
    assert!(!fixture
        .dir
        .path()
        .join("accounts")
        .join(OTHER_ACCOUNT)
        .exists());
}

#[tokio::test]
async fn reset_storage_deletes_every_account() {
    let fixture = Fixture::new();
    fixture
        .sdk
        .open_account(OTHER_ACCOUNT.into())
        .await
        .unwrap();
    fixture.store().await;
    fixture.sdk.reset_storage().await.unwrap();
    assert!(!fixture.dir.path().join("accounts").exists());
    assert!(fixture.sdk.current_account().is_none());
}

#[tokio::test]
async fn rejects_account_ids_that_are_not_plain_names() {
    let fixture = Fixture::new();
    for id in [
        "",
        "..",
        ".hidden",
        "a/b",
        "a\\b",
        "with space",
        &"x".repeat(129),
    ] {
        assert!(
            matches!(
                fixture.sdk.open_account(id.to_owned()).await,
                Err(NomiError::InvalidArgument { .. })
            ),
            "{id:?}"
        );
    }
}

#[tokio::test]
async fn shutdown_closes_the_account_and_refuses_new_ones() {
    let fixture = Fixture::new();
    let store = fixture.store().await;
    let observation = store.observe_sessions(filter());
    emitted(observation.next()).await;

    fixture.sdk.shutdown().await;

    assert!(fixture.sdk.current_account().is_none());
    assert_eq!(emitted(observation.next()).await, None);
    assert_eq!(
        store.get_session("a".into()).await,
        Err(NomiError::ShutDown)
    );
    assert!(matches!(
        fixture.sdk.open_account(ACCOUNT.into()).await,
        Err(NomiError::ShutDown)
    ));
}
