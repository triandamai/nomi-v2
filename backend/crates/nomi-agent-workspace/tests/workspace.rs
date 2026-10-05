use std::sync::Arc;

use serde_json::{json, Value};
use sqlx::PgPool;
use uuid::Uuid;
use wiremock::matchers::{body_string_contains, header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

use nomi_agent_core::content_block::ContentBlock;
use nomi_agent_core::SubAgent;
use nomi_agent_workspace::connection::{self, GoogleConfig};
use nomi_agent_workspace::WorkspaceAgent;

const KEY: [u8; 32] = [9u8; 32];

async fn seed_user(pool: &PgPool) -> Uuid {
    sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(pool).await.unwrap()
}

/// Connects `user` to the fake Google with the given services, the way the callback does.
async fn connect(pool: &PgPool, server: &MockServer, user: Uuid, services: &[&str], token: &str) {
    let config = GoogleConfig::all_at(&server.uri());
    let scopes: Vec<&str> = services.iter().filter_map(|s| connection::service_scope(s)).collect();
    Mock::given(method("POST"))
        .and(path("/token"))
        .and(body_string_contains(format!("code={token}-code")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "access_token": token, "expires_in": 3600, "refresh_token": format!("{token}-refresh"), "scope": format!("openid email {}", scopes.join(" "))
        })))
        .mount(server)
        .await;
    Mock::given(method("GET"))
        .and(path("/v1/userinfo"))
        .and(header("authorization", format!("Bearer {token}").as_str()))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "email": format!("{token}@example.com") })))
        .mount(server)
        .await;
    let mut conn = pool.acquire().await.unwrap();
    let wanted: Vec<String> = services.iter().map(|s| s.to_string()).collect();
    let url = connection::start_authorization(&mut conn, &config, user, &wanted, None).await.unwrap();
    let state = reqwest::Url::parse(&url).unwrap().query_pairs().find(|(k, _)| k == "state").unwrap().1.to_string();
    connection::complete_authorization(&mut conn, &reqwest::Client::new(), &config, &KEY, user, &format!("{token}-code"), &state).await.unwrap();
}

fn agent(server: &MockServer) -> WorkspaceAgent {
    WorkspaceAgent::new(reqwest::Client::new(), Some(Arc::new(GoogleConfig::all_at(&server.uri()))), KEY)
}

async fn run(agent: &WorkspaceAgent, pool: &PgPool, user: Uuid, tool: &str, input: Value) -> nomi_agent_core::content_block::ToolOutcome {
    let mut conn = pool.acquire().await.unwrap();
    agent.execute_tool(&mut conn, Uuid::new_v4(), Uuid::new_v4(), user, tool, input).await.unwrap()
}

