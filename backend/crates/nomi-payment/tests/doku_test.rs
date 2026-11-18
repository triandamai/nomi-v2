use http::{HeaderMap, HeaderValue};
use nomi_payment::doku::{external_id_from, invoice_number_for, signature, DokuClient, DokuConfig};
use nomi_payment::types::{CreateInvoiceRequest, PaymentStatus};
use nomi_payment::{PaymentError, PaymentGateway};
use serde_json::json;
use wiremock::matchers::{body_partial_json, header, header_exists, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const CLIENT_ID: &str = "BRN-0001";
const SECRET: &str = "SK-secret";
const PAYMENT_ID: &str = "0f8fad5b-d9cb-469f-a165-70867728950e";
const INVOICE: &str = "NMX5YG3BFYREHLAS5FMD3RSLXQ";
/// A notification body and the signature DOKU would send with it (worked out independently).
const NOTIFICATION: &[u8] = br#"{"order":{"invoice_number":"INV-1","amount":150000},"transaction":{"status":"SUCCESS","date":"2026-11-18T03:24:23Z"},"service":{"id":"VIRTUAL_ACCOUNT"},"channel":{"id":"VIRTUAL_ACCOUNT_BCA"}}"#;
const NOTIFICATION_SIGNATURE: &str = "HMACSHA256=Z1CmaUMxiI9qmLf4+TAmHvOgDIa4NaVSOLrlEsD47r8=";

fn config(base_url: Option<String>) -> DokuConfig {
    DokuConfig { client_id: CLIENT_ID.to_string(), secret_key: SECRET.to_string(), base_url, notification_path: None }
}

fn notification_headers(signature: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert("client-id", HeaderValue::from_static(CLIENT_ID));
    headers.insert("request-id", HeaderValue::from_static("req-1"));
    headers.insert("request-timestamp", HeaderValue::from_static("2026-11-18T03:24:25Z"));
    headers.insert("signature", HeaderValue::from_str(signature).unwrap());
    headers
}

#[test]
fn signs_the_way_doku_documents() {
    let get = signature(SECRET, CLIENT_ID, "r", "2020-08-11T08:45:42Z", "/orders/v1/status/INV-1", None);
    assert_eq!(get, "HMACSHA256=NXQn/hWEFtj516UyAQt7CHjhJSE/jTKT+nj0YAg9A6I=");
    let post = signature(SECRET, CLIENT_ID, "req-1", "2026-11-18T03:24:25Z", "/api/payments/doku/webhook", Some(NOTIFICATION));
    assert_eq!(post, NOTIFICATION_SIGNATURE);
}

#[test]
fn a_payment_id_fits_the_card_limit_and_comes_back() {
    let invoice = invoice_number_for(PAYMENT_ID);
    assert_eq!(invoice, INVOICE);
    assert!(invoice.len() <= 30, "card payments take 30 characters at most");
    assert_eq!(external_id_from(&invoice), PAYMENT_ID);
    // Anything that isn't one of ours passes through untouched.
    assert_eq!(invoice_number_for("order-1"), "order-1");
    assert_eq!(external_id_from("INV-1"), "INV-1");
}

#[test]
fn needs_a_client_id_and_secret() {
    let mut empty = config(None);
    empty.secret_key = " ".to_string();
    assert!(matches!(DokuClient::new(empty), Err(PaymentError::Configuration(_))));
    let mut no_client = config(None);
    no_client.client_id = String::new();
    assert!(matches!(DokuClient::new(no_client), Err(PaymentError::Configuration(_))));
}

#[tokio::test]
async fn creates_a_checkout_page() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/checkout/v1/payment"))
        .and(header("Client-Id", CLIENT_ID))
        .and(header("Request-Id", INVOICE))
        .and(header_exists("Request-Timestamp"))
        .and(header_exists("Signature"))
        .and(body_partial_json(json!({
            "order": { "amount": 249000, "invoice_number": INVOICE, "currency": "IDR", "callback_url": "https://nomi.build/billing" },
            "payment": { "payment_due_date": 1440 },
            "customer": { "email": "budi@example.com" }
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "message": ["SUCCESS"],
            "response": {
                "order": { "amount": "249000", "invoice_number": INVOICE, "currency": "IDR" },
                "payment": {
                    "token_id": "tok",
                    "url": "https://sandbox.doku.com/checkout-link-v2/tok",
                    "expired_date": "20261119104531"
                }
            }
        })))
        .mount(&server)
        .await;

    let client = DokuClient::new(config(Some(server.uri()))).unwrap();
    let invoice = client
        .create_invoice(CreateInvoiceRequest {
            external_id: PAYMENT_ID.to_string(),
            amount: 249000.0,
            description: Some("Pro — 1 month".to_string()),
            payer_email: Some("budi@example.com".to_string()),
            invoice_duration: Some(86_400),
            customer: None,
            currency: Some("IDR".to_string()),
            success_redirect_url: Some("https://nomi.build/billing".to_string()),
            failure_redirect_url: None,
        })
        .await
        .unwrap();

    assert_eq!(invoice.id, INVOICE);
    assert_eq!(invoice.external_id, PAYMENT_ID);
    assert_eq!(invoice.status, PaymentStatus::Pending);
    assert_eq!(invoice.amount, 249000.0);
    assert_eq!(invoice.invoice_url, "https://sandbox.doku.com/checkout-link-v2/tok");
    // DOKU gives the expiry in Jakarta time (UTC+7).
    assert_eq!(invoice.expiry_date.unwrap().to_rfc3339(), "2026-11-19T03:45:31+00:00");
}

