//! Admin → Tools switches and the web tools, as the engine applies them.

use std::borrow::Cow;

use sqlx::pool::PoolConnection;
use sqlx::{PgPool, Postgres};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use uuid::Uuid;

use nomi_agent_core::{run_agent_turn, AgentRegistry, LoopOutcome, SubAgent, ToolOutcome};
use nomi_llm::{ContentBlock, LlmResponse, StopReason, ToolDefinition};
use nomi_test_support::{FakeEmbeddingProvider, FakeLlmProvider, TEST_SETTINGS_KEY};

/// `CUSTOM` makes it behave like a custom agent, whose tools come from the switchable catalog.
struct TestAgent<const CUSTOM: bool>;

#[async_trait::async_trait]
impl<const CUSTOM: bool> SubAgent for TestAgent<CUSTOM> {
    fn agent_type(&self) -> Cow<'static, str> {
        Cow::Borrowed("test")
    }
    fn system_prompt(&self) -> Cow<'static, str> {
        Cow::Borrowed("test prompt")
    }
    fn tools(&self) -> Vec<ToolDefinition> {
        vec![ToolDefinition { name: "echo".to_string(), description: "Echoes".to_string(), input_schema: serde_json::json!({"type": "object"}) }]
    }
    async fn execute_tool(
        &self,
        _conn: &mut PoolConnection<Postgres>,
        _session_id: Uuid,
        _agent_session_id: Uuid,
        _user_id: Uuid,
        name: &str,
        input: serde_json::Value,
    ) -> Result<ToolOutcome, String> {
        match name {
            "echo" => Ok(ToolOutcome::text(format!("echoed: {input}"))),
            other => Err(format!("unknown tool: {other}")),
        }
    }
    fn intent_label(&self) -> Cow<'static, str> {
        Cow::Borrowed("test")
    }
    fn intent_description(&self) -> Cow<'static, str> {
        Cow::Borrowed("test agent")
    }
    fn is_default(&self) -> bool {
        true
    }
    fn own_tools_switchable(&self) -> bool {
        CUSTOM
    }
}

async fn seed(pool: &PgPool) -> (Uuid, Uuid, Uuid) {
    let org_id: Uuid = sqlx::query_scalar("INSERT INTO organizations (name) VALUES ('Acme') RETURNING id").fetch_one(pool).await.unwrap();
    let session_id: Uuid = sqlx::query_scalar("INSERT INTO sessions (org_id, channel, chat_id) VALUES ($1, 'telegram', 'chat-1') RETURNING id")
        .bind(org_id)
        .fetch_one(pool)
        .await
        .unwrap();
    let user_id: Uuid = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(pool).await.unwrap();
    let identity_id: Uuid =
        sqlx::query_scalar("INSERT INTO channel_identities (user_id, channel, channel_user_id) VALUES ($1, 'telegram', 'u1') RETURNING id")
            .bind(user_id)
            .fetch_one(pool)
            .await
            .unwrap();
    let agent_session_id: Uuid = sqlx::query_scalar(
        "INSERT INTO agent_sessions (session_id, sender_channel_identity_id, agent_type, status) VALUES ($1, $2, 'test', 'active') RETURNING id",
    )
    .bind(session_id)
    .bind(identity_id)
    .fetch_one(pool)
    .await
    .unwrap();
    sqlx::query("INSERT INTO tool_permission_rules (user_id, tool_name, decision) VALUES ($1, 'echo', 'allow')")
        .bind(user_id)
        .execute(pool)
        .await
        .unwrap();
    (session_id, user_id, agent_session_id)
}

fn text(text: &str) -> LlmResponse {
    LlmResponse { content: vec![ContentBlock::Text { text: text.to_string() }], stop_reason: StopReason::EndTurn, input_tokens: 1, output_tokens: 1 }
}

fn call(name: &str, input: serde_json::Value) -> LlmResponse {
    LlmResponse {
        content: vec![ContentBlock::ToolUse { id: "t1".to_string(), name: name.to_string(), input, thought_signature: None }],
        stop_reason: StopReason::ToolUse,
        input_tokens: 1,
        output_tokens: 1,
    }
}

async fn run(pool: &PgPool, provider: &FakeLlmProvider) -> LoopOutcome {
    run_as::<false>(pool, provider).await
}

async fn run_as<const CUSTOM: bool>(pool: &PgPool, provider: &FakeLlmProvider) -> LoopOutcome {
    let (session_id, user_id, agent_session_id) = seed(pool).await;
    let mut conn = pool.acquire().await.unwrap();
    let embedding = FakeEmbeddingProvider::success(vec![0.0; 1536]);
    let registry = AgentRegistry::new(vec![Box::new(TestAgent::<CUSTOM>)]);
    run_agent_turn(&mut conn, None, None, provider, &embedding, &registry, &TestAgent::<CUSTOM>, session_id, agent_session_id, user_id, vec![], 100)
        .await
        .unwrap()
}

fn offered(provider: &FakeLlmProvider) -> Vec<String> {
    provider.received_requests.lock().unwrap()[0].tools.iter().map(|t| t.name.clone()).collect()
}

