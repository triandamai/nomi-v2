//! Moving to a plan: its waitlist, paying for it through Xendit (a fake one here), the gateway's
//! callback that moves the person onto it, and what Billing then shows was paid.

mod support;

use std::collections::BTreeMap;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::json;
use sqlx::PgPool;
use support::App;
use uuid::Uuid;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// The payment settings are read through the process-wide settings cache and pool, so the tests
/// take turns.
static GLOBALS: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

const WEBHOOK_TOKEN: &str = "callback-token-123";
const PRICE_IDR: i64 = 49_000;

/// What Admin → Payment has saved: nothing, or Xendit at `base_url`.
async fn payments(pool: &PgPool, admin: Uuid, base_url: Option<&str>) {
    std::env::set_var(
        "SETTINGS_ENCRYPTION_KEY",
        hex::encode(nomi_test_support::TEST_SETTINGS_KEY),
    );
    nomi_settings::config::install(pool.clone());
    if let Some(base_url) = base_url {
        let values: BTreeMap<String, String> = [
            ("secret_key", "xnd_development_abc"),
            ("webhook_token", WEBHOOK_TOKEN),
            ("base_url", base_url),
            ("usd_to_idr", "15500"),
        ]
        .into_iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
        nomi_settings::config::save(
            pool,
            &nomi_test_support::TEST_SETTINGS_KEY,
            &nomi_settings::config::XENDIT,
            &values,
            admin,
        )
        .await
        .unwrap();
    }
    nomi_settings::config::forget();
}

async fn plan_id(pool: &PgPool, slug: &str) -> Uuid {
    sqlx::query_scalar("SELECT id FROM plans WHERE slug = $1")
        .bind(slug)
        .fetch_one(pool)
        .await
        .unwrap()
}

