use axum::{body::Body, http::{Request, StatusCode}};
use http_body_util::BodyExt;
use nomi_server::app::{build_router, AppState};
use serde_json::{json, Value};
use sqlx::PgPool;
use tower::ServiceExt;

const SECRET: &str = "test-secret-do-not-use-in-prod";

fn test_state(pool: PgPool) -> AppState {
    AppState {
        pool,
        jwt_secret: SECRET.to_string(),
        http_client: reqwest::Client::new(),
        settings_key: nomi_test_support::TEST_SETTINGS_KEY,
        mqtt_broker_host: nomi_test_support::TEST_MQTT_BROKER_HOST.to_string(),
        mqtt_broker_port: nomi_test_support::TEST_MQTT_BROKER_PORT,
        s3: None,
        project_storage: nomi_test_support::test_project_storage(),
        tool_catalog: std::sync::Arc::new(nomi_agent_core::ToolCatalog::empty()),
    }
}

async fn register_and_login(router: axum::Router, pool: &PgPool, email: &str) -> (String, uuid::Uuid) {
    json_request_post(
        router.clone(),
        "/api/auth/register",
        json!({ "email": email, "password": "correct-password", "org": { "mode": "create", "name": "Acme" } }),
    )
    .await;
    let (_, login_body) = json_request_post(router, "/api/auth/login", json!({ "email": email, "password": "correct-password" })).await;
    let token = login_body["access_token"].as_str().unwrap().to_string();
    let user_id: uuid::Uuid = sqlx::query_scalar("SELECT user_id FROM web_credentials WHERE email = $1")
        .bind(email)
        .fetch_one(pool)
        .await
        .unwrap();
    (token, user_id)
}

async fn json_request_post(router: axum::Router, uri: &str, body: Value) -> (StatusCode, Value) {
    let response = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(uri)
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let json_body = if bytes.is_empty() { Value::Null } else { serde_json::from_slice(&bytes).unwrap_or(Value::Null) };
    (status, json_body)
}


async fn authed(router: axum::Router, method: &str, uri: &str, token: &str, body: Option<Value>) -> (StatusCode, Value) {
    let mut builder = Request::builder().method(method).uri(uri).header("authorization", format!("Bearer {token}"));
    let body = match body {
        Some(b) => {
            builder = builder.header("content-type", "application/json");
            Body::from(b.to_string())
        }
        None => Body::empty(),
    };
    let response = router.oneshot(builder.body(body).unwrap()).await.unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    (status, if bytes.is_empty() { Value::Null } else { serde_json::from_slice(&bytes).unwrap_or(Value::Null) })
}

async fn org_session(pool: &PgPool, user_id: uuid::Uuid, title: &str) -> uuid::Uuid {
    sqlx::query_scalar(
        "INSERT INTO sessions (org_id, user_id, channel, chat_id, title) \
         VALUES ((SELECT org_id FROM memberships WHERE user_id = $1 LIMIT 1), $1, 'web', gen_random_uuid()::text, $2) RETURNING id",
    )
    .bind(user_id)
    .bind(title)
    .fetch_one(pool)
    .await
    .unwrap()
}

