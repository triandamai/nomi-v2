//! Sora, the designer: turns the plan Rena agreed with the person into visual design.

use std::borrow::Cow;

use async_trait::async_trait;
use serde_json::Value;
use sqlx::pool::PoolConnection;
use sqlx::Postgres;
use uuid::Uuid;

use nomi_agent_core::crew::DESIGN_AGENT_TYPE;
use nomi_agent_core::prompts::DESIGN_SYSTEM_PROMPT;
use nomi_agent_core::SubAgent;
use nomi_llm::ToolDefinition;

/// Sora. She is registered for everyone, and reachable only where the `design` setting is
/// rolled out (see `nomi_agent_core::personal_crew::with_design_gate`).
pub struct DesignAgent;

#[async_trait]
impl SubAgent for DesignAgent {
    fn agent_type(&self) -> Cow<'static, str> {
        Cow::Borrowed(DESIGN_AGENT_TYPE)
    }

    fn system_prompt(&self) -> Cow<'static, str> {
        Cow::Borrowed(DESIGN_SYSTEM_PROMPT)
    }

    /// None of her own yet: she works with the engine's tools (read_plan, ask_user, the web).
    fn tools(&self) -> Vec<ToolDefinition> {
        Vec::new()
    }

    async fn execute_tool(
        &self,
        _conn: &mut PoolConnection<Postgres>,
        _session_id: Uuid,
        _agent_session_id: Uuid,
        _user_id: Uuid,
        name: &str,
        _input: Value,
    ) -> Result<nomi_agent_core::ToolOutcome, String> {
        Err(format!("unknown tool: {name}"))
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
        let sora = DesignAgent;
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
        let sora = DesignAgent;
        assert!(sora.reads_plans(), "she designs what was agreed");
        assert!(sora.asks_questions());
        assert!(sora.can_delegate(), "an approved design goes to Koda");
        assert!(sora.is_delegation_target(), "Rena hands design work to her");
        assert!(sora.uses_memory() && sora.uses_personality() && sora.supports_todos());
        assert!(!sora.supports_plans(), "the plan stays Rena's");
    }

    #[test]
    fn sora_has_no_tools_of_her_own_yet() {
        assert!(DesignAgent.tools().is_empty());
    }

    #[test]
    fn the_classifier_is_told_design_is_about_the_look_not_the_build() {
        let description = DesignAgent.intent_description();
        assert!(description.contains("design") && description.contains("look"));
        assert!(!description.contains("built"), "building stays Rena's and Koda's");
    }
}
