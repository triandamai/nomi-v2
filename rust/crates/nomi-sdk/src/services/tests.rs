//! The account pages against a fake backend (wiremock): each call's method, path and body as the
//! backend routes take them, and its answer decoded.

use std::sync::Arc;

use serde_json::{json, Value};
use wiremock::matchers::{body_json, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

use super::home::PreferencesUpdate;
use super::money::{NewTransaction, NewTransactionItem};
use super::reminders::{NewReminder, ReminderAction};
use crate::error::NomiError;
use crate::test_support::{credentials, MemoryCredentials};
use crate::{default_sdk_config, NomiSdk};

const USER: &str = "5b2d8e14-user";
const AT: &str = "2026-10-31T09:00:00Z";

struct Harness {
    _dir: tempfile::TempDir,
    server: MockServer,
    sdk: Arc<NomiSdk>,
}

impl Harness {
    async fn new() -> Self {
        let server = MockServer::start().await;
        let dir = tempfile::tempdir().unwrap();
        let config = default_sdk_config(server.uri(), dir.path().display().to_string());
        let store = MemoryCredentials::holding(credentials(USER, "a"));
        let sdk = NomiSdk::new(config, store.boxed()).unwrap();
        Self {
            _dir: dir,
            server,
            sdk,
        }
    }

    async fn answer(&self, verb: &str, url_path: &str, status: u16, body: Option<Value>) {
        let response = match body {
            Some(body) => ResponseTemplate::new(status).set_body_json(body),
            None => ResponseTemplate::new(status),
        };
        Mock::given(method(verb))
            .and(path(url_path))
            .respond_with(response)
            .expect(1)
            .mount(&self.server)
            .await;
    }

    /// Answers `verb url_path` with `status` only if the request carried exactly `body`.
    async fn expect_body(&self, verb: &str, url_path: &str, body: Value, status: u16) {
        Mock::given(method(verb))
            .and(path(url_path))
            .and(body_json(body))
            .respond_with(ResponseTemplate::new(status))
            .expect(1)
            .mount(&self.server)
            .await;
    }
}

#[tokio::test]
async fn reads_the_crew_preferences_and_home_summary() {
    let h = Harness::new().await;
    h.answer(
        "GET",
        "/api/agents",
        200,
        Some(json!({ "agents": [{
            "agent_type": "money", "name": "Finley", "role": "Money", "is_dynamic": false,
            "shape": null, "tone": null, "motion": null, "state": "done", "status": "Finished"
        }] })),
    )
    .await;
    h.answer(
        "GET",
        "/api/preferences",
        200,
        Some(json!({ "theme": "dark", "accent_color": "ocean", "timezone": "Asia/Jakarta", "extra": 1 })),
    )
    .await;
    h.answer(
        "GET",
        "/api/home",
        200,
        Some(json!({
            "since": AT, "timezone": "Asia/Jakarta",
            "while_you_were_out": [{ "kind": "reply", "agent": "money", "title": "T", "detail": "D", "session_id": null, "at": AT }],
            "today": [], "plans": [{ "kind": "plan", "title": "P", "agent": "coder", "done": 1, "total": 3, "session_id": "s1", "updated_at": AT }],
            "while_you_were_out_total": 4, "today_total": 0, "plans_total": 1
        })),
    )
    .await;

    let crew = h.sdk.crew().await.unwrap();
    assert_eq!(
        (crew[0].name.as_str(), crew[0].state.as_str()),
        ("Finley", "done")
    );

    let preferences = h.sdk.preferences().await.unwrap();
    assert_eq!(preferences.theme, "dark");
    // Fields the backend left out take the web app's defaults.
    assert!(preferences.drawer_recents);
    assert_eq!(preferences.drawer_recents_limit, 10);

    let home = h.sdk.home_summary().await.unwrap();
    assert_eq!(home.while_you_were_out_total, 4);
    assert_eq!(home.plans[0].done, 1);
}

#[tokio::test]
async fn updates_only_the_preferences_given() {
    let h = Harness::new().await;
    Mock::given(method("PUT"))
        .and(path("/api/preferences"))
        .and(body_json(json!({ "theme": "light" })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "theme": "light" })))
        .expect(1)
        .mount(&h.server)
        .await;

    let update = PreferencesUpdate {
        theme: Some("light".into()),
        ..PreferencesUpdate::default()
    };
    assert_eq!(
        h.sdk.update_preferences(update).await.unwrap().theme,
        "light"
    );
}

