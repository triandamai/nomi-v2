//! Koda's built-in coding knowledge: one guide per stack (checked against real installs and
//! builds), plus the database and preview guides. They're compiled in and read on demand with
//! read_guide, so the system prompt stays short.

use serde_json::{json, Value};

use nomi_llm::ToolDefinition;

/// (topic, what it covers, content).
pub const GUIDES: &[(&str, &str, &str)] = &[
    ("sveltekit", "SvelteKit 3 + Svelte 5: full-stack apps (the default)", include_str!("../guides/sveltekit.md")),
    ("database", "Drizzle with Postgres or SQLite, and how the preview runs them", include_str!("../guides/database.md")),
    ("preview", "How Nomi runs the project in the browser, and what can't run there", include_str!("../guides/preview.md")),
    ("react", "React 19 + Vite single-page apps", include_str!("../guides/react.md")),
    ("vue", "Vue 3 + Vite single-page apps", include_str!("../guides/vue.md")),
    ("astro", "Astro 7 content sites with Svelte islands", include_str!("../guides/astro.md")),
];

/// The guide for a project's stack, if it has one.
pub fn guide_for_stack(stack: &str) -> Option<&'static str> {
    match stack {
        "sveltekit" | "svelte" => Some("sveltekit"),
        "react" => Some("react"),
        "vue" => Some("vue"),
        "astro" => Some("astro"),
        _ => None,
    }
}

pub fn read_guide_tool_definition() -> ToolDefinition {
    let topics: Vec<&str> = GUIDES.iter().map(|(topic, _, _)| *topic).collect();
    let listing = GUIDES.iter().map(|(topic, about, _)| format!("{topic}: {about}")).collect::<Vec<_>>().join("; ");
    ToolDefinition {
        name: "read_guide".to_string(),
        description: format!(
            "Read one of your coding guides: exact package versions, config files and conventions that are known to work in Nomi. Read the stack's guide before writing a project's first files, and the database guide before adding a database. Guides: {listing}."
        ),
        input_schema: json!({
            "type": "object",
            "properties": {"topic": {"type": "string", "enum": topics}},
            "required": ["topic"]
        }),
    }
}

pub fn read_guide(input: &Value) -> Result<String, String> {
    let topic = input.get("topic").and_then(|v| v.as_str()).ok_or("topic is required")?.trim().to_lowercase();
    GUIDES
        .iter()
        .find(|(name, _, _)| *name == topic)
        .map(|(_, _, content)| content.to_string())
        .ok_or_else(|| format!("no guide called {topic}; there are: {}", GUIDES.iter().map(|g| g.0).collect::<Vec<_>>().join(", ")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_stack_with_a_guide_points_at_a_real_one() {
        for stack in ["sveltekit", "svelte", "react", "vue", "astro"] {
            let topic = guide_for_stack(stack).unwrap();
            assert!(read_guide(&json!({ "topic": topic })).unwrap().len() > 500, "{stack}");
        }
        assert!(guide_for_stack("static").is_none());
        assert!(read_guide(&json!({ "topic": "cobol" })).is_err());
    }

    #[test]
    fn the_sveltekit_guide_teaches_version_3() {
        let guide = read_guide(&json!({ "topic": "sveltekit" })).unwrap();
        assert!(guide.contains("$app/env/private") && guide.contains("#lib/server/db/index.ts"));
    }
}
