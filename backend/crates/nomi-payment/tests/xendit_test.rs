use nomi_payment::types::{CreateInvoiceRequest, CustomerInfo, PaymentStatus};
use nomi_payment::xendit::{XenditClient, XenditConfig};
use nomi_payment::{PaymentError, PaymentGateway};
use serde_json::json;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_create_invoice_success() {
    let mock_server = MockServer::start().await;

    let response_body = json!({
        "id": "inv_test_123",
        "external_id": "order-abc-1",
        "user_id": "user_123",
        "status": "PENDING",
        "merchant_name": "Nomi",
        "amount": 50000,
        "payer_email": "customer@example.com",
        "description": "Subscription Plan",
        "invoice_url": "https://checkout.xendit.co/web/inv_test_123",
        "expiry_date": "2026-11-03T00:00:00.000Z",
        "currency": "IDR"
    });

    // Basic Auth with username "xnd_test_key" and empty password -> "eG5kX3Rlc3Rfa2V5Og=="
    Mock::given(method("POST"))
        .and(path("/v2/invoices"))
        .and(header("Authorization", "Basic eG5kX3Rlc3Rfa2V5Og=="))
        .respond_with(ResponseTemplate::new(200).set_body_json(&response_body))
        .mount(&mock_server)
        .await;

    let config = XenditConfig {
        api_key: "xnd_test_key".to_string(),
        webhook_token: Some("webhook-secret-token".to_string()),
        base_url: Some(mock_server.uri()),
    };

    let client = XenditClient::new(config).unwrap();

    let req = CreateInvoiceRequest {
        external_id: "order-abc-1".to_string(),
        amount: 50000.0,
        description: Some("Subscription Plan".to_string()),
        payer_email: Some("customer@example.com".to_string()),
        invoice_duration: Some(86400),
        customer: Some(CustomerInfo {
            given_names: Some("John".to_string()),
            surname: Some("Doe".to_string()),
            email: Some("customer@example.com".to_string()),
            mobile_number: None,
        }),
        currency: Some("IDR".to_string()),
        success_redirect_url: Some("https://nomi.ai/success".to_string()),
        failure_redirect_url: Some("https://nomi.ai/failure".to_string()),
    };

    let invoice = client.create_invoice(req).await.expect("Failed to create invoice");

    assert_eq!(invoice.id, "inv_test_123");
    assert_eq!(invoice.external_id, "order-abc-1");
    assert_eq!(invoice.amount, 50000.0);
    assert_eq!(invoice.status, PaymentStatus::Pending);
    assert_eq!(invoice.invoice_url, "https://checkout.xendit.co/web/inv_test_123");
    assert_eq!(invoice.currency, "IDR");
}

#[tokio::test]
async fn test_get_invoice_success() {
    let mock_server = MockServer::start().await;

    let response_body = json!({
        "id": "inv_test_123",
        "external_id": "order-abc-1",
        "status": "PAID",
        "amount": 50000,
        "payer_email": "customer@example.com",
        "invoice_url": "https://checkout.xendit.co/web/inv_test_123",
        "paid_at": "2026-11-02T01:00:00.000Z",
        "payment_method": "BANK_TRANSFER",
        "currency": "IDR"
    });

    Mock::given(method("GET"))
        .and(path("/v2/invoices/inv_test_123"))
        .and(header("Authorization", "Basic eG5kX3Rlc3Rfa2V5Og=="))
        .respond_with(ResponseTemplate::new(200).set_body_json(&response_body))
        .mount(&mock_server)
        .await;

    let config = XenditConfig {
        api_key: "xnd_test_key".to_string(),
        webhook_token: None,
        base_url: Some(mock_server.uri()),
    };

    let client = XenditClient::new(config).unwrap();
    let invoice = client.get_invoice("inv_test_123").await.expect("Failed to get invoice");

    assert_eq!(invoice.id, "inv_test_123");
    assert_eq!(invoice.status, PaymentStatus::Paid);
    assert_eq!(invoice.payment_method, Some("BANK_TRANSFER".to_string()));
    assert!(invoice.paid_at.is_some());
}