fn summary_json() -> Value {
    json!({
        "month": "2026-10", "start_day": 1, "period_start": "2026-10-01", "period_end": "2026-10-31",
        "timezone": "UTC", "total_cents": 12_345, "income_cents": 0, "previous_total_cents": 100,
        "transaction_count": 1,
        "by_category": [{ "category": "food", "cents": 12_345, "count": 1 }],
        "by_day": [{ "date": "2026-10-31", "cents": 12_345 }],
        "transactions": [{
            "id": "t1", "occurred_at": AT, "amount_cents": 12_345, "category": "food",
            "description": "Lunch", "kind": "expense", "source": "manual", "created_at": AT,
            "items": [{ "name": "Rice", "quantity": 2.0, "unit_amount_cents": 500, "amount_cents": 1_000, "category": null }]
        }],
        "months": ["2026-10"], "budgets": [{ "category": "food", "limit_cents": 50_000, "spent_cents": 12_345 }]
    })
}

#[tokio::test]
async fn reads_a_money_month() {
    let h = Harness::new().await;
    Mock::given(method("GET"))
        .and(path("/api/money"))
        .and(query_param("month", "2026-10"))
        .respond_with(ResponseTemplate::new(200).set_body_json(summary_json()))
        .expect(1)
        .mount(&h.server)
        .await;

    let summary = h.sdk.money_summary(Some("2026-10".into())).await.unwrap();
    assert_eq!(summary.total_cents, 12_345);
    assert_eq!(summary.transactions[0].items[0].quantity, 2.0);
    assert_eq!(summary.budgets[0].limit_cents, 50_000);
}

#[tokio::test]
async fn adds_a_transaction_in_currency_units() {
    let h = Harness::new().await;
    // The backend reads `amount` and items' `amount`/`unit_amount` in currency units.
    h.expect_body(
        "POST",
        "/api/money/transactions",
        json!({
            "kind": "expense", "amount": 123.45, "category": "food", "description": "Lunch",
            "items": [{ "name": "Rice", "quantity": 2.0, "unit_amount": 5.0 }]
        }),
        201,
    )
    .await;

    h.sdk
        .add_transaction(NewTransaction {
            kind: "expense".into(),
            amount_cents: Some(12_345),
            category: "food".into(),
            description: "Lunch".into(),
            occurred_at: None,
            items: vec![NewTransactionItem {
                name: "Rice".into(),
                quantity: 2.0,
                unit_amount_cents: Some(500),
                amount_cents: None,
                category: None,
            }],
        })
        .await
        .unwrap();
}

#[tokio::test]
async fn sets_a_budget_with_put() {
    let h = Harness::new().await;
    h.expect_body(
        "PUT",
        "/api/money/budgets",
        json!({ "category": "food", "monthly_limit": 500.0 }),
        204,
    )
    .await;

    h.sdk.set_budget("food".into(), 50_000).await.unwrap();
}

#[tokio::test]
async fn a_refused_write_carries_the_backends_reason() {
    let h = Harness::new().await;
    Mock::given(method("POST"))
        .and(path("/api/money/transactions"))
        .respond_with(ResponseTemplate::new(400).set_body_string("category is required"))
        .mount(&h.server)
        .await;

    let refused = h
        .sdk
        .add_transaction(NewTransaction {
            kind: "expense".into(),
            amount_cents: Some(100),
            category: "".into(),
            description: "".into(),
            occurred_at: None,
            items: vec![],
        })
        .await;
    assert_eq!(
        refused,
        Err(NomiError::Http {
            status: 400,
            reason: "category is required".into()
        })
    );
}

