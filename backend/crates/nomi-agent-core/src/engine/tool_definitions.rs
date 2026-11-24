//! The tools the engine itself gives agents (complete_task, delegate_to_agent, ask_user, plans,
//! reminders, the tool finder, making crew members), as the model sees them.

use super::*;

pub(super) fn complete_task_tool_definition() -> ToolDefinition {
    ToolDefinition {
        name: COMPLETE_TASK_TOOL_NAME.to_string(),
        description: "Call this when you are done helping with this task, whether it succeeded or the user wants to stop.".to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "status": {"type": "string", "enum": ["completed", "cancelled"]},
                "summary": {"type": "string"}
            },
            "required": ["status", "summary"]
        }),
    }
}

pub(super) fn delegate_tool_definition(targets: &[String]) -> ToolDefinition {
    ToolDefinition {
        name: DELEGATE_TOOL_NAME.to_string(),
        description: format!(
            "Hand the user's request to a specialist. The specialist sees this conversation, takes \
             it over right away and answers the user itself, so don't write a reply of your own \
             when you hand off. Use this only when the request genuinely needs a specialist; \
             answer anything else yourself. Available specialists: {}.",
            targets.join(", "),
        ),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "target_agent": {"type": "string", "enum": targets, "description": "Which specialist to delegate to"},
                "task": {"type": "string", "description": "What the user wants from the specialist, with any details from this conversation it needs"}
            },
            "required": ["target_agent", "task"]
        }),
    }
}

pub(super) fn rename_chat_tool_definition() -> ToolDefinition {
    ToolDefinition {
        name: RENAME_CHAT_TOOL_NAME.to_string(),
        description: "Rename this chat. Use it when the person asks you to name or rename the chat; pick a short title (a few words) in their language if they don't give one.".to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {"title": {"type": "string", "description": "The chat's new name, a few words."}},
            "required": ["title"]
        }),
    }
}

pub(super) fn update_todos_tool_definition() -> ToolDefinition {
    ToolDefinition {
        name: UPDATE_TODOS_TOOL_NAME.to_string(),
        description: "Set or replace your current multi-step task checklist, shown live to the user. Call this again with the full updated list whenever a step's status changes.".to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "items": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "id": {"type": "string"},
                            "text": {"type": "string"},
                            "status": {"type": "string", "enum": ["pending", "in_progress", "done"]}
                        },
                        "required": ["id", "text", "status"]
                    }
                }
            },
            "required": ["items"]
        }),
    }
}

pub(super) fn ask_user_tool_definition() -> ToolDefinition {
    ToolDefinition {
        name: ASK_USER_TOOL_NAME.to_string(),
        description: "Ask the person something you need to know before you act, as a card in the chat with \
                      answers they can tap (and room for their own). Use it whenever the request is missing \
                      a detail you can't sensibly assume (when, how much, which one, one-time or repeating), \
                      instead of guessing. Put everything you need to know into this one call, the most \
                      important question first: the card shows the questions one at a time with Next, and \
                      the person sends all the answers together. Never ask again in a later turn for \
                      something you could have asked here; only a question that depends on these answers \
                      waits for them. Each question has 2-5 short options (add a one-line detail where a \
                      label alone isn't clear, e.g. what a design direction looks like). Skip what they \
                      already told you, and never ask the same thing twice. Your turn ends here; their \
                      answers arrive as their next message. Call it on its own, not alongside other tools, \
                      and don't repeat the questions in your reply.".to_string()
            + crate::prompts::ASK_USER_RECOMMEND_GUIDE,
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "intro": {"type": "string", "description": "One short line before the questions, optional"},
                "questions": {
                    "type": "array",
                    "description": "Every question you need answered now, the most important first",
                    "minItems": 1,
                    "maxItems": MAX_QUESTIONS_PER_CARD,
                    "items": {
                        "type": "object",
                        "properties": {
                            "question": {"type": "string"},
                            "helper": {"type": "string", "description": crate::prompts::ASK_USER_HELPER_DESCRIPTION},
                            "options": {
                                "type": "array",
                                "maxItems": 5,
                                "items": {
                                    "type": "object",
                                    "properties": {
                                        "label": {"type": "string"},
                                        "detail": {"type": "string"},
                                        "recommended": {"type": "boolean", "description": crate::prompts::ASK_USER_RECOMMENDED_DESCRIPTION}
                                    },
                                    "required": ["label"]
                                }
                            },
                            "multiple": {"type": "boolean", "description": "true when several options can be picked"}
                        },
                        "required": ["question", "options"]
                    }
                }
            },
            "required": ["questions"]
        }),
    }
}

pub(super) fn read_plan_tool_definition() -> ToolDefinition {
    ToolDefinition {
        name: READ_PLAN_TOOL_NAME.to_string(),
        description: "Read the latest plan drafted in this chat: what was agreed with the person, including \
                      their answers on design, pages and data. Read it before you start building and \
                      whenever you're unsure what was asked for.".to_string(),
        input_schema: serde_json::json!({"type": "object", "properties": {}}),
    }
}

