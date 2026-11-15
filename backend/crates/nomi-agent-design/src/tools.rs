//! Sora's own tools: the tools on the MCP servers the person connected (visual references,
//! moodboards, whatever they have), through `McpOps`.

use async_trait::async_trait;
use serde_json::{json, Value};
use sqlx::pool::PoolConnection;
use sqlx::Postgres;
use uuid::Uuid;

use nomi_agent_core::mcp_ops::{format_listing, McpOps, SharedMcpOps, CARRY_ON};
use nomi_agent_core::{Tool, ToolOutcome};
use nomi_llm::ToolDefinition;

pub const LIST_CONNECTED_TOOLS: &str = "list_connected_tools";
pub const USE_CONNECTED_TOOL: &str = "use_connected_tool";
/// The tools' group in listings.
const GROUP: &str = "design";

/// What is on the person's servers, as Sora reads it.
pub async fn list_connected(mcp: &dyn McpOps, user_id: Uuid) -> String {
    format_listing(&mcp.tools(user_id).await)
}

/// Calls the tool the input names. A failure comes back as words for Sora, never as a failed turn.
pub async fn use_connected(mcp: &dyn McpOps, user_id: Uuid, input: &Value) -> Result<String, String> {
    let text = |field: &str| input.get(field).and_then(Value::as_str).map(str::trim).filter(|v| !v.is_empty());
    let server = text("server").ok_or("server is required")?;
    let tool = text("tool").ok_or("tool is required")?;
    let arguments = match input.get("arguments") {
        None | Some(Value::Null) => json!({}),
        Some(arguments) if arguments.is_object() => arguments.clone(),
        Some(_) => return Err("arguments must be an object".to_string()),
    };
    mcp.call(user_id, server, tool, arguments).await.map_err(|why| format!("{tool} on {server} didn't work: {why}. {CARRY_ON}"))
}

/// `list_connected_tools`.
pub struct ListConnectedTools {
    pub mcp: SharedMcpOps,
}

#[async_trait]
impl Tool for ListConnectedTools {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: LIST_CONNECTED_TOOLS.to_string(),
            description: "List the tools on the MCP servers the person connected (visual references, moodboards, their own design \
                          tools), each with its server and inputs. Call it before use_connected_tool."
                .to_string(),
            input_schema: json!({"type": "object", "properties": {}}),
        }
    }

    fn group(&self) -> &'static str {
        GROUP
    }

    fn keywords(&self) -> &'static [&'static str] {
        &["mcp", "connected", "references"]
    }

    async fn execute(&self, _conn: &mut PoolConnection<Postgres>, _session_id: Uuid, _agent_session_id: Uuid, user_id: Uuid, _input: Value) -> Result<ToolOutcome, String> {
        Ok(ToolOutcome::text(list_connected(self.mcp.as_ref(), user_id).await))
    }
}

/// `use_connected_tool`.
pub struct UseConnectedTool {
    pub mcp: SharedMcpOps,
}

#[async_trait]
impl Tool for UseConnectedTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: USE_CONNECTED_TOOL.to_string(),
            description: "Call one tool from list_connected_tools: its server and tool exactly as listed, and arguments that fit \
                          its inputs. The person approves it first. If it fails, carry on without it."
                .to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "server": {"type": "string", "description": "The server's name, as listed"},
                    "tool": {"type": "string", "description": "The tool's name, as listed"},
                    "arguments": {"type": "object", "description": "The tool's inputs"}
                },
                "required": ["server", "tool"]
            }),
        }
    }

    fn group(&self) -> &'static str {
        GROUP
    }

    fn keywords(&self) -> &'static [&'static str] {
        &["mcp", "connected", "references"]
    }

    async fn execute(&self, _conn: &mut PoolConnection<Postgres>, _session_id: Uuid, _agent_session_id: Uuid, user_id: Uuid, input: Value) -> Result<ToolOutcome, String> {
        use_connected(self.mcp.as_ref(), user_id, &input).await.map(ToolOutcome::text)
    }
}

