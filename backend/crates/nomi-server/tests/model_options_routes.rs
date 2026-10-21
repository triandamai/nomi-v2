use axum::{body::Body, http::{Request, StatusCode}};
use http_body_util::BodyExt;
use nomi_server::app::{build_router, AppState};
use nomi_settings::crypto;
use nomi_settings::provider_connections::{create_connection, NewConnection, Scope};
use nomi_test_support::TEST_SETTINGS_KEY;
use serde_json::{json, Value};
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const SECRET: &str = "test-secret-do-not-use-in-prod";

fn test_state(pool: PgPool) -> AppState {
    AppState {
        pool,
        jwt_secret: SECRET.to_string(),
        http_client: reqwest::Client::new(),
        settings_key: TEST_SETTINGS_KEY,
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

async fn request(router: axum::Router, method: &str, uri: &str, body: Value, bearer: Option<&str>) -> (StatusCode, Value) {
    let mut builder = Request::builder().method(method).uri(uri).header("content-type", "application/json");
    if let Some(token) = bearer {
        builder = builder.header("authorization", format!("Bearer {token}"));
    }
    let response = router.oneshot(builder.body(Body::from(body.to_string())).unwrap()).await.unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    (status, serde_json::from_slice(&bytes).unwrap_or(Value::Null))
}

/// Signs someone up; returns their token and user id.
async fn sign_up(router: axum::Router, pool: &PgPool, email: &str) -> (String, Uuid) {
    let body = json!({ "email": email, "password": "correct-password", "org": { "mode": "create", "name": "Acme" } });
    request(router.clone(), "POST", "/api/auth/register", body, None).await;
    let (_, login) = request(router, "POST", "/api/auth/login", json!({ "email": email, "password": "correct-password" }), None).await;
    let user_id = sqlx::query_scalar("SELECT user_id FROM web_credentials WHERE email = $1").bind(email).fetch_one(pool).await.unwrap();
    (login["access_token"].as_str().unwrap().to_string(), user_id)
}

async fn admin_model(pool: &PgPool, by: Uuid, label: &str, provider: &str, model_id: &str) -> Uuid {
    sqlx::query_scalar("INSERT INTO admin_llm_models (label, provider, model_id, api_key_encrypted, is_default, updated_by) VALUES ($1, $2, $3, $4, NOT EXISTS (SELECT 1 FROM admin_llm_models), $5) RETURNING id")
        .bind(label)
        .bind(provider)
        .bind(model_id)
        .bind(crypto::encrypt(&TEST_SETTINGS_KEY, "sk"))
        .bind(by)
        .fetch_one(pool)
        .await
        .unwrap()
}

async fn own_connection(pool: &PgPool, owner: Uuid, provider_id: &str, kind: &str, base_url: Option<&str>) -> Uuid {
    let custom = provider_id == "custom";
    create_connection(
        pool,
        Scope::User(owner),
        NewConnection {
            provider_id,
            label: if custom { "My gateway" } else { provider_id },
            kind,
            auth_type: if custom { "custom_endpoint" } else { "api_key" },
            api_key_encrypted: Some(crypto::encrypt(&TEST_SETTINGS_KEY, "key")),
            base_url,
            headers_encrypted: None,
            updated_by: owner,
        },
    )
    .await
    .unwrap()
    .id
}

async fn new_chat(router: axum::Router, token: &str) -> String {
    let (_, body) = request(router, "POST", "/api/sessions", Value::Null, Some(token)).await;
    body["session_id"].as_str().unwrap().to_string()
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_selector_offers_nomis_models_by_provider_and_the_persons_own_providers(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let (token, ana) = sign_up(router.clone(), &pool, "ana@example.com").await;
    let haiku = admin_model(&pool, ana, "Haiku", "anthropic", "claude-haiku-4-5").await;
    let flash = admin_model(&pool, ana, "Flash", "gemini", "gemini-2.5-flash").await;
    let opencode = own_connection(&pool, ana, "opencode", "openai_compatible", None).await;

    let (status, _) = request(router.clone(), "GET", "/api/llm/model-options", Value::Null, None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    let (status, body) = request(router, "GET", "/api/llm/model-options", Value::Null, Some(&token)).await;
    assert_eq!(status, StatusCode::OK);
    let groups = body["groups"].as_array().unwrap();
    let labels: Vec<&str> = groups.iter().map(|g| g["label"].as_str().unwrap()).collect();
    assert_eq!(labels, ["Anthropic", "Google", "OpenCode"]);

    assert_eq!(groups[0]["source"], "nomi");
    assert_eq!(groups[0]["provider_id"], "anthropic");
    assert_eq!(groups[0]["models"], json!([{ "key": format!("admin:{haiku}"), "label": "Haiku", "model_id": "claude-haiku-4-5", "free": false, "enabled": true }]));
    assert_eq!(groups[1]["provider_id"], "google");
    assert_eq!(groups[1]["models"][0]["key"], format!("admin:{flash}"));

    let own = &groups[2];
    assert_eq!((own["source"].as_str(), own["id"].as_str(), own["connection_id"].as_str()), (Some("connection"), Some(format!("conn:{opencode}").as_str()), Some(opencode.to_string().as_str())));
    let models = own["models"].as_array().unwrap();
    assert!(models.len() > 10, "the catalog's models for the provider");
    assert!(models.iter().all(|m| m["enabled"] == false), "a person's own provider's models start hidden");
    assert!(models.iter().any(|m| m["free"] == true), "free models are marked");
    assert!(models[0]["key"].as_str().unwrap().starts_with(&format!("conn:{opencode}:")));

    assert_eq!(body["current"]["key"], format!("admin:{haiku}"), "the default model");
    assert_eq!(body["current"]["chat_only"], false);
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_custom_endpoint_lists_its_live_models_or_says_it_cannot(pool: PgPool) {
    std::env::set_var("NOMI_WEB_ALLOW_PRIVATE", "1");
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/models"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "data": [{ "id": "llama-4" }, { "id": "qwen-3" }] })))
        .mount(&server)
        .await;
    let router = build_router(test_state(pool.clone()));
    let (token, ana) = sign_up(router.clone(), &pool, "ana@example.com").await;
    own_connection(&pool, ana, "custom", "openai_compatible", Some(&format!("{}/v1", server.uri()))).await;
    create_connection(
        &pool,
        Scope::User(ana),
        NewConnection {
            provider_id: "custom",
            label: "Down",
            kind: "openai_compatible",
            auth_type: "custom_endpoint",
            api_key_encrypted: None,
            base_url: Some("http://127.0.0.1:9/v1"),
            headers_encrypted: None,
            updated_by: ana,
        },
    )
    .await
    .unwrap();

    let (_, body) = request(router, "GET", "/api/llm/model-options", Value::Null, Some(&token)).await;
    let groups = body["groups"].as_array().unwrap();
    let live = groups.iter().find(|g| g["label"] == "My gateway").unwrap();
    assert_eq!(live["provider_id"], "custom");
    assert_eq!(live["models"].as_array().unwrap().iter().map(|m| m["model_id"].as_str().unwrap()).collect::<Vec<_>>(), ["llama-4", "qwen-3"]);
    assert!(live["error"].is_null());
    let down = groups.iter().find(|g| g["label"] == "Down").unwrap();
    assert_eq!((down["error"].as_str(), down["models"].as_array().unwrap().len()), (Some("unreachable"), 0));
}

#[sqlx::test(migrations = "../../migrations")]
async fn each_person_chooses_what_their_selector_shows(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let (ana_token, ana) = sign_up(router.clone(), &pool, "ana@example.com").await;
    let (budi_token, _) = sign_up(router.clone(), &pool, "budi@example.com").await;
    let haiku = admin_model(&pool, ana, "Haiku", "anthropic", "claude-haiku-4-5").await;
    let opencode = own_connection(&pool, ana, "opencode", "openai_compatible", None).await;
    let (_, options) = request(router.clone(), "GET", "/api/llm/model-options", Value::Null, Some(&ana_token)).await;
    let free_model = options["groups"][1]["models"].as_array().unwrap().iter().find(|m| m["free"] == true).unwrap()["key"].clone();

    let changes = json!({ "changes": [{ "key": format!("admin:{haiku}"), "enabled": false }, { "key": free_model, "enabled": true }] });
    let (status, _) = request(router.clone(), "PUT", "/api/llm/model-options", changes, Some(&ana_token)).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (_, options) = request(router.clone(), "GET", "/api/llm/model-options", Value::Null, Some(&ana_token)).await;
    assert_eq!(options["groups"][0]["models"][0]["enabled"], false);
    assert!(options["groups"][1]["models"].as_array().unwrap().iter().any(|m| m["key"] == free_model && m["enabled"] == true));
    let (_, theirs) = request(router.clone(), "GET", "/api/llm/model-options", Value::Null, Some(&budi_token)).await;
    assert_eq!(theirs["groups"][0]["models"][0]["enabled"], true);

    for key in ["nomi:x", "admin:not-a-uuid", &format!("conn:{opencode}:")] {
        let (status, body) = request(router.clone(), "PUT", "/api/llm/model-options", json!({ "changes": [{ "key": key, "enabled": true }] }), Some(&ana_token)).await;
        assert_eq!((status, body["error"].as_str()), (StatusCode::BAD_REQUEST, Some("invalid_key")), "{key}");
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_chat_can_run_on_its_own_model(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let (ana_token, ana) = sign_up(router.clone(), &pool, "ana@example.com").await;
    let (budi_token, budi) = sign_up(router.clone(), &pool, "budi@example.com").await;
    let haiku = admin_model(&pool, ana, "Haiku", "anthropic", "claude-haiku-4-5").await;
    let opus = admin_model(&pool, ana, "Opus", "anthropic", "claude-opus-5-5").await;
    let opencode = own_connection(&pool, ana, "opencode", "openai_compatible", None).await;
    let budis = own_connection(&pool, budi, "openrouter", "openrouter", None).await;
    let chat = new_chat(router.clone(), &ana_token).await;
    let uri = format!("/api/sessions/{chat}/model");

    let (status, body) = request(router.clone(), "GET", &uri, Value::Null, Some(&ana_token)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!((body["model"]["key"].clone(), body["model"]["chat_only"].clone()), (json!(format!("admin:{haiku}")), json!(false)));

    let (status, body) = request(router.clone(), "PUT", &uri, json!({ "key": format!("admin:{opus}") }), Some(&ana_token)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!((body["model"]["label"].clone(), body["model"]["provider_label"].clone(), body["model"]["chat_only"].clone()), (json!("Opus"), json!("Anthropic"), json!(true)));

    let free = format!("conn:{opencode}:big-pickle");
    let (status, body) = request(router.clone(), "PUT", &uri, json!({ "key": free }), Some(&ana_token)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!((body["model"]["provider_id"].clone(), body["model"]["provider_label"].clone()), (json!("opencode"), json!("OpenCode")));
    let (_, body) = request(router.clone(), "GET", &uri, Value::Null, Some(&ana_token)).await;
    assert_eq!(body["model"]["key"], free);

    // Someone else's provider, an unknown or switched-off model, a bad key, someone else's chat.
    let (status, _) = request(router.clone(), "PUT", &uri, json!({ "key": format!("conn:{budis}:x/y") }), Some(&ana_token)).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    sqlx::query("UPDATE admin_llm_models SET enabled = false WHERE id = $1").bind(opus).execute(&pool).await.unwrap();
    let (status, _) = request(router.clone(), "PUT", &uri, json!({ "key": format!("admin:{opus}") }), Some(&ana_token)).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (status, body) = request(router.clone(), "PUT", &uri, json!({ "key": "gpt-5" }), Some(&ana_token)).await;
    assert_eq!((status, body["error"].as_str()), (StatusCode::BAD_REQUEST, Some("invalid_key")));
    let (status, _) = request(router.clone(), "GET", &uri, Value::Null, Some(&budi_token)).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (status, _) = request(router, "PUT", &uri, json!({ "key": format!("admin:{haiku}") }), Some(&budi_token)).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[sqlx::test(migrations = "../../migrations")]
async fn the_model_page_sets_the_default_from_the_same_list(pool: PgPool) {
    let router = build_router(test_state(pool.clone()));
    let (ana_token, ana) = sign_up(router.clone(), &pool, "ana@example.com").await;
    let (_, budi) = sign_up(router.clone(), &pool, "budi@example.com").await;
    admin_model(&pool, ana, "Haiku", "anthropic", "claude-haiku-4-5").await;
    let opus = admin_model(&pool, ana, "Opus", "anthropic", "claude-opus-5-5").await;
    let opencode = own_connection(&pool, ana, "opencode", "openai_compatible", None).await;
    let budis = own_connection(&pool, budi, "openrouter", "openrouter", None).await;
    let default_model = |router: axum::Router, key: String| {
        let token = ana_token.clone();
        async move { request(router, "PUT", "/api/llm/default-model", json!({ "key": key }), Some(&token)).await }
    };

    let (status, body) = default_model(router.clone(), format!("admin:{opus}")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!((body["model"]["key"].clone(), body["model"]["chat_only"].clone()), (json!(format!("admin:{opus}")), json!(false)));

    // One of their own provider's models can be the default too, named from the catalog.
    let free = format!("conn:{opencode}:big-pickle");
    let (status, body) = default_model(router.clone(), free.clone()).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!((body["model"]["key"].clone(), body["model"]["provider_label"].clone()), (json!(free), json!("OpenCode")));
    assert_ne!(body["model"]["label"], "big-pickle", "the catalog's name, not the id");

    // A new chat starts on it.
    let token = ana_token_again(router.clone(), "ana@example.com").await;
    let (_, options) = request(router.clone(), "GET", "/api/llm/model-options", Value::Null, Some(&token)).await;
    assert_eq!(options["current"]["key"], free);
    let chat = new_chat(router.clone(), &token).await;
    let (_, body) = request(router.clone(), "GET", &format!("/api/sessions/{chat}/model"), Value::Null, Some(&token)).await;
    assert_eq!(body["model"]["key"], free);

    let (status, _) = default_model(router.clone(), format!("conn:{budis}:x/y")).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    sqlx::query("UPDATE admin_llm_models SET enabled = false WHERE id = $1").bind(opus).execute(&pool).await.unwrap();
    let (status, _) = default_model(router.clone(), format!("admin:{opus}")).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (status, body) = default_model(router, "claude".to_string()).await;
    assert_eq!((status, body["error"].as_str()), (StatusCode::BAD_REQUEST, Some("invalid_key")));
}

async fn ana_token_again(router: axum::Router, email: &str) -> String {
    let (_, login) = request(router, "POST", "/api/auth/login", json!({ "email": email, "password": "correct-password" }), None).await;
    login["access_token"].as_str().unwrap().to_string()
}
