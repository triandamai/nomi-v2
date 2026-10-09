
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
        email_codes: nomi_server::sign_in_codes::EmailCodes::Off,
    }
}

async fn json_request(
    router: axum::Router,
    method: &str,
    uri: &str,
    body: Value,
    bearer: Option<&str>,
) -> (StatusCode, Value) {
    let mut builder = Request::builder().method(method).uri(uri).header("content-type", "application/json");
    if let Some(token) = bearer {
        builder = builder.header("authorization", format!("Bearer {token}"));
    }
    let response = router.oneshot(builder.body(Body::from(body.to_string())).unwrap()).await.unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let json_body = if bytes.is_empty() { Value::Null } else { serde_json::from_slice(&bytes).unwrap_or(Value::Null) };
    (status, json_body)
}

async fn register_and_login(router: axum::Router, email: &str) -> String {
    json_request(
        router.clone(),
        "POST",
        "/api/auth/register",
        json!({ "email": email, "password": "correct-password", "org": { "mode": "create", "name": "Acme" } }),
        None,
    )
    .await;
    let (_, login_body) =
        json_request(router, "POST", "/api/auth/login", json!({ "email": email, "password": "correct-password" }), None).await;
    login_body["access_token"].as_str().unwrap().to_string()
}


async fn user_id(pool: &PgPool, email: &str) -> uuid::Uuid {
    sqlx::query_scalar("SELECT user_id FROM web_credentials WHERE email = $1").bind(email).fetch_one(pool).await.unwrap()
}

async fn admin(router: axum::Router, pool: &PgPool) -> String {
    register_and_login(router.clone(), "boss@example.com").await;
    sqlx::query("UPDATE users SET is_platform_admin = true WHERE id = (SELECT user_id FROM web_credentials WHERE email = 'boss@example.com')")
        .execute(pool)
        .await
        .unwrap();
    // A fresh token carries the new permissions.
    let (_, login) = json_request(router, "POST", "/api/auth/login", json!({ "email": "boss@example.com", "password": "correct-password" }), None).await;
    login["access_token"].as_str().unwrap().to_string()
}

async fn plan_id(router: axum::Router, token: &str, slug: &str) -> String {
    let (_, plans) = json_request(router, "GET", "/api/admin/plans", Value::Null, Some(token)).await;
    plans.as_array().unwrap().iter().find(|p| p["slug"] == slug).unwrap()["id"].as_str().unwrap().to_string()
}

