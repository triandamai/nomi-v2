//! Routes that had no tests of their own: the crew's project permissions, Admin → Payment (the
//! default provider, Xendit and DOKU), and
//! trying a failed run again.

mod support;

use axum::http::StatusCode;
use serde_json::{json, Value};
use sqlx::PgPool;
use support::App;
use uuid::Uuid;

#[sqlx::test(migrations = "../../migrations")]
async fn project_admins_set_what_the_crew_may_see_and_others_only_read_it(pool: PgPool) {
    let app = App::new(pool.clone());
    let owner = app.person("owner@acme.test").await;
    let stranger = app.person("stranger@acme.test").await;
    let (project, _) = app.project(&owner, "Bakery").await;
    let base = format!("/api/projects/{project}/agent-permissions");

    let (status, list) = app.get(&base, &owner).await;
    assert_eq!((status, list), (StatusCode::OK, json!([])));
    assert_eq!(app.get(&base, &stranger).await.0, StatusCode::NOT_FOUND);

    let allow = json!({"agent_type": "inspector", "permission": "read_logs", "scope": "allow"});
    assert_eq!(app.send("PUT", &base, &stranger, allow.clone()).await.0, StatusCode::NOT_FOUND);
    assert_eq!(app.send("PUT", &base, &owner, json!({"agent_type": "coding", "permission": "deploy", "scope": "allow"})).await.0, StatusCode::BAD_REQUEST);
    assert_eq!(app.send("PUT", &base, &owner, json!({"agent_type": "inspector", "permission": "read_logs", "scope": "maybe"})).await.0, StatusCode::BAD_REQUEST);
    assert_eq!(app.send("PUT", &base, &owner, allow).await.0, StatusCode::NO_CONTENT);

    let (_, list) = app.get(&base, &owner).await;
    let rows = list.as_array().unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["permission"], "read_logs");
    assert_eq!(rows[0]["scope"], "allow");
    assert_eq!(rows[0]["for_this_project"], true);

    let id = rows[0]["id"].as_str().unwrap();
    assert_eq!(app.call("DELETE", &format!("{base}/{id}"), Some(&owner.token), None).await.0, StatusCode::NO_CONTENT);
    assert_eq!(app.call("DELETE", &format!("{base}/{id}"), Some(&owner.token), None).await.0, StatusCode::NOT_FOUND);
}

#[sqlx::test(migrations = "../../migrations")]
async fn xendit_settings_are_admin_only_and_tested_from_what_is_saved(pool: PgPool) {
    let app = App::new(pool.clone());
    let person = app.person("ana@acme.test").await;
    let admin = app.admin("root@nomi.test").await;

    assert_eq!(app.get("/api/admin/payment/xendit", &person).await.0, StatusCode::FORBIDDEN);
    assert_eq!(app.call("POST", "/api/admin/payment/xendit/test", Some(&person.token), None).await.0, StatusCode::FORBIDDEN);

    let (status, view) = app.get("/api/admin/payment/xendit", &admin).await;
    assert_eq!(status, StatusCode::OK, "{view}");
    assert_eq!(view["provider"], "xendit");
    assert_eq!(view["can_manage"], true);
    let field_names: Vec<String> = view["fields"].as_array().unwrap().iter().map(|f| f["name"].as_str().unwrap().to_string()).collect();
    assert!(!field_names.is_empty());

    let (status, saved) = app.send("PUT", "/api/admin/payment/xendit", &admin, json!({"values": {}})).await;
    assert_eq!(status, StatusCode::OK, "{saved}");
    assert_eq!(app.send("PUT", "/api/admin/payment/xendit", &person, json!({"values": {}})).await.0, StatusCode::FORBIDDEN);

    let (status, tested) = app.call("POST", "/api/admin/payment/xendit/test", Some(&admin.token), None).await;
    assert_eq!(status, StatusCode::OK);
    assert!(tested["success"].is_boolean());
    assert!(!tested["message"].as_str().unwrap().is_empty());
}

/// Nothing listens here, so the DOKU key check fails without reaching DOKU.
const UNREACHABLE_DOKU: &str = "http://127.0.0.1:9";

