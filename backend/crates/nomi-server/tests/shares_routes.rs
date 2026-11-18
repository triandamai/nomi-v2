
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
    let mut builder = Request::builder()
        .method(method)
        .uri(uri)
        .header("content-type", "application/json");
    if let Some(token) = bearer {
        builder = builder.header("authorization", format!("Bearer {token}"));
    }
    let response = router
        .oneshot(builder.body(Body::from(body.to_string())).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let json_body = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).unwrap_or(Value::Null)
    };
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
    let (_, login_body) = json_request(
        router,
        "POST",
        "/api/auth/login",
        json!({ "email": email, "password": "correct-password" }),
        None,
    )
    .await;
    login_body["access_token"].as_str().unwrap().to_string()
}

/// Starts a chat the way the app does: create it, then send its first message.
async fn start_chat(router: axum::Router, token: &str) -> String {
    let (_, created) = json_request(router.clone(), "POST", "/api/sessions", Value::Null, Some(token)).await;
    let session_id = created["session_id"].as_str().unwrap().to_string();
    let (status, _) = json_request(router, "POST", &format!("/api/sessions/{session_id}/messages"), json!({ "text": "hi" }), Some(token)).await;
    assert_eq!(status, StatusCode::ACCEPTED);
    session_id
}

/// An agent's reply in the chat, as the crew writes them.
async fn reply(pool: &PgPool, session_id: &str, text: &str) -> Uuid {
    sqlx::query_scalar("INSERT INTO messages (session_id, content, agent_display_name, created_at) VALUES ($1::uuid, $2, 'Dana', now() + interval '1 second') RETURNING id")
        .bind(session_id)
        .bind(text)
        .fetch_one(pool)
        .await
        .unwrap()
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_shared_chat_opens_for_anyone_with_amounts_masked(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register_and_login(router.clone(), "sharer@example.com").await;
    let session_id = start_chat(router.clone(), &token).await;
    reply(&pool, &session_id, "Food came to Rp 1.250.000 this month.").await;

    let (status, share) = json_request(router.clone(), "POST", &format!("/api/sessions/{session_id}/shares"), json!({ "scope": "chat", "expires_in_days": 7 }), Some(&token)).await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(share["mask"], true);
    assert_eq!(share["chat_messages"], 2);
    assert_eq!(share["preview"], "Food came to Rp ••• this month.");
    assert_eq!(share["preview_by"], "Dana");
    assert!(share["expires_at"].is_string());

    let link = format!("/api/shared/{}", share["token"].as_str().unwrap());
    let (status, shared) = json_request(router.clone(), "GET", &link, Value::Null, None).await;
    assert_eq!(status, StatusCode::OK);
    let messages = shared["messages"].as_array().unwrap();
    assert_eq!(messages.len(), 2);
    assert_eq!(messages[0]["sender"], "user");
    assert_eq!(messages[1]["content"], "Food came to Rp ••• this month.");

    // Unmasked, the same link shows the amounts.
    let path = format!("/api/sessions/{session_id}/shares/{}", share["id"].as_str().unwrap());
    let (status, _) = json_request(router.clone(), "PUT", &path, json!({ "scope": "chat", "mask": false }), Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    let (_, shared) = json_request(router, "GET", &link, Value::Null, None).await;
    assert_eq!(shared["messages"][1]["content"], "Food came to Rp 1.250.000 this month.");
    assert!(shared["expires_at"].is_null());
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_shared_reply_shows_only_that_reply(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register_and_login(router.clone(), "reply-sharer@example.com").await;
    let session_id = start_chat(router.clone(), &token).await;
    let message_id = reply(&pool, &session_id, "Here's the plan.").await;

    let (status, _) = json_request(router.clone(), "POST", &format!("/api/sessions/{session_id}/shares"), json!({ "scope": "reply" }), Some(&token)).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let (_, share) = json_request(router.clone(), "POST", &format!("/api/sessions/{session_id}/shares"), json!({ "scope": "reply", "message_id": message_id }), Some(&token)).await;
    let (_, shared) = json_request(router, "GET", &format!("/api/shared/{}", share["token"].as_str().unwrap()), Value::Null, None).await;
    let messages = shared["messages"].as_array().unwrap();
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0]["content"], "Here's the plan.");
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_link_for_signed_in_people_asks_for_a_sign_in(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register_and_login(router.clone(), "private-sharer@example.com").await;
    let reader = register_and_login(router.clone(), "reader@example.com").await;
    let session_id = start_chat(router.clone(), &token).await;

    let (_, share) = json_request(router.clone(), "POST", &format!("/api/sessions/{session_id}/shares"), json!({ "scope": "chat", "anyone": false }), Some(&token)).await;
    let link = format!("/api/shared/{}", share["token"].as_str().unwrap());
    let (status, _) = json_request(router.clone(), "GET", &link, Value::Null, None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    let (status, _) = json_request(router, "GET", &link, Value::Null, Some(&reader)).await;
    assert_eq!(status, StatusCode::OK);
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_revoked_or_expired_link_is_gone_and_only_the_owner_shares(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register_and_login(router.clone(), "owner-share@example.com").await;
    let outsider = register_and_login(router.clone(), "outsider-share@example.com").await;
    let session_id = start_chat(router.clone(), &token).await;
    let shares = format!("/api/sessions/{session_id}/shares");

    let (status, _) = json_request(router.clone(), "POST", &shares, json!({ "scope": "chat" }), Some(&outsider)).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (status, _) = json_request(router.clone(), "POST", &shares, json!({ "scope": "chat", "expires_in_days": 3 }), Some(&token)).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    let (_, revoked) = json_request(router.clone(), "POST", &shares, json!({ "scope": "chat" }), Some(&token)).await;
    let (status, _) = json_request(router.clone(), "DELETE", &format!("{shares}/{}", revoked["id"].as_str().unwrap()), Value::Null, Some(&token)).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (status, _) = json_request(router.clone(), "GET", &format!("/api/shared/{}", revoked["token"].as_str().unwrap()), Value::Null, None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    let (_, expired) = json_request(router.clone(), "POST", &shares, json!({ "scope": "chat", "expires_in_days": 1 }), Some(&token)).await;
    sqlx::query("UPDATE chat_shares SET expires_at = now() - interval '1 minute' WHERE token = $1").bind(expired["token"].as_str().unwrap()).execute(&pool).await.unwrap();
    let (status, _) = json_request(router, "GET", &format!("/api/shared/{}", expired["token"].as_str().unwrap()), Value::Null, None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_person_lists_their_live_links_newest_first(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register_and_login(router.clone(), "lister@example.com").await;
    let other = register_and_login(router.clone(), "someone-else@example.com").await;
    let session_id = start_chat(router.clone(), &token).await;
    let shares = format!("/api/sessions/{session_id}/shares");
    let theirs = start_chat(router.clone(), &other).await;
    json_request(router.clone(), "POST", &format!("/api/sessions/{theirs}/shares"), json!({ "scope": "chat" }), Some(&other)).await;

    let (_, first) = json_request(router.clone(), "POST", &shares, json!({ "scope": "chat", "anyone": false }), Some(&token)).await;
    let (_, revoked) = json_request(router.clone(), "POST", &shares, json!({ "scope": "chat" }), Some(&token)).await;
    json_request(router.clone(), "DELETE", &format!("{shares}/{}", revoked["id"].as_str().unwrap()), Value::Null, Some(&token)).await;
    let (_, expired) = json_request(router.clone(), "POST", &shares, json!({ "scope": "chat", "expires_in_days": 1 }), Some(&token)).await;
    sqlx::query("UPDATE chat_shares SET expires_at = now() - interval '1 minute' WHERE id = $1")
        .bind(Uuid::parse_str(expired["id"].as_str().unwrap()).unwrap())
        .execute(&pool)
        .await
        .unwrap();
    let (_, last) = json_request(router.clone(), "POST", &shares, json!({ "scope": "chat", "expires_in_days": 7 }), Some(&token)).await;

    let (status, body) = json_request(router, "GET", "/api/shares", Value::Null, Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    let ids: Vec<&str> = body["shares"].as_array().unwrap().iter().map(|s| s["id"].as_str().unwrap()).collect();
    assert_eq!(ids, vec![last["id"].as_str().unwrap(), first["id"].as_str().unwrap()]);
    assert_eq!(body["shares"][1]["anyone"], false);
    assert_eq!(body["shares"][1]["session_id"], session_id.as_str());
    assert!(body["shares"][0]["expires_at"].is_string());
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_link_names_its_sharer_and_counts_views_for_its_owner(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register_and_login(router.clone(), "maya.putri@example.com").await;
    let reader = register_and_login(router.clone(), "reader-views@example.com").await;
    sqlx::query(
        "INSERT INTO user_profiles (user_id, display_name) SELECT user_id, 'Maya Putri' FROM web_credentials WHERE email = 'maya.putri@example.com' \
         ON CONFLICT (user_id) DO UPDATE SET display_name = EXCLUDED.display_name",
    )
    .execute(&pool)
    .await
    .unwrap();
    let session_id = start_chat(router.clone(), &token).await;
    let (_, share) = json_request(
        router.clone(),
        "POST",
        &format!("/api/sessions/{session_id}/shares"),
        json!({ "scope": "chat" }),
        Some(&token),
    )
    .await;
    let link = format!("/api/shared/{}", share["token"].as_str().unwrap());

    // Two others open it: one signed out, one signed in. Masked, the sharer reads as initials.
    let (_, shared) = json_request(router.clone(), "GET", &link, Value::Null, None).await;
    assert_eq!(shared["shared_by"], "M. P.");
    assert!(shared["shared_at"].is_string());
    assert!(
        shared.get("own").is_none()
            && shared.get("views").is_none()
            && shared.get("share_id").is_none()
    );
    let (_, shared) = json_request(router.clone(), "GET", &link, Value::Null, Some(&reader)).await;
    assert!(shared.get("own").is_none());

    // The owner sees their banner's facts; their own opens don't count.
    let (_, own) = json_request(router.clone(), "GET", &link, Value::Null, Some(&token)).await;
    assert_eq!(own["own"], true);
    assert_eq!(own["share_id"], share["id"]);
    assert_eq!(own["session_id"], session_id.as_str());
    assert_eq!(own["views"], 2);
    let (_, own) = json_request(router.clone(), "GET", &link, Value::Null, Some(&token)).await;
    assert_eq!(own["views"], 2);

    // Unmasked, the full name shows.
    let path = format!(
        "/api/sessions/{session_id}/shares/{}",
        share["id"].as_str().unwrap()
    );
    json_request(
        router.clone(),
        "PUT",
        &path,
        json!({ "scope": "chat", "mask": false }),
        Some(&token),
    )
    .await;
    let (_, shared) = json_request(router, "GET", &link, Value::Null, None).await;
    assert_eq!(shared["shared_by"], "Maya Putri");
}

#[sqlx::test(migrations = "../../migrations")]
async fn anyone_may_report_a_shared_page_a_few_times_an_hour(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register_and_login(router.clone(), "reported@example.com").await;
    let session_id = start_chat(router.clone(), &token).await;
    let (_, share) = json_request(
        router.clone(),
        "POST",
        &format!("/api/sessions/{session_id}/shares"),
        json!({ "scope": "chat" }),
        Some(&token),
    )
    .await;
    let report = format!("/api/shared/{}/report", share["token"].as_str().unwrap());

    let (status, _) = json_request(
        router.clone(),
        "POST",
        &report,
        json!({ "reason": "boring" }),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let (status, _) = json_request(
        router.clone(),
        "POST",
        "/api/shared/nope/report",
        json!({ "reason": "spam" }),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (status, _) = json_request(
        router.clone(),
        "POST",
        &report,
        json!({ "reason": "spam", "note": "  ads everywhere  " }),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    for _ in 1..5 {
        let (status, _) = json_request(
            router.clone(),
            "POST",
            &report,
            json!({ "reason": "other" }),
            None,
        )
        .await;
        assert_eq!(status, StatusCode::CREATED);
    }
    let (status, _) = json_request(
        router.clone(),
        "POST",
        &report,
        json!({ "reason": "other" }),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
    // A signed-in viewer is counted on their own.
    let (status, _) = json_request(
        router.clone(),
        "POST",
        &report,
        json!({ "reason": "abuse" }),
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);

    // Admins see them; others don't.
    let (status, _) = json_request(
        router.clone(),
        "GET",
        "/api/admin/shared-reports",
        Value::Null,
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    register_and_login(router.clone(), "report-admin@example.com").await;
    sqlx::query("UPDATE users SET is_platform_admin = true WHERE id = (SELECT user_id FROM web_credentials WHERE email = 'report-admin@example.com')")
        .execute(&pool)
        .await
        .unwrap();
    let admin = register_and_login(router.clone(), "report-admin@example.com").await;
    let (status, body) = json_request(
        router,
        "GET",
        "/api/admin/shared-reports",
        Value::Null,
        Some(&admin),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let reports = body["reports"].as_array().unwrap();
    assert_eq!(reports.len(), 6);
    assert_eq!(reports[5]["note"], "ads everywhere");
    assert_eq!(reports[0]["reason"], "abuse");
    assert!(reports[0].get("reporter").is_none());
}

/// A file attached to `message_id` in the chat (the way a sent message carries one).
async fn attach(pool: &PgPool, session_id: &str, message_id: Uuid, name: &str) -> Uuid {
    sqlx::query_scalar(
        "INSERT INTO attachments (user_id, session_id, message_id, name, mime, kind, size_bytes, storage_key, status, details) \
         SELECT s.user_id, s.id, $2, $3, 'application/pdf', 'pdf', 2048, 'attachments/test/' || $3, 'ready', '{\"pages\": 12}'::jsonb FROM sessions s WHERE s.id = $1::uuid RETURNING id",
    )
    .bind(session_id)
    .bind(message_id)
    .bind(name)
    .fetch_one(pool)
    .await
    .unwrap()
}

#[sqlx::test(migrations = "../../migrations")]
async fn files_are_closed_by_default_and_cannot_be_open_on_a_masked_link(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register_and_login(router.clone(), "files@example.com").await;
    let session_id = start_chat(router.clone(), &token).await;
    let shares = format!("/api/sessions/{session_id}/shares");

    let (status, share) = json_request(router.clone(), "POST", &shares, json!({ "scope": "chat" }), Some(&token)).await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(share["files_open"], false);

    // Masking is on unless said otherwise, so asking for open files alone is refused too.
    let (status, body) = json_request(router.clone(), "POST", &shares, json!({ "scope": "chat", "files_open": true }), Some(&token)).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"], "mask_and_files_open");
    let (status, body) = json_request(router.clone(), "POST", &shares, json!({ "scope": "chat", "mask": true, "files_open": true }), Some(&token)).await;
    assert_eq!((status, &body["error"]), (StatusCode::BAD_REQUEST, &json!("mask_and_files_open")));

    let (status, share) = json_request(router.clone(), "POST", &shares, json!({ "scope": "chat", "mask": false, "files_open": true }), Some(&token)).await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(share["files_open"], true);

    // Changing it later: open → closed, and open with a mask is refused.
    let path = format!("{shares}/{}", share["id"].as_str().unwrap());
    let (status, body) = json_request(router.clone(), "PUT", &path, json!({ "scope": "chat", "mask": true, "files_open": true }), Some(&token)).await;
    assert_eq!((status, &body["error"]), (StatusCode::BAD_REQUEST, &json!("mask_and_files_open")));
    let (status, updated) = json_request(router, "PUT", &path, json!({ "scope": "chat", "mask": false }), Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(updated["files_open"], false);
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_shared_chat_lists_its_files_and_gives_urls_only_when_files_are_open(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register_and_login(router.clone(), "lister@example.com").await;
    let session_id = start_chat(router.clone(), &token).await;
    let answer = reply(&pool, &session_id, "Here is the report.").await;
    let file = attach(&pool, &session_id, answer, "report.pdf").await;
    let shares = format!("/api/sessions/{session_id}/shares");

    let (_, closed) = json_request(router.clone(), "POST", &shares, json!({ "scope": "chat", "mask": false }), Some(&token)).await;
    let (_, shared) = json_request(router.clone(), "GET", &format!("/api/shared/{}", closed["token"].as_str().unwrap()), Value::Null, None).await;
    assert_eq!(shared["files_open"], false);
    let files = shared["files"].as_array().unwrap();
    assert_eq!(files.len(), 1);
    assert_eq!((&files[0]["name"], &files[0]["kind"], &files[0]["size"], &files[0]["pages"]), (&json!("report.pdf"), &json!("pdf"), &json!(2048), &json!(12)));
    assert_eq!((files[0]["message_index"].as_u64(), files[0]["made_by_crew"].as_bool()), (Some(1), Some(true)));
    assert!(files[0].get("url").is_none(), "a closed link names its files but never links them");
    let (status, _) = json_request(router.clone(), "GET", &format!("/api/shared/{}/files/{file}", closed["token"].as_str().unwrap()), Value::Null, None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    let (_, open) = json_request(router.clone(), "POST", &shares, json!({ "scope": "chat", "mask": false, "files_open": true }), Some(&token)).await;
    let open_token = open["token"].as_str().unwrap();
    let (_, shared) = json_request(router, "GET", &format!("/api/shared/{open_token}"), Value::Null, None).await;
    assert_eq!(shared["files_open"], true);
    assert_eq!(shared["files"][0]["url"], format!("/api/shared/{open_token}/files/{file}"));
}

#[sqlx::test(migrations = "../../migrations")]
async fn only_files_of_the_shared_messages_open_from_a_link(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register_and_login(router.clone(), "scoped@example.com").await;
    let session_id = start_chat(router.clone(), &token).await;
    let other_chat = start_chat(router.clone(), &token).await;
    let first = reply(&pool, &session_id, "First reply").await;
    let second = reply(&pool, &session_id, "Second reply").await;
    let in_first = attach(&pool, &session_id, first, "first.pdf").await;
    let in_second = attach(&pool, &session_id, second, "second.pdf").await;
    let elsewhere_message = reply(&pool, &other_chat, "Another chat's reply").await;
    let elsewhere = attach(&pool, &other_chat, elsewhere_message, "elsewhere.pdf").await;

    // A link to one reply lists that reply's file only.
    let (_, share) = json_request(router.clone(), "POST", &format!("/api/sessions/{session_id}/shares"), json!({ "scope": "reply", "message_id": first, "mask": false, "files_open": true }), Some(&token)).await;
    let link = share["token"].as_str().unwrap();
    let (_, shared) = json_request(router.clone(), "GET", &format!("/api/shared/{link}"), Value::Null, None).await;
    let names: Vec<&str> = shared["files"].as_array().unwrap().iter().map(|f| f["name"].as_str().unwrap()).collect();
    assert_eq!(names, vec!["first.pdf"]);
    assert_eq!(shared["files"][0]["message_index"], 0);

    // The others — another message of the chat, another chat's file — don't open.
    for hidden in [in_second, elsewhere] {
        let (status, _) = json_request(router.clone(), "GET", &format!("/api/shared/{link}/files/{hidden}"), Value::Null, None).await;
        assert_eq!(status, StatusCode::NOT_FOUND, "{hidden}");
    }
    let _ = in_first;
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_link_for_signed_in_viewers_keeps_its_files_behind_the_sign_in(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register_and_login(router.clone(), "private@example.com").await;
    let session_id = start_chat(router.clone(), &token).await;
    let answer = reply(&pool, &session_id, "Here it is.").await;
    let file = attach(&pool, &session_id, answer, "secret.pdf").await;
    let (_, share) = json_request(router.clone(), "POST", &format!("/api/sessions/{session_id}/shares"), json!({ "scope": "chat", "mask": false, "anyone": false, "files_open": true }), Some(&token)).await;

    let (status, _) = json_request(router, "GET", &format!("/api/shared/{}/files/{file}", share["token"].as_str().unwrap()), Value::Null, None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_file_the_crew_made_in_the_chat_is_listed_and_opens_from_a_link(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let token = register_and_login(router.clone(), "made@example.com").await;
    let session_id = start_chat(router.clone(), &token).await;
    reply(&pool, &session_id, "Here is the sheet.").await;
    // A made file belongs to the chat: it has no message.
    let made: Uuid = sqlx::query_scalar(
        "INSERT INTO attachments (user_id, session_id, name, mime, kind, size_bytes, storage_key, status, made_by, created_at) \
         SELECT s.user_id, s.id, 'rows.csv', 'text/csv', 'csv', 10, 'attachments/test/rows.csv', 'ready', 'files', now() + interval '2 seconds' FROM sessions s WHERE s.id = $1::uuid RETURNING id",
    )
    .bind(&session_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    let (_, share) = json_request(router.clone(), "POST", &format!("/api/sessions/{session_id}/shares"), json!({ "scope": "chat", "mask": false, "files_open": true }), Some(&token)).await;
    let link = share["token"].as_str().unwrap();
    let (_, shared) = json_request(router, "GET", &format!("/api/shared/{link}"), Value::Null, None).await;
    let files = shared["files"].as_array().unwrap();
    assert_eq!(files.len(), 1);
    assert_eq!((files[0]["name"].as_str(), files[0]["made_by_crew"].as_bool()), (Some("rows.csv"), Some(true)));
    assert_eq!(files[0]["url"], format!("/api/shared/{link}/files/{made}"));
}
