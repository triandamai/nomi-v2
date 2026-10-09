
use axum::{body::Body, http::{Request, StatusCode}};
use http_body_util::BodyExt;
use nomi_server::app::{build_router, AppState};
use serde_json::{json, Value};
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;

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


/// Attachments go to disk in tests, in one temp folder per test run.
fn use_temp_attachment_dir() {
    std::env::set_var("ATTACHMENTS_DIR", std::env::temp_dir().join(format!("nomi-test-attachments-{}", std::process::id())));
}

async fn upload(router: axum::Router, token: &str, name: &str, content_type: &str, bytes: Vec<u8>) -> (StatusCode, Value) {
    let request = Request::builder()
        .method("POST")
        .uri("/api/attachments")
        .header("authorization", format!("Bearer {token}"))
        .header("content-type", content_type)
        .header("x-file-name", name)
        .body(Body::from(bytes))
        .unwrap();
    let response = router.oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    (status, serde_json::from_slice(&bytes).unwrap_or(Value::Null))
}

fn png(width: u32, height: u32) -> Vec<u8> {
    let image = image::RgbImage::from_pixel(width, height, image::Rgb([10, 120, 200]));
    let mut out = std::io::Cursor::new(Vec::new());
    image::DynamicImage::ImageRgb8(image).write_to(&mut out, image::ImageFormat::Png).unwrap();
    out.into_inner()
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_text_file_is_read_at_upload_and_an_image_waits_for_the_files_model(pool: PgPool) {
    use_temp_attachment_dir();
    let router = build_router(test_state(pool.clone()));
    let token = register_and_login(router.clone(), "ana@example.com").await;

    let (status, csv) = upload(router.clone(), &token, "spend%20may.csv", "text/csv", b"date,amount\n2026-05-01,12.50\n".to_vec()).await;
    assert_eq!(status, StatusCode::CREATED, "{csv}");
    assert_eq!(csv["name"], "spend may.csv");
    assert_eq!(csv["kind"], "text");
    assert_eq!(csv["status"], "ready");
    assert!(csv["reference"].as_str().unwrap().starts_with("<attachment id=\""));
    let text: String = sqlx::query_scalar("SELECT extracted_text FROM attachments WHERE id = $1")
        .bind(csv["id"].as_str().unwrap().parse::<Uuid>().unwrap())
        .fetch_one(&pool)
        .await
        .unwrap();
    assert!(text.contains("12.50"));

    // The name says .txt, the bytes say PNG: the bytes win.
    let (status, photo) = upload(router.clone(), &token, "receipt.txt", "text/plain", png(2400, 1200)).await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(photo["kind"], "image");
    assert_eq!(photo["mime"], "image/png");
    assert_eq!(photo["status"], "pending");
    let id = photo["id"].as_str().unwrap();

    let request = Request::builder().uri(format!("/api/attachments/{id}/preview")).header("authorization", format!("Bearer {token}")).body(Body::empty()).unwrap();
    let response = router.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()["content-type"], "image/jpeg");

    let request = Request::builder()
        .uri(format!("/api/attachments/{id}/content"))
        .header("authorization", format!("Bearer {token}"))
        .header("range", "bytes=0-7")
        .body(Body::empty())
        .unwrap();
    let response = router.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::PARTIAL_CONTENT);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    assert_eq!(&bytes[..], &png(2400, 1200)[..8]);
}

#[sqlx::test(migrations = "../../migrations")]
async fn programs_and_files_over_the_limit_are_refused(pool: PgPool) {
    use_temp_attachment_dir();
    let router = build_router(test_state(pool));
    let token = register_and_login(router.clone(), "bo@example.com").await;

    let (status, _) = upload(router.clone(), &token, "invoice.pdf", "application/pdf", b"MZ\x90\x00 not really a pdf".to_vec()).await;
    assert_eq!(status, StatusCode::UNSUPPORTED_MEDIA_TYPE);

    let mut big = b"a,b\n".to_vec();
    big.resize(10 * 1024 * 1024 + 1, b'x');
    let (status, _) = upload(router.clone(), &token, "huge.csv", "text/csv", big).await;
    assert_eq!(status, StatusCode::PAYLOAD_TOO_LARGE);

    let (status, _) = upload(router, &token, "empty.txt", "text/plain", Vec::new()).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_message_can_only_point_at_its_senders_own_files(pool: PgPool) {
    use_temp_attachment_dir();
    let router = build_router(test_state(pool.clone()));
    let owner = register_and_login(router.clone(), "cy@example.com").await;
    let stranger = register_and_login(router.clone(), "dee@example.com").await;

    let (_, file) = upload(router.clone(), &owner, "notes.md", "text/markdown", b"# Trip\n- flights".to_vec()).await;
    let reference = file["reference"].as_str().unwrap().to_string();
    let id: Uuid = file["id"].as_str().unwrap().parse().unwrap();

    // Someone else can neither open it nor send it.
    let (status, _) = json_request(router.clone(), "GET", &format!("/api/attachments/{id}"), Value::Null, Some(&stranger)).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (_, theirs) = json_request(router.clone(), "POST", "/api/sessions", Value::Null, Some(&stranger)).await;
    let their_session = theirs["session_id"].as_str().unwrap();
    let (status, _) = json_request(router.clone(), "POST", &format!("/api/sessions/{their_session}/messages"), json!({ "text": reference }), Some(&stranger)).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    // The owner sends it with no text; the tag is rewritten from the stored record.
    let forged = reference.replace("notes.md", "something else.md");
    let (_, mine) = json_request(router.clone(), "POST", "/api/sessions", Value::Null, Some(&owner)).await;
    let session = mine["session_id"].as_str().unwrap();
    let (status, sent) = json_request(router.clone(), "POST", &format!("/api/sessions/{session}/messages"), json!({ "text": forged }), Some(&owner)).await;
    assert_eq!(status, StatusCode::ACCEPTED, "{sent}");
    assert!(sent["user_message"]["content"].as_str().unwrap().contains("name=\"notes.md\""));
    let linked: Option<Uuid> = sqlx::query_scalar("SELECT message_id FROM attachments WHERE id = $1").bind(id).fetch_one(&pool).await.unwrap();
    assert!(linked.is_some());

    // Once sent, it stays with the chat.
    let (status, _) = json_request(router, "DELETE", &format!("/api/attachments/{id}"), Value::Null, Some(&owner)).await;
    assert_eq!(status, StatusCode::CONFLICT);
}