async fn callback(app: &App, token: Option<&str>, body: serde_json::Value) -> StatusCode {
    let mut request = Request::builder()
        .method("POST")
        .uri("/api/payments/xendit/webhook")
        .header("content-type", "application/json");
    if let Some(token) = token {
        request = request.header("x-callback-token", token);
    }
    app.raw(request.body(Body::from(body.to_string())).unwrap())
        .await
        .0
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_plan_not_sold_yet_has_a_waitlist(pool: PgPool) {
    let _g = GLOBALS.lock().await;
    let app = App::new(pool.clone());
    let ana = app.person("ana@acme.test").await;
    payments(&pool, ana.id, None).await;
    let (free, pro) = (plan_id(&pool, "free").await, plan_id(&pool, "pro").await);

    let (status, body) = app
        .send(
            "POST",
            &format!("/api/plans/{pro}/checkout"),
            &ana,
            json!({}),
        )
        .await;
    assert_eq!(
        (status, body["error"].as_str()),
        (StatusCode::CONFLICT, Some("not_for_sale"))
    );
    let (status, body) = app
        .send(
            "POST",
            &format!("/api/plans/{free}/checkout"),
            &ana,
            json!({}),
        )
        .await;
    assert_eq!(
        (status, body["error"].as_str()),
        (StatusCode::CONFLICT, Some("already_on_plan"))
    );
    let (status, _) = app
        .send(
            "POST",
            &format!("/api/plans/{}/waitlist", Uuid::new_v4()),
            &ana,
            json!({}),
        )
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    for _ in 0..2 {
        let (status, body) = app
            .send(
                "POST",
                &format!("/api/plans/{pro}/waitlist"),
                &ana,
                json!({}),
            )
            .await;
        assert_eq!(
            (status, body["on_waitlist"].as_bool()),
            (StatusCode::OK, Some(true))
        );
    }
    let (_, plans) = app.get("/api/plans", &ana).await;
    assert_eq!(plans["waitlist"], json!([pro]));
    assert_eq!(plans["checkout_available"], json!(false));
    assert!(plans["plans"]
        .as_array()
        .unwrap()
        .iter()
        .all(|p| p["price_idr"].is_null()));

    // A plan with a price, but no payment gateway set up.
    sqlx::query("UPDATE plans SET price_idr = $2 WHERE id = $1")
        .bind(pro)
        .bind(PRICE_IDR)
        .execute(&pool)
        .await
        .unwrap();
    let (status, body) = app
        .send(
            "POST",
            &format!("/api/plans/{pro}/checkout"),
            &ana,
            json!({}),
        )
        .await;
    assert_eq!(
        (status, body["error"].as_str()),
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Some("payments_not_configured")
        )
    );
    assert_eq!(
        callback(&app, Some(WEBHOOK_TOKEN), json!({})).await,
        StatusCode::SERVICE_UNAVAILABLE
    );

    let (_, usage) = app.get("/api/usage", &ana).await;
    assert_eq!(
        (usage["paid_idr"].as_i64(), usage["payment_method"].clone()),
        (Some(0), json!(null))
    );
    assert_eq!(
        usage["usd_to_idr"].as_u64(),
        Some(nomi_server::routes::usage::DEFAULT_USD_TO_IDR)
    );
    assert!(
        usage["plan"].is_object(),
        "the month's usage is still there: {usage}"
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn paying_for_a_plan_moves_the_person_onto_it(pool: PgPool) {
    let _g = GLOBALS.lock().await;
    let app = App::new(pool.clone());
    let ana = app.person("ana@acme.test").await;
    let pro = plan_id(&pool, "pro").await;
    sqlx::query("UPDATE plans SET price_idr = $2 WHERE id = $1")
        .bind(pro)
        .bind(PRICE_IDR)
        .execute(&pool)
        .await
        .unwrap();
    app.send(
        "POST",
        &format!("/api/plans/{pro}/waitlist"),
        &ana,
        json!({}),
    )
    .await;

    let xendit = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v2/invoices"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "inv_1", "external_id": "ignored", "status": "PENDING", "amount": PRICE_IDR,
            "invoice_url": "https://checkout.xendit.test/inv_1", "currency": "IDR"
        })))
        .expect(1)
        .mount(&xendit)
        .await;
    payments(&pool, ana.id, Some(&xendit.uri())).await;

    let (_, plans) = app.get("/api/plans", &ana).await;
    assert_eq!(plans["checkout_available"], json!(true));
    let (status, body) = app
        .send(
            "POST",
            &format!("/api/plans/{pro}/checkout"),
            &ana,
            json!({}),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["url"], "https://checkout.xendit.test/inv_1");
    let external_id: String =
        sqlx::query_scalar("SELECT external_id FROM plan_payments WHERE user_id = $1 AND invoice_id = 'inv_1' AND status = 'pending'").bind(ana.id).fetch_one(&pool).await.unwrap();

    let paid = |amount: i64| json!({ "id": "inv_1", "external_id": external_id, "status": "PAID", "amount": amount, "payment_method": "BANK_TRANSFER", "payment_channel": "BCA", "paid_at": chrono::Utc::now() });
    assert_eq!(
        callback(&app, None, paid(PRICE_IDR)).await,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        callback(&app, Some("wrong"), paid(PRICE_IDR)).await,
        StatusCode::UNAUTHORIZED
    );
    // Another amount than asked moves nobody.
    assert_eq!(
        callback(&app, Some(WEBHOOK_TOKEN), paid(1)).await,
        StatusCode::OK
    );
    let (_, plans) = app.get("/api/plans", &ana).await;
    assert_ne!(plans["current_plan_id"], json!(pro));

    assert_eq!(
        callback(&app, Some(WEBHOOK_TOKEN), paid(PRICE_IDR)).await,
        StatusCode::OK
    );
    let (_, plans) = app.get("/api/plans", &ana).await;
    assert_eq!(plans["current_plan_id"], json!(pro));
    assert_eq!(plans["waitlist"], json!([]));
    let told: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM notifications WHERE user_id = $1 AND kind = 'subscription'",
    )
    .bind(ana.id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(told, 1);
    // The same callback again changes nothing.
    assert_eq!(
        callback(&app, Some(WEBHOOK_TOKEN), paid(PRICE_IDR)).await,
        StatusCode::OK
    );
    let changes: i64 =
        sqlx::query_scalar("SELECT count(*) FROM subscription_changes WHERE user_id = $1")
            .bind(ana.id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(changes, 1);

    let (_, usage) = app.get("/api/usage", &ana).await;
    assert_eq!(usage["paid_idr"].as_i64(), Some(PRICE_IDR));
    assert_eq!(usage["payment_method"], "BANK_TRANSFER · BCA");
    assert_eq!(usage["usd_to_idr"].as_u64(), Some(15_500));

    let (status, body) = app
        .send(
            "POST",
            &format!("/api/plans/{pro}/checkout"),
            &ana,
            json!({}),
        )
        .await;
    assert_eq!(
        (status, body["error"].as_str()),
        (StatusCode::CONFLICT, Some("already_on_plan"))
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_gateway_that_refuses_fails_the_checkout(pool: PgPool) {
    let _g = GLOBALS.lock().await;
    let app = App::new(pool.clone());
    let ana = app.person("ana@acme.test").await;
    let pro = plan_id(&pool, "pro").await;
    sqlx::query("UPDATE plans SET price_idr = $2 WHERE id = $1")
        .bind(pro)
        .bind(PRICE_IDR)
        .execute(&pool)
        .await
        .unwrap();
    let xendit = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v2/invoices"))
        .respond_with(
            ResponseTemplate::new(400)
                .set_body_json(json!({ "error_code": "API_VALIDATION_ERROR", "message": "nope" })),
        )
        .mount(&xendit)
        .await;
    payments(&pool, ana.id, Some(&xendit.uri())).await;

    let (status, body) = app
        .send(
            "POST",
            &format!("/api/plans/{pro}/checkout"),
            &ana,
            json!({}),
        )
        .await;
    assert_eq!(
        (status, body["error"].as_str()),
        (StatusCode::BAD_GATEWAY, Some("gateway_error"))
    );
    let failed: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM plan_payments WHERE user_id = $1 AND status = 'failed'",
    )
    .bind(ana.id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(failed, 1);
    // Expiry of an unknown invoice is answered, and ignored.
    let status = callback(
        &app,
        Some(WEBHOOK_TOKEN),
        json!({ "id": "x", "external_id": "nomi-plan-unknown", "status": "EXPIRED", "amount": 1 }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
}
