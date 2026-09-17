use std::sync::Arc;

use crate::subagent::SubAgent;

pub struct AgentRegistry {
    agents: Vec<Arc<dyn SubAgent>>,
    default_index: usize,
}

impl AgentRegistry {
    /// Panics if zero or more than one agent returns `true` from `is_default()` — a
    /// misconfigured registry is a startup-time programmer error, not something to degrade
    /// gracefully from at runtime. Still takes owned `Box`es at the call site (every existing
    /// `AgentRegistry::new(vec![Box::new(Agent), ...])` call keeps compiling unchanged) —
    /// converted to `Arc` once here, so every built-in agent is still constructed exactly once.
    pub fn new(agents: Vec<Box<dyn SubAgent>>) -> Self {
        let agents: Vec<Arc<dyn SubAgent>> = agents.into_iter().map(Arc::from).collect();

        let default_indices: Vec<usize> = agents
            .iter()
            .enumerate()
            .filter(|(_, a)| a.is_default())
            .map(|(i, _)| i)
            .collect();

        match default_indices.as_slice() {
            [index] => Self { agents, default_index: *index },
            [] => panic!("AgentRegistry::new: no registered agent returns true from is_default()"),
            _ => panic!(
                "AgentRegistry::new: more than one registered agent returns true from is_default(): {:?}",
                default_indices.iter().map(|&i| agents[i].agent_type().into_owned()).collect::<Vec<_>>()
            ),
        }
    }

    pub fn default_agent(&self) -> Arc<dyn SubAgent> {
        self.agents[self.default_index].clone()
    }

    pub fn find(&self, agent_type: &str) -> Option<Arc<dyn SubAgent>> {
        self.agents.iter().find(|a| a.agent_type().as_ref() == agent_type).cloned()
    }

    /// Finds the agent whose `intent_label()` matches, ignoring case/whitespace (matching
    /// how the classifier's raw LLM text response is normalized before lookup). Despite the
    /// classifier prompt asking for exactly one word, some models don't reliably comply —
    /// a trailing period, a quoted label, or a whole sentence ("This is about planning.")
    /// would otherwise fail an exact-string match and silently misroute to the default agent
    /// with no visible error. Falling back to a whole-word search inside the response is far
    /// cheaper than that failure mode: exact match is tried first (the common, well-behaved
    /// case), then each whitespace/punctuation-separated token is checked for an exact match
    /// against a registered label.
    pub fn find_by_intent_label(&self, label: &str) -> Option<Arc<dyn SubAgent>> {
        let normalized = label.trim().to_lowercase();
        if let Some(agent) = self.agents.iter().find(|a| a.intent_label().as_ref() == normalized) {
            return Some(agent.clone());
        }
        self.agents
            .iter()
            .find(|a| normalized.split(|c: char| !c.is_alphanumeric()).any(|word| word == a.intent_label().as_ref()))
            .cloned()
    }

    /// Builds the classifier's prompt from every non-default registered agent's
    /// intent_label/intent_description. The default agent is the fallback, not a
    /// classification target (matching today's behavior, where chitchat only wins via
    /// explicit classification OR fallback, never by having its own enum variant picked
    /// exclusively for it).
    pub fn classification_prompt(&self) -> String {
        self.classification_prompt_with_extra(&[], &[])
    }

    /// Same as `classification_prompt`, with `extra_labels`/`extra_options` (same "label:
    /// description" shape, one entry per active dynamic agent) merged in after the built-in
    /// agents — used by `nomi_turn::routing::classify_intent` so dynamic agents are routable
    /// through the identical classifier prompt, without this registry needing to know anything
    /// about the `dynamic_agents` table itself.
    pub fn classification_prompt_with_extra(&self, extra_labels: &[String], extra_options: &[String]) -> String {
        let default_agent = self.default_agent();

        let mut labels: Vec<String> =
            self.agents.iter().filter(|a| !a.is_default()).map(|a| a.intent_label().into_owned()).collect();
        let mut options: Vec<String> = self
            .agents
            .iter()
            .filter(|a| !a.is_default())
            .map(|a| format!("{}: {}", a.intent_label(), a.intent_description()))
            .collect();
        labels.extend(extra_labels.iter().cloned());
        options.extend(extra_options.iter().cloned());

        crate::prompts::INTENT_CLASSIFICATION_PROMPT_TEMPLATE
            .replace("{labels}", &labels.join(", "))
            .replace("{default_label}", default_agent.intent_label().as_ref())
            .replace("{options}", &options.join("\n"))
    }

    /// agent_type() of every non-default, delegation-eligible agent other than `excluding` —
    /// the valid target list for a delegate_to_agent tool call from that agent. Only ever
    /// built-in agents: dynamic agents are never delegation targets in v1 (see design spec's
    /// Out of Scope), so this never needs to consult the `dynamic_agents` table.
    pub fn delegatable_agent_types(&self, excluding: &str) -> Vec<String> {
        self.agents
            .iter()
            .filter(|a| !a.is_default() && a.is_delegation_target() && a.agent_type().as_ref() != excluding)
            .map(|a| a.agent_type().into_owned())
            .collect()
    }
}
