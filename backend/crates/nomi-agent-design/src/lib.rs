//! Sora, the designer: turns the plan Rena agreed with the person into visual design.

pub mod tools;

use std::borrow::Cow;

use async_trait::async_trait;
use serde_json::Value;
use sqlx::pool::PoolConnection;
use sqlx::Postgres;
use uuid::Uuid;

use nomi_agent_core::crew::DESIGN_AGENT_TYPE;
use nomi_agent_core::prompts::DESIGN_SYSTEM_PROMPT;
use nomi_agent_core::mcp_ops::{NoMcpOps, SharedMcpOps};
use nomi_agent_core::{find_tool, tool_definitions, SubAgent};
use nomi_llm::ToolDefinition;

/// Sora. She is registered for everyone, and reachable only where the `design` setting is
/// rolled out (see `nomi_agent_core::personal_crew::with_design_gate`).
pub struct DesignAgent {
    mcp: SharedMcpOps,
}

impl DesignAgent {
    pub fn new() -> Self {
        Self { mcp: std::sync::Arc::new(NoMcpOps) }
    }

    /// The person's connected MCP servers, through the server's access to them.
    pub fn with_mcp_ops(mut self, mcp: SharedMcpOps) -> Self {
        self.mcp = mcp;
        self
    }
}

impl Default for DesignAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SubAgent for DesignAgent {
    fn agent_type(&self) -> Cow<'static, str> {
        Cow::Borrowed(DESIGN_AGENT_TYPE)
    }

    fn system_prompt(&self) -> Cow<'static, str> {
        Cow::Borrowed(DESIGN_SYSTEM_PROMPT)
    }

    /// The person's connected tools; the rest of what she uses (read_plan, ask_user, the web)
    /// is the engine's.
    fn tools(&self) -> Vec<ToolDefinition> {
        tool_definitions(&tools::own_tools(&self.mcp))
    }

    async fn execute_tool(
        &self,
        conn: &mut PoolConnection<Postgres>,
        session_id: Uuid,
        agent_session_id: Uuid,
        user_id: Uuid,
        name: &str,
        input: Value,
    ) -> Result<nomi_agent_core::ToolOutcome, String> {
        let own = tools::own_tools(&self.mcp);
        match find_tool(&own, name) {
            Some(tool) => tool.execute(conn, session_id, agent_session_id, user_id, input).await,
            None => Err(format!("unknown tool: {name}")),
        }
    }

    /// Looking at what is connected needs no OK; calling a connected tool does.
    fn tool_needs_approval(&self, tool_name: &str) -> bool {
        tool_name != tools::LIST_CONNECTED_TOOLS
    }

    fn describe_action(&self, tool_name: &str, input: &Value, _locale: nomi_agent_core::Locale) -> Option<String> {
        if tool_name != tools::USE_CONNECTED_TOOL {
            return None;
        }
        let text = |field: &str| input.get(field).and_then(Value::as_str).map(str::trim).filter(|v| !v.is_empty());
        Some(format!("Use {} on {}", text("tool")?, text("server")?))
    }

    fn intent_label(&self) -> Cow<'static, str> {
        Cow::Borrowed(DESIGN_AGENT_TYPE)
    }

    fn intent_description(&self) -> Cow<'static, str> {
        Cow::Borrowed(
            "the user wants design work on a site or app: its look and feel, visual directions to choose from, layout, colours, fonts, or feedback on how a design looks",
        )
    }

    /// Remembers durable facts the user mentions here (their brand, what they like).
    fn uses_memory(&self) -> bool {
        true
    }

    fn uses_personality(&self) -> bool {
        true
    }

    /// An approved design goes to Koda.
    fn can_delegate(&self) -> bool {
        true
    }

    fn supports_todos(&self) -> bool {
        true
    }

    /// She designs what Rena and the person agreed.
    fn reads_plans(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nomi_agent_core::crew::DESIGN_AGENT_TYPE;
    use nomi_agent_core::prompts::DESIGN_SYSTEM_PROMPT;
    use nomi_agent_core::SubAgent;

    #[test]
    fn sora_is_the_design_agent_in_the_crew() {
        let sora = DesignAgent::new();
        assert_eq!(sora.agent_type(), DESIGN_AGENT_TYPE);
        assert_eq!(sora.agent_type(), "design");
        assert_eq!(sora.display_name(), "Sora");
        assert_eq!(sora.intent_label(), "design");
        assert_eq!(sora.system_prompt(), DESIGN_SYSTEM_PROMPT);
        assert!(sora.system_prompt().starts_with("You are Sora, the designer in Nomi's crew."));
        assert!(!sora.is_default());
    }

    #[test]
    fn sora_reads_the_plan_asks_and_can_hand_work_on() {
        let sora = DesignAgent::new();
        assert!(sora.reads_plans(), "she designs what was agreed");
        assert!(sora.asks_questions());
        assert!(sora.can_delegate(), "an approved design goes to Koda");
        assert!(sora.is_delegation_target(), "Rena hands design work to her");
        assert!(sora.uses_memory() && sora.uses_personality() && sora.supports_todos());
        assert!(!sora.supports_plans(), "the plan stays Rena's");
    }

    #[test]
    fn soras_own_tools_are_the_persons_connected_tools() {
        let names: Vec<String> = DesignAgent::new().tools().into_iter().map(|t| t.name).collect();
        assert_eq!(names, [tools::LIST_CONNECTED_TOOLS, tools::USE_CONNECTED_TOOL]);
    }

    #[test]
    fn looking_needs_no_ok_but_using_a_connected_tool_does() {
        let sora = DesignAgent::new();
        assert!(!sora.tool_needs_approval(tools::LIST_CONNECTED_TOOLS));
        assert!(sora.tool_needs_approval(tools::USE_CONNECTED_TOOL));
        let action = sora.describe_action(tools::USE_CONNECTED_TOOL, &serde_json::json!({"server": "Refs", "tool": "search_references"}), nomi_agent_core::Locale::En);
        assert_eq!(action.as_deref(), Some("Use search_references on Refs"));
        assert_eq!(sora.describe_action(tools::LIST_CONNECTED_TOOLS, &serde_json::json!({}), nomi_agent_core::Locale::En), None);
    }

    #[test]
    fn sora_is_told_to_carry_on_when_research_fails() {
        let prompt = DesignAgent::new().system_prompt();
        for said in ["web_search", "read_web_page", "list_connected_tools", "use_connected_tool", "carry on"] {
            assert!(prompt.contains(said), "her prompt still says {said:?}");
        }
    }

    #[test]
    fn the_classifier_is_told_design_is_about_the_look_not_the_build() {
        let description = DesignAgent::new().intent_description();
        assert!(description.contains("design") && description.contains("look"));
        assert!(!description.contains("built"), "building stays Rena's and Koda's");
    }
}
