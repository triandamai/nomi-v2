
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
        push: std::sync::Arc::new(nomi_server::push::NoopPush),
        shipyard: None,
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

#[sqlx::test(migrations = "../../migrations")]
async fn unsent_uploads_are_swept_after_six_hours_and_sent_ones_stay(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    register_and_login(router, "sweep@example.com").await;
    let user_id: Uuid = sqlx::query_scalar("SELECT user_id FROM web_credentials WHERE email = 'sweep@example.com'").fetch_one(&pool).await.unwrap();
    let session_id: Uuid = sqlx::query_scalar(
        "INSERT INTO sessions (org_id, channel, chat_id) SELECT org_id, 'web', 'sweep' FROM memberships WHERE user_id = $1 LIMIT 1 RETURNING id",
    )
    .bind(user_id)
    .fetch_one(&pool)
        .await
        .unwrap();
    let message_id: Uuid = sqlx::query_scalar("INSERT INTO messages (session_id, content) VALUES ($1, 'here') RETURNING id")
        .bind(session_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    let add = |name: &'static str, hours_old: i32, sent: bool| {
        let pool = pool.clone();
        async move {
            sqlx::query(
                "INSERT INTO attachments (user_id, message_id, name, mime, kind, size_bytes, storage_key, status, created_at) \
                 VALUES ($1, $2, $3, 'text/plain', 'text', 1, $3, 'ready', now() - make_interval(hours => $4))",
            )
            .bind(user_id)
            .bind(sent.then_some(message_id))
            .bind(name)
            .bind(hours_old)
            .execute(&pool)
            .await
            .unwrap();
        }
    };
    add("old-draft.txt", 7, false).await;
    add("fresh-draft.txt", 1, false).await;
    add("old-sent.txt", 30, true).await;

    nomi_server::attachment_worker::pass(&pool, &[7u8; 32], &reqwest::Client::new()).await.unwrap();

    let mut left: Vec<String> = sqlx::query_scalar("SELECT name FROM attachments").fetch_all(&pool).await.unwrap();
    left.sort();
    assert_eq!(left, vec!["fresh-draft.txt".to_string(), "old-sent.txt".to_string()]);
}

async fn raw_upload(router: axum::Router, token: &str, name: &str, body: Body) -> (StatusCode, Value) {
    let request = Request::builder()
        .method("POST")
        .uri("/api/attachments")
        .header("authorization", format!("Bearer {token}"))
        .header("content-type", "application/octet-stream")
        .header("x-file-name", name)
        .body(body)
        .unwrap();
    let response = router.oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    (status, serde_json::from_slice(&bytes).unwrap_or(Value::Null))
}

#[sqlx::test(migrations = "../../migrations")]
async fn upload_failures_carry_a_code_the_composer_can_act_on(pool: PgPool) {
    use_temp_attachment_dir();
    let router = build_router(test_state(pool.clone()));
    let token = register_and_login(router.clone(), "code@example.com").await;

    // Too big: the code, a sentence, and the biggest a file of that kind may be.
    let mut big = b"a,b\n".to_vec();
    big.resize(10 * 1024 * 1024 + 1, b'x');
    let (status, body) = upload(router.clone(), &token, "huge.csv", "text/csv", big).await;
    assert_eq!(status, StatusCode::PAYLOAD_TOO_LARGE);
    assert_eq!(body["error"], "too_large", "{body}");
    assert_eq!(body["max_bytes"], 10 * 1024 * 1024);
    assert!(body["message"].as_str().is_some_and(|m| !m.is_empty()));

    // A program: the code and the extension that was refused.
    let (status, body) = upload(router.clone(), &token, "Setup.DMG", "application/octet-stream", b"MZ\x90\x00 program".to_vec()).await;
    assert_eq!(status, StatusCode::UNSUPPORTED_MEDIA_TYPE);
    assert_eq!(body["error"], "unsupported_type", "{body}");
    assert_eq!(body["extension"], ".dmg");

    // A connection that drops part-way.
    let chunks: Vec<Result<Vec<u8>, std::io::Error>> = vec![Ok(b"a,b\n1,2\n".to_vec()), Err(std::io::Error::other("connection reset"))];
    let (status, body) = raw_upload(router.clone(), &token, "cut.csv", Body::from_stream(futures_util::stream::iter(chunks))).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"], "interrupted", "{body}");

    // A full store.
    let user_id: Uuid = sqlx::query_scalar("SELECT user_id FROM web_credentials WHERE email = 'code@example.com'").fetch_one(&pool).await.unwrap();
    sqlx::query(
        "INSERT INTO attachments (user_id, name, mime, kind, size_bytes, storage_key, status) VALUES ($1, 'old.bin', 'text/plain', 'text', $2, 'old', 'ready')",
    )
    .bind(user_id)
    .bind(2048_i64 * 1024 * 1024 + 1)
    .execute(&pool)
    .await
    .unwrap();
    let (status, body) = upload(router, &token, "one.csv", "text/csv", b"a,b\n1,2\n".to_vec()).await;
    assert_eq!(status, StatusCode::INSUFFICIENT_STORAGE);
    assert_eq!(body["error"], "storage_full", "{body}");
}

