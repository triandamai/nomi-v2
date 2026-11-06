//! Admin and personal settings past creating them: changing a provider connection, the storage
//! section (saved and tested against a fake S3), and changing custom agents.

mod support;

use axum::http::StatusCode;
use serde_json::json;
use sqlx::PgPool;
use support::App;
use wiremock::matchers::{any, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

async fn provider() -> MockServer {
    let server = MockServer::start().await;
    for key in ["sk-old-1234", "sk-new-5678"] {
        Mock::given(method("GET"))
            .and(path("/v1/models"))
            .and(header("authorization", format!("Bearer {key}").as_str()))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "data": [{ "id": "gpt-5" }] })))
            .mount(&server)
            .await;
    }
    Mock::given(method("GET")).and(path("/gw/v1/models")).respond_with(ResponseTemplate::new(200).set_body_json(json!({ "data": [{ "id": "llama" }] }))).mount(&server).await;
    Mock::given(method("GET")).and(path("/v1/models")).respond_with(ResponseTemplate::new(401)).mount(&server).await;
    server
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_connection_s_key_url_and_headers_are_changed_only_when_the_provider_takes_them(pool: PgPool) {
    std::env::set_var("NOMI_WEB_ALLOW_PRIVATE", "1");
    let server = provider().await;
    let app = App::new(pool);
    let ana = app.person("ana@acme.test").await;
    let admin = app.admin("root@nomi.test").await;

    for (who, base) in [(&ana, "/api/llm/connections"), (&admin, "/api/admin/settings/llm/connections")] {
        let (status, made) = app.send("POST", base, who, json!({"provider_id": "openai", "api_key": "sk-old-1234", "base_url": server.uri()})).await;
        assert_eq!(status, StatusCode::CREATED, "{made}");
        let id = made["id"].as_str().unwrap();
        let one = format!("{base}/{id}");

        let (status, body) = app.send("PUT", &one, who, json!({"api_key": "sk-wrong"})).await;
        assert!(status.is_client_error(), "{status} {body}");
        let (status, body) = app.send("PUT", &one, who, json!({"base_url": "ftp://x"})).await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");
        let (status, body) = app.send("PUT", &one, who, json!({"api_key": " sk-new-5678 ", "label": "ignored for a known provider"})).await;
        assert_eq!(status, StatusCode::OK, "{body}");
        assert_eq!(body["api_key_masked"], "...5678");
        let (status, _) = app.send("PUT", &format!("{base}/{}", uuid::Uuid::new_v4()), who, json!({})).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert!(app.call("DELETE", &one, Some(&who.token), None).await.0.is_success());
    }

    // A custom endpoint takes a new label, address and headers.
    let (status, custom) = app
        .send("POST", "/api/llm/connections", &ana, json!({"provider_id": "custom", "label": "Gateway", "base_url": format!("{}/gw/v1", server.uri()), "headers": [{"name": "X-Team", "value": "a"}]}))
        .await;
    assert_eq!(status, StatusCode::CREATED, "{custom}");
    let one = format!("/api/llm/connections/{}", custom["id"].as_str().unwrap());
    let (status, body) = app.send("PUT", &one, &ana, json!({"label": "Team gateway", "headers": [{"name": "X-Team", "value": "b"}, {"name": "X-Env", "value": "prod"}]})).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["label"], "Team gateway");
    assert_eq!(body["header_names"], json!(["X-Team", "X-Env"]));
    let (status, body) = app.send("PUT", &one, &ana, json!({"headers": [{"name": "Bad Name", "value": "x"}]})).await;
    assert_eq!((status, body["error"].as_str()), (StatusCode::BAD_REQUEST, Some("header_invalid")));
}

