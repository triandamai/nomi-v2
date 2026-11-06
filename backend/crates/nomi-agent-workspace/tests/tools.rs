//! Every Workspace tool, run through the agent the way a turn runs it: the asker's own token,
//! what it sends Google, the card it shows, the activity it records, and how failures read.

use std::sync::Arc;

use base64::Engine;
use serde_json::{json, Value};
use sqlx::PgPool;
use uuid::Uuid;
use wiremock::matchers::{body_partial_json, body_string_contains, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

use nomi_agent_core::content_block::{ContentBlock, ToolOutcome};
use nomi_agent_core::{Locale, SubAgent};
use nomi_agent_workspace::connection::{self, GoogleConfig};
use nomi_agent_workspace::WorkspaceAgent;

const KEY: [u8; 32] = [7u8; 32];
const ALL: [&str; 5] = ["gmail", "sheets", "docs", "drive", "calendar"];

async fn seed_user(pool: &PgPool) -> Uuid {
    sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(pool).await.unwrap()
}

async fn connect(pool: &PgPool, server: &MockServer, user: Uuid, services: &[&str]) {
    let config = GoogleConfig::all_at(&server.uri());
    let scopes: Vec<&str> = services.iter().filter_map(|s| connection::service_scope(s)).collect();
    Mock::given(method("POST"))
        .and(path("/token"))
        .and(body_string_contains("grant_type=authorization_code"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "access_token": "tok", "expires_in": 3600, "refresh_token": "tok-refresh", "scope": format!("openid email {}", scopes.join(" "))
        })))
        .mount(server)
        .await;
    Mock::given(method("GET"))
        .and(path("/v1/userinfo"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "email": "ana@example.com" })))
        .mount(server)
        .await;
    let mut conn = pool.acquire().await.unwrap();
    let wanted: Vec<String> = services.iter().map(|s| s.to_string()).collect();
    let url = connection::start_authorization(&mut conn, &config, user, &wanted, None, Some("ana@example.com")).await.unwrap();
    assert!(url.contains("login_hint=ana%40example.com"));
    let state = reqwest::Url::parse(&url).unwrap().query_pairs().find(|(k, _)| k == "state").unwrap().1.to_string();
    connection::complete_authorization(&mut conn, &reqwest::Client::new(), &config, &KEY, user, "code", &state).await.unwrap();
}

fn agent(server: &MockServer) -> WorkspaceAgent {
    WorkspaceAgent::new(reqwest::Client::new(), Some(Arc::new(GoogleConfig::all_at(&server.uri()))), KEY)
}

async fn run(agent: &WorkspaceAgent, pool: &PgPool, user: Uuid, tool: &str, input: Value) -> Result<ToolOutcome, String> {
    let mut conn = pool.acquire().await.unwrap();
    agent.execute_tool(&mut conn, Uuid::new_v4(), Uuid::new_v4(), user, tool, input).await
}

async fn google(server: &MockServer, verb: &str, at: &str, body: Value) {
    Mock::given(method(verb)).and(path(at)).respond_with(ResponseTemplate::new(200).set_body_json(body)).mount(server).await;
}

async fn activity(pool: &PgPool, user: Uuid) -> Vec<String> {
    let mut conn = pool.acquire().await.unwrap();
    connection::recent_activity(&mut conn, user, 50).await.unwrap().into_iter().map(|a| format!("{}: {}", a.service, a.summary)).collect()
}

fn card(outcome: &ToolOutcome) -> Value {
    match &outcome.block {
        Some(ContentBlock::View(view)) => serde_json::to_value(view).unwrap(),
        other => panic!("expected a card, got {other:?}"),
    }
}

fn b64(text: &str) -> String {
    base64::engine::general_purpose::URL_SAFE.encode(text)
}

