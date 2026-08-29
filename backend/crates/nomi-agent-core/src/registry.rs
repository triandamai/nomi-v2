use crate::subagent::SubAgent;

pub struct AgentRegistry {
    agents: Vec<Box<dyn SubAgent>>,
    default_index: usize,
}

impl AgentRegistry {
    /// Panics if zero or more than one agent returns `true` from `is_default()` — a
    /// misconfigured registry is a startup-time programmer error, not something to degrade
    /// gracefully from at runtime.
    pub fn new(agents: Vec<Box<dyn SubAgent>>) -> Self {
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
                default_indices.iter().map(|&i| agents[i].agent_type()).collect::<Vec<_>>()
            ),
        }
    }

    pub fn default_agent(&self) -> &dyn SubAgent {
        self.agents[self.default_index].as_ref()
    }

    pub fn find(&self, agent_type: &str) -> Option<&dyn SubAgent> {
        self.agents.iter().find(|a| a.agent_type() == agent_type).map(|a| a.as_ref())
    }

    /// Finds the agent whose `intent_label()` matches, ignoring case/whitespace (matching
    /// how the classifier's raw LLM text response is normalized before lookup).
    pub fn find_by_intent_label(&self, label: &str) -> Option<&dyn SubAgent> {
        let normalized = label.trim().to_lowercase();
        self.agents.iter().find(|a| a.intent_label() == normalized).map(|a| a.as_ref())
    }

    /// Builds the classifier's prompt from every non-default registered agent's
    /// intent_label/intent_description. The default agent is the fallback, not a
    /// classification target (matching today's behavior, where chitchat only wins via
    /// explicit classification OR fallback, never by having its own enum variant picked
    /// exclusively for it).
    pub fn classification_prompt(&self) -> String {
        let options: Vec<String> = self
            .agents
            .iter()
            .filter(|a| !a.is_default())
            .map(|a| format!("{}: {}", a.intent_label(), a.intent_description()))
            .collect();

        let labels: Vec<&str> = self
            .agents
            .iter()
            .filter(|a| !a.is_default())
            .map(|a| a.intent_label())
            .collect();

        format!(
            "Classify the user's message as exactly one of: {}, or \"{}\" if none of the specific \
             categories apply. Reply with only that single word, nothing else.\n\nCategories:\n{}",
            labels.join(", "),
            self.default_agent().intent_label(),
            options.join("\n"),
        )
    }

    /// agent_type() of every non-default, delegation-eligible agent other than `excluding` —
    /// the valid target list for a delegate_to_agent tool call from that agent.
    pub fn delegatable_agent_types(&self, excluding: &str) -> Vec<&'static str> {
        self.agents
            .iter()
            .filter(|a| !a.is_default() && a.is_delegation_target() && a.agent_type() != excluding)
            .map(|a| a.agent_type())
            .collect()
    }
}