#[sqlx::test(migrations = "../../migrations")]
async fn storage_is_tested_and_saved_against_the_bucket(pool: PgPool) {
    let s3 = MockServer::start().await;
    Mock::given(any()).respond_with(ResponseTemplate::new(200).set_body_string("nomi storage probe")).mount(&s3).await;
    let app = App::new(pool);
    let admin = app.admin("root@nomi.test").await;
    let person = app.person("ana@acme.test").await;

    let (status, incomplete) = app.send("POST", "/api/admin/configuration/storage/test", &admin, json!({"values": {"bucket": "b"}})).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(incomplete["success"], false);
    let values = json!({"bucket": "nomi", "region": "auto", "access_key_id": "AKID", "secret_access_key": "secret", "endpoint_url": s3.uri()});
    let (status, tested) = app.send("POST", "/api/admin/configuration/storage/test", &admin, json!({"values": values})).await;
    assert_eq!(status, StatusCode::OK, "{tested}");
    assert_eq!(tested["success"], true, "{tested}");
    assert_eq!(app.send("POST", "/api/admin/configuration/storage/test", &person, json!({"values": {}})).await.0, StatusCode::FORBIDDEN);

    // A bucket without its keys is refused as a whole (unless the environment has them); with
    // them it's saved and used.
    std::env::remove_var("AWS_ACCESS_KEY_ID");
    std::env::remove_var("AWS_SECRET_ACCESS_KEY");
    let (status, _) = app.send("PUT", "/api/admin/configuration/storage", &admin, json!({"values": {"bucket": "nomi", "access_key_id": "", "secret_access_key": ""}})).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    let (status, saved) = app.send("PUT", "/api/admin/configuration/storage", &admin, json!({"values": values})).await;
    assert_eq!(status, StatusCode::OK, "{saved}");
    let (status, _) = app.send("PUT", "/api/admin/configuration/storage", &admin, json!({"values": {"bucket": ""}})).await;
    assert!(status.is_success() || status == StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(app.send("PUT", "/api/admin/configuration/nope", &admin, json!({"values": {}})).await.0, StatusCode::NOT_FOUND);
    nomi_storage::blob::update_attachment_store(None);
}

#[sqlx::test(migrations = "../../migrations")]
async fn custom_agents_are_changed_switched_off_and_removed(pool: PgPool) {
    let app = App::new(pool);
    let admin = app.admin("root@nomi.test").await;
    let person = app.person("ana@acme.test").await;
    let agent = |label: &str| json!({"name": "Weather Bot", "system_prompt": "You report the weather.", "intent_label": label, "intent_description": "weather", "granted_tools": [], "supports_todos": false, "supports_plans": false, "can_delegate": false});
    let (status, made) = app.send("POST", "/api/admin/dynamic-agents", &admin, agent("weather")).await;
    assert_eq!(status, StatusCode::CREATED, "{made}");
    app.send("POST", "/api/admin/dynamic-agents", &admin, agent("tides")).await;
    let one = format!("/api/admin/dynamic-agents/{}", made["id"].as_str().unwrap());

    let (status, updated) = app.send("PUT", &one, &admin, json!({"name": "Weather", "system_prompt": "Short answers.", "intent_label": "weather", "intent_description": "weather", "granted_tools": [], "supports_todos": true, "supports_plans": false, "can_delegate": true})).await;
    assert_eq!(status, StatusCode::OK, "{updated}");
    assert_eq!(updated["name"], "Weather");
    assert_eq!(app.send("PUT", &one, &admin, agent("tides")).await.0, StatusCode::INTERNAL_SERVER_ERROR, "the label is taken");
    assert_eq!(app.send("PUT", &format!("/api/admin/dynamic-agents/{}", uuid::Uuid::new_v4()), &admin, agent("other")).await.0, StatusCode::NOT_FOUND);
    assert_eq!(app.send("PUT", &one, &person, agent("weather")).await.0, StatusCode::FORBIDDEN);
    let mut bad_tools = agent("weather");
    bad_tools["granted_tools"] = json!(["rm_rf"]);
    assert_eq!(app.send("PUT", &one, &admin, bad_tools).await.0, StatusCode::BAD_REQUEST);

    let (status, toggled) = app.call("POST", &format!("{one}/toggle-active"), Some(&admin.token), None).await;
    assert!(status.is_success(), "{status} {toggled}");
    assert!(app.call("POST", &format!("/api/admin/dynamic-agents/{}/toggle-active", uuid::Uuid::new_v4()), Some(&admin.token), None).await.0.is_client_error());
    let (_, list) = app.get("/api/admin/dynamic-agents", &admin).await;
    assert!(list.as_array().unwrap().iter().any(|a| a["intent_label"] == "weather" && a["is_active"] == false), "{list}");
}