#[sqlx::test(migrations = "../../migrations")]
async fn connecting_asks_google_for_the_chosen_services_and_stores_the_tokens_encrypted(pool: PgPool) {
    let server = MockServer::start().await;
    let user = seed_user(&pool).await;
    let config = GoogleConfig::all_at(&server.uri());
    let mut conn = pool.acquire().await.unwrap();

    let url = connection::start_authorization(&mut conn, &config, user, &["sheets".into(), "gmail".into()], None).await.unwrap();
    let url = reqwest::Url::parse(&url).unwrap();
    let scope = url.query_pairs().find(|(k, _)| k == "scope").unwrap().1.to_string();
    assert_eq!(scope, "openid email https://www.googleapis.com/auth/gmail.modify https://www.googleapis.com/auth/spreadsheets");
    assert!(url.query_pairs().any(|(k, v)| k == "access_type" && v == "offline"));
    drop(conn);

    connect(&pool, &server, user, &["gmail", "sheets"], "tok-ana").await;
    let (email, services, access): (String, Vec<String>, Vec<u8>) =
        sqlx::query_as("SELECT google_email, services, access_token_encrypted FROM workspace_connections WHERE user_id = $1")
            .bind(user)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(email, "tok-ana@example.com");
    assert_eq!(services, vec!["gmail".to_string(), "sheets".to_string()]);
    assert!(!String::from_utf8_lossy(&access).contains("tok-ana"), "the token must not be stored in plain text");
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_sign_in_started_by_one_user_cannot_be_finished_by_another(pool: PgPool) {
    let server = MockServer::start().await;
    let (ana, budi) = (seed_user(&pool).await, seed_user(&pool).await);
    let config = GoogleConfig::all_at(&server.uri());
    let mut conn = pool.acquire().await.unwrap();
    let url = connection::start_authorization(&mut conn, &config, ana, &["gmail".into()], None).await.unwrap();
    let state = reqwest::Url::parse(&url).unwrap().query_pairs().find(|(k, _)| k == "state").unwrap().1.to_string();

    let result = connection::complete_authorization(&mut conn, &reqwest::Client::new(), &config, &KEY, budi, "code", &state).await;
    assert!(result.is_err());
    let connected: i64 = sqlx::query_scalar("SELECT count(*) FROM workspace_connections").fetch_one(&pool).await.unwrap();
    assert_eq!(connected, 0);
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_user_who_has_not_connected_gets_a_connect_card_even_when_someone_else_has(pool: PgPool) {
    let server = MockServer::start().await;
    let (ana, budi) = (seed_user(&pool).await, seed_user(&pool).await);
    connect(&pool, &server, ana, &["gmail"], "tok-ana").await;

    let outcome = run(&agent(&server), &pool, budi, "gmail_search", json!({ "query": "invoice" })).await;

    assert_eq!(outcome.block, Some(ContentBlock::WorkspaceConnect { reason: "not_connected".into(), services: vec!["gmail".into()] }));
    // No Gmail call was made with anyone's token.
    let gmail_calls = server.received_requests().await.unwrap().iter().filter(|r| r.url.path().starts_with("/gmail")).count();
    assert_eq!(gmail_calls, 0);
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_service_the_user_did_not_allow_asks_them_to_turn_it_on(pool: PgPool) {
    let server = MockServer::start().await;
    let ana = seed_user(&pool).await;
    connect(&pool, &server, ana, &["gmail"], "tok-ana").await;

    let outcome = run(&agent(&server), &pool, ana, "sheets_read", json!({ "spreadsheet_id": "s1" })).await;

    assert_eq!(outcome.block, Some(ContentBlock::WorkspaceConnect { reason: "service_not_allowed".into(), services: vec!["sheets".into()] }));
}

#[sqlx::test(migrations = "../../migrations")]
async fn gmail_search_uses_the_asking_users_own_token(pool: PgPool) {
    let server = MockServer::start().await;
    let (ana, budi) = (seed_user(&pool).await, seed_user(&pool).await);
    connect(&pool, &server, ana, &["gmail"], "tok-ana").await;
    connect(&pool, &server, budi, &["gmail"], "tok-budi").await;
    Mock::given(method("GET"))
        .and(path("/gmail/v1/users/me/messages"))
        .and(query_param("q", "invoice"))
        .and(header("authorization", "Bearer tok-budi"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "messages": [{ "id": "m1" }] })))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/gmail/v1/users/me/messages/m1"))
        .and(header("authorization", "Bearer tok-budi"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "snippet": "Total Rp 2.850.000",
            "payload": { "headers": [{ "name": "From", "value": "Ubud Villas <stay@ubud.example>" }, { "name": "Subject", "value": "Invoice" }] }
        })))
        .mount(&server)
        .await;

    let outcome = run(&agent(&server), &pool, budi, "gmail_search", json!({ "query": "invoice" })).await;

    let result: Value = serde_json::from_str(&outcome.display_text).unwrap();
    assert_eq!(result["messages"][0]["subject"], "Invoice");
    assert_eq!(result["messages"][0]["snippet"], "Total Rp 2.850.000");
}

#[sqlx::test(migrations = "../../migrations")]
async fn appending_rows_writes_to_the_sheet_and_shows_up_in_recent_activity(pool: PgPool) {
    let server = MockServer::start().await;
    let ana = seed_user(&pool).await;
    connect(&pool, &server, ana, &["sheets"], "tok-ana").await;
    Mock::given(method("POST"))
        .and(path("/v4/spreadsheets/budget1/values/Stays!A:C:append"))
        .and(query_param("valueInputOption", "USER_ENTERED"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "updates": { "updatedRange": "Stays!A5:C6", "updatedRows": 2 } })))
        .mount(&server)
        .await;

    let outcome = run(
        &agent(&server),
        &pool,
        ana,
        "sheets_append",
        json!({ "spreadsheet_id": "budget1", "range": "Stays!A:C", "rows": [["Ubud Villas", 3, 2850000], ["Canggu Surf Stay", 2, 1600000]] }),
    )
    .await;

    let result: Value = serde_json::from_str(&outcome.display_text).unwrap();
    assert_eq!(result["rows_added"], 2);
    let mut conn = pool.acquire().await.unwrap();
    let activity = connection::recent_activity(&mut conn, ana, 5).await.unwrap();
    assert_eq!(activity[0].summary, "Added 2 rows to Stays!A:C");
    assert_eq!(activity[0].link.as_deref(), Some("https://docs.google.com/spreadsheets/d/budget1/edit"));
}

#[sqlx::test(migrations = "../../migrations")]
async fn an_expired_token_is_refreshed_before_calling_google(pool: PgPool) {
    let server = MockServer::start().await;
    let ana = seed_user(&pool).await;
    connect(&pool, &server, ana, &["drive"], "tok-old").await;
    sqlx::query("UPDATE workspace_connections SET expires_at = now() - interval '1 minute' WHERE user_id = $1").bind(ana).execute(&pool).await.unwrap();
    Mock::given(method("POST"))
        .and(path("/token"))
        .and(body_string_contains("grant_type=refresh_token"))
        .and(body_string_contains("refresh_token=tok-old-refresh"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "access_token": "tok-new", "expires_in": 3600 })))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/drive/v3/files"))
        .and(header("authorization", "Bearer tok-new"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "files": [{ "id": "f1", "name": "Bali budget" }] })))
        .mount(&server)
        .await;

    let outcome = run(&agent(&server), &pool, ana, "drive_search", json!({ "query": "Bali" })).await;

    let result: Value = serde_json::from_str(&outcome.display_text).unwrap();
    assert_eq!(result["files"][0]["name"], "Bali budget");
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_revoked_connection_is_forgotten_and_the_user_is_asked_to_reconnect(pool: PgPool) {
    let server = MockServer::start().await;
    let ana = seed_user(&pool).await;
    connect(&pool, &server, ana, &["docs"], "tok-ana").await;
    Mock::given(method("GET"))
        .and(path("/v1/documents/d1"))
        .respond_with(ResponseTemplate::new(401).set_body_json(json!({ "error": { "message": "Invalid Credentials" } })))
        .mount(&server)
        .await;

    let outcome = run(&agent(&server), &pool, ana, "docs_read", json!({ "document_id": "d1" })).await;

    assert!(matches!(outcome.block, Some(ContentBlock::WorkspaceConnect { .. })));
    let mut conn = pool.acquire().await.unwrap();
    assert_eq!(connection::status(&mut conn, ana).await.unwrap(), None);
}