#[sqlx::test(migrations = "../../migrations")]
async fn home_reports_what_happened_whats_due_today_and_open_plans(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let (token, user_id) = register_and_login(router.clone(), &pool, "home@example.com").await;
    let chat = org_session(&pool, user_id, "Bali, 6 days").await;

    sqlx::query(
        "INSERT INTO agent_delegations (session_id, user_id, requesting_agent_type, target_agent_type, task, status, result, completed_at) \
         VALUES ($1, $2, 'chitchat', 'money', 'Find idle subscriptions', 'completed', '3 unused since July', now() - interval '1 hour')",
    )
    .bind(chat)
    .bind(user_id)
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO scheduled_jobs (session_id, user_id, created_by_agent_type, target_agent_type, label, prompt, run_at) \
         VALUES ($1, $2, 'planning', 'planning', 'Call Mum back', 'remind', now() + interval '1 minute')",
    )
    .bind(chat)
    .bind(user_id)
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO messages (session_id, content, content_blocks, agent_display_name) VALUES ($1, 'To-do', \
         '[{\"kind\":\"todo_list\",\"items\":[{\"id\":\"1\",\"text\":\"a\",\"status\":\"done\"},{\"id\":\"2\",\"text\":\"b\",\"status\":\"pending\"}]}]', 'Planning')",
    )
    .bind(chat)
    .execute(&pool)
    .await
    .unwrap();

    let (status, home) = authed(router.clone(), "GET", "/api/home", &token, None).await;
    assert_eq!(status, StatusCode::OK);
    let out = home["while_you_were_out"].as_array().unwrap();
    assert!(out.iter().any(|i| i["kind"] == "finished" && i["title"] == "Money finished Find idle subscriptions"), "{out:?}");
    // The reminder is due later today, unless the test runs in the last minute of the UTC day.
    let today = home["today"].as_array().unwrap();
    assert!(today.is_empty() || today[0]["label"] == "Call Mum back");
    let plans = home["plans"].as_array().unwrap();
    assert_eq!(plans[0]["title"], "Bali, 6 days");
    assert_eq!((plans[0]["done"].as_i64(), plans[0]["total"].as_i64()), (Some(1), Some(2)));

    // A second look within the same visit reports the same things.
    let (_, again) = authed(router, "GET", "/api/home", &token, None).await;
    assert_eq!(again["since"], home["since"]);
}