/// Sora's own tools.
pub fn own_tools(mcp: &SharedMcpOps) -> Vec<Box<dyn Tool>> {
    vec![Box::new(ListConnectedTools { mcp: mcp.clone() }), Box::new(UseConnectedTool { mcp: mcp.clone() })]
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use nomi_agent_core::mcp_ops::{McpListing, McpOps, McpToolInfo, CARRY_ON, NOTHING_CONNECTED};
    use serde_json::json;
    use std::sync::Mutex;

    /// The person's servers as a test wants them, recording what was called.
    #[derive(Default)]
    struct FakeMcp {
        listing: McpListing,
        answer: Option<Result<String, String>>,
        called: Mutex<Vec<(String, String, Value)>>,
    }

    #[async_trait]
    impl McpOps for FakeMcp {
        async fn tools(&self, _: Uuid) -> McpListing {
            self.listing.clone()
        }
        async fn call(&self, _: Uuid, server: &str, tool: &str, arguments: Value) -> Result<String, String> {
            self.called.lock().unwrap().push((server.to_string(), tool.to_string(), arguments));
            self.answer.clone().unwrap_or(Ok("done".to_string()))
        }
    }

    #[tokio::test]
    async fn listing_shows_what_is_connected_or_says_nothing_is() {
        assert_eq!(list_connected(&FakeMcp::default(), Uuid::nil()).await, NOTHING_CONNECTED);
        let refs = McpToolInfo { server: "Refs".into(), tool: "search_references".into(), description: None, input_schema: json!({"type": "object"}) };
        let listed = list_connected(&FakeMcp { listing: McpListing { tools: vec![refs], unavailable: vec![] }, ..Default::default() }, Uuid::nil()).await;
        assert!(listed.contains("server \"Refs\", tool \"search_references\""));
    }

    #[tokio::test]
    async fn using_a_tool_passes_the_server_tool_and_arguments_through() {
        let mcp = FakeMcp { answer: Some(Ok("Three references".into())), ..Default::default() };
        let answer = use_connected(&mcp, Uuid::nil(), &json!({"server": " Refs ", "tool": "search_references", "arguments": {"query": "bakery"}})).await;
        assert_eq!(answer.as_deref(), Ok("Three references"));
        assert_eq!(*mcp.called.lock().unwrap(), [("Refs".to_string(), "search_references".to_string(), json!({"query": "bakery"}))]);
    }

    #[tokio::test]
    async fn arguments_left_out_are_an_empty_object() {
        let mcp = FakeMcp::default();
        use_connected(&mcp, Uuid::nil(), &json!({"server": "Refs", "tool": "list_boards"})).await.unwrap();
        assert_eq!(mcp.called.lock().unwrap()[0].2, json!({}));
    }

    #[tokio::test]
    async fn a_call_without_its_server_or_tool_is_not_made() {
        let mcp = FakeMcp::default();
        assert_eq!(use_connected(&mcp, Uuid::nil(), &json!({"tool": "x"})).await.unwrap_err(), "server is required");
        assert_eq!(use_connected(&mcp, Uuid::nil(), &json!({"server": "Refs", "tool": " "})).await.unwrap_err(), "tool is required");
        assert_eq!(use_connected(&mcp, Uuid::nil(), &json!({"server": "Refs", "tool": "x", "arguments": "nope"})).await.unwrap_err(), "arguments must be an object");
        assert!(mcp.called.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn a_tool_that_fails_tells_sora_why_and_to_carry_on() {
        let mcp = FakeMcp { answer: Some(Err("the server couldn't be reached (no connection)".into())), ..Default::default() };
        let failed = use_connected(&mcp, Uuid::nil(), &json!({"server": "Refs", "tool": "search_references"})).await.unwrap_err();
        assert!(failed.starts_with("search_references on Refs didn't work: the server couldn't be reached (no connection)."), "{failed}");
        assert!(failed.ends_with(CARRY_ON));
    }
}
