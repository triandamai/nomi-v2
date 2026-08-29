use async_trait::async_trait;
use serde_json::Value;
use sqlx::pool::PoolConnection;
use sqlx::Postgres;
use uuid::Uuid;

use nomi_agent_core::{AgentRegistry, SubAgent};
use nomi_llm::ToolDefinition;

struct DefaultAgent;

#[async_trait]
impl SubAgent for DefaultAgent {
    fn agent_type(&self) -> &'static str {
        "default"
    }
    fn system_prompt(&self) -> &'static str {
        "default"
    }
    fn tools(&self) -> Vec<ToolDefinition> {
        vec![]
    }
    async fn execute_tool(&self, _: &mut PoolConnection<Postgres>, _: Uuid, _: Uuid, _: Uuid, _: &str, _: Value) -> Result<String, String> {
        Err("no tools".to_string())
    }
    fn intent_label(&self) -> &'static str {
        "default"
    }
    fn intent_description(&self) -> &'static str {
        "default"
    }
    fn is_default(&self) -> bool {
        true
    }
    fn can_delegate(&self) -> bool {
        true
    }
}

struct SpecialistAgent;

#[async_trait]
impl SubAgent for SpecialistAgent {
    fn agent_type(&self) -> &'static str {
        "specialist"
    }
    fn system_prompt(&self) -> &'static str {
        "specialist"
    }
    fn tools(&self) -> Vec<ToolDefinition> {
        vec![]
    }
    async fn execute_tool(&self, _: &mut PoolConnection<Postgres>, _: Uuid, _: Uuid, _: Uuid, _: &str, _: Value) -> Result<String, String> {
        Err("no tools".to_string())
    }
    fn intent_label(&self) -> &'static str {
        "specialist"
    }
    fn intent_description(&self) -> &'static str {
        "specialist"
    }
}

struct NonTargetableAgent;

#[async_trait]
impl SubAgent for NonTargetableAgent {
    fn agent_type(&self) -> &'static str {
        "non-targetable"
    }
    fn system_prompt(&self) -> &'static str {
        "non-targetable"
    }
    fn tools(&self) -> Vec<ToolDefinition> {
        vec![]
    }
    async fn execute_tool(&self, _: &mut PoolConnection<Postgres>, _: Uuid, _: Uuid, _: Uuid, _: &str, _: Value) -> Result<String, String> {
        Err("no tools".to_string())
    }
    fn intent_label(&self) -> &'static str {
        "non-targetable"
    }
    fn intent_description(&self) -> &'static str {
        "non-targetable"
    }
    fn is_delegation_target(&self) -> bool {
        false
    }
}

#[test]
fn delegatable_agent_types_excludes_default_non_targetable_and_the_requester() {
    let registry = AgentRegistry::new(vec![
        Box::new(DefaultAgent),
        Box::new(SpecialistAgent),
        Box::new(NonTargetableAgent),
    ]);

    let targets = registry.delegatable_agent_types("default");

    assert_eq!(targets, vec!["specialist"]);
}

#[test]
fn delegatable_agent_types_excludes_the_named_requester_even_if_delegation_eligible() {
    let registry = AgentRegistry::new(vec![Box::new(DefaultAgent), Box::new(SpecialistAgent)]);

    let targets = registry.delegatable_agent_types("specialist");

    assert!(targets.is_empty());
}