#[sqlx::test(migrations = "../../migrations")]
async fn gmail_reads_drafts_replies_and_sends(pool: PgPool) {
    let server = MockServer::start().await;
    let user = seed_user(&pool).await;
    connect(&pool, &server, user, &ALL).await;
    let agent = agent(&server);

    let long_body = "é".repeat(7_000);
    google(&server, "GET", "/gmail/v1/users/me/messages/m1", json!({
        "threadId": "t1",
        "payload": { "mimeType": "multipart/mixed", "headers": [
            {"name": "From", "value": "Villa <stay@villa.example>"}, {"name": "to", "value": "ana@example.com"},
            {"name": "Subject", "value": "Booking"}, {"name": "Date", "value": "Mon, 2 Nov 2026"}, {"name": "Message-ID", "value": "<abc@villa>"}
        ], "parts": [{ "mimeType": "text/plain", "body": { "data": b64(&long_body) } }] }
    }))
    .await;
    let read = run(&agent, &pool, user, "gmail_read", json!({"message_id": "m1"})).await.unwrap();
    let read: Value = serde_json::from_str(&read.display_text).unwrap();
    assert_eq!(read["subject"], "Booking");
    assert_eq!(read["to"], "ana@example.com", "headers match whatever their case");
    assert!(read["body"].as_str().unwrap().ends_with("[… cut]"), "a long body is cut on a character boundary");
    assert!(read.get("thread_id").is_some());

    // A reply keeps the thread and the subject, and says what it answers.
    Mock::given(method("POST"))
        .and(path("/gmail/v1/users/me/drafts"))
        .and(body_partial_json(json!({"message": {"threadId": "t1"}})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"id": "d1"})))
        .mount(&server)
        .await;
    let draft = run(&agent, &pool, user, "gmail_draft", json!({"to": "stay@villa.example", "body": "Thanks!", "reply_to_message_id": "m1"})).await.unwrap();
    assert!(draft.display_text.contains("d1"));
    let draft_card = card(&draft);
    assert!(draft_card.to_string().contains("stay@villa.example"));
    let raw = server.received_requests().await.unwrap().into_iter().find(|r| r.url.path() == "/gmail/v1/users/me/drafts").unwrap();
    let raw: Value = serde_json::from_slice(&raw.body).unwrap();
    let raw = String::from_utf8(base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(raw["message"]["raw"].as_str().unwrap()).unwrap()).unwrap();
    assert!(raw.contains("In-Reply-To: <abc@villa>"));
    assert!(raw.contains(&format!("Subject: =?UTF-8?B?{}?=", base64::engine::general_purpose::STANDARD.encode("Re: Booking"))));

    google(&server, "POST", "/gmail/v1/users/me/messages/send", json!({"id": "s1"})).await;
    let sent = run(&agent, &pool, user, "gmail_send", json!({"to": "stay@villa.example", "subject": "Dates", "body": "3 nights"})).await.unwrap();
    assert!(sent.display_text.contains("s1"));
    assert!(card(&sent).to_string().contains("Dates"));

    let log = activity(&pool, user).await;
    assert_eq!(log.len(), 2);
    assert!(log.iter().all(|l| l.starts_with("gmail: ")));
}

#[sqlx::test(migrations = "../../migrations")]
async fn gmail_reads_html_only_mail_and_searches_with_metadata(pool: PgPool) {
    let server = MockServer::start().await;
    let user = seed_user(&pool).await;
    connect(&pool, &server, user, &["gmail"]).await;
    let agent = agent(&server);
    google(&server, "GET", "/gmail/v1/users/me/messages/h1", json!({
        "payload": { "mimeType": "text/html", "headers": [], "body": { "data": b64("<p>Your <b>receipt</b></p>") } }
    }))
    .await;
    let read: Value = serde_json::from_str(&run(&agent, &pool, user, "gmail_read", json!({"message_id": "h1"})).await.unwrap().display_text).unwrap();
    assert_eq!(read["body"], "Your receipt");
    assert_eq!(read["from"], Value::Null);

    Mock::given(method("GET"))
        .and(path("/gmail/v1/users/me/messages"))
        .and(query_param("q", "from:villa"))
        .and(query_param("maxResults", "25"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"messages": [{"id": "h1"}]})))
        .mount(&server)
        .await;
    let found: Value = serde_json::from_str(&run(&agent, &pool, user, "gmail_search", json!({"query": "from:villa", "max_results": 99})).await.unwrap().display_text).unwrap();
    assert_eq!(found["messages"][0]["id"], "h1");
}

