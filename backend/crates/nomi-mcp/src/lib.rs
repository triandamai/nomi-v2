//! A small client for remote MCP servers (Model Context Protocol, streamable HTTP): list a
//! server's tools and call one. Only what Nomi's crew needs; a failure is always an `McpError`
//! the caller can word for the agent, never a panic.

use std::time::Duration;

use serde_json::{json, Value};

/// The protocol version Nomi asks for; the server answers with the one it speaks.
pub const PROTOCOL_VERSION: &str = "2025-06-18";
/// The most of a server's answer that is read, in bytes.
pub const MAX_RESPONSE_BYTES: usize = 1024 * 1024;
/// The most of a tool's answer handed to an agent, in characters.
pub const MAX_RESULT_CHARS: usize = 20_000;
/// The most pages of a server's tool list that are followed.
pub const MAX_TOOL_PAGES: usize = 5;
/// The most tools read from one server.
pub const MAX_TOOLS: usize = 100;
/// How long connecting to a server may take, in seconds.
const CONNECT_TIMEOUT_SECS: u64 = 10;
/// How long one request to a server may take, in seconds.
const REQUEST_TIMEOUT_SECS: u64 = 30;
/// Each request gets its own HTTP response, so one id serves them all.
const REQUEST_ID: u64 = 1;
/// The first HTTP status that isn't a success.
const FIRST_ERROR_STATUS: u16 = 400;

const SESSION_HEADER: &str = "mcp-session-id";
const VERSION_HEADER: &str = "mcp-protocol-version";
const EVENT_STREAM: &str = "text/event-stream";
const CUT_SHORT: &str = "\n(cut short)";

/// One tool a server offers.
#[derive(Debug, Clone, PartialEq)]
pub struct McpTool {
    pub name: String,
    pub description: Option<String>,
    pub input_schema: Value,
}

/// Why a server couldn't be used. `Display` is worded for an agent to pass on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum McpError {
    /// No answer: not there, too slow, or the connection broke.
    Unreachable(String),
    /// An HTTP error status.
    Refused(u16),
    /// An answer that isn't what the protocol says, or a protocol-level error.
    Protocol(String),
    /// The tool ran and reported an error.
    Tool(String),
    /// More than `MAX_RESPONSE_BYTES`.
    TooLarge,
}

impl std::fmt::Display for McpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            McpError::Unreachable(why) => write!(f, "the server couldn't be reached ({why})"),
            McpError::Refused(status) => write!(f, "the server refused the request (HTTP {status}); it may need a sign-in Nomi can't do yet"),
            McpError::Protocol(why) => write!(f, "the server's answer couldn't be used: {why}"),
            McpError::Tool(why) => write!(f, "the tool reported an error: {why}"),
            McpError::TooLarge => write!(f, "the server's answer was too large"),
        }
    }
}

impl std::error::Error for McpError {}

/// An HTTP client for MCP servers: bounded in time, and never following a redirect (a server
/// must not be able to send a request somewhere else).
pub fn http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(CONNECT_TIMEOUT_SECS))
        .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS))
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("failed to build the MCP HTTP client")
}

/// What `initialize` settled: the session to quote (when the server keeps one) and the version.
struct Session {
    id: Option<String>,
    version: String,
}

pub struct McpClient<'a> {
    http: &'a reqwest::Client,
    url: String,
}

impl<'a> McpClient<'a> {
    pub fn new(http: &'a reqwest::Client, url: &str) -> Self {
        Self { http, url: url.to_string() }
    }

    /// The server's tools (at most `MAX_TOOLS`, over at most `MAX_TOOL_PAGES` pages).
    pub async fn list_tools(&self) -> Result<Vec<McpTool>, McpError> {
        let session = self.initialize().await?;
        let mut tools = Vec::new();
        let mut cursor: Option<String> = None;
        for _ in 0..MAX_TOOL_PAGES {
            let params = cursor.as_ref().map(|c| json!({"cursor": c})).unwrap_or_else(|| json!({}));
            let result = self.request(&session, "tools/list", params).await?;
            let page = result.get("tools").and_then(Value::as_array).ok_or_else(|| McpError::Protocol("no tool list".to_string()))?;
            tools.extend(page.iter().filter_map(tool_from));
            cursor = result.get("nextCursor").and_then(Value::as_str).filter(|c| !c.is_empty()).map(str::to_string);
            if cursor.is_none() || tools.len() >= MAX_TOOLS {
                break;
            }
        }
        tools.truncate(MAX_TOOLS);
        Ok(tools)
    }

