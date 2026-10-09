//! The names people see for the built-in crew. Agents are still identified by their
//! `agent_type` ("money", "reminders"…) everywhere in code and data; these are what the chat,
//! the crew page, emails and hand-off messages call them.

/// The name of a built-in crew member, by `agent_type`.
pub fn crew_name(agent_type: &str) -> Option<&'static str> {
    Some(match agent_type {
        "chitchat" => "Nomi",
        "money" => "Dana",
        "reminders" => "Kala",
        "files" => "Maya",
        "planning" => "Rena",
        "coding" => "Koda",
        "workspace" => "Tara",
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_the_crew_and_title_cases_the_rest() {
        assert_eq!(display_name_for("money"), "Dana");
        assert_eq!(display_name_for("chitchat"), "Nomi");
        assert_eq!(display_name_for("supervisor"), "Supervisor");
    }
}