fn reminder_json(id: &str) -> Value {
    json!({
        "id": id, "user_id": USER, "session_id": "s1", "title": "Call mum", "notes": null,
        "due_at": AT, "recurrence": "weekly", "recurrence_weekday": 6, "recurrence_day_of_month": null,
        "status": "active", "created_by": "user", "last_fired_at": null, "created_at": AT
    })
}

#[tokio::test]
async fn reads_and_creates_reminders() {
    let h = Harness::new().await;
    h.answer(
        "GET",
        "/api/reminders",
        200,
        Some(json!({
            "timezone": "Asia/Jakarta", "upcoming": [reminder_json("r1")], "past": [],
            "scheduled_tasks": [{ "id": "j1", "label": "Weekly digest", "run_at": AT, "recurrence": null, "agent": "nomi", "session_id": "s2" }]
        })),
    )
    .await;
    Mock::given(method("POST"))
        .and(path("/api/reminders"))
        .and(body_json(
            json!({ "title": "Call mum", "due_at": AT, "recurrence": "weekly", "notes": null }),
        ))
        .respond_with(ResponseTemplate::new(201).set_body_json(reminder_json("r2")))
        .expect(1)
        .mount(&h.server)
        .await;

    let overview = h.sdk.reminders().await.unwrap();
    assert_eq!(overview.upcoming[0].title, "Call mum");
    assert_eq!(overview.scheduled_tasks[0].label, "Weekly digest");

    let created = h
        .sdk
        .create_reminder(NewReminder {
            title: "Call mum".into(),
            due_at: AT.into(),
            recurrence: Some("weekly".into()),
            notes: None,
        })
        .await
        .unwrap();
    assert_eq!(created.id, "r2");
}

#[tokio::test]
async fn acts_on_reminders_and_cancels_scheduled_work_where_the_backend_routes_them() {
    let h = Harness::new().await;
    h.expect_body(
        "POST",
        "/api/reminders/r1",
        json!({ "action": "done" }),
        204,
    )
    .await;
    h.expect_body(
        "POST",
        "/api/reminders/r2",
        json!({ "action": "snooze", "minutes": 30 }),
        204,
    )
    .await;
    h.answer("POST", "/api/scheduled-tasks/j1/cancel", 204, None)
        .await;

    h.sdk
        .act_on_reminder("r1".into(), ReminderAction::Done)
        .await
        .unwrap();
    h.sdk
        .act_on_reminder("r2".into(), ReminderAction::Snooze { minutes: 30 })
        .await
        .unwrap();
    h.sdk.cancel_scheduled_task("j1".into()).await.unwrap();
}

#[tokio::test]
async fn reads_edits_confirms_and_forgets_memories() {
    let h = Harness::new().await;
    h.answer(
        "GET",
        "/api/memory",
        200,
        Some(json!({ "memories": [{
            "id": "m1", "content": "Likes tea", "weight": 0.8, "kind": "preference",
            "created_at": AT, "updated_at": AT, "uses": 3, "last_used_at": null,
            "needs_check": true, "embedding": [0.25, -0.5]
        }] })),
    )
    .await;
    h.expect_body(
        "PUT",
        "/api/memory/m1",
        json!({ "content": "Likes green tea" }),
        204,
    )
    .await;
    h.answer("POST", "/api/memory/m1/confirm", 204, None).await;
    h.answer("DELETE", "/api/memory/m1", 204, None).await;

    let memories = h.sdk.memories().await.unwrap();
    assert!(memories[0].needs_check);
    assert_eq!(memories[0].embedding, vec![0.25, -0.5]);

    h.sdk
        .edit_memory("m1".into(), "Likes green tea".into(), None)
        .await
        .unwrap();
    h.sdk.confirm_memory("m1".into()).await.unwrap();
    h.sdk.delete_memory("m1".into()).await.unwrap();
}