#[sqlx::test(migrations = "../../migrations")]
async fn an_admin_moves_someone_to_pro_and_they_are_told(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let boss = admin(router.clone(), &pool).await;
    let ana = register_and_login(router.clone(), "ana@example.com").await;
    let ana_id = user_id(&pool, "ana@example.com").await;

    let (_, brief) = json_request(router.clone(), "GET", "/api/usage/brief", Value::Null, Some(&ana)).await;
    assert_eq!(brief["plan"]["id"], "free");
    assert_eq!(brief["plan"]["monthly_tokens"], 1_000_000);

    // Only admins see or change plans.
    let (status, _) = json_request(router.clone(), "GET", "/api/admin/plans", Value::Null, Some(&ana)).await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    let pro = plan_id(router.clone(), &boss, "pro").await;
    let (status, sub) = json_request(
        router.clone(),
        "PUT",
        &format!("/api/admin/users/{ana_id}/subscription"),
        json!({ "plan_id": pro, "note": "Welcome aboard" }),
        Some(&boss),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{sub}");
    assert_eq!(sub["plan"]["slug"], "pro");
    assert_eq!(sub["monthly_tokens"], 10_000_000);
    assert_eq!(sub["history"].as_array().unwrap().len(), 1);

    let (_, brief) = json_request(router.clone(), "GET", "/api/usage/brief", Value::Null, Some(&ana)).await;
    assert_eq!(brief["plan"]["id"], "pro");

    let (_, inbox) = json_request(router.clone(), "GET", "/api/notifications", Value::Null, Some(&ana)).await;
    assert_eq!(inbox["unread"], 1);
    let first = &inbox["items"][0];
    assert_eq!(first["kind"], "subscription");
    assert_eq!(first["title"], "You're on Pro now");
    assert!(first["body"].as_str().unwrap().contains("Welcome aboard"));

    // An override of the allowance on the same plan is its own notice.
    let (status, sub) = json_request(
        router.clone(),
        "PUT",
        &format!("/api/admin/users/{ana_id}/subscription"),
        json!({ "plan_id": pro, "quota_override": 25_000_000, "override_until": "2099-01-01T00:00:00Z" }),
        Some(&boss),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(sub["monthly_tokens"], 25_000_000);
    let (_, inbox) = json_request(router.clone(), "GET", "/api/notifications", Value::Null, Some(&ana)).await;
    assert_eq!(inbox["unread"], 2);
    assert_eq!(inbox["items"][0]["title"], "Your monthly allowance is now 25,000,000 tokens");

    let (status, _) = json_request(router.clone(), "POST", "/api/notifications/read-all", Value::Null, Some(&ana)).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (_, unread) = json_request(router.clone(), "GET", "/api/notifications/unread", Value::Null, Some(&ana)).await;
    assert_eq!(unread["unread"], 0);

    // The admin list shows the plan and allowance.
    let (_, users) = json_request(router, "GET", "/api/admin/users?query=ana", Value::Null, Some(&boss)).await;
    assert_eq!(users["users"][0]["plan_name"], "Pro");
    assert_eq!(users["users"][0]["custom_quota"], true);
}

#[sqlx::test(migrations = "../../migrations")]
async fn admins_make_plans_with_a_promo_and_people_see_them(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let boss = admin(router.clone(), &pool).await;
    let bo = register_and_login(router.clone(), "bo@example.com").await;

    let (status, team) = json_request(
        router.clone(),
        "POST",
        "/api/admin/plans",
        json!({ "slug": "team", "name": "Team", "monthly_tokens": 30_000_000, "price_label": "Rp 99.000 / month",
                "features": ["30M tokens", ""], "card_tone": "ember", "promo_label": "Launch week", "promo_price_label": "Rp 49.000",
                "promo_ends_at": "2099-01-01T00:00:00Z", "sort_order": 2 }),
        Some(&boss),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{team}");
    assert_eq!(team["features"], json!(["30M tokens"]));

    let (status, _) = json_request(router.clone(), "POST", "/api/admin/plans", json!({ "slug": "team", "name": "Again", "monthly_tokens": 1 }), Some(&boss)).await;
    assert_eq!(status, StatusCode::CONFLICT);

    let (_, offered) = json_request(router.clone(), "GET", "/api/plans", Value::Null, Some(&bo)).await;
    let names: Vec<&str> = offered["plans"].as_array().unwrap().iter().map(|p| p["name"].as_str().unwrap()).collect();
    assert_eq!(names, vec!["Free", "Pro", "Team"]);
    assert_eq!(offered["plans"][2]["promo_label"], "Launch week");

    // Hidden plans aren't offered; a plan in use can't be deleted, an unused one can.
    let team_id = team["id"].as_str().unwrap();
    let (status, _) = json_request(router.clone(), "PUT", &format!("/api/admin/plans/{team_id}"),
        json!({ "slug": "team", "name": "Team", "monthly_tokens": 30_000_000, "is_active": false }), Some(&boss)).await;
    assert_eq!(status, StatusCode::OK);
    let (_, offered) = json_request(router.clone(), "GET", "/api/plans", Value::Null, Some(&bo)).await;
    assert_eq!(offered["plans"].as_array().unwrap().len(), 2);
    let free = plan_id(router.clone(), &boss, "free").await;
    let (status, _) = json_request(router.clone(), "DELETE", &format!("/api/admin/plans/{free}"), Value::Null, Some(&boss)).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let (status, _) = json_request(router, "DELETE", &format!("/api/admin/plans/{team_id}"), Value::Null, Some(&boss)).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_promo_reaches_everyone_and_email_preferences_stick(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let boss = admin(router.clone(), &pool).await;
    let cy = register_and_login(router.clone(), "cy@example.com").await;

    let (status, _) = json_request(router.clone(), "POST", "/api/admin/notifications/broadcast",
        json!({ "title": "Pro is half price", "body": "This week only.", "link": "https://evil.example" }), Some(&boss)).await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "links stay inside the app");
    let (status, sent) = json_request(router.clone(), "POST", "/api/admin/notifications/broadcast",
        json!({ "title": "Pro is half price", "body": "This week only.", "link": "/billing" }), Some(&boss)).await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(sent["recipients"], 2);

    let (_, inbox) = json_request(router.clone(), "GET", "/api/notifications", Value::Null, Some(&cy)).await;
    assert_eq!(inbox["items"][0]["kind"], "promo");
    let id = inbox["items"][0]["id"].as_str().unwrap();
    json_request(router.clone(), "POST", &format!("/api/notifications/{id}/read"), Value::Null, Some(&cy)).await;
    let (_, broadcasts) = json_request(router.clone(), "GET", "/api/admin/notifications/broadcasts", Value::Null, Some(&boss)).await;
    assert_eq!(broadcasts[0]["read"], 1);

    let (status, _) = json_request(router.clone(), "PUT", "/api/notifications/preferences", json!({ "email_account": true, "email_promos": false }), Some(&cy)).await;
    assert_eq!(status, StatusCode::OK);
    let (_, prefs) = json_request(router, "GET", "/api/notifications/preferences", Value::Null, Some(&cy)).await;
    assert_eq!(prefs, json!({ "email_account": true, "email_promos": false }));
}

#[sqlx::test(migrations = "../../migrations")]
async fn past_the_allowance_nomi_uses_their_own_key_or_stops(pool: PgPool) {
    use nomi_llm::{ContentBlock, LlmMessage, LlmRequest, LlmRole};
    let router = build_router(test_state(pool.clone()));
    let boss = admin(router.clone(), &pool).await;
    register_and_login(router.clone(), "dee@example.com").await;
    let dee = user_id(&pool, "dee@example.com").await;
    let free = plan_id(router.clone(), &boss, "free").await;

    // An allowance of zero is used up before anything is said.
    json_request(router.clone(), "PUT", &format!("/api/admin/users/{dee}/subscription"), json!({ "plan_id": free, "quota_override": 0 }), Some(&boss)).await;
    sqlx::query("UPDATE notifications SET read_at = now() WHERE user_id = $1").bind(dee).execute(&pool).await.unwrap();
    std::env::set_var("LLM_PROVIDER", "fake");

    let request = || LlmRequest {
        system: None,
        messages: vec![LlmMessage { role: LlmRole::User, content: vec![ContentBlock::Text { text: "hi".into() }] }],
        tools: vec![],
        max_tokens: 10,
        enable_reasoning: false,
        reasoning_effort: Default::default(),
    };
    let key = nomi_test_support::TEST_SETTINGS_KEY;
    let provider = nomi_server::bootstrap::build_llm_provider_for_user(&pool, dee, &key, reqwest::Client::new()).await;
    let blocked = nomi_llm::complete(provider.as_ref(), request()).await;
    assert!(matches!(blocked, Err(nomi_llm::LlmError::QuotaExceeded)));
    let titles: Vec<String> = sqlx::query_scalar("SELECT title FROM notifications WHERE user_id = $1 AND kind = 'quota'").bind(dee).fetch_all(&pool).await.unwrap();
    assert_eq!(titles, vec!["This month's allowance is used up".to_string()]);

    // With a key of their own saved (kept while they use Nomi's model), Nomi switches to it.
    let nomis_model: uuid::Uuid = sqlx::query_scalar(
        "INSERT INTO admin_llm_models (label, provider, model_id, api_key_encrypted, is_default, updated_by) \
         VALUES ('Nomi model', 'fake', 'fake', $1, true, $2) RETURNING id",
    )
    .bind(nomi_settings::crypto::encrypt(&key, "nomi-key"))
    .bind(dee)
    .fetch_one(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO user_llm_selections (user_id, admin_model_id, custom_label, custom_provider, custom_model_id, custom_api_key_encrypted) \
         VALUES ($1, $2, 'My key', 'fake', 'fake-model', $3)",
    )
    .bind(dee)
    .bind(nomis_model)
    .bind(nomi_settings::crypto::encrypt(&key, "sk-mine"))
    .execute(&pool)
    .await
    .unwrap();
    let provider = nomi_server::bootstrap::build_llm_provider_for_user(&pool, dee, &key, reqwest::Client::new()).await;
    assert!(nomi_llm::complete(provider.as_ref(), request()).await.is_ok());
    let switched: i64 = sqlx::query_scalar("SELECT count(*) FROM notifications WHERE user_id = $1 AND title = 'Nomi switched to your own API key'")
        .bind(dee)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(switched, 1);
}