#[sqlx::test(migrations = "../../migrations")]
async fn doku_settings_are_admin_only_keep_their_secret_and_are_checked_with_doku(pool: PgPool) {
    let app = App::new(pool.clone());
    let person = app.person("ana@acme.test").await;
    let admin = app.admin("root@nomi.test").await;

    assert_eq!(app.get("/api/admin/payment/doku", &person).await.0, StatusCode::FORBIDDEN);
    assert_eq!(app.send("PUT", "/api/admin/payment/doku", &person, json!({"values": {}})).await.0, StatusCode::FORBIDDEN);
    assert_eq!(app.call("POST", "/api/admin/payment/doku/test", Some(&person.token), None).await.0, StatusCode::FORBIDDEN);

    let (status, tested) = app.call("POST", "/api/admin/payment/doku/test", Some(&admin.token), None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(tested["success"], false, "{tested}");

    let values = json!({"values": {"client_id": "BRN-0000-TEST", "secret_key": "SK-test-only", "base_url": UNREACHABLE_DOKU}});
    let (status, saved) = app.send("PUT", "/api/admin/payment/doku", &admin, values).await;
    assert_eq!(status, StatusCode::OK, "{saved}");
    assert_eq!(saved["provider"], "doku");
    let secret = saved["fields"].as_array().unwrap().iter().find(|f| f["name"] == "secret_key").unwrap();
    assert_eq!(secret["set"], true);
    assert_eq!(secret["value"], Value::Null, "a secret is never sent back");
    assert!(!saved.to_string().contains("SK-test-only"));

    // An empty secret keeps the saved one.
    let (_, kept) = app.send("PUT", "/api/admin/payment/doku", &admin, json!({"values": {"secret_key": ""}})).await;
    assert_eq!(kept["fields"].as_array().unwrap().iter().find(|f| f["name"] == "secret_key").unwrap()["set"], true);

    let (status, tested) = app.call("POST", "/api/admin/payment/doku/test", Some(&admin.token), None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(tested["success"], false, "{tested}");
    assert!(tested["message"].as_str().unwrap().contains("DOKU"));
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_default_payment_provider_is_chosen_by_admins_and_shows_what_is_set_up(pool: PgPool) {
    let app = App::new(pool.clone());
    let person = app.person("ana@acme.test").await;
    let admin = app.admin("root@nomi.test").await;

    assert_eq!(app.get("/api/admin/payment", &person).await.0, StatusCode::FORBIDDEN);
    assert_eq!(app.send("PUT", "/api/admin/payment", &person, json!({"provider": "doku"})).await.0, StatusCode::FORBIDDEN);

    let (status, overview) = app.get("/api/admin/payment", &admin).await;
    assert_eq!(status, StatusCode::OK, "{overview}");
    let names: Vec<&str> = overview["providers"].as_array().unwrap().iter().map(|p| p["name"].as_str().unwrap()).collect();
    assert_eq!(names, ["xendit", "doku"]);
    assert_eq!(overview["can_manage"], true);

    let (status, chosen) = app.send("PUT", "/api/admin/payment", &admin, json!({"provider": "doku"})).await;
    assert_eq!(status, StatusCode::OK, "{chosen}");
    assert_eq!(chosen["chosen"], "doku");
    assert_eq!(chosen["source"], "saved");
    // Chosen but not set up: new payments go nowhere until DOKU has keys.
    assert_eq!(chosen["in_use"], Value::Null);

    app.send("PUT", "/api/admin/payment/doku", &admin, json!({"values": {"client_id": "BRN-0000-TEST", "secret_key": "SK-test-only"}})).await;
    let (_, overview) = app.get("/api/admin/payment", &admin).await;
    assert_eq!(overview["in_use"], "doku");
    assert_eq!(overview["providers"][1]["configured"], true);

    assert_eq!(app.send("PUT", "/api/admin/payment", &admin, json!({"provider": "paypal"})).await.0, StatusCode::UNPROCESSABLE_ENTITY);
    let (_, cleared) = app.send("PUT", "/api/admin/payment", &admin, json!({"provider": ""})).await;
    assert_eq!(cleared["chosen"], Value::Null);
}

struct Chat {
    session: Uuid,
}

async fn chat_with_a_sent_message(app: &App, who: &support::Person) -> Chat {
    let (_, created) = app.call("POST", "/api/sessions", Some(&who.token), None).await;
    let session: Uuid = created["session_id"].as_str().unwrap().parse().unwrap();
    let (status, _) = app.send("POST", &format!("/api/sessions/{session}/messages"), who, json!({"text": "How much did I spend?"})).await;
    assert_eq!(status, StatusCode::ACCEPTED);
    Chat { session }
}

async fn failure_message(pool: &PgPool, session: Uuid, retry: Value) -> Uuid {
    sqlx::query_scalar("INSERT INTO messages (session_id, content, content_blocks) VALUES ($1, 'It failed', $2) RETURNING id")
        .bind(session)
        .bind(json!([{"kind": "run_failed", "retry": retry}]))
        .fetch_one(pool)
        .await
        .unwrap()
}

async fn pending_turns(pool: &PgPool, session: Uuid) -> i64 {
    sqlx::query_scalar("SELECT count(*) FROM turn_jobs WHERE session_id = $1 AND status = 'pending'").bind(session).fetch_one(pool).await.unwrap()
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_failed_turn_runs_again_once_the_chat_is_free(pool: PgPool) {
    let app = App::new(pool.clone());
    let ana = app.person("ana@acme.test").await;
    let ben = app.person("ben@acme.test").await;
    let chat = chat_with_a_sent_message(&app, &ana).await;
    let turn: Uuid = sqlx::query_scalar("SELECT id FROM turn_jobs WHERE session_id = $1").bind(chat.session).fetch_one(&pool).await.unwrap();
    let failed = failure_message(&pool, chat.session, json!({"target": "turn", "turn_job_id": turn})).await;
    let retry = format!("/api/sessions/{}/messages/{failed}/retry", chat.session);

    // Someone else's chat, and a chat that's still busy, can't be retried.
    assert_ne!(app.call("POST", &retry, Some(&ben.token), None).await.0, StatusCode::OK);
    let (status, body) = app.call("POST", &retry, Some(&ana.token), None).await;
    assert_eq!((status, body), (StatusCode::CONFLICT, json!("the crew is already working in this chat")));

    sqlx::query("UPDATE turn_jobs SET status = 'failed' WHERE session_id = $1").bind(chat.session).execute(&pool).await.unwrap();
    let (status, body) = app.call("POST", &retry, Some(&ana.token), None).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["removed_message_id"], failed.to_string());
    assert_eq!(pending_turns(&pool, chat.session).await, 1);
    // The error message is gone.
    assert_eq!(app.call("POST", &retry, Some(&ana.token), None).await.0, StatusCode::NOT_FOUND);
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_last_message_or_a_failed_hand_off_can_be_tried_again(pool: PgPool) {
    let app = App::new(pool.clone());
    let ana = app.person("ana@acme.test").await;
    let chat = chat_with_a_sent_message(&app, &ana).await;
    sqlx::query("UPDATE turn_jobs SET status = 'done' WHERE session_id = $1").bind(chat.session).execute(&pool).await.unwrap();

    let failed = failure_message(&pool, chat.session, json!({"target": "last_message"})).await;
    let (status, body) = app.call("POST", &format!("/api/sessions/{}/messages/{failed}/retry", chat.session), Some(&ana.token), None).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(pending_turns(&pool, chat.session).await, 1);

    let delegation: Uuid = sqlx::query_scalar(
        "INSERT INTO agent_delegations (session_id, user_id, requesting_agent_type, target_agent_type, task, status) VALUES ($1, $2, 'supervisor', 'money', 'sum', 'failed') RETURNING id",
    )
    .bind(chat.session)
    .bind(ana.id)
    .fetch_one(&pool)
    .await
    .unwrap();
    let handed = failure_message(&pool, chat.session, json!({"target": "delegation", "delegation_id": delegation})).await;
    let uri = format!("/api/sessions/{}/messages/{handed}/retry", chat.session);
    assert_eq!(app.call("POST", &uri, Some(&ana.token), None).await.0, StatusCode::OK);
    let pending: i64 = sqlx::query_scalar("SELECT count(*) FROM agent_delegations WHERE session_id = $1 AND status = 'pending'").bind(chat.session).fetch_one(&pool).await.unwrap();
    assert_eq!(pending, 1);
    // Going already: a second retry of the same failure is refused.
    let again = failure_message(&pool, chat.session, json!({"target": "delegation", "delegation_id": delegation})).await;
    let (status, body) = app.call("POST", &format!("/api/sessions/{}/messages/{again}/retry", chat.session), Some(&ana.token), None).await;
    assert_eq!((status, body), (StatusCode::CONFLICT, json!("that run is already going again")));

    // A message that isn't a failure can't be retried.
    let plain: Uuid = sqlx::query_scalar("INSERT INTO messages (session_id, content) VALUES ($1, 'hi') RETURNING id").bind(chat.session).fetch_one(&pool).await.unwrap();
    assert_eq!(app.call("POST", &format!("/api/sessions/{}/messages/{plain}/retry", chat.session), Some(&ana.token), None).await.0, StatusCode::CONFLICT);
}