#[tokio::test]
async fn a_refused_checkout_says_why() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/checkout/v1/payment"))
        .respond_with(ResponseTemplate::new(400).set_body_json(json!({ "error_messages": ["order.amount must greater than 0"] })))
        .mount(&server)
        .await;
    let client = DokuClient::new(config(Some(server.uri()))).unwrap();
    let request = CreateInvoiceRequest {
        external_id: PAYMENT_ID.to_string(),
        amount: 0.0,
        description: None,
        payer_email: None,
        invoice_duration: None,
        customer: None,
        currency: None,
        success_redirect_url: None,
        failure_redirect_url: None,
    };
    match client.create_invoice(request).await {
        Err(PaymentError::BadRequest { status, message }) => {
            assert_eq!(status, 400);
            assert!(message.contains("order.amount must greater than 0"));
        }
        other => panic!("expected a bad request, got {other:?}"),
    }
}

#[tokio::test]
async fn reads_a_paid_order() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(format!("/orders/v1/status/{INVOICE}")))
        .and(header("Client-Id", CLIENT_ID))
        .and(header_exists("Signature"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "order": { "invoice_number": INVOICE, "amount": 249000.0, "status": "ORDER_GENERATED" },
            "transaction": { "status": "SUCCESS", "date": "2026-11-18T03:00:48Z", "original_request_id": INVOICE },
            "service": { "id": "VIRTUAL_ACCOUNT" },
            "channel": { "id": "VIRTUAL_ACCOUNT_BCA" }
        })))
        .mount(&server)
        .await;
    let client = DokuClient::new(config(Some(server.uri()))).unwrap();
    let invoice = client.get_invoice(INVOICE).await.unwrap();
    assert_eq!(invoice.status, PaymentStatus::Paid);
    assert_eq!(invoice.external_id, PAYMENT_ID);
    assert_eq!(invoice.amount, 249000.0);
    assert_eq!(invoice.paid_at.unwrap().to_rfc3339(), "2026-11-18T03:00:48+00:00");
    assert_eq!(invoice.payment_method.as_deref(), Some("VIRTUAL_ACCOUNT_BCA"));
}