#[tokio::test]
async fn test_expire_invoice_success() {
    let mock_server = MockServer::start().await;

    let response_body = json!({
        "id": "inv_test_123",
        "external_id": "order-abc-1",
        "status": "EXPIRED",
        "amount": 50000,
        "invoice_url": "https://checkout.xendit.co/web/inv_test_123",
        "currency": "IDR"
    });

    Mock::given(method("POST"))
        .and(path("/invoices/inv_test_123/expire!"))
        .and(header("Authorization", "Basic eG5kX3Rlc3Rfa2V5Og=="))
        .respond_with(ResponseTemplate::new(200).set_body_json(&response_body))
        .mount(&mock_server)
        .await;

    let config = XenditConfig {
        api_key: "xnd_test_key".to_string(),
        webhook_token: None,
        base_url: Some(mock_server.uri()),
    };

    let client = XenditClient::new(config).unwrap();
    let invoice = client.expire_invoice("inv_test_123").await.expect("Failed to expire invoice");

    assert_eq!(invoice.id, "inv_test_123");
    assert_eq!(invoice.status, PaymentStatus::Expired);
}

#[tokio::test]
async fn test_api_error_handling() {
    let mock_server = MockServer::start().await;

    let error_body = json!({
        "error_code": "INVOICE_ALREADY_EXPIRED_ERROR",
        "message": "Invoice already expired"
    });

    Mock::given(method("POST"))
        .and(path("/invoices/inv_test_123/expire!"))
        .respond_with(ResponseTemplate::new(400).set_body_json(&error_body))
        .mount(&mock_server)
        .await;

    let config = XenditConfig {
        api_key: "xnd_test_key".to_string(),
        webhook_token: None,
        base_url: Some(mock_server.uri()),
    };

    let client = XenditClient::new(config).unwrap();
    let result = client.expire_invoice("inv_test_123").await;

    match result {
        Err(PaymentError::BadRequest { status, message }) => {
            assert_eq!(status, 400);
            assert!(message.contains("Invoice already expired"));
        }
        other => panic!("Expected BadRequest error, got {:?}", other),
    }
}

#[tokio::test]
async fn test_webhook_verification() {
    let config = XenditConfig {
        api_key: "xnd_test_key".to_string(),
        webhook_token: Some("valid-token-123".to_string()),
        base_url: None,
    };
    let client = XenditClient::new(config).unwrap();

    let webhook_body = json!({
        "id": "inv_test_123",
        "external_id": "order-abc-1",
        "status": "PAID",
        "amount": 50000,
        "paid_at": "2026-11-02T01:00:00.000Z",
        "payment_method": "EWALLET",
        "payment_channel": "OVO"
    });
    let raw_payload = serde_json::to_vec(&webhook_body).unwrap();

    // Valid token
    let event = client.verify_webhook(Some("valid-token-123"), &raw_payload).expect("Verification failed");
    assert_eq!(event.invoice_id, "inv_test_123");
    assert_eq!(event.external_id, "order-abc-1");
    assert_eq!(event.status, PaymentStatus::Paid);
    assert_eq!(event.payment_method, Some("EWALLET".to_string()));
    assert_eq!(event.payment_channel, Some("OVO".to_string()));

    // Invalid token
    let invalid_res = client.verify_webhook(Some("wrong-token"), &raw_payload);
    assert!(matches!(invalid_res, Err(PaymentError::WebhookVerificationFailed(_))));

    // Missing token
    let missing_res = client.verify_webhook(None, &raw_payload);
    assert!(matches!(missing_res, Err(PaymentError::WebhookVerificationFailed(_))));
}

#[tokio::test]
async fn test_empty_api_key_fails() {
    let config = XenditConfig {
        api_key: "   ".to_string(),
        webhook_token: None,
        base_url: None,
    };
    let res = XenditClient::new(config);
    assert!(matches!(res, Err(PaymentError::Configuration(_))));
}

#[tokio::test]
async fn test_build_payment_gateway_factory() {
    use nomi_payment::config::{build_payment_gateway, PaymentConfig, PaymentProviderKind};

    let config = PaymentConfig {
        provider: PaymentProviderKind::Xendit,
        api_key: "xnd_test_key".to_string(),
        webhook_token: Some("secret".to_string()),
        base_url: None,
    };

    let gateway = build_payment_gateway(config).expect("Factory should build Xendit gateway");
    assert!(gateway.verify_webhook(Some("secret"), b"{\"id\":\"1\",\"external_id\":\"e1\",\"status\":\"PAID\",\"amount\":100}").is_ok());
}