#[sqlx::test(migrations = "../../migrations")]
async fn disconnecting_revokes_the_token_and_forgets_the_account(pool: PgPool) {
    let server = MockServer::start().await;
    let ana = seed_user(&pool).await;
    connect(&pool, &server, ana, &["gmail"], "tok-ana").await;
    Mock::given(method("POST"))
        .and(path("/revoke"))
        .and(body_string_contains("token=tok-ana-refresh"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&server)
        .await;

    let mut conn = pool.acquire().await.unwrap();
    let config = GoogleConfig::all_at(&server.uri());
    assert!(connection::disconnect(&mut conn, &reqwest::Client::new(), Some(&config), &KEY, ana).await.unwrap());
    assert_eq!(connection::status(&mut conn, ana).await.unwrap(), None);
}

/// A stand-in default agent: the registry needs exactly one.
struct Fallback;

#[async_trait::async_trait]
impl SubAgent for Fallback {
    fn agent_type(&self) -> std::borrow::Cow<'static, str> {
        "chitchat".into()
    }
    fn system_prompt(&self) -> std::borrow::Cow<'static, str> {
        "".into()
    }
    fn tools(&self) -> Vec<nomi_llm::ToolDefinition> {
        vec![]
    }
    async fn execute_tool(
        &self,
        _: &mut sqlx::pool::PoolConnection<sqlx::Postgres>,
        _: Uuid,
        _: Uuid,
        _: Uuid,
        _: &str,
        _: Value,
    ) -> Result<nomi_agent_core::content_block::ToolOutcome, String> {
        Err("no tools".into())
    }
    fn intent_label(&self) -> std::borrow::Cow<'static, str> {
        "chat".into()
    }
    fn intent_description(&self) -> std::borrow::Cow<'static, str> {
        "".into()
    }
    fn is_default(&self) -> bool {
        true
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn asking_before_connecting_puts_a_connect_card_in_the_chat(pool: PgPool) {
    use nomi_llm::{ContentBlock as Llm, LlmResponse, StopReason};
    let server = MockServer::start().await;
    let user = seed_user(&pool).await;
    let org: Uuid = sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Acme') RETURNING id").fetch_one(&pool).await.unwrap();
    let session: Uuid = sqlx::query_scalar("INSERT INTO sessions (org_id, channel, chat_id) VALUES ($1, 'web', 'c1') RETURNING id").bind(org).fetch_one(&pool).await.unwrap();
    let provider = nomi_test_support::FakeLlmProvider::sequence(vec![
        LlmResponse {
            content: vec![Llm::ToolUse { id: "t1".into(), name: "gmail_search".into(), input: json!({ "query": "invoice" }), thought_signature: None }],
            stop_reason: StopReason::ToolUse,
            input_tokens: 1,
            output_tokens: 1,
        },
        LlmResponse { content: vec![Llm::Text { text: "Connect your Google account first.".into() }], stop_reason: StopReason::EndTurn, input_tokens: 1, output_tokens: 1 },
        // Memory extraction after the reply.
        LlmResponse { content: vec![Llm::Text { text: "NONE".into() }], stop_reason: StopReason::EndTurn, input_tokens: 1, output_tokens: 1 },
    ]);
    let embedder = nomi_test_support::FakeEmbeddingProvider::success(vec![0.0; 1536]);
    let workspace = agent(&server);
    let registry = nomi_agent_core::AgentRegistry::new(vec![Box::new(Fallback), Box::new(agent(&server))]);
    let mut conn = pool.acquire().await.unwrap();

    nomi_agent_core::run_agent_turn(&mut conn, None, None, &provider, &embedder, &registry, &workspace, session, session, user, vec![], 1024).await.unwrap();

    let blocks: Vec<Value> = sqlx::query_scalar("SELECT content_blocks FROM messages WHERE session_id = $1 AND content_blocks IS NOT NULL")
        .bind(session)
        .fetch_all(&pool)
        .await
        .unwrap();
    assert!(
        blocks.iter().any(|b| b[0]["kind"] == "workspace_connect" && b[0]["services"] == json!(["gmail"])),
        "no connect card in {blocks:?}"
    );
}
