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

struct StubAgent {
    agent_type: &'static str,
    intent_label: &'static str,
    is_default: bool,
}

#[async_trait]
impl SubAgent for StubAgent {
    fn agent_type(&self) -> &'static str {
        self.agent_type
    }
    fn system_prompt(&self) -> &'static str {
        "stub"
    }
    fn tools(&self) -> Vec<ToolDefinition> {
        vec![]
    }
    async fn execute_tool(
        &self,
        _: &mut PoolConnection<Postgres>,
        _: Uuid,
        _: Uuid,
        _: Uuid,
        _: &str,
        _: Value,
    ) -> Result<String, String> {
        Err("no tools".to_string())
    }
    fn intent_label(&self) -> &'static str {
        self.intent_label
    }
    fn intent_description(&self) -> &'static str {
        "a stub agent"
    }
    fn is_default(&self) -> bool {
        self.is_default
    }
}

fn stub(agent_type: &'static str, intent_label: &'static str, is_default: bool) -> Box<dyn SubAgent> {
    Box::new(StubAgent { agent_type, intent_label, is_default })
}

#[test]
fn finds_a_registered_agent_by_agent_type() {
    let registry = AgentRegistry::new(vec![stub("money", "money", false), stub("chitchat", "chitchat", true)]);
    assert_eq!(registry.find("money").unwrap().agent_type(), "money");
    assert!(registry.find("nonexistent").is_none());
}

#[test]
fn finds_the_default_agent() {
    let registry = AgentRegistry::new(vec![stub("money", "money", false), stub("chitchat", "chitchat", true)]);
    assert_eq!(registry.default_agent().agent_type(), "chitchat");
}

#[test]
fn finds_an_agent_by_intent_label_case_insensitively() {
    let registry = AgentRegistry::new(vec![stub("money", "money", false), stub("chitchat", "chitchat", true)]);
    assert_eq!(registry.find_by_intent_label("Money").unwrap().agent_type(), "money");
    assert_eq!(registry.find_by_intent_label("  money  ").unwrap().agent_type(), "money");
}

#[test]
fn classification_prompt_excludes_the_default_agent_as_a_target_but_names_it_as_the_fallback() {
    let registry = AgentRegistry::new(vec![stub("money", "money", false), stub("chitchat", "chitchat", true)]);
    let prompt = registry.classification_prompt();
    assert!(prompt.contains("money"));
    assert!(prompt.contains("chitchat")); // named as the fallback, not filtered out of the text entirely
}

#[test]
#[should_panic(expected = "no registered agent returns true from is_default")]
fn panics_with_zero_default_agents() {
    AgentRegistry::new(vec![stub("money", "money", false)]);
}

#[test]
#[should_panic(expected = "more than one registered agent returns true from is_default")]
fn panics_with_more_than_one_default_agent() {
    AgentRegistry::new(vec![stub("a", "a", true), stub("b", "b", true)]);
}
