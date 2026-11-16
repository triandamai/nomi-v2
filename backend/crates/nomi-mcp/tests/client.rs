//! The MCP client against a server that answers as the protocol says: JSON or an event stream,
//! with or without a session, and the ways a server fails.

use nomi_mcp::{McpClient, McpError, MAX_RESULT_CHARS};
use serde_json::json;
use wiremock::matchers::{body_partial_json, header, method};
use wiremock::{Mock, MockServer, ResponseTemplate};

const SESSION: &str = "session-1";

fn rpc(result: serde_json::Value) -> ResponseTemplate {
    ResponseTemplate::new(200).set_body_json(json!({"jsonrpc": "2.0", "id": 1, "result": result}))
}

/// A server that starts a session and accepts the client's "initialized" note.
async fn server() -> MockServer {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(body_partial_json(json!({"method": "initialize"})))
        .respond_with(
            rpc(json!({"protocolVersion": "2025-03-26", "capabilities": {}, "serverInfo": {"name": "refs", "version": "1"}}))
                .insert_header("mcp-session-id", SESSION),
        )
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(body_partial_json(json!({"method": "notifications/initialized"})))
        .respond_with(ResponseTemplate::new(202))
        .mount(&server)
        .await;
    server
}

#[tokio::test]
async fn lists_a_servers_tools_inside_the_session_it_was_given() {
    let server = server().await;
    Mock::given(method("POST"))
        .and(body_partial_json(json!({"method": "tools/list"})))
        .and(header("mcp-session-id", SESSION))
        .and(header("mcp-protocol-version", "2025-03-26"))
        .respond_with(rpc(json!({"tools": [
            {"name": "search_references", "description": "Find visual references.", "inputSchema": {"type": "object", "properties": {"query": {"type": "string"}}}},
            {"name": "bare"}
        ]})))
        .expect(1)
        .mount(&server)
        .await;

    let http = nomi_mcp::http_client();
    let tools = McpClient::new(&http, &server.uri()).list_tools().await.unwrap();
    assert_eq!(tools.len(), 2);
    assert_eq!((tools[0].name.as_str(), tools[0].description.as_deref()), ("search_references", Some("Find visual references.")));
    assert_eq!(tools[0].input_schema["properties"]["query"]["type"], "string");
    assert_eq!((tools[1].description.as_deref(), &tools[1].input_schema), (None, &json!({"type": "object"})));
}

#[tokio::test]
async fn follows_the_cursor_to_the_rest_of_the_tools() {
    let server = server().await;
    Mock::given(method("POST"))
        .and(body_partial_json(json!({"method": "tools/list", "params": {"cursor": "page-2"}})))
        .respond_with(rpc(json!({"tools": [{"name": "second"}]})))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(body_partial_json(json!({"method": "tools/list"})))
        .respond_with(rpc(json!({"tools": [{"name": "first"}], "nextCursor": "page-2"})))
        .mount(&server)
        .await;

    let http = nomi_mcp::http_client();
    let tools = McpClient::new(&http, &server.uri()).list_tools().await.unwrap();
    assert_eq!(tools.iter().map(|t| t.name.as_str()).collect::<Vec<_>>(), ["first", "second"]);
}

#[tokio::test]
async fn calls_a_tool_and_reads_its_answer_from_an_event_stream() {
    let server = server().await;
    let stream = "event: message\ndata: {\"jsonrpc\":\"2.0\",\"method\":\"notifications/progress\",\"params\":{}}\n\n\
                  event: message\ndata: {\"jsonrpc\":\"2.0\",\"id\":1,\"result\":{\"content\":[{\"type\":\"text\",\"text\":\"Three references\"},{\"type\":\"image\",\"data\":\"AAAA\",\"mimeType\":\"image/png\"},{\"type\":\"text\",\"text\":\"found.\"}]}}\n\n";
    Mock::given(method("POST"))
        .and(body_partial_json(json!({"method": "tools/call", "params": {"name": "search_references", "arguments": {"query": "bakery"}}})))
        .respond_with(ResponseTemplate::new(200).set_body_raw(stream, "text/event-stream"))
        .mount(&server)
        .await;

    let http = nomi_mcp::http_client();
    let answer = McpClient::new(&http, &server.uri()).call_tool("search_references", &json!({"query": "bakery"})).await.unwrap();
    assert_eq!(answer, "Three references\n[image]\nfound.");
}

