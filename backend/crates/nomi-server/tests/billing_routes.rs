//! Paying for a plan: checkout, the Xendit webhook, polling, cancelling, the renewal worker and
//! the admin routes, against a fake payment gateway.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

use async_trait::async_trait;
use axum::{body::Body, http::{Request, StatusCode}};
use chrono::{Duration, Utc};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;

use nomi_payment::{CreateInvoiceRequest, Invoice, PaymentError, PaymentGateway, PaymentStatus, WebhookEvent};
use nomi_server::app::{build_router, AppState};
use nomi_server::billing::{set_gateway_override, Gateway};
use nomi_usage::billing as usage_billing;
use nomi_usage::plans::{self, PlanInput, PlanRow};

const SECRET: &str = "test-secret-do-not-use-in-prod";
const HOOK_TOKEN: &str = "hook-secret";
const PRICE: i64 = 50_000;
const SIX_MONTHS_AMOUNT: i64 = 270_000;
const INVOICE_HOURS: u32 = 24;

/// A gateway that remembers the invoices it made and answers `get_invoice` with a status the
/// test sets. Every test shares one, so they take turns (the override is process-wide).
#[derive(Default)]
struct FakeGateway {
    invoices: Mutex<HashMap<String, (CreateInvoiceRequest, PaymentStatus)>>,
    expired: Mutex<Vec<String>>,
}

impl FakeGateway {
    fn set_status(&self, invoice_id: &str, status: PaymentStatus) {
        self.invoices.lock().unwrap().get_mut(invoice_id).unwrap().1 = status;
    }
    fn request(&self, invoice_id: &str) -> CreateInvoiceRequest {
        self.invoices.lock().unwrap().get(invoice_id).unwrap().0.clone()
    }
    fn invoice(&self, invoice_id: &str) -> Invoice {
        let (request, status) = self.invoices.lock().unwrap().get(invoice_id).unwrap().clone();
        Invoice {
            id: invoice_id.to_string(),
            external_id: request.external_id,
            user_id: None,
            status,
            amount: request.amount,
            invoice_url: format!("https://pay.test/{invoice_id}"),
            expiry_date: None,
            paid_at: None,
            payment_method: Some("QRIS".into()),
            currency: request.currency.unwrap_or_default(),
        }
    }
}

#[async_trait]
impl PaymentGateway for FakeGateway {
    async fn create_invoice(&self, request: CreateInvoiceRequest) -> Result<Invoice, PaymentError> {
        let id = format!("inv-{}", request.external_id);
        self.invoices.lock().unwrap().insert(id.clone(), (request, PaymentStatus::Pending));
        Ok(self.invoice(&id))
    }
    async fn get_invoice(&self, invoice_id: &str) -> Result<Invoice, PaymentError> {
        Ok(self.invoice(invoice_id))
    }
    async fn expire_invoice(&self, invoice_id: &str) -> Result<Invoice, PaymentError> {
        self.expired.lock().unwrap().push(invoice_id.to_string());
        self.set_status(invoice_id, PaymentStatus::Expired);
        Ok(self.invoice(invoice_id))
    }
    fn verify_webhook(&self, _token: Option<&str>, payload: &[u8]) -> Result<WebhookEvent, PaymentError> {
        let raw: Value = serde_json::from_slice(payload)?;
        Ok(WebhookEvent {
            event_type: "invoice".into(),
            invoice_id: raw["id"].as_str().unwrap_or_default().to_string(),
            external_id: raw["external_id"].as_str().unwrap_or_default().to_string(),
            status: serde_json::from_value(raw["status"].clone())?,
            amount: raw["amount"].as_f64().unwrap_or_default(),
            paid_at: None,
            payment_method: raw["payment_method"].as_str().map(str::to_string),
            payment_channel: None,
            raw_payload: raw,
        })
    }
}

fn fake() -> Arc<FakeGateway> {
    static FAKE: OnceLock<Arc<FakeGateway>> = OnceLock::new();
    FAKE.get_or_init(|| Arc::new(FakeGateway::default())).clone()
}

static TURN: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

/// Takes the turn and installs the fake with `token` as the webhook token.
async fn with_gateway(token: Option<&str>) -> tokio::sync::MutexGuard<'static, ()> {
    let guard = TURN.lock().await;
    set_gateway_override(Some(Gateway { client: fake(), webhook_token: token.map(str::to_string) }));
    guard
}

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
        push: std::sync::Arc::new(nomi_server::push::NoopPush),
        shipyard: None,
    }
}