#[tokio::test]
async fn reads_connections_usage_and_projects() {
    let h = Harness::new().await;
    h.answer(
        "GET",
        "/api/connections",
        200,
        Some(json!({
            "google": { "configured": true, "email": "me@nomi.test", "services": ["calendar"] },
            "hevy": null
        })),
    )
    .await;
    h.answer(
        "GET",
        "/api/usage/brief",
        200,
        Some(json!({ "plan": { "name": "Pro", "monthly_tokens": 1_000_000 }, "month": "2026-10", "tokens_used": 2_500, "can_upgrade": false })),
    )
    .await;
    Mock::given(method("GET"))
        .and(path("/api/projects"))
        .and(query_param("org_id", "o1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "org_id": "o1",
            "projects": [{
                "id": "p1", "session_id": "s1", "name": "Site", "description": null, "status": "live",
                "created_at": AT, "updated_at": AT, "created_by": "Ana", "open_issues": 2
            }]
        })))
        .expect(1)
        .mount(&h.server)
        .await;

    let connections = h.sdk.connections().await.unwrap();
    assert_eq!(connections.google.email.as_deref(), Some("me@nomi.test"));
    assert!(connections.hevy.is_none());
    assert_eq!(
        h.sdk.usage_brief().await.unwrap().plan.monthly_tokens,
        1_000_000
    );
    let projects = h.sdk.projects(Some("o1".into())).await.unwrap();
    assert_eq!(
        (projects[0].name.as_str(), projects[0].open_issues),
        ("Site", 2)
    );
}

#[tokio::test]
async fn account_pages_need_a_session() {
    let server = MockServer::start().await;
    let dir = tempfile::tempdir().unwrap();
    let config = default_sdk_config(server.uri(), dir.path().display().to_string());
    let sdk = NomiSdk::new(config, MemoryCredentials::default().boxed()).unwrap();

    assert_eq!(sdk.memories().await, Err(NomiError::NotSignedIn));
    assert!(server.received_requests().await.unwrap().is_empty());
}

#[tokio::test]
async fn fetches_an_attachments_preview_and_content_as_bytes() {
    const PREVIEW_BYTES: &[u8] = &[0x89, b'P', b'N', b'G'];
    const CONTENT_BYTES: &[u8] = b"report body";
    let h = Harness::new().await;
    for (url_path, bytes) in [
        ("/api/attachments/f1/preview", PREVIEW_BYTES),
        ("/api/attachments/f1/content", CONTENT_BYTES),
    ] {
        Mock::given(method("GET"))
            .and(path(url_path))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(bytes))
            .expect(1)
            .mount(&h.server)
            .await;
    }
    assert_eq!(
        h.sdk.attachment_preview("f1".into()).await.unwrap(),
        PREVIEW_BYTES
    );
    assert_eq!(
        h.sdk.attachment_content("f1".into()).await.unwrap(),
        CONTENT_BYTES
    );

    h.answer("GET", "/api/attachments/gone/preview", 404, None)
        .await;
    assert!(matches!(
        h.sdk.attachment_preview("gone".into()).await,
        Err(NomiError::Http { status: 404, .. })
    ));
}