#[tokio::test]
async fn a_long_answer_is_cut_to_what_an_agent_can_take() {
    let server = server().await;
    Mock::given(method("POST"))
        .and(body_partial_json(json!({"method": "tools/call"})))
        .respond_with(rpc(json!({"content": [{"type": "text", "text": "x".repeat(MAX_RESULT_CHARS + 500)}]})))
        .mount(&server)
        .await;
    let http = nomi_mcp::http_client();
    let answer = McpClient::new(&http, &server.uri()).call_tool("big", &json!({})).await.unwrap();
    assert!(answer.chars().count() < MAX_RESULT_CHARS + 100 && answer.ends_with("(cut short)"));
}

#[tokio::test]
async fn a_tool_that_reports_an_error_is_an_error_with_its_words() {
    let server = server().await;
    Mock::given(method("POST"))
        .and(body_partial_json(json!({"method": "tools/call"})))
        .respond_with(rpc(json!({"isError": true, "content": [{"type": "text", "text": "No such board."}]})))
        .mount(&server)
        .await;
    let http = nomi_mcp::http_client();
    let failed = McpClient::new(&http, &server.uri()).call_tool("open_board", &json!({})).await.unwrap_err();
    assert_eq!(failed, McpError::Tool("No such board.".to_string()));
}

#[tokio::test]
async fn a_protocol_error_carries_the_servers_message() {
    let server = server().await;
    Mock::given(method("POST"))
        .and(body_partial_json(json!({"method": "tools/call"})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"jsonrpc": "2.0", "id": 1, "error": {"code": -32602, "message": "Unknown tool: nope"}})))
        .mount(&server)
        .await;
    let http = nomi_mcp::http_client();
    let failed = McpClient::new(&http, &server.uri()).call_tool("nope", &json!({})).await.unwrap_err();
    assert_eq!(failed, McpError::Protocol("Unknown tool: nope".to_string()));
}

#[tokio::test]
async fn a_server_that_wants_a_sign_in_is_refused_not_broken() {
    let server = MockServer::start().await;
    Mock::given(method("POST")).respond_with(ResponseTemplate::new(401)).mount(&server).await;
    let http = nomi_mcp::http_client();
    assert_eq!(McpClient::new(&http, &server.uri()).list_tools().await.unwrap_err(), McpError::Refused(401));
}

#[tokio::test]
async fn a_server_that_talks_nonsense_is_a_protocol_error() {
    let server = MockServer::start().await;
    Mock::given(method("POST")).respond_with(ResponseTemplate::new(200).set_body_string("<html>hello</html>")).mount(&server).await;
    let http = nomi_mcp::http_client();
    assert!(matches!(McpClient::new(&http, &server.uri()).list_tools().await.unwrap_err(), McpError::Protocol(_)));
}

#[tokio::test]
async fn a_server_that_is_not_there_is_unreachable() {
    // A port nothing listens on: bound once to learn a free one, then closed.
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let gone = format!("http://{}", listener.local_addr().unwrap());
    drop(listener);
    let http = nomi_mcp::http_client();
    assert!(matches!(McpClient::new(&http, &gone).list_tools().await.unwrap_err(), McpError::Unreachable(_)));
}

#[test]
fn every_error_says_what_happened_in_words_an_agent_can_pass_on() {
    assert_eq!(McpError::Refused(401).to_string(), "the server refused the request (HTTP 401); it may need a sign-in Nomi can't do yet");
    assert_eq!(McpError::Tool("No such board.".into()).to_string(), "the tool reported an error: No such board.");
    assert!(McpError::Unreachable("timed out".into()).to_string().starts_with("the server couldn't be reached"));
    assert!(McpError::Protocol("bad".into()).to_string().starts_with("the server's answer couldn't be used"));
    assert_eq!(McpError::TooLarge.to_string(), "the server's answer was too large");
}