#[tokio::test]
async fn an_expired_order_reads_expired_and_a_failed_try_still_waits() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/orders/v1/status/EXP"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "order": { "invoice_number": "EXP", "amount": 1000, "status": "ORDER_EXPIRED" },
            "transaction": { "status": "PENDING" }
        })))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/orders/v1/status/FAIL"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "order": { "invoice_number": "FAIL", "amount": 1000, "status": "ORDER_GENERATED" },
            "transaction": { "status": "FAILED" }
        })))
        .mount(&server)
        .await;
    let client = DokuClient::new(config(Some(server.uri()))).unwrap();
    assert_eq!(client.get_invoice("EXP").await.unwrap().status, PaymentStatus::Expired);
    // On DOKU's checkout page the person can try another way to pay after a failure.
    assert_eq!(client.get_invoice("FAIL").await.unwrap().status, PaymentStatus::Pending);
}

#[tokio::test]
async fn cancels_with_the_request_id_it_was_made_with() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/checkout/v3/cancellations"))
        .and(header_exists("Signature"))
        .and(body_partial_json(json!({ "order": { "invoice_number": INVOICE }, "payment": { "original_request_id": INVOICE } })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "message": ["SUCCESS"] })))
        .mount(&server)
        .await;
    let client = DokuClient::new(config(Some(server.uri()))).unwrap();
    let invoice = client.expire_invoice(INVOICE).await.unwrap();
    assert_eq!(invoice.status, PaymentStatus::Expired);
    assert_eq!(invoice.external_id, PAYMENT_ID);
}

#[test]
fn accepts_a_signed_notification() {
    let client = DokuClient::new(config(None)).unwrap();
    let event = client.verify_webhook(&notification_headers(NOTIFICATION_SIGNATURE), NOTIFICATION).unwrap();
    assert_eq!(event.event_type, "invoice.paid");
    assert_eq!(event.invoice_id, "INV-1");
    assert_eq!(event.external_id, "INV-1");
    assert_eq!(event.status, PaymentStatus::Paid);
    assert_eq!(event.amount, 150000.0);
    assert_eq!(event.paid_at.unwrap().to_rfc3339(), "2026-11-18T03:24:23+00:00");
    assert_eq!(event.payment_method.as_deref(), Some("VIRTUAL_ACCOUNT"));
    assert_eq!(event.payment_channel.as_deref(), Some("VIRTUAL_ACCOUNT_BCA"));
}

#[test]
fn refuses_a_notification_that_isnt_from_doku() {
    let client = DokuClient::new(config(None)).unwrap();
    let forged = "HMACSHA256=Z1CmaUMxiI9qmLf4+TAmHvOgDIa4NaVSOLrlEsD47r9=";
    assert!(matches!(client.verify_webhook(&notification_headers(forged), NOTIFICATION), Err(PaymentError::WebhookVerificationFailed(_))));
    // The same signature over a changed body.
    let changed = String::from_utf8(NOTIFICATION.to_vec()).unwrap().replace("150000", "1");
    assert!(client.verify_webhook(&notification_headers(NOTIFICATION_SIGNATURE), changed.as_bytes()).is_err());
    // Another merchant's client id.
    let mut other = notification_headers(NOTIFICATION_SIGNATURE);
    other.insert("client-id", HeaderValue::from_static("BRN-9999"));
    assert!(client.verify_webhook(&other, NOTIFICATION).is_err());
    // No signature at all.
    let mut unsigned = notification_headers(NOTIFICATION_SIGNATURE);
    unsigned.remove("signature");
    assert!(client.verify_webhook(&unsigned, NOTIFICATION).is_err());
}

#[test]
fn signs_against_the_configured_notification_path() {
    let mut cfg = config(None);
    cfg.notification_path = Some("/elsewhere".to_string());
    let client = DokuClient::new(cfg).unwrap();
    assert!(client.verify_webhook(&notification_headers(NOTIFICATION_SIGNATURE), NOTIFICATION).is_err());
}
