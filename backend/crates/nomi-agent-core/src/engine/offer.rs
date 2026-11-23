//! What the engine itself offers an agent on top of the agent's own tools, decided from data the
//! turn loads up front, so it can be checked without a database.

use std::collections::HashSet;

use nomi_llm::ToolDefinition;

use super::*;
use crate::tool_catalog::ToolCatalog;
use crate::tools::ToolSwitches;
use nomi_tool::{AutoOffer, OfferContext, Tool};

/// What the offer depends on, loaded once per turn.
#[derive(Debug, Default, Clone)]
pub struct OfferInputs {
    /// The crew members this person switched off.
    pub off: HashSet<String>,
    /// Admin → Tools.
    pub switches: ToolSwitches,
    /// Catalog tools whose `ready` check failed this turn (no search provider, no voice key).
    pub not_ready: HashSet<String>,
    /// The conversation holds an attached file.
    pub mentions_files: bool,
}

/// The generic tools the engine gives agents, as catalog tools, in the order they're offered:
/// background tasks, records, the crew's reminders, the web, speaking, attached files.
pub fn engine_catalog_tools() -> Vec<Box<dyn Tool>> {
    let speak = crate::voice::SpeakCopy { description: crate::prompts::SPEAK_TOOL_DESCRIPTION, done: crate::prompts::SPEAK_TOOL_DONE };
    [
        crate::background::tools(),
        crate::records::tools(),
        crate::scheduled_jobs::tools(),
        crate::web::tools(),
        crate::voice::tools(speak),
        nomi_tools_attachments::tools(),
    ]
    .into_iter()
    .flatten()
    .collect()
}

/// Whether `agent` gets `tool` without a grant.
pub fn auto_offered_to(tool: &dyn Tool, agent: &dyn SubAgent) -> bool {
    match tool.auto_offer() {
        AutoOffer::GrantOnly => false,
        AutoOffer::Everyone => true,
        AutoOffer::IfUsesRecords => agent.uses_records(),
        AutoOffer::IfSupportsReminders => agent.supports_reminders(),
    }
}

/// The agents a reminder can be handed to. Sora takes no reminders while she is off for this person.
fn reminder_targets(registry: &AgentRegistry, off: &HashSet<String>) -> Vec<String> {
    let mut targets = registry.reminder_target_agent_types();
    targets.retain(|t| t != crate::crew::DESIGN_AGENT_TYPE || !off.contains(t));
    targets
}

/// The catalog tools `agent` gets automatically this turn, in catalog order, kept to those
/// `keep` picks: switched on, ready, and offered by the tool for this conversation.
fn catalog_offer(agent: &dyn SubAgent, registry: &AgentRegistry, inputs: &OfferInputs, offer: &OfferContext<'_>, keep: impl Fn(&dyn Tool) -> bool) -> Vec<ToolDefinition> {
    let Some(catalog) = registry.tool_catalog() else { return Vec::new() };
    catalog
        .in_registration_order()
        .filter(|t| keep(*t) && auto_offered_to(*t, agent))
        .filter(|t| {
            let name = t.definition().name;
            inputs.switches.allows(&name) && !inputs.not_ready.contains(&name)
        })
        .filter_map(|t| t.offer(offer))
        .collect()
}

/// The engine's tools for `agent`, in the order they're offered. Admin switches on the agent's
/// own tools are applied by the caller.
pub fn engine_tools_for(agent: &dyn SubAgent, registry: &AgentRegistry, inputs: &OfferInputs) -> Vec<ToolDefinition> {
    let off = &inputs.off;
    let targets = reminder_targets(registry, off);
    let offer = OfferContext { reminder_targets: &targets, mentions_files: inputs.mentions_files };
    let background = |t: &dyn Tool| t.group() == crate::background::GROUP;
    let gated = |t: &dyn Tool| matches!(t.auto_offer(), AutoOffer::IfUsesRecords | AutoOffer::IfSupportsReminders);

    let mut tools = Vec::new();
    tools.push(complete_task_tool_definition());
    if agent.can_delegate() {
        let mut targets = registry.delegation_targets_of(agent);
        targets.retain(|t| !off.contains(t));
        if !targets.is_empty() {
            tools.push(delegate_tool_definition(&targets));
        }
    }
    tools.push(crate::views::show_tool_definition());
    tools.push(rename_chat_tool_definition());
    tools.extend(catalog_offer(agent, registry, inputs, &offer, background));
    let mut crew = crate::crew_talk::ask_targets(registry, agent.agent_type().as_ref());
    crew.retain(|t| !off.contains(t));
    if !crew.is_empty() {
        tools.push(crate::crew_talk::ask_agent_tool_definition(&crew));
    }
    tools.push(crate::crew_talk::note_tool_definition());
    if agent.supports_todos() {
        tools.push(update_todos_tool_definition());
    }
    if agent.supports_plans() {
        tools.push(write_plan_tool_definition());
    }
    if agent.asks_questions() {
        tools.push(ask_user_tool_definition());
    }
    if agent.reads_plans() {
        tools.push(read_plan_tool_definition());
    }
    // Records and the crew's reminders, for the agents that use them.
    tools.extend(catalog_offer(agent, registry, inputs, &offer, gated));
    // Looking through the tool catalog, and (Nomi and the supervisor) making crew members with it.
    if registry.tool_catalog().is_some() {
        tools.push(find_tools_tool_definition());
        if agent.creates_crew_members() {
            tools.push(create_agent_tool_definition());
        }
    }
    // The web, speaking aloud, and reading on in an attached file.
    tools.extend(catalog_offer(agent, registry, inputs, &offer, |t| !background(t) && !gated(t)));
    tools
}

/// The loop's own tools whose calls never post an activity message.
const QUIET_LOOP_TOOLS: [&str; 6] =
    [COMPLETE_TASK_TOOL_NAME, DELEGATE_TOOL_NAME, UPDATE_TODOS_TOOL_NAME, WRITE_PLAN_TOOL_NAME, READ_PLAN_TOOL_NAME, FIND_TOOLS_TOOL_NAME];

/// Engine tools whose calls never post an activity message, even for an agent that surfaces
/// its activity: the quiet loop tools, and catalog tools that say they post nothing.
pub fn is_quiet(catalog: Option<&ToolCatalog>, name: &str) -> bool {
    QUIET_LOOP_TOOLS.contains(&name) || catalog.and_then(|c| c.get(name)).is_some_and(|t| !t.posts_activity())
}