/// The `ask_user` input as a questions block, and the same questions as plain text (the message's
/// fallback, for channels without cards and for the conversation the agents read back).
pub(super) fn parse_questions(input: &serde_json::Value) -> Result<(crate::content_block::ContentBlock, String), String> {
    let block = crate::content_block::ContentBlock::Questions {
        intro: input.get("intro").and_then(|v| v.as_str()).map(str::trim).filter(|s| !s.is_empty()).map(str::to_string),
        questions: serde_json::from_value(input.get("questions").cloned().unwrap_or_default())
            .map_err(|e| format!("questions must be a list of {{question, options}}: {e}"))?,
    };
    let crate::content_block::ContentBlock::Questions { intro, questions } = &block else { unreachable!() };
    match questions.len() {
        0 => return Err("ask a question".to_string()),
        n if n > MAX_QUESTIONS_PER_CARD => return Err(format!("ask at most {MAX_QUESTIONS_PER_CARD} questions in one card: the ones that matter most")),
        _ => {}
    }
    let mut text = intro.clone().map(|i| format!("{i}\n\n")).unwrap_or_default();
    let numbered = questions.len() > 1;
    for (index, q) in questions.iter().enumerate() {
        if q.question.trim().is_empty() {
            return Err("the question needs its text".to_string());
        }
        if numbered {
            text.push_str(&format!("{}. ", index + 1));
        }
        text.push_str(&format!("{}\n", q.question.trim()));
        for option in &q.options {
            match &option.detail {
                Some(detail) => text.push_str(&format!("   - {}: {}\n", option.label, detail)),
                None => text.push_str(&format!("   - {}\n", option.label)),
            }
        }
    }
    Ok((block, text.trim_end().to_string()))
}

pub(super) fn write_plan_tool_definition() -> ToolDefinition {
    ToolDefinition {
        name: WRITE_PLAN_TOOL_NAME.to_string(),
        description: "Write a new version of your plan for this task, before or during a \
                       multi-step build. Call this again whenever the plan changes significantly \
                       — each call is a new, separately viewable version, not an edit to the last \
                       one.".to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "title": {"type": "string", "description": "Short label for this plan version"},
                "content": {"type": "string", "description": "The plan, in markdown"}
            },
            "required": ["title", "content"]
        }),
    }
}

pub fn find_tools_tool_definition() -> ToolDefinition {
    ToolDefinition {
        name: FIND_TOOLS_TOOL_NAME.to_string(),
        description: "Search every tool the crew can be given, by what a task needs (e.g. \"track my spending\", \
                       \"send an email\"). Returns the best matches with what each does, whether you have it \
                       (granted), and which crew members have it (carried_by). It only informs: you can still call \
                       only the tools you were given."
            .to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "query": {"type": "string", "description": "A few words about the task or the tool you're looking for"},
                "limit": {"type": "integer", "description": format!("How many tools at most (default {}, up to {})", crate::tool_finder::FIND_TOOLS_DEFAULT_LIMIT, crate::tool_finder::FIND_TOOLS_MAX_LIMIT)}
            },
            "required": ["query"]
        }),
    }
}

pub fn create_agent_tool_definition() -> ToolDefinition {
    ToolDefinition {
        name: CREATE_AGENT_TOOL_NAME.to_string(),
        description: "Add a new member to the person's own crew, for a task they want an agent for. Call find_tools \
                       first and pass the names of the tools that fit. The member is theirs alone; their plan may \
                       limit how many they can have."
            .to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "name": {"type": "string", "description": format!("A short name, unlike any crew member's (at most {} characters)", crate::personal_crew::NAME_MAX_CHARS)},
                "role": {"type": "string", "description": format!("Its job in one sentence (at most {} characters)", crate::personal_crew::ROLE_MAX_CHARS)},
                "soul": {"type": "string", "description": "Optional soul.md: who it is and how it works, in markdown"},
                "tools": {"type": "array", "items": {"type": "string"}, "description": "Tool names from find_tools; may be empty"}
            },
            "required": ["name", "role", "tools"]
        }),
    }
}

/// A built-in agent's own tools, which Admin → Tools switches never take away from it.
pub(super) fn always_on_own_tools(agent: &dyn SubAgent) -> std::collections::HashSet<String> {
    if agent.own_tools_switchable() {
        Default::default()
    } else {
        agent.tools().into_iter().map(|t| t.name).collect()
    }
}

/// Every tool the engine itself gives agents (as opposed to an agent's own tools), for the
/// list in Admin → Tools. Some only go to agents that use them (plans, records, reminders).
pub fn crew_tool_definitions(registry: &AgentRegistry) -> Vec<ToolDefinition> {
    let mut tools = vec![
        complete_task_tool_definition(),
        delegate_tool_definition(&registry.delegatable_agent_types("")),
        crate::views::show_tool_definition(),
        rename_chat_tool_definition(),
        update_todos_tool_definition(),
        write_plan_tool_definition(),
        crate::crew_talk::ask_agent_tool_definition(&registry.reminder_target_agent_types()),
        crate::crew_talk::note_tool_definition(),
    ];
    // The catalog's generic tools (records, background tasks, reminders, the web, voice, files),
    // each as it is offered in a conversation that could use all of it.
    let targets = registry.reminder_target_agent_types();
    let offer = nomi_tool::OfferContext { reminder_targets: &targets, mentions_files: true, skills: &[] };
    if let Some(catalog) = registry.tool_catalog() {
        let generic = catalog.in_registration_order().filter(|t| t.auto_offer() != nomi_tool::AutoOffer::GrantOnly);
        tools.extend(generic.filter_map(|t| t.offer(&offer)));
    }
    tools.push(find_tools_tool_definition());
    tools.push(create_agent_tool_definition());
    tools
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ask_user_accepts_a_recommended_option_and_a_helper_line() {
        let schema = ask_user_tool_definition().input_schema.to_string();
        assert!(schema.contains("\"recommended\"") && schema.contains("\"helper\""), "{schema}");
        let (block, _) = parse_questions(&serde_json::json!({
            "questions": [{"question": "Which days?", "helper": "From Jakarta.", "options": [{"label": "1-2 Nov", "recommended": true}]}]
        }))
        .unwrap();
        let crate::content_block::ContentBlock::Questions { questions, .. } = block else { panic!() };
        assert_eq!(questions[0].helper.as_deref(), Some("From Jakarta."));
        assert!(questions[0].options[0].recommended);
    }
}
