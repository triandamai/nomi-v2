//! The person's own MCP servers, as an agent uses them: what tools they offer and calling one.
//! The server implements this (`nomi_server::mcp_ops`); agents and tests only see the trait, so
//! the provider can be swapped or absent. A server that fails never fails the turn: it is named
//! as unavailable and the agent carries on without it.

use std::sync::Arc;

use async_trait::async_trait;
use serde_json::Value;
use uuid::Uuid;

/// What an agent is told when nothing can be listed.
pub const NOTHING_CONNECTED: &str = "No connected tools: the person hasn't added an MCP server (they can under Connections). Carry on with what you have.";
/// What an agent is told after a connected tool or server failed.
pub const CARRY_ON: &str = "Carry on without it, and mention it only if it matters to the person.";
/// The most of a tool's description shown in a listing, in characters.
pub const MAX_DESCRIPTION_CHARS: usize = 300;
/// The most of a tool's input schema shown in a listing, in characters.
pub const MAX_SCHEMA_CHARS: usize = 600;

/// One tool on one of the person's servers.
#[derive(Debug, Clone, PartialEq)]
pub struct McpToolInfo {
    /// The name the person gave the server.
    pub server: String,
    pub tool: String,
    pub description: Option<String>,
    pub input_schema: Value,
}

/// The tools on the person's servers, and the servers that couldn't be used (name, why).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct McpListing {
    pub tools: Vec<McpToolInfo>,
    pub unavailable: Vec<(String, String)>,
}

#[async_trait]
pub trait McpOps: Send + Sync {
    /// Every tool on `user_id`'s servers. Never fails: a server that can't be used is listed
    /// under `unavailable`.
    async fn tools(&self, user_id: Uuid) -> McpListing;

    /// Calls `tool` on the person's server named `server` and gives its answer as text, or why
    /// it couldn't (worded for the agent).
    async fn call(&self, user_id: Uuid, server: &str, tool: &str, arguments: Value) -> Result<String, String>;
}

pub type SharedMcpOps = Arc<dyn McpOps>;

/// No provider (tests, or a Nomi without one): nothing is connected.
pub struct NoMcpOps;

#[async_trait]
impl McpOps for NoMcpOps {
    async fn tools(&self, _user_id: Uuid) -> McpListing {
        McpListing::default()
    }

    async fn call(&self, _user_id: Uuid, _server: &str, _tool: &str, _arguments: Value) -> Result<String, String> {
        Err("Connected tools aren't available on this Nomi.".to_string())
    }
}

fn clip(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        return text.to_string();
    }
    let cut: String = text.chars().take(max).collect();
    format!("{cut}…")
}

/// A listing as an agent reads it: each tool with its server, what it does and its inputs, then
/// the servers that couldn't be used.
pub fn format_listing(listing: &McpListing) -> String {
    if listing.tools.is_empty() && listing.unavailable.is_empty() {
        return NOTHING_CONNECTED.to_string();
    }
    let mut lines: Vec<String> = listing
        .tools
        .iter()
        .map(|t| {
            let description = t.description.as_deref().map(|d| format!(": {}", clip(d.trim(), MAX_DESCRIPTION_CHARS))).unwrap_or_default();
            format!("- server \"{}\", tool \"{}\"{description}\n  inputs: {}", t.server, t.tool, clip(&t.input_schema.to_string(), MAX_SCHEMA_CHARS))
        })
        .collect();
    if !listing.unavailable.is_empty() {
        lines.extend(listing.unavailable.iter().map(|(server, why)| format!("- \"{server}\" couldn't be used: {why}")));
        lines.push(CARRY_ON.to_string());
    }
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn tool(server: &str, name: &str, description: Option<&str>) -> McpToolInfo {
        McpToolInfo { server: server.to_string(), tool: name.to_string(), description: description.map(str::to_string), input_schema: json!({"type": "object", "properties": {"query": {"type": "string"}}}) }
    }

    #[test]
    fn a_listing_names_each_tool_by_its_server_with_its_inputs() {
        let listing = McpListing { tools: vec![tool("Refs", "search_references", Some("Find visual references.")), tool("Refs", "bare", None)], unavailable: vec![] };
        let text = format_listing(&listing);
        assert!(text.contains("server \"Refs\", tool \"search_references\": Find visual references."), "{text}");
        assert!(text.contains("\"query\""), "the input schema is there to call it with");
        assert!(text.contains("server \"Refs\", tool \"bare\""));
        assert!(!text.contains("couldn't be used"));
    }

    #[test]
    fn servers_that_failed_are_named_and_the_agent_is_told_to_carry_on() {
        let listing = McpListing { tools: vec![tool("Refs", "search_references", None)], unavailable: vec![("Moodboard".to_string(), "the server couldn't be reached (no connection)".to_string())] };
        let text = format_listing(&listing);
        assert!(text.contains("\"Moodboard\" couldn't be used: the server couldn't be reached (no connection)"), "{text}");
        assert!(text.contains(CARRY_ON));
    }

    #[test]
    fn nothing_connected_says_so_and_how_to_connect() {
        let text = format_listing(&McpListing::default());
        assert_eq!(text, NOTHING_CONNECTED);
        assert!(text.contains("Connections"));
    }

    #[test]
    fn a_long_description_or_schema_is_cut() {
        let long = "d".repeat(MAX_DESCRIPTION_CHARS + 50);
        let listing = McpListing { tools: vec![tool("Refs", "wordy", Some(&long))], unavailable: vec![] };
        assert!(!format_listing(&listing).contains(&long));
    }

    #[tokio::test]
    async fn without_a_provider_nothing_is_listed_and_a_call_says_why() {
        let none = NoMcpOps;
        assert_eq!(none.tools(uuid::Uuid::nil()).await, McpListing::default());
        let refused = none.call(uuid::Uuid::nil(), "Refs", "search", json!({})).await.unwrap_err();
        assert!(refused.contains("aren't available"));
    }
}