#[tokio::test]
async fn reads_a_projects_workspace() {
    const PROJECT: &str = "7f3a2c10-proj";
    let h = Harness::new().await;
    h.answer(
        "GET",
        "/api/projects/by-session/c9",
        200,
        Some(json!({
            "id": PROJECT, "name": "Bakery site", "description": null, "plan": "…", "status": "ready",
            "stack": "sveltekit", "files_version": 12,
            "files": [{ "path": "src/routes/+page.svelte", "content_type": "text/plain", "size_bytes": 812 }],
            "session_id": "c9", "org_id": "o1", "org_name": "Trian's team", "open_issues": 2, "backend": "drizzle"
        })),
    )
    .await;
    h.answer(
        "GET",
        &format!("/api/projects/{PROJECT}/deploy"),
        200,
        Some(json!({
            "available": true, "role": "admin", "app": "bakery", "database": "none",
            "url": "https://bakery.example.app",
            "publishes": [{
                "id": "p1", "environment": "production", "status": "live", "url": "https://bakery.example.app",
                "error": null, "files_version": 12, "rollback_of": null, "can_roll_back": true,
                "created_by": "Trian", "created_at": AT, "finished_at": AT, "failure": null,
                "rollback_of_version": null, "started_by_agent": null, "current": true
            }],
            "live": null, "backend_setup": null
        })),
    )
    .await;
    h.answer(
        "GET",
        &format!("/api/projects/{PROJECT}/members"),
        200,
        Some(json!({
            "access": "org", "my_role": "admin", "me": "u1",
            "people": [{ "user_id": "u1", "name": "Trian", "email": null, "avatar_url": null,
                "org_role": "owner", "project_role": "admin", "is_owner": true, "added_at": AT }]
        })),
    )
    .await;
    let file = format!("/api/projects/{PROJECT}/files/src/routes/+page.svelte");
    Mock::given(method("GET"))
        .and(path(file.as_str()))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(b"<h1>Hi</h1>".as_slice()))
        .expect(1)
        .mount(&h.server)
        .await;

    let project = h.sdk.project_for_chat("c9".into()).await.unwrap();
    assert_eq!(project.name, "Bakery site");
    assert_eq!(project.files[0].size_bytes, 812);
    let deploy = h.sdk.project_deploy(PROJECT.into()).await.unwrap();
    assert_eq!(deploy.url.as_deref(), Some("https://bakery.example.app"));
    assert_eq!(deploy.publishes[0].status, "live");
    let members = h.sdk.project_members(PROJECT.into()).await.unwrap();
    assert!(members.people[0].is_owner);
    let content = h
        .sdk
        .project_file(PROJECT.into(), "src/routes/+page.svelte".into())
        .await
        .unwrap();
    assert_eq!(content, b"<h1>Hi</h1>");
}

#[tokio::test]
async fn an_ordinary_chat_has_no_project() {
    let h = Harness::new().await;
    h.answer("GET", "/api/projects/by-session/c1", 404, None)
        .await;
    assert!(matches!(
        h.sdk.project_for_chat("c1".into()).await,
        Err(NomiError::Http { status: 404, .. })
    ));
}