    /// Calls `name` with `arguments` and gives its answer as text (see `result_text`).
    pub async fn call_tool(&self, name: &str, arguments: &Value) -> Result<String, McpError> {
        let session = self.initialize().await?;
        let result = self.request(&session, "tools/call", json!({"name": name, "arguments": arguments})).await?;
        let text = result_text(&result);
        if result.get("isError").and_then(Value::as_bool).unwrap_or(false) {
            return Err(McpError::Tool(text));
        }
        Ok(text)
    }

    async fn initialize(&self) -> Result<Session, McpError> {
        let params = json!({
            "protocolVersion": PROTOCOL_VERSION,
            "capabilities": {},
            "clientInfo": {"name": "nomi", "version": env!("CARGO_PKG_VERSION")},
        });
        let (result, session_id) = self.send(None, "initialize", params).await?;
        let version = result.get("protocolVersion").and_then(Value::as_str).unwrap_or(PROTOCOL_VERSION).to_string();
        let session = Session { id: session_id, version };
        // The server may answer this note with anything; only a failure to send it matters.
        let note = json!({"jsonrpc": "2.0", "method": "notifications/initialized"});
        self.post(Some(&session), &note).await?;
        Ok(session)
    }

    async fn request(&self, session: &Session, method: &str, params: Value) -> Result<Value, McpError> {
        self.send(Some(session), method, params).await.map(|(result, _)| result)
    }

    /// One request: its result, and the session id the server set (on `initialize`).
    async fn send(&self, session: Option<&Session>, method: &str, params: Value) -> Result<(Value, Option<String>), McpError> {
        let body = json!({"jsonrpc": "2.0", "id": REQUEST_ID, "method": method, "params": params});
        let response = self.post(session, &body).await?;
        let session_id = response.headers().get(SESSION_HEADER).and_then(|v| v.to_str().ok()).map(str::to_string);
        let stream = response.headers().get(reqwest::header::CONTENT_TYPE).and_then(|v| v.to_str().ok()).is_some_and(|t| t.starts_with(EVENT_STREAM));
        let message = read_message(response, stream).await?;
        if let Some(error) = message.get("error") {
            let why = error.get("message").and_then(Value::as_str).unwrap_or("the request failed");
            return Err(McpError::Protocol(why.to_string()));
        }
        let result = message.get("result").cloned().ok_or_else(|| McpError::Protocol("no result".to_string()))?;
        Ok((result, session_id))
    }

    async fn post(&self, session: Option<&Session>, body: &Value) -> Result<reqwest::Response, McpError> {
        let mut request = self.http.post(&self.url).header(reqwest::header::ACCEPT, format!("application/json, {EVENT_STREAM}")).json(body);
        if let Some(session) = session {
            request = request.header(VERSION_HEADER, &session.version);
            if let Some(id) = &session.id {
                request = request.header(SESSION_HEADER, id);
            }
        }
        let response = request.send().await.map_err(|e| McpError::Unreachable(unreachable_reason(&e)))?;
        let status = response.status().as_u16();
        if status >= FIRST_ERROR_STATUS {
            return Err(McpError::Refused(status));
        }
        Ok(response)
    }
}

fn unreachable_reason(error: &reqwest::Error) -> String {
    if error.is_timeout() {
        "it took too long to answer".to_string()
    } else if error.is_connect() {
        "no connection".to_string()
    } else {
        "the connection broke".to_string()
    }
}

/// Reads the JSON-RPC answer out of a response: the body itself, or (for an event stream, which
/// a server may leave open) the first answer among its events.
async fn read_message(mut response: reqwest::Response, stream: bool) -> Result<Value, McpError> {
    let mut body: Vec<u8> = Vec::new();
    loop {
        let chunk = response.chunk().await.map_err(|e| McpError::Unreachable(unreachable_reason(&e)))?;
        let Some(chunk) = chunk else { break };
        if body.len() + chunk.len() > MAX_RESPONSE_BYTES {
            return Err(McpError::TooLarge);
        }
        body.extend_from_slice(&chunk);
        if stream {
            if let Some(message) = answer_in_events(&String::from_utf8_lossy(&body)) {
                return Ok(message);
            }
        }
    }
    let text = String::from_utf8_lossy(&body);
    if stream {
        return answer_in_events(&text).ok_or_else(|| McpError::Protocol("the stream ended without an answer".to_string()));
    }
    let value: Value = serde_json::from_str(text.trim()).map_err(|_| McpError::Protocol("not JSON".to_string()))?;
    answer_among(std::slice::from_ref(&value))
        .or_else(|| value.as_array().and_then(|batch| answer_among(batch)))
        .ok_or_else(|| McpError::Protocol("no answer to the request".to_string()))
}