#[sqlx::test(migrations = "../../migrations")]
async fn sheets_read_append_update_and_create(pool: PgPool) {
    let server = MockServer::start().await;
    let user = seed_user(&pool).await;
    connect(&pool, &server, user, &ALL).await;
    let agent = agent(&server);

    google(&server, "GET", "/v4/spreadsheets/sh1", json!({"properties": {"title": "Trips"}, "spreadsheetUrl": "https://docs.google.com/x", "sheets": [{"properties": {"title": "Stays"}}]})).await;
    let info: Value = serde_json::from_str(&run(&agent, &pool, user, "sheets_read", json!({"spreadsheet_id": "sh1"})).await.unwrap().display_text).unwrap();
    assert_eq!(info["tabs"], json!(["Stays"]));

    google(&server, "GET", "/v4/spreadsheets/sh1/values/Stays!A1:B2", json!({"range": "Stays!A1:B2", "values": [["a", "b"]]})).await;
    let cells: Value = serde_json::from_str(&run(&agent, &pool, user, "sheets_read", json!({"spreadsheet_id": "sh1", "range": "Stays!A1:B2"})).await.unwrap().display_text).unwrap();
    assert_eq!(cells["values"], json!([["a", "b"]]));

    google(&server, "POST", "/v4/spreadsheets/sh1/values/Stays!A:F:append", json!({"updates": {"updatedRange": "Stays!A5:F6", "updatedRows": 2}})).await;
    let appended = run(&agent, &pool, user, "sheets_append", json!({"spreadsheet_id": "sh1", "range": "Stays!A:F", "rows": [["x"], ["y"]]})).await.unwrap();
    assert!(appended.display_text.contains("Stays!A5:F6"));
    assert!(appended.block.is_none(), "edits to a sheet show no card of their own");
    run(&agent, &pool, user, "sheets_append", json!({"spreadsheet_id": "sh1", "range": "Stays!A:F", "rows": [["z"]]})).await.unwrap();

    Mock::given(method("PUT"))
        .and(path("/v4/spreadsheets/sh1/values/Stays!A1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"updatedRange": "Stays!A1", "updatedCells": 1})))
        .mount(&server)
        .await;
    run(&agent, &pool, user, "sheets_update", json!({"spreadsheet_id": "sh1", "range": "Stays!A1", "rows": [["new"]]})).await.unwrap();

    let bad = run(&agent, &pool, user, "sheets_update", json!({"spreadsheet_id": "sh1", "range": "Stays!A1", "rows": "nope"})).await.unwrap_err();
    assert_eq!(bad, "rows must be a list of rows");

    google(&server, "POST", "/v4/spreadsheets", json!({"spreadsheetId": "sh2", "spreadsheetUrl": "https://docs.google.com/sh2"})).await;
    Mock::given(method("PUT"))
        .and(path("/v4/spreadsheets/sh2/values/A1"))
        .and(body_partial_json(json!({"values": [["Date", "Place"]]})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({})))
        .expect(1)
        .mount(&server)
        .await;
    let created = run(&agent, &pool, user, "sheets_create", json!({"title": "Budget", "header": ["Date", "Place"]})).await.unwrap();
    assert!(card(&created).to_string().contains("https://docs.google.com/spreadsheets/d/sh2/edit"));

    let log = activity(&pool, user).await;
    assert_eq!(log.len(), 4, "{log:?}");
    assert!(log.iter().all(|l| l.starts_with("sheets: ")));
}

#[sqlx::test(migrations = "../../migrations")]
async fn docs_drive_and_calendar(pool: PgPool) {
    let server = MockServer::start().await;
    let user = seed_user(&pool).await;
    connect(&pool, &server, user, &ALL).await;
    let agent = agent(&server);

    google(&server, "GET", "/v1/documents/doc1", json!({"title": "Notes", "body": {"content": [
        {"paragraph": {"elements": [{"textRun": {"content": "Hello "}}, {"textRun": {"content": "world"}}]}}, {"sectionBreak": {}}
    ]}}))
    .await;
    let doc: Value = serde_json::from_str(&run(&agent, &pool, user, "docs_read", json!({"document_id": "doc1"})).await.unwrap().display_text).unwrap();
    assert_eq!(doc["text"], "Hello world");

    google(&server, "POST", "/v1/documents", json!({"documentId": "doc2"})).await;
    Mock::given(method("POST")).and(path("/v1/documents/doc2:batchUpdate")).respond_with(ResponseTemplate::new(200).set_body_string("")).expect(2).mount(&server).await;
    let created = run(&agent, &pool, user, "docs_create", json!({"title": "Plan", "text": "Day 1"})).await.unwrap();
    assert!(card(&created).to_string().contains("https://docs.google.com/document/d/doc2/edit"));
    let appended = run(&agent, &pool, user, "docs_append", json!({"document_id": "doc2", "text": "Day 2"})).await.unwrap();
    assert!(appended.display_text.contains("text added"));

    Mock::given(method("GET"))
        .and(path("/drive/v3/files"))
        .and(query_param("q", "name contains 'Ana\\'s \\\\ trip' and trashed = false"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"files": [{"id": "f1", "name": "Ana's trip"}]})))
        .mount(&server)
        .await;
    let files: Value = serde_json::from_str(&run(&agent, &pool, user, "drive_search", json!({"query": "Ana's \\ trip"})).await.unwrap().display_text).unwrap();
    assert_eq!(files["files"][0]["id"], "f1");

    Mock::given(method("GET"))
        .and(path("/calendar/v3/calendars/primary/events"))
        .and(query_param("timeMax", "2026-11-08T00:00:00Z"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"items": [{"summary": "Flight", "start": {"dateTime": "x"}, "htmlLink": "https://cal/1"}]})))
        .mount(&server)
        .await;
    let events: Value = serde_json::from_str(
        &run(&agent, &pool, user, "calendar_events", json!({"time_min": "2026-11-07T00:00:00Z", "time_max": "2026-11-08T00:00:00Z"})).await.unwrap().display_text,
    )
    .unwrap();
    assert_eq!(events["events"][0]["summary"], "Flight");

    Mock::given(method("POST"))
        .and(path("/calendar/v3/calendars/primary/events"))
        .and(query_param("sendUpdates", "all"))
        .and(body_partial_json(json!({"location": "Ubud", "attendees": [{"email": "ben@example.com"}]})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"id": "ev1", "htmlLink": "https://calendar.google.com/ev1"})))
        .mount(&server)
        .await;
    let added = run(&agent, &pool, user, "calendar_add_event", json!({
        "summary": "Dinner", "start": "2026-11-07T19:00:00+08:00", "end": "2026-11-07T21:00:00+08:00", "location": "Ubud", "description": "", "attendees": ["ben@example.com"]
    }))
    .await
    .unwrap();
    let shown = card(&added).to_string();
    assert!(shown.contains("Dinner") && shown.contains("ben@example.com") && shown.contains("https://calendar.google.com/ev1"), "{shown}");

    let log = activity(&pool, user).await;
    assert_eq!(log.iter().filter(|l| l.starts_with("docs: ")).count(), 2);
    assert_eq!(log.iter().filter(|l| l.starts_with("calendar: ")).count(), 1);
}