#[tokio::test]
async fn publishes_and_works_through_issues() {
    let h = Harness::new().await;
    let issue = json!({
        "number": 3, "title": "Menu overlaps", "body": "", "kind": "bug", "severity": "high",
        "status": "open", "created_by": "Trian", "mine": true, "closed_at": null,
        "created_at": AT, "updated_at": AT
    });
    Mock::given(method("POST"))
        .and(path("/api/projects/p/deploy/publish"))
        .respond_with(ResponseTemplate::new(202).set_body_json(json!({ "id": "x" })))
        .expect(1)
        .mount(&h.server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/projects/p/issues"))
        .and(query_param("status", "open"))
        .respond_with(ResponseTemplate::new(200).set_body_json(
            json!({ "issues": [issue.clone()], "counts": { "open": 1, "closed": 4 } }),
        ))
        .expect(1)
        .mount(&h.server)
        .await;
    Mock::given(method("POST"))
        .and(path("/api/projects/p/issues"))
        .and(body_json(
            json!({ "title": "Menu overlaps", "body": "", "kind": "bug", "severity": "high" }),
        ))
        .respond_with(ResponseTemplate::new(201).set_body_json(issue.clone()))
        .expect(1)
        .mount(&h.server)
        .await;
    let mut closed = issue.clone();
    closed["status"] = json!("closed");
    Mock::given(method("PATCH"))
        .and(path("/api/projects/p/issues/3"))
        .and(body_json(json!({ "status": "closed" })))
        .respond_with(ResponseTemplate::new(200).set_body_json(closed))
        .expect(1)
        .mount(&h.server)
        .await;

    h.sdk.publish_project("p".into()).await.unwrap();
    let list = h
        .sdk
        .project_issues("p".into(), "open".into())
        .await
        .unwrap();
    assert_eq!((list.counts.open, list.counts.closed), (1, 4));
    let filed = h
        .sdk
        .file_project_issue(
            "p".into(),
            crate::NewProjectIssue {
                title: "Menu overlaps".into(),
                body: String::new(),
                kind: "bug".into(),
                severity: "high".into(),
            },
        )
        .await
        .unwrap();
    assert_eq!(filed.number, 3);
    let after = h
        .sdk
        .set_project_issue_status("p".into(), 3, "closed".into())
        .await
        .unwrap();
    assert_eq!(after.status, "closed");
}

#[tokio::test]
async fn renames_production_or_says_why_not() {
    let h = Harness::new().await;
    h.answer(
        "GET",
        "/api/projects/p/domain/check",
        200,
        Some(
            json!({ "subdomain": "bakery", "hostname": null, "available": false, "reason": "taken",
            "suggestions": [{ "subdomain": "bakery-2", "hostname": "bakery-2.nomi.app" }] }),
        ),
    )
    .await;
    Mock::given(method("PUT"))
        .and(path("/api/projects/p/domain"))
        .and(body_json(json!({ "subdomain": "bakery" })))
        .respond_with(ResponseTemplate::new(409).set_body_json(
            json!({ "available": false, "reason": "taken", "message": "taken a moment ago", "suggestions": [] }),
        ))
        .up_to_n_times(1)
        .mount(&h.server)
        .await;
    Mock::given(method("PUT"))
        .and(path("/api/projects/p/domain"))
        .and(body_json(json!({ "subdomain": "bakery-2" })))
        .respond_with(ResponseTemplate::new(200).set_body_json(
            json!({ "hostname": "bakery-2.nomi.app", "url": "https://bakery-2.nomi.app" }),
        ))
        .expect(1)
        .mount(&h.server)
        .await;

    let check = h
        .sdk
        .check_project_domain("p".into(), "bakery".into())
        .await
        .unwrap();
    assert!(!check.available);
    assert_eq!(check.suggestions[0].subdomain, "bakery-2");
    match h
        .sdk
        .change_project_domain("p".into(), "bakery".into())
        .await
        .unwrap()
    {
        crate::DomainChange::Refused { check } => {
            assert_eq!(check.message.as_deref(), Some("taken a moment ago"))
        }
        other => panic!("expected a refusal, got {other:?}"),
    }
    assert_eq!(
        h.sdk
            .change_project_domain("p".into(), "bakery-2".into())
            .await
            .unwrap(),
        crate::DomainChange::Changed {
            hostname: "bakery-2.nomi.app".into()
        }
    );
}

#[tokio::test]
async fn reads_how_production_is_doing() {
    let h = Harness::new().await;
    h.answer(
        "GET",
        "/api/projects/p/deploy/usage",
        200,
        Some(json!({ "containers": [
            { "service": "web", "container_id": "c1", "limits": { "cpu": null, "memory_mb": 512 },
              "stats": { "cpu_percent": 3.5, "memory_usage_bytes": 104857600.0, "memory_limit_bytes": 536870912.0,
                "memory_percent": 19.5, "net_rx_bytes": 0, "net_tx_bytes": 0, "block_read_bytes": 0,
                "block_write_bytes": 0, "pids": 4, "timestamp": AT } },
            { "service": "worker", "container_id": "c2", "limits": { "cpu": null, "memory_mb": null }, "stats": null }
        ] })),
    )
    .await;
    let counters = json!({ "visitors": 40, "requests": 1200, "errors_5xx": 2, "errors_4xx": 9, "p50_ms": 31, "p95_ms": null });
    Mock::given(method("GET"))
        .and(path("/api/projects/p/deploy/metrics"))
        .and(query_param("range", "24h"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "range": "24h", "step": "1h",
            "buckets": [{ "at": AT, "visitors": 4, "requests": 50, "errors_5xx": 0, "errors_4xx": 1, "p50_ms": 30, "p95_ms": 90 }],
            "totals": counters, "previous_totals": counters
        })))
        .expect(1)
        .mount(&h.server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/projects/p/deploy/logs"))
        .and(query_param("tail", "200"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "lines": ["GET / 200"] })))
        .expect(1)
        .mount(&h.server)
        .await;

    let usage = h.sdk.project_usage("p".into()).await.unwrap();
    assert_eq!(usage[0].memory_usage_bytes, Some(104_857_600));
    assert_eq!(usage[1].cpu_percent, None);
    let traffic = h
        .sdk
        .project_traffic("p".into(), "24h".into())
        .await
        .unwrap();
    assert_eq!(traffic.totals.requests, Some(1200.0));
    assert_eq!(traffic.totals.p95_ms, None);
    assert_eq!(
        h.sdk.project_logs("p".into()).await.unwrap().lines,
        vec!["GET / 200"]
    );
}

