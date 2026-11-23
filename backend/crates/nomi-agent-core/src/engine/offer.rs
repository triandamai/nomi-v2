//! What the engine itself offers an agent on top of the agent's own tools, decided from data the
//! turn loads up front, so it can be checked without a database.

use std::collections::HashSet;

use nomi_llm::ToolDefinition;

use super::*;
use crate::tools::ToolSwitches;

/// What the offer depends on, loaded once per turn.
#[derive(Debug, Default, Clone)]
pub struct OfferInputs {
    /// The crew members this person switched off.
    pub off: HashSet<String>,
    /// Admin → Tools.
    pub switches: ToolSwitches,
    /// Web search has a provider set up.
    pub web_search_ready: bool,
    /// Voice is on and has its key.
    pub voice_ready: bool,
    /// The conversation holds an attached file.
    pub mentions_files: bool,
}

/// The engine's tools for `agent`, in the order they're offered. Admin switches on the agent's
/// own tools are applied by the caller.
pub fn engine_tools_for(agent: &dyn SubAgent, registry: &AgentRegistry, inputs: &OfferInputs) -> Vec<ToolDefinition> {
    let mut tools = Vec::new();
    tools.push(complete_task_tool_definition());
    let off = &inputs.off;
    if agent.can_delegate() {
        let mut targets = registry.delegation_targets_of(agent);
        targets.retain(|t| !off.contains(t));
        if !targets.is_empty() {
            tools.push(delegate_tool_definition(&targets));
        }
    }
    tools.push(crate::views::show_tool_definition());
    tools.push(rename_chat_tool_definition());
    tools.extend(crate::background::background_tool_definitions());
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
    if agent.uses_records() {
        tools.extend(crate::records::record_tool_definitions());
    }
    if agent.supports_reminders() {
        let mut targets = registry.reminder_target_agent_types();
        // Sora takes no reminders while she is off for this person.
        targets.retain(|t| t != crate::crew::DESIGN_AGENT_TYPE || !off.contains(t));
        tools.push(create_reminder_tool_definition(&targets));
        tools.push(list_reminders_tool_definition());
        tools.push(cancel_reminder_tool_definition());
    }
    // Looking through the tool catalog, and (Nomi and the supervisor) making crew members with it.
    if registry.tool_catalog().is_some() {
        tools.push(find_tools_tool_definition());
        if agent.creates_crew_members() {
            tools.push(create_agent_tool_definition());
        }
    }
    let switches = &inputs.switches;
    if switches.allows(crate::web::WEB_SEARCH_TOOL_NAME) && inputs.web_search_ready {
        tools.push(crate::web::web_search_tool_definition());
    }
    if switches.allows(crate::web::READ_WEB_PAGE_TOOL_NAME) {
        tools.push(crate::web::read_web_page_tool_definition());
    }
    // Saying things aloud, while it's on and has its ElevenLabs key.
    if switches.allows(crate::voice::SPEAK_TOOL_NAME) && inputs.voice_ready {
        tools.push(crate::voice::speak_tool_definition(crate::prompts::SPEAK_TOOL_DESCRIPTION));
    }
    // Reading on in a file the user attached, whenever the conversation holds one.
    if inputs.mentions_files {
        tools.push(crate::attachments::read_attachment_tool_definition());
    }
    tools
}

/// Engine tools whose calls never post an activity message, even for an agent that surfaces
/// its activity.
pub fn is_quiet(name: &str) -> bool {
    name == COMPLETE_TASK_TOOL_NAME
        || name == DELEGATE_TOOL_NAME
        || name == UPDATE_TODOS_TOOL_NAME
        || name == WRITE_PLAN_TOOL_NAME
        || name == READ_PLAN_TOOL_NAME
        || name == CREATE_REMINDER_TOOL_NAME
        || name == LIST_REMINDERS_TOOL_NAME
        || name == CANCEL_REMINDER_TOOL_NAME
        || name == FIND_TOOLS_TOOL_NAME
        || crate::background::BACKGROUND_TOOL_NAMES.contains(&name)
        || crate::web::is_web_tool(name)
        || crate::records::RECORD_TOOL_NAMES.contains(&name)
}