/// Whether `message` answers a request (a result or an error), not a note or a request of the
/// server's own.
fn is_answer(message: &Value) -> bool {
    message.get("method").is_none() && (message.get("result").is_some() || message.get("error").is_some())
}

fn answer_among(messages: &[Value]) -> Option<Value> {
    messages.iter().find(|m| is_answer(m)).cloned()
}

/// The first answer among the complete events of an event stream (`data:` lines, events
/// separated by a blank line).
pub fn answer_in_events(stream: &str) -> Option<Value> {
    let normalized = stream.replace("\r\n", "\n");
    // Only events that ended (with a blank line) are complete.
    let complete = normalized.rsplit_once("\n\n").map(|(done, _)| done)?;
    complete.split("\n\n").find_map(|event| {
        let data: Vec<&str> = event.lines().filter_map(|line| line.strip_prefix("data:")).map(|d| d.strip_prefix(' ').unwrap_or(d)).collect();
        let message: Value = serde_json::from_str(&data.join("\n")).ok()?;
        is_answer(&message).then_some(message)
    })
}

fn tool_from(tool: &Value) -> Option<McpTool> {
    let name = tool.get("name").and_then(Value::as_str).filter(|n| !n.is_empty())?;
    Some(McpTool {
        name: name.to_string(),
        description: tool.get("description").and_then(Value::as_str).map(str::to_string),
        input_schema: tool.get("inputSchema").filter(|s| s.is_object()).cloned().unwrap_or_else(|| json!({"type": "object"})),
    })
}

/// A tool result as text: its text parts in order, anything else named by its kind (`[image]`),
/// cut to `MAX_RESULT_CHARS`.
pub fn result_text(result: &Value) -> String {
    let parts: Vec<String> = result
        .get("content")
        .and_then(Value::as_array)
        .map(|content| {
            content
                .iter()
                .map(|part| match (part.get("type").and_then(Value::as_str), part.get("text").and_then(Value::as_str)) {
                    (Some("text"), Some(text)) => text.to_string(),
                    (Some(kind), _) => format!("[{kind}]"),
                    (None, _) => "[content]".to_string(),
                })
                .collect()
        })
        .unwrap_or_default();
    let text = parts.join("\n");
    if text.chars().count() <= MAX_RESULT_CHARS {
        return text;
    }
    let cut: String = text.chars().take(MAX_RESULT_CHARS).collect();
    format!("{cut}{CUT_SHORT}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_answer_is_found_only_in_events_that_ended() {
        let answer = "data: {\"jsonrpc\":\"2.0\",\"id\":1,\"result\":{\"ok\":true}}";
        assert_eq!(answer_in_events(answer), None, "the event hasn't ended yet");
        assert_eq!(answer_in_events(&format!("{answer}\r\n\r\n")).unwrap()["result"]["ok"], true);
        let note_only = "data: {\"jsonrpc\":\"2.0\",\"method\":\"notifications/message\"}\n\n";
        assert_eq!(answer_in_events(note_only), None);
        assert_eq!(answer_in_events(": keep-alive\n\ndata: not json\n\n"), None);
    }

    #[test]
    fn data_split_over_lines_is_one_message() {
        let split = "event: message\ndata: {\"jsonrpc\":\"2.0\",\ndata: \"id\":1,\"error\":{\"message\":\"no\"}}\n\n";
        assert_eq!(answer_in_events(split).unwrap()["error"]["message"], "no");
    }

    #[test]
    fn a_result_without_content_is_empty_text() {
        assert_eq!(result_text(&json!({})), "");
        assert_eq!(result_text(&json!({"content": [{"text": "untyped"}]})), "[content]");
    }
}