#[tokio::test]
async fn queries_the_projects_data_as_text_cells() {
    let h = Harness::new().await;
    Mock::given(method("GET"))
        .and(path("/api/projects/p/database/tables"))
        .and(query_param("source", "production"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(
                json!([{ "schema": "public", "name": "orders", "rows_estimate": 42 }]),
            ),
        )
        .expect(1)
        .mount(&h.server)
        .await;
    Mock::given(method("POST"))
        .and(path("/api/projects/p/relay/query"))
        .and(body_json(json!({ "sql": "select * from orders", "source": "development" })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "columns": ["id", "total", "note", "paid"], "rows": [[1, 9.5, null, true], ["x", "y", "z", false]],
            "truncated": false, "rows_affected": null, "took_ms": 3.2
        })))
        .expect(1)
        .mount(&h.server)
        .await;

    let tables = h
        .sdk
        .project_tables("p".into(), crate::DataSource::Production)
        .await
        .unwrap();
    assert_eq!(tables[0].rows_estimate, 42);
    let result = h
        .sdk
        .query_project_data(
            "p".into(),
            "select * from orders".into(),
            crate::DataSource::Development,
            true,
        )
        .await
        .unwrap();
    assert_eq!(
        result.rows[0],
        vec![
            Some("1".into()),
            Some("9.5".into()),
            None,
            Some("true".into())
        ]
    );
    assert_eq!(result.rows[1][0].as_deref(), Some("x"));
}

#[tokio::test]
async fn changes_a_projects_settings() {
    let h = Harness::new().await;
    Mock::given(method("PUT"))
        .and(path("/api/projects/p/showcase"))
        .and(body_json(json!({ "showcase": true, "blurb": "Fresh bread daily" })))
        .respond_with(ResponseTemplate::new(200).set_body_json(
            json!({ "showcase": true, "blurb": "Fresh bread daily", "url": "https://bakery.nomi.app", "can_edit": true }),
        ))
        .expect(1)
        .mount(&h.server)
        .await;
    h.answer(
        "DELETE",
        "/api/projects/p/deploy/environments/production",
        204,
        None,
    )
    .await;
    Mock::given(method("GET"))
        .and(path("/api/projects/p/transfer/orgs"))
        .and(query_param("q", "acme"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            { "id": "o2", "name": "Acme", "is_personal": false, "owner_id": null, "owner_name": null, "my_role": "admin" }
        ])))
        .expect(1)
        .mount(&h.server)
        .await;
    h.expect_body(
        "POST",
        "/api/projects/p/transfer",
        json!({ "target_org_id": "o2", "confirm_name": "Bakery site" }),
        200,
    )
    .await;
    h.answer("DELETE", "/api/projects/p", 204, None).await;

    let showcase = h
        .sdk
        .set_project_showcase("p".into(), true, Some("Fresh bread daily".into()))
        .await
        .unwrap();
    assert!(showcase.showcase);
    h.sdk.take_project_offline("p".into()).await.unwrap();
    let orgs = h
        .sdk
        .transfer_orgs("p".into(), "acme".into())
        .await
        .unwrap();
    assert_eq!(orgs[0].name, "Acme");
    h.sdk
        .transfer_project("p".into(), "o2".into(), "Bakery site".into())
        .await
        .unwrap();
    h.sdk.delete_project("p".into()).await.unwrap();
}
