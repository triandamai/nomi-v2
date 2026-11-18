use std::sync::Arc;

use crate::subagent::SubAgent;
use crate::tool_catalog::ToolCatalog;

pub struct AgentRegistry {
    agents: Vec<Arc<dyn SubAgent>>,
    default_index: usize,
    /// The catalog of tools a member can be given, for `find_tools` and `create_agent`. Without
    /// one (tests, mostly) neither is offered.
    tool_catalog: Option<Arc<ToolCatalog>>,
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
            [index] => Self { agents, default_index: *index, tool_catalog: None },
            [] => panic!("AgentRegistry::new: no registered agent returns true from is_default()"),
            _ => panic!(
                "AgentRegistry::new: more than one registered agent returns true from is_default(): {:?}",
                default_indices.iter().map(|&i| agents[i].agent_type().into_owned()).collect::<Vec<_>>()
            ),
        }
    }

    /// The same registry, knowing the tool catalog (see `tool_catalog`).
    pub fn with_tool_catalog(mut self, catalog: Arc<ToolCatalog>) -> Self {
        self.tool_catalog = Some(catalog);
        self
    }

    pub fn tool_catalog(&self) -> Option<&Arc<ToolCatalog>> {
        self.tool_catalog.as_ref()
    }

    /// Every built-in agent, the default one included.
    pub fn agents(&self) -> &[Arc<dyn SubAgent>] {
        &self.agents
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
        self.classification_prompt_excluding(&Default::default(), extra_labels, extra_options)
    }

    /// `classification_prompt_with_extra` without the built-in agents in `excluded` (by
    /// agent_type): the members a person switched off are never routed to.
    pub fn classification_prompt_excluding(
        &self,
        excluded: &std::collections::HashSet<String>,
        extra_labels: &[String],
        extra_options: &[String],
    ) -> String {
        let default_agent = self.default_agent();
        let routable = |a: &&Arc<dyn SubAgent>| !a.is_default() && !excluded.contains(a.agent_type().as_ref());

        let mut labels: Vec<String> = self.agents.iter().filter(routable).map(|a| a.intent_label().into_owned()).collect();
        let mut options: Vec<String> = self
            .agents
            .iter()
            .filter(routable)
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
    /// The members `agent` may hand work to: `delegatable_agent_types` without itself, cut to its
    /// `delegates_only_to` when it names some.
    pub fn delegation_targets_of(&self, agent: &dyn SubAgent) -> Vec<String> {
        let mut targets = self.delegatable_agent_types(agent.agent_type().as_ref());
        targets.retain(|target| agent.may_delegate_to(target));
        targets
    }

    pub fn delegatable_agent_types(&self, excluding: &str) -> Vec<String> {
        self.agents
            .iter()
            .filter(|a| !a.is_default() && a.is_delegation_target() && a.agent_type().as_ref() != excluding)
            .map(|a| a.agent_type().into_owned())
            .collect()
    }

    /// agent_type() of every delegation-eligible agent, INCLUDING the default agent
    /// (chitchat) and the calling agent itself — the valid `target_agent` list for a
    /// `create_reminder` tool call. Deliberately does not apply `delegatable_agent_types`'s
    /// `!is_default()`/`excluding` exclusions: those exist because "delegate to yourself" or
    /// "delegate to the always-active default agent" are nonsensical for an *immediate*
    /// specialist hand-off, but a reminder is a deferred task fired later, often by the very
    /// same agent that created it — self-targeting chitchat is this feature's primary use case
    /// ("remind me to take a bath", fired later by chitchat itself), not an edge case to filter
    /// out. Still excludes non-`is_delegation_target()` agents (the supervisor), since those
    /// exist to report on other agents' work, not to be handed a task of their own.
    pub fn reminder_target_agent_types(&self) -> Vec<String> {
        self.agents
            .iter()
            .filter(|a| a.is_delegation_target())
            .map(|a| a.agent_type().into_owned())
            .collect()
    }
}