#[sqlx::test(migrations = "../../migrations")]
async fn limits_list_what_can_be_attached(pool: PgPool) {
    let router = build_router(test_state(pool));
    let token = register_and_login(router.clone(), "types@example.com").await;
    let (status, body) = json_request(router, "GET", "/api/attachments/limits", Value::Null, Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    let list = |key: &str| body["accepted"][key].as_array().unwrap_or_else(|| panic!("accepted.{key} in {body}")).iter().map(|v| v.as_str().unwrap().to_string()).collect::<Vec<_>>();
    let kinds = list("kinds");
    for kind in ["pdf", "image", "spreadsheet", "archive"] {
        assert!(kinds.iter().any(|k| k == kind), "{kind} in {kinds:?}");
    }
    let extensions = list("extensions");
    assert!(extensions.iter().all(|e| e.starts_with('.')));
    assert!(extensions.iter().any(|e| e == ".pdf") && extensions.iter().any(|e| e == ".jpg") && extensions.iter().any(|e| e == ".zip"));
    assert!(!extensions.iter().any(|e| e == ".exe"));
    assert!(list("mimes").iter().any(|m| m == "application/pdf"));
}

#[sqlx::test(migrations = "../../migrations")]
async fn files_left_out_of_a_message_stay_on_it_and_the_crew_is_told(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register_and_login(router.clone(), "leftout@example.com").await;
    let (_, mine) = json_request(router.clone(), "POST", "/api/sessions", Value::Null, Some(&token)).await;
    let session = mine["session_id"].as_str().unwrap();
    let uri = format!("/api/sessions/{session}/messages");

    let left_out = json!([{ "name": "struk-belanja-nov.jpg", "reason": "model_cannot_see", "model_label": "deepseek-v3" }]);
    let (status, sent) = json_request(router.clone(), "POST", &uri, json!({ "text": "ini struknya", "left_out": left_out }), Some(&token)).await;
    assert_eq!(status, StatusCode::ACCEPTED, "{sent}");
    assert_eq!(sent["user_message"]["content"], "ini struknya");
    assert_eq!(sent["user_message"]["left_out"], left_out);

    // It's still there when the chat is loaded again.
    let (_, listed) = json_request(router.clone(), "GET", &uri, Value::Null, Some(&token)).await;
    assert_eq!(listed["messages"][0]["left_out"], left_out, "{listed}");

    // The crew reads a line naming the file and why.
    let session_id: Uuid = session.parse().unwrap();
    let mut conn = pool.acquire().await.unwrap();
    let history = nomi_turn::fetch_recent_messages(&mut conn, session_id).await.unwrap();
    let text = format!("{:?}", history.last().unwrap().content);
    assert!(text.contains("struk-belanja-nov.jpg") && text.contains("deepseek-v3"), "{text}");

    // A reason the contract doesn't know is refused.
    let bad = json!([{ "name": "a.jpg", "reason": "because", "model_label": "x" }]);
    let (status, _) = json_request(router.clone(), "POST", &uri, json!({ "text": "hi", "left_out": bad }), Some(&token)).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);

    // A message without any keeps none.
    let (status, plain) = json_request(router, "POST", &uri, json!({ "text": "hello" }), Some(&token)).await;
    assert_eq!(status, StatusCode::ACCEPTED);
    assert!(plain["user_message"].get("left_out").is_none());
}

async fn upload_replacing(router: axum::Router, token: &str, name: &str, replaces: &str, bytes: Vec<u8>) -> (StatusCode, Value) {
    let request = Request::builder()
        .method("POST")
        .uri("/api/attachments")
        .header("authorization", format!("Bearer {token}"))
        .header("content-type", "text/plain")
        .header("x-file-name", name)
        .header("x-replaces", replaces)
        .body(Body::from(bytes))
        .unwrap();
    let response = router.oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    (status, serde_json::from_slice(&bytes).unwrap_or(Value::Null))
}