async fn request(router: axum::Router, method: &str, uri: &str, body: Value, bearer: Option<&str>, headers: &[(&str, &str)]) -> (StatusCode, Value) {
    let mut builder = Request::builder().method(method).uri(uri).header("content-type", "application/json");
    if let Some(token) = bearer {
        builder = builder.header("authorization", format!("Bearer {token}"));
    }
    for (name, value) in headers {
        builder = builder.header(*name, *value);
    }
    let response = router.oneshot(builder.body(Body::from(body.to_string())).unwrap()).await.unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    (status, if bytes.is_empty() { Value::Null } else { serde_json::from_slice(&bytes).unwrap_or(Value::Null) })
}

async fn call(router: &axum::Router, method: &str, uri: &str, body: Value, bearer: &str) -> (StatusCode, Value) {
    request(router.clone(), method, uri, body, Some(bearer), &[]).await
}

async fn register_and_login(router: &axum::Router, email: &str) -> String {
    request(
        router.clone(),
        "POST",
        "/api/auth/register",
        json!({ "email": email, "password": "correct-password", "org": { "mode": "create", "name": "Acme" } }),
        None,
        &[],
    )
    .await;
    login(router, email).await
}

async fn login(router: &axum::Router, email: &str) -> String {
    let (_, body) = request(router.clone(), "POST", "/api/auth/login", json!({ "email": email, "password": "correct-password" }), None, &[]).await;
    body["access_token"].as_str().unwrap().to_string()
}

async fn user_id(pool: &PgPool, email: &str) -> Uuid {
    sqlx::query_scalar("SELECT user_id FROM web_credentials WHERE email = $1").bind(email).fetch_one(pool).await.unwrap()
}

async fn admin(router: &axum::Router, pool: &PgPool) -> String {
    register_and_login(router, "boss@example.com").await;
    sqlx::query("UPDATE users SET is_platform_admin = true WHERE id = (SELECT user_id FROM web_credentials WHERE email = 'boss@example.com')").execute(pool).await.unwrap();
    login(router, "boss@example.com").await
}

async fn paid_plan(pool: &PgPool) -> PlanRow {
    let input: PlanInput = serde_json::from_value(json!({ "slug": "plus", "name": "Plus", "monthly_tokens": 100, "price_amount": PRICE })).unwrap();
    plans::create_plan(pool, &input.validate().unwrap()).await.unwrap()
}

async fn notices(pool: &PgPool, user: Uuid) -> i64 {
    sqlx::query_scalar("SELECT count(*) FROM notifications WHERE user_id = $1 AND kind = 'subscription'").bind(user).fetch_one(pool).await.unwrap()
}

fn webhook_body(payment_id: &str, status: &str, amount: i64) -> Value {
    json!({ "id": format!("inv-{payment_id}"), "external_id": payment_id, "status": status, "amount": amount, "payment_method": "QRIS" })
}

async fn webhook(router: &axum::Router, token: Option<&str>, body: Value) -> StatusCode {
    let headers: Vec<(&str, &str)> = token.map(|t| vec![("x-callback-token", t)]).unwrap_or_default();
    request(router.clone(), "POST", "/api/payments/xendit/webhook", body, None, &headers).await.0
}