#[sqlx::test(migrations = "../../migrations")]
async fn failures_read_as_google_said_and_a_revoked_token_asks_to_reconnect(pool: PgPool) {
    let server = MockServer::start().await;
    let user = seed_user(&pool).await;
    connect(&pool, &server, user, &ALL).await;
    let agent = agent(&server);

    assert_eq!(run(&agent, &pool, user, "gmail_read", json!({})).await.unwrap_err(), "missing message_id");
    assert_eq!(run(&agent, &pool, user, "gmail_read", json!({"message_id": "  "})).await.unwrap_err(), "missing message_id");
    assert_eq!(run(&agent, &pool, user, "teleport", json!({})).await.unwrap_err(), "unknown tool teleport");

    Mock::given(method("GET"))
        .and(path("/v1/documents/missing"))
        .respond_with(ResponseTemplate::new(404).set_body_json(json!({"error": {"message": "Requested entity was not found."}})))
        .mount(&server)
        .await;
    let error = run(&agent, &pool, user, "docs_read", json!({"document_id": "missing"})).await.unwrap_err();
    assert_eq!(error, "Google returned 404 Not Found: Requested entity was not found.");
    Mock::given(method("GET")).and(path("/v1/documents/plain")).respond_with(ResponseTemplate::new(500).set_body_string("oops")).mount(&server).await;
    assert_eq!(run(&agent, &pool, user, "docs_read", json!({"document_id": "plain"})).await.unwrap_err(), "Google returned 500 Internal Server Error: oops");

    Mock::given(method("GET")).and(path("/drive/v3/files")).respond_with(ResponseTemplate::new(401)).mount(&server).await;
    let revoked = run(&agent, &pool, user, "drive_search", json!({"query": "x"})).await.unwrap();
    assert!(matches!(revoked.block, Some(ContentBlock::WorkspaceConnect { ref reason, .. }) if reason == "not_connected"));
    let left: i64 = sqlx::query_scalar("SELECT count(*) FROM workspace_connections WHERE user_id = $1").bind(user).fetch_one(&pool).await.unwrap();
    assert_eq!(left, 0, "a revoked connection is forgotten");
}

