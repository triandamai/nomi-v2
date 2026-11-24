//! The names people see for the built-in crew. Agents are still identified by their
//! `agent_type` ("money", "reminders"…) everywhere in code and data; these are what the chat,
//! the crew page, emails and hand-off messages call them.

/// Sora, the designer (nomi-agent-design).
pub const DESIGN_AGENT_TYPE: &str = "design";
/// Rena, the planner (nomi-agent-planning).
pub const PLANNING_AGENT_TYPE: &str = "planning";
/// Koda, the developer (nomi-agent-coding).
pub const CODING_AGENT_TYPE: &str = "coding";

/// The name of a built-in crew member, by `agent_type`.
pub fn crew_name(agent_type: &str) -> Option<&'static str> {
    Some(match agent_type {
        "chitchat" => "Nomi",
        "money" => "Dana",
        "reminders" => "Kala",
        "files" => "Maya",
        PLANNING_AGENT_TYPE => "Rena",
        CODING_AGENT_TYPE => "Koda",
        "workspace" => "Tara",
        "health" => "Raga",
        "inspector" => "Iris",
        DESIGN_AGENT_TYPE => "Sora",
        _ => return None,
    })
}

/// What to call an agent by its type: its crew name, else its type in Title Case.
pub fn display_name_for(agent_type: &str) -> String {
    if let Some(name) = crew_name(agent_type) {
        return name.to_string();
    }
    let mut chars = agent_type.chars();
    chars.next().map(|first| first.to_uppercase().chain(chars).collect()).unwrap_or_default()
}