/// The tool result the agent got back for its one call.
fn tool_result(provider: &FakeLlmProvider) -> (String, bool) {
    let requests = provider.received_requests.lock().unwrap();
    requests[1]
        .messages
        .iter()
        .flat_map(|m| m.content.iter())
        .find_map(|b| match b {
            ContentBlock::ToolResult { content, is_error, .. } => Some((content.clone(), *is_error)),
            _ => None,
        })
        .expect("a tool result")
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_switched_off_tool_is_not_offered_and_a_call_to_it_is_refused(pool: PgPool) {
    sqlx::query("INSERT INTO tool_settings (name, enabled) VALUES ('echo', false), ('complete_task', false)").execute(&pool).await.unwrap();
    let provider = FakeLlmProvider::sequence(vec![call("echo", serde_json::json!({})), text("ok")]);
    run_as::<true>(&pool, &provider).await;

    let tools = offered(&provider);
    assert!(!tools.contains(&"echo".to_string()));
    assert!(tools.contains(&"complete_task".to_string()), "required tools stay on");
    let (content, is_error) = tool_result(&provider);
    assert!(is_error);
    assert!(content.contains("switched off"), "{content}");
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_built_in_agent_keeps_its_own_tools_whatever_the_switches(pool: PgPool) {
    sqlx::query("INSERT INTO tool_settings (name, enabled) VALUES ('echo', false), ('show_table', false)").execute(&pool).await.unwrap();
    let provider = FakeLlmProvider::sequence(vec![call("echo", serde_json::json!({})), text("ok")]);
    run(&pool, &provider).await;

    let tools = offered(&provider);
    assert!(tools.contains(&"echo".to_string()));
    assert!(!tools.contains(&"show_table".to_string()), "crew tools still follow their switch");
    let (content, is_error) = tool_result(&provider);
    assert!(!is_error, "{content}");
}

#[sqlx::test(migrations = "../../migrations")]
async fn web_search_waits_for_a_key_and_read_web_page_is_on_by_default(pool: PgPool) {
    std::env::remove_var("TAVILY_API_KEY");
    let provider = FakeLlmProvider::sequence(vec![text("hi")]);
    run(&pool, &provider).await;
    let tools = offered(&provider);
    assert!(!tools.contains(&"web_search".to_string()));
    assert!(tools.contains(&"read_web_page".to_string()));
}

#[sqlx::test(migrations = "../../migrations")]
async fn read_web_page_refuses_local_addresses(pool: PgPool) {
    let provider = FakeLlmProvider::sequence(vec![call("read_web_page", serde_json::json!({"url": "http://127.0.0.1:9/secret"})), text("ok")]);
    run(&pool, &provider).await;
    let (content, is_error) = tool_result(&provider);
    assert!(is_error, "{content}");
}

/// Answers one HTTP request with `body` and hands back the request it got.
async fn mock_search_server(body: &'static str) -> (String, tokio::task::JoinHandle<String>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let handle = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        let mut buf = vec![0u8; 16 * 1024];
        let mut request = Vec::new();
        loop {
            let n = socket.read(&mut buf).await.unwrap();
            request.extend_from_slice(&buf[..n]);
            let text = String::from_utf8_lossy(&request);
            if let Some(end) = text.find("\r\n\r\n") {
                let len = text
                    .lines()
                    .find_map(|l| l.to_ascii_lowercase().strip_prefix("content-length:").map(|v| v.trim().parse::<usize>().unwrap_or(0)))
                    .unwrap_or(0);
                if request.len() >= end + 4 + len || n == 0 {
                    break;
                }
            }
        }
        let response = format!("HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}", body.len());
        socket.write_all(response.as_bytes()).await.unwrap();
        String::from_utf8_lossy(&request).to_string()
    });
    (format!("http://{addr}"), handle)
}

#[sqlx::test(migrations = "../../migrations")]
async fn web_search_uses_the_saved_provider_and_key(pool: PgPool) {
    std::env::set_var("SETTINGS_ENCRYPTION_KEY", TEST_SETTINGS_KEY.iter().map(|b| format!("{b:02x}")).collect::<String>());
    let (base_url, server) =
        mock_search_server(r#"{"results":[{"title":"Rust 2.0 released","url":"https://example.com/rust","content":"Big news."}]}"#).await;
    let mut conn = pool.acquire().await.unwrap();
    let secrets = std::collections::HashMap::from([("tavily".to_string(), "tvly-test-key".to_string())]);
    nomi_agent_core::tools::save_settings(
        &mut conn,
        "web_search",
        &serde_json::json!({"provider": "tavily", "max_results": 3, "base_url": base_url}),
        Some(nomi_agent_core::tools::encrypt_secrets_with(&TEST_SETTINGS_KEY, &secrets)),
    )
    .await
    .unwrap();
    drop(conn);

    let provider = FakeLlmProvider::sequence(vec![call("web_search", serde_json::json!({"query": "rust news"})), text("Rust 2.0 is out.")]);
    let outcome = run(&pool, &provider).await;

    assert!(offered(&provider).contains(&"web_search".to_string()));
    let (content, is_error) = tool_result(&provider);
    assert!(!is_error, "{content}");
    assert!(content.contains("Rust 2.0 released") && content.contains("https://example.com/rust"), "{content}");
    let request = server.await.unwrap();
    assert!(request.starts_with("POST /search"), "{request}");
    assert!(request.contains("tvly-test-key"));
    assert!(request.contains("\"max_results\":3"), "{request}");
    assert!(matches!(outcome, LoopOutcome::Reply { .. }));
}