#[sqlx::test(migrations = "../../migrations")]
async fn home_shows_four_per_card_and_each_section_pages_through_the_rest(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let (token, user_id) = register_and_login(router.clone(), &pool, "pages@example.com").await;
    let chat = org_session(&pool, user_id, "Errands").await;
    for i in 0..7 {
        sqlx::query(
            "INSERT INTO agent_delegations (session_id, user_id, requesting_agent_type, target_agent_type, task, status, result, completed_at) \
             VALUES ($1, $2, 'chitchat', 'money', $3, 'completed', 'done', now() - ($4 || ' minutes')::interval)",
        )
        .bind(chat)
        .bind(user_id)
        .bind(format!("Task {i}"))
        .bind((i + 1).to_string())
        .execute(&pool)
        .await
        .unwrap();
    }

    let (_, home) = authed(router.clone(), "GET", "/api/home", &token, None).await;
    assert_eq!(home["while_you_were_out"].as_array().unwrap().len(), 4);
    assert_eq!(home["while_you_were_out_total"], 7);

    let (status, first) = authed(router.clone(), "GET", "/api/home/updates?page=1&per_page=5", &token, None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!((first["total"].as_u64(), first["page"].as_u64(), first["per_page"].as_u64()), (Some(7), Some(1), Some(5)));
    assert_eq!(first["items"].as_array().unwrap().len(), 5);
    assert_eq!(first["items"][0]["title"], "Money finished Task 0");
    let (_, second) = authed(router.clone(), "GET", "/api/home/updates?page=2&per_page=5", &token, None).await;
    assert_eq!(second["items"].as_array().unwrap().len(), 2);
    // Reading a section doesn't count as a new visit: Home's window stays put.
    assert_eq!(second["since"], home["since"]);

    for path in ["/api/home/today", "/api/home/plans"] {
        let (status, page) = authed(router.clone(), "GET", path, &token, None).await;
        assert_eq!(status, StatusCode::OK, "{path}");
        assert_eq!(page["total"], 0, "{path}");
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn money_totals_a_month_against_the_one_before(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let (token, user_id) = register_and_login(router.clone(), &pool, "money@example.com").await;
    for (when, cents, category) in [("2026-03-03", 1200, "food"), ("2026-03-09", 800, "food"), ("2026-03-10", 5000, "travel"), ("2026-02-20", 1000, "food")] {
        sqlx::query("INSERT INTO mock_transactions (user_id, occurred_at, amount_cents, category, description) VALUES ($1, $2::date + time '12:00', $3, $4, 'x')")
            .bind(user_id)
            .bind(when)
            .bind(cents as i64)
            .bind(category)
            .execute(&pool)
            .await
            .unwrap();
    }

    let (status, money) = authed(router, "GET", "/api/money?month=2026-03", &token, None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(money["month"], "2026-03");
    assert_eq!(money["total_cents"], 7000);
    assert_eq!(money["previous_total_cents"], 1000);
    assert_eq!(money["by_category"][0]["category"], "travel");
    assert_eq!(money["by_day"].as_array().unwrap().len(), 3);
    assert_eq!(money["transactions"].as_array().unwrap().len(), 3);
    assert_eq!(money["months"], json!(["2026-03", "2026-02"]));
}

#[sqlx::test(migrations = "../../migrations")]
async fn reminders_live_in_the_reminders_agents_table_with_done_snooze_and_cancel(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let (token, user_id) = register_and_login(router.clone(), &pool, "remind@example.com").await;
    let due_at = (chrono::Utc::now() + chrono::Duration::hours(2)).to_rfc3339();

    let (status, created) =
        authed(router.clone(), "POST", "/api/reminders", &token, Some(json!({"title": "Stretch", "due_at": due_at, "recurrence": "daily"}))).await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!((created["recurrence"].as_str(), created["created_by"].as_str()), (Some("daily"), Some("user")));
    let (_, second) = authed(router.clone(), "POST", "/api/reminders", &token, Some(json!({"title": "Water", "due_at": due_at}))).await;
    assert_eq!(second["session_id"], created["session_id"], "both share one Reminders chat");

    let (status, _) = authed(router.clone(), "POST", "/api/reminders", &token, Some(json!({"title": "Past", "due_at": "2020-01-01T00:00:00Z"}))).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    let first = created["id"].as_str().unwrap();
    let second_id = second["id"].as_str().unwrap();
    let (status, _) = authed(router.clone(), "POST", &format!("/api/reminders/{first}"), &token, Some(json!({"action": "done"}))).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (status, _) =
        authed(router.clone(), "POST", &format!("/api/reminders/{second_id}"), &token, Some(json!({"action": "snooze", "minutes": 30}))).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // A task an agent scheduled stays in the core scheduler, listed separately.
    let session: uuid::Uuid = created["session_id"].as_str().unwrap().parse().unwrap();
    let task: uuid::Uuid = sqlx::query_scalar(
        "INSERT INTO scheduled_jobs (session_id, user_id, created_by_agent_type, target_agent_type, label, prompt, run_at) \
         VALUES ($1, $2, 'chitchat', 'money', 'Summarize spending', 'summarize', now() + interval '1 day') RETURNING id",
    )
    .bind(session)
    .bind(user_id)
    .fetch_one(&pool)
    .await
    .unwrap();

    let (_, list) = authed(router.clone(), "GET", "/api/reminders", &token, None).await;
    assert_eq!(list["upcoming"].as_array().unwrap().len(), 1);
    assert_eq!(list["upcoming"][0]["title"], "Water");
    assert_eq!(list["past"][0]["status"], "done");
    assert_eq!(list["scheduled_tasks"][0]["label"], "Summarize spending");

    let (status, _) = authed(router.clone(), "POST", &format!("/api/scheduled-tasks/{task}/cancel"), &token, None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (other, _) = register_and_login(router.clone(), &pool, "other@example.com").await;
    let (status, _) = authed(router, "POST", &format!("/api/reminders/{second_id}"), &other, Some(json!({"action": "cancel"}))).await;
    assert_eq!(status, StatusCode::NOT_FOUND, "another user can't touch it");
}

#[sqlx::test(migrations = "../../migrations")]
async fn money_budgets_and_manual_transactions_live_in_moneys_own_tables(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let (token, user_id) = register_and_login(router.clone(), &pool, "budget@example.com").await;
    let month = chrono::Utc::now().format("%Y-%m").to_string();

    let (status, _) =
        authed(router.clone(), "PUT", "/api/money/budgets", &token, Some(json!({"category": "Food", "monthly_limit": 300}))).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (status, _) = authed(
        router.clone(),
        "POST",
        "/api/money/transactions",
        &token,
        Some(json!({"amount": 45.5, "category": "food", "description": "Lunch with Maya"})),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let (status, _) =
        authed(router.clone(), "POST", "/api/money/transactions", &token, Some(json!({"amount": -3, "category": "food", "description": "x"}))).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    let (_, money) = authed(router.clone(), "GET", &format!("/api/money?month={month}"), &token, None).await;
    assert_eq!(money["budgets"], json!([{"category": "food", "limit_cents": 30000, "spent_cents": 4550}]));
    assert_eq!(money["transactions"][0]["description"], "Lunch with Maya");

    // Stored in the Money agent's table, marked as entered by hand; the old name still reads it.
    let source: String = sqlx::query_scalar("SELECT source FROM money_transactions WHERE user_id = $1").bind(user_id).fetch_one(&pool).await.unwrap();
    assert_eq!(source, "manual");
    let via_old_name: i64 = sqlx::query_scalar("SELECT count(*) FROM mock_transactions WHERE user_id = $1").bind(user_id).fetch_one(&pool).await.unwrap();
    assert_eq!(via_old_name, 1);

    let (status, _) = authed(router.clone(), "DELETE", "/api/money/budgets/food", &token, None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (status, _) = authed(router, "DELETE", "/api/money/budgets/food", &token, None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[sqlx::test(migrations = "../../migrations")]
async fn connections_show_each_users_own_google_account_only(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let (ana_token, ana) = register_and_login(router.clone(), &pool, "ana@example.com").await;
    let (budi_token, _) = register_and_login(router.clone(), &pool, "budi@example.com").await;
    sqlx::query(
        "INSERT INTO workspace_connections (user_id, google_email, services, access_token_encrypted, expires_at) \
         VALUES ($1, 'ana@gmail.example', ARRAY['gmail','sheets'], '\\x00', now() + interval '1 hour')",
    )
    .bind(ana)
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query("INSERT INTO workspace_activity (user_id, service, summary) VALUES ($1, 'sheets', 'Added 2 rows to Stays')").bind(ana).execute(&pool).await.unwrap();

    let (status, body) = authed(router.clone(), "GET", "/api/connections/google", &ana_token, None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["connection"]["email"], "ana@gmail.example");
    assert_eq!(body["connection"]["services"], json!(["gmail", "sheets"]));
    assert_eq!(body["activity"][0]["summary"], "Added 2 rows to Stays");
    assert_eq!(body["services"], json!(["gmail", "sheets", "docs", "drive", "calendar"]));

    let (_, body) = authed(router.clone(), "GET", "/api/connections/google", &budi_token, None).await;
    assert_eq!(body["connection"], Value::Null);
    assert_eq!(body["activity"], json!([]));

    let (status, _) = authed(router.clone(), "DELETE", "/api/connections/google", &ana_token, None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (_, body) = authed(router, "GET", "/api/connections/google", &ana_token, None).await;
    assert_eq!(body["connection"], Value::Null);
}

#[sqlx::test(migrations = "../../migrations")]
async fn home_lists_open_and_checklist_free_plans_but_not_finished_ones(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let (token, user_id) = register_and_login(router.clone(), &pool, "plans-home@example.com").await;
    let chat = org_session(&pool, user_id, "Trips").await;
    for (title, content, minutes_ago) in [
        ("Bali", "- [x] Dates\n- [ ] Flights", 3),
        ("Bandung weekend", "Coffee in Dago, a walk at Tahura.", 2),
        ("Done trip", "- [x] Dates\n- [x] Flights", 1),
    ] {
        sqlx::query(
            "INSERT INTO agent_plans (session_id, agent_session_id, user_id, title, content, version, created_at) \
             VALUES ($1, gen_random_uuid(), $2, $3, $4, 1, now() - make_interval(mins => $5))",
        )
        .bind(chat)
        .bind(user_id)
        .bind(title)
        .bind(content)
        .bind(minutes_ago)
        .execute(&pool)
        .await
        .unwrap();
    }

    let (status, home) = authed(router, "GET", "/api/home", &token, None).await;
    assert_eq!(status, StatusCode::OK);
    let plans: Vec<(String, i64, i64)> = home["plans"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| (p["title"].as_str().unwrap().to_string(), p["done"].as_i64().unwrap(), p["total"].as_i64().unwrap()))
        .collect();
    assert_eq!(plans, vec![("Bandung weekend".to_string(), 0, 0), ("Bali".to_string(), 1, 2)]);
}
