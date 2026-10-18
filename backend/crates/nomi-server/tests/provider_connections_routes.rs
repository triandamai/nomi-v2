use axum::{body::Body, http::{Request, StatusCode}};
use http_body_util::BodyExt;
use nomi_server::app::{build_router, AppState};
use serde_json::{json, Value};
use sqlx::PgPool;
use tower::ServiceExt;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const SECRET: &str = "test-secret-do-not-use-in-prod";

fn test_state(pool: PgPool) -> AppState {
    // The fake providers below listen on 127.0.0.1, which connections otherwise refuse.
    std::env::set_var("NOMI_WEB_ALLOW_PRIVATE", "1");
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

async fn request(router: axum::Router, method: &str, uri: &str, body: Value, bearer: Option<&str>) -> (StatusCode, Value, String) {
    let mut builder = Request::builder().method(method).uri(uri).header("content-type", "application/json");
    if let Some(token) = bearer {
        builder = builder.header("authorization", format!("Bearer {token}"));
    }
    let response = router.oneshot(builder.body(Body::from(body.to_string())).unwrap()).await.unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let text = String::from_utf8_lossy(&bytes).to_string();
    (status, serde_json::from_slice(&bytes).unwrap_or(Value::Null), text)
}

async fn sign_up(router: axum::Router, email: &str) -> String {
    let body = json!({ "email": email, "password": "correct-password", "org": { "mode": "create", "name": "Acme" } });
    request(router.clone(), "POST", "/api/auth/register", body, None).await;
    let (_, login, _) = request(router, "POST", "/api/auth/login", json!({ "email": email, "password": "correct-password" }), None).await;
    login["access_token"].as_str().unwrap().to_string()
}

async fn sign_up_admin(router: axum::Router, pool: &PgPool, email: &str) -> String {
    let body = json!({ "email": email, "password": "correct-password", "org": { "mode": "create", "name": "Acme" } });
    request(router.clone(), "POST", "/api/auth/register", body, None).await;
    sqlx::query("UPDATE users SET is_platform_admin = true WHERE id = (SELECT user_id FROM web_credentials WHERE email = $1)")
        .bind(email)
        .execute(pool)
        .await
        .unwrap();
    let (_, login, _) = request(router, "POST", "/api/auth/login", json!({ "email": email, "password": "correct-password" }), None).await;
    login["access_token"].as_str().unwrap().to_string()
}

/// A provider speaking OpenAI's API: lists one model and answers chats, for `key` only.
async fn openai_like(key: &str) -> MockServer {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/models"))
        .and(header("authorization", format!("Bearer {key}").as_str()))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "data": [{ "id": "gpt-5" }] })))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .and(header("authorization", format!("Bearer {key}").as_str()))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            "data: {\"choices\":[{\"delta\":{\"content\":\"hi\"},\"finish_reason\":\"stop\"}]}\n\ndata: [DONE]\n\n",
            "text/event-stream",
        ))
        .mount(&server)
        .await;
    Mock::given(method("GET")).and(path("/v1/models")).respond_with(ResponseTemplate::new(401)).mount(&server).await;
    server
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_catalog_lists_providers_but_not_their_models(pool: PgPool) {
    let router = build_router(test_state(pool));
    let token = sign_up(router.clone(), "cat@example.com").await;

    let (status, _, _) = request(router.clone(), "GET", "/api/llm/providers", Value::Null, None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    let (status, body, _) = request(router.clone(), "GET", "/api/llm/providers", Value::Null, Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    let providers = body["providers"].as_array().unwrap();
    assert_eq!(providers[0]["id"], "opencode");
    assert_eq!(providers[0]["recommended"], true);
    assert_eq!(providers[0]["icon"], "/provider-logos/opencode.svg");
    assert!(providers[0]["model_count"].as_u64().unwrap() > 0);
    assert!(providers[0].get("models").is_none());
    assert!(providers.iter().all(|p| p["id"] != "github-copilot"));
    assert_eq!(providers.last().unwrap()["auth_type"], "custom_endpoint");

    let (status, body, _) = request(router.clone(), "GET", "/api/llm/providers/anthropic/models", Value::Null, Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    assert!(body["models"].as_array().unwrap().iter().any(|m| m["id"] == "claude-haiku-4-5" && m["cost"]["input"].as_f64().is_some()));

    let (status, body, _) = request(router, "GET", "/api/llm/providers/nope/models", Value::Null, Some(&token)).await;
    assert_eq!((status, body["error"].as_str()), (StatusCode::NOT_FOUND, Some("not_found")));
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_person_connects_a_provider_and_its_key_never_comes_back(pool: PgPool) {
    let provider = openai_like("sk-ana-1234").await;
    let router = build_router(test_state(pool));
    let ana = sign_up(router.clone(), "ana@example.com").await;
    let budi = sign_up(router.clone(), "budi@example.com").await;

    let connect = json!({ "provider_id": "openai", "api_key": "sk-ana-1234", "base_url": provider.uri() });
    let (status, body, text) = request(router.clone(), "POST", "/api/llm/connections", connect.clone(), Some(&ana)).await;
    assert_eq!(status, StatusCode::CREATED, "{text}");
    assert_eq!(body["provider_id"], "openai");
    assert_eq!(body["auth_type"], "api_key");
    assert_eq!(body["api_key_masked"], "...1234");
    assert!(!text.contains("sk-ana-1234"));

    let (_, list, text) = request(router.clone(), "GET", "/api/llm/connections", Value::Null, Some(&ana)).await;
    assert_eq!(list["connections"].as_array().unwrap().len(), 1);
    assert!(!text.contains("sk-ana-1234"));
    let (_, theirs, _) = request(router.clone(), "GET", "/api/llm/connections", Value::Null, Some(&budi)).await;
    assert!(theirs["connections"].as_array().unwrap().is_empty());

    let (status, body, _) = request(router, "POST", "/api/llm/connections", connect, Some(&ana)).await;
    assert_eq!((status, body["error"].as_str()), (StatusCode::CONFLICT, Some("already_connected")));
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_key_the_provider_refuses_is_not_saved(pool: PgPool) {
    let provider = openai_like("sk-real").await;
    let router = build_router(test_state(pool));
    let ana = sign_up(router.clone(), "ana@example.com").await;

    let connect = json!({ "provider_id": "openai", "api_key": "sk-wrong", "base_url": provider.uri() });
    let (status, body, _) = request(router.clone(), "POST", "/api/llm/connections", connect, Some(&ana)).await;
    assert_eq!((status, body["error"].as_str()), (StatusCode::BAD_REQUEST, Some("key_rejected")));

    let (status, body, _) = request(router.clone(), "POST", "/api/llm/connections", json!({ "provider_id": "openai" }), Some(&ana)).await;
    assert_eq!((status, body["error"].as_str()), (StatusCode::BAD_REQUEST, Some("key_required")));
    let (status, body, _) = request(router.clone(), "POST", "/api/llm/connections", json!({ "provider_id": "github-copilot", "api_key": "x" }), Some(&ana)).await;
    assert_eq!((status, body["error"].as_str()), (StatusCode::BAD_REQUEST, Some("unknown_provider")));

    let (_, list, _) = request(router, "GET", "/api/llm/connections", Value::Null, Some(&ana)).await;
    assert!(list["connections"].as_array().unwrap().is_empty());
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_custom_endpoint_connects_with_its_own_url_and_headers(pool: PgPool) {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/gw/v1/models"))
        .and(header("x-team", "nomi"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "data": [{ "id": "llama-4" }] })))
        .mount(&server)
        .await;
    let router = build_router(test_state(pool));
    let ana = sign_up(router.clone(), "ana@example.com").await;

    let custom = |extra: Value| {
        let mut body = json!({ "provider_id": "custom", "label": "Team gateway", "base_url": format!("{}/gw/v1", server.uri()), "headers": [{ "name": "X-Team", "value": "nomi" }] });
        body.as_object_mut().unwrap().extend(extra.as_object().unwrap().clone());
        body
    };
    for (extra, code) in [
        (json!({ "label": "" }), "label_required"),
        (json!({ "base_url": null }), "base_url_required"),
        (json!({ "base_url": "ftp://gw.example.com" }), "base_url_invalid"),
        (json!({ "headers": [{ "name": "Bad Name", "value": "x" }] }), "header_invalid"),
        (json!({ "headers": [{ "name": "Authorization", "value": "x" }] }), "header_invalid"),
    ] {
        let (status, body, _) = request(router.clone(), "POST", "/api/llm/connections", custom(extra.clone()), Some(&ana)).await;
        assert_eq!((status, body["error"].as_str()), (StatusCode::BAD_REQUEST, Some(code)), "{extra}");
    }

    let (status, body, text) = request(router.clone(), "POST", "/api/llm/connections", custom(json!({})), Some(&ana)).await;
    assert_eq!(status, StatusCode::CREATED, "{text}");
    assert_eq!(body["auth_type"], "custom_endpoint");
    assert_eq!(body["kind"], "openai_compatible");
    assert_eq!(body["header_names"], json!(["X-Team"]));
    assert!(body["api_key_masked"].is_null());
    assert!(!text.contains("\"nomi\""), "header values never come back");

    let id = body["id"].as_str().unwrap();
    let (status, models, _) = request(router, "POST", &format!("/api/llm/connections/{id}/models"), Value::Null, Some(&ana)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(models["models"], json!([{ "id": "llama-4", "label": null }]));
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_person_runs_on_a_model_from_their_provider_until_they_disconnect_it(pool: PgPool) {
    let provider = openai_like("sk-ana").await;
    let router = build_router(test_state(pool));
    let ana = sign_up(router.clone(), "ana@example.com").await;
    let budi = sign_up(router.clone(), "budi@example.com").await;
    let connect = json!({ "provider_id": "openai", "api_key": "sk-ana", "base_url": provider.uri() });
    let (_, connection, _) = request(router.clone(), "POST", "/api/llm/connections", connect, Some(&ana)).await;
    let id = connection["id"].as_str().unwrap();

    // Someone else can't pick, read the models of, or remove it.
    let pick = json!({ "kind": "connection", "connection_id": id, "model_id": "gpt-5", "label": "GPT-5" });
    let (status, _, _) = request(router.clone(), "PUT", "/api/llm/selection", pick.clone(), Some(&budi)).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (status, _, _) = request(router.clone(), "POST", &format!("/api/llm/connections/{id}/models"), Value::Null, Some(&budi)).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (status, _, _) = request(router.clone(), "DELETE", &format!("/api/llm/connections/{id}"), Value::Null, Some(&budi)).await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    let (status, _, text) = request(router.clone(), "PUT", "/api/llm/selection", pick, Some(&ana)).await;
    assert_eq!(status, StatusCode::NO_CONTENT, "{text}");
    let (_, models, _) = request(router.clone(), "GET", "/api/llm/models", Value::Null, Some(&ana)).await;
    assert_eq!(models["selection"], json!({ "kind": "connection", "connection_id": id, "provider_id": "openai", "label": "GPT-5", "model_id": "gpt-5" }));

    let (status, _, _) = request(router.clone(), "DELETE", &format!("/api/llm/connections/{id}"), Value::Null, Some(&ana)).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (_, models, _) = request(router, "GET", "/api/llm/models", Value::Null, Some(&ana)).await;
    assert!(models["selection"].is_null());
}

#[sqlx::test(migrations = "../../migrations")]
async fn admins_connect_providers_for_everyone_and_add_their_models(pool: PgPool) {
    let provider = openai_like("sk-shared").await;
    let router = build_router(test_state(pool.clone()));
    let admin = sign_up_admin(router.clone(), &pool, "admin@example.com").await;
    let person = sign_up(router.clone(), "person@example.com").await;

    let (status, _, _) = request(router.clone(), "GET", "/api/admin/settings/llm/connections", Value::Null, Some(&person)).await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    let connect = json!({ "provider_id": "openai", "api_key": "sk-shared", "base_url": provider.uri() });
    let (status, connection, text) = request(router.clone(), "POST", "/api/admin/settings/llm/connections", connect, Some(&admin)).await;
    assert_eq!(status, StatusCode::CREATED, "{text}");
    let id = connection["id"].as_str().unwrap();
    // It's the instance's: not in the admin's own list.
    let (_, own, _) = request(router.clone(), "GET", "/api/llm/connections", Value::Null, Some(&admin)).await;
    assert!(own["connections"].as_array().unwrap().is_empty());

    let (status, live, _) = request(router.clone(), "POST", &format!("/api/admin/settings/llm/connections/{id}/models"), Value::Null, Some(&admin)).await;
    assert_eq!((status, live["models"][0]["id"].as_str()), (StatusCode::OK, Some("gpt-5")));

    let model = json!({ "label": "GPT-5", "connection_id": id, "model_id": "gpt-5", "input_usd_per_mtok": 1.25 });
    let (status, created, text) = request(router.clone(), "POST", "/api/admin/settings/llm/models", model, Some(&admin)).await;
    assert_eq!(status, StatusCode::CREATED, "{text}");
    assert_eq!(created["connection_id"], id);
    assert_eq!(created["connection_label"], "OpenAI");
    assert_eq!(created["provider"], "openai");
    assert_eq!(created["api_key_masked"], "");

    let (status, body, _) = request(router.clone(), "DELETE", &format!("/api/admin/settings/llm/connections/{id}"), Value::Null, Some(&admin)).await;
    assert_eq!((status, body["error"].as_str(), body["models"].clone()), (StatusCode::CONFLICT, Some("in_use"), json!(["GPT-5"])));
    let (_, list, _) = request(router, "GET", "/api/admin/settings/llm/connections", Value::Null, Some(&admin)).await;
    assert_eq!(list["connections"][0]["models_in_use"], json!(["GPT-5"]));
}

#[sqlx::test(migrations = "../../migrations")]
async fn an_admin_edits_a_model_from_a_provider_without_a_key(pool: PgPool) {
    let provider = openai_like("sk-shared").await;
    let router = build_router(test_state(pool.clone()));
    let admin = sign_up_admin(router.clone(), &pool, "admin@example.com").await;
    let connect = json!({ "provider_id": "openai", "api_key": "sk-shared", "base_url": provider.uri() });
    let (_, connection, _) = request(router.clone(), "POST", "/api/admin/settings/llm/connections", connect, Some(&admin)).await;
    let id = connection["id"].as_str().unwrap();
    let (_, model, _) = request(router.clone(), "POST", "/api/admin/settings/llm/models", json!({ "label": "GPT-5", "connection_id": id, "model_id": "gpt-5" }), Some(&admin)).await;
    let model_id = model["id"].as_str().unwrap();

    let edit = json!({ "label": "GPT-5 mini", "connection_id": id, "model_id": "gpt-5-mini", "output_usd_per_mtok": 2.0, "media_inputs": ["image"] });
    let (status, updated, text) = request(router, "PUT", &format!("/api/admin/settings/llm/models/{model_id}"), edit, Some(&admin)).await;
    assert_eq!(status, StatusCode::OK, "{text}");
    assert_eq!((updated["label"].as_str(), updated["model_id"].as_str()), (Some("GPT-5 mini"), Some("gpt-5-mini")));
    assert_eq!((updated["connection_id"].as_str(), updated["provider"].as_str()), (Some(id), Some("openai")));
    assert_eq!(updated["output_usd_per_mtok"], 2.0);
    assert_eq!(updated["api_key_masked"], "");
}
