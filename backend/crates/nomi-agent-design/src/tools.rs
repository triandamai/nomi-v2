//! Sora's own tools: the tools on the MCP servers the person connected (visual references,
//! moodboards, whatever they have), through `McpOps`.

use async_trait::async_trait;
use serde_json::{json, Value};
use sqlx::pool::PoolConnection;
use sqlx::Postgres;
use uuid::Uuid;

use nomi_agent_core::mcp_ops::{format_listing, McpOps, SharedMcpOps, CARRY_ON};
use nomi_agent_core::design_brief;
use nomi_agent_core::design_directions::{self, DesignDirections};
use nomi_agent_core::{ContentBlock, Tool, ToolOutcome, View};
use nomi_llm::ToolDefinition;

pub const PROPOSE_DIRECTIONS: &str = "propose_directions";
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

/// What Sora is told after a set was saved and shown.
pub fn proposed_message(version: i32, brief_version: Option<i32>, set: &DesignDirections) -> String {
    let from = brief_version.map(|v| format!(", from design brief version {v}")).unwrap_or_default();
    let names: Vec<String> = set.directions.iter().map(|d| format!("{} ({})", d.name, d.id)).collect();
    format!(
        "Directions saved as version {version}{from}: {}. The person now sees them as a choice with each one's reason and trade-offs: \
         say in a sentence or two how they differ and which you'd pick, then wait for their answer.",
        names.join(", ")
    )
}

/// `propose_directions`: checks the set, saves it as the chat's next version and shows the
/// person the choice.
pub struct ProposeDirections;

#[async_trait]
impl Tool for ProposeDirections {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: PROPOSE_DIRECTIONS.to_string(),
            description: "Propose two to four clearly different design directions for the person to choose from, each with what \
                          it looks like, why it fits the brief, its trade-offs, colours, fonts and layout. It saves them as a \
                          new version and shows the person the choice. Call it again with the whole set (same ids for the \
                          directions you keep) when they ask for changes."
                .to_string(),
            input_schema: design_directions::directions_input_schema(),
        }
    }

    fn group(&self) -> &'static str {
        GROUP
    }

    fn keywords(&self) -> &'static [&'static str] {
        &["design", "directions", "look"]
    }

    async fn execute(&self, conn: &mut PoolConnection<Postgres>, session_id: Uuid, agent_session_id: Uuid, user_id: Uuid, input: Value) -> Result<ToolOutcome, String> {
        let set = DesignDirections::parse(&input)?;
        let brief_version = design_brief::latest(&mut **conn, session_id).await.ok().flatten().map(|(version, _)| version);
        let version = design_directions::save(&mut **conn, session_id, agent_session_id, user_id, brief_version, &set).await.map_err(|e| {
            tracing::error!(error = %e, "design directions: save failed");
            "the directions couldn't be saved right now".to_string()
        })?;
        Ok(ToolOutcome { display_text: proposed_message(version, brief_version, &set), block: Some(ContentBlock::View(View::Decision(set.to_decision()))) })
    }
}

/// Sora's own tools.
pub fn own_tools(mcp: &SharedMcpOps) -> Vec<Box<dyn Tool>> {
    vec![Box::new(ProposeDirections), Box::new(ListConnectedTools { mcp: mcp.clone() }), Box::new(UseConnectedTool { mcp: mcp.clone() })]
}

#[cfg(test)]
pub(crate) mod tests {
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

    pub(crate) fn two_directions() -> Value {
        let direction = |id: &str, name: &str, accent: &str, font: &str| {
            json!({
                "id": id, "name": name,
                "summary": "What it looks like.", "rationale": "Why it fits.", "trade_offs": ["What it costs"],
                "palette": [{"name": "Paper", "hex": "#F6EFE4"}, {"name": "Accent", "hex": accent}],
                "typography": {"heading": font, "body": "Inter"},
                "layout": "One long page."
            })
        };
        json!({"directions": [direction("editorial", "Editorial", "#B5532A", "Fraunces"), direction("swiss", "Swiss utility", "#1F4FFF", "Helvetica")]})
    }

    #[test]
    fn proposing_tells_sora_the_version_the_brief_it_came_from_and_to_wait() {
        let set = DesignDirections::parse(&two_directions()).unwrap();
        let said = proposed_message(3, Some(2), &set);
        assert!(said.starts_with("Directions saved as version 3, from design brief version 2: Editorial (editorial), Swiss utility (swiss)."), "{said}");
        assert!(said.contains("wait for their answer"));
        assert!(proposed_message(1, None, &set).starts_with("Directions saved as version 1: Editorial"));
    }

    #[test]
    fn the_propose_tool_takes_the_directions_schema() {
        let tool = ProposeDirections.definition();
        assert_eq!(tool.name, PROPOSE_DIRECTIONS);
        assert_eq!(tool.input_schema, nomi_agent_core::design_directions::directions_input_schema());
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