async fn get(router: axum::Router, token: &str, uri: &str) -> (StatusCode, Value) {
    let request = Request::builder().uri(uri).header("authorization", format!("Bearer {token}")).body(Body::empty()).unwrap();
    let response = router.oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    (status, serde_json::from_slice(&bytes).unwrap_or(Value::Null))
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_file_says_how_it_was_read_and_which_file_it_replaces(pool: PgPool) {
    use_temp_attachment_dir();
    let router = build_router(test_state(pool.clone()));
    let ana = register_and_login(router.clone(), "ana@example.com").await;
    let ben = register_and_login(router.clone(), "ben@example.com").await;

    let (_, old) = upload(router.clone(), &ana, "bca-agustus.txt", "text/plain", b"Saldo Rp 1.200.000".to_vec()).await;
    let old_id = old["id"].as_str().unwrap().to_string();
    let (_, shown) = get(router.clone(), &ana, &format!("/api/attachments/{old_id}")).await;
    assert_eq!(shown["read"]["result"], "ok", "{shown}");
    assert_eq!(shown["replaces"], Value::Null);

    let (status, new) = upload_replacing(router.clone(), &ana, "bca-september.txt", &old_id, b"Saldo Rp 900.000".to_vec()).await;
    assert_eq!(status, StatusCode::CREATED, "{new}");
    assert_eq!(new["replaces"], old_id);
    let (_, shown) = get(router.clone(), &ana, &format!("/api/attachments/{}", new["id"].as_str().unwrap())).await;
    assert_eq!(shown["replaces"], old_id);

    // Someone else's file, or something that isn't an id, can't be replaced.
    let (status, _) = upload_replacing(router.clone(), &ben, "x.txt", &old_id, b"hi".to_vec()).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (status, _) = upload_replacing(router.clone(), &ana, "x.txt", "nope", b"hi".to_vec()).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    // A photo waiting for the files model has no result yet; one that failed is damaged.
    let (_, photo) = upload(router.clone(), &ana, "struk.png", "image/png", png(40, 40)).await;
    let photo_id = photo["id"].as_str().unwrap().to_string();
    let (_, shown) = get(router.clone(), &ana, &format!("/api/attachments/{photo_id}")).await;
    assert_eq!(shown["read"], Value::Null);
    sqlx::query("UPDATE attachments SET status = 'failed' WHERE id = $1").bind(photo_id.parse::<Uuid>().unwrap()).execute(&pool).await.unwrap();
    let (_, shown) = get(router, &ana, &format!("/api/attachments/{photo_id}")).await;
    assert_eq!(shown["read"]["result"], "damaged");
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_estimate_prices_files_for_nomis_models_and_is_free_of_cost_on_the_persons_key(pool: PgPool) {
    use_temp_attachment_dir();
    let router = build_router(test_state(pool.clone()));
    let ana = register_and_login(router.clone(), "ana@example.com").await;
    let ben = register_and_login(router.clone(), "ben@example.com").await;
    let ana_id: Uuid = sqlx::query_scalar("SELECT user_id FROM web_credentials WHERE email = 'ana@example.com'").fetch_one(&pool).await.unwrap();
    let model: Uuid = sqlx::query_scalar(
        "INSERT INTO admin_llm_models (label, provider, model_id, updated_by, input_usd_per_mtok, output_usd_per_mtok) VALUES ('Claude Sonnet', 'fake', 'sonnet', $1, 3, 15) RETURNING id",
    )
    .bind(ana_id)
    .fetch_one(&pool)
    .await
    .unwrap();

    let (_, text) = upload(router.clone(), &ana, "notes.txt", "text/plain", "x".repeat(4_000).into_bytes()).await;
    let (_, photo) = upload(router.clone(), &ana, "struk.png", "image/png", png(40, 40)).await;
    let ids = json!([text["id"], photo["id"]]);

    let (status, out) = json_request(router.clone(), "POST", "/api/attachments/estimate", json!({ "attachment_ids": ids, "model_key": format!("admin:{model}") }), Some(&ana)).await;
    assert_eq!(status, StatusCode::OK, "{out}");
    assert_eq!(out["files"][0]["tokens"], 1_000);
    assert_eq!(out["files"][1]["tokens"], 1_600);
    assert_eq!(out["tokens"], 2_600);
    assert_eq!(out["payer"], "credits");
    assert_eq!(out["cost_idr"], 125, "2,600 tokens at $3 per million, Rp 16,000 to the dollar, rounded up: {out}");
    assert!(out["threshold_tokens"].as_u64().unwrap() > 0);
    assert!(out["plan"]["after_pct"].as_u64().is_some(), "{out}");

    let (status, _) = json_request(router.clone(), "POST", "/api/attachments/estimate", json!({ "attachment_ids": ids, "model_key": format!("admin:{model}") }), Some(&ben)).await;
    assert_eq!(status, StatusCode::NOT_FOUND, "someone else's files");
    let (status, _) = json_request(router, "POST", "/api/attachments/estimate", json!({ "attachment_ids": ids, "model_key": "nope" }), Some(&ana)).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

async fn open_upload(router: axum::Router, token: &str, name: &str, length: Option<u64>) -> (StatusCode, Value) {
    let mut builder = Request::builder()
        .method("POST")
        .uri("/api/attachments/uploads")
        .header("authorization", format!("Bearer {token}"))
        .header("content-type", "text/csv")
        .header("x-file-name", name);
    if let Some(length) = length {
        builder = builder.header("x-upload-length", length.to_string());
    }
    let response = router.oneshot(builder.body(Body::empty()).unwrap()).await.unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    (status, serde_json::from_slice(&bytes).unwrap_or(Value::Null))
}

async fn send_piece(router: axum::Router, token: &str, upload_id: &str, from: usize, bytes: &[u8], total: usize) -> (StatusCode, Value) {
    let request = Request::builder()
        .method("PATCH")
        .uri(format!("/api/attachments/uploads/{upload_id}"))
        .header("authorization", format!("Bearer {token}"))
        .header("content-range", format!("bytes {from}-{}/{total}", from + bytes.len() - 1))
        .body(Body::from(bytes.to_vec()))
        .unwrap();
    let response = router.oneshot(request).await.unwrap();
    let status = response.status();
    let body = response.into_body().collect().await.unwrap().to_bytes();
    (status, serde_json::from_slice(&body).unwrap_or(Value::Null))
}

async fn upload_offset(router: axum::Router, token: &str, upload_id: &str) -> (StatusCode, Option<u64>) {
    let request = Request::builder()
        .method("HEAD")
        .uri(format!("/api/attachments/uploads/{upload_id}"))
        .header("authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap();
    let response = router.oneshot(request).await.unwrap();
    let offset = response.headers().get("upload-offset").and_then(|v| v.to_str().ok()).and_then(|v| v.parse().ok());
    (response.status(), offset)
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_file_sent_in_pieces_resumes_where_it_stopped_and_ends_as_a_normal_attachment(pool: PgPool) {
    use_temp_attachment_dir();
    let router = build_router(test_state(pool.clone()));
    let token = register_and_login(router.clone(), "chunks@example.com").await;
    let csv = b"date,amount\n2026-05-01,12.50\n2026-05-02,8.00\n".to_vec();

    let (status, opened) = open_upload(router.clone(), &token, "spend.csv", Some(csv.len() as u64)).await;
    assert_eq!(status, StatusCode::CREATED, "{opened}");
    assert_eq!(opened["offset"], 0);
    let id = opened["upload_id"].as_str().unwrap().to_string();

    let (status, first) = send_piece(router.clone(), &token, &id, 0, &csv[..20], csv.len()).await;
    assert_eq!(status, StatusCode::OK, "{first}");
    assert_eq!(first["offset"], 20);

    // After a drop the client asks where the upload stands.
    let (status, offset) = upload_offset(router.clone(), &token, &id).await;
    assert_eq!((status, offset), (StatusCode::OK, Some(20)));

    // A piece from the wrong place is a 409 with the offset to pick up at.
    let (status, wrong) = send_piece(router.clone(), &token, &id, 30, &csv[30..], csv.len()).await;
    assert_eq!(status, StatusCode::CONFLICT, "{wrong}");
    assert_eq!(wrong["offset"], 20);
    assert_eq!(wrong["error"], "offset_mismatch");

    // The last piece answers what a single upload does.
    let (status, done) = send_piece(router.clone(), &token, &id, 20, &csv[20..], csv.len()).await;
    assert_eq!(status, StatusCode::CREATED, "{done}");
    assert_eq!(done["name"], "spend.csv");
    assert_eq!(done["kind"], "text");
    assert_eq!(done["status"], "ready");
    assert_eq!(done["size"], csv.len());
    assert!(done["reference"].as_str().unwrap().starts_with("<attachment id=\""));
    let text: String = sqlx::query_scalar("SELECT extracted_text FROM attachments WHERE id = $1")
        .bind(done["id"].as_str().unwrap().parse::<Uuid>().unwrap())
        .fetch_one(&pool)
        .await
        .unwrap();
    assert!(text.contains("12.50"));

    // The upload is over.
    let (status, _) = upload_offset(router, &token, &id).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[sqlx::test(migrations = "../../migrations")]
async fn pieces_are_held_to_the_same_size_limit_and_to_their_owner(pool: PgPool) {
    use_temp_attachment_dir();
    let router = build_router(test_state(pool.clone()));
    let owner = register_and_login(router.clone(), "own-chunks@example.com").await;
    let stranger = register_and_login(router.clone(), "other-chunks@example.com").await;

    // Too big is refused at once when the length is known, and at the first piece when it isn't.
    let over = 10 * 1024 * 1024 + 1;
    let (status, body) = open_upload(router.clone(), &owner, "huge.csv", Some(over as u64 * 10)).await;
    assert_eq!(status, StatusCode::PAYLOAD_TOO_LARGE, "{body}");
    assert_eq!(body["error"], "too_large");
    let (_, opened) = open_upload(router.clone(), &owner, "huge.csv", None).await;
    let id = opened["upload_id"].as_str().unwrap().to_string();
    let (status, body) = send_piece(router.clone(), &owner, &id, 0, b"a,b\n", over * 10).await;
    assert_eq!(status, StatusCode::PAYLOAD_TOO_LARGE, "{body}");
    // The kind isn't known before the last piece, so the most any upload may be (the video limit) holds.
    assert_eq!(body["max_bytes"], 50 * 1024 * 1024);
    assert_eq!(upload_offset(router.clone(), &owner, &id).await.0, StatusCode::NOT_FOUND);

    // Someone else's upload is not theirs to see or add to.
    let (_, opened) = open_upload(router.clone(), &owner, "mine.csv", None).await;
    let id = opened["upload_id"].as_str().unwrap().to_string();
    assert_eq!(upload_offset(router.clone(), &stranger, &id).await.0, StatusCode::NOT_FOUND);
    let (status, _) = send_piece(router, &stranger, &id, 0, b"a,b\n", 8).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_last_piece_that_is_a_program_is_refused_like_a_single_upload(pool: PgPool) {
    use_temp_attachment_dir();
    let router = build_router(test_state(pool.clone()));
    let token = register_and_login(router.clone(), "exe-chunks@example.com").await;
    let (_, opened) = open_upload(router.clone(), &token, "Setup.DMG", None).await;
    let id = opened["upload_id"].as_str().unwrap().to_string();
    let program = b"MZ\x90\x00 program";
    let (status, body) = send_piece(router.clone(), &token, &id, 0, program, program.len()).await;
    assert_eq!(status, StatusCode::UNSUPPORTED_MEDIA_TYPE, "{body}");
    assert_eq!(body["error"], "unsupported_type");
    assert_eq!(upload_offset(router, &token, &id).await.0, StatusCode::NOT_FOUND);
}

#[sqlx::test(migrations = "../../migrations")]
async fn expired_uploads_are_swept_and_fresh_ones_stay(pool: PgPool) {
    use_temp_attachment_dir();
    let router = build_router(test_state(pool.clone()));
    let token = register_and_login(router.clone(), "expire-chunks@example.com").await;
    let (_, old) = open_upload(router.clone(), &token, "old.csv", None).await;
    let (_, fresh) = open_upload(router.clone(), &token, "fresh.csv", None).await;
    let old_id: Uuid = old["upload_id"].as_str().unwrap().parse().unwrap();
    sqlx::query("UPDATE attachment_uploads SET expires_at = now() - interval '1 minute' WHERE id = $1").bind(old_id).execute(&pool).await.unwrap();

    // An expired upload can't be picked up even before the sweep.
    assert_eq!(upload_offset(router.clone(), &token, &old_id.to_string()).await.0, StatusCode::NOT_FOUND);
    nomi_server::attachment_worker::pass(&pool, &[7u8; 32], &reqwest::Client::new()).await.unwrap();

    let left: Vec<String> = sqlx::query_scalar("SELECT name FROM attachment_uploads").fetch_all(&pool).await.unwrap();
    assert_eq!(left, vec!["fresh.csv".to_string()]);
    assert_eq!(upload_offset(router, &token, fresh["upload_id"].as_str().unwrap()).await.0, StatusCode::OK);
}

async fn get_status(router: axum::Router, uri: &str, token: &str) -> (StatusCode, Value) {
    let request = Request::builder().method("GET").uri(uri).header("authorization", format!("Bearer {token}")).body(Body::empty()).unwrap();
    let response = router.oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    (status, serde_json::from_slice(&bytes).unwrap_or(Value::Null))
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_page_is_404_past_the_last_and_501_when_it_cannot_be_drawn(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register_and_login(router.clone(), "pages@example.com").await;
    let other = register_and_login(router.clone(), "pages-other@example.com").await;
    let user_id: Uuid = sqlx::query_scalar("SELECT user_id FROM web_credentials WHERE email = 'pages@example.com'").fetch_one(&pool).await.unwrap();
    let pdf: Uuid = sqlx::query_scalar(
        "INSERT INTO attachments (user_id, name, mime, kind, size_bytes, storage_key, status, details) VALUES ($1, 'statement.pdf', 'application/pdf', 'pdf', 10, 'k', 'ready', '{\"pages\": 3}') RETURNING id",
    )
    .bind(user_id)
    .fetch_one(&pool)
    .await
    .unwrap();

    let (status, _) = get_status(router.clone(), &format!("/api/attachments/{pdf}/pages/4"), &token).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (status, _) = get_status(router.clone(), &format!("/api/attachments/{pdf}/pages/0"), &token).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (status, body) = get_status(router.clone(), &format!("/api/attachments/{pdf}/pages/2"), &token).await;
    assert_eq!(status, StatusCode::NOT_IMPLEMENTED);
    assert_eq!(body["error"], "page_render_unavailable");
    let (status, _) = get_status(router, &format!("/api/attachments/{pdf}/pages/2"), &other).await;
    assert_eq!(status, StatusCode::NOT_FOUND, "someone else's file");
}

#[sqlx::test(migrations = "../../migrations")]
async fn an_images_only_page_is_its_preview(pool: PgPool) {
    use_temp_attachment_dir();
    let router = build_router(test_state(pool.clone()));
    let token = register_and_login(router.clone(), "imagepage@example.com").await;
    let (status, photo) = upload(router.clone(), &token, "receipt.png", "image/png", png(2400, 1200)).await;
    assert_eq!(status, StatusCode::CREATED);
    let id = photo["id"].as_str().unwrap();
    let request = Request::builder().method("GET").uri(format!("/api/attachments/{id}/pages/1")).header("authorization", format!("Bearer {token}")).body(Body::empty()).unwrap();
    let response = router.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()["content-type"], "image/jpeg");
    let (status, _) = get_status(router, &format!("/api/attachments/{id}/pages/2"), &token).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_file_the_crew_made_is_not_swept_or_removed_as_an_unsent_draft(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register_and_login(router.clone(), "keeper@example.com").await;
    let user_id: Uuid = sqlx::query_scalar("SELECT user_id FROM web_credentials WHERE email = 'keeper@example.com'").fetch_one(&pool).await.unwrap();
    let made: Uuid = sqlx::query_scalar(
        "INSERT INTO attachments (user_id, name, mime, kind, size_bytes, storage_key, status, made_by, created_at) \
         VALUES ($1, 'made.csv', 'text/csv', 'text', 1, 'made.csv', 'ready', 'files', now() - interval '30 hours') RETURNING id",
    )
    .bind(user_id)
    .fetch_one(&pool)
    .await
    .unwrap();

    nomi_server::attachment_worker::pass(&pool, &[7u8; 32], &reqwest::Client::new()).await.unwrap();
    let still: i64 = sqlx::query_scalar("SELECT count(*) FROM attachments WHERE id = $1").bind(made).fetch_one(&pool).await.unwrap();
    assert_eq!(still, 1, "the sweep leaves made files");

    let response = router
        .oneshot(Request::builder().method("DELETE").uri(format!("/api/attachments/{made}")).header("authorization", format!("Bearer {token}")).body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CONFLICT);
    let still: i64 = sqlx::query_scalar("SELECT count(*) FROM attachments WHERE id = $1").bind(made).fetch_one(&pool).await.unwrap();
    assert_eq!(still, 1);
}