async fn checkout(router: &axum::Router, token: &str, plan: &PlanRow, months: i32, code: Option<&str>) -> (StatusCode, Value) {
    call(router, "POST", "/api/billing/checkout", json!({ "plan_id": plan.id, "months": months, "code": code }), token).await
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_webhook_needs_a_configured_token_and_the_right_one(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let body = webhook_body(&Uuid::new_v4().to_string(), "PAID", PRICE);
    {
        let _turn = with_gateway(None).await;
        assert_eq!(webhook(&router, Some(HOOK_TOKEN), body.clone()).await, StatusCode::SERVICE_UNAVAILABLE, "no token configured: refuse, even with a token sent");
        assert_eq!(webhook(&router, None, body.clone()).await, StatusCode::SERVICE_UNAVAILABLE);
    }
    {
        let _turn = with_gateway(Some(HOOK_TOKEN)).await;
        assert_eq!(webhook(&router, None, body.clone()).await, StatusCode::UNAUTHORIZED);
        assert_eq!(webhook(&router, Some("wrong"), body.clone()).await, StatusCode::UNAUTHORIZED);
        assert_eq!(webhook(&router, Some("hook-secre"), body.clone()).await, StatusCode::UNAUTHORIZED);
        // An unknown payment is acknowledged and ignored.
        assert_eq!(webhook(&router, Some(HOOK_TOKEN), body).await, StatusCode::OK);
        assert_eq!(webhook(&router, Some(HOOK_TOKEN), json!({ "id": "x", "external_id": "not-a-uuid", "status": "PAID", "amount": 1 })).await, StatusCode::OK);
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn paying_an_invoice_activates_the_plan_once(pool: PgPool) {
    let _turn = with_gateway(Some(HOOK_TOKEN)).await;
    let router = build_router(test_state(pool.clone()));
    let plan = paid_plan(&pool).await;
    let ana = register_and_login(&router, "ana@example.com").await;
    let ana_id = user_id(&pool, "ana@example.com").await;

    let (status, first) = checkout(&router, &ana, &plan, 6, None).await;
    assert_eq!(status, StatusCode::OK, "{first}");
    assert_eq!(first["status"], "pending");
    let payment_id = first["payment_id"].as_str().unwrap().to_string();
    let invoice_id = format!("inv-{payment_id}");
    assert_eq!(first["invoice_url"], format!("https://pay.test/{invoice_id}"));
    let sent = fake().request(&invoice_id);
    assert_eq!((sent.amount, sent.currency.as_deref(), sent.invoice_duration), (SIX_MONTHS_AMOUNT as f64, Some("IDR"), Some(INVOICE_HOURS * 3600)));
    assert_eq!(sent.description.as_deref(), Some("Plus \u{2014} 6 months"));
    assert_eq!(sent.payer_email.as_deref(), Some("ana@example.com"));

    // The same request while the invoice is open gives the same invoice back.
    let (_, again) = checkout(&router, &ana, &plan, 6, None).await;
    assert_eq!(again["payment_id"], first["payment_id"]);

    assert_eq!(webhook(&router, Some(HOOK_TOKEN), webhook_body(&payment_id, "PAID", SIX_MONTHS_AMOUNT)).await, StatusCode::OK);
    let (_, sub) = call(&router, "GET", "/api/billing/subscription", Value::Null, &ana).await;
    assert_eq!((sub["plan"]["id"].as_str(), sub["term_months"].as_i64(), sub["auto_renew"].as_bool()), (Some(plan.id.to_string().as_str()), Some(6), Some(true)));
    assert!(sub["paid_until"].is_string());
    assert_eq!(notices(&pool, ana_id).await, 1);

    // Xendit retries: nothing changes.
    let before = sub["paid_until"].clone();
    assert_eq!(webhook(&router, Some(HOOK_TOKEN), webhook_body(&payment_id, "PAID", SIX_MONTHS_AMOUNT)).await, StatusCode::OK);
    let (_, sub) = call(&router, "GET", "/api/billing/subscription", Value::Null, &ana).await;
    assert_eq!(sub["paid_until"], before);
    assert_eq!(notices(&pool, ana_id).await, 1);
    let changes: i64 = sqlx::query_scalar("SELECT count(*) FROM subscription_changes WHERE user_id = $1").bind(ana_id).fetch_one(&pool).await.unwrap();
    assert_eq!(changes, 1);

    let (_, payments) = call(&router, "GET", "/api/billing/payments", Value::Null, &ana).await;
    assert_eq!(payments[0]["status"], "paid");
    assert_eq!(payments[0]["plan_name"], "Plus");
    assert_eq!(payments[0]["payment_method"], "QRIS");
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_payment_for_the_wrong_amount_activates_nothing(pool: PgPool) {
    let _turn = with_gateway(Some(HOOK_TOKEN)).await;
    let router = build_router(test_state(pool.clone()));
    let plan = paid_plan(&pool).await;
    let ana = register_and_login(&router, "ana@example.com").await;
    let (_, made) = checkout(&router, &ana, &plan, 1, None).await;
    let payment_id = made["payment_id"].as_str().unwrap();
    assert_eq!(webhook(&router, Some(HOOK_TOKEN), webhook_body(payment_id, "PAID", PRICE - 1)).await, StatusCode::OK);
    let (_, payment) = call(&router, "GET", &format!("/api/billing/payments/{payment_id}"), Value::Null, &ana).await;
    // The poll asks the gateway too, which knows it as still pending.
    assert_eq!(payment["status"], "pending");
    let (_, sub) = call(&router, "GET", "/api/billing/subscription", Value::Null, &ana).await;
    assert_eq!(sub["paid_until"], Value::Null);
}

#[sqlx::test(migrations = "../../migrations")]
async fn an_expired_invoice_gives_its_coupon_back(pool: PgPool) {
    let _turn = with_gateway(Some(HOOK_TOKEN)).await;
    let router = build_router(test_state(pool.clone()));
    let boss = admin(&router, &pool).await;
    let plan = paid_plan(&pool).await;
    let (status, coupon) = call(&router, "POST", "/api/admin/coupons", json!({ "code": "once", "discount_kind": "percent", "discount_value": 20, "max_redemptions": 1 }), &boss).await;
    assert_eq!(status, StatusCode::CREATED, "{coupon}");
    assert_eq!(coupon["code"], "ONCE");

    let ana = register_and_login(&router, "ana@example.com").await;
    let ben = register_and_login(&router, "ben@example.com").await;
    let (_, made) = checkout(&router, &ana, &plan, 1, Some("once")).await;
    let payment_id = made["payment_id"].as_str().unwrap().to_string();
    assert_eq!(fake().request(&format!("inv-{payment_id}")).amount, 40_000.0);

    let (status, refused) = call(&router, "POST", "/api/billing/quote", json!({ "plan_id": plan.id, "months": 1, "code": "ONCE" }), &ben).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(refused, Value::Null, "plain-text message, not JSON");

    assert_eq!(webhook(&router, Some(HOOK_TOKEN), webhook_body(&payment_id, "EXPIRED", 40_000)).await, StatusCode::OK);
    let status: String = sqlx::query_scalar("SELECT status FROM payments WHERE id = $1").bind(Uuid::parse_str(&payment_id).unwrap()).fetch_one(&pool).await.unwrap();
    assert_eq!(status, "expired");
    let (status, quote) = call(&router, "POST", "/api/billing/quote", json!({ "plan_id": plan.id, "months": 1, "code": "once" }), &ben).await;
    assert_eq!(status, StatusCode::OK, "{quote}");
    assert_eq!(quote["amount"], 40_000);

    // An expired payment that is later paid still activates (the money arrived).
    assert_eq!(webhook(&router, Some(HOOK_TOKEN), webhook_body(&payment_id, "PAID", 40_000)).await, StatusCode::OK);
    let (_, sub) = call(&router, "GET", "/api/billing/subscription", Value::Null, &ana).await;
    assert!(sub["paid_until"].is_string());
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_free_checkout_activates_without_an_invoice_and_needs_no_gateway(pool: PgPool) {
    let turn = TURN.lock().await;
    set_gateway_override(None);
    let router = build_router(test_state(pool.clone()));
    let boss = admin(&router, &pool).await;
    let plan = paid_plan(&pool).await;
    call(&router, "POST", "/api/admin/coupons", json!({ "code": "FREE", "discount_kind": "percent", "discount_value": 100 }), &boss).await;
    let ana = register_and_login(&router, "ana@example.com").await;

    // Something to pay and no gateway: 503, and no payment is left behind.
    let (status, _) = checkout(&router, &ana, &plan, 1, None).await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    let left: i64 = sqlx::query_scalar("SELECT count(*) FROM payments").fetch_one(&pool).await.unwrap();
    assert_eq!(left, 0);

    let (status, free) = checkout(&router, &ana, &plan, 1, Some("free")).await;
    assert_eq!(status, StatusCode::OK, "{free}");
    assert_eq!((free["status"].as_str(), &free["invoice_url"]), (Some("paid"), &Value::Null));
    let (_, sub) = call(&router, "GET", "/api/billing/subscription", Value::Null, &ana).await;
    assert_eq!(sub["plan"]["id"], plan.id.to_string());
    drop(turn);
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_return_page_asks_the_gateway_and_a_payment_can_be_cancelled(pool: PgPool) {
    let _turn = with_gateway(Some(HOOK_TOKEN)).await;
    let router = build_router(test_state(pool.clone()));
    let plan = paid_plan(&pool).await;
    let ana = register_and_login(&router, "ana@example.com").await;
    let ben = register_and_login(&router, "ben@example.com").await;

    // The webhook is late: opening the payment applies what Xendit says.
    let (_, made) = checkout(&router, &ana, &plan, 1, None).await;
    let payment_id = made["payment_id"].as_str().unwrap().to_string();
    fake().set_status(&format!("inv-{payment_id}"), PaymentStatus::Paid);
    let (status, payment) = call(&router, "GET", &format!("/api/billing/payments/{payment_id}"), Value::Null, &ana).await;
    assert_eq!((status, payment["status"].as_str()), (StatusCode::OK, Some("paid")));
    // Not someone else's.
    let (status, _) = call(&router, "GET", &format!("/api/billing/payments/{payment_id}"), Value::Null, &ben).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    // A paid one can't be cancelled.
    let (status, _) = call(&router, "POST", &format!("/api/billing/payments/{payment_id}/cancel"), Value::Null, &ana).await;
    assert_eq!(status, StatusCode::CONFLICT);

    let (_, other) = checkout(&router, &ben, &plan, 6, None).await;
    let other_id = other["payment_id"].as_str().unwrap().to_string();
    let (status, cancelled) = call(&router, "POST", &format!("/api/billing/payments/{other_id}/cancel"), Value::Null, &ben).await;
    assert_eq!((status, cancelled["status"].as_str()), (StatusCode::OK, Some("cancelled")));
    assert!(fake().expired.lock().unwrap().contains(&format!("inv-{other_id}")));
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_renewal_worker_invoices_reminds_and_announces_once(pool: PgPool) {
    let _turn = with_gateway(Some(HOOK_TOKEN)).await;
    let plan = paid_plan(&pool).await;
    let cfg = usage_billing::BillingConfig::default();
    let mut people = Vec::new();
    for _ in 0..4 {
        let user: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(&pool).await.unwrap();
        let q = usage_billing::quote(&pool, user, plan.id, 1, None, &cfg).await.unwrap();
        let payment = usage_billing::create_payment(&pool, user, &q, usage_billing::KIND_NEW, INVOICE_HOURS).await.unwrap();
        let mut tx = pool.begin().await.unwrap();
        usage_billing::activate(&mut tx, payment.id, None, None, &cfg).await.unwrap();
        tx.commit().await.unwrap();
        people.push(user);
    }
    let [renews, manual, ended, later] = people[..] else { unreachable!() };
    for (user, interval) in [(renews, "2 days"), (manual, "2 days"), (ended, "-10 days"), (later, "20 days")] {
        sqlx::query(&format!("UPDATE user_subscriptions SET paid_until = now() + interval '{interval}' WHERE user_id = $1")).bind(user).execute(&pool).await.unwrap();
    }
    sqlx::query("UPDATE user_subscriptions SET auto_renew = false WHERE user_id = $1").bind(manual).execute(&pool).await.unwrap();
    // Activation above told each of them once; count from there.
    let base = notices(&pool, renews).await;

    let summary = nomi_server::billing_worker::run_once(&pool, Utc::now()).await.unwrap();
    assert_eq!((summary.invoiced, summary.reminded, summary.ended), (1, 1, 1));
    let renewal: (String, String, i32, i64) = sqlx::query_as("SELECT kind, status, term_months, amount FROM payments WHERE user_id = $1 AND kind = 'renewal'").bind(renews).fetch_one(&pool).await.unwrap();
    assert_eq!(renewal, ("renewal".to_string(), "pending".to_string(), 1, PRICE));
    assert_eq!(notices(&pool, renews).await, base + 1);
    assert_eq!(notices(&pool, manual).await, base + 1);
    assert_eq!(notices(&pool, ended).await, base + 1);
    assert_eq!(notices(&pool, later).await, base);
    let invoice: Option<String> = sqlx::query_scalar("SELECT invoice_url FROM payments WHERE user_id = $1 AND kind = 'renewal'").bind(renews).fetch_one(&pool).await.unwrap();
    assert!(invoice.unwrap().starts_with("https://pay.test/inv-"));

    // A second pass finds everything already done.
    let summary = nomi_server::billing_worker::run_once(&pool, Utc::now()).await.unwrap();
    assert_eq!((summary.invoiced, summary.reminded, summary.ended), (0, 0, 0));
    let renewals: i64 = sqlx::query_scalar("SELECT count(*) FROM payments WHERE kind = 'renewal'").fetch_one(&pool).await.unwrap();
    assert_eq!(renewals, 1);

    // Paying the invoice moves the end out by the term, from where it ended.
    let paid_until: chrono::DateTime<Utc> = sqlx::query_scalar("SELECT paid_until FROM user_subscriptions WHERE user_id = $1").bind(renews).fetch_one(&pool).await.unwrap();
    let renewal_id: Uuid = sqlx::query_scalar("SELECT id FROM payments WHERE kind = 'renewal'").fetch_one(&pool).await.unwrap();
    let router = build_router(test_state(pool.clone()));
    assert_eq!(webhook(&router, Some(HOOK_TOKEN), webhook_body(&renewal_id.to_string(), "PAID", PRICE)).await, StatusCode::OK);
    let after: chrono::DateTime<Utc> = sqlx::query_scalar("SELECT paid_until FROM user_subscriptions WHERE user_id = $1").bind(renews).fetch_one(&pool).await.unwrap();
    assert!(after > paid_until + Duration::days(27), "{after} vs {paid_until}");
}

#[sqlx::test(migrations = "../../migrations")]
async fn terms_coupons_and_payments_are_managed_by_admins_and_read_by_everyone(pool: PgPool) {
    let _turn = with_gateway(Some(HOOK_TOKEN)).await;
    let router = build_router(test_state(pool.clone()));
    let boss = admin(&router, &pool).await;
    let ana = register_and_login(&router, "ana@example.com").await;
    let plan = paid_plan(&pool).await;

    let (_, terms) = call(&router, "GET", "/api/billing/terms", Value::Null, &ana).await;
    assert_eq!(terms.as_array().unwrap().iter().map(|t| t["months"].as_i64().unwrap()).collect::<Vec<_>>(), vec![1, 6, 12]);
    for (method, uri) in [("GET", "/api/admin/billing/terms"), ("GET", "/api/admin/coupons"), ("GET", "/api/admin/payments")] {
        assert_eq!(call(&router, method, uri, Value::Null, &ana).await.0, StatusCode::FORBIDDEN, "{uri}");
    }

    let (status, term) = call(&router, "POST", "/api/admin/billing/terms", json!({ "months": 3, "discount_percent": 5, "label": "Quarter" }), &boss).await;
    assert_eq!(status, StatusCode::CREATED, "{term}");
    let term_id = term["id"].as_str().unwrap();
    assert_eq!(call(&router, "POST", "/api/admin/billing/terms", json!({ "months": 3 }), &boss).await.0, StatusCode::CONFLICT);
    assert_eq!(call(&router, "POST", "/api/admin/billing/terms", json!({ "months": 0 }), &boss).await.0, StatusCode::BAD_REQUEST);
    let (status, term) = call(&router, "PUT", &format!("/api/admin/billing/terms/{term_id}"), json!({ "months": 3, "discount_percent": 8, "is_active": false }), &boss).await;
    assert_eq!((status, term["discount_percent"].as_i64()), (StatusCode::OK, Some(8)));
    let (_, terms) = call(&router, "GET", "/api/billing/terms", Value::Null, &ana).await;
    assert_eq!(terms.as_array().unwrap().len(), 3, "a hidden term isn't offered");
    assert_eq!(call(&router, "DELETE", &format!("/api/admin/billing/terms/{term_id}"), Value::Null, &boss).await.0, StatusCode::NO_CONTENT);

    let (_, coupon) = call(&router, "POST", "/api/admin/coupons", json!({ "code": "save", "discount_kind": "amount", "discount_value": 5000 }), &boss).await;
    let coupon_id = coupon["id"].as_str().unwrap();
    assert_eq!(call(&router, "POST", "/api/admin/coupons", json!({ "code": "SAVE", "discount_kind": "amount", "discount_value": 1 }), &boss).await.0, StatusCode::CONFLICT);
    let (_, made) = checkout(&router, &ana, &plan, 1, Some("save")).await;
    assert_eq!(fake().request(&format!("inv-{}", made["payment_id"].as_str().unwrap())).amount, 45_000.0);
    let (_, coupons) = call(&router, "GET", "/api/admin/coupons", Value::Null, &boss).await;
    assert_eq!(coupons[0]["redemptions"], 1);
    assert_eq!(call(&router, "DELETE", &format!("/api/admin/coupons/{coupon_id}"), Value::Null, &boss).await.0, StatusCode::CONFLICT);
    let (status, off) = call(&router, "PUT", &format!("/api/admin/coupons/{coupon_id}"), json!({ "code": "save", "discount_kind": "amount", "discount_value": 5000, "is_active": false }), &boss).await;
    assert_eq!((status, off["is_active"].as_bool()), (StatusCode::OK, Some(false)));

    let (status, page) = call(&router, "GET", "/api/admin/payments?status=pending&page=1", Value::Null, &boss).await;
    assert_eq!(status, StatusCode::OK, "{page}");
    assert_eq!((page["total"].as_i64(), page["items"][0]["user_email"].as_str(), page["items"][0]["plan_name"].as_str()), (Some(1), Some("ana@example.com"), Some("Plus")));
    assert_eq!(call(&router, "GET", "/api/admin/payments?status=bogus", Value::Null, &boss).await.0, StatusCode::BAD_REQUEST);
}

#[sqlx::test(migrations = "../../migrations")]
async fn plans_carry_prices_and_referral_and_auto_renew_work(pool: PgPool) {
    let _turn = with_gateway(Some(HOOK_TOKEN)).await;
    let router = build_router(test_state(pool.clone()));
    let plan = paid_plan(&pool).await;
    let ended: PlanInput = serde_json::from_value(
        json!({ "slug": "old-promo", "name": "Old promo", "monthly_tokens": 100, "price_amount": PRICE, "promo_price_amount": 1000, "promo_ends_at": Utc::now() - Duration::days(1) }),
    )
    .unwrap();
    plans::create_plan(&pool, &ended.validate().unwrap()).await.unwrap();
    let ana = register_and_login(&router, "ana@example.com").await;
    let ben = register_and_login(&router, "ben@example.com").await;

    let (_, listed) = call(&router, "GET", "/api/plans", Value::Null, &ana).await;
    let find = |slug: &str| listed["plans"].as_array().unwrap().iter().find(|p| p["slug"] == slug).unwrap().clone();
    assert_eq!((find("plus")["price_amount"].as_i64(), find("plus")["currency"].as_str(), find("plus")["promo_price_amount"].clone()), (Some(PRICE), Some("IDR"), Value::Null));
    assert_eq!(find("old-promo")["promo_price_amount"], Value::Null, "an ended promo is dropped");

    let (_, referral) = call(&router, "GET", "/api/billing/referral", Value::Null, &ana).await;
    let code = referral["code"].as_str().unwrap().to_string();
    assert_eq!((code.len(), referral["referred_count"].as_i64(), referral["referee_discount_percent"].as_i64(), referral["referrer_bonus_days"].as_i64()), (8, Some(0), Some(10), Some(30)));
    let (_, again) = call(&router, "GET", "/api/billing/referral", Value::Null, &ana).await;
    assert_eq!(again["code"], referral["code"]);

    // Ben pays with Ana's code: 10% off, and Ana's reward waits for her next paid period.
    let (_, made) = checkout(&router, &ben, &plan, 1, Some(&code)).await;
    let payment_id = made["payment_id"].as_str().unwrap().to_string();
    assert_eq!(fake().request(&format!("inv-{payment_id}")).amount, 45_000.0);
    assert_eq!(webhook(&router, Some(HOOK_TOKEN), webhook_body(&payment_id, "PAID", 45_000)).await, StatusCode::OK);
    let (_, referral) = call(&router, "GET", "/api/billing/referral", Value::Null, &ana).await;
    assert_eq!((referral["referred_count"].as_i64(), referral["rewarded_count"].as_i64()), (Some(1), Some(1)));
    let (_, sub) = call(&router, "GET", "/api/billing/subscription", Value::Null, &ana).await;
    assert_eq!(sub["bonus_days"], 30);

    let (status, saved) = call(&router, "PUT", "/api/billing/auto-renew", json!({ "auto_renew": false }), &ben).await;
    assert_eq!((status, saved["auto_renew"].as_bool()), (StatusCode::OK, Some(false)));
    let (_, sub) = call(&router, "GET", "/api/billing/subscription", Value::Null, &ben).await;
    assert_eq!(sub["auto_renew"], false);

    let (status, _) = call(&router, "POST", "/api/billing/quote", json!({ "plan_id": plan.id, "months": 1, "code": "NOSUCH" }), &ben).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}
