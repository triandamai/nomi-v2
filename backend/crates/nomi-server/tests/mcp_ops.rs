//! The person's MCP servers as the crew reaches them: their own servers only, public addresses
//! only, and one failing server never hides the rest.

use nomi_server::mcp_ops::ServerMcpOps;
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;
use wiremock::matchers::{body_partial_json, method};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// `NOMI_WEB_ALLOW_PRIVATE` is process-wide, so the tests that set or clear it take turns.
static ENV: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());
const ALLOW_PRIVATE: &str = "NOMI_WEB_ALLOW_PRIVATE";

fn rpc(result: serde_json::Value) -> ResponseTemplate {
    ResponseTemplate::new(200).set_body_json(json!({"jsonrpc": "2.0", "id": 1, "result": result}))
}

/// An MCP server with one tool, `search_references`, that answers "Three references".
async fn references_server() -> MockServer {
    let server = MockServer::start().await;
    for (rpc_method, result) in [
        ("initialize", json!({"protocolVersion": "2025-06-18", "capabilities": {}, "serverInfo": {"name": "refs", "version": "1"}})),
        ("tools/list", json!({"tools": [{"name": "search_references", "description": "Find visual references."}]})),
        ("tools/call", json!({"content": [{"type": "text", "text": "Three references"}]})),
    ] {
        Mock::given(method("POST")).and(body_partial_json(json!({"method": rpc_method}))).respond_with(rpc(result)).mount(&server).await;
    }
    Mock::given(method("POST")).and(body_partial_json(json!({"method": "notifications/initialized"}))).respond_with(ResponseTemplate::new(202)).mount(&server).await;
    server
}

async fn person(pool: &PgPool) -> Uuid {
    sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id").fetch_one(pool).await.unwrap()
}

async fn connect(pool: &PgPool, user_id: Uuid, name: &str, url: &str) {
    sqlx::query("INSERT INTO mcp_servers (user_id, name, url) VALUES ($1, $2, $3)").bind(user_id).bind(name).bind(url).execute(pool).await.unwrap();
}

#[sqlx::test(migrations = "../../migrations")]
async fn lists_and_calls_the_persons_own_servers_and_names_the_one_that_failed(pool: PgPool) {
    let _env = ENV.lock().await;
    std::env::set_var(ALLOW_PRIVATE, "1");
    let refs = references_server().await;
    let broken = MockServer::start().await;
    Mock::given(method("POST")).respond_with(ResponseTemplate::new(401)).mount(&broken).await;
    let ada = person(&pool).await;
    let bob = person(&pool).await;
    connect(&pool, ada, "Refs", &refs.uri()).await;
    connect(&pool, ada, "Moodboard", &broken.uri()).await;
    let mcp = ServerMcpOps::shared(pool.clone());

    let listing = mcp.tools(ada).await;
    assert_eq!(listing.tools.iter().map(|t| (t.server.as_str(), t.tool.as_str())).collect::<Vec<_>>(), [("Refs", "search_references")]);
    assert_eq!(listing.unavailable.len(), 1);
    assert_eq!(listing.unavailable[0].0, "Moodboard");
    assert!(listing.unavailable[0].1.contains("HTTP 401"));

    assert_eq!(mcp.call(ada, "refs", "search_references", json!({"query": "bakery"})).await.as_deref(), Ok("Three references"));
    assert!(mcp.call(ada, "Moodboard", "open", json!({})).await.unwrap_err().contains("HTTP 401"));
    assert_eq!(mcp.call(ada, "Nope", "x", json!({})).await.unwrap_err(), "the person has no connected server named \"Nope\"");

    // Someone else's servers are not theirs to use.
    assert!(mcp.tools(bob).await.tools.is_empty());
    assert!(mcp.call(bob, "Refs", "search_references", json!({})).await.unwrap_err().contains("no connected server"));
    std::env::remove_var(ALLOW_PRIVATE);
}

#[sqlx::test(migrations = "../../migrations")]
async fn a_server_on_a_private_address_is_never_called(pool: PgPool) {
    let _env = ENV.lock().await;
    std::env::remove_var(ALLOW_PRIVATE);
    let local = references_server().await;
    let ada = person(&pool).await;
    connect(&pool, ada, "Local", &local.uri()).await;
    let mcp = ServerMcpOps::shared(pool.clone());

    let listing = mcp.tools(ada).await;
    assert!(listing.tools.is_empty());
    assert_eq!(listing.unavailable, [("Local".to_string(), "its address is private or couldn't be found".to_string())]);
    assert_eq!(mcp.call(ada, "Local", "search_references", json!({})).await.unwrap_err(), "its address is private or couldn't be found");
    assert!(local.received_requests().await.unwrap_or_default().is_empty(), "nothing was sent to it");
}