#[sqlx::test(migrations = "../../migrations")]
async fn without_google_set_up_the_agent_says_so(pool: PgPool) {
    let user = seed_user(&pool).await;
    let agent = WorkspaceAgent::new(reqwest::Client::new(), None, KEY);
    let outcome = run(&agent, &pool, user, "gmail_search", json!({"query": "x"})).await.unwrap();
    assert!(outcome.display_text.contains("isn't set up"));
    assert!(outcome.block.is_none());
}

#[test]
fn the_agent_describes_itself_and_what_waits_for_approval() {
    let agent = WorkspaceAgent::new(reqwest::Client::new(), None, KEY);
    assert_eq!(agent.agent_type(), "workspace");
    assert_eq!(agent.intent_label(), "workspace");
    assert!(agent.system_prompt().contains("Tara"));
    assert!(agent.intent_description().contains("Google"));
    assert!(agent.uses_memory() && agent.uses_personality() && agent.is_delegation_target() && agent.supports_todos() && agent.wants_current_time());
    assert_eq!(agent.tools().len(), 14);

    let none = agent.describe_action("calendar_add_event", &json!({"start": "9am"}), Locale::En).unwrap();
    let one = agent.describe_action("calendar_add_event", &json!({"summary": "Dinner", "start": "7pm", "attendees": ["a@x"]}), Locale::En).unwrap();
    let many = agent.describe_action("calendar_add_event", &json!({"summary": "Dinner", "start": "7pm", "attendees": ["a@x", "b@x", "c@x"]}), Locale::En).unwrap();
    assert!(one.contains("Dinner") && one.len() > none.len().min(one.len()));
    assert!(many.contains('3'), "{many}");
    assert_ne!(one, many);
    let to_someone = agent.describe_action("gmail_send", &json!({"body": "x".repeat(400)}), Locale::En).unwrap();
    assert!(to_someone.ends_with('…'), "a long body is clipped: {to_someone}");
    assert!(agent.describe_action("gmail_draft", &json!({}), Locale::En).is_none());
    // The agent built from the environment reads Google from Admin → Configuration each call.
    let _ = WorkspaceAgent::from_env();
}
